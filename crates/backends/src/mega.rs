use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use std::fs;
use std::path::Path;

/// Mega.nz cloud storage backend.
/// Uses the `megalib` crate for all Mega API operations.
/// Token format: `email|password`
pub struct MegaBackend {
    email: String,
    password: String,
    rt: tokio::runtime::Runtime,
}

impl MegaBackend {
    pub fn new(email: &str, password: &str) -> Result<Self, String> {
        if email.is_empty() || password.is_empty() {
            return Err("Mega backend requires email and password".to_string());
        }
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
        Ok(Self {
            email: email.to_string(),
            password: password.to_string(),
            rt,
        })
    }

    pub fn new_with_email_password(email: &str, password: &str) -> Result<Self, String> {
        Self::new(email, password)
    }

    async fn login_inner(&self) -> Result<megalib::SessionHandle, String> {
        let session = megalib::SessionHandle::login(&self.email, &self.password)
            .await
            .map_err(|e| format!("Mega login failed: {}", e))?;
        session
            .refresh()
            .await
            .map_err(|e| format!("Mega refresh failed: {}", e))?;
        Ok(session)
    }

    fn remote_parent(&self, remote_path: &str) -> String {
        if let Some(pos) = remote_path.rfind('/') {
            if pos == 0 {
                "/Root".to_string()
            } else {
                remote_path[..pos].to_string()
            }
        } else {
            "/Root".to_string()
        }
    }

    fn remote_name(&self, remote_path: &str) -> String {
        remote_path
            .rsplit('/')
            .next()
            .unwrap_or(remote_path)
            .to_string()
    }
}

impl StorageBackend for MegaBackend {
    fn name(&self) -> &str { "Mega" }
    fn backend_type(&self) -> SyncBackendType { SyncBackendType::Mega }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let data = fs::read(local_path).map_err(|e| format!("read: {}", e))?;
        let parent = self.remote_parent(remote_path);
        let name = self.remote_name(remote_path);

        self.rt.block_on(async {
            let session = self.login_inner().await?;
            let path = megalib::Path::new("/Root")
                .join(&parent)
                .map_err(|e| format!("path: {}", e))?;
            let folder = session
                .ensure_folder(&path)
                .await
                .map_err(|e| format!("ensure folder: {}", e))?;
            folder
                .upload(data, &name)
                .await
                .map_err(|e| format!("upload: {}", e))?;
            Ok(format!("mega://{}/{}", parent, name))
        })
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let name = self.remote_name(remote_path);

        self.rt.block_on(async {
            let session = self.login_inner().await?;
            let path = megalib::Path::new("/Root");
            let root = session
                .find_folder(&path)
                .await
                .map_err(|e| format!("find root: {}", e))?;
            let nodes = root
                .get_children()
                .await
                .map_err(|e| format!("list children: {}", e))?;
            let node = nodes
                .into_iter()
                .find(|n| n.name() == name)
                .ok_or_else(|| format!("file not found: {}", name))?;
            let data = node
                .download()
                .await
                .map_err(|e| format!("download: {}", e))?;
            if let Some(p) = Path::new(local_path).parent() {
                fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?;
            }
            fs::write(local_path, &data).map_err(|e| format!("write: {}", e))?;
            Ok(())
        })
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let name = self.remote_name(remote_path);

        self.rt.block_on(async {
            let session = self.login_inner().await?;
            let path = megalib::Path::new("/Root");
            let root = session
                .find_folder(&path)
                .await
                .map_err(|e| format!("find root: {}", e))?;
            let nodes = root
                .get_children()
                .await
                .map_err(|e| format!("list children: {}", e))?;
            let node = nodes
                .into_iter()
                .find(|n| n.name() == name)
                .ok_or_else(|| format!("file not found: {}", name))?;
            node.delete().await.map_err(|e| format!("delete: {}", e))?;
            Ok(())
        })
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        self.rt.block_on(async {
            let session = self.login_inner().await?;
            let path = megalib::Path::new("/Root");
            let root = session
                .find_folder(&path)
                .await
                .map_err(|e| format!("find root: {}", e))?;
            let nodes = root
                .get_children()
                .await
                .map_err(|e| format!("list children: {}", e))?;
            Ok(nodes
                .into_iter()
                .filter(|n| n.node_type() == 0) // 0 = file
                .map(|n| RemoteFile {
                    name: n.name().to_string(),
                    path: n.name().to_string(),
                    size_bytes: n.size() as u64,
                    modified_at: String::new(),
                    url: format!("mega://{}", n.handle()),
                })
                .collect())
        })
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        let name = self.remote_name(remote_path);
        Ok(format!("mega://{}", name))
    }

    fn test_connection(&self) -> Result<bool, String> {
        self.rt.block_on(async {
            self.login_inner().await?;
            Ok(true)
        })
    }
}
