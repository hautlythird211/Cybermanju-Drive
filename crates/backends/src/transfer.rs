use cybermanju_types::sync::StorageBackend;
use std::fs;
use std::path::PathBuf;

fn tmp_dir() -> PathBuf {
    let d = std::env::temp_dir().join("cybermanju-transfer");
    let _ = fs::create_dir_all(&d);
    d
}

/// Transfer all files from source backend to destination backend.
/// Calls `progress(f64, &str)` with progress percentage and status message.
/// Returns total number of files transferred.
pub fn transfer_files(
    src: &dyn StorageBackend,
    dst: &dyn StorageBackend,
    progress: impl Fn(f64, &str),
) -> Result<usize, String> {
    let files = src.list_files("")?;
    let total = files.len();
    let tmp = tmp_dir();

    for (i, f) in files.iter().enumerate() {
        let local = tmp.join(&f.name);
        progress(
            i as f64 / total as f64,
            &format!("downloading {}/{}: {}", i + 1, total, f.name),
        );

        if let Err(e) = src.download_file(&f.path, local.to_str().unwrap_or("")) {
            progress(
                i as f64 / total as f64,
                &format!("download error {}: {}", f.name, e),
            );
            continue;
        }

        progress(
            (i as f64 + 0.5) / total as f64,
            &format!("uploading {}/{}: {}", i + 1, total, f.name),
        );

        if let Err(e) = dst.upload_file(local.to_str().unwrap_or(""), &f.name) {
            progress(
                i as f64 / total as f64,
                &format!("upload error {}: {}", f.name, e),
            );
        }

        let _ = fs::remove_file(&local);
    }

    progress(
        1.0,
        &format!("transferred {} files", total),
    );
    Ok(total)
}
