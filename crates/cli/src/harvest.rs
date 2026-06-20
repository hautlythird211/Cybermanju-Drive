use crate::backends::create_backend;
use crate::tui::{StoredBackend, TaskMessage};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

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
    run_harvest_with_output(backends, tx, None, 0);
}

pub fn run_harvest_with_output(backends: Vec<StoredBackend>, tx: Sender<TaskMessage>, output_path: Option<String>, seq: u64) {
    let total = backends.len();
    let mut completed = 0usize;
    let mut grand_files = 0usize;
    let mut grand_bytes = 0u64;
    let mut grand_errors = 0usize;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();

    for sb in &backends {
        let be = match create_backend(sb.backend_type, &sb.token, &sb.config) {
            Some(b) => b,
            None => {
                let _ = tx.send(TaskMessage::HarvestError(seq, sb.name.clone(), "failed to create backend".into()));
                completed += 1;
                continue;
            }
        };

        if let Err(e) = be.test_connection() {
            let _ = tx.send(TaskMessage::HarvestError(seq, sb.name.clone(), format!("connection: {}", e)));
            completed += 1;
            continue;
        }

        let _ = tx.send(TaskMessage::HarvestProgress(seq, sb.name.clone(), 0, 0, 0, "listing files...".into()));

        let files = match be.list_files("") {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(TaskMessage::HarvestError(seq, sb.name.clone(), format!("list: {}", e)));
                completed += 1;
                continue;
            }
        };

        let total_files = files.len();
        let _ = tx.send(TaskMessage::HarvestProgress(seq, sb.name.clone(), 0, total_files, 0, format!("found {} files", total_files)));

        let backend_dir = staging_dir().join(&sb.name);
        let _ = fs::create_dir_all(&backend_dir);

        let mut dl_bytes = 0u64;
        let mut dl_ok = 0usize;
        for f in &files {
            let local = backend_dir.join(&f.name);
            match be.download_file(&f.path, local.to_str().unwrap_or("")) {
                Ok(_) => {
                    dl_ok += 1;
                    dl_bytes += f.size_bytes;
                    let _ = tx.send(TaskMessage::HarvestProgress(seq, sb.name.clone(), dl_ok, total_files, dl_bytes, format!("{}/{} files", dl_ok, total_files)));
                }
                Err(e) => {
                    grand_errors += 1;
                    let _ = tx.send(TaskMessage::HarvestError(seq, sb.name.clone(), format!("download error for {}: {}", f.name, e)));
                }
            }
        }

        let _ = tx.send(TaskMessage::HarvestDone(seq, sb.name.clone(), dl_ok, dl_bytes));
        completed += 1;
        grand_files += dl_ok;
        grand_bytes += dl_bytes;
        let pct = completed as f64 / total as f64;
        let _ = tx.send(TaskMessage::HarvestOverall(seq, pct, format!("{}/{} backends done", completed, total)));
    }

    // Determine output path
    let harvest_dir = staging_dir();
    let pdb_path = match output_path {
        Some(p) => PathBuf::from(p),
        None => harvest_dir.join(format!("harvest-{}.cybermanju", stamp)),
    };
    let _ = tx.send(TaskMessage::HarvestOverall(seq, 0.95, "packing .cybermanju...".into()));

    // Create portable database with all harvested files as recovery entries
    if let Some(parent) = pdb_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
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
                    let _ = tx.send(TaskMessage::HarvestOverall(seq, 1.0, format!("db error: {}", e)));
                }
            }
        }
        Err(e) => {
            let _ = tx.send(TaskMessage::HarvestOverall(seq, 1.0, format!("portable error: {}", e)));
        }
    }

    let _ = tx.send(TaskMessage::HarvestOverall(seq, 1.0, format!("done — output: {}", pdb_path.display())));
    let _ = tx.send(TaskMessage::HarvestComplete(seq, grand_files, grand_bytes));
}
