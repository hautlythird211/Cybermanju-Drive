# Cybermanju Drive — Portable Database (`.cybermanju`) System

## Overview

The `.cybermanju` file is a **self-contained, compressed, encrypted portable database** that is automatically created and synced across **all connected platforms**. It serves as the unified shared library, containing the redb database, file metadata, compressed content blobs, preview thumbnails, cross-platform file relations, deletion tracking, and recovery fallbacks.

### Core Principle

> **Every platform holds a complete copy of the `.cybermanju` database.**  
> When you delete a file on one platform, the deletion is recorded and propagated to all others.  
> Compressed/preview versions are always retained for recovery.

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   .cybermanju File Layout                  │
├──────────────────────────────────────────────────────────┤
│ Magic: "CYBERMANJU_PORTABLE_DB_v1" (32 bytes)            │
│ Header JSON length (u32 LE)                               │
│ Header JSON (PortableHeader — version, stats, hash)       │
│ Encrypted/compressed redb database blob                   │
│ Content store (compressed file blobs, .cyb3)              │
│ Preview store (compressed thumbnails, .preview)           │
│ Sidecar blobs directory: {filename}.blobs/                │
└──────────────────────────────────────────────────────────┘
```

### Components

| Component | File | Purpose |
|-----------|------|---------|
| Main file | `.cybermanju` | Portable database file (synced to all backends) |
| Blobs dir | `.cybermanju.blobs/` | Local-only content/preview blob cache |
| Local DB | `cybermanju.db` | Active redb database (extracted from `.cybermanju`) |

---

## Database Tables

The `.cybermanju` wraps a standard redb database with all existing tables plus 4 new ones:

### Existing Tables (16)

| Table | Purpose |
|-------|---------|
| `files` | Central file/folder metadata (FileNode) |
| `accounts` | Storage origin accounts |
| `collections` | Curated file groups |
| `collection_items` | File-collection junction |
| `face_groups` | AI face person clusters |
| `loose_groups` | Ad-hoc file groupings |
| `encryption_keys` | PQC keypairs |
| `locations` | GPS coordinates |
| `users` | User accounts |
| `user_file_perms` | Per-file access control |
| `sync_configs` | Sync backend configurations |
| `parent_index` | Folder child index |
| `trash` | Soft-deleted files |
| `audit_log` | Audit trail |
| `file_versions` | File version history |
| `share_links` | Share link tokens |

### New Portable DB Tables (4)

| Table | Key | Value | Purpose |
|-------|-----|-------|---------|
| `file_relations` | relation_id (UUID) | JSON `FileRelation` | Cross-platform file location tracking |
| `deletion_log` | record_id (UUID) | JSON `DeletionRecord` | Cross-platform deletion propagation |
| `recovery_store` | entry_id (UUID) | JSON `RecoveryEntry` | Compressed/preview references for recovery |
| `portable_meta` | key (string) | value (string) | Portable DB metadata (path, origin, version) |

---

## Key Types

### `FileRelation` — Cross-platform file relation

```rust
struct FileRelation {
    id: String,                // UUID
    local_file_id: String,     // Local FileNode.id
    backend_type: String,      // "local", "github", "gitlab", "googleDrive", "googlePhotos", "telegram"
    remote_file_id: Option<String>,
    remote_path: String,       // Path on the remote backend
    remote_url: Option<String>, // Public URL to the file
    synced_at: String,         // ISO 8601
    last_verified_at: Option<String>,
    status: String,            // "active", "deleted", "pending_delete"
}
```

### `DeletionRecord` — Cross-platform deletion

```rust
struct DeletionRecord {
    id: String,
    local_file_id: String,
    file_name: String,
    deleted_from: String,          // Platform where deletion originated
    deleted_at: String,            // ISO 8601
    deleted_by: Option<String>,
    propagated_to: Vec<String>,    // Platforms deletion already applied to
    pending_platforms: Vec<String>, // Platforms still needing deletion
    has_compressed_version: bool,
    has_preview: bool,
    recovery_file_id: Option<String>,
}
```

### `RecoveryEntry` — Recovery data reference

```rust
struct RecoveryEntry {
    id: String,
    original_file_id: String,
    original_name: String,
    original_mime: Option<String>,
    has_compressed: bool,
    has_preview: bool,
    compressed_hash: Option<String>,
    preview_hash: Option<String>,
    compressed_size: u64,
    preview_size: u64,
    stored_at: String,            // ISO 8601
    blob_offset_compressed: Option<u64>,
    blob_offset_preview: Option<u64>,
}
```

### `PortableHeader` — `.cybermanju` file metadata

```rust
struct PortableHeader {
    version: String,               // "1.0.0"
    created_at: String,            // ISO 8601
    last_modified_at: String,
    app_version: String,
    db_hash: String,               // BLAKE3 of the redb database
    encryption_algorithm: Option<String>,
    compression_algorithm: String, // "lz4+zstd+brotli"
    key_id: Option<String>,
    total_files: u64,
    total_previews: u64,
    total_relations: u64,
    total_deletions: u64,
    db_size_bytes: u64,
    content_store_size: u64,
    preview_store_size: u64,
    platform_origin: String,       // "local", "docker", etc.
    synced_platforms: Vec<String>, // Platforms that have a copy
}
```

---

## Cross-Platform Workflow

### 1. Auto-Creation (Startup)

On every startup, the app calls `PortableDatabase::open_or_create()` which:
- Checks if `.cybermanju` exists at the configured path
- If not, creates it with an empty redb database
- Sets `portable_meta` for `portable_db_path` and `portable_db_origin`
- Extracts the redb database for active use

### 2. File Sync → Relation Recording

When `sync_all` or `sync_single_file` completes:
1. File is uploaded to the target backend
2. `FileRelation` is recorded linking `local_file_id` → `remote_path`
3. If compression was enabled, the compressed version is stored in `recovery_store`
4. After **all files** are synced, the `.cybermanju` file itself is uploaded to the backend

### 3. Deletion → Cross-Platform Propagation

When a file is deleted locally:
1. File is moved to the local trash (soft delete)
2. A `DeletionRecord` is created with all connected platforms as `pending_platforms`
3. When syncing, `propagate_all_pending_deletions()` iterates pending records
4. For each pending platform: calls `backend.delete_file()` with the remote path
5. Updates `DeletionRecord.propagated_to` and removes from `pending_platforms`

### 4. Recovery

When a file's original is deleted but a compressed version exists:
1. The `RecoveryEntry` remains in the database
2. User can browse recoverable files via `list_recoverable_files()`
3. `recover_file()` decompresses the blob back to the original file
4. Preview data is also available via `get_recovery_preview()`

### 5. Platform Independence

Each platform stores its own copy of `.cybermanju`:
- **Local**: On disk at configured path (default: `.cybermanju`)
- **GitHub**: In the repository root via Contents API
- **GitLab**: In the project root via Repository Files API
- **Google Drive**: Uploaded as a file (optionally in a specific folder)
- **Google Photos**: Uploaded as a media item
- **Telegram**: Sent as a document in the target chat

---

## Sync Pipeline Integration

The `SyncPipeline` class in `src-tauri/src/sync/pipeline.rs` now includes:

| Step | Action | Portable DB Impact |
|------|--------|-------------------|
| Scan | Read files from DB | — |
| Compress | Triple-compress file | Content stored in recovery store |
| Preview | Generate Lanczos3 thumbnail | Preview stored in recovery store |
| Upload | Upload to backend | — |
| Link | Update FileNode context | FileRelation recorded |
| **Portable DB Sync** | Upload `.cybermanju` | File is uploaded after all files sync |
| Clean | Delete originals | — |

---

## Tauri Commands

### Portable DB Initialization

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `init_portable_db` | `path`, `platform_origin` | `PortableHeader` | Create/open `.cybermanju` at path |
| `get_portable_db_header` | `path` | `PortableHeader` | Get file header/metadata |
| `sync_portable_db` | — | `Vec<(platform, url)>` | Sync `.cybermanju` to all backends |
| `repack_portable_db` | `local_db_path` | `PortableHeader` | Rebuild `.cybermanju` with latest blobs |
| `get_portable_db_meta` | — | `Vec<(key, value)>` | List all portable DB metadata |

### File Relations

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `record_file_relation` | `local_file_id`, `backend_type`, `remote_path`, ... | `FileRelation` | Link local file to remote copy |
| `get_file_relations` | `local_file_id` | `Vec<FileRelation>` | Get all remote copies of a file |
| `list_all_relations` | — | `Vec<FileRelation>` | List all cross-platform relations |

### Deletion Propagation

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `record_deletion` | `local_file_id`, `file_name`, `deleted_from` | `DeletionRecord` | Record a cross-platform deletion |
| `list_pending_deletions` | — | `Vec<DeletionRecord>` | Get pending deletion propagations |
| `list_all_deletions` | — | `Vec<DeletionRecord>` | Get all deletion records |
| `mark_deletion_propagated` | `record_id`, `platform` | `bool` | Mark deletion as propagated |

### Recovery

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `store_compressed_for_recovery` | `file_id`, `file_path` | `RecoveryEntry` | Store compressed content for recovery |
| `store_preview_for_recovery` | `file_id`, `preview_path`, `width`, `height` | `RecoveryEntry` | Store preview thumbnail for recovery |
| `list_recoverable_files` | — | `Vec<RecoveryEntry>` | List all recoverable files |
| `recover_file` | `file_id`, `output_path` | `u64` (bytes written) | Recover file from compressed blob |
| `get_recovery_preview` | `file_id` | `Option<Vec<u8>>` | Get preview bytes for recovery |

---

## Recovery Flow

```
File deleted locally
    ↓
DeletionRecord created (pending_platforms = [github, gitlab, gdrive, ...])
    ↓
RecoveryEntry has_compressed = true (created during sync)
    ↓
User opens "Recovery" panel
    ↓
list_recoverable_files() → shows entries with has_compressed=true
    ↓
User selects file → recover_file(output_path)
    ↓
Compressed blob decompressed (Brotli → ZSTD → LZ4)
    ↓
Original file written to output_path
```

**Always retained:** If sync had `compress_before_upload = true`, the triple-compressed version is always stored in the portable DB's content store before upload. Even if the original is deleted, the compressed version survives.

---

## Security

- The `.cybermanju` file can be encrypted with **ML-KEM-1024** or **Hybrid (ML-KEM-768 + X25519)** at the pack level
- Encryption keys are stored in the `encryption_keys` table (base64-encoded ML-KEM keypairs)
- BLAKE3 hashing verifies integrity on every unpack
- Compression uses triple-layer (LZ4 → ZSTD → Brotli) for maximum ratio

---

## Implementation Files

| File | Purpose |
|------|---------|
| `crates/portable-db/src/lib.rs` | Core `PortableDatabase` struct with pack/unpack, sync, relations, deletions, recovery |
| `crates/portable-db/Cargo.toml` | Crate dependencies |
| `crates/db/src/database.rs` | 4 new tables + 15 new CRUD methods for portable DB |
| `crates/types/src/schema.rs` | 6 new types (FileRelation, DeletionRecord, RecoveryEntry, PortableHeader, ContentStoreEntry, PreviewStoreEntry) |
| `src-tauri/src/commands/portable_db.rs` | 16 new Tauri commands |
| `src-tauri/src/commands/files.rs` | Deletion recording hook |
| `src-tauri/src/sync/pipeline.rs` | Portable DB integration in sync pipeline |
| `src-tauri/src/lib.rs` | Auto-init on startup + command registration |
| `src-tauri/Cargo.toml` | Dependency on `cybermanju-portable-db` |
| `system.md` | This file |

---

## Summary

The `.cybermanju` portable database system ensures:

1. ✅ **Every connected platform** (local, GitHub, GitLab, Google Drive, Google Photos, Telegram) stores a copy of the `.cybermanju` file
2. ✅ **Triple-compressed + optionally encrypted** using existing crypto/compression crates
3. ✅ **Shared library** with all metadata, content, and previews
4. ✅ **Cross-platform relations** tracking where files exist
5. ✅ **Deletion propagation** — delete once, propagate to all
6. ✅ **Recovery** — compressed versions always retained, even if originals are deleted
7. ✅ **Backward compatible** — all existing functionality preserved
8. ✅ **16 new Tauri commands** for portable DB management
