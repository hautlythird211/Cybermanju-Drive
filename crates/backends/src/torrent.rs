use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use librqbit::{
    AddTorrent, AddTorrentOptions, CreateTorrentOptions, Session, SessionOptions,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct TorrentBackend {
    save_dir: PathBuf,
    seed_port: u16,
    tracker_url: Option<String>,
    rt: Runtime,
    session: Arc<Session>,
    uploaded: std::sync::Mutex<Vec<(String, String)>>,
}

impl TorrentBackend {
    pub fn new(save_dir: PathBuf, seed_port: u16, tracker_url: Option<String>) -> Self {
        std::fs::create_dir_all(&save_dir).ok();

        let rt = Runtime::new().expect("failed to create tokio runtime for torrent backend");

        let session_opts = SessionOptions {
            dht: Some(librqbit::DhtSessionConfig {
                port: Some(seed_port),
                ..Default::default()
            }),
            listen: Some(librqbit::ListenerOptions {
                listen_addr: (std::net::Ipv4Addr::UNSPECIFIED, seed_port).into(),
                enable_upnp_port_forwarding: true,
                ..Default::default()
            }),
            disable_local_service_discovery: false,
            ..Default::default()
        };

        let session = rt.block_on(async {
            Session::new_with_opts(save_dir.clone(), session_opts)
                .await
                .expect("failed to create librqbit session")
        });

        Self {
            save_dir,
            seed_port,
            tracker_url,
            rt,
            session,
            uploaded: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn create_torrent_from_file(&self, file_path: &str) -> Result<(Vec<u8>, String), String> {
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            return Err(format!("file not found: {}", file_path));
        }

        let torrent_bytes = self.rt.block_on(async {
            let mut opts = CreateTorrentOptions::default();
            opts.name = path.file_name().and_then(|n| n.to_str());
            if let Some(ref tracker) = self.tracker_url {
                opts.trackers = vec![tracker.clone()];
            }
            opts.piece_length = Some(256 * 1024);

            let spawner = librqbit::spawn_utils::BlockingSpawner::new(4);

            librqbit::create_torrent(path, opts, &spawner)
                .await
                .map(|r| r.as_bytes().map(|b| b.to_vec()))
                .map_err(|e| format!("create torrent: {}", e))
                .and_then(|r| r.map_err(|e| format!("serialize torrent: {}", e)))
        })?;

        let info_hash = {
            let meta = librqbit::torrent_from_bytes(&torrent_bytes)
                .map_err(|e| format!("parse torrent: {}", e))?;
            meta.info_hash.as_string()
        };

        let torrent_path = self.save_dir.join(format!("{}.torrent", &info_hash[..16]));
        std::fs::write(&torrent_path, &torrent_bytes)
            .map_err(|e| format!("write torrent: {}", e))?;

        let magnet = format!("magnet:?xt=urn:btih:{}", info_hash);
        Ok((torrent_bytes, magnet))
    }

    pub fn create_torrent_from_data(
        &self,
        data: &[u8],
        _name: &str,
    ) -> Result<(Vec<u8>, String), String> {
        let temp_path = self.save_dir.join(format!("tmp_{}.dat", &blake3::hash(data).to_hex()[..16]));
        std::fs::write(&temp_path, data)
            .map_err(|e| format!("write temp file: {}", e))?;

        let result = self.create_torrent_from_file(temp_path.to_str().unwrap_or(""));
        std::fs::remove_file(&temp_path).ok();
        result
    }

    pub fn start_seeding(&self, torrent_bytes: &[u8]) -> Result<String, String> {
        let session = self.session.clone();
        let torrent_data = torrent_bytes.to_vec();

        self.rt.block_on(async move {
            let response = session
                .add_torrent(
                    AddTorrent::from_bytes(torrent_data),
                    Some(AddTorrentOptions {
                        overwrite: true,
                        ..Default::default()
                    }),
                )
                .await
                .map_err(|e| format!("add torrent: {}", e))?;

            let handle = response
                .into_handle()
                .ok_or("failed to get torrent handle")?;

            let info_hash = handle.info_hash().as_string();

            let handle_clone = handle.clone();
            tokio::spawn(async move {
                handle_clone.wait_until_completed().await.ok();
            });

            Ok(format!("magnet:?xt=urn:btih:{}", info_hash))
        })
    }

    pub fn download_from_magnet(
        &self,
        magnet_link: &str,
        local_path: &str,
    ) -> Result<String, String> {
        let session = self.session.clone();
        let magnet = magnet_link.to_string();
        let dest = local_path.to_string();

        self.rt.block_on(async move {
            let response = session
                .add_torrent(
                    AddTorrent::from_url(&magnet),
                    Some(AddTorrentOptions {
                        overwrite: true,
                        ..Default::default()
                    }),
                )
                .await
                .map_err(|e| format!("add torrent: {}", e))?;

            let handle = response
                .into_handle()
                .ok_or("failed to get torrent handle")?;

            let info_hash = handle.info_hash().as_string();

            handle.wait_until_completed().await
                .map_err(|e| format!("download failed: {}", e))?;

            let dest_path = std::path::Path::new(&dest);
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let stats = handle.stats();
            if stats.finished {
                let save_folder = std::path::Path::new("/tmp/cybermanju_torrent_download");
                std::fs::create_dir_all(save_folder).ok();
            }

            Ok(info_hash)
        })
    }

    pub fn get_torrent_stats(&self, info_hash: &str) -> Result<TorrentStats, String> {
        let session = self.session.clone();
        let hash = info_hash.to_string();

        self.rt.block_on(async {
            let result = session.with_torrents(|iter| {
                for (_id, handle) in iter {
                    if handle.info_hash().as_string() == hash {
                        let stats = handle.stats();
                        return Some(TorrentStats {
                            info_hash: hash.clone(),
                            state: format!("{:?}", stats.state),
                            progress_bytes: stats.progress_bytes,
                            total_bytes: stats.total_bytes,
                            upload_speed: stats.live.as_ref().map(|l| l.upload_speed.as_bytes()).unwrap_or(0),
                            download_speed: stats.live.as_ref().map(|l| l.download_speed.as_bytes()).unwrap_or(0),
                        });
                    }
                }
                None
            });
            result.ok_or_else(|| format!("torrent {} not found", &hash[..16.min(hash.len())]))
        })
    }

    pub fn list_active_torrents(&self) -> Vec<TorrentInfo> {
        let session = self.session.clone();

        self.rt.block_on(async {
            session.with_torrents(|iter| {
                iter.map(|(_id, handle)| {
                    let info_hash = handle.info_hash().as_string();
                    let name = handle.name().unwrap_or_default();
                    TorrentInfo {
                        info_hash: info_hash.clone(),
                        name,
                        magnet: format!("magnet:?xt=urn:btih:{}", info_hash),
                    }
                })
                .collect()
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct TorrentStats {
    pub info_hash: String,
    pub state: String,
    pub progress_bytes: u64,
    pub total_bytes: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
}

#[derive(Debug, Clone)]
pub struct TorrentInfo {
    pub info_hash: String,
    pub name: String,
    pub magnet: String,
}

impl StorageBackend for TorrentBackend {
    fn name(&self) -> &str {
        "torrent"
    }

    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::Torrent
    }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let (torrent_bytes, magnet) = self.create_torrent_from_file(local_path)?;

        let info_hash = magnet
            .strip_prefix("magnet:?xt=urn:btih:")
            .unwrap_or("")
            .to_string();
        self.uploaded.lock().unwrap()
            .push((remote_path.to_string(), info_hash));

        self.start_seeding(&torrent_bytes)?;

        log::info!("Torrent created and seeding: {}", &magnet[..64.min(magnet.len())]);
        Ok(magnet)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let magnet = if remote_path.starts_with("magnet:") {
            remote_path.to_string()
        } else {
            format!("magnet:?xt=urn:btih:{}", remote_path)
        };
        self.download_from_magnet(&magnet, local_path)?;
        Ok(())
    }

    fn delete_file(&self, _remote_path: &str) -> Result<(), String> {
        Ok(())
    }

    fn list_files(&self, _prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let torrents = self.list_active_torrents();
        Ok(torrents
            .into_iter()
            .map(|t| RemoteFile {
                name: t.name,
                path: t.magnet.clone(),
                size_bytes: 0,
                modified_at: String::new(),
                url: t.magnet,
            })
            .collect())
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        if remote_path.starts_with("magnet:") {
            Ok(remote_path.to_string())
        } else {
            Ok(format!("magnet:?xt=urn:btih:{}", remote_path))
        }
    }

    fn test_connection(&self) -> Result<bool, String> {
        let session = self.session.clone();
        self.rt.block_on(async {
            let count = session.with_torrents(|iter| iter.count());
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_torrent_backend_creation() {
        let dir = PathBuf::from("/tmp/cybermanju_torrent_test_cr");
        let backend = TorrentBackend::new(dir, 0, None);
        assert_eq!(backend.name(), "torrent");
        assert_eq!(backend.backend_type(), SyncBackendType::Torrent);
    }

    #[test]
    fn test_create_torrent_from_data() {
        let dir = PathBuf::from("/tmp/cybermanju_torrent_test_cd");
        let backend = TorrentBackend::new(dir, 0, None);

        let data = b"Hello, Cybermanju torrent test!";
        let (torrent_bytes, magnet) = backend
            .create_torrent_from_data(data, "test.txt")
            .unwrap();

        assert!(!torrent_bytes.is_empty());
        assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
        assert!(magnet.len() > 40);
    }

    #[test]
    fn test_create_torrent_from_file() {
        let dir = PathBuf::from("/tmp/cybermanju_torrent_test_cf");
        std::fs::create_dir_all(&dir).ok();
        let backend = TorrentBackend::new(dir.clone(), 0, None);

        let file_path = dir.join("test_input.bin");
        let test_data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        std::fs::write(&file_path, &test_data).unwrap();

        let (torrent_bytes, magnet) = backend
            .create_torrent_from_file(file_path.to_str().unwrap())
            .unwrap();

        assert!(!torrent_bytes.is_empty());
        assert!(magnet.starts_with("magnet:?xt=urn:btih:"));

        let torrent_files: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "torrent").unwrap_or(false))
            .collect();
        assert!(!torrent_files.is_empty());
    }

    #[test]
    fn test_upload_and_list() {
        let dir = PathBuf::from("/tmp/cybermanju_torrent_test_ul");
        std::fs::create_dir_all(&dir).ok();
        let backend = TorrentBackend::new(dir.clone(), 0, None);

        let file_path = dir.join("upload_test.bin");
        let test_data = b"upload test data";
        std::fs::write(&file_path, test_data).unwrap();

        let magnet = backend
            .upload_file(file_path.to_str().unwrap(), "upload_test.bin")
            .unwrap();
        assert!(magnet.starts_with("magnet:?xt=urn:btih:"));

        let files = backend.list_files("").unwrap();
        assert!(!files.is_empty());
    }

    #[test]
    fn test_get_file_url() {
        let dir = PathBuf::from("/tmp/cybermanju_torrent_test_gu");
        let backend = TorrentBackend::new(dir, 0, None);

        let url = backend.get_file_url("abc123").unwrap();
        assert!(url.starts_with("magnet:?xt=urn:btih:abc123"));

        let magnet = "magnet:?xt=urn:btih:abc123";
        let url = backend.get_file_url(magnet).unwrap();
        assert_eq!(url, magnet);
    }

    #[test]
    fn test_with_tracker() {
        let dir = PathBuf::from("/tmp/cybermanju_torrent_test_wt");
        let backend = TorrentBackend::new(
            dir,
            0,
            Some("udp://tracker.opentrackr.org:1337/announce".to_string()),
        );

        let data = b"tracker test";
        let (_, magnet) = backend
            .create_torrent_from_data(data, "tracker_test.txt")
            .unwrap();
        assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
    }
}
