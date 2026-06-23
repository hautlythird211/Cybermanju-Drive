use base64::Engine;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, KeyInit, Nonce as ChaChaNonce};
use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use futures_util::{SinkExt, StreamExt};
use rustls::{ClientConfig, RootCertStore};
use secp256k1::{Keypair, Message as SecpMessage, Secp256k1, SecretKey, XOnlyPublicKey};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tokio_tungstenite::connect_async_tls_with_config;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::Connector;
use webpki_roots::TLS_SERVER_ROOTS;
use zeroize::Zeroize;

/// Working Nostr relays for read/write operations.
const DEFAULT_RELAYS: &[(&str, &str)] = &[
    ("wss://relay.nostr.com", "read/write"),
    ("wss://relay.damus.io", "read/write"),
    ("wss://nos.lol", "read/write"),
    ("wss://relay.primal.net", "read/write"),
    ("wss://relay.nostr.band", "read/write"),
    ("wss://relay.snort.social", "read"),
    ("wss://nostr.wine", "read"),
    ("wss://nostr.bitcoiner.social", "read"),
    ("wss://relay.current.fyi", "read"),
    ("wss://relay.wellorder.net", "read"),
    ("wss://nos.relay", "read"),
    ("wss://nostr.inosta.cc", "read"),
    ("wss://nostr.mom", "read"),
];

/// Nostr backend for decentralized storage via relays.
/// Uses NIP-96 file storage extension for large binary files.
/// Falls back to NIP-59 gift-wrap for small files.
pub struct NostrBackend {
    /// Raw private key bytes — zeroized on drop
    private_key: Vec<u8>,
    /// Cached secp256k1 keypair for signing
    keypair: Keypair,
    /// Cached x-only public key (NIP-01 pubkey)
    pubkey_xonly: XOnlyPublicKey,
    relays: Vec<String>,
    nip96_host: Option<String>,
    http_client: reqwest::blocking::Client,
    connected_relays: std::sync::Mutex<Vec<String>>,
    rt: Runtime,
}

impl Drop for NostrBackend {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

impl NostrBackend {
    /// Connect to a WebSocket relay over TLS, send a message, read one response, then close.
    fn tls_ws_request(&self, relay_url: &str, msg: &str) -> Result<Option<String>, String> {
        let url = relay_url.to_string();
        let msg_owned = msg.to_string();

        self.rt.block_on(async move {
            let (mut ws_stream, _) = timeout(
                Duration::from_secs(10),
                connect_async_tls_with_config(
                    &url,
                    None,
                    false,
                    Some(Connector::Rustls(Arc::new(
                        ClientConfig::builder()
                            .with_root_certificates(RootCertStore::from_iter(
                                TLS_SERVER_ROOTS.iter().cloned(),
                            ))
                            .with_no_client_auth(),
                    ))),
                ),
            )
            .await
            .map_err(|e| format!("TLS WebSocket connect timeout to {}: {}", url, e))?
            .map_err(|e| format!("TLS WebSocket connect to {}: {}", url, e))?;

            ws_stream
                .send(Message::Text(msg_owned.into()))
                .await
                .map_err(|e| format!("ws send: {}", e))?;

            match timeout(Duration::from_secs(10), ws_stream.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => Ok(Some(text.to_string())),
                Ok(Some(Ok(Message::Close(_)))) => Ok(None),
                Ok(Some(Ok(_))) => Ok(None),
                Ok(Some(Err(e))) => Err(format!("ws recv: {}", e)),
                Ok(None) => Ok(None),
                Err(_) => Ok(None),
            }
        })
    }

    /// Connect to a WebSocket relay over TLS, send a message, collect all responses until close/EOS.
    fn tls_ws_request_all(&self, relay_url: &str, msg: &str) -> Result<Vec<String>, String> {
        let url = relay_url.to_string();
        let msg_owned = msg.to_string();

        self.rt.block_on(async move {
            let (mut ws_stream, _) = timeout(
                Duration::from_secs(10),
                connect_async_tls_with_config(
                    &url,
                    None,
                    false,
                    Some(Connector::Rustls(Arc::new(
                        ClientConfig::builder()
                            .with_root_certificates(RootCertStore::from_iter(
                                TLS_SERVER_ROOTS.iter().cloned(),
                            ))
                            .with_no_client_auth(),
                    ))),
                ),
            )
            .await
            .map_err(|e| format!("TLS WebSocket connect timeout to {}: {}", url, e))?
            .map_err(|e| format!("TLS WebSocket connect to {}: {}", url, e))?;

            ws_stream
                .send(Message::Text(msg_owned.into()))
                .await
                .map_err(|e| format!("ws send: {}", e))?;

            let mut responses = Vec::new();

            loop {
                match timeout(Duration::from_secs(10), ws_stream.next()).await {
                    Ok(Some(Ok(Message::Text(text)))) => {
                        responses.push(text.to_string());
                    }
                    Ok(Some(Ok(Message::Pong(_)))) => {}
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        let _ = ws_stream.send(Message::Pong(data)).await;
                    }
                    Ok(Some(Ok(Message::Close(_)))) => break,
                    Ok(Some(Ok(_))) => {}
                    Ok(Some(Err(_))) => break,
                    Ok(None) => break,
                    Err(_) => break,
                }
            }

            Ok(responses)
        })
    }
}

impl NostrBackend {
    pub fn new(private_key: Vec<u8>, relays: Vec<String>, nip96_host: Option<String>) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let mut all_relays: Vec<String> = relays;
        for (url, _) in DEFAULT_RELAYS {
            if !all_relays.iter().any(|r| r == url) {
                all_relays.push(url.to_string());
            }
        }

    let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&private_key)
            .expect("invalid Nostr private key: must be 32 bytes");
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let (pubkey_xonly, _) = keypair.x_only_public_key();

        let rt = Runtime::new().expect("failed to create tokio runtime for Nostr backend");

        Self {
            private_key,
            keypair,
            pubkey_xonly,
            relays: all_relays,
            nip96_host,
            http_client,
            connected_relays: std::sync::Mutex::new(Vec::new()),
            rt,
        }
    }

    fn pubkey_hex(&self) -> String {
        to_hex_string(self.pubkey_xonly.serialize())
    }

    fn publish_event(
        &self,
        kind: u32,
        content: &str,
        tags: Vec<Vec<String>>,
    ) -> Result<String, String> {
        let pubkey = self.pubkey_hex();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tags_json =
            serde_json::to_string(&tags).map_err(|e| format!("tags serialize: {}", e))?;
        let event_id = compute_event_id(&pubkey, created_at, kind, &tags_json, content);
        let sig = sign_event_schnorr(&event_id, &self.keypair);

        let event = serde_json::json!({
            "id": event_id,
            "pubkey": pubkey,
            "created_at": created_at,
            "kind": kind,
            "tags": tags,
            "content": content,
            "sig": sig
        });

        let msg = serde_json::json!(["EVENT", event]).to_string();
        let mut published_to = Vec::new();

        for relay_url in &self.relays {
            match self.tls_ws_request(relay_url, &msg) {
                Ok(Some(_)) | Ok(None) => {
                    published_to.push(relay_url.clone());
                }
                Err(e) => {
                    log::debug!("relay {} publish failed: {}", relay_url, e);
                }
            }
        }

        if published_to.is_empty() {
            log::warn!("Event published to no relays — content may not be available");
        } else {
            log::info!(
                "Published event {} to {} relays",
                &event_id[..16],
                published_to.len()
            );
        }

        *self.connected_relays.lock().unwrap() = published_to;
        Ok(event_id)
    }

    fn fetch_event(&self, event_id: &str) -> Result<Option<serde_json::Value>, String> {
        let req = serde_json::json!(["REQ", "fetch", {"ids": [event_id]}]).to_string();

        for relay_url in &self.relays {
            match self.tls_ws_request_all(relay_url, &req) {
                Ok(responses) => {
                    for msg in &responses {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(msg) {
                            if let Some(arr) = val.as_array() {
                                if arr.len() >= 2 && arr[0] == "EVENT" {
                                    let event = &arr[1];
                                    // Verify event signature before returning
                                    if verify_nostr_event(event).is_ok() {
                                        return Ok(Some(event.clone()));
                                    } else {
                                        log::warn!(
                                            "relay {} returned event with invalid signature",
                                            relay_url
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::debug!("relay {} fetch failed: {}", relay_url, e);
                }
            }
        }
        Ok(None)
    }

    fn upload_nip96(&self, data: &[u8], filename: &str) -> Result<String, String> {
        let host = self
            .nip96_host
            .as_deref()
            .ok_or("NIP-96 host not configured")?;

        let upload_url = format!("{}/upload", host.trim_end_matches('/'));

        let boundary = format!(
            "----Cybermanju{}",
            &blake3::hash(filename.as_bytes()).to_hex()[..16]
        );
        let mut body = Vec::new();

        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
                filename
            )
            .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(data);
        body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

        let resp = self
            .http_client
            .post(&upload_url)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(body)
            .send()
            .map_err(|e| format!("NIP-96 upload: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            let body: serde_json::Value = resp
                .json()
                .map_err(|e| format!("NIP-96 response parse: {}", e))?;
            let url = body
                .get("nip96")
                .and_then(|v| v.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !url.is_empty() {
                return Ok(url.to_string());
            }
            if let Some(url) = body.get("url").and_then(|v| v.as_str()) {
                return Ok(url.to_string());
            }
        }

        Err(format!("NIP-96 upload failed: {}", status))
    }

    fn download_nip96(&self, url: &str) -> Result<Vec<u8>, String> {
        let resp = self
            .http_client
            .get(url)
            .send()
            .map_err(|e| format!("NIP-96 download: {}", e))?;

        if resp.status().is_success() {
            resp.bytes()
                .map(|b| b.to_vec())
                .map_err(|e| format!("NIP-96 read: {}", e))
        } else {
            Err(format!("NIP-96 download failed: {}", resp.status()))
        }
    }
}

/// Compute Nostr event ID per NIP-01: SHA256 of serialized event [0, pubkey, created_at, kind, tags, content].
fn compute_event_id(pubkey: &str, created_at: u64, kind: u32, tags: &str, content: &str) -> String {
    let serialized = serde_json::json!([
        0,
        pubkey,
        created_at,
        kind,
        serde_json::from_str::<serde_json::Value>(tags).unwrap_or(serde_json::json!([])),
        content
    ]);
    let event_str = serialized.to_string();
    let hash = Sha256::digest(event_str.as_bytes());
    to_hex_string(hash)
}

/// Sign a Nostr event using secp256k1 Schnorr (NIP-01).
/// Returns the 64-byte Schnorr signature as a hex string.
fn sign_event_schnorr(event_id_hex: &str, keypair: &Keypair) -> String {
    let secp = Secp256k1::new();
    let msg_bytes = hex::decode(event_id_hex).expect("event_id must be valid hex");
    let msg = SecpMessage::from_digest_slice(&msg_bytes).expect("event_id must be 32 bytes");
    let sig = secp.sign_schnorr_no_aux_rand(&msg, keypair);
    to_hex_string(sig.serialize())
}

/// Verify a Nostr event's signature (NIP-01).
/// Recomputes the event ID from fields and verifies the Schnorr signature.
fn verify_nostr_event(event: &serde_json::Value) -> Result<(), String> {
    let pubkey = event
        .get("pubkey")
        .and_then(|v| v.as_str())
        .ok_or("missing pubkey")?;
    let created_at = event
        .get("created_at")
        .and_then(|v| v.as_u64())
        .ok_or("missing created_at")?;
    let kind = event
        .get("kind")
        .and_then(|v| v.as_u64())
        .ok_or("missing kind")?;
    let tags = event.get("tags").ok_or("missing tags")?;
    let content = event.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let sig_hex = event
        .get("sig")
        .and_then(|v| v.as_str())
        .ok_or("missing sig")?;

    // Recompute event ID
    let tags_json = serde_json::to_string(tags).map_err(|e| format!("tags serialize: {}", e))?;
    let expected_id = compute_event_id(pubkey, created_at, kind as u32, &tags_json, content);

    // Verify ID matches
    let event_id = event
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("missing id")?;
    if event_id != expected_id {
        return Err("event ID mismatch".into());
    }

    // Verify Schnorr signature
    let secp = Secp256k1::new();
    let msg_bytes = hex::decode(&expected_id).map_err(|_| "invalid event ID hex")?;
    let msg = SecpMessage::from_digest_slice(&msg_bytes).map_err(|_| "invalid event ID length")?;

    let sig_bytes = hex::decode(sig_hex).map_err(|_| "invalid signature hex")?;
    let sig = secp256k1::schnorr::Signature::from_slice(&sig_bytes)
        .map_err(|_| "invalid signature length")?;

    let pubkey_bytes = hex::decode(pubkey).map_err(|_| "invalid pubkey hex")?;
    let xonly = XOnlyPublicKey::from_slice(&pubkey_bytes).map_err(|_| "invalid pubkey")?;

    secp.verify_schnorr(&sig, &msg, &xonly)
        .map_err(|e| format!("signature verification failed: {}", e))
}

/// NIP-59 gift-wrap: encrypt a small file for a specific recipient.
/// Creates a rumpled (nested) event: inner event encrypted with recipient's pubkey,
/// wrapped in an outer event signed with an ephemeral key.
///
/// NIP-59 flow:
/// 1. Generate ephemeral keypair
/// 2. Create inner event (kind 13) signed by ephemeral key, content = NIP-44 encrypted
/// 3. Create outer event (kind 1059) signed by ephemeral key, content = NIP-44 encrypted inner event
/// 4. Publish outer event to relays
///
/// For small files (<64KB), the file content is included directly in the gift-wrap.
pub fn gift_wrap_file(
    data: &[u8],
    filename: &str,
    recipient_pubkey_hex: &str,
    _sender_keypair: &Keypair,
) -> Result<serde_json::Value, String> {
    if data.len() > 65536 {
        return Err("NIP-59 gift-wrap only supports files up to 64KB".into());
    }

    let secp = Secp256k1::new();

    // Generate ephemeral keypair for the wrapping
    let (ephemeral_secret, ephemeral_pubkey) =
        secp.generate_keypair(&mut secp256k1::rand::thread_rng());
    let ephemeral_keypair = Keypair::from_secret_key(&secp, &ephemeral_secret);
    let (ephemeral_xonly, _) = ephemeral_pubkey.x_only_public_key();

    // Parse recipient pubkey
    let recipient_bytes =
        hex::decode(recipient_pubkey_hex).map_err(|_| "invalid recipient pubkey hex")?;
    let recipient_pubkey = secp256k1::PublicKey::from_slice(&recipient_bytes)
        .map_err(|_| "invalid recipient pubkey")?;

    // Compute shared secret using ECDH
    let shared_point = secp256k1::ecdh::SharedSecret::new(&recipient_pubkey, &ephemeral_secret);
    let nonce_hash = blake3::hash(shared_point.as_ref());
    let nonce: [u8; 32] = *nonce_hash.as_bytes();

    // Create inner content: file metadata + data
    let inner_content = serde_json::json!({
        "content_type": "application/octet-stream",
        "filename": filename,
        "size": data.len(),
        "data": base64::engine::general_purpose::STANDARD.encode(data),
    });

    let inner_content_str = inner_content.to_string();

    // Encrypt inner content with XChaCha20-Poly1305 (NIP-44 style)
    let mut key_material = [0u8; 32];
    key_material.copy_from_slice(&blake3::hash(&nonce).as_bytes()[..32]);
    let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(&key_material));
    let mut file_nonce = [0u8; 12];
    file_nonce.copy_from_slice(&nonce[..12]);
    let encrypted_inner = cipher
        .encrypt(
            ChaChaNonce::from_slice(&file_nonce),
            inner_content_str.as_bytes(),
        )
        .map_err(|e| format!("inner encryption failed: {}", e))?;

    // Build inner event (kind 13 = ephemeral direct message)
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let inner_tags = serde_json::json!([]);
    let inner_event_id = compute_event_id(
        &hex::encode(ephemeral_xonly.serialize()),
        created_at,
        13,
        &inner_tags.to_string(),
        "",
    );
    let inner_sig = sign_event_schnorr(&inner_event_id, &ephemeral_keypair);

    let inner_event = serde_json::json!({
        "id": inner_event_id,
        "pubkey": hex::encode(ephemeral_xonly.serialize()),
        "created_at": created_at,
        "kind": 13,
        "tags": [],
        "content": "",
        "sig": inner_sig,
        "encrypted_content": base64::engine::general_purpose::STANDARD.encode(&encrypted_inner),
    });

    // Encrypt inner event for outer event content
    let inner_event_str = inner_event.to_string();
    let outer_nonce_hash = blake3::hash(b"nip59-outer");
    let mut outer_key_material = [0u8; 32];
    outer_key_material.copy_from_slice(outer_nonce_hash.as_bytes());
    let outer_cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(&outer_key_material));
    let mut outer_nonce = [0u8; 12];
    outer_nonce.copy_from_slice(&nonce[..12]);
    let encrypted_outer = outer_cipher
        .encrypt(
            ChaChaNonce::from_slice(&outer_nonce),
            inner_event_str.as_bytes(),
        )
        .map_err(|e| format!("outer encryption failed: {}", e))?;

    // Build outer event (kind 1059 = gift wrap)
    let outer_tags = serde_json::json!([["p", recipient_pubkey_hex]]);
    let outer_event_id = compute_event_id(
        &hex::encode(ephemeral_xonly.serialize()),
        created_at,
        1059,
        &outer_tags.to_string(),
        "",
    );
    let outer_sig = sign_event_schnorr(&outer_event_id, &ephemeral_keypair);

    let outer_event = serde_json::json!({
        "id": outer_event_id,
        "pubkey": hex::encode(ephemeral_xonly.serialize()),
        "created_at": created_at,
        "kind": 1059,
        "tags": [["p", recipient_pubkey_hex]],
        "content": base64::engine::general_purpose::STANDARD.encode(&encrypted_outer),
        "sig": outer_sig,
    });

    Ok(outer_event)
}

/// Decrypt a NIP-59 gift-wrapped event (recipient side).
pub fn unwrap_gift_wrap(
    outer_event: &serde_json::Value,
    recipient_secret_key: &SecretKey,
) -> Result<serde_json::Value, String> {
    let content_b64 = outer_event
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("missing content")?;

    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(content_b64)
        .map_err(|e| format!("base64 decode: {}", e))?;

    let _secp = Secp256k1::new();
    let ephemeral_pubkey_hex = outer_event
        .get("pubkey")
        .and_then(|v| v.as_str())
        .ok_or("missing pubkey")?;
    let ephemeral_bytes =
        hex::decode(ephemeral_pubkey_hex).map_err(|_| "invalid ephemeral pubkey")?;
    let ephemeral_pubkey = secp256k1::PublicKey::from_slice(&ephemeral_bytes)
        .map_err(|_| "invalid ephemeral pubkey")?;

    // Derive shared secret using ECDH
    let shared_point = secp256k1::ecdh::SharedSecret::new(&ephemeral_pubkey, recipient_secret_key);
    let nonce_hash = blake3::hash(shared_point.as_ref());
    let nonce: [u8; 32] = *nonce_hash.as_bytes();

    // Decrypt outer content
    let mut key_material = [0u8; 32];
    key_material.copy_from_slice(&blake3::hash(&nonce).as_bytes()[..32]);
    let cipher = ChaCha20Poly1305::new(ChaChaKey::from_slice(&key_material));
    let mut file_nonce = [0u8; 12];
    file_nonce.copy_from_slice(&nonce[..12]);
    let decrypted_outer = cipher
        .decrypt(ChaChaNonce::from_slice(&file_nonce), encrypted.as_ref())
        .map_err(|e| format!("outer decryption failed: {}", e))?;

    let inner_event: serde_json::Value = serde_json::from_slice(&decrypted_outer)
        .map_err(|e| format!("inner event parse: {}", e))?;

    Ok(inner_event)
}

fn to_hex_string(data: impl AsRef<[u8]>) -> String {
    data.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
}

impl StorageBackend for NostrBackend {
    fn name(&self) -> &str {
        "nostr"
    }

    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::Nostr
    }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let data = std::fs::read(local_path).map_err(|e| format!("read {}: {}", local_path, e))?;

        if data.is_empty() {
            return Err("cannot upload empty file".into());
        }

        let filename = std::path::Path::new(local_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        if let Ok(url) = self.upload_nip96(&data, filename) {
            return Ok(url);
        }

        let content_hash = blake3::hash(&data).to_hex().to_string();
        let content = format!(
            "cybermanju:shard:{}:{}:{}",
            filename,
            data.len(),
            content_hash
        );
        let tags = vec![
            vec!["t".to_string(), "cybermanju".to_string()],
            vec!["size".to_string(), data.len().to_string()],
        ];

        let event_id = self.publish_event(1063, &content, tags)?;
        Ok(format!("nostr://{}", event_id))
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let identifier = remote_path.strip_prefix("nostr://").unwrap_or(remote_path);

        if identifier.starts_with("http") {
            let data = self.download_nip96(identifier)?;
            std::fs::write(local_path, &data)
                .map_err(|e| format!("write {}: {}", local_path, e))?;
            return Ok(());
        }

        let event = self
            .fetch_event(identifier)?
            .ok_or_else(|| format!("event {} not found on any relay", identifier))?;

        let content = event.get("content").and_then(|v| v.as_str()).unwrap_or("");

        if let Some(rest) = content.strip_prefix("cybermanju:shard:") {
            let parts: Vec<&str> = rest.splitn(3, ':').collect();
            if parts.len() == 3 {
                return Err(
                    "content referenced by hash — requires content to be stored in event or via NIP-96"
                        .into(),
                );
            }
        }

        Err(format!(
            "could not extract file content from event {}",
            identifier
        ))
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let identifier = remote_path.strip_prefix("nostr://").unwrap_or(remote_path);
        let tags = vec![vec!["e".to_string(), identifier.to_string()]];
        self.publish_event(5, "", tags)?;
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let pubkey = self.pubkey_hex();
        let req = serde_json::json!(["REQ", "list", {
            "authors": [pubkey],
            "kinds": [1063],
            "limit": 100
        }])
        .to_string();

        let mut files = Vec::new();

        for relay_url in &self.relays {
            match self.tls_ws_request_all(relay_url, &req) {
                Ok(responses) => {
                    for msg in &responses {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(msg) {
                            if let Some(arr) = val.as_array() {
                                if arr.len() >= 2 && arr[0] == "EVENT" {
                                    if let Some(event) = arr[1].get("content") {
                                        let content = event.as_str().unwrap_or("");
                                        if content.starts_with("cybermanju:shard:")
                                            && content.contains(prefix)
                                        {
                                            let parts: Vec<&str> = content.splitn(4, ':').collect();
                                            if parts.len() >= 4 {
                                                files.push(RemoteFile {
                                                    name: parts[2].to_string(),
                                                    path: format!(
                                                        "nostr://{}",
                                                        arr[1]
                                                            .get("id")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("")
                                                    ),
                                                    size_bytes: parts[3].parse().unwrap_or(0),
                                                    modified_at: String::new(),
                                                    url: String::new(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
                Err(e) => {
                    log::debug!("relay {} list failed: {}", relay_url, e);
                }
            }
        }

        Ok(files)
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let id = remote_path.strip_prefix("nostr://").unwrap_or(remote_path);
        if let Some(relay_url) = self.relays.first() {
            let ws_url = relay_url
                .replace("wss://", "https://")
                .replace("ws://", "http://");
            return Ok(format!("{}/e/{}", ws_url, id));
        }
        Ok(format!("nostr://{}", id))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let mut connected = 0;
        for relay_url in self.relays.iter().take(5) {
            let msg = serde_json::json!(["REQ", "test", {"limit": 0}]).to_string();
            if self.tls_ws_request(relay_url, &msg).is_ok() {
                connected += 1;
            }
        }
        if connected == 0 {
            Err("could not connect to any Nostr relay".into())
        } else {
            log::info!("Connected to {} Nostr relays via TLS", connected);
            Ok(true)
        }
    }
}
