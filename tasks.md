# Cybermanju Drive — Development Tasks

## Task 1: CLI ↔ Tauri Backend Refactoring (Completed)

### Problem

The CLI (`crates/cli`) and Tauri desktop app (`src-tauri`) each had **independent,
duplicated implementations** of all 7 storage backends (Local, GitHub, GitLab,
Google Drive, Google Photos, Telegram, Mega). The two copies drifted over time:
the Tauri version had better error messages + a safer `safe_join`, the CLI
version had Google Photos download fix, the Mega backend used completely
different APIs (`megalib::MegaClient` vs `megalib::SessionHandle`).

### Solution

Extract all backend implementations into a new shared workspace crate —
`crates/backends` (`cybermanju-backends`). Both apps now delegate to it.

## Task 2: Workspace Expansion (Completed)

Expanded workspace from 12 to 17 crates:

| Crate | Purpose | Status |
|-------|---------|--------|
| `crates/media` | Image processing, video thumbnails, metadata extraction | ✅ In workspace |
| `crates/erasure` | Reed-Solomon, fountain codes, Shamir secret sharing | ✅ In workspace |
| `crates/resolutions` | Merkle tree shard resolution for distributed storage | ✅ In workspace |
| `crates/preview-keys` | Key derivation and view token generation | ✅ In workspace |
| `crates/recovery` | Recovery and reconstruction pipeline | ⚠️ Exists but not in workspace |

## Task 3: Documentation Overhaul (Completed)

Updated all documentation files to reflect current state:
- **ARCHITECTURE.md**: Fixed component count (16→80+), added WASM mode, added 11 missing DB tables, added Codeberg/Gitea/Telegram/Mega backends, added 5 new crates to module list
- **RELEASE_NOTES.md**: Fixed version (0.0.1→0.1.0), fixed crate count (12→17), added Codeberg/Gitea backends
- **worklog.md**: Added 4 comprehensive task entries covering all major development phases
- **tasks.md**: Added current task tracking

## New Crate: `crates/backends/`

```
crates/backends/
├── Cargo.toml
├── src/
│   ├── lib.rs            — Factory function + re-exports
│   ├── util.rs           — safe_join() + http_client()
│   ├── local.rs          — LocalBackend
│   ├── github.rs         — GitHubBackend (Contents API + Git LFS)
│   ├── gitlab.rs         — GitLabBackend
│   ├── codeberg.rs       — CodebergBackend (Forgejo API)
│   ├── gitea.rs          — GiteaBackend (self-hosted)
│   ├── google_drive.rs   — GoogleDriveBackend
│   ├── google_photos.rs  — GooglePhotosBackend
│   ├── telegram.rs       — TelegramBackend
│   ├── mega.rs           — MegaBackend (tokio runtime)
│   ├── git_lfs.rs        — GitLfsClient (batch API)
│   ├── repo_layout.rs    — .cybermanju repo structure manager
│   └── transfer.rs       — transfer_files()
```

### Key design decisions

| Decision | Rationale |
|---|---|
| Uses `cybermanju_types::sync` trait/types | Both apps already depend on the types crate; no new trait |
| Uses Tauri's `safe_join` (symlink-safe) | More secure than the CLI version |
| Uses Tauri's MegaBackend (tokio + SessionHandle) | Async `megalib` API is the correct one; CLI's `MegaClient` was wrong |
| Uses CLI's Google Photos fix (`baseUrl` + `=d`) | The CLI had the correct download endpoint |
| `create_backend` takes `(&SyncBackendType, &str, &Value)` | Generic enough for both apps |

## Changes per file

### `crates/types/src/sync.rs`
- Added `created_at` + `updated_at` fields to `SyncConfig` (Tauri had them, types didn't)
- Fixed `Display` for `SyncBackendType`: `"github"` → `"gitHub"`, `"gitlab"` → `"gitLab"` (matches Tauri)

### `crates/backends/` (new)
- 10 backend implementations consolidated from the best of both apps
- `safe_join` from Tauri (parent-directory validation, symlink rejection)
- `MegaBackend` using `megalib::SessionHandle` + `tokio::runtime::Runtime`
- `GitLfsClient` for large file uploads via Git LFS batch API
- `transfer_files()` shared transfer function
- Factory `create_backend()` with per-backend config extraction

### `crates/cli/`
- `backends.rs`: replaced all 770 lines with a 15-line re-export wrapper
- Fixed `StoredBackend` import in `harvest.rs` + `transfer.rs` (was pointing to `crate::backends`, now to `crate::tui`)
- Removed unused imports in `tui.rs`, `harvest.rs`, `portable.rs` (queue, symbols, Text, Cell, Clear, Row, Table, TableState, Arc, Mutex, StorageBackend, etc.)
- `urlencoding::urlencoding` → `urlencoding::encode` (was a compilation error)
- Google Photos `download_file`: uses `baseUrl` + `=d` instead of broken `:download` endpoint
- `safe_join`: handles non-existent paths (parent-directory fallback)

### `src-tauri/`
- `sync/backends.rs`: replaced all 1897 lines with a ~75-line wrapper that maps `SyncConfig` → shared `create_backend`
- `sync/models.rs`: re-exports `{RemoteFile, StorageBackend, SyncBackendType, SyncConfig, SyncFile, SyncProgress, SyncResult, SyncStatus}` from `cybermanju_types::sync` instead of defining them locally
- `sync/mod.rs`: unchanged (`pub use models::*` + `pub use oauth::OAuthCredentials`)
- `sync/oauth.rs`: unchanged (keeps local `OAuthCredentials` to preserve inherent `impl` methods)

### `Cargo.toml` (workspace root)
- Added `crates/backends` to workspace members
- Added `crates/media`, `crates/erasure`, `crates/resolutions`, `crates/preview-keys` to workspace members

## Type compatibility

| Type | Old (Tauri models) | New (types crate) |
|---|---|---|
| `SyncBackendType` | Local, GitHub, GitLab, GoogleDrive, GooglePhotos, Telegram, Mega | Same — Display now matches Tauri's camelCase |
| `SyncConfig` | 20 fields (with created_at/updated_at) | 20 fields (now has created_at/updated_at) |
| `RemoteFile` | name, path, size_bytes, modified_at, url | Same |
| `StorageBackend` trait | 7 methods | Same signature |
| `OAuthCredentials` | oauth.rs (local) | oauth.rs (unchanged — local type with impl methods) |
| `CloudAccount` | models.rs → types crate | Not re-exported (not used anywhere in Tauri) |

## What's NOT changed

- Tauri commands (`commands/*.rs`) — import `create_backend` from `crate::sync::backends`, which still has the same function signature `(&SyncConfig) -> Result<Box<dyn StorageBackend>, String>`
- Tauri pipeline (`sync/pipeline.rs`) — imports from `sync::models::*`, works unchanged
- Tauri transfer (`transfer/mod.rs`) — same
- CLI TUI (`tui.rs`) — imports from `crate::backends`, which re-exports the shared crate
