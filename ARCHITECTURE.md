# Cybermanju Drive — Architecture

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Module Dependency Graph](#2-module-dependency-graph)
3. [Data Flow](#3-data-flow)
4. [Database Schema](#4-database-schema)
5. [Storage Backend Architecture](#5-storage-backend-architecture)
6. [Compression Pipeline](#6-compression-pipeline)
7. [Encryption Architecture](#7-encryption-architecture)
8. [Sync Pipeline](#8-sync-pipeline)
9. [ZimaOS Integration](#9-zimaos-integration)
10. [Web Dashboard REST API](#10-web-dashboard-rest-api)
11. [Deployment Options](#11-deployment-options)

---

## 1. System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CYBERMANJU DRIVE                             │
│                                                                     │
│  ┌──────────────────────┐    ┌──────────────────────────────────┐   │
│  │   FRONTEND (Vue 3)   │    │         BACKEND (Rust)           │   │
│  │                      │    │                                  │   │
│  │  ┌────────────────┐  │    │  ┌────────────────────────────┐  │   │
│  │  │  80+ Vue       │  │    │  │     Tauri IPC Handlers     │  │   │
│  │  │  Components    │  │    │  │     (commands/*.rs)        │  │   │
│  │  │ (25 UI+55 App) │  │    │  └────────────┬───────────────┘  │   │
│  │  └───────┬────────┘  │    │               │                  │   │
│  │          │           │    │               │                  │   │
│  │  ┌───────▼────────┐  │    │  ┌────────────▼───────────────┐  │   │
│  │  │  Pinia Store   │  │    │  │     Core Modules           │  │   │
│  │  │  (stores/app)  │  │    │  │                            │  │   │
│  │  └───────┬────────┘  │    │  │  ┌────────┐ ┌───────────┐ │  │   │
│  │          │           │    │  │  │  db/   │ │  crypto/  │ │  │   │
│  │  ┌───────▼────────┐  │    │  │  │  redb  │ │ PQC+AEAD  │ │  │   │
│  │  │  useTauri.ts   │◄─┼────┼──┼─►├────────┤ ├───────────┤ │  │   │
│  │  │  Triple-mode:  │  │    │  │  │ 21 tbls│ │ChaCha20   │ │  │   │
│  │  │  IPC/WASM/REST │  │    │  │  └────────┘ │ML-KEM/DSA │ │  │   │
│  │  └────────┬────────┘  │    │  │  ┌────────┐ └───────────┘ │  │   │
│  │           │           │    │  │  │ search/│ ┌───────────┐ │  │   │
│  └───────────┼───────────┘    │  │  │Tantivy │ │compress/  │ │  │   │
│              │                │  │  │BM25    │ │LZ4/ZSTD/  │ │  │   │
│     ┌────────┴─────────┐      │  │  └────────┘ │Brotli     │ │  │   │
│     │                  │      │  │  ┌────────┐ └───────────┘ │  │   │
│  ┌──▼──┐  ┌────────┐   │      │  │  │ sync/ │ ┌───────────┐ │  │   │
│  │Tauri│  │HTTP    │   │      │  │  │Backends│ │ faces/    │ │  │   │
│  │ IPC │  │REST    │   │      │  │  │7 types  │ │ONNX/DBSCAN│ │  │   │
│  └──┬──┘  └────┬───┘   │      │  │  └────────┘ └───────────┘ │  │   │
│     │         │        │      │  └────────────────────────────┘  │   │
└─────┼─────────┼────────┴──────┴──────────────────────────────────┘   │
      │         │                                                       │
   Desktop   Browser                                                     │
  (native)  (port 3456)                                                  │
                                                                        │
│  ┌────────────────────────────────────────────────────────────────┐   │
│  │                  EXTERNAL SERVICES                              │   │
│  │  ┌────────┐ ┌──────────────┐ ┌───────────┐ ┌───────────────┐  │   │
│  │  │ Local  │ │   GitHub     │ │Google Drive│ │Google Photos  │  │   │
│  │  │  FS    │ │Contents API  │ │Drive API  │ │  (curl API)   │  │   │
│  │  └────────┘ │GitLab API v4 │ └───────────┘ └───────────────┘  │   │
│  │             │Codeberg/Gitea│ ┌───────────┐ ┌───────────────┐  │   │
│  │             │Git LFS client│ │ Telegram  │ │    Mega.nz    │  │   │
│  │             └──────────────┘ │  Bot API  │ │  megalib      │  │   │
│  │                              └───────────┘ └───────────────┘  │   │
│  └────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Runtime Modes

| Mode | Entry Point | Transport | Use Case |
|------|------------|-----------|----------|
| **Tauri Desktop** | `src-tauri/src/main.rs` | Tauri IPC (`invoke`) | Native desktop app with full filesystem access |
| **Web Dashboard** | Embedded HTTP on `0.0.0.0:3456` | REST API (`fetch`) | Browser access from any device; Docker/ZimaOS |
| **WASM/GitHub Pages** | `dist-wasm/` static files | SPA + WASM + IndexedDB | Public showcase, no backend needed |
| **WASM Bridge** | Browser with WASM support | IndexedDB + crypto | Core engine in-browser, offline-capable |

The frontend composable `useTauri.ts` auto-detects the environment via `window.__TAURI__` and routes commands through Tauri IPC, WASM bridge, or REST API with automatic fallback.

---

## 2. Module Dependency Graph

```
                          lib.rs (orchestrator)
                               │
              ┌────────────────┼────────────────────┐
              │                │                    │
         commands/        web_dashboard/         AppState
        (23 modules)      (HTTP server)        (db + index + compressor)
              │
    ┌─────────┼──────────┬──────────┬──────────┬──────────┐
    │         │          │          │          │          │
  files.rs  search.rs  sync.rs   users.rs  faces.rs  map.rs
    │         │          │          │          │          │
    │    ┌────┘    ┌─────┘          │          │          │
    │    │         │                │          │          │
    ▼    ▼         ▼                ▼          ▼          ▼
  ┌──────────────────────────────────────────────────────────┐
  │                     Core Modules                          │
  │                                                          │
│   db/          crypto/      compression/    search/      │
│   ├── mod.rs   ├── mod.rs   ├── mod.rs      ├── mod.rs   │
│   └── schema.rs└── pqc.rs   └── triple.rs   └── tantivy  │
│                                         _index.rs        │
│   faces/        tree_sitter/    preview/    sync/         │
│   └── mod.rs    └── mod.rs      └── mod.rs  ├── mod.rs   │
│                                                ├── backends│
│                                                ├── models │
│                                                └── pipeline│
│   media/        erasure/      resolutions/  preview-keys/ │
│   ├── thumbnail ├── reed_     ├── shard     ├── key_     │
│   ├── image_ops │   solomon   ├── merkle    │   derivation│
│   ├── video     ├── fountain  └── root      └── view_token│
│   └── info      └── shamir                                  │
│   recovery/                                                  │
│   └── pipeline                                               │
└──────────────────────────────────────────────────────────┘
```

### Module Responsibilities

| Module | Path | Responsibility |
|--------|------|---------------|
| **commands/** | `src-tauri/src/commands/` | 23 Tauri IPC handler files translating frontend calls into core module operations |
| **db/** | `crates/db/` | redb ACID database wrapper with 21 tables, read/write transaction management |
| **crypto/** | `crates/crypto/` | Post-quantum cryptography: ChaCha20Poly1305 AEAD + ML-KEM/ML-DSA key management |
| **compression/** | `crates/compression/` | Triple-layer cascading compression (LZ4 -> ZSTD-15 -> Brotli-11) |
| **search/** | `crates/search/` | Tantivy full-text search with BM25 ranking, faceted filtering, term dictionary autocomplete |
| **faces/** | `crates/faces/` | Face detection embeddings and DBSCAN clustering via connected components |
| **tree_sitter/** | `src-tauri/src/tree_sitter/` | Code intelligence: language detection for 50+ extensions, heuristic symbol extraction |
| **preview/** | `src-tauri/src/preview/` | Lanczos3 thumbnail generation, media metadata extraction |
| **sync/** | `crates/backends/` | Storage backend trait + 10 implementations (Local, GitHub, GitLab, Codeberg, Gitea, Google Drive, Google Photos, Telegram, Mega) + Git LFS client |
| **web_dashboard/** | `src-tauri/src/web_dashboard/` | Embedded HTTP/1.1 server on port 3456 with REST API |
| **media/** | `crates/media/` | Image processing, video thumbnails, metadata extraction |
| **erasure/** | `crates/erasure/` | Erasure coding: Reed-Solomon, fountain codes, Shamir secret sharing |
| **resolutions/** | `crates/resolutions/` | Merkle tree shard resolution for distributed storage |
| **preview-keys/** | `crates/preview-keys/` | Key derivation and view token generation for encrypted previews |
| **recovery/** | `crates/recovery/` | Recovery and reconstruction pipeline (orphaned — not in workspace) |

---

## 3. Data Flow

### Desktop Mode (Tauri IPC)

```
Vue Component
    │
    ▼
Pinia Store (stores/app.ts)
    │  calls action like loadFiles()
    ▼
useTauri.ts → invoke("list_files", { parentPath: "/" })
    │
    ▼ (Tauri IPC bridge — serialized JSON)
    │
commands/files.rs → list_files()
    │
    ├──► db.lock().begin_read()
    │       └──► redb: scan files table, filter by parentId
    │
    ▼
Serde JSON (camelCase) → Frontend
    │
    ▼
Pinia Store updates → Vue reactivity → UI re-renders
```

### Web Dashboard Mode (REST)

```
Vue Component (served from /static)
    │
    ▼
Pinia Store → useTauri.ts → isWebMode() = true
    │
    ▼
REST_ROUTES mapping: list_files → GET /api/files
    │
    ▼
fetch("http://localhost:3456/api/files", { headers: { Authorization: Bearer ... } })
    │
    ▼
web_dashboard/mod.rs → TcpListener → handle_connection()
    │
    ├──► redb: open database, scan files table
    │
    ▼
HTTP/1.1 200 JSON response (snake_case)
    │
    ▼
transformResponseKeys(): snake_case → camelCase
    │
    ▼
Pinia Store → Vue reactivity → UI re-renders
```

### Write Pipeline Example (File Import)

```
1. User selects files via dialog (Tauri) or upload (Web)
2. import_file() command:
   a. Read file bytes from disk
   b. Compute BLAKE3 hash → deduplication check
   c. Detect MIME type via `infer` crate
   d. Extract EXIF GPS via `kamadak-exif` (if image)
   e. Create FileNode with UUID, timestamps, metadata
   f. Serialize to JSON, write to redb files table
   g. Index in Tantivy (file_name, tags, content_text)
   h. Update FileNode with Tantivy doc ID
3. Pinia store refreshes file list
```

---

## 4. Database Schema

All data is stored in a single redb file (`cybermanju.db`) using ACID MVCC transactions. Each table maps `&str` keys to `&str` JSON values.

### Table: `files`

The central table. One row per file or folder tracked by the application.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 primary key |
| `name` | `string` | Display filename |
| `fileType` | `string` | `"file"` or `"folder"` |
| `parentId` | `string?` | Parent folder ID (null = root) |
| `sizeBytes` | `u64` | Original file size in bytes |
| `mimeType` | `string?` | Detected MIME type (infer crate) |
| `hashBlake3` | `string?` | BLAKE3 hex digest for deduplication |
| `encrypted` | `bool` | Whether the file is encrypted |
| `encryptionAlgorithm` | `string?` | Algorithm identifier (e.g., "ML-KEM+ChaCha20Poly1305") |
| `compressionLayers` | `string[]` | Applied compression layers: ["lz4", "zstd", "brotli"] |
| `thumbnailPath` | `string?` | Path to generated preview thumbnail |
| `contextData` | `object?` | Extensible metadata: original_path, sync_url, sync_backend, synced_at |
| `tags` | `string[]` | User-defined searchable tags |
| `collectionIds` | `string[]` | IDs of collections containing this file |
| `faceGroupIds` | `string[]` | IDs of face groups this file belongs to |
| `looseGroupIds` | `string[]` | IDs of user-defined loose groups |
| `gpsLat` | `f64?` | EXIF GPS latitude |
| `gpsLon` | `f64?` | EXIF GPS longitude |
| `createdAt` | `string` | ISO 8601 creation timestamp |
| `modifiedAt` | `string` | ISO 8601 last modification timestamp |

### Table: `accounts`

Storage origin accounts (local directories, cloud services, network mounts).

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `name` | `string` | Display name |
| `accountType` | `string` | `"local"`, `"cloud"`, or `"network"` |
| `path` | `string?` | Local filesystem path |
| `color` | `string` | UI color hex code |
| `isActive` | `bool` | Whether this account is currently active |
| `createdAt` | `string` | ISO 8601 |
| `updatedAt` | `string` | ISO 8601 |

### Table: `collections`

Curated groups of files (highlights, best moments, custom albums).

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `name` | `string` | Collection name |
| `collectionType` | `string` | `"highlights"`, `"best_moments"`, or `"custom"` |
| `color` | `string` | UI color hex code |
| `description` | `string?` | Optional description |
| `itemIds` | `string[]` | File IDs in this collection |
| `createdAt` | `string` | ISO 8601 |
| `updatedAt` | `string` | ISO 8601 |

### Table: `collection_items`

Junction table linking files to collections with optional notes.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `collectionId` | `string` | Parent collection ID |
| `fileId` | `string` | Referenced file ID |
| `note` | `string?` | User annotation |
| `addedAt` | `string` | ISO 8601 |

### Table: `face_groups`

Clusters of files containing the same person's face.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `name` | `string` | Person label (e.g., "Person 1") |
| `fileIds` | `string[]` | File IDs in this face group |
| `centroidEmbedding` | `f32[]?` | 128-dim centroid vector for the cluster |
| `createdAt` | `string` | ISO 8601 |

### Table: `loose_groups`

User-defined ad-hoc file groupings.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `name` | `string` | Group name |
| `color` | `string` | UI color hex code |
| `fileIds` | `string[]` | File IDs in this group |
| `createdAt` | `string` | ISO 8601 |

### Table: `encryption_keys`

PQC and classical encryption keypairs.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `algorithm` | `string` | Algorithm identifier |
| `publicKey` | `string` | Base64-encoded public key bytes |
| `privateKey` | `string` | Base64-encoded private key bytes |
| `label` | `string?` | User-friendly label |
| `createdAt` | `string` | ISO 8601 |

### Table: `users`

Application users with role-based access control.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `username` | `string` | Unique login name |
| `passwordHash` | `string` | Argon2id hash string |
| `displayName` | `string?` | Human-readable name |
| `role` | `string` | `"admin"`, `"user"`, or `"viewer"` |
| `isActive` | `bool` | Whether account is active |
| `createdAt` | `string` | ISO 8601 |
| `updatedAt` | `string` | ISO 8601 |

### Table: `user_file_perms`

Per-file access control entries.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `userId` | `string` | Referenced user ID |
| `fileId` | `string` | Referenced file ID |
| `access` | `string` | `"read"`, `"write"`, or `"admin"` |
| `grantedBy` | `string` | Admin user ID who granted |
| `grantedAt` | `string` | ISO 8601 |

### Table: `sync_configs`

Backend sync configuration entries.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | UUID v4 |
| `backendType` | `string` | `"local"`, `"github"`, `"googleDrive"`, `"googlePhotos"` |
| `enabled` | `bool` | Whether this sync config is active |
| `basePath` | `string?` | Local or remote base path |
| `repoName` | `string?` | GitHub repo (owner/repo) |
| `branch` | `string?` | Git branch |
| `token` | `string?` | OAuth token / personal access token |
| `folderId` | `string?` | Google Drive folder ID |
| `albumId` | `string?` | Google Photos album ID |
| `autoSync` | `bool` | Enable automatic sync |
| `compressBeforeUpload` | `bool` | Apply triple compression before upload |
| `createPreviews` | `bool` | Generate Lanczos3 thumbnails |
| `deleteRawAfterSync` | `bool` | Delete original after successful sync |
| `maxConcurrentUploads` | `u32` | Upload concurrency limit |

### Table: `locations`

Geographic and storage location metadata.

| JSON Field | Type | Description |
|------------|------|-------------|
| (dynamic) | - | Extensible location records |

### Table: `parent_index`

Mapping from file ID to parent folder ID for fast tree queries.

| JSON Field | Type | Description |
|------------|------|-------------|
| `fileId` | `string` | Child file ID |
| `parentId` | `string` | Parent folder ID |

### Table: `trash`

Soft-deleted files pending permanent removal or restore.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | Trash item UUID |
| `fileId` | `string` | Original file ID |
| `originalPath` | `string` | Path before deletion |
| `deletedAt` | `string` | ISO 8601 timestamp |
| `restorable` | `bool` | Whether the item can be restored |

### Table: `audit_log`

Immutable append-only log of all file operations for compliance.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | Log entry UUID |
| `action` | `string` | Operation type (create, delete, encrypt, etc.) |
| `fileId` | `string?` | Affected file ID |
| `userId` | `string?` | Actor user ID |
| `details` | `object?` | Additional metadata |
| `timestamp` | `string` | ISO 8601 timestamp |

### Table: `file_versions`

Snapshot-based file versioning for revert operations.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | Version UUID |
| `fileId` | `string` | Parent file ID |
| `versionNumber` | `u32` | Sequential version number |
| `snapshotData` | `string` | Serialized file state |
| `createdAt` | `string` | ISO 8601 timestamp |

### Table: `share_links`

Token-based share links with optional expiration.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | Share link UUID |
| `fileId` | `string` | Shared file ID |
| `token` | `string` | Random access token |
| `expiresAt` | `string?` | Optional expiration timestamp |
| `createdAt` | `string` | ISO 8601 timestamp |

### Table: `file_relations`

Cross-backend file relationship tracking for sync and portable DB.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | Relation UUID |
| `sourceFileId` | `string` | Original file ID |
| `targetFileId` | `string` | Synced/related file ID |
| `backend` | `string` | Target backend identifier |
| `syncedAt` | `string` | ISO 8601 timestamp |

### Table: `deletion_log`

Deletion propagation log for cross-platform sync.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | Log entry UUID |
| `fileId` | `string` | Deleted file ID |
| `backend` | `string` | Backend where deletion occurred |
| `deletedAt` | `string` | ISO 8601 timestamp |
| `propagated` | `bool` | Whether deletion was synced to other backends |

### Table: `recovery_store`

Recovery blobs for portable DB reconstruction.

| JSON Field | Type | Description |
|------------|------|-------------|
| `id` | `string` | Recovery entry UUID |
| `fileId` | `string` | Source file ID |
| `compressedBlob` | `string` | Triple-compressed file content |
| `blake3Hash` | `string` | Content hash for deduplication |
| `createdAt` | `string` | ISO 8601 timestamp |

### Table: `portable_meta`

Portable database metadata for sync and versioning.

| JSON Field | Type | Description |
|------------|------|-------------|
| `key` | `string` | Metadata key |
| `value` | `string` | Metadata value |
| `updatedAt` | `string` | ISO 8601 timestamp |

---

## 5. Storage Backend Architecture

The sync system uses a trait-based backend architecture. All HTTP backends use `curl` subprocess calls to avoid external HTTP dependencies.

```
┌───────────────────────────────────────────────────┐
│              StorageBackend Trait                 │
│                                                   │
│  + name() -> &str                                 │
│  + backend_type() -> SyncBackendType              │
│  + upload_file(local, remote) -> url              │
│  + download_file(remote, local)                   │
│  + delete_file(remote)                            │
│  + list_files(prefix) -> Vec<RemoteFile>          │
│  + get_file_url(remote) -> url                    │
│  + test_connection() -> bool                      │
└───────────┬───────────┬───────────┬───────────────┘
            │           │           │
    ┌───────▼──┐  ┌────▼────┐  ┌──▼────────┐  ┌─────────────┐
    │  Local   │  │ GitHub  │  │Google Drive│  │Google Photos│
    │  Backend │  │ Backend │  │  Backend   │  │  Backend    │
    │          │  │ + GitLFS│  │            │  │             │
    │ fs::copy │  │Contents │  │ Drive API  │  │ curl upload │
    │ rename   │  │ API v3  │  │ v3 + curl  │  │ API v1      │
    └──────────┘  └─────────┘  │ multipart  │  └─────────────┘
                               │ upload     │
                         ┌─────▼──────┐  ┌─▼──────────┐
                         │  GitLab    │  │  Codeberg   │
                         │  Backend   │  │  Backend    │
                         │            │  │ (Forgejo)   │
                         │ API v4     │  │ API + curl  │
                         │ + curl     │  └─────────────┘
                         └────────────┘
                         ┌────────────┐  ┌─────────────┐
                         │   Gitea    │  │   Telegram  │
                         │  Backend   │  │   Backend   │
                         │ (self-host)│  │   Bot API   │
                         │ API + curl │  │   + curl    │
                         └────────────┘  └─────────────┘
                         ┌────────────┐
                         │   Mega     │
                         │  Backend   │
                         │ megalib    │
                         └────────────┘
```

### Backend Details

| Backend | Auth | Upload Mechanism | Max File Size |
|---------|------|-----------------|---------------|
| **Local** | None | `fs::copy` | Unlimited |
| **GitHub** | Personal Access Token | Contents API (base64), Git LFS for >100MB | 100MB (Contents), 2GB (Releases) |
| **GitLab** | OAuth token | GitLab API v4 | Project-dependent |
| **Codeberg** | Personal Access Token | Forgejo API | 100MB (Contents), 2GB (Releases) |
| **Gitea** | Personal Access Token | Gitea API (self-hosted) | Instance-dependent |
| **Google Drive** | OAuth2 token | Drive API v3 multipart upload via `curl` | 5TB |
| **Google Photos** | OAuth2 token | Upload endpoint via `curl` | 200MB (photos), 10GB (videos) |
| **Telegram** | Bot token | Bot API via `curl` | 50MB |
| **Mega** | Email + password | megalib crate | 20GB (free tier) |

---

## 6. Compression Pipeline

### Triple-Layer Cascading Compression

```
Plaintext
    │
    ▼
┌─────────────────────────┐
│  Layer 1: LZ4 (lz4_flex)│  ~400 MB/s — ultra-fast, size-prefixed
│  Color: #00D4FF          │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  Layer 2: ZSTD (level 15)│  Fast, configurable 1-22, excellent ratio
│  Color: #00FF41           │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  Layer 3: Brotli (level 11)│  Maximum compression ratio, slowest
│  Color: #FFB800            │  Window: 22
└────────────┬────────────┘
             │
             ▼
     Compressed (.cyb3)
     + BLAKE3 hash of original
```

### Decompression (Reverse Order)

```
Compressed (.cyb3)
    │
    ▼
Brotli decode → ZSTD decode → LZ4 decode → Original Plaintext
```

### Single-Layer Modes

| Mode | Algorithm | Speed | Use Case |
|------|-----------|-------|----------|
| `None` | Pass-through | Instant | Already-compressed files |
| `LZ4` | lz4_flex | ~400 MB/s | Real-time previews, streaming |
| `ZSTD` | zstd (configurable) | Fast | Balanced ratio/speed |
| `TripleLayer` | LZ4 -> ZSTD-15 -> Brotli-11 | Slow | Maximum archival compression |

### BLAKE3 Content Hashing

Every file is hashed with BLAKE3 before and after compression. The hash is stored in `FileNode.hashBlake3` and `CompressionStats.blake3Hash` for:
- Deduplication: skip files with identical hashes
- Integrity verification: confirm decompression matches original
- Content-addressed storage foundation

---

## 7. Encryption Architecture

### Hybrid PQC + Classical Design

```
┌─────────────────────────────────────────────────────┐
│                 Encryption Pipeline                  │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  Key Generation (PqcEngine)                   │   │
│  │                                               │   │
│  │  ML-KEM-1024 (FIPS 203)  ◄── NIST Level 5   │   │
│  │  ML-DSA-65   (FIPS 204)  ◄── NIST Level 5   │   │
│  │  SLH-DSA-128f (FIPS 205) ◄── NIST Level 1   │   │
│  │  Hybrid (ML-KEM + X25519) ◄── Transitional   │   │
│  │  AES-256-GCM              ◄── Classical only  │   │
│  └──────────────────┬───────────────────────────┘   │
│                     │                                │
│                     ▼                                │
│  ┌──────────────────────────────────────────────┐   │
│  │  Symmetric Layer: ChaCha20Poly1305 (AEAD)    │   │
│  │                                               │   │
│  │  • 256-bit key from CSPRNG (OsRng)            │   │
│  │  • 96-bit random nonce per operation           │   │
│  │  • Authenticated encryption (Poly1305 MAC)     │   │
│  │  • ~2^96 nonce space — collision-proof         │   │
│  └──────────────────┬───────────────────────────┘   │
│                     │                                │
│                     ▼                                │
│  ┌──────────────────────────────────────────────┐   │
│  │  Integrity Layer: BLAKE3                      │   │
│  │                                               │   │
│  │  • Hash of original plaintext stored in       │   │
│  │    FileEncryptedData.blake3Original            │   │
│  │  • Verified on decryption                     │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### FileEncryptedData Package

Every encrypted file is stored as a `FileEncryptedData` struct containing:

| Field | Type | Description |
|-------|------|-------------|
| `ciphertext` | `Vec<u8>` | Ciphertext + Poly1305 auth tag (appended by AEAD) |
| `nonce` | `[u8; 12]` | Random 96-bit nonce |
| `algorithm` | `String` | Full algorithm string (e.g., "ML-KEM (Kyber-1024) — FIPS 203+ChaCha20Poly1305") |
| `keyId` | `String` | Reference to the EncryptionKey used |
| `blake3Original` | `String` | BLAKE3 hex of the original plaintext |
| `encryptedAt` | `String` | ISO 8601 timestamp |

### ML-DSA Digital Signatures

The `rustpq` crate provides FIPS 204 compliant signing/verification:

- **ML-DSA-65 (Dilithium-5)**: 1952-byte public key, 3309-byte signature, NIST Level 5
- **ML-DSA-44**: 1312-byte public key, 2420-byte signature

`sign_message()` and `verify_signature()` functions in `crypto/pqc.rs` handle key loading, signing, and verification via `rustpq::ml_dsa`.

### Authentication

User passwords are hashed with **Argon2id** (memory-hard, salted, key-stretched) via the `argon2` crate. Session tokens are generated as UUID v4 strings on successful authentication.

---

## 8. Sync Pipeline

### File Processing Flow

```
                    ┌──────────────┐
                    │  File IDs    │
                    │  (selection) │
                    └──────┬───────┘
                           │
                           ▼
              ┌────────────────────────┐
              │   1. SCAN (Scanning)   │
              │   Read FileNode from   │
              │   redb, resolve path   │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ 2. COMPRESS (Compressing)│
              │ Triple-layer pipeline   │
              │ LZ4 -> ZSTD -> Brotli   │
              │ Output: file.cyb3       │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ 3. PREVIEW (Linking)   │
              │ Lanczos3 resize to     │
              │ 512px max, PNG output  │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ 4. UPLOAD (Uploading)  │
              │ Backend-specific:      │
              │ fs::copy / curl POST   │
              │ Remote: cybermanju_sync/│
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ 5. LINK (Linking)      │
              │ Update FileNode        │
              │ context_data with:     │
              │  - sync_url            │
              │  - sync_backend        │
              │  - synced_at           │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ 6. CLEAN (Cleaning)    │
              │ Delete original if     │
              │ deleteRawAfterSync &&  │
              │ compressed exists      │
              └────────────┬───────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  SyncResult  │
                    │  files_synced│
                    │  bytes_uploaded│
                    │  bytes_saved │
                    │  duration_ms │
                    └──────────────┘
```

### Progress Tracking

`SyncPipeline` maintains atomic progress state accessible from any thread:

- `totalFiles` / `processedFiles` (AtomicU32)
- `currentFile` (Mutex<Option<String>>)
- `status` (Mutex<SyncStatus>) — one of: Idle, Scanning, Compressing, Uploading, Linking, Cleaning, Error, Done
- `bytesUploaded` (AtomicU64)
- `errors` (Mutex<Vec<String>>)
- `estimatedRemainingSeconds` — computed from elapsed time / files processed rate

Cancellation is supported via `AtomicBool` flag checked between each file.

---

## 9. ZimaOS Integration

### Docker Compose

```yaml
services:
  cybermanju-drive:
    image: cybermanju-drive:latest
    container_name: cybermanju-drive
    restart: unless-stopped
    ports:
      - "3456:3456"
    volumes:
      - /DATA/AppData/cybermanju-drive/config:/data
    environment:
      - RUST_LOG=info
      - PORT=3456
      - DB_PATH=/data/cybermanju.db
      - STATIC_DIR=/app/static
      - TZ=UTC
```

### Multi-Stage Docker Build

```
┌─────────────────────┐   ┌──────────────────────────┐   ┌──────────────────┐
│  Stage 1: Frontend  │   │  Stage 2: Rust Backend   │   │  Stage 3: Runtime│
│  node:20-alpine     │   │  rust:1.85-alpine        │   │  alpine:3.21     │
│                     │   │                          │   │                  │
│  npm install        │   │  cargo build --release   │   │  ca-certificates │
│  npm run build:wasm │──►│  (web_dashboard only)    │──►│  wget            │
│  → dist-wasm/       │   │  → cybermanju-drive-     │   │  non-root user   │
│                     │   │    server binary          │   │                  │
└─────────────────────┘   └──────────────────────────┘   │  /app/static/    │
                                                        │  /data/ (volume) │
                                                        └──────────────────┘
```

The Docker build uses a standalone Rust server (`docker/server/`) that links the `web_dashboard` module directly — no Tauri or GTK dependencies. The Vue frontend is built with `DOCKER_BUILD=true` to set the correct base path.

### ZimaOS App Store Metadata

The `x-casaos` section in `docker-compose.yml` provides ZimaOS App Store integration:

- **Architectures:** `amd64`, `arm64`
- **Category:** File Sync, Utilities
- **Port:** 3456
- **Volume:** `/DATA/AppData/cybermanju-drive/config` mapped to `/data`
- **Health Check:** `GET /api/health` every 30s
- **Localization:** English (en_us) and Chinese (zh_cn)

---

## 10. Web Dashboard REST API

The embedded HTTP server uses `std::net::TcpListener` with manual HTTP/1.1 parsing (no external HTTP crate). It opens the redb database independently per request.

### Authentication

Login via `POST /api/users/login` with `{ "username": "...", "password": "..." }`. Returns a JWT-like token (UUID v4). Pass as `Authorization: Bearer <token>` header on subsequent requests.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| **Files** | | |
| `GET` | `/api/files` | List all file nodes |
| `GET` | `/api/files/{id}` | Get single file by ID |
| `DELETE` | `/api/files/{id}` | Delete a file |
| **Accounts** | | |
| `GET` | `/api/accounts` | List all storage accounts |
| **Collections** | | |
| `GET` | `/api/collections` | List all collections |
| `GET` | `/api/collection-items` | List all collection items |
| **Face Groups** | | |
| `GET` | `/api/face-groups` | List all face groups |
| **Loose Groups** | | |
| `GET` | `/api/loose-groups` | List all loose groups |
| **Encryption** | | |
| `GET` | `/api/encryption/status` | Get encryption engine status and supported algorithms |
| `GET` | `/api/encryption/keys` | List all encryption keys |
| **Geo** | | |
| `GET` | `/api/geo-files` | List files with GPS coordinates |
| **Search** | | |
| `GET` | `/api/search?q={term}` | Search files by name, path, or content |
| **Locations** | | |
| `GET` | `/api/locations` | List all locations |
| **Users** | | |
| `GET` | `/api/users` | List users (password hashes redacted) |
| `POST` | `/api/users/register` | Register a new user |
| `POST` | `/api/users/login` | Authenticate and receive a token |
| **Permissions** | | |
| `GET` | `/api/permissions/{fileId}` | Get permissions for a file |
| `POST` | `/api/permissions` | Grant a file permission |
| `POST` | `/api/permissions/verify` | Verify file access for a user |
| **Health** | | |
| `GET` | `/api/health` | Health check (returns `{ service, status, timestamp }`) |

### Response Format

All responses are JSON. List endpoints return arrays where each item includes a `_key` field (the redb primary key). The frontend `useTauri.ts` composable automatically strips `_key`/`_raw` fields and transforms `snake_case` keys to `camelCase` to match TypeScript types.

### Static File Serving

In Docker mode, the compiled Vue frontend (`/app/static/`) is served for all non-`/api/` paths. The standalone server at `docker/server/src/main.rs` handles this routing.

---

## 11. Deployment Options

### 1. Tauri Desktop App

Full-featured native desktop application with filesystem access, system tray, and all features.

**Build:**
```bash
npm run tauri:build
```

**Output:** Platform-specific installer (`.dmg`, `.msi`, `.deb`, `.AppImage`)

**Requirements:**
- Node.js 20+
- Rust 1.85+
- Platform-specific: GTK3/WebKit2GTK (Linux), WebView2 (Windows), nothing extra (macOS)

### 2. Docker / ZimaOS Container

Headless deployment for NAS devices, servers, and ZimaOS. Runs the web dashboard on port 3456.

**Build:**
```bash
docker build -t cybermanju-drive:latest .
```

**Run:**
```bash
docker compose up -d
```

**Requirements:** Docker with BuildKit support.

**Architecture Support:** `linux/amd64`, `linux/arm64`

### 3. WASM / GitHub Pages

Static frontend-only deployment for public showcase. Connects to a remote Cybermanju Drive instance via REST API, or runs in read-only demo mode.

**Build:**
```bash
npm run build:wasm
```

**Output:** `dist-wasm/` directory served via GitHub Actions to GitHub Pages.

**Limitations:** No Tauri IPC — only REST API commands with a mapped route. Write operations are unavailable.

### CI/CD Pipeline

```
Push to main
    │
    ├──► rust-check: cargo fmt, clippy, test
    │
    ├──► docker-build: multi-stage Docker image
    │
    ├──► wasm-build: npm ci, vue-tsc, npm run build:wasm
    │         │
    │         └──► deploy-pages: upload to GitHub Pages
    │
    └──► (all jobs run in parallel where possible)
```