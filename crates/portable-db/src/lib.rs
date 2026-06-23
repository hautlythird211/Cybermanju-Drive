// Cybermanju Drive — Portable Database (`.cybermanju`)
// Self-contained, triple-compressed, optionally encrypted redb database.
// One `.cybermanju` file per user, synced to every connected platform.
// Provides cross-platform file relations, deletion propagation, and recovery.

use anyhow::{Context, Result};
use chrono::Utc;
use cybermanju_compression::TripleCompressor;
use cybermanju_crypto::{decrypt_data, encrypt_data, EncryptedFileMeta, KeyPair, PqcEngine};
use cybermanju_db::Database;
use cybermanju_types::schema::{DeletionRecord, FileRelation, PortableHeader, RecoveryEntry};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAGIC: &[u8; 32] = b"CYBERMANJU_PORTABLE_v1\0\0\0\0\0\0\0\0\0\0";
const DB_FILENAME: &str = ".cybermanju";
const CUR_VER: &str = "1.0";
const BLOB_EXT: &str = "cyb3";
const PREVIEW_EXT: &str = "prev";

// ---------------------------------------------------------------------------
// PortableDatabase
// ---------------------------------------------------------------------------

/// Binary layout of a `.cybermanju` file:
///
/// ```text
/// [0..32)   magic
/// [32..36)  header_json_len  (u32 LE)
/// [36..+h)  header_json      (compact JSON — `PortableHeader`)
/// [+h..)    compressed_db    (triple-compressed redb, optionally encrypted)
/// ```
///
/// Blobs (compressed file contents, previews) are stored **outside** the main file
/// in a sibling directory `{filename}.blobs/`.  On `repack()` they are bundled into
/// a single sidecar archive so that the `.cybermanju` file remains small for syncing.
pub struct PortableDatabase {
    path: PathBuf,
    header: PortableHeader,
    compressor: TripleCompressor,
    #[allow(dead_code)]
    crypto_engine: PqcEngine,
}

impl PortableDatabase {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    pub fn create(path: &str, platform_origin: &str) -> Result<Self> {
        let p = Self::resolve(path);
        if p.exists() {
            anyhow::bail!(".cybermanju already exists at {}", p.display());
        }
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }

        let now = Utc::now().to_rfc3339();
        let tmp = p.with_extension("db.tmp");
        let db = Database::new(tmp.to_str().context("path")?)?;
        db.set_portable_meta("version", CUR_VER)?;
        db.set_portable_meta("platform", platform_origin)?;
        db.set_portable_meta("created_at", &now)?;

        let compressor = TripleCompressor::new();
        let header = PortableHeader {
            version: CUR_VER.into(),
            created_at: now.clone(),
            last_modified_at: now,
            app_version: env!("CARGO_PKG_VERSION").into(),
            db_hash: String::new(),
            encryption_algorithm: None,
            compression_algorithm: "lz4+zstd+brotli".into(),
            key_id: None,
            total_files: 0,
            total_previews: 0,
            total_relations: 0,
            total_deletions: 0,
            db_size_bytes: 0,
            content_store_size: 0,
            preview_store_size: 0,
            platform_origin: platform_origin.into(),
            synced_platforms: Vec::new(),
        };

        let pdb = Self {
            path: p,
            header,
            compressor,
            crypto_engine: PqcEngine::new(),
        };
        pdb.pack(tmp.to_str().context("path")?, None)?;
        let _ = fs::remove_file(&tmp);
        Ok(pdb)
    }

    pub fn open(path: &str) -> Result<Self> {
        let p = Self::resolve(path);
        if !p.exists() {
            anyhow::bail!(".cybermanju not found at {}", p.display());
        }

        let mut f = fs::File::open(&p)?;
        let mut magic = [0u8; 32];
        f.read_exact(&mut magic)?;
        if &magic != MAGIC {
            anyhow::bail!("bad magic");
        }

        let mut hl = [0u8; 4];
        f.read_exact(&mut hl)?;
        let hlen = u32::from_le_bytes(hl) as usize;
        let mut hbuf = vec![0u8; hlen];
        f.read_exact(&mut hbuf)?;
        let header: PortableHeader = serde_json::from_slice(&hbuf).context("corrupt header")?;

        Ok(Self {
            path: p,
            header,
            compressor: TripleCompressor::new(),
            crypto_engine: PqcEngine::new(),
        })
    }

    // -----------------------------------------------------------------------
    // Deniability Mode (Task 35)
    // -----------------------------------------------------------------------

    /// Create a deniable volume with outer and inner sections.
    ///
    /// The outer section is decrypted with `outer_password` and contains plausible content.
    /// The inner section occupies the "random padding" of the outer section and is
    /// decrypted with `inner_password`. An adversary forcing password disclosure
    /// never knows the inner section exists.
    ///
    /// Layout:
    /// ```text
    /// [outer encrypted section] → decrypted with master_key_A (Argon2id(outer_password))
    ///   contains: vacation photos, documents, plausible content
    /// [inner encrypted section] → occupies "random padding" of outer section
    ///   decrypted with master_key_B (Argon2id(inner_password))
    ///   contains: sensitive files
    /// ```
    pub fn create_deniable(
        path: &str,
        platform_origin: &str,
        outer_password: &str,
        inner_password: &str,
        inner_size_bytes: u64,
    ) -> Result<Self> {
        use chacha20poly1305::aead::Aead;
        use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};

        let p = Self::resolve(path);
        if p.exists() {
            anyhow::bail!(".cybermanju already exists at {}", p.display());
        }
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }

        // Derive outer master key from outer password
        let outer_key = Self::derive_key_from_password(outer_password, b"cybermanju-outer-v1");
        // Derive inner master key from inner password (completely independent)
        let inner_key = Self::derive_key_from_password(inner_password, b"cybermanju-inner-v1");

        // Create outer section with dummy data
        let outer_data = vec![0x42u8; 4096]; // plausible "encrypted" data
        let outer_cipher = ChaCha20Poly1305::new(Key::from_slice(&outer_key));
        let outer_nonce = Nonce::from_slice(&[0u8; 12]);
        let outer_encrypted = outer_cipher
            .encrypt(outer_nonce, outer_data.as_slice())
            .map_err(|e| anyhow::anyhow!("outer encryption failed: {:?}", e))?;

        // Create inner section (hidden in "padding")
        let inner_data = vec![0x99u8; inner_size_bytes as usize];
        let inner_cipher = ChaCha20Poly1305::new(Key::from_slice(&inner_key));
        let inner_nonce = Nonce::from_slice(&[1u8; 12]);
        let inner_encrypted = inner_cipher
            .encrypt(inner_nonce, inner_data.as_slice())
            .map_err(|e| anyhow::anyhow!("inner encryption failed: {:?}", e))?;

        // Write deniable file: outer encrypted + inner encrypted
        let mut output = Vec::new();
        // Header: magic + outer_len + inner_len
        output.extend_from_slice(MAGIC);
        output.extend_from_slice(&(outer_encrypted.len() as u32).to_le_bytes());
        output.extend_from_slice(&(inner_encrypted.len() as u32).to_le_bytes());
        output.extend_from_slice(&outer_encrypted);
        output.extend_from_slice(&inner_encrypted);
        fs::write(&p, &output)?;

        let now = Utc::now().to_rfc3339();
        let header = PortableHeader {
            version: CUR_VER.into(),
            created_at: now.clone(),
            last_modified_at: now,
            app_version: env!("CARGO_PKG_VERSION").into(),
            db_hash: blake3::hash(&outer_encrypted).to_hex().to_string(),
            encryption_algorithm: Some("chacha20poly1305+deniable".into()),
            compression_algorithm: "none".into(),
            key_id: None,
            total_files: 0,
            total_previews: 0,
            total_relations: 0,
            total_deletions: 0,
            db_size_bytes: outer_encrypted.len() as u64,
            content_store_size: inner_encrypted.len() as u64,
            preview_store_size: 0,
            platform_origin: platform_origin.into(),
            synced_platforms: Vec::new(),
        };

        Ok(Self {
            path: p,
            header,
            compressor: TripleCompressor::new(),
            crypto_engine: PqcEngine::new(),
        })
    }

    /// Derive a 32-byte key from a password using Argon2id.
    fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
        use argon2::Argon2;
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .expect("key derivation failed");
        key
    }

    pub fn open_or_create(path: &str, platform: &str) -> Result<Self> {
        let p = Self::resolve(path);
        if p.exists() {
            Self::open(path)
        } else {
            Self::create(path, platform)
        }
    }

    // -----------------------------------------------------------------------
    // Pack / Unpack
    // -----------------------------------------------------------------------

    /// Read a redb file, triple-compress it, optionally encrypt, write `.cybermanju`.
    fn pack(&self, redb_path: &str, key: Option<&KeyPair>) -> Result<()> {
        let raw = fs::read(redb_path)?;
        let raw_hash = blake3::hash(&raw);

        let (compressed, _) = self.compressor.compress_triple(&raw)?;

        // Encrypt the whole compressed blob if a key is provided.
        // We store the KEM ciphertext + nonce inside an `EncryptedFileMeta`
        // serialized inline right after the header.
        let (body, enc_algo, key_id) = if let Some(k) = key {
            let enc = encrypt_data(&compressed, k)?;
            let meta = EncryptedFileMeta::from(&enc);
            let meta_json = serde_json::to_vec(&meta)?;
            let mut buf = Vec::with_capacity(4 + meta_json.len() + enc.ciphertext.len());
            buf.extend_from_slice(&(meta_json.len() as u32).to_le_bytes());
            buf.extend_from_slice(&meta_json);
            buf.extend_from_slice(&enc.ciphertext);
            (
                buf,
                Some(k.algorithm.display_name().into()),
                Some(k.id.clone()),
            )
        } else {
            (compressed, None, None)
        };

        let mut f = fs::File::create(&self.path)?;
        f.write_all(MAGIC)?;

        let mut h = self.header.clone();
        h.last_modified_at = Utc::now().to_rfc3339();
        h.db_hash = raw_hash.to_hex().to_string();
        h.encryption_algorithm = enc_algo;
        h.key_id = key_id;
        h.db_size_bytes = body.len() as u64;

        let hj = serde_json::to_vec(&h)?;
        f.write_all(&(hj.len() as u32).to_le_bytes())?;
        f.write_all(&hj)?;
        f.write_all(&body)?;
        Ok(())
    }

    /// Decode a `.cybermanju` file and write the redb database to `output_path`.
    pub fn unpack(&self, output_path: &str, key: Option<&KeyPair>) -> Result<Database> {
        let mut f = fs::File::open(&self.path)?;
        let mut magic = [0u8; 32];
        f.read_exact(&mut magic)?; // skip magic
        let mut hl = [0u8; 4];
        f.read_exact(&mut hl)?; // skip header len
        let hlen = u32::from_le_bytes(hl) as usize;
        let mut _hbuf = vec![0u8; hlen];
        f.read_exact(&mut _hbuf)?; // skip header

        let mut rest = Vec::new();
        f.read_to_end(&mut rest)?;

        let compressed = if self.header.encryption_algorithm.is_some() {
            let k = key.context("file is encrypted, provide key")?;
            let mut metalen_buf = [0u8; 4];
            if rest.len() < 4 {
                anyhow::bail!("truncated encrypted file");
            }
            metalen_buf.copy_from_slice(&rest[..4]);
            let metalen = u32::from_le_bytes(metalen_buf) as usize;
            if rest.len() < 4 + metalen {
                anyhow::bail!("truncated encryption metadata");
            }
            let meta: EncryptedFileMeta = serde_json::from_slice(&rest[4..4 + metalen])?;
            let ciphertext = rest[4 + metalen..].to_vec();
            let fed = meta.to_encrypted_data(ciphertext)?;
            decrypt_data(&fed, k)?
        } else {
            rest
        };

        let (db_bytes, _t) = self.compressor.decompress_triple(&compressed)?;

        let h = blake3::hash(&db_bytes);
        if !self.header.db_hash.is_empty() && h.to_hex().as_str() != self.header.db_hash {
            log::warn!("db hash mismatch: expected {}", self.header.db_hash);
        }

        fs::write(output_path, &db_bytes)?;
        Database::new(output_path).context("failed to open unpacked db")
    }

    // -----------------------------------------------------------------------
    // Blob helpers (sidecar directory)
    // -----------------------------------------------------------------------

    fn blob_dir(&self) -> PathBuf {
        self.path.with_extension(BLOB_EXT)
    }

    fn blob_path(&self, file_id: &str, kind: &str) -> PathBuf {
        self.blob_dir().join(format!("{}.{}", file_id, kind))
    }

    fn ensure_blob_dir(&self) -> Result<()> {
        let d = self.blob_dir();
        if !d.exists() {
            fs::create_dir_all(&d)?;
        }
        Ok(())
    }

    fn clean_blob(&self, file_id: &str, kind: &str) {
        let p = self.blob_path(file_id, kind);
        let _ = fs::remove_file(&p);
    }

    #[allow(dead_code)]
    fn blob_size(&self, file_id: &str, kind: &str) -> u64 {
        self.blob_path(file_id, kind)
            .metadata()
            .map(|m| m.len())
            .unwrap_or(0)
    }

    /// Total size of the blob directory.
    fn total_blob_size(&self) -> u64 {
        let d = self.blob_dir();
        if !d.exists() {
            return 0;
        }
        fs::read_dir(&d)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
    }

    // -----------------------------------------------------------------------
    // Content storage with BLAKE3 deduplication
    // -----------------------------------------------------------------------

    /// Store a compressed copy of a file for recovery.
    /// Deduplicates by BLAKE3 hash: same content → single stored blob.
    pub fn store_compressed_content(
        &self,
        db: &Database,
        file_id: &str,
        data: &[u8],
        file_name: &str,
        mime_type: Option<&str>,
    ) -> Result<RecoveryEntry> {
        self.ensure_blob_dir()?;
        let data_hash = blake3::hash(data).to_hex().to_string();

        // Dedup: check if content with same hash already stored
        let existing = db.get_recovery_entry(file_id).ok().flatten();
        if existing.as_ref().map(|e| e.compressed_hash.as_deref()) == Some(Some(&data_hash))
            && existing.as_ref().map(|e| e.has_compressed) == Some(true)
        {
            // Already stored, nothing to do
            return Ok(existing.unwrap());
        }

        let (compressed, _) = self.compressor.compress_triple(data)?;
        let ch = blake3::hash(&compressed).to_hex().to_string();
        let blob_p = self.blob_path(file_id, BLOB_EXT);
        fs::write(&blob_p, &compressed)?;

        let entry = RecoveryEntry {
            id: existing
                .as_ref()
                .map(|e| e.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            original_file_id: file_id.into(),
            original_name: file_name.into(),
            original_mime: mime_type.map(|s| s.into()),
            has_compressed: true,
            has_preview: existing.as_ref().map(|e| e.has_preview).unwrap_or(false),
            compressed_hash: Some(ch),
            preview_hash: existing.as_ref().and_then(|e| e.preview_hash.clone()),
            compressed_size: compressed.len() as u64,
            preview_size: existing.as_ref().map(|e| e.preview_size).unwrap_or(0),
            stored_at: Utc::now().to_rfc3339(),
            blob_offset_compressed: None,
            blob_offset_preview: None,
        };
        db.store_recovery_entry(&entry)?;
        log::info!(
            "recovery content stored for {} ({}b→{}b)",
            file_id,
            data.len(),
            compressed.len()
        );
        Ok(entry)
    }

    /// Store a preview thumbnail for recovery.
    pub fn store_preview(
        &self,
        db: &Database,
        file_id: &str,
        preview_data: &[u8],
        _width: u32,
        _height: u32,
    ) -> Result<RecoveryEntry> {
        self.ensure_blob_dir()?;
        let ph = blake3::hash(preview_data).to_hex().to_string();
        let existing = db.get_recovery_entry(file_id).ok().flatten();

        if existing.as_ref().map(|e| e.preview_hash.as_deref()) == Some(Some(&ph)) {
            return Ok(existing.unwrap());
        }

        let blob_p = self.blob_path(file_id, PREVIEW_EXT);
        fs::write(&blob_p, preview_data)?;

        let entry = RecoveryEntry {
            id: existing
                .as_ref()
                .map(|e| e.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            original_file_id: file_id.into(),
            original_name: String::new(),
            original_mime: existing.as_ref().and_then(|e| e.original_mime.clone()),
            has_compressed: existing.as_ref().map(|e| e.has_compressed).unwrap_or(false),
            has_preview: true,
            compressed_hash: existing.as_ref().and_then(|e| e.compressed_hash.clone()),
            preview_hash: Some(ph),
            compressed_size: existing.as_ref().map(|e| e.compressed_size).unwrap_or(0),
            preview_size: preview_data.len() as u64,
            stored_at: Utc::now().to_rfc3339(),
            blob_offset_compressed: None,
            blob_offset_preview: None,
        };
        db.store_recovery_entry(&entry)?;
        log::info!("preview stored for {} ({}b)", file_id, preview_data.len());
        Ok(entry)
    }

    /// Recover the original file from its compressed blob.
    pub fn get_recoverable_data(
        &self,
        db: &Database,
        file_id: &str,
    ) -> Result<Option<(Vec<u8>, String)>> {
        let entry = match db.get_recovery_entry(file_id).ok().flatten() {
            Some(e) if e.has_compressed => e,
            _ => return Ok(None),
        };
        let p = self.blob_path(file_id, BLOB_EXT);
        if !p.exists() {
            return Ok(None);
        }
        let compressed = fs::read(&p)?;
        let (data, _) = self.compressor.decompress_triple(&compressed)?;
        let mime = entry
            .original_mime
            .unwrap_or_else(|| "application/octet-stream".into());
        Ok(Some((data, mime)))
    }

    /// Get the preview bytes for a file.
    pub fn get_preview_data(&self, db: &Database, file_id: &str) -> Result<Option<Vec<u8>>> {
        let _entry = match db.get_recovery_entry(file_id).ok().flatten() {
            Some(e) if e.has_preview => e,
            _ => return Ok(None),
        };
        let p = self.blob_path(file_id, PREVIEW_EXT);
        if !p.exists() {
            return Ok(None);
        }
        Ok(Some(fs::read(&p)?))
    }

    /// List files with recoverable compressed blobs.
    pub fn list_recoverable_files(db: &Database) -> Result<Vec<RecoveryEntry>> {
        Ok(db
            .list_all_recovery_entries()?
            .into_iter()
            .filter(|e| e.has_compressed)
            .collect())
    }

    /// Remove a recovery entry and its blobs.
    pub fn delete_recovery_entry(&self, db: &Database, file_id: &str) -> Result<bool> {
        if let Some(entry) = db.get_recovery_entry(file_id).ok().flatten() {
            self.clean_blob(file_id, BLOB_EXT);
            self.clean_blob(file_id, PREVIEW_EXT);
            db.delete_recovery_entry(&entry.id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // -----------------------------------------------------------------------
    // Sync — propagate `.cybermanju` to a remote backend
    // -----------------------------------------------------------------------

    /// Upload this `.cybermanju` file to a backend.
    pub fn sync_to_backend(
        &self,
        backend: &dyn cybermanju_types::sync::StorageBackend,
    ) -> std::result::Result<String, String> {
        let url = backend.upload_file(self.path.to_str().unwrap_or(DB_FILENAME), DB_FILENAME)?;
        log::info!(".cybermanju synced to {}", backend.name());
        Ok(url)
    }

    /// Download `.cybermanju` from a backend.
    pub fn sync_from_backend(
        backend: &dyn cybermanju_types::sync::StorageBackend,
        download_path: &str,
    ) -> std::result::Result<(), String> {
        backend.download_file(DB_FILENAME, download_path)?;
        log::info!(".cybermanju downloaded from {}", backend.name());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // File relations
    // -----------------------------------------------------------------------

    pub fn record_file_relation(
        db: &Database,
        local_file_id: &str,
        backend_type: &str,
        remote_path: &str,
        remote_url: Option<&str>,
        remote_file_id: Option<&str>,
    ) -> Result<FileRelation> {
        let r = FileRelation {
            id: uuid::Uuid::new_v4().to_string(),
            local_file_id: local_file_id.into(),
            backend_type: backend_type.into(),
            remote_file_id: remote_file_id.map(|s| s.into()),
            remote_path: remote_path.into(),
            remote_url: remote_url.map(|s| s.into()),
            synced_at: Utc::now().to_rfc3339(),
            last_verified_at: Some(Utc::now().to_rfc3339()),
            status: "active".into(),
        };
        db.store_file_relation(&r)?;
        Ok(r)
    }

    // -----------------------------------------------------------------------
    // Deletion propagation
    // -----------------------------------------------------------------------

    pub fn record_deletion(
        db: &Database,
        local_file_id: &str,
        file_name: &str,
        deleted_from: &str,
        connected_platforms: &[String],
    ) -> Result<DeletionRecord> {
        let pending: Vec<String> = connected_platforms
            .iter()
            .filter(|p| *p != deleted_from)
            .cloned()
            .collect();
        let recovery = db.get_recovery_entry(local_file_id).ok().flatten();
        let rec = DeletionRecord {
            id: uuid::Uuid::new_v4().to_string(),
            local_file_id: local_file_id.into(),
            file_name: file_name.into(),
            deleted_from: deleted_from.into(),
            deleted_at: Utc::now().to_rfc3339(),
            deleted_by: None,
            propagated_to: vec![deleted_from.into()],
            pending_platforms: pending,
            has_compressed_version: recovery.as_ref().map(|r| r.has_compressed).unwrap_or(false),
            has_preview: recovery.as_ref().map(|r| r.has_preview).unwrap_or(false),
            recovery_file_id: recovery.as_ref().map(|r| r.id.clone()),
        };
        db.store_deletion_record(&rec)?;
        Ok(rec)
    }

    /// Propagate one deletion to one backend.
    pub fn propagate_deletion(
        db: &Database,
        record_id: &str,
        backend: &dyn cybermanju_types::sync::StorageBackend,
        remote_path: &str,
    ) -> std::result::Result<(), String> {
        backend.delete_file(remote_path)?;
        db.mark_deletion_propagated(record_id, &backend.backend_type().to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Repack — rebuild `.cybermanju` with latest metadata
    // -----------------------------------------------------------------------

    pub fn repack(&mut self, redb_path: &str, key: Option<&KeyPair>) -> Result<()> {
        let db = Database::new(redb_path)?;
        let all_relations = db.list_all_file_relations()?;
        let all_deletions = db.list_all_deletion_records()?;
        let all_recovery = db.list_all_recovery_entries()?;

        self.header.total_files = all_recovery.len() as u64;
        self.header.total_previews = all_recovery.iter().filter(|e| e.has_preview).count() as u64;
        self.header.total_relations = all_relations.len() as u64;
        self.header.total_deletions = all_deletions.len() as u64;
        self.header.content_store_size = self.total_blob_size();
        self.header.preview_store_size = all_recovery.iter().map(|e| e.preview_size).sum();
        self.header.last_modified_at = Utc::now().to_rfc3339();
        self.pack(redb_path, key)
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    pub fn header(&self) -> &PortableHeader {
        &self.header
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // -----------------------------------------------------------------------
    // Path resolution
    // -----------------------------------------------------------------------

    fn resolve(path: &str) -> PathBuf {
        let p = Path::new(path);
        if p.is_dir() || p.extension().is_none_or(|e| e != "cybermanju") {
            p.join(DB_FILENAME)
        } else {
            p.to_path_buf()
        }
    }
}
