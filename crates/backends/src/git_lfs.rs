use crate::util::http_client;
use base64::Engine;
use cybermanju_types::sync::{
    LfsAction, LfsBatchRequest, LfsBatchResponse, LfsObject, LfsObjectResponse, LfsPointer,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Identifies which git platform the LFS client is interacting with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LfsPlatform {
    GitHub,
    GitLab,
    Gitea, // Also Codeberg (Forgejo)
}

impl LfsPlatform {
    pub fn from_backend_type(s: &str) -> Self {
        match s {
            "github" => Self::GitHub,
            "gitlab" => Self::GitLab,
            "codeberg" | "gitea" => Self::Gitea,
            _ => Self::GitHub,
        }
    }
}

/// Git LFS client for interacting with LFS-enabled git remotes.
/// Follows the Git LFS Batch API v1 spec:
/// https://github.com/git-lfs/git-lfs/blob/main/docs/api/batch.md
pub struct GitLfsClient {
    /// Git remote URL (e.g. "https://github.com/owner/repo")
    git_url: String,
    /// Auth token (Personal Access Token or OAuth token)
    token: String,
    /// Platform type for platform-specific auth/URL logic
    platform: LfsPlatform,
}

impl GitLfsClient {
    /// Create a new LFS client.
    ///
    /// `platform_url` is the base URL of the git platform (e.g. "https://github.com")
    /// `repo_path` is the repository path (e.g. "owner/repo" or URL-encoded project ID)
    /// `token` is the authentication token
    /// `platform` identifies the platform for correct URL construction and auth
    pub fn new(platform_url: &str, repo_path: &str, token: &str, platform: LfsPlatform) -> Self {
        let base = platform_url.trim_end_matches('/');

        // Build the git remote URL (used to derive the LFS batch API endpoint)
        let git_url = match platform {
            LfsPlatform::GitHub => {
                // GitHub LFS endpoint: https://github.com/{owner}/{repo}.git/info/lfs/objects/batch
                format!("https://github.com/{}", repo_path.trim_start_matches('/'))
            }
            LfsPlatform::GitLab => {
                // GitLab LFS endpoint: https://gitlab.com/{project_path}.git/info/lfs/objects/batch
                format!("{}/{}", base, repo_path.trim_start_matches('/'))
            }
            LfsPlatform::Gitea => {
                // Gitea/Forgejo LFS endpoint: https://instance/{owner}/{repo}.git/info/lfs/objects/batch
                format!("{}/{}", base, repo_path.trim_start_matches('/'))
            }
        };

        Self {
            git_url,
            token: token.to_string(),
            platform,
        }
    }

    /// Build the standard LFS batch API endpoint.
    /// Standard: {git_url}.git/info/lfs/objects/batch
    fn batch_url(&self) -> String {
        format!("{}.git/info/lfs/objects/batch", self.git_url)
    }

    /// Build the auth header value for LFS batch requests.
    /// Different platforms use different auth mechanisms.
    fn lfs_auth_header(&self) -> (String, String) {
        match self.platform {
            LfsPlatform::GitHub => {
                // GitHub LFS accepts Bearer tokens or Basic auth with token as password
                (
                    String::from("Authorization"),
                    format!("Bearer {}", self.token),
                )
            }
            LfsPlatform::GitLab => {
                // GitLab LFS uses HTTP Basic auth with:
                //   username: anything (often "user" or empty)
                //   password: personal access token
                let basic = base64::engine::general_purpose::STANDARD
                    .encode(format!("user:{}", self.token));
                (String::from("Authorization"), format!("Basic {}", basic))
            }
            LfsPlatform::Gitea => {
                // Gitea/Forgejo LFS uses HTTP Basic auth with:
                //   username: access token (or empty)
                //   password: (empty) — or token used as password
                let basic =
                    base64::engine::general_purpose::STANDARD.encode(format!("{}:", self.token));
                (String::from("Authorization"), format!("Basic {}", basic))
            }
        }
    }

    /// Compute SHA-256 hex digest of file contents (LFS OID).
    pub fn compute_oid(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Check if a local file should use LFS (over threshold, default 1MB).
    pub fn should_use_lfs(size_bytes: u64, threshold: u64) -> bool {
        size_bytes >= threshold
    }

    /// Create an LFS pointer file for a large object.
    pub fn create_pointer(oid: &str, size: u64) -> LfsPointer {
        LfsPointer {
            version: "https://git-lfs.github.com/spec/v1".to_string(),
            oid: oid.to_string(),
            size,
        }
    }

    /// Upload a file via Git LFS batch API.
    /// Returns the LFS OID (SHA-256) of the uploaded content.
    pub fn upload_via_lfs(&self, local_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read file for LFS: {}", e))?;
        let oid = Self::compute_oid(&data);
        let size = data.len() as u64;

        // 1. Send batch request to get upload URL
        let batch_url = self.batch_url();
        let (auth_name, auth_val) = self.lfs_auth_header();

        let batch_req = LfsBatchRequest {
            operation: "upload".to_string(),
            transfers: vec!["basic".to_string()],
            objects: vec![LfsObject {
                oid: oid.clone(),
                size,
                authenticated: None,
            }],
            hash_algo: None,
        };

        let client = http_client()?;
        let resp = client
            .post(&batch_url)
            .header(&auth_name, &auth_val)
            .header("Accept", "application/vnd.git-lfs+json")
            .header("Content-Type", "application/vnd.git-lfs+json")
            .json(&batch_req)
            .send()
            .map_err(|e| format!("LFS batch request: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(format!("LFS batch request failed ({}): {}", status, body));
        }

        let batch_resp: LfsBatchResponse = resp
            .json()
            .map_err(|e| format!("parse LFS batch response: {}", e))?;

        // 2. Find our object in the response
        let obj = batch_resp
            .objects
            .into_iter()
            .find(|o| o.oid == oid)
            .ok_or_else(|| format!("LFS object {} not found in batch response", oid))?;

        // 3. Check for errors
        if let Some(err) = &obj.error {
            return Err(format!("LFS error for {}: {}", oid, err.message));
        }

        // 4. If already exists on server, we're done
        if obj.actions.is_none() {
            return Ok(oid);
        }

        // 5. Upload to the action URL using headers from the batch response
        let upload = obj
            .actions
            .as_ref()
            .and_then(|a| a.upload.as_ref())
            .ok_or_else(|| format!("No upload action for LFS object {}", oid))?;

        let upload_resp = client
            .put(&upload.href)
            .headers({
                let mut headers = reqwest::header::HeaderMap::new();
                if let Some(h) = &upload.header {
                    for (k, v) in h {
                        if let (Ok(name), Ok(val)) = (
                            reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                            reqwest::header::HeaderValue::from_str(v),
                        ) {
                            headers.insert(name, val);
                        }
                    }
                }
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/octet-stream"),
                );
                headers
            })
            .body(data)
            .send()
            .map_err(|e| format!("LFS upload to {}: {}", upload.href, e))?;

        if !upload_resp.status().is_success() {
            return Err(format!("LFS upload failed: HTTP {}", upload_resp.status()));
        }

        Ok(oid)
    }

    /// Download a file via Git LFS batch API.
    pub fn download_via_lfs(&self, oid: &str, size: u64, local_path: &str) -> Result<(), String> {
        let batch_url = self.batch_url();
        let (auth_name, auth_val) = self.lfs_auth_header();

        let batch_req = LfsBatchRequest {
            operation: "download".to_string(),
            transfers: vec!["basic".to_string()],
            objects: vec![LfsObject {
                oid: oid.to_string(),
                size,
                authenticated: None,
            }],
            hash_algo: None,
        };

        let client = http_client()?;
        let resp = client
            .post(&batch_url)
            .header(&auth_name, &auth_val)
            .header("Accept", "application/vnd.git-lfs+json")
            .header("Content-Type", "application/vnd.git-lfs+json")
            .json(&batch_req)
            .send()
            .map_err(|e| format!("LFS download batch request: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!(
                "LFS download batch failed ({}): {}",
                resp.status(),
                body
            ));
        }

        let batch_resp: LfsBatchResponse = resp
            .json()
            .map_err(|e| format!("parse LFS batch response: {}", e))?;

        let obj = batch_resp
            .objects
            .into_iter()
            .find(|o| o.oid == oid)
            .ok_or_else(|| format!("LFS object {} not found", oid))?;

        if let Some(err) = &obj.error {
            return Err(format!("LFS download error: {}", err.message));
        }

        let download = obj
            .actions
            .as_ref()
            .and_then(|a| a.download.as_ref())
            .ok_or_else(|| format!("No download action for LFS object {}", oid))?;

        let download_resp = client
            .get(&download.href)
            .headers({
                let mut headers = reqwest::header::HeaderMap::new();
                if let Some(h) = &download.header {
                    for (k, v) in h {
                        if let (Ok(name), Ok(val)) = (
                            reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                            reqwest::header::HeaderValue::from_str(v),
                        ) {
                            headers.insert(name, val);
                        }
                    }
                }
                headers
            })
            .send()
            .map_err(|e| format!("LFS download from {}: {}", download.href, e))?;

        if !download_resp.status().is_success() {
            return Err(format!(
                "LFS download failed: HTTP {}",
                download_resp.status()
            ));
        }

        let data = download_resp
            .bytes()
            .map_err(|e| format!("read LFS download body: {}", e))?;

        // Verify OID
        let actual_oid = Self::compute_oid(&data);
        if actual_oid != oid {
            return Err(format!(
                "LFS OID mismatch: expected {}, got {}",
                oid, actual_oid
            ));
        }

        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &data).map_err(|e| format!("write: {}", e))?;

        Ok(())
    }

    /// Check if bytes are an LFS pointer file (starts with "version ").
    pub fn is_lfs_pointer(data: &[u8]) -> bool {
        data.starts_with(b"version ")
    }

    /// Generate `.gitattributes` content for tracking encrypted/compressed files via LFS.
    pub fn generate_gitattributes() -> String {
        r#"# Cybermanju Drive — Git LFS tracking rules
# Auto-generated — do not edit manually

# Portable database
*.cybermanju filter=lfs diff=lfs merge=lfs -text

# Compressed blobs (LZ4 → Zstd → Brotli)
*.cyb3 filter=lfs diff=lfs merge=lfs -text

# Encrypted files
*.enc filter=lfs diff=lfs merge=lfs -text

# Encryption metadata
*.enc.meta.json filter=lfs diff=lfs merge=lfs -text

# Preview thumbnails
*.preview.png filter=lfs diff=lfs merge=lfs -text
*.preview.jpg filter=lfs diff=lfs merge=lfs -text
*.preview.webp filter=lfs diff=lfs merge=lfs -text

# Preview directory
.previews/** filter=lfs diff=lfs merge=lfs -text

# Blob storage directory
.blobs/** filter=lfs diff=lfs merge=lfs -text
"#
        .to_string()
    }
}
