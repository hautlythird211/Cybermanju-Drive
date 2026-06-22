# Cybermanju Drive

> Quantum-resistant encrypted file manager with AI face grouping, triple-layer compression, code intelligence, GPS map view, web dashboard, and multi-user access control.

**Version:** 0.1.0
**Identifier:** `com.cybermanju.drive`
**License:** MIT

---

## Features

### Core File Management
- **Virtual file system** with folders, tagging, and metadata
- **BLAKE3 content hashing** for deduplication and integrity verification
- **MIME type detection** via the `infer` crate (magic bytes, not extensions)
- **File previews** with Lanczos3 thumbnail generation (512px max, PNG)
- **EXIF GPS extraction** from images via `kamadak-exif`
- **Collections** — curated groups: highlights, best moments, custom albums
- **Loose groups** — ad-hoc user-defined file groupings

### Post-Quantum Cryptography
- **ML-KEM-1024** (FIPS 203) — lattice-based key encapsulation, NIST Level 5
- **Hybrid ML-KEM-768 + X25519** — defense-in-depth transitional security
- **ML-DSA-44/65/87** (FIPS 204) — lattice-based digital signatures (Levels 2/3/5)
- **ChaCha20Poly1305** AEAD symmetric encryption with HKDF-SHA256 derived keys
- **BLAKE3 integrity verification** on every encrypt/decrypt cycle
- **Argon2id** password hashing for user authentication

### Triple-Layer Compression
- **LZ4** (lz4_flex) — ~400 MB/s ultra-fast, real-time previews
- **Zstandard** (level 15) — balanced ratio/speed
- **Brotli** (level 11) — maximum compression ratio for archival
- Cascading pipeline: LZ4 → ZSTD → Brotli (or any single layer)
- Auto-skip when LZ4 ratio > 0.98 (incompressible data)
- Per-layer stats reporting with compression ratios and timing

### Full-Text Search
- **Tantivy** search engine with BM25 ranking
- Indexed fields: filename, content text, tags, file type, encryption status, GPS, timestamp, BLAKE3 hash
- Query support: terms, phrases, booleans (`AND`/`OR`), wildcards, fuzzy (`~1`)
- Real autocomplete from Tantivy term dictionary
- Faceted filtering by file type, encryption status, GPS presence

### AI Face Detection & Clustering
- 512-dim deterministic embedding extraction (BLAKE3-seeded)
- 4 clustering algorithms, auto-selected by dataset size:
  - **BruteForce** O(n²) — exact, for n ≤ 200
  - **SimHash LSH** — approximate, for n > 1000 (64-bit binary codes, 3 hash tables, ~97% recall)
  - **Chinese Whispers** — graph label propagation, for 200 < n ≤ 1000
  - **HDBSCAN** — MST-based hierarchical density, no eps parameter
- Optional ONNX integration: SCRFD-2.5G detection + ArcFace 512-d embeddings
- SimHash index: O(1) Hamming distance pre-filtering (POPCNT instruction)

### Code Intelligence
- **tree-sitter** integration with language detection for 50+ file extensions
- Heuristic symbol extraction: functions, classes, structs, traits, interfaces
- Language-aware keyword sets for Rust, Python, Go, TypeScript, Java, C/C++, Ruby, Swift

### Multi-User Access Control
- Role-based access control: `admin`, `user`, `viewer`
- Per-file permissions: `read`, `write`, `admin`
- Argon2id password hashing with cryptographically secure salts
- JWT-like session tokens (jsonwebtoken + HMAC-SHA256)

### Cloud Sync (10 Backends)
- **Local** — filesystem copy to any local directory
- **GitHub** — Contents API + Releases for large files (up to 2GB)
- **GitLab** — GitLab API
- **Google Drive** — Drive API v3 with full CRUD + OAuth
- **Google Photos** — optimized media upload
- **Telegram** — Telegram Bot API
- **Mega** — MEGA API via megalib crate
- Configurable pipeline: compress → preview → upload → link → delete raw
- Real-time progress with ETA estimation and cancellation support

### .cybermanju Portable Database
- Self-contained, triple-compressed, optionally encrypted redb database
- Cross-platform file relation tracking, deletion propagation, and recovery
- One `.cybermanju` file per user, synced to every connected platform
- BLAKE3-deduplicated content + preview blob storage
- Binary format: `[32B magic][4B header_len][PortableHeader JSON][compressed DB]`

### GPS Map View
- EXIF GPS coordinate extraction from photos
- Interactive map display via MapLibre GL
- Per-file geo-marker clustering

### Web Dashboard
- Embedded HTTP/1.1 server on port 3456 (no external HTTP dependency)
- REST API mirroring all Tauri IPC commands
- JWT authentication
- Works in Docker containers and ZimaOS NAS devices
- Browser access from any device on the network

### Triple-Mode IPC Bridge
- **Tauri IPC** (via Conduit plugin — 2.4x faster) for desktop
- **WASM bridge** (IndexedDB-backed) for browser with WASM support
- **REST API** (HTTP fetch) for Docker/ZimaOS web mode
- Auto-detects environment and routes commands transparently

### Additional Features
- **Trash/Recycle Bin** with restore and empty
- **File versioning** with snapshot and revert
- **Share links** with token-based access and expiration
- **Audit log** for all file operations
- **Batch operations** (delete, encrypt, compress)
- **Duplicate detection** via BLAKE3 hash matching
- **Keyboard shortcuts** with configurable keymaps
- **Drag-and-drop**, **context menus**, **touch/swipe support**
- **Animated desktop** with Matrix rain, prayer flags, moire textures
- **ArtMaker canvas engine** and BookWriter tool
- **Built-in terminal** and web browser panels

---

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Desktop Framework | Tauri | v2 |
| Backend Language | Rust | 2021 edition |
| Frontend Framework | Vue 3 (Composition API) | ^3.5.13 |
| State Management | Pinia | ^3.0.2 |
| Type System | TypeScript | ^5.8.3 |
| Build Tool | Vite + Vite WASM plugin | ^6.3.5 |
| Icons | Iconify Vue | ^4.3.0 |
| Maps | MapLibre GL | ^5.4.0 |
| Animation | GSAP | ^3.15.0 |
| Database | redb | 2.x |
| Full-Text Search | Tantivy | 0.22 |
| Compression | lz4_flex + zstd + brotli | 0.11 / 0.13 / 7 |
| Post-Quantum Crypto | pqcrypto-mlkem + ml-dsa | 0.1 / 0.1.1 |
| Symmetric Crypto | ChaCha20Poly1305 | 0.10 |
| X25519 KEX | x25519-dalek | 2.x |
| Password Hashing | argon2 | 0.5 |
| Hashing | BLAKE3 | 1 |
| Content ID | UUID v4 | 1 |
| JWT | jsonwebtoken | 9 |
| ML Inference | ort (ONNX Runtime) | 2.0.0-rc.12 |
| Code Parsing | tree-sitter | 0.24 |
| EXIF | kamadak-exif | 0.5 |
| Image Processing | image | 0.25 |
| MIME Detection | infer + mime_guess | 0.16 / 2 |
| CLI/TUI | clap + ratatui | 4 / 0.29 |
| WASM | wasm-bindgen + wasm-pack | 0.2 |
| Async Runtime | tokio | 1.x |
| HTTP Client | reqwest (rustls-tls) | 0.12 |
| HTML Parsing | scraper | 0.21 |
| Mega SDK | megalib | 0.11 |
| System Info | sysinfo | 0.31 |
| Directory Walk | walkdir | 2.5 |
| Parallelism | rayon | 1.10 |

---

## Project Structure

```
cybermanju-drive/
├── crates/                                  # 17 Rust workspace crates
│   ├── types/                               # src/schema.rs + sync.rs — shared data types
│   ├── crypto/                              # src/pqc.rs — ML-KEM, ML-DSA, ChaCha20Poly1305
│   ├── compression/                         # LZ4→Zstd→Brotli triple-layer pipeline
│   ├── search/                              # Tantivy full-text search index
│   ├── db/                                  # redb embedded database (21 tables)
│   ├── portable-db/                         # .cybermanju portable database format
│   ├── web/                                 # Browser engine, DuckDuckGo search, HTML renderer
│   ├── faces/                               # Face detection & 4 clustering algorithms
│   ├── backends/                            # 10 storage backends (Local, GitHub, GitLab, Codeberg, Gitea, GDrive, GPhotos, Telegram, Mega + Git LFS)
│   ├── cli/                                 # CLI/TUI (clap + ratatui)
│   ├── drive-wasm/                          # WASM bridge (crypto, compression, drive, sync)
│   └── tests/                               # Integration tests
├── src/                                     # Vue 3 + TypeScript frontend
│   ├── components/                          #
│   │   ├── ui/                              # 25 Os-prefixed UI primitives
│   │   ├── FileExplorer.vue                 # File browser (grid/list/masonry)
│   │   ├── FilePreview.vue                  # File preview panel
│   │   ├── DesktopShell.vue                 # Root desktop window manager
│   │   ├── SyncPanel.vue                    # Cloud sync configuration
│   │   ├── CompressionPanel.vue             # Compression controls
│   │   ├── EncryptionPanel.vue              # PQC key management
│   │   ├── FaceGroupingPanel.vue            # Face clustering UI
│   │   ├── MapView.vue                      # GPS map with geo-markers
│   │   ├── WebBrowserPanel.vue              # Built-in web browser
│   │   ├── CodeIntelligencePanel.vue        # Symbol extraction viewer
│   │   ├── Terminal.vue                     # Built-in terminal
│   │   ├── TransferWindow.vue               # Cross-backend file transfer
│   │   └── ... (80+ total components)
│   ├── composables/                         #
│   │   ├── useTauri.ts                      # Triple-mode IPC bridge (Tauri/WASM/REST)
│   │   ├── useWindowManager.ts              # Window position/size/focus
│   │   ├── useLogin.ts                      # Authentication state
│   │   ├── useShortcuts.ts                  # Keyboard shortcuts
│   │   ├── useGsapAnimation.ts              # GSAP animation control
│   │   ├── useDrag.ts                       # Drag-and-drop
│   │   ├── useSwipe.ts                      # Touch swipe gestures
│   │   └── ... (15 total)
│   ├── stores/                              #
│   │   ├── app.ts                           # Pinia store — all application state
│   │   └── history.ts                       # File operation undo history
│   ├── wasm/                                # WASM bridge modules
│   │   ├── bridge.ts                        # Module loader & lifecycle
│   │   ├── crypto.ts                        # ChaCha20, ML-DSA65, ML-KEM1024, X25519
│   │   ├── compression.ts                   # LZ4/Brotli/Zstd
│   │   ├── drive.ts                         # VirtualDrive + IndexedDB
│   │   ├── storage.ts                       # IndexedDB KV store
│   │   ├── sync.ts                          # Sync engine
│   │   ├── oauth.ts                         # OAuth flow
│   │   ├── transfer.ts                      # Cross-backend transfer
│   │   ├── native-fs.ts                     # File System Access API
│   │   └── data.ts                          # IndexedDB data layer
│   ├── types/index.ts                       # TypeScript type definitions
│   ├── configs/                             # artMaker.ts, windowMenus.ts
│   └── directives/clickOutside.ts           # Click-outside directive
├── src-tauri/                               # Tauri v2 desktop app
│   ├── tauri.conf.json                      # Tauri configuration
│   ├── capabilities/default.json            # Permissions
│   └── src/
│       ├── main.rs                          # Process entry point
│       ├── lib.rs                           # AppState, 60+ IPC handlers, plugins
│       ├── commands/                        # 23 command modules
│       │   ├── files.rs                     # File CRUD, folder ops, loose groups
│       │   ├── encryption.rs                # PQC encrypt/decrypt/keygen
│       │   ├── compression.rs               # Compress/decompress/stats
│       │   ├── search.rs                    # Tantivy search & suggest
│       │   ├── faces.rs                     # Face detection & clustering
│       │   ├── sync.rs                      # Sync config & pipeline
│       │   ├── portable_db.rs               # .cybermanju operations
│       │   ├── accounts.rs                  # Storage accounts
│       │   ├── collections.rs               # Collection CRUD
│       │   ├── users.rs                     # Auth, RBAC, permissions
│       │   ├── trash.rs                     # Trash/recycle bin
│       │   ├── versions.rs                  # File versioning
│       │   ├── share.rs                     # Share links
│       │   ├── import.rs                    # File import & scan
│       │   ├── transfer.rs                  # Cross-backend transfer
│       │   ├── web.rs                       # Web search & fetch
│       │   ├── map.rs                       # GPS/EXIF
│       │   ├── batch.rs                     # Batch operations
│       │   ├── audit.rs                     # Audit log
│       │   ├── duplicates.rs                # Duplicate detection
│       │   ├── diagnostics.rs              # Crash log
│       │   ├── kv.rs                        # Key-value store
│       │   ├── dashboard.rs                 # Dashboard lifecycle
│       │   └── system_info.rs               # OS/hardware info
│       ├── db/schema.rs                     # redb wrapper
│       ├── crypto/mod.rs                    # Tauri crypto module
│       ├── compression/mod.rs               # Tauri compression module
│       ├── search/mod.rs                    # Tauri search module
│       ├── sync/                            # Sync pipeline, OAuth, backends
│       ├── transfer/mod.rs                  # Transfer state
│       ├── preview/mod.rs                   # Thumbnail generation
│       ├── faces/mod.rs                     # Face pipeline
│       ├── tree_sitter/mod.rs               # Code parsing
│       └── web_dashboard/mod.rs             # Embedded HTTP server (port 3456)
├── architecture.jsonl                       # Machine-readable architecture context
├── ARCHITECTURE.md                          # Full architecture documentation (799 lines)
├── worklog.md                               # Development work log
├── tasks.md                                 # Development tasks
├── system.md                                # System prompt
├── AGENTS.md                                # AI agents guide
├── Dockerfile                               # Multi-stage Docker build
├── docker-compose.yml                       # ZimaOS-compatible Compose
├── docker/server/                           # Standalone Docker server
├── .github/workflows/ci.yml                 # CI/CD pipeline
├── aur/                                     # Arch Linux PKGBUILD
└── keymaps/                                 # Keyboard shortcut keymaps
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          CYBERMANJU DRIVE                               │
│                                                                         │
│  ┌──────────────────────────┐    ┌─────────────────────────────────┐   │
│  │    FRONTEND (Vue 3 + TS)  │    │        BACKEND (Rust)           │   │
│  │                           │    │                                 │   │
│  │  ┌─────────────────────┐  │    │  ┌───────────────────────────┐  │   │
│  │  │    80+ Components   │  │    │  │   Tauri IPC Handlers      │  │   │
│  │  │  (25 UI + 55 App)   │  │    │  │   (commands/*.rs — 23     │  │   │
│  │  └─────────┬───────────┘  │    │  │    modules, 60+ commands) │  │   │
│  │            │              │    │  └───────────┬───────────────┘  │   │
│  │  ┌─────────▼───────────┐  │    │              │                 │   │
│  │  │    Pinia Stores      │  │    │  ┌───────────▼─────────────┐  │   │
│  │  │  (app + history)     │  │    │  │     Workspace Crates    │  │   │
│  │  └─────────┬───────────┘  │    │  │                         │  │   │
│  │            │              │    │  │  ┌──────┐ ┌──────────┐  │  │   │
│  │  ┌─────────▼───────────┐  │    │  │  │ db/  │ │ crypto/  │  │  │   │
│  │  │    useTauri.ts       │◄─┼────┼──┼─►│ redb │ │ PQC      │  │  │   │
│  │  │  Triple-mode IPC    │  │    │  │  │ 21   │ │ ML-KEM   │  │  │   │
│  │  │  Tauri / WASM / REST│  │    │  │  │ tbls │ │ ML-DSA   │  │  │   │
│  │  └─────────┬───────────┘  │    │  │  └──────┘ └──────────┘  │  │   │
│  │            │              │    │  │  ┌──────┐ ┌──────────┐  │  │   │
│  │  ┌─────────▼───────────┐  │    │  │  │search│ │compress/ │  │  │   │
│  │  │     WASM Bridge     │  │    │  │  │Tantivy││LZ4→Zstd  │  │  │   │
│  │  │  (11 modules)       │  │    │  │  │BM25   │ │→Brotli   │  │  │   │
│  │  │  IndexedDB + Crypto │  │    │  │  └──────┘ └──────────┘  │  │   │
│  │  └─────────┬───────────┘  │    │  │  ┌──────┐ ┌──────────┐  │  │   │
│  │            │              │    │  │  │sync/ │ │ faces/   │  │  │   │
│  │  ┌─────────▼───────────┐  │    │  │  │7 bknds││4 cluster │  │  │   │
│  │  │   REST API Client    │  │    │  │  └──────┘ └──────────┘  │  │   │
│  │  └─────────┬───────────┘  │    │  └──────────────────────────┘  │   │
│  │            │              │    │                                 │   │
│  └────────────┼──────────────┘    └─────────────────────────────────┘   │
│               │                                                        │
│     ┌─────────┴──────────┐                                             │
│  Desktop  Browser  WASM/Pages  Docker/ZimaOS                            │
│  (Tauri)  (REST)    (SPA)    (0.0.0.0:3456)                            │
└─────────────────────────────────────────────────────────────────────────┘
```

### Runtime Modes

| Mode | Entry Point | Transport | Use Case |
|------|------------|-----------|----------|
| **Tauri Desktop** | `src-tauri/src/main.rs` | Tauri IPC (Conduit 2.4x) | Native desktop with full filesystem access |
| **Web Dashboard** | Embedded HTTP on `0.0.0.0:3456` | REST API (fetch) | Browser from any device; Docker/ZimaOS |
| **WASM/GitHub Pages** | `dist-wasm/` static files | SPA + WASM + IndexedDB | Public showcase, no backend needed |

The composable `src/composables/useTauri.ts` auto-detects environment via `window.__TAURI__`
and routes commands through the appropriate transport with automatic fallback.

---

## Cross-Crate Dependency Graph

```
cybermanju-types (root types — used by ALL)
  ├── cybermanju-crypto (PQC engine)
  │     ├── cybermanju-portable-db
  │     └── cybermanju-drive-wasm (standalone pure-Rust)
  ├── cybermanju-compression (LZ4→Zstd→Brotli)
  │     ├── cybermanju-portable-db
  │     ├── cybermanju-drive-wasm
  │     └── cybermanju-cli
  ├── cybermanju-search (Tantivy)
  │     ├── cybermanju-web
  │     └── src-tauri
  ├── cybermanju-db (redb)
  │     ├── cybermanju-portable-db
  │     ├── cybermanju-cli
  │     └── src-tauri
  ├── cybermanju-web (browser + DuckDuckGo)
  ├── cybermanju-faces (face clustering)
  │     └── src-tauri
  ├── cybermanju-backends (10 storage backends)
  │     ├── cybermanju-cli
  │     └── src-tauri
  ├── cybermanju-portable-db (.cybermanju format)
  │     ├── cybermanju-cli
  │     └── src-tauri
  ├── cybermanju-cli (CLI/TUI) — standalone binary
  ├── cybermanju-tests (integration tests)
  └── src-tauri — ORCHESTRATES ALL CRATES
```

---

## Data Flow

```
Import / Scan Directory
  │
  ▼
FileNode stored in redb (files table) + Tantivy index (add_document)
  │
  ├──▶ Encrypt (optional)
  │     ML-KEM encapsulate → HKDF derive key → ChaCha20Poly1305 → .enc.meta.json
  │     Or sign: ML-DSA sign_message
  │
  ├──▶ Compress (optional)
  │     LZ4 → Zstd → Brotli cascade → CompressionStats reported
  │
  ├──▶ Face Detection
  │     Extract embeddings → SimHash index → Auto-select cluster algorithm → FaceGroup stored
  │
  ├──▶ Sync to Backends
  │     Compress → Generate preview → Upload → Link → (optionally) delete raw
  │     FileRelation stored in redb
  │
  ├──▶ Portable DB Recovery
  │     Triple-compress file → BLAKE3-dedup → Store as .cyb3 blob → RecoveryEntry
  │
  ├──▶ Trash
  │     TrashItem in redb → FileNode removed from active listing → Restorable
  │
  ├──▶ Share
  │     Generate random token → ShareLink in redb → /api/shared/{token}
  │
  └──▶ Version
        FileVersion created with snapshot data → Revert possible
```

---

## Quick Start

### Prerequisites

- **Node.js** 20+
- **Rust** 1.85+ (via [rustup](https://rustup.rs/))
- **Platform dependencies:**
  - **Linux (Debian/Ubuntu):** `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`
  - **Linux (Arch/CachyOS):** `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `librsvg`, `libsoup3`, `pkg-config`, `base-devel`, `openssl`
  - **macOS:** Xcode Command Line Tools
  - **Windows:** WebView2 Runtime (usually pre-installed)

### Install & Run

```bash
# Clone the repository
git clone https://github.com/cybermanju/cybermanju-drive.git
cd cybermanju-drive

# Install frontend dependencies
npm install

# Run in development mode (Tauri desktop + Vite HMR)
npm run tauri:dev
```

The app will open a native window. The web dashboard simultaneously starts on `http://localhost:3456`.

---

## Build Instructions

### Desktop App

```bash
# Type-check frontend
npm run typecheck

# Check Rust code
npm run rust:check

# Lint Rust code
npm run rust:clippy

# Build production desktop app
npm run tauri:build

# Build debug desktop app (faster, larger)
npm run tauri:build:debug
```

Output installers are in `src-tauri/target/release/bundle/`.

#### CachyOS / Arch Linux

```bash
# Install system dependencies
sudo pacman -S \
  webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg libsoup3 \
  pkg-config base-devel openssl nodejs npm

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build
npm install
npm run tauri:build
```

Or install from AUR:

```bash
cd aur/
makepkg -si
# Or with an AUR helper:
yay -S cybermanju-drive
```

### Docker Image

```bash
# Build the multi-stage Docker image
docker build -t cybermanju-drive:latest .

# Run with Docker Compose
docker compose up -d

# Access at http://localhost:3456
```

The container runs as a non-root user with persistent data in `/data`.

### WASM / GitHub Pages

```bash
# Build Rust WASM module
npm run wasm:build-rust

# Build frontend for web deployment
npm run build:wasm
```

Output is in `dist-wasm/`. The CI pipeline automatically deploys to GitHub Pages on push to `main`.

---

## ZimaOS Installation

Cybermanju Drive is packaged as a ZimaOS App Store application with full x-casaos metadata:

1. **Add the app** to your ZimaOS instance via the App Store, or deploy manually with:
   ```bash
   docker compose up -d
   ```
2. The container maps persistent data to `/DATA/AppData/cybermanju-drive/config`
3. Access the web dashboard at `http://<your-nas-ip>:3456`
4. Supported architectures: **amd64** and **arm64**

### ZimaOS Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Rust log level |
| `PORT` | `3456` | Web dashboard port |
| `DB_PATH` | `/data/cybermanju.db` | Database file path |
| `STATIC_DIR` | `/app/static` | Vue frontend static files |
| `TZ` | `UTC` | Timezone |

---

## API Documentation

### Tauri IPC Commands

60+ IPC commands across 23 modules, registered in `src-tauri/src/lib.rs`:

| Module | Key Commands |
|--------|-------------|
| `files` | list_files, get_file, create_folder, delete_file, rename_file, move_file, get_preview |
| `search` | search_files, search_files_paginated, suggest |
| `encryption` | encrypt_file, decrypt_file, get_encryption_status, generate_keypair, list_keys |
| `compression` | compress_file, decompress_file, get_compression_stats |
| `faces` | detect_faces, recluster_faces, rename_face_group, merge_face_groups, find_similar_faces |
| `sync` | create_sync_config, start_sync, get_sync_progress, test_sync_connection, cancel_sync |
| `portable_db` | sync_portable_db, record_file_relation, store_compressed_for_recovery, repack_portable_db |
| `users` | register_user, authenticate_user, grant_file_permission, verify_file_access |
| `trash` | list_trash, restore_from_trash, empty_trash |
| `versions` | list_file_versions, create_file_version, revert_file_version, snapshot_all_versions |
| `transfer` | transfer_files, get_transfer_progress, cancel_transfer |

### Web Dashboard REST API

When running as a Docker container or with the embedded web dashboard:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/files` | GET | List all files |
| `/api/files/{id}` | GET | Get file by ID |
| `/api/files/{id}` | DELETE | Delete a file |
| `/api/accounts` | GET | List storage accounts |
| `/api/collections` | GET | List collections |
| `/api/collection-items` | GET | List collection items |
| `/api/face-groups` | GET | List face groups |
| `/api/loose-groups` | GET | List loose groups |
| `/api/encryption/status` | GET | Encryption engine status |
| `/api/encryption/keys` | GET | List encryption keys |
| `/api/geo-files` | GET | Files with GPS coordinates |
| `/api/search?q={term}` | GET | Search files |
| `/api/locations` | GET | List locations |
| `/api/users` | GET | List users |
| `/api/users/register` | POST | Register user |
| `/api/users/login` | POST | Login |
| `/api/permissions/{fileId}` | GET | Get file permissions |
| `/api/permissions` | POST | Grant permission |
| `/api/permissions/verify` | POST | Verify access |

Full REST API documentation with request/response schemas is in [ARCHITECTURE.md](./ARCHITECTURE.md#10-web-dashboard-rest-api).

---

## Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server (frontend only) |
| `npm run tauri:dev` | Start Tauri desktop app with HMR |
| `npm run build` | Type-check + build frontend |
| `npm run tauri:build` | Build production desktop installer |
| `npm run build:wasm` | Build frontend for web/GitHub Pages |
| `npm run wasm:build-rust` | Build Rust WASM module via wasm-pack |
| `npm run typecheck` | TypeScript type checking |
| `npm run rust:check` | Cargo check (all crates) |
| `npm run rust:clippy` | Cargo clippy lints |
| `npm run rust:fmt` | Check Rust formatting |
| `npm run rust:test` | Run Rust tests |
| `npm run check:all` | TypeScript + cargo check + clippy |
| `npm run clean` | Remove build artifacts and database |
| `npm run db:reset` | Remove database and search index |

---

## License

MIT
