# Cybermanju Drive v0.0.1

**Quantum-resistant encrypted file manager** — desktop, mobile, web, Docker, and headless server.

> **⚠️ Early Access / Pre-Release Notice**  
> This is an early public release. Many features are functional but not yet production-hardened.  
> Enterprise-grade stability, formal security audits, and a plugin SDK are on the roadmap.

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
  - [Post-Quantum Cryptography](#post-quantum-cryptography)
  - [Compression Pipeline](#compression-pipeline)
  - [Full-Text Search](#full-text-search)
  - [Sync Engine & Cloud Backends](#sync-engine--cloud-backends)
  - [Face Detection & Clustering](#face-detection--clustering)
  - [Embedded Web Dashboard](#embedded-web-dashboard)
  - [Multi-User Access Control](#multi-user-access-control)
  - [Geolocation / GPS Map](#geolocation--gps-map)
  - [File Versioning & Audit Log](#file-versioning--audit-log)
  - [Portable Database (.cybermanju)](#portable-database-cybermanju)
  - [Cross-Platform WASM Bridge](#cross-platform-wasm-bridge)
- [Platform Support & Bundles](#platform-support--bundles)
- [Sync Backend Details](#sync-backend-details)
- [Current Limitations](#current-limitations)
- [Roadmap](#roadmap)
- [Installation](#installation)
- [Building from Source](#building-from-source)

---

## Overview

Cybermanju Drive is an end-to-end encrypted file management platform built with Rust and Vue 3. It combines post-quantum cryptography (NIST FIPS 203/204), multi-algorithm compression, AI face clustering, full-text search, and a modular sync engine that connects to 7 cloud backends — all in a single self-contained binary.

The architecture is a **monorepo with 12 Rust workspace crates** plus a TypeScript/Vue 3 frontend, communicating via Tauri v2 IPC. A separate WASM bridge crate makes the core engine available in the browser without the desktop shell.

---

## Features

### Post-Quantum Cryptography

| Algorithm | Standard | Type | Status |
|-----------|----------|------|--------|
| ML-KEM-768 (Kyber) | NIST FIPS 203 | Key Encapsulation | ✅ Implemented |
| ML-KEM-1024 (Kyber) | NIST FIPS 203 | Key Encapsulation | ✅ Implemented |
| ML-DSA-44 (Dilithium) | NIST FIPS 204 | Digital Signature | ✅ Implemented |
| ML-DSA-65 (Dilithium) | NIST FIPS 204 | Digital Signature | ✅ Implemented |
| ML-DSA-87 (Dilithium) | NIST FIPS 204 | Digital Signature | ✅ Implemented |
| X25519 + ML-KEM Hybrid | — | Hybrid PQ/Classical | ✅ Implemented |
| ChaCha20-Poly1305 | — | Symmetric AEAD | ✅ Implemented |
| SLH-DSA (SPHINCS+) | NIST FIPS 205 | Digital Signature | 🔜 Planned |
| FrodoKEM-1344 | — | Key Encapsulation | 🔜 Planned |

All PQ operations use the `pqcrypto-mlkem` and `pqcrypto-mldsa` crates (FIPS 203/204 certified implementations).

### Compression Pipeline

| Mode | Algorithm(s) | Use Case |
|------|-------------|----------|
| **LZ4** | LZ4-fast | Speed-optimized, low-latency |
| **Zstd** | Zstd (levels 1–22) | Balanced compression |
| **Brotli** | Brotli (level 11) | Maximum compression ratio |
| **Triple** | LZ4 → Zstd-15 → Brotli-11 | Ultra-compression for archival |

Per-layer statistics (bytes in/out, ratio, time) are tracked for transparency.

### Full-Text Search

Tantivy-based search engine indexing:
- File names
- File content text
- Tags and metadata
- BM25 ranking
- Levenshtein fuzzy matching
- Faceted search filtering
- Term completions / autocomplete

### Sync Engine & Cloud Backends

Orchestrates: **scan → compress → preview → upload → link → clean** in parallel via Rayon.

| Backend | API | Auth | File Size Limit | Status |
|---------|-----|------|----------------|--------|
| **Local** | Filesystem | — | Unlimited | ✅ |
| **GitHub** | Contents API + Releases | OAuth token | 25 MB (API) / 2 GB (Releases) | ✅ |
| **GitLab** | API v4 | OAuth token | Project-dependent | ✅ |
| **Google Drive** | Drive API v3 | OAuth 2.0 refresh | 5 TB per file | ✅ |
| **Google Photos** | Photos Library API | OAuth 2.0 refresh | 200 MB per photo / 10 GB per video | ✅ (media only) |
| **Telegram** | Bot API | Bot token | 50 MB per file | ✅ |
| **Mega** | Mega.nz API | Email + password | 20 GB free tier | ✅ (partial — upload only) |

Progress tracking, cancellation, and multi-file parallel sync are supported.

### Face Detection & Clustering

Four clustering algorithms (all functional):
1. **BruteForce** — Exact nearest-neighbor, O(n²), guaranteed optimal
2. **SimHash** — Approximate binary hashing, fast large-scale matching
3. **Chinese Whispers** — Graph-based label propagation, no threshold tuning needed
4. **HDBSCAN** — Hierarchical density-based, handles noise/outliers

> **Note:** The current detection uses BLAKE3 perceptual hashing (pseudo-embeddings) as a stub.  
> True ONNX SCRFD model inference is implemented behind the `onnx-face` feature flag but requires the model weights file to be placed in the app data directory.

### Embedded Web Dashboard

A lightweight HTTP REST server (`crates/web`) built on `std::net::TcpListener` with:
- JWT authentication
- IP-based rate limiting
- Full CRUD API for files, collections, users, sync configs
- Serves the Vue 3 frontend as static files
- Binds to `127.0.0.1` by default (configurable via `DOCKER_MODE` for container deployments)

### Multi-User Access Control

Role-based system with three levels:
- **Admin** — Full system access
- **User** — Read/write own files
- **Viewer** — Read-only access to shared files

File permissions can be granted/revoked per user.

### Geolocation / GPS Map

Extracts EXIF GPS data from photos and displays on an interactive MapLibre GL map. Supports markers, clustering, and region-based filtering.

### File Versioning & Audit Log

- **Versions:** Snapshot-based file versioning with create/revert/list operations
- **Audit Log:** Immutable append-only activity log for compliance and forensics
- **Trash:** Soft-delete with restore and permanent delete
- **History:** Undo/redo for 20+ action types

### Portable Database (.cybermanju)

Self-contained `.cybermanju` format for portability:
- Encrypted/compressed redb snapshot
- Triple-compressed (LZ4→Zstd→Brotli) blob
- Sidecar `.cyberspace.blobs/` directory for content and preview blobs
- Designed for USB drives, cloud sync, and offline backup

### Cross-Platform WASM Bridge

The `crates/drive-wasm` crate compiles to WebAssembly, making the core crypto, compression, and sync engine available directly in the browser for web deployments.

---

## Platform Support & Bundles

| Platform | Bundle Type | CI Status |
|----------|-------------|-----------|
| **Windows** | `.msi` (MSI installer), `.exe` (NSIS installer) | ✅ |
| **Linux (Debian/Ubuntu)** | `.deb` package | ✅ |
| **Linux (Fedora/RHEL)** | `.rpm` package | ✅ |
| **Linux (Universal)** | `.AppImage` | ✅ |
| **Linux (Flatpak)** | `.flatpak` bundle | ✅ |
| **Linux (Arch/CachyOS)** | AUR-compatible (AppImage + deb) | ✅ |
| **macOS** | `.dmg` disk image, `.app` bundle | ✅ |
| **Android** | `.apk` (universal unsigned) | ✅ |
| **Web (WASM)** | Static files for any web server | ✅ |
| **Docker** | `docker pull hautlythird211/cybermanju-drive` | ✅ |
| **Headless Server** | Standalone Rust binary (no GUI deps) | ✅ |

> **Note:** macOS, Windows, and Android bundles are built in CI but signed builds require setting `TAURI_SIGNING_PRIVATE_KEY` in repository secrets.

---

## Sync Backend Details

### Local
- Source/destination paths on the local filesystem
- Supports copy, move, hardlink, symlink

### GitHub
- OAuth app authentication
- Uploads files to repository contents API
- Large files (>25 MB) use GitHub Releases API
- Supports public and private repos

### GitLab
- OAuth app authentication (GitLab API v4)
- File operations on project repositories
- Personal access token fallback

### Google Drive
- OAuth 2.0 with refresh token rotation
- Folder hierarchy CRUD
- Upload/download with resumable sessions
- MIME type detection

### Google Photos
- OAuth 2.0 with refresh token rotation
- Media-only upload (photos/videos)
- Album management
- 200 MB photo / 10 GB video limit

### Telegram
- Bot API integration
- Chat/channel/group uploads
- 50 MB file size limit
- No download support yet

### Mega
- Mega.nz API (email + password auth)
- Encrypted cloud storage
- 20 GB free tier
- Upload-only currently

---

## Current Limitations

1. **Windows Installed App:** Previous versions used relative paths (`"cybermanju.db"`) which failed when installed in `Program Files`. Fixed in this release — app data now lives in `%LOCALAPPDATA%/cybermanju-drive/`.
2. **macOS Signing:** Notaris not configured — Gatekeeper may block the unsigned `.app`.
3. **Android:** APK is unsigned — must enable "Install unknown apps" on device.
4. **WASM Build:** Requires `wasm-pack` to be installed in CI (fixed in this release pipeline).
5. **ONNX Face Detection:** Model file not bundled due to licensing — must be downloaded separately.
6. **SLH-DSA / SPHINCS+:** Not yet implemented (FIPS 205).
7. **FrodoKEM:** TypeScript type exists in the frontend but no Rust implementation yet.
8. **Mega Backend:** Download and full CRUD not yet implemented.
9. **Plugin System:** No user-extensibility mechanism currently exists.

---

## Roadmap

### Short Term (v0.0.2)
- [ ] Plugin SDK: WASM-based plugin runtime for sync backends, crypto providers, compression algorithms
- [ ] macOS code signing + notarization
- [ ] Windows code signing (Authenticode)
- [ ] Android APK signing for Play Store

### Medium Term (v0.1.0)
- [ ] SLH-DSA (SPHINCS+) implementation (NIST FIPS 205)
- [ ] FrodoKEM-1344 implementation
- [ ] Mega backend: full CRUD (download, delete, list)
- [ ] Telegram download support
- [ ] WebDAV server mode
- [ ] S3-compatible storage backend
- [ ] SSH/SFTP sync backend
- [ ] Formal cryptography audit
- [ ] Flatpak publishing on Flathub

### Long Term (v1.0.0+)
- [ ] **Plugin System v1** — Dynamic plugin loading with WASM sandboxing:
  - Custom sync backends (write a WASM module, drop it in, it works)
  - Custom compression algorithms
  - Custom encryption providers
  - Custom file preview generators
  - Custom metadata extractors
- [ ] Encrypted peer-to-peer sync (libp2p / wormhole)
- [ ] Distributed storage network (filecoin / IPFS integration)
- [ ] End-to-end encrypted collaboration (shared folders with invite codes)
- [ ] Mobile native apps (iOS + Android with full offline support)
- [ ] FUSE filesystem mount
- [ ] CLI tool (`cmj` command)
- [ ] Homebrew tap for macOS
- [ ] Windows Package Manager (winget) support

---

## Installation

### Windows
1. Download `Cybermanju.Drive_0.0.1_x64-setup.exe` or `Cybermanju.Drive_0.0.1_x64_en-US.msi`
2. Run the installer (SmartScreen may show a warning — click "Run anyway")
3. App data is stored at `%LOCALAPPDATA%/cybermanju-drive/`

### Linux (Debian/Ubuntu)
```bash
sudo dpkg -i Cybermanju.Drive_0.0.1_amd64.deb
```

### Linux (Fedora/RHEL)
```bash
sudo rpm -i Cybermanju.Drive-0.0.1-1.x86_64.rpm
```

### Linux (Universal AppImage)
```bash
chmod +x Cybermanju.Drive_0.0.1_amd64.AppImage
./Cybermanju.Drive_0.0.1_amd64.AppImage
```

### Linux (Flatpak)
```bash
flatpak install cybermanju-drive.flatpak
```

### macOS
1. Download the `.dmg`
2. Open and drag `Cybermanju Drive.app` to Applications
3. Right-click → Open (first launch bypasses Gatekeeper)

### Android
1. Download `app-universal-release-unsigned.apk`
2. Enable "Install from unknown sources"
3. Install the APK

### Docker
```bash
docker run -d \
  -p 3456:3456 \
  -v cybermanju-data:/data \
  hautlythird211/cybermanju-drive:latest
```

---

## Building from Source

### Prerequisites
- Rust 1.85+ (stable)
- Node.js 22+
- wasm-pack
- System libs: webkit2gtk-4.1, libgtk-3, librsvg2, libsoup-3.0, libjavascriptcoregtk-4.1

### Build Steps
```bash
# 1. Compile WASM bridge
npm run wasm:build-rust

# 2. Install frontend deps + build
npm ci
npm run build

# 3. Build Tauri desktop app
cd src-tauri
cargo tauri build
```

### Docker
```bash
docker build -t cybermanju-drive .
```

---

## License

MIT — see `LICENSE` file for details.

---

*Cybermanju Drive — quantum-resistant encrypted file management for the post-quantum era.*
