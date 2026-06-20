use crate::util::http_client;
use base64::Engine;
use cybermanju_types::sync::RepoLayout;
use sha2::{Digest, Sha256};

/// Manages the .cybermanju repository layout on git-based backends.
/// Supports flat, sharded (by hash prefix), and split-repo layouts.
pub struct RepoLayoutManager {
    pub layout: RepoLayout,
    /// LFS blob repo (for split layout), e.g. "owner/cybermanju-blobs"
    pub blob_repo: Option<String>,
    /// Repo identifier for the main repo
    pub main_repo: String,
    /// Branch name
    pub branch: String,
}

impl RepoLayoutManager {
    pub fn new(
        layout: RepoLayout,
        main_repo: &str,
        blob_repo: Option<String>,
        branch: &str,
    ) -> Self {
        Self {
            layout,
            blob_repo,
            main_repo: main_repo.to_string(),
            branch: branch.to_string(),
        }
    }

    /// Compute the remote path for a file based on the repo layout.
    pub fn compute_remote_path(
        &self,
        logical_path: &str,
        file_hash: Option<&str>,
        is_blob: bool,
    ) -> String {
        let hash = file_hash.map(|h| h.to_string()).unwrap_or_else(|| {
            let mut h = Sha256::new();
            h.update(logical_path.as_bytes());
            format!("{:x}", h.finalize())
        });
        match self.layout {
            RepoLayout::Flat => {
                if is_blob {
                    format!(".blobs/{}", logical_path)
                } else {
                    logical_path.to_string()
                }
            }
            RepoLayout::Sharded => {
                if is_blob {
                    // blobs/{first 2 hex chars}/{next 2 hex chars}/{hash}.cyb3
                    format!(".blobs/{}/{}/{}.cyb3", &hash[..2], &hash[2..4], hash)
                } else {
                    // Metadata files are still flat under meta/
                    format!("meta/{}", logical_path)
                }
            }
            RepoLayout::Split => {
                // In split layout, blobs go to the blob_repo, metadata to main
                if is_blob {
                    // Path for blob repo (sharded by default in blob repo)
                    format!("{}/{}/{}.cyb3", &hash[..2], &hash[2..4], hash)
                } else {
                    format!(".cybermanju/{}", logical_path)
                }
            }
        }
    }

    /// Determine which repo (main or blob) a path belongs to in split layout.
    pub fn get_repo_for_path(&self, _remote_path: &str, is_blob: bool) -> &str {
        if self.layout == RepoLayout::Split && is_blob {
            self.blob_repo.as_deref().unwrap_or(&self.main_repo)
        } else {
            &self.main_repo
        }
    }

    /// Generate the .gitattributes content for the repo layout.
    pub fn generate_gitattributes(&self) -> String {
        let mut attrs = String::new();
        attrs.push_str("# Cybermanju Drive — Git LFS tracking rules\n");
        attrs.push_str("# Auto-generated — do not edit manually\n\n");

        match self.layout {
            RepoLayout::Flat => {
                attrs.push_str("# All blobs tracked via LFS\n");
                attrs.push_str(".blobs/** filter=lfs diff=lfs merge=lfs -text\n");
                attrs.push_str("*.cyb3 filter=lfs diff=lfs merge=lfs -text\n");
                attrs.push_str("*.enc filter=lfs diff=lfs merge=lfs -text\n");
                attrs.push_str("*.preview.* filter=lfs diff=lfs merge=lfs -text\n");
                attrs.push_str("*.cybermanju filter=lfs diff=lfs merge=lfs -text\n");
            }
            RepoLayout::Sharded => {
                attrs.push_str("# Blob storage (hash-sharded)\n");
                attrs.push_str(".blobs/** filter=lfs diff=lfs merge=lfs -text\n");
                attrs.push_str("# Encrypted files\n");
                attrs.push_str("*.enc.* filter=lfs diff=lfs merge=lfs -text\n");
                attrs.push_str("# Preview images\n");
                attrs.push_str(".previews/** filter=lfs diff=lfs merge=lfs -text\n");
                attrs.push_str("# Portable database\n");
                attrs.push_str("*.cybermanju filter=lfs diff=lfs merge=lfs -text\n");
            }
            RepoLayout::Split => {
                attrs.push_str("# Main repo — metadata only (small files)\n");
                attrs.push_str("# .cybermanju directory contains portable DB and manifest\n");
                attrs.push_str(".cybermanju/** filter=lfs diff=lfs merge=lfs -text\n");

                if let Some(blob_repo) = &self.blob_repo {
                    attrs.push_str(&format!(
                        "\n# Blobs are stored in a separate repo: {}\n",
                        blob_repo
                    ));
                    attrs.push_str(
                        "# That repo should have: * filter=lfs diff=lfs merge=lfs -text\n",
                    );
                    attrs.push_str(".gitmodules\n");
                }
            }
        }

        attrs
    }

    /// Ensure the remote repo has a .gitattributes file with proper LFS config.
    /// This is a no-op for non-git backends; for git providers it checks/creates the file.
    pub fn ensure_gitattributes(
        &self,
        backend_type: &str,
        token: &str,
        api_base: &str,
    ) -> Result<(), String> {
        let attrs_content = self.generate_gitattributes();
        let remote_path = ".gitattributes";

        match backend_type {
            "github" => {
                let (owner, repo) = self.parse_repo(&self.main_repo)?;
                let url = format!(
                    "{}/repos/{}/{}/contents/{}",
                    api_base.trim_end_matches('/'),
                    owner,
                    repo,
                    remote_path
                );
                let client = http_client()?;

                // Check if .gitattributes exists
                let check = client
                    .get(&url)
                    .header("Authorization", format!("token {}", token))
                    .header("Accept", "application/vnd.github+json")
                    .send()
                    .map_err(|e| format!("check .gitattributes: {}", e))?;

                let body = serde_json::json!({
                    "message": "chore: add Cybermanju Drive LFS tracking rules",
                    "content": base64::engine::general_purpose::STANDARD.encode(attrs_content.as_bytes()),
                    "branch": self.branch,
                });

                if check.status().as_u16() == 404 {
                    // Create new file
                    let resp = client
                        .put(&url)
                        .header("Authorization", format!("token {}", token))
                        .header("Accept", "application/vnd.github+json")
                        .json(&body)
                        .send()
                        .map_err(|e| format!("create .gitattributes: {}", e))?;
                    if !resp.status().is_success() {
                        return Err(format!(
                            "Failed to create .gitattributes: HTTP {}",
                            resp.status()
                        ));
                    }
                }
                // File exists — skip update to avoid overwriting user modifications
                // In the future we could check SHA and optionally update
            }
            "gitlab" | "codeberg" | "gitea" => {
                let encoded_path = urlencoding::encode(remote_path);
                let url = format!(
                    "{}/api/v4/projects/{}/repository/files/{}",
                    api_base.trim_end_matches('/'),
                    self.main_repo,
                    encoded_path
                );
                let client = http_client()?;

                let body = serde_json::json!({
                    "branch": self.branch,
                    "content": attrs_content,
                    "encoding": "text",
                    "commit_message": "chore: add Cybermanju Drive LFS tracking rules",
                });

                let check = client.head(&url).header("PRIVATE-TOKEN", token).send();

                if check
                    .ok()
                    .map(|r| r.status().as_u16() == 404)
                    .unwrap_or(true)
                {
                    let resp = client
                        .post(&url)
                        .header("PRIVATE-TOKEN", token)
                        .json(&body)
                        .send()
                        .map_err(|e| format!("create .gitattributes: {}", e))?;
                    if !resp.status().is_success() && resp.status().as_u16() != 201 {
                        let s = resp.status();
                        return Err(format!("Failed to create .gitattributes: HTTP {}", s));
                    }
                }
            }
            _ => {} // Non-git backends don't need .gitattributes
        }

        Ok(())
    }

    fn parse_repo(&self, repo: &str) -> Result<(String, String), String> {
        let parts: Vec<&str> = repo.trim_start_matches('/').splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!("Invalid repo '{}', need owner/repo", repo));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}
