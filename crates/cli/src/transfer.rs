use crate::backends::{create_backend, StoredBackend};
use crate::tui::TaskMessage;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

fn tmp_dir() -> PathBuf {
    let d = std::env::temp_dir().join("cybermanju-transfer");
    let _ = fs::create_dir_all(&d);
    d
}

pub fn run_transfer(backends: Vec<StoredBackend>, src_idx: usize, dst_idx: usize, tx: Sender<TaskMessage>) {
    let src = match backends.get(src_idx) {
        Some(s) => s,
        None => { let _ = tx.send(TaskMessage::TransferError("invalid source".into())); return; }
    };
    let dst = match backends.get(dst_idx) {
        Some(d) => d,
        None => { let _ = tx.send(TaskMessage::TransferError("invalid destination".into())); return; }
    };

    let src_be = match create_backend(src.backend_type, &src.token, &src.config) {
        Some(b) => b,
        None => { let _ = tx.send(TaskMessage::TransferError("cannot create source backend".into())); return; }
    };
    let dst_be = match create_backend(dst.backend_type, &dst.token, &dst.config) {
        Some(b) => b,
        None => { let _ = tx.send(TaskMessage::TransferError("cannot create destination backend".into())); return; }
    };

    let files = match src_be.list_files("") {
        Ok(f) => f,
        Err(e) => { let _ = tx.send(TaskMessage::TransferError(format!("list source: {}", e))); return; }
    };

    let total = files.len();
    let tmp = tmp_dir();

    for (i, f) in files.iter().enumerate() {
        let local = tmp.join(&f.name);
        let _ = tx.send(TaskMessage::TransferProgress(i as f64 / total as f64, format!("downloading {}/{}: {}", i + 1, total, f.name)));

        if let Err(e) = src_be.download_file(&f.path, local.to_str().unwrap_or("")) {
            let _ = tx.send(TaskMessage::TransferError(format!("download {}: {}", f.name, e)));
            continue;
        }

        let _ = tx.send(TaskMessage::TransferProgress((i as f64 + 0.5) / total as f64, format!("uploading {}/{}: {}", i + 1, total, f.name)));

        if let Err(e) = dst_be.upload_file(local.to_str().unwrap_or(""), &f.name) {
            let _ = tx.send(TaskMessage::TransferProgress(i as f64 / total as f64, format!("upload {} error: {}", f.name, e)));
        }

        let _ = fs::remove_file(&local);
    }

    let _ = tx.send(TaskMessage::TransferProgress(1.0, format!("transferred {} files from {} to {}", total, src.name, dst.name)));
    let _ = tx.send(TaskMessage::TransferDone);
}
