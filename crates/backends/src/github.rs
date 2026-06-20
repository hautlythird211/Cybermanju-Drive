use crate::git_lfs::{GitLfsClient, LfsPlatform};
use crate::repo_layout::RepoLayoutManager;
use crate::util::http_client;
use base64::Engine;
use cybermanju_types::sync::{RemoteFile, RepoLayout, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;

/// LFS threshold: files >= 1MB use Git LFS
const LFS_SIZE_THRESHOLD: u64 = 1_048_576;
/// GitHub Contents API cannot handle files > 100 MB
const CONTENTS_API_MAX: u64 = 100 * 1_048_576;
/// GitHub Releases API supports up to 2 GiB per asset
const RELEASES_API_MAX: u64 = 2 * 1_073_741_824;

pub struct GitHubBackend {
    token: String,
    repo: String,
    branch: String,
    use_lfs: bool,
    layout: RepoLayout,
    lfs_repo: Option<String>,
}

impl GitHubBackend {
    pub fn new(token: &str, repo: &str, branch: &str) -> Self {
        Self {
            token: token.to_string(),
            repo: repo.to_string(),
            branch: branch.to_string(),
            use_lfs: false,
            layout: RepoLayout::Flat,
            lfs_repo: None,
        }
    }

    pub fn with_lfs(mut self, use_lfs: bool) -> Self {
        self.use_lfs = use_lfs;
        self
    }

    pub fn with_layout(mut self, layout: RepoLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_lfs_repo(mut self, lfs_repo: Option<String>) -> Self {
        self.lfs_repo = lfs_repo;
        self
    }

    fn parse_repo(&self) -> Result<(String, String), String> {
        let parts: Vec<&str> = self.repo.trim_start_matches('/').splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!("Invalid repo '{}', need owner/repo", self.repo));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    fn layout_mgr(&self) -> RepoLayoutManager {
        RepoLayoutManager::new(
            self.layout.clone(),
            &self.repo,
            self.lfs_repo.clone(),
            &self.branch,
        )
    }

    fn should_use_lfs(&self, size: u64) -> bool {
        self.use_lfs && (LFS_SIZE_THRESHOLD..=RELEASES_API_MAX).contains(&size)
    }

    /// Upload via Contents API (for files < 100 MB).
    fn upload_via_contents_api(&self, data: &[u8], path: &str) -> Result<String, String> {
        let (owner, repo) = self.parse_repo()?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(data);
        let body = serde_json::json!({
            "message": format!("Upload: {}", path),
            "content": b64,
            "branch": self.branch,
        });
        let client = http_client()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );
        let resp = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&body)
            .send()
            .map_err(|e| format!("request: {}", e))?;
        let status = resp.status().as_u16();
        let body_text = resp.text().unwrap_or_default();
        if !(200..300).contains(&status) {
            return Err(format!("GitHub upload ({}): {}", status, body_text));
        }
        let v: serde_json::Value =
            serde_json::from_str(&body_text).map_err(|e| format!("parse: {}", e))?;
        Ok(v["content"]["download_url"].as_str().unwrap_or("").into())
    }

    /// Upload via GitHub Releases API (fallback for files > 100 MB).
    /// Creates a release tagged with the file path and uploads as a release asset.
    fn upload_via_releases_api(&self, data: &[u8], path: &str) -> Result<String, String> {
        let (owner, repo) = self.parse_repo()?;
        let client = http_client()?;
        let tag = format!("cybermanju-blob/{}", path.replace('/', "-"));

        // 1. Create a release
        let release_body = serde_json::json!({
            "tag_name": tag,
            "name": format!("Blob: {}", path),
            "body": format!("Cybermanju Drive encrypted/compressed blob: {}", path),
            "draft": false,
            "prerelease": false,
        });

        let release_resp = client
            .post(format!(
                "https://api.github.com/repos/{}/{}/releases",
                owner, repo
            ))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&release_body)
            .send()
            .map_err(|e| format!("create release: {}", e))?;

        if !release_resp.status().is_success() {
            let status = release_resp.status();
            let body = release_resp.text().unwrap_or_default();
            return Err(format!("GitHub release creation ({}): {}", status, body));
        }

        let release_json: serde_json::Value = release_resp
            .json()
            .map_err(|e| format!("parse release: {}", e))?;

        let upload_url_tpl = release_json["upload_url"]
            .as_str()
            .ok_or("no upload_url")?
            .to_string();

        // 2. Replace template variables in upload_url
        // upload_url looks like: https://uploads.github.com/.../releases/123/assets{?name,label}
        let upload_url = upload_url_tpl.replace("{?name,label}", "");
        let file_name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| std::borrow::Cow::Borrowed("blob"));

        // 3. Upload the asset
        let asset_resp = client
            .post(format!("{}?name={}", upload_url, file_name))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .map_err(|e| format!("upload asset: {}", e))?;

        if !asset_resp.status().is_success() {
            let status = asset_resp.status();
            let body = asset_resp.text().unwrap_or_default();
            return Err(format!("GitHub asset upload ({}): {}", status, body));
        }

        let asset_json: serde_json::Value = asset_resp
            .json()
            .map_err(|e| format!("parse asset: {}", e))?;

        Ok(asset_json["browser_download_url"]
            .as_str()
            .unwrap_or("")
            .into())
    }
}

impl StorageBackend for GitHubBackend {
    fn name(&self) -> &str {
        "GitHub"
    }
    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::GitHub
    }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let file_size = data.len() as u64;

        let layout_mgr = self.layout_mgr();
        let hash = GitLfsClient::compute_oid(&data);
        let effective_path =
            layout_mgr.compute_remote_path(remote_path, Some(&hash), file_size > 1024);

        // Strategy: LFS > Contents API > Releases API
        if self.should_use_lfs(file_size) {
            let lfs = GitLfsClient::new(
                "https://github.com",
                &self.repo,
                &self.token,
                LfsPlatform::GitHub,
            );
            let oid = lfs.upload_via_lfs(local_path)?;
            let pointer = GitLfsClient::create_pointer(&oid, file_size);
            let pointer_content = pointer.to_string();
            return self.upload_via_contents_api(pointer_content.as_bytes(), &effective_path);
        }

        if file_size <= CONTENTS_API_MAX {
            return self.upload_via_contents_api(&data, &effective_path);
        }

        // Fallback to Releases API for files that are too large for Contents API
        // and LFS is unavailable or disabled
        self.upload_via_releases_api(&data, &effective_path)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, remote_path
        );
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitHub download: HTTP {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().map_err(|e| format!("parse: {}", e))?;
        let content = v["content"].as_str().ok_or("no content")?;
        let clean: String = content.chars().filter(|c| *c != '\n').collect();
        let data = base64::engine::general_purpose::STANDARD
            .decode(&clean)
            .map_err(|e| format!("b64: {}", e))?;

        // Check if it's an LFS pointer file
        if self.use_lfs && GitLfsClient::is_lfs_pointer(&data) {
            let pointer_str = String::from_utf8_lossy(&data);
            let pointer = cybermanju_types::sync::LfsPointer::from_string(&pointer_str)?;

            let lfs = GitLfsClient::new(
                "https://github.com",
                &self.repo,
                &self.token,
                LfsPlatform::GitHub,
            );
            return lfs.download_via_lfs(&pointer.oid, pointer.size, local_path);
        }

        if let Some(p) = Path::new(local_path).parent() {
            fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(local_path, &data).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, remote_path
        );
        let client = http_client()?;
        let get = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|e| format!("get: {}", e))?;
        if get.status().as_u16() == 404 {
            return Ok(());
        }
        let v: serde_json::Value = get.json().map_err(|e| format!("parse: {}", e))?;
        let sha = v["sha"].as_str().ok_or("no sha")?;
        let body = serde_json::json!({
            "message": format!("Delete: {}", remote_path),
            "sha": sha,
            "branch": self.branch,
        });
        let del = client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .map_err(|e| format!("delete: {}", e))?;
        if !del.status().is_success() {
            return Err(format!("GitHub delete: HTTP {}", del.status()));
        }
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let (owner, repo) = self.parse_repo()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, prefix
        );
        let client = http_client()?;
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|e| format!("request: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("GitHub list: HTTP {}", resp.status()));
        }
        let items: Vec<serde_json::Value> = resp.json().map_err(|e| format!("parse: {}", e))?;
        let mut files = vec![];
        for item in &items {
            if item["type"].as_str() == Some("dir") {
                continue;
            }
            files.push(RemoteFile {
                name: item["name"].as_str().unwrap_or("?").into(),
                path: item["path"].as_str().unwrap_or("?").into(),
                size_bytes: item["size"].as_u64().unwrap_or(0),
                modified_at: item
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                url: item["download_url"].as_str().unwrap_or("").into(),
            });
        }
        Ok(files)
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let (owner, repo) = self.parse_repo()?;
        Ok(format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, self.branch, remote_path
        ))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let client = http_client()?;
        let resp = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(|e| format!("request: {}", e))?;
        Ok(resp.status().is_success())
    }
}
