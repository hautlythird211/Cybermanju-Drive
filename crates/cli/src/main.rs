use anyhow::{Context, Result};
use base64::Engine;
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;

use cybermanju_compression::TripleCompressor;
use cybermanju_crypto::{decrypt_data, encrypt_data, EncryptionAlgo, EncryptedFileMeta, KeyPair, PqcEngine};

mod backends;
mod harvest;
mod portable;
mod transfer;
mod tui;

#[derive(Parser)]
#[command(
    name = "cybermanju",
    version = "0.1.0",
    about = "Cybermanju Drive — Post-Quantum Encrypted OS CLI",
    styles = clap::builder::Styles::styled()
        .header(anstyle::Style::new().bold().fg_color(Some(anstyle::RgbColor(0x00, 0xff, 0x41).into())))
        .usage(anstyle::Style::new().fg_color(Some(anstyle::RgbColor(0x00, 0xff, 0x41).into())))
        .literal(anstyle::Style::new().fg_color(Some(anstyle::RgbColor(0x00, 0xff, 0x41).into())))
        .placeholder(anstyle::Style::new().fg_color(Some(anstyle::RgbColor(0x55, 0x55, 0x55).into()))),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive terminal UI
    Tui,

    /// Display system status overview
    Status,
    /// Run full diagnostic report
    Diagnostic,
    /// Encrypt a file using post-quantum cryptography
    Encrypt { path: PathBuf, #[arg(short, long)] output: Option<PathBuf> },
    /// Decrypt a file
    Decrypt { path: PathBuf, #[arg(short, long)] output: Option<PathBuf>, #[arg(short, long)] key: String },
    /// Manage encryption keys
    Keys { #[command(subcommand)] action: KeyAction },
    /// Compress a file using triple-layer compression
    Compress { path: PathBuf, #[arg(short, long)] output: Option<PathBuf> },
    /// Decompress a .cyber file
    Decompress { path: PathBuf, #[arg(short, long)] output: Option<PathBuf> },
    /// Database operations
    Db { #[command(subcommand)] action: DbAction },

    // ── Data Liberation Kit ──
    /// Harvest all data from all configured backends into .cybermanju
    Harvest {
        /// Output .cybermanju path (default: auto-generated)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Transfer files between backends
    Transfer {
        /// Source backend name
        source: String,
        /// Destination backend name
        dest: String,
    },

    /// .cybermanju portable database operations
    Portable {
        #[command(subcommand)]
        action: PortableAction,
    },

    /// Test connection to a configured backend
    TestConnection {
        /// Backend name
        name: String,
    },

    /// List files on a remote backend
    ListRemote {
        /// Backend name
        name: String,
        /// Remote path prefix filter
        #[arg(short, long, default_value = "")]
        prefix: String,
    },
}

#[derive(Subcommand)]
enum KeyAction { List, Generate { #[arg(short, long)] name: Option<String> }, Show { id: String } }

#[derive(Subcommand)]
enum DbAction { Check, Stats }

#[derive(Subcommand)]
enum PortableAction {
    /// Create a new .cybermanju archive
    Create { path: String },
    /// Extract files from a .cybermanju archive
    Extract { path: String },
    /// List contents of a .cybermanju archive
    List { path: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tui => {
            let mut app = tui::App::new();
            app.run()
        }
        Commands::Status => cmd_status(),
        Commands::Diagnostic => cmd_diagnostic(),
        Commands::Encrypt { path, output } => cmd_encrypt(path, output),
        Commands::Decrypt { path, output, key } => cmd_decrypt(path, output, &key),
        Commands::Keys { action } => cmd_keys(action),
        Commands::Compress { path, output } => cmd_compress(path, output),
        Commands::Decompress { path, output } => cmd_decompress(path, output),
        Commands::Db { action } => cmd_db(action),
        Commands::Harvest { output } => cmd_harvest_cli(output),
        Commands::Transfer { source, dest } => cmd_transfer_cli(&source, &dest),
        Commands::Portable { action } => cmd_portable(action),
        Commands::TestConnection { name } => cmd_test_connection(&name),
        Commands::ListRemote { name, prefix } => cmd_list_remote(&name, &prefix),
    }
}

// ── New CLI commands ──────────────────────────────────────────

fn cmd_harvest_cli(output: Option<PathBuf>) -> Result<()> {
    let cfg = tui::BackendConfig::load();
    if cfg.backends.is_empty() {
        anyhow::bail!("No backends configured. Run `cybermanju tui` to add backends");
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let backends = cfg.backends.clone();
    let out_str = output.as_ref().map(|p| p.to_string_lossy().to_string());
    println!(" Harvesting from {} backends...", backends.len());
    std::thread::spawn(move || harvest::run_harvest_with_output(backends, tx, out_str));
    while let Ok(msg) = rx.recv() {
        match msg {
            tui::TaskMessage::HarvestProgress(_, _, _, s) => {
                println!("  {}", s);
            }
            tui::TaskMessage::HarvestOverall(p, s) => {
                println!("  {:.0}% — {}", p * 100.0, s);
            }
            tui::TaskMessage::HarvestDone(name, files, bytes) => {
                println!("  \x1b[32m{}: {} files, {} bytes\x1b[0m", name, files, bytes);
            }
            tui::TaskMessage::HarvestError(name, err) => {
                println!("  \x1b[31m{}: {}\x1b[0m", name, err);
            }
            tui::TaskMessage::HarvestComplete(files, bytes) => {
                println!(" \x1b[32mDone — {} files, {} bytes harvested\x1b[0m", files, bytes);
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

fn cmd_test_connection(name: &str) -> Result<()> {
    let cfg = tui::BackendConfig::load();
    let sb = cfg.backends.iter().find(|b| b.name == name)
        .ok_or_else(|| anyhow::anyhow!("Backend '{}' not found", name))?;
    let be = backends::create_backend(sb.backend_type, &sb.token, &sb.config)
        .ok_or_else(|| anyhow::anyhow!("Failed to create backend"))?;
    heading(&format!("Testing connection: {}", name));
    match be.test_connection() {
        Ok(true) => { ok("Connection successful"); Ok(()) }
        Ok(false) => { err_msg("Connection failed (unexpected response)"); Ok(()) }
        Err(e) => { err_msg(&format!("Connection error: {}", e)); Ok(()) }
    }
}

fn cmd_list_remote(name: &str, prefix: &str) -> Result<()> {
    let cfg = tui::BackendConfig::load();
    let sb = cfg.backends.iter().find(|b| b.name == name)
        .ok_or_else(|| anyhow::anyhow!("Backend '{}' not found", name))?;
    let be = backends::create_backend(sb.backend_type, &sb.token, &sb.config)
        .ok_or_else(|| anyhow::anyhow!("Failed to create backend"))?;
    heading(&format!("Remote files: {} ({})", name, prefix));
    let files = be.list_files(prefix).map_err(|e| anyhow::anyhow!("List failed: {}", e))?;
    for f in &files {
        println!("  {}  {:>12}  {}", f.path, f.size_bytes, f.name);
    }
    kv("Total files", &files.len().to_string());
    Ok(())
}

fn cmd_transfer_cli(source: &str, dest: &str) -> Result<()> {
    let cfg = tui::BackendConfig::load();
    let src = cfg.backends.iter().find(|b| b.name == source)
        .ok_or_else(|| anyhow::anyhow!("source backend '{}' not found", source))?;
    let dst = cfg.backends.iter().find(|b| b.name == dest)
        .ok_or_else(|| anyhow::anyhow!("dest backend '{}' not found", dest))?;
    println!(" {} -> {} transfer initiated (use `cybermanju tui` for interactive)", source, dest);
    let (tx, rx) = std::sync::mpsc::channel();
    let backends = cfg.backends.clone();
    let src_idx = backends.iter().position(|b| b.name == source).unwrap();
    let dst_idx = backends.iter().position(|b| b.name == dest).unwrap();
    std::thread::spawn(move || transfer::run_transfer(backends, src_idx, dst_idx, tx));
    while let Ok(msg) = rx.recv() {
        match msg {
            tui::TaskMessage::TransferProgress(p, s) => {
                println!("  {}% — {}", (p * 100.0) as u16, s);
            }
            tui::TaskMessage::TransferDone => {
                println!(" \x1b[32mDone\x1b[0m");
                break;
            }
            tui::TaskMessage::TransferError(e) => {
                println!(" \x1b[31mError: {}\x1b[0m", e);
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

fn cmd_portable(action: PortableAction) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    match action {
        PortableAction::Create { path } => {
            println!(" Creating .cybermanju archive: {}", path);
            std::thread::spawn(move || portable::create_archive(&path, tx));
            while let Ok(msg) = rx.recv() {
                match msg {
                    tui::TaskMessage::PortableProgress(_, s) => println!("  {}", s),
                    tui::TaskMessage::PortableDone(s) => { println!(" \x1b[32m{}\x1b[0m", s); break; }
                    tui::TaskMessage::PortableError(e) => { println!(" \x1b[31m{}\x1b[0m", e); break; }
                    _ => {}
                }
            }
        }
        PortableAction::Extract { path } => {
            println!(" Extracting .cybermanju archive: {}", path);
            std::thread::spawn(move || portable::extract_archive(&path, tx));
            while let Ok(msg) = rx.recv() {
                match msg {
                    tui::TaskMessage::PortableProgress(_, s) => println!("  {}", s),
                    tui::TaskMessage::PortableDone(s) => { println!(" \x1b[32m{}\x1b[0m", s); break; }
                    tui::TaskMessage::PortableError(e) => { println!(" \x1b[31m{}\x1b[0m", e); break; }
                    _ => {}
                }
            }
        }
        PortableAction::List { path } => {
            std::thread::spawn(move || portable::list_archive(&path, tx));
            while let Ok(msg) = rx.recv() {
                match msg {
                    tui::TaskMessage::PortableProgress(_, s) => println!("  {}", s),
                    tui::TaskMessage::PortableDone(s) => { println!(" \x1b[32m{}\x1b[0m", s); break; }
                    tui::TaskMessage::PortableError(e) => { println!(" \x1b[31m{}\x1b[0m", e); break; }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ── LEGACY COMMANDS (unchanged) ─────────────────────────────

fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cybermanju-drive")
}

fn keys_dir() -> PathBuf {
    data_dir().join("keys")
}

fn ok(text: &str) { println!(" \x1b[32m[\x1b[1mOK\x1b[22m]\x1b[0m {}", text); }
fn warn(text: &str) { println!(" \x1b[33m[WARN]\x1b[0m {}", text); }
fn err_msg(text: &str) { println!(" \x1b[31m[ERR]\x1b[0m {}", text); }
fn info(text: &str) { println!("  \x1b[90m{}\x1b[0m", text); }
fn kv(key: &str, val: &str) { println!("  \x1b[32m{}\x1b[0m: \x1b[97m{}\x1b[0m", key, val); }

fn heading(text: &str) {
    println!("\x1b[32m╔═══════════════════════════════════════════════════════════════╗\n║  {:<67} ║\n╚═══════════════════════════════════════════════════════════════╝\x1b[0m", text);
}

fn cmd_status() -> Result<()> {
    heading("Cybermanju Drive — System Status");
    println!();
    kv("CLI Version", "0.1.0");
    kv("Build", env!("CARGO_PKG_VERSION"));
    let cfg = tui::BackendConfig::load();
    kv("Configured backends", &cfg.backends.len().to_string());
    println!();
    check_data_dir();
    check_env_deps();
    println!();
    ok("Status check complete");
    Ok(())
}

fn check_data_dir() {
    let dd = data_dir();
    if dd.exists() { ok("Data directory found"); info(&dd.display().to_string()); }
    else { info("No data directory yet"); return; }
    let db_path = dd.join("cybermanju.db");
    if db_path.exists() {
        let meta = std::fs::metadata(&db_path).ok();
        if let Some(m) = meta { kv("Database", &format!("{} ({} MB)", db_path.display(), m.len() / 1024 / 1024)); }
    } else { warn("No database — run the desktop app to initialize"); }
    let kd = keys_dir();
    if kd.exists() {
        let count = std::fs::read_dir(&kd).map(|e| e.filter_map(|e| e.ok()).count()).unwrap_or(0);
        kv("Encryption keys", &format!("{} found", count));
    } else { info("No keys — generate with `cybermanju keys generate`"); }
}

fn check_env_deps() {
    #[cfg(target_os = "linux")]
    {
        let wkit_paths = [
            "/usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so",
            "/usr/lib/aarch64-linux-gnu/libwebkit2gtk-4.1.so",
            "/usr/lib/libwebkit2gtk-4.1.so",
        ];
        if wkit_paths.iter().any(|p| PathBuf::from(p).exists()) { ok("WebKit2GTK found"); }
        else {
            let via_dpkg = std::process::Command::new("dpkg").args(["-l", "libwebkit2gtk-4.1-0"]).output().ok().map(|o| o.status.success()).unwrap_or(false);
            let via_rpm = std::process::Command::new("rpm").args(["-q", "webkit2gtk4.1"]).output().ok().map(|o| o.status.success()).unwrap_or(false);
            if !via_dpkg && !via_rpm { warn("WebKit2GTK not detected — install libwebkit2gtk-4.1-0"); }
        }
        let gtk_paths = ["/usr/lib/x86_64-linux-gnu/libgtk-3.so", "/usr/lib/aarch64-linux-gnu/libgtk-3.so", "/usr/lib/libgtk-3.so"];
        if gtk_paths.iter().any(|p| PathBuf::from(p).exists()) { ok("GTK3 found"); }
        else { warn("GTK3 not detected — install libgtk-3-0"); }
    }
    #[cfg(target_os = "windows")] {
        let wv2 = PathBuf::from(std::env::var("PROGRAMFILES(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".into())).join("Microsoft\\Edge\\Application\\webview2.exe");
        if wv2.exists() { ok("WebView2 found"); }
        else { let alt = PathBuf::from(std::env::var("PROGRAMFILES").unwrap_or_else(|_| "C:\\Program Files".into())).join("Microsoft\\Edge\\Application\\webview2.exe");
            if alt.exists() { ok("WebView2 found (alt)"); } else { warn("WebView2 not detected"); } }
    }
    #[cfg(target_os = "macos")] { ok("WebKit (system) — bundled with macOS"); }
}

fn cmd_diagnostic() -> Result<()> {
    heading("Cybermanju Drive — Full Diagnostic");
    println!();
    kv("CLI Version", env!("CARGO_PKG_VERSION"));
    kv("Platform", std::env::consts::OS);
    kv("Architecture", std::env::consts::ARCH);
    println!();
    let dd = data_dir();
    kv("Data directory", &dd.display().to_string());
    if dd.exists() { let entries = std::fs::read_dir(&dd).map(|e| e.filter_map(|e| e.ok()).count()).unwrap_or(0); kv("Data entries", &entries.to_string()); }
    println!();
    let crash_log = dd.join("crash.log");
    if crash_log.exists() { warn("Previous crash log found:"); let content = std::fs::read_to_string(&crash_log)?; for line in content.lines() { info(line); } }
    else { ok("No crash log — clean shutdown"); }
    println!();
    check_env_deps();
    println!();
    ok("Diagnostic complete");
    Ok(())
}

fn cmd_encrypt(path: PathBuf, output: Option<PathBuf>) -> Result<()> {
    heading("Cybermanju Drive — Encrypt File");
    println!();
    if !path.exists() { anyhow::bail!("File not found: {}", path.display()); }
    let data = std::fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    info(&format!("Read {} bytes", data.len()));
    let kd = keys_dir();
    let key = if kd.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&kd)?.filter_map(|e| e.ok()).filter(|e| e.path().extension().map_or(false, |ext| ext == "json")).collect();
        entries.sort_by_key(|e| e.path().metadata().ok().and_then(|m| m.modified().ok()));
        if let Some(entry) = entries.last() {
            let key_data = std::fs::read_to_string(entry.path())?;
            let kp: KeyPair = serde_json::from_str(&key_data)?;
            info(&format!("Using key: {}", kp.id)); kp
        } else { warn("No saved keys — generating ephemeral"); let mut engine = PqcEngine::new(); engine.generate_keypair(EncryptionAlgo::Kyber1024)? }
    } else { warn("No keys dir — generating ephemeral key"); let mut engine = PqcEngine::new(); engine.generate_keypair(EncryptionAlgo::Kyber1024)? };
    let encrypted = encrypt_data(&data, &key).map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
    let ciphertext = encrypted.ciphertext.clone();
    let meta = EncryptedFileMeta::from(&encrypted);
    let meta_json = serde_json::to_string(&meta)?;
    let out_path = output.unwrap_or_else(|| { let mut p = path.clone(); let _ = p.set_extension("encrypted"); p });
    let mut out_file = std::fs::File::create(&out_path).with_context(|| format!("Failed to create {}", out_path.display()))?;
    writeln!(out_file, "{}", meta_json)?; out_file.write_all(&ciphertext)?;
    kv("Output", &out_path.display().to_string()); kv("Algorithm", &meta.algorithm); kv("Key ID", &meta.key_id); ok("Encrypted successfully");
    Ok(())
}

fn cmd_decrypt(path: PathBuf, output: Option<PathBuf>, key_id: &str) -> Result<()> {
    heading("Cybermanju Drive — Decrypt File");
    println!();
    if !path.exists() { anyhow::bail!("File not found: {}", path.display()); }
    let raw = std::fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut lines = raw.splitn(2, |&b| b == b'\n');
    let meta_line = lines.next().ok_or_else(|| anyhow::anyhow!("Invalid encrypted file"))?;
    let ciphertext = lines.next().ok_or_else(|| anyhow::anyhow!("Missing ciphertext"))?.to_vec();
    let meta: EncryptedFileMeta = serde_json::from_slice(meta_line)?;
    let encrypted = meta.to_encrypted_data(ciphertext)?;
    info(&format!("Algorithm: {}", encrypted.algorithm)); info(&format!("Key ID: {}", encrypted.key_id));
    let key_path = keys_dir().join(format!("{}.json", key_id));
    if !key_path.exists() { anyhow::bail!("Key not found: {}", key_id); }
    let key_data = std::fs::read_to_string(&key_path)?; let key: KeyPair = serde_json::from_str(&key_data)?;
    let plaintext = decrypt_data(&encrypted, &key).map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    let out_path = output.unwrap_or_else(|| { let mut p = path.clone(); let _ = p.set_extension("decrypted"); p });
    std::fs::write(&out_path, &plaintext).with_context(|| format!("Failed to write {}", out_path.display()))?;
    kv("Output", &out_path.display().to_string()); kv("Size", &format!("{} bytes", plaintext.len())); ok("Decrypted successfully");
    Ok(())
}

fn cmd_keys(action: KeyAction) -> Result<()> {
    match action {
        KeyAction::List => {
            heading("Cybermanju Drive — Encryption Keys");
            println!();
            let kd = keys_dir();
            if !kd.exists() { info("No keys found. Generate one with: cybermanju keys generate"); return Ok(()); }
            for entry in std::fs::read_dir(&kd)? {
                let entry = entry?; let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(data) = std::fs::read_to_string(&path) { if let Ok(kp) = serde_json::from_str::<KeyPair>(&data) { kv(&kp.id, &format!("{:?} — created {}", kp.algorithm, kp.created_at)); } }
                }
            }
            ok("Key listing complete");
        }
        KeyAction::Generate { name } => {
            heading("Cybermanju Drive — Generate Key");
            println!();
            let key_name = name.unwrap_or_else(|| { let id = uuid::Uuid::new_v4(); format!("key-{}", &id.to_string()[..8]) });
            let mut engine = PqcEngine::new();
            let key = engine.generate_keypair(EncryptionAlgo::Kyber1024)?;
            let kd = keys_dir(); std::fs::create_dir_all(&kd)?;
            let key_path = kd.join(format!("{}.json", &key_name));
            std::fs::write(&key_path, &serde_json::to_string_pretty(&key)?)?;
            kv("Key ID", &key.id); kv("Name", &key_name); kv("Algorithm", &format!("{:?}", key.algorithm)); kv("Saved to", &key_path.display().to_string()); ok("Key generated successfully");
        }
        KeyAction::Show { id } => {
            let key_path = keys_dir().join(format!("{}.json", &id));
            if !key_path.exists() { anyhow::bail!("Key not found: {}", id); }
            let data = std::fs::read_to_string(&key_path)?; let key: KeyPair = serde_json::from_str(&data)?;
            kv("ID", &key.id); kv("Algorithm", &format!("{:?}", key.algorithm)); kv("Created", &key.created_at);
            kv("Public key", &base64::engine::general_purpose::STANDARD.encode(&key.public_key)[..48]); kv("Fingerprint", &blake3::hash(&key.public_key).to_hex()[..16]);
        }
    }
    Ok(())
}

fn cmd_compress(path: PathBuf, output: Option<PathBuf>) -> Result<()> {
    heading("Cybermanju Drive — Compress File");
    println!();
    if !path.exists() { anyhow::bail!("File not found: {}", path.display()); }
    let data = std::fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    info(&format!("Read {} bytes", data.len()));
    let compressor = TripleCompressor::new();
    let (compressed, stats) = compressor.compress_triple(&data)?;
    let out_path = output.unwrap_or_else(|| { let mut p = path.clone(); let _ = p.set_extension("cyber"); p });
    std::fs::write(&out_path, &compressed).with_context(|| format!("Failed to write {}", out_path.display()))?;
    kv("Output", &out_path.display().to_string()); kv("Original", &format!("{} bytes", stats.original_size)); kv("Compressed", &format!("{} bytes", stats.compressed_size));
    kv("Ratio", &format!("{:.1}%", stats.ratio * 100.0)); kv("Duration", &format!("{} ms", stats.duration_ms)); ok("Compressed successfully");
    Ok(())
}

fn cmd_decompress(path: PathBuf, output: Option<PathBuf>) -> Result<()> {
    heading("Cybermanju Drive — Decompress File");
    println!();
    if !path.exists() { anyhow::bail!("File not found: {}", path.display()); }
    let data = std::fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    info(&format!("Read {} bytes", data.len()));
    let compressor = TripleCompressor::new();
    let (decompressed, _) = compressor.decompress_triple(&data)?;
    let out_path = output.unwrap_or_else(|| { let mut p = path.clone(); if path.extension().map_or(false, |ext| ext == "cyber") { let _ = p.set_extension(""); } else { let _ = p.set_extension("decompressed"); } p });
    std::fs::write(&out_path, &decompressed).with_context(|| format!("Failed to write {}", out_path.display()))?;
    kv("Output", &out_path.display().to_string()); kv("Size", &format!("{} bytes", decompressed.len())); ok("Decompressed successfully");
    Ok(())
}

fn cmd_db(action: DbAction) -> Result<()> {
    match action {
        DbAction::Check => {
            heading("Cybermanju Drive — Database Check");
            println!();
            let db_path = data_dir().join("cybermanju.db");
            if !db_path.exists() { warn("No database found"); info("Run the desktop app first"); return Ok(()); }
            let meta = std::fs::metadata(&db_path)?; kv("Database", &db_path.display().to_string()); kv("Size", &format!("{} MB", meta.len() / 1024 / 1024)); ok("Database file is present");
        }
        DbAction::Stats => {
            heading("Cybermanju Drive — Database Stats");
            println!();
            let db_path = data_dir().join("cybermanju.db");
            if !db_path.exists() { warn("No database found"); return Ok(()); }
            match cybermanju_db::Database::new(db_path.to_str().unwrap_or("")) {
                Ok(_db) => { kv("Database", &db_path.display().to_string()); let meta = std::fs::metadata(&db_path)?; kv("Size", &format!("{} MB", meta.len() / 1024 / 1024)); ok("Database opened successfully"); }
                Err(e) => anyhow::bail!("Failed to open database: {}", e),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clap_parses_status() {
        let cli = Cli::try_parse_from(["cybermanju", "status"]).unwrap();
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn test_clap_parses_tui() {
        let cli = Cli::try_parse_from(["cybermanju", "tui"]).unwrap();
        assert!(matches!(cli.command, Commands::Tui));
    }

    #[test]
    fn test_clap_parses_harvest() {
        let cli = Cli::try_parse_from(["cybermanju", "harvest"]).unwrap();
        assert!(matches!(cli.command, Commands::Harvest { .. }));
    }

    #[test]
    fn test_clap_parses_portable_create() {
        let cli = Cli::try_parse_from(["cybermanju", "portable", "create", "/tmp/test.cybermanju"]).unwrap();
        assert!(matches!(cli.command, Commands::Portable { .. }));
    }

    #[test]
    fn test_clap_parses_transfer() {
        let cli = Cli::try_parse_from(["cybermanju", "transfer", "source-name", "dest-name"]).unwrap();
        assert!(matches!(cli.command, Commands::Transfer { .. }));
    }
}
