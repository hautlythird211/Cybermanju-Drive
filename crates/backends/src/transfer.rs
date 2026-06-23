use cybermanju_types::sync::StorageBackend;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

type ProgressCallback = Arc<dyn Fn(f64, &str) + Send + Sync>;

/// Create a secure temporary directory with restricted permissions (0o700).
/// Uses a unique subdirectory per transfer to avoid collisions.
fn secure_tmp_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join("cybermanju-transfer");

    // Create base dir with restricted permissions
    fs::create_dir_all(&base).map_err(|e| format!("create tmp dir: {}", e))?;

    // Set permissions to owner-only (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&base, fs::Permissions::from_mode(0o700));
    }

    // Create unique subdirectory for this transfer session
    let session_id = {
        let mut buf = [0u8; 8];
        getrandom::getrandom(&mut buf).map_err(|e| format!("generate session id: {}", e))?;
        hex::encode(buf)
    };

    let session_dir = base.join(session_id);
    fs::create_dir_all(&session_dir).map_err(|e| format!("create session dir: {}", e))?;

    Ok(session_dir)
}

/// Sanitize a filename for temp storage: reject path components.
fn sanitize_filename(name: &str) -> String {
    #[allow(clippy::collapsible_str_replace)]
    name.replace('/', "_")
        .replace('\\', "_")
        .replace("..", "_")
        .trim_start_matches('.')
        .to_string()
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
    let tmp = secure_tmp_dir()?;

    for (i, f) in files.iter().enumerate() {
        let safe_name = sanitize_filename(&f.name);
        let local = tmp.join(&safe_name);
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

        // Set file to owner-only read/write (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&local, fs::Permissions::from_mode(0o600));
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

        // Securely remove file
        let _ = fs::remove_file(&local);
    }

    // Clean up session directory
    let _ = fs::remove_dir_all(&tmp);

    progress(1.0, &format!("transferred {} files", total));
    Ok(total)
}

/// Parallel transfer using rayon thread pool.
/// Each file is downloaded from src and uploaded to dst concurrently.
pub fn transfer_files_parallel(
    src: &(dyn StorageBackend + Sync + Send),
    dst: &(dyn StorageBackend + Sync + Send),
    max_parallel: usize,
    progress: ProgressCallback,
) -> Result<usize, String> {
    let files = src.list_files("")?;
    let total = files.len();
    if total == 0 {
        progress(1.0, "no files to transfer");
        return Ok(0);
    }

    let tmp = secure_tmp_dir()?;
    let completed = Arc::new(Mutex::new(0u32));
    let src = Arc::new(src);
    let dst = Arc::new(dst);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(max_parallel.max(1))
        .build()
        .map_err(|e| format!("thread pool: {}", e))?;

    pool.install(|| {
        use rayon::prelude::*;
        files.par_iter().for_each(|f| {
            let safe_name = sanitize_filename(&f.name);
            let local = tmp.join(&safe_name);
            progress(0.0, &format!("downloading: {}", f.name));

            if let Err(e) = src.download_file(&f.path, local.to_str().unwrap_or("")) {
                progress(0.0, &format!("download error {}: {}", f.name, e));
                return;
            }

            // Set file to owner-only read/write (Unix)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&local, fs::Permissions::from_mode(0o600));
            }

            progress(0.5, &format!("uploading: {}", f.name));

            if let Err(e) = dst.upload_file(local.to_str().unwrap_or(""), &f.name) {
                progress(0.0, &format!("upload error {}: {}", f.name, e));
            }

            let _ = fs::remove_file(&local);

            let count = {
                let mut c = completed.lock().unwrap();
                *c += 1;
                *c
            };
            progress(
                count as f64 / total as f64,
                &format!("{}/{} done", count, total),
            );
        });
    });

    // Clean up session directory
    let _ = fs::remove_dir_all(&tmp);

    progress(1.0, &format!("transferred {} files", total));
    Ok(total)
}
