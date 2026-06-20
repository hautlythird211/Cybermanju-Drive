use crate::backends::{create_backend, StoredBackend};
use crate::tui::TaskMessage;
use chrono::Utc;
use cybermanju_compression::TripleCompressor;
use cybermanju_crypto::{encrypt_data, KeyPair, PqcEngine, EncryptionAlgo};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Instant;

fn staging_dir() -> PathBuf {
    let d = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cybermanju-cli")
        .join("harvest")
        .join(Utc::now().format("%Y%m%d-%H%M%S").to_string());
    let _ = fs::create_dir_all(&d);
    d
}

pub fn run_harvest(backends: Vec<StoredBackend>, tx: Sender<TaskMessage>) {
    let total = backends.len();
    let mut completed = 0usize;
    let mut grand_bytes = 0u64;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();

    for sb in &backends {
        let be = match create_backend(sb.backend_type, &sb.token, &sb.config) {
            Some(b) => b,
            None => {
                let _ = tx.send(TaskMessage::HarvestError(sb.name.clone(), "failed to create backend".into()));
                completed += 1;
                continue;
            }
        };

        if let Err(e) = be.test_connection() {
            let _ = tx.send(TaskMessage::HarvestError(sb.name.clone(), format!("connection: {}", e)));
            completed += 1;
            continue;
        }

        let _ = tx.send(TaskMessage::HarvestProgress(sb.name.clone(), 0, 0, "listing files...".into()));

        let files = match be.list_files("") {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(TaskMessage::HarvestError(sb.name.clone(), format!("list: {}", e)));
                completed += 1;
                continue;
            }
        };

        let total_files = files.len();
        let _ = tx.send(TaskMessage::HarvestProgress(sb.name.clone(), 0, 0, format!("found {} files", total_files)));

        let backend_dir = staging_dir().join(&sb.name);
        let _ = fs::create_dir_all(&backend_dir);

        let mut dl_bytes = 0u64;
        for (i, f) in files.iter().enumerate() {
            let local = backend_dir.join(&f.name);
            let _ = be.download_file(&f.path, local.to_str().unwrap_or(""));
            dl_bytes += f.size_bytes;
            let _ = tx.send(TaskMessage::HarvestProgress(sb.name.clone(), i + 1, dl_bytes, format!("{}/{} files", i + 1, total_files)));
        }

        let _ = tx.send(TaskMessage::HarvestDone(sb.name.clone(), total_files, dl_bytes));
        completed += 1;
        grand_bytes += dl_bytes;
        let pct = completed as f64 / total as f64;
        let _ = tx.send(TaskMessage::HarvestOverall(pct, format!("{}/{} backends done", completed, total)));
    }

    // Package into .cybermanju
    let harvest_dir = staging_dir();
    let pdb_path = harvest_dir.join(format!("harvest-{}.cybermanju", stamp));
    let _ = tx.send(TaskMessage::HarvestOverall(0.95, "packing .cybermanju...".into()));

    // Create portable database with all harvested files as recovery entries
    match cybermanju_portable_db::PortableDatabase::create(pdb_path.to_str().unwrap(), "cli-harvest") {
        Ok(mut pdb) => {
            let redb_path = harvest_dir.join("tmp.db");
            match cybermanju_db::Database::new(redb_path.to_str().unwrap_or("")) {
                Ok(db) => {
                    for sb in &backends {
                        let backend_dir = harvest_dir.join(&sb.name);
                        if !backend_dir.exists() { continue; }
                        if let Ok(entries) = fs::read_dir(&backend_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_file() {
                                    if let Ok(data) = fs::read(&path) {
                                        let file_id = uuid::Uuid::new_v4().to_string();
                                        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                        let _ = pdb.store_compressed_content(&db, &file_id, &data, &name, None);
                                    }
                                }
                            }
                        }
                    }
                    let _ = pdb.repack(redb_path.to_str().unwrap(), None);
                    let _ = fs::remove_file(&redb_path);
                }
                Err(e) => {
                    let _ = tx.send(TaskMessage::HarvestOverall(1.0, format!("db error: {}", e)));
                }
            }
        }
        Err(e) => {
            let _ = tx.send(TaskMessage::HarvestOverall(1.0, format!("portable error: {}", e)));
        }
    }

    let _ = tx.send(TaskMessage::HarvestOverall(1.0, format!("done — output: {}", pdb_path.display())));
    let _ = tx.send(TaskMessage::HarvestComplete(0, grand_bytes));
}
