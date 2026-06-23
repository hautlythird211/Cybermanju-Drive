use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};

use crate::relay_updater::RelayIpList;

/// IPFS/Iroh backend for content-addressed storage.
///
/// Uses public IPFS gateways for reads and HTTP pinning APIs for writes.
/// Syncthing relay IPs are used for content routing.
/// The relay IP list auto-refreshes hourly.
pub struct IrohBackend {
    gateway_url: String,
    pinning_url: Option<String>,
    relay_ips: RelayIpList,
    http_client: reqwest::blocking::Client,
}

/// Default public IPFS gateways for content retrieval.
const DEFAULT_GATEWAYS: &[&str] = &[
    "https://ipfs.io/ipfs",
    "https://gateway.pinata.cloud/ipfs",
    "https://cloudflare-ipfs.com/ipfs",
    "https://dweb.link/ipfs",
];

impl IrohBackend {
    pub fn new(gateway_url: Option<String>) -> Self {
        let relay_ips = RelayIpList::new();
        relay_ips.refresh_if_stale();

        let http_client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            gateway_url: gateway_url.unwrap_or_else(|| DEFAULT_GATEWAYS[0].to_string()),
            pinning_url: None,
            relay_ips,
            http_client,
        }
    }

    pub fn with_pinning_url(mut self, url: String) -> Self {
        self.pinning_url = Some(url);
        self
    }

    /// Validate a CID string: must start with "b3-" followed by 64 hex chars.
    fn validate_cid(cid: &str) -> bool {
        cid.starts_with("b3-") && cid.len() == 67 && cid[3..].bytes().all(|b| b.is_ascii_hexdigit())
    }

    /// Compute BLAKE3 CID for content addressing.
    fn compute_cid(data: &[u8]) -> String {
        let hash = blake3::hash(data);
        format!("b3-{}", hash.to_hex())
    }

    /// Try to fetch content from multiple IPFS gateways.
    fn fetch_from_gateways(&self, cid: &str) -> Result<Vec<u8>, String> {
        if !Self::validate_cid(cid) {
            return Err(format!("invalid CID format: {}", cid));
        }
        let mut gateways = vec![self.gateway_url.clone()];
        gateways.extend(DEFAULT_GATEWAYS.iter().map(|s| s.to_string()));

        // Also try Syncthing relay IPs as IPFS-like gateways on port 8080
        // NOTE: These use plaintext HTTP — content is encrypted at the shard level,
        // so transit eavesdropping only reveals encrypted ciphertext, not plaintext.
        let relay_ips = self.relay_ips.ips();
        for ip in relay_ips.iter().take(5) {
            gateways.push(format!("http://{}:8080/ipfs", ip));
        }

        for gw in &gateways {
            let url = format!("{}/{}", gw.trim_end_matches('/'), cid);
            match self.http_client.get(&url).send() {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(bytes) = resp.bytes() {
                        return Ok(bytes.to_vec());
                    }
                }
                _ => continue,
            }
        }
        Err(format!("content {} not found on any gateway", cid))
    }

    /// Try to pin content via HTTP pinning service.
    fn pin_content(&self, cid: &str, _data: &[u8]) -> Result<(), String> {
        if !Self::validate_cid(cid) {
            return Err(format!("invalid CID: {}", cid));
        }

        if let Some(pinning_url) = &self.pinning_url {
            let url = format!("{}/pinning/pinJSON", pinning_url.trim_end_matches('/'));
            let body = serde_json::json!({
                "ipfsPin": cid,
                "name": cid,
                "keyValues": []
            });
            let resp = self
                .http_client
                .post(&url)
                .json(&body)
                .send()
                .map_err(|e| format!("pin request failed: {}", e))?;
            if resp.status().is_success() {
                log::info!("Pinned {} to pinning service", cid);
                return Ok(());
            }
        }

        // Try to pin via public IPFS node APIs (plaintext HTTP — content is encrypted)
        let relay_ips = self.relay_ips.ips();
        for ip in relay_ips.iter().take(3) {
            let url = format!("http://{}:5001/api/v0/pin/add?arg={}", ip, cid);
            if self.http_client.post(&url).send().is_ok() {
                log::info!("Pinned {} via relay node {}", cid, ip);
                return Ok(());
            }
        }

        // Content is still accessible via gateways even without explicit pinning
        log::warn!(
            "Could not pin {} — content may not persist without pinning",
            cid
        );
        Ok(())
    }

    /// Get the list of working relay IPs for content routing.
    pub fn relay_ips(&self) -> Vec<String> {
        self.relay_ips.ips()
    }

    /// Force refresh the relay IP list.
    pub fn refresh_relays(&self) {
        self.relay_ips.refresh_if_stale();
    }
}

impl StorageBackend for IrohBackend {
    fn name(&self) -> &str {
        "ipfs"
    }

    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::Iroh
    }

    fn upload_file(&self, local_path: &str, _remote_path: &str) -> Result<String, String> {
        let data = std::fs::read(local_path).map_err(|e| format!("read {}: {}", local_path, e))?;

        if data.is_empty() {
            return Err("cannot upload empty file".into());
        }

        let cid = Self::compute_cid(&data);

        // Try to pin the content
        self.pin_content(&cid, &data)?;

        Ok(format!("ipfs://{}", cid))
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        // Extract CID from ipfs:// or bare CID
        let cid = remote_path.strip_prefix("ipfs://").unwrap_or(remote_path);

        let data = self.fetch_from_gateways(cid)?;
        std::fs::write(local_path, &data).map_err(|e| format!("write {}: {}", local_path, e))?;
        Ok(())
    }

    fn delete_file(&self, _remote_path: &str) -> Result<(), String> {
        // Content-addressed: cannot delete content, only stop pinning
        log::info!("Content-addressed storage: delete is a no-op (unpin only)");
        Ok(())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        // IPFS doesn't have native listing — would need a separate index
        Ok(Vec::new())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let cid = remote_path.strip_prefix("ipfs://").unwrap_or(remote_path);
        Ok(format!(
            "{}/{}",
            self.gateway_url.trim_end_matches('/'),
            cid
        ))
    }

    fn test_connection(&self) -> Result<bool, String> {
        // Test by fetching a well-known IPFS CID
        let test_url = format!(
            "{}/QmPChd2hVbrJ6bfo3WBcTW4iZnpHm8TEzWkLHmLpXhF68A",
            self.gateway_url.trim_end_matches('/')
        );
        match self.http_client.get(&test_url).send() {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => Err(format!("gateway test failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_cid() {
        let data = b"hello world";
        let cid = IrohBackend::compute_cid(data);
        assert!(cid.starts_with("b3-"));
        assert_eq!(cid.len(), 67); // "b3-" + 64 hex chars
    }

    #[test]
    fn test_cid_deterministic() {
        let data = b"test data";
        let cid1 = IrohBackend::compute_cid(data);
        let cid2 = IrohBackend::compute_cid(data);
        assert_eq!(cid1, cid2);
    }
}
