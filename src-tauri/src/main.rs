// Cybermanju Drive — Tauri v2 Entry Point
// Neobrutalism × Buddhist-Nepalese × Matrix × Cyberpunk

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// On Windows, when `windows_subsystem = "windows"` hides the console,
// `--debug` re-allocates one so users can see log output.
#[cfg(target_os = "windows")]
extern "system" {
    fn AllocConsole() -> i32;
}

fn main() {
    // Parse CLI args: `--debug` enables verbose logging to stderr
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--debug") {
        // Override RUST_LOG for verbose output; if already set, keep it
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "cybermanju_drive=debug,info");
        }
        // On Windows with `windows_subsystem = "windows"`, alloc a console
        #[cfg(target_os = "windows")]
        unsafe {
            let _ = AllocConsole();
        }
    }

    cybermanju_drive_lib::run()
}
