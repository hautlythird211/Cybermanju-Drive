use crate::tui::TaskMessage;
use cybermanju_portable_db::PortableDatabase;
use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;

pub fn create_archive(path: &str, tx: Sender<TaskMessage>) {
    let _ = tx.send(TaskMessage::PortableProgress(
        0.0,
        "creating .cybermanju archive...".into(),
    ));
    match PortableDatabase::create(path, "cli") {
        Ok(_) => {
            let _ = tx.send(TaskMessage::PortableProgress(
                1.0,
                format!("created: {}", path),
            ));
            let _ = tx.send(TaskMessage::PortableDone(format!("Created: {}", path)));
        }
        Err(e) => {
            let _ = tx.send(TaskMessage::PortableError(format!("create failed: {}", e)));
        }
    }
}

pub fn extract_archive(path: &str, tx: Sender<TaskMessage>) {
    let _ = tx.send(TaskMessage::PortableProgress(
        0.0,
        "opening .cybermanju...".into(),
    ));
    let pdb = match PortableDatabase::open(path) {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(TaskMessage::PortableError(format!("open failed: {}", e)));
            return;
        }
    };
    let _ = tx.send(TaskMessage::PortableProgress(
        0.3,
        "unpacking database...".into(),
    ));

    let out_path = format!("{}.unpacked.db", path);
    match pdb.unpack(&out_path, None) {
        Ok(db) => {
            let header = pdb.header();
            let _ = tx.send(TaskMessage::PortableProgress(
                0.6,
                format!(
                    "db: {} files, {} relations, {} deletions",
                    header.total_files, header.total_relations, header.total_deletions
                ),
            ));

            let extract_dir = format!("{}.extracted", path);
            let _ = fs::create_dir_all(&extract_dir);

            let entries = PortableDatabase::list_recoverable_files(&db).unwrap_or_default();
            let total = entries.len();
            let _ = tx.send(TaskMessage::PortableProgress(
                0.7,
                format!("extracting {} files...", total),
            ));

            for (i, entry) in entries.iter().enumerate() {
                if let Ok(Some((data, mime))) =
                    pdb.get_recoverable_data(&db, &entry.original_file_id)
                {
                    let fpath = Path::new(&extract_dir).join(&entry.original_name);
                    let _ = fs::write(&fpath, &data);
                }
                let pct = 0.7 + (i as f64 / total as f64) * 0.25;
                let _ = tx.send(TaskMessage::PortableProgress(
                    pct,
                    format!("extracting {}/{}", i + 1, total),
                ));
            }

            let _ = tx.send(TaskMessage::PortableProgress(
                1.0,
                format!("extracted {} files to {}", total, extract_dir),
            ));
            let _ = tx.send(TaskMessage::PortableDone(format!(
                "Extracted {} files to {} (db: {})",
                total, extract_dir, out_path
            )));
        }
        Err(e) => {
            let _ = tx.send(TaskMessage::PortableError(format!("unpack error: {}", e)));
        }
    }
}

pub fn list_archive(path: &str, tx: Sender<TaskMessage>) {
    let pdb = match PortableDatabase::open(path) {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(TaskMessage::PortableError(format!("open failed: {}", e)));
            return;
        }
    };
    let h = pdb.header();
    let info = format!(
        ".cybermanju: version={}, created={}, modified={}, platform={}, files={}, relations={}, deletions={}, encrypted={}, compression={}",
        h.version, h.created_at, h.last_modified_at, h.platform_origin,
        h.total_files, h.total_relations, h.total_deletions,
        h.encryption_algorithm.as_deref().unwrap_or("none"),
        h.compression_algorithm,
    );
    let _ = tx.send(TaskMessage::PortableProgress(1.0, info.clone()));
    let _ = tx.send(TaskMessage::PortableDone(info));
}
