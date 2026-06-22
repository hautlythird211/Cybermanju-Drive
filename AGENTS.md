# Cybermanju Drive — Agent Guide

## Project Overview

Cybermanju Drive is a quantum-resistant encrypted file manager with AI face grouping, triple-layer compression, full-text search, GPS map view, multi-user access control, and cross-platform cloud sync. It runs as a Tauri v2 desktop app, a Docker container with Web Dashboard, or a WASM SPA on GitHub Pages.

## Repository Structure

```
cybermanju-drive/
├── crates/                          # 17 Rust workspace crates
│   ├── types/                       # Shared data types (FileNode, User, etc.)
│   ├── crypto/                      # PQC engine: ML-KEM, ML-DSA, ChaCha20Poly1305
│   ├── compression/                 # Triple-layer: LZ4 → Zstd → Brotli
│   ├── search/                      # Tantivy full-text search index
│   ├── db/                          # redb embedded database (21 tables)
│   ├── portable-db/                 # .cybermanju compressed/encrypted portable DB
│   ├── web/                         # Browser engine, DuckDuckGo search, HTML renderer
│   ├── faces/                       # Face detection & clustering (4 algorithms)
│   ├── backends/                    # Storage backends: Local, GitHub, GitLab, Codeberg, Gitea, GDrive, GPhotos, Telegram, Mega + Git LFS client + repo layout manager
│   ├── cli/                         # CLI/TUI app (clap + ratatui)
│   ├── drive-wasm/                  # WASM bridge (crypto, compression, virtual drive, sync)
│   └── tests/                       # Integration tests
├── src/                             # Vue 3 + TypeScript frontend
│   ├── components/                  # 80+ Vue components (25 UI primitives + 55 app panels)
│   ├── composables/                 # 15 composables (useTauri, useWindowManager, etc.)
│   ├── stores/                      # Pinia stores (app, history)
│   ├── wasm/                        # WASM bridge modules (bridge, crypto, compression, drive, sync, storage, etc.)
│   └── types/                       # TypeScript type definitions
├── src-tauri/                       # Tauri v2 desktop app
│   └── src/
│       ├── lib.rs                   # Core orchestrator: AppState, 60+ IPC handlers, plugins, web dashboard
│       ├── commands/                # 23 command modules
│       ├── sync/                    # Sync pipeline + OAuth + backends
│       ├── web_dashboard/           # Embedded HTTP server on port 3456
│       └── ...
├── architecture.jsonl               # Machine-readable architecture context (for AI)
├── ARCHITECTURE.md                  # Human-readable architecture docs
├── Dockerfile                       # Multi-stage Docker build
├── docker-compose.yml               # ZimaOS-compatible Compose
└── README.md                        # Project README
```

## Key Architectural Patterns

### Triple-Mode IPC (`useTauri.ts`)
The frontend communicates with the backend via a three-tier fallback:
1. **Tauri Conduit** (2.4x faster IPC) — when running as desktop app with `window.__TAURI__`
2. **WASM bridge** (IndexedDB) — when in browser with WASM support
3. **REST API** (HTTP fetch) — when connected to the Web Dashboard on port 3456

### Cross-Crate Dependencies
- `cybermanju-types` is the root types crate, used by every other crate
- `cybermanju-portable-db` aggregates types + crypto + compression + db
- `src-tauri` depends on ALL workspace crates and orchestrates them
- `cybermanju-drive-wasm` is standalone (pure Rust deps, no C bindings)

### Data Flow
```
Import → FileNode in redb + Tantivy index → Encrypt (ML-KEM) → Compress (LZ4→Zstd→Brotli) 
→ Face detect (auto-cluster) → Sync to backends (compress→preview→upload→link) 
→ Portable DB (compressed recovery blobs) → Trash/Share/Version
```

### Tauri Commands → Crate Mapping
Each command module in `src-tauri/src/commands/` delegates to the corresponding workspace crate:
- `commands/files.rs` → `cybermanju-db` (Database methods)
- `commands/encryption.rs` → `cybermanju-crypto` (PqcEngine)
- `commands/compression.rs` → `cybermanju-compression` (TripleCompressor)
- `commands/search.rs` → `cybermanju-search` (SearchIndex)
- `commands/faces.rs` → `cybermanju-faces` (clustering)
- `commands/sync.rs` → `cybermanju-backends` (StorageBackend impls incl. GitHub/GitLab/Codeberg/Gitea, Git LFS, repo layout)
- `commands/portable_db.rs` → `cybermanju-portable-db` (PortableDatabase)

## Build & Test Commands

```bash
# Type-check frontend
npm run typecheck

# Check Rust (all crates)
npm run rust:check

# Lint Rust
npm run rust:clippy

# Run all Rust tests
npm run rust:test

# TypeScript + Rust check + Clippy
npm run check:all

# Build desktop app
npm run tauri:build

# Build WASM for web
npm run build:wasm

# Build Docker image
docker build -t cybermanju-drive:latest .
```

## Key Files for AI Understanding

| File | Purpose |
|------|---------|
| `architecture.jsonl` | Machine-readable architecture context (JSONL) |
| `ARCHITECTURE.md` | Human-readable architecture documentation |
| `src-tauri/src/lib.rs` | AppState, command registration, plugin setup |
| `src/composables/useTauri.ts` | Triple-mode IPC bridge |
| `src/wasm/bridge.ts` | WASM module loader |
| `src/stores/app.ts` | Central Pinia store |
| `crates/types/src/schema.rs` | All shared data types |
| `crates/crypto/src/pqc.rs` | PQC engine implementation |
| `crates/db/src/database.rs` | Database schema and operations |
| `crates/portable-db/src/lib.rs` | Portable database format |
| `crates/backends/src/github.rs` | GitHub + Git LFS backend |
| `crates/backends/src/gitlab.rs` | GitLab + Git LFS backend |
| `crates/backends/src/codeberg.rs` | Codeberg (Forgejo) backend |
| `crates/backends/src/gitea.rs` | Gitea/Forgejo self-hosted backend |
| `crates/backends/src/git_lfs.rs` | Git LFS batch API client |
| `crates/backends/src/repo_layout.rs` | .cybermanju repo structure manager |

## Code Style
- Rust: 2021 edition, serde camelCase rename for all API types
- TypeScript: Strict mode, camelCase, Vue 3 Composition API with `<script setup>`
- Frontend: Pinia stores, composables for reusable logic, Os-prefixed UI components
- Errors: `anyhow::Result` in Rust, descriptive `Error` throws in TypeScript
