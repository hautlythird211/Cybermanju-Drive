# Cybermanju Drive — Comprehensive Task Roadmap

> Full-spectrum analysis of the 17-crate Rust workspace + Vue 3 frontend.
> Priority scale: 🔴 Critical / 🟠 High / 🟡 Medium / 🟢 Exploratory.
> Punk-anarchist principle: every external platform (GitHub, Telegram, GDrive, Mega) is
> a dumb pipe. Cybermanju owns the encryption, the format, the keys, the trust chain.

---

## Table of Contents

1. [Recovery Crate — Workspace Integration](#1-recovery-crate--workspace-integration)
2. [Shard Pipeline — Write Path Implementation](#2-shard-pipeline--write-path-implementation)
3. [Erasure Coding — Real Codec Wiring](#3-erasure-coding--real-codec-wiring)
4. [WASM Bridge — IndexedDB Persistence Layer](#4-wasm-bridge--indexeddb-persistence-layer)
5. [Video Thumbnail — Native Extraction](#5-video-thumbnail--native-extraction)
6. [Neural Upscaler — ONNX Real Inference](#6-neural-upscaler--onnx-real-inference)
7. [Sprite Sheet Engine — Batch Thumbnail Packing](#7-sprite-sheet-engine--batch-thumbnail-packing)
8. [Adaptive Resolution Streaming — Dynamic Decode](#8-adaptive-resolution-streaming--dynamic-decode)
9. [Rayon Parallel Shard Ingestion](#9-rayon-parallel-shard-ingestion)
10. [BLAKE3-Keyed MAC — Shard Integrity at Rest](#10-blake3-keyed-mac--shard-integrity-at-rest)
11. [Nostr Backend — Decentralized Storage via Relays](#11-nostr-backend--decentralized-storage-via-relays)
12. [IPFS / Iroh Backend](#12-ipfs--iroh-backend)
13. [WebTorrent / Bittorrent Backend](#13-webtorrent--bittorrent-backend)
14. [Fediverse Share + ActivityPub Integration](#14-fediverse-share--activitypub-integration)
15. [Zero-Knowledge Proof Layer for View Tokens](#15-zero-knowledge-proof-layer-for-view-tokens)
16. [Memory-Mapped Shard I/O (memmap2)](#16-memory-mapped-shard-io-memmap2)
17. [Tantivy — Persistent Index Across Sessions](#17-tantivy--persistent-index-across-sessions)
18. [redb — Snapshot Export to Portable Shard](#18-redb--snapshot-export-to-portable-shard)
19. [Incremental Merkle Updates (Append-Only)](#19-incremental-merkle-updates-append-only)
20. [CLI — TUI Shard Browser](#20-cli--tui-shard-browser)
21. [Compression — Content-Aware Algorithm Selector](#21-compression--content-aware-algorithm-selector)
22. [Compression — Zstd Dictionary Training Pipeline](#22-compression--zstd-dictionary-training-pipeline)
23. [Face Clustering — ONNX Model Auto-Download](#23-face-clustering--onnx-model-auto-download)
24. [SimHash Index — Persistent Disk Cache](#24-simhash-index--persistent-disk-cache)
25. [Key Hierarchy — Hardware Security Key (FIDO2)](#25-key-hierarchy--hardware-security-key-fido2)
26. [Token Revocation — Merkle Accumulator CRL](#26-token-revocation--merkle-accumulator-crl)
27. [Backends — Headscale / Tailscale LAN Sync](#27-backends--headscale--tailscale-lan-sync)
28. [Backends — Rclone Abstraction Layer](#28-backends--rclone-abstraction-layer)
29. [Web Dashboard — WebSocket Live Push](#29-web-dashboard--websocket-live-push)
30. [Vue Frontend — Offline PWA Mode](#30-vue-frontend--offline-pwa-mode)
31. [Vue Frontend — Drag-Drop Shard Visualizer](#31-vue-frontend--drag-drop-shard-visualizer)
32. [System Monitor — Real sysinfo Integration](#32-system-monitor--real-sysinfo-integration)
33. [Audit Log — Tamper-Evident Chaining](#33-audit-log--tamper-evident-chaining)
34. [Plugin SDK — WASM Sandbox for Extensions](#34-plugin-sdk--wasm-sandbox-for-extensions)
35. [Paranoid Mode — Plausible Deniability Volumes](#35-paranoid-mode--plausible-deniability-volumes)
36. [Benchmark Suite — Criterion.rs Harness](#36-benchmark-suite--criterionrs-harness)

---

## 1. Recovery Crate — Workspace Integration

**Priority:** 🔴 Critical  
**Crates affected:** `Cargo.toml` (workspace root), `crates/recovery/`

### Problem

`crates/recovery/` has a full `Cargo.toml`, four source files (`pipeline.rs`, `neural_upscaler.rs`, `image_utils.rs`, `errors.rs`), and real logic — but it is **not listed in the workspace `[workspace.members]`**. This means:

- `cargo check --workspace` silently skips it
- `src-tauri` cannot import it (no workspace path resolution)
- The `RecoveryPipeline` that the resolution tree depends on is permanently unreachable

### Fix

Add `"crates/recovery"` to `Cargo.toml` workspace members list, add it as a dependency in `src-tauri/Cargo.toml`, and wire `RecoveryPipeline` into the Tauri `commands/` layer.

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    # ... existing 17 members ...
    "crates/recovery",   # ← ADD THIS
]
```

```toml
# src-tauri/Cargo.toml
cybermanju-recovery = { path = "../crates/recovery" }
```

Wire into `src-tauri/src/lib.rs`:

```rust
pub mod recovery; // new command module
```

Create `src-tauri/src/commands/recovery.rs` exposing:
- `recover_file(file_id, chunks, request)` — calls `RecoveryPipeline::recover_from_chunks`
- `upscale_file(file_id, model, scale)` — calls `NeuralUpscaler::upscale`
- `upscale_region(file_id, x, y, w, h, model)` — tile-based upscale for large files

---

## 2. Shard Pipeline — Write Path Implementation

**Priority:** 🔴 Critical  
**Crates affected:** `crates/resolutions/`, `crates/erasure/`, `crates/preview-keys/`, `src-tauri/`

### Problem

The entire resolution tree architecture (`ARCHITECTURE_RESOLUTION_TREE.md`) describes a sophisticated shard write pipeline, but no crate actually **creates** `.cybermanju` v2 shard files. The types are complete (`ShardHeader`, `ShardIndex`, `ContentMap`, `ErasureMeta`), the key hierarchy works, the Merkle tree builds correctly — but there is no `ShardWriter` that assembles the binary format.

### Implementation

Create `crates/resolutions/src/writer.rs`:

```rust
pub struct ShardWriter {
    shard_id: String,
    header: ShardHeader,
    index: ShardIndex,
    content_buf: Vec<u8>,
    content_offset: u64,
    key_hierarchy: KeyHierarchy,
    compressor: TripleCompressor,
}

impl ShardWriter {
    pub fn new(shard_id: &str, root_hash: &str, key_hierarchy: KeyHierarchy) -> Self { ... }

    /// Ingest one file at all resolution levels
    pub fn add_file(
        &mut self,
        file: &FileNode,
        r0: &[u8],    // 200px thumbnail, WebP
        r1: &[u8],    // 640px preview, WebP
        r2: &[u8],    // 1920px medium, WebP or JPEG
        r3: &[u8],    // original bytes
    ) -> Result<ResolutionEntry> { ... }

    /// Finalize shard: compress + encrypt all layers, compute Merkle, write binary
    pub fn finalize(self, output_path: &Path) -> Result<ShardManifest> { ... }
}
```

Binary layout to implement (from `ARCHITECTURE_RESOLUTION_TREE.md`):

```
[PLAINTEXT HEADER]  88 bytes — magic, shard_id, size, root_hash_backlink
[INDEX LAYER]       AES-256-GCM(index_key, ShardIndex JSON + compression)
[CONTENT MAP]       AES-256-GCM(index_key, ContentMap JSON)
[ERASURE META]      AES-256-GCM(index_key, ErasureMeta JSON)
[CONTENT BLOBS]     Ordered: r0 blobs | r1 blobs | r2 blobs | r3 chunks | parity
[FOOTER]            BLAKE3(full_content) + ML-DSA-65 signature
```

### Key design constraints to preserve

- All blobs inside content section encrypted with tier-appropriate key: r0/r1 with `preview_key`, r2/r3 with `content_key`
- Per-chunk nonce derivation from `BLAKE3(file_id || chunk_index)` — already implemented in `key_derivation.rs`, just needs to be called
- Content offsets in `ShardIndex.files[file_id].resolutions["r0"].content_offset` must be byte-accurate

---

## 3. Erasure Coding — Real Codec Wiring

**Priority:** 🟠 High  
**Crates affected:** `crates/erasure/`, `crates/resolutions/`

### Problem

`crates/erasure/` has complete GF(2^8) Reed-Solomon math, Shamir secret sharing, and a functional XOR-based fountain encoder. But `crates/resolutions/src/shard.rs` declares `erasure_codec: "clay-codes"` and types like `ErasureClayShard` that have no implementing crate. Clay codes (coupled-layer codes with repair bandwidth ~= 1 shard) are a specialized codec that doesn't yet exist in Rust's ecosystem at production quality.

### Pragmatic resolution

Replace the Clay codes aspiration with a working Reed-Solomon configuration that matches the security and redundancy goals:

**Phase 1 — Reed-Solomon (k=4, m=2):** Any 4 of 6 shards can reconstruct. Use the existing `ReedSolomonEncoder` in `crates/erasure/src/reed_solomon.rs`. Wire it into shard write pipeline.

**Phase 2 — Fountain overlay:** Generate 2x source count of fountain packets per shard using `FountainEncoder`. Store in `ErasureMap.fountain_packets`. This gives rateless recovery from partial downloads.

**Phase 3 — Clay codes research:** Add `leopard` crate (`lrc-leopard`) once upstream stabilizes; swap codec via `ErasureCodecType` enum without breaking existing `.cybermanju` v2 shards.

New function in `crates/erasure/src/lib.rs`:

```rust
pub struct ShardErasureEngine {
    pub codec: ErasureCodecType,
    pub k: u32,  // data shards
    pub m: u32,  // parity shards
}

impl ShardErasureEngine {
    pub fn encode(&self, data: &[u8]) -> Result<Vec<Vec<u8>>> { ... }
    pub fn decode(&self, shards: &[Option<Vec<u8>>]) -> Result<Vec<u8>> { ... }
    pub fn parity_shards(&self, data_shards: &[Vec<u8>]) -> Result<Vec<Vec<u8>>> { ... }
}
```

---

## 4. WASM Bridge — IndexedDB Persistence Layer

**Priority:** 🟠 High  
**Crates affected:** `crates/drive-wasm/`, `src/wasm/`

### Problem

`VirtualDrive` and `SyncEngine` in the WASM bridge hold all state **in-memory**. If the tab refreshes, all files are lost. `src/wasm/storage.ts` creates an IndexedDB schema for `files` and `sync_entries` tables, but `drive.rs` never writes to it — the WASM Rust layer doesn't call back into the JS storage layer.

### Solution

In `crates/drive-wasm/src/drive.rs`, expose persistence callbacks via `wasm_bindgen` closures:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = cybermanjuStorage)]
    fn persist_file_node(json: &str);

    #[wasm_bindgen(js_namespace = cybermanjuStorage)]
    async fn load_all_files() -> JsValue;
}
```

In `src/wasm/drive.ts`, create a `cybermanjuStorage` namespace that delegates to `storage.ts` IndexedDB operations. `VirtualDrive::create_file()` should call `persist_file_node(serde_json::to_string(&node))` after inserting.

On `VirtualDrive::new()`, issue an async `load_all_files()` call to hydrate state from IndexedDB on startup.

This gives the WASM mode the same persistence guarantee as the Tauri + redb mode — files survive tab close/refresh.

---

## 5. Video Thumbnail — Native Extraction

**Priority:** 🟠 High  
**Crates affected:** `crates/media/src/thumbnail.rs`

### Problem

`generate_video_thumbnail_placeholder()` exists in `crates/media/src/thumbnail.rs` and returns a grey placeholder image. Real video thumbnail extraction requires seeking to ~10% of stream duration and extracting a frame.

### Solution

Add `ffmpeg-next` or `video-rs` as an **optional** feature dependency in `crates/media/Cargo.toml`:

```toml
[features]
default = []
ffmpeg = ["ffmpeg-next"]
vlc = []  # existing flag

[dependencies]
ffmpeg-next = { version = "7", optional = true }
```

Implement in `crates/media/src/video.rs`:

```rust
#[cfg(feature = "ffmpeg")]
pub fn extract_frame_at_percent(path: &Path, percent: f64) -> Result<ThumbnailResult> {
    use ffmpeg_next as ffmpeg;
    ffmpeg::init()?;
    let mut ictx = ffmpeg::format::input(path)?;
    let duration = ictx.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64;
    let seek_ts = (duration * percent * 1_000_000.0) as i64;
    ictx.seek(seek_ts, ..=seek_ts)?;
    // decode first video frame, resize to max_size via fast_image_resize
    ...
}

// Fallback: always available, uses embedded placeholder
#[cfg(not(feature = "ffmpeg"))]
pub fn extract_frame_at_percent(path: &Path, percent: f64) -> Result<ThumbnailResult> {
    generate_video_thumbnail_placeholder(320, 180)
}
```

The Tauri `preview/` module should call `extract_frame_at_percent(path, 0.1)` for `.mp4 .mkv .avi .mov .webm` files.

---

## 6. Neural Upscaler — ONNX Real Inference

**Priority:** 🟡 Medium  
**Crates affected:** `crates/recovery/src/neural_upscaler.rs`

### Problem

`NeuralUpscaler::run_onnx_model()` is stubbed with a comment: "In production, this would load ONNX model and run inference." When a model path is supplied, it falls through to Lanczos anyway.

### Solution

Wire the existing `ort` (ONNX Runtime) crate already used in `crates/faces/` into `crates/recovery/`:

```toml
# crates/recovery/Cargo.toml
[features]
default = []
onnx-upscale = ["ort", "ndarray", "dep:image"]

[dependencies]
ort = { version = "2.0.0-rc.12", features = ["download-binaries", "ndarray"], optional = true }
ndarray = { version = "0.17", optional = true }
image = { version = "0.25", optional = true }
```

Implement `run_onnx_model` for Real-ESRGAN x2/x4:

```rust
#[cfg(feature = "onnx-upscale")]
fn run_onnx_model(&self, input: &[u8], w: u32, h: u32, scale: u32, path: &str) -> Result<Vec<u8>, RecoveryError> {
    use ort::{Session, Value};
    use ndarray::{Array4, s};

    // Normalize to [0,1] float32 NCHW
    let session = Session::builder()?.with_model_from_file(path)?;
    let tensor = normalize_to_nchw(input, w, h)?;
    let input_val = Value::from_array(session.allocator(), &tensor)?;
    let outputs = session.run(vec![input_val])?;
    let output_tensor = outputs[0].try_extract::<f32>()?;
    denormalize_from_nchw(&output_tensor.view(), w * scale, h * scale)
}
```

Include tile-based inference (`upscale_region`) to avoid OOM on large images: divide into 256x256 tiles with 16px overlap, upscale each, stitch.

Automatic model download:

```rust
pub fn ensure_model(model: &UpscaleModel) -> Result<PathBuf> {
    let cache = dirs::cache_dir().unwrap().join("cybermanju/models");
    let path = cache.join(model.filename());
    if !path.exists() {
        std::fs::create_dir_all(&cache)?;
        download_model(model.url(), &path)?; // reqwest blocking
    }
    Ok(path)
}
```

---

## 7. Sprite Sheet Engine — Batch Thumbnail Packing

**Priority:** 🟡 Medium  
**Crates affected:** `crates/media/` (new file), `crates/resolutions/src/shard.rs`

### Problem

`SpriteSheetEntry` is defined in `shard.rs` with fields `grid`, `thumb_count`, `content_offset`, `content_length` — but nothing creates sprite sheets. For a library of 10,000 photos, loading thumbnails individually (10K separate decrypt+decompress calls) is too slow.

### Solution

New `crates/media/src/sprite_sheet.rs`:

```rust
pub struct SpriteSheetBuilder {
    tile_size: u32,      // e.g. 128 (per thumbnail)
    grid_cols: u32,      // e.g. 16
    max_tiles: u32,      // e.g. 256 (16x16 grid)
    tiles: Vec<Vec<u8>>, // raw RGBA tiles
}

impl SpriteSheetBuilder {
    pub fn new(tile_size: u32, grid_cols: u32) -> Self { ... }

    /// Add a pre-decoded r0 thumbnail tile
    pub fn add_tile(&mut self, rgba: &[u8], w: u32, h: u32) -> Result<usize> { ... }

    /// Pack all tiles into a single WebP sprite sheet, return encoded bytes
    pub fn build(&self) -> Result<SpriteSheetResult> {
        // Use fast_image_resize to place tiles into a grid canvas
        // Encode as WebP lossless at quality=80
        // Return: encoded bytes + grid string "16x16"
    }
}

pub struct SpriteSheetResult {
    pub data: Vec<u8>,
    pub grid: String,
    pub tile_size: u32,
    pub thumb_count: u32,
    pub width: u32,
    pub height: u32,
}
```

Frontend `FileGrid.vue` batches thumbnail loads: load one sprite sheet blob, decode once, CSS `background-position` to show per-file tile. This replaces one-by-one IPC calls with a single shard content read.

---

## 8. Adaptive Resolution Streaming — Dynamic Decode

**Priority:** 🟡 Medium  
**Crates affected:** `crates/resolutions/`, new `src-tauri/src/commands/resolution.rs`

### Problem

`ResolutionLevel` in `crates/types/src/resolution.rs` defines a clean R0→R3 hierarchy (200px → 640px → 1920px → original), but the Tauri commands don't use it. `FilePreview.vue` either loads the full original or a static 512px thumbnail — there's no progressive loading.

### Solution

New Tauri command `fetch_resolution(file_id, target_resolution, quality)` in `src-tauri/src/commands/resolution.rs`:

```rust
#[tauri::command]
pub async fn fetch_resolution(
    state: State<'_, AppState>,
    file_id: String,
    target: String,   // "r0" | "r1" | "r2" | "r3"
    quality: Option<u8>,
) -> Result<ResolutionResponse, String> {
    // 1. Look up FileNode in redb
    // 2. Check which shard holds this file_id + target resolution
    // 3. If shard not local: fetch from backend (async)
    // 4. Decrypt with appropriate key tier (preview_key for r0/r1, content_key for r2/r3)
    // 5. Decompress
    // 6. Return bytes + dimensions
}
```

In `FilePreview.vue`, implement progressive loading sequence:

```
mount → request r0 (immediate, 200px, preview_key only) →
  display blurred r0 →
  request r1 (640px) →
  crossfade r0→r1 →
  if user holds view > 2s: request r2 →
  if user clicks "download": request r3
```

Uses `IntersectionObserver` to cancel pending requests when file scrolls out of view. This gives instant feedback even for files on remote backends — r0 can be cached locally as a sprite sheet while r3 lives on Mega.

---

## 9. Rayon Parallel Shard Ingestion

**Priority:** 🟠 High  
**Crates affected:** `crates/backends/`, `src-tauri/src/sync/`

### Problem

`transfer_files()` in `crates/backends/src/transfer.rs` uses a sequential loop. Syncing 1000 files to GitHub takes serial HTTP calls — no parallelism.

### Solution

Add `rayon` (already in workspace) parallelism to the ingestion pipeline:

```rust
use rayon::prelude::*;

pub fn transfer_files_parallel(
    files: &[SyncFile],
    backend: &dyn StorageBackend,
    max_parallel: usize,
    progress: Arc<dyn Fn(SyncProgress) + Send + Sync>,
) -> Vec<SyncResult> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(max_parallel)
        .build()
        .unwrap();

    pool.install(|| {
        files.par_iter()
            .map(|file| {
                let result = backend.upload_file(&file.path, &file.remote_path, &file.content);
                let status = match &result {
                    Ok(_) => SyncStatus::Done,
                    Err(e) => SyncStatus::Error,
                };
                progress(SyncProgress { file_id: file.id.clone(), status, ... });
                result
            })
            .collect()
    })
}
```

For backends with rate limits (GitHub API: 5000 req/h, Telegram: 20 files/min), wrap with a `RateLimiter` using `governor` crate:

```toml
[dependencies]
governor = "0.6"
```

Implement per-backend `rate_limit()` method on `StorageBackend` trait:

```rust
fn rate_limit_policy(&self) -> Option<RateLimit> {
    None  // default: no limit
}
```

GitHub backend overrides with `Some(RateLimit::per_minute(60))`.

---

## 10. BLAKE3-Keyed MAC — Shard Integrity at Rest

**Priority:** 🟠 High  
**Crates affected:** `crates/resolutions/src/shard.rs`, `crates/preview-keys/`

### Problem

Shard footer contains a `shard_blake3` hash but it's computed over the shard content without a key. An attacker with write access to a storage backend (e.g., a compromised GitHub account) could substitute an entire shard with a crafted file that passes the integrity check.

### Solution

Replace bare BLAKE3 with BLAKE3 keyed MAC in the shard footer:

```rust
/// Compute a keyed shard MAC: BLAKE3-keyed(shard_mac_key, content)
/// shard_mac_key derived: HKDF(master_key, "cybermanju-shard-mac-v1", shard_id)
pub fn compute_shard_mac(content: &[u8], shard_id: &str, master_key: &[u8; 32]) -> [u8; 32] {
    let mac_key = derive_shard_mac_key(master_key, shard_id);
    *blake3::keyed_hash(&mac_key, content).as_bytes()
}

pub fn verify_shard_mac(content: &[u8], shard_id: &str, master_key: &[u8; 32], expected: &[u8; 32]) -> bool {
    let computed = compute_shard_mac(content, shard_id, master_key);
    constant_time_eq(&computed, expected) // subtle crate for timing-safe compare
}
```

Add to `ShardHeader`:

```rust
/// BLAKE3-keyed MAC over entire shard content (replaces bare shard_blake3)
pub shard_mac: String,  // hex-encoded [u8; 32]
```

Add `subtle` crate to `crates/preview-keys/Cargo.toml` for constant-time comparison.

---

## 11. Nostr Backend — Decentralized Storage via Relays

**Priority:** 🟢 Exploratory  
**Crates affected:** `crates/backends/src/nostr.rs` (new)

### Concept

Nostr (Notes and Other Stuff Transmitted by Relays) is an open, censorship-resistant protocol. Large binary files can be stored as NIP-95 or NIP-96 (file storage extension) events. Each shard becomes a signed Nostr event published to multiple relays simultaneously.

This makes Cybermanju the first encrypted file manager where your data lives on a **community-owned relay network** that nobody controls — true punk infrastructure.

```rust
pub struct NostrBackend {
    private_key: [u8; 32],  // secp256k1 private key (or npub from nostr-sdk)
    relays: Vec<String>,     // e.g. ["wss://relay.damus.io", "wss://relay.snort.social"]
    nip96_host: Option<String>,  // for hosted file storage
}

impl StorageBackend for NostrBackend {
    fn upload_file(&self, path: &str, remote_path: &str, content: &[u8]) -> Result<RemoteFile> {
        // NIP-96: POST to nip96_host/upload with Bearer auth derived from privkey
        // or NIP-95: publish kind:1063 event with base64 content (small files only)
        // Returns magnet-like event ID as URL
    }

    fn download_file(&self, remote_path: &str) -> Result<Vec<u8>> {
        // Fetch by event ID from any relay (content-addressed)
    }
}
```

Add `nostr-sdk` crate (pure Rust, async Nostr client):

```toml
nostr-sdk = { version = "0.35", features = ["nip96"] }
```

**Key insight:** Because Cybermanju shards are already encrypted and authenticated, Nostr's encryption layer (NIP-04/NIP-44) is redundant but harmless — the shard MAC ensures authenticity, NIP-96 just handles transport.

---

## 12. IPFS / Iroh Backend

**Priority:** 🟢 Exploratory  
**Crates affected:** `crates/backends/src/iroh.rs` (new)

### Concept

Iroh (by n0.computer) is a Rust-native content-addressed IPFS implementation with a clean SDK. Each `.cybermanju` shard has a BLAKE3 content hash — this maps directly to IPFS CIDs (using BLAKE3 multihash in a CIDv1).

```rust
pub struct IrohBackend {
    node: iroh::node::Node,
    gateway_url: Option<String>,
}

impl StorageBackend for IrohBackend {
    fn upload_file(&self, _path: &str, _remote_path: &str, content: &[u8]) -> Result<RemoteFile> {
        let hash = self.node.blobs().add_bytes(content.to_vec()).await?;
        Ok(RemoteFile {
            name: hash.to_string(),
            path: hash.to_string(),
            url: format!("ipfs://{}", hash),
            size_bytes: content.len() as u64,
            ...
        })
    }
}
```

Store CIDs in `RootShardInfo.remote_url`. Any IPFS gateway or local node can serve shards. Combined with Nostr for CID publishing, this creates a fully decentralized, community-replicated archive.

```toml
iroh = { version = "0.26", features = ["default"] }
```

---

## 13. WebTorrent / Bittorrent Backend

**Priority:** 🟢 Exploratory  
**Crates affected:** `crates/backends/src/torrent.rs` (new)

### Concept

Pack a set of `.cybermanju` shards into a `.torrent` file. Anyone who seeds becomes a distribution node. Combined with the existing encryption, this is **BitTorrent for encrypted personal archives** — the seeder sees opaque blobs, not your data.

Use `librqbit` (pure Rust BitTorrent client with magnet link support):

```toml
librqbit = { version = "7", features = ["http-api"] }
```

Implementation sketch:

```rust
pub struct TorrentBackend {
    save_dir: PathBuf,       // where .torrent files live
    seed_port: u16,          // local DHT/peer port
    tracker_url: Option<String>,
}

impl StorageBackend for TorrentBackend {
    fn upload_file(&self, path: &str, remote_path: &str, content: &[u8]) -> Result<RemoteFile> {
        // Write content to temp file
        // Create .torrent via librqbit
        // Start seeding
        // Return magnet link as URL
    }
}
```

For CybermanjuOS community libraries (shared public encrypted photo archives, zines, etc.), this means **zero hosting cost** and resilience against takedowns.

---

## 14. Fediverse Share + ActivityPub Integration

**Priority:** 🟢 Exploratory  
**Crates affected:** `crates/web/`, `crates/backends/src/activitypub.rs` (new)

### Concept

Cybermanju collections (curated groups of encrypted files) can be shared as ActivityPub `Collection` objects. Other Fediverse users follow the collection — when new files are added, they receive `Add` activities. The files themselves are encrypted shards; the AP metadata only exposes what the sharer chooses to reveal.

```rust
pub struct ActivityPubShare {
    pub collection_id: String,
    pub actor_id: String,   // e.g. "@username@instance.social"
    pub endpoint: String,   // ActivityPub inbox URL
    pub access_token: String,
}
```

`POST /outbox` with an `Add` activity whose `object` contains the shard URL (IPFS, Nostr event ID, or GitHub release URL) but NOT the decryption key. Recipients can download the encrypted shard; the sharer gives them the key out-of-band.

Optionally implement `announce_collection()` for Mastodon/Misskey/Pixelfed-compatible `Note` posts.

---

## 15. Zero-Knowledge Proof Layer for View Tokens

**Priority:** 🟡 Medium  
**Crates affected:** `crates/preview-keys/src/view_token.rs`

### Problem

`ViewToken.signature` is currently BLAKE3-keyed hash (not a real signature — just a double BLAKE3). The comment says "simplified — truncated to 64 bytes" and acknowledges this isn't HMAC-SHA256 as described. For a system that claims quantum-resistant guarantees, the view token layer should use ML-DSA.

### Fix

Replace double-BLAKE3 with real ML-DSA-44 signature in `generate_view_token`:

```rust
use ml_dsa::{MlDsa44, Keypair, Signer, Verifier};

pub fn generate_view_token(...) -> Result<ViewToken, PreviewKeyError> {
    let signing_key = derive_signing_key_from_master(master_key, file_id, token_id)?;
    let keypair = Keypair::<MlDsa44>::from_seed(&signing_key)?;

    let claims_json = serde_json::to_vec(&claims)?;
    let signature = keypair.signing_key().sign(&claims_json).to_bytes().to_vec();
    let verifying_key = keypair.verifying_key().to_bytes().to_vec();

    Ok(ViewToken {
        token_id,
        claims,
        encrypted_preview_data: encrypted_preview,
        signature,
        verifying_key_hint: verifying_key[..16].to_vec(), // fingerprint only
    })
}
```

ML-DSA-44 signature is 2420 bytes — store as Vec<u8> (remove the fixed `[u8; 64]` array). Verification uses the verifying key rederived from master_key at validation time (no need to store full pubkey in token).

---

## 16. Memory-Mapped Shard I/O (memmap2)

**Priority:** 🟡 Medium  
**Crates affected:** `crates/resolutions/` (writer + reader)

### Problem

Reading a single r0 thumbnail from a multi-gigabyte shard file requires seeking to `content_offset` — currently done via `File::seek()` which copies bytes through the kernel buffer cache. For a shard with 10,000 files and frequent thumbnail fetches, this is a bottleneck.

### Solution

Add `memmap2` for zero-copy shard reads:

```toml
# crates/resolutions/Cargo.toml
memmap2 = "0.9"
```

```rust
pub struct ShardReader {
    mmap: memmap2::Mmap,
    index: ShardIndex,
    key_hierarchy: KeyHierarchy,
}

impl ShardReader {
    pub fn open(path: &Path, key_hierarchy: KeyHierarchy) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        // Parse and decrypt index from mmap[header.encrypted_index_offset..]
        let index = decrypt_and_parse_index(&mmap, &key_hierarchy)?;
        Ok(Self { mmap, index, key_hierarchy })
    }

    /// Zero-copy read: returns a slice into the mmap for the requested resolution
    pub fn read_resolution_raw(&self, file_id: &str, res: &str) -> Result<&[u8]> {
        let entry = self.index.files.get(file_id).ok_or(...)?;
        let level = entry.resolutions.get(res).ok_or(...)?;
        let start = level.content_offset as usize;
        let end = start + level.content_length as usize;
        Ok(&self.mmap[start..end])
    }

    /// Decrypt and decompress one resolution level
    pub fn read_resolution(&self, file_id: &str, res: &str) -> Result<Vec<u8>> {
        let raw = self.read_resolution_raw(file_id, res)?;
        let decrypted = self.decrypt_blob(raw, file_id, res)?;
        decompress(&decrypted)
    }
}
```

`memmap2` is `Send + Sync` and lets multiple async tasks share the same mapped shard concurrently. This enables the sprite sheet renderer to decode 256 tiles from one shard without 256 separate seeks.

---

## 17. Tantivy — Persistent Index Across Sessions

**Priority:** 🟠 High  
**Crates affected:** `crates/search/src/lib.rs`, `src-tauri/src/lib.rs`

### Problem

`AppState.tantivy_index` is created fresh on startup and only written when files are imported. If the app crashes, the index is gone. On restart, users see no search results until `rebuild_search_index` is manually called.

### Fix

Store the Tantivy index on disk (it already supports this via `MmapDirectory`):

```rust
pub fn open_or_create_index(index_path: &Path) -> Result<Index> {
    let dir = MmapDirectory::open(index_path)?;
    if Index::exists(&dir)? {
        Index::open(dir)
    } else {
        let schema = build_schema();
        Index::create(dir, schema, IndexSettings::default())
    }
}
```

In `app_data_dir()` flow, pass `data_dir.join("tantivy_index")` to `open_or_create_index`. The index survives restarts. Add `IndexWriter::commit()` after every batch import (already called — just needs disk path).

Add periodic background index compaction:

```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        index.write().unwrap().writer.merge(&[]).await?;
    }
});
```

---

## 18. redb — Snapshot Export to Portable Shard

**Priority:** 🟡 Medium  
**Crates affected:** `crates/portable-db/src/lib.rs`, `crates/resolutions/`

### Problem

`PortableDatabase::pack()` compresses and writes the redb database bytes but treats the `.cybermanju` portable format as independent from the `.cybermanju` shard format. The portable v1 (`CYBERMANJU_PORTABLE_v1`) and shard v2 (`CYBSHARD_V2`) are now two separate file formats with different magic bytes, different encryption schemes, and different layouts.

### Proposal

Unify: make the portable database a **special preview shard** in the v2 shard format. The portable `.cybermanju` file becomes `shard_type: Preview` carrying the redb snapshot as a single `r0` blob encrypted with `preview_key`. This means:

- One format, one parser
- The portable DB syncs to any backend using the same `StorageBackend::upload_file()` already used for other shards
- The `root.cybermanju` index tracks the portable DB shard alongside file shards
- Any device with `index_key + preview_key` can reconstruct the file list metadata without downloading content

Migration path: detect `CYBERMANJU_PORTABLE_v1` magic bytes → convert to v2 shard format on first read. Write only v2 going forward.

---

## 19. Incremental Merkle Updates (Append-Only)

**Priority:** 🟡 Medium  
**Crates affected:** `crates/resolutions/src/merkle.rs`

### Problem

`ResolutionMerkleTree::build()` recomputes the entire tree from scratch every time any file is added/modified. For a library with 100,000 files (100K leaves), full rebuild is expensive.

### Solution

Implement a sparse Merkle tree that supports O(log n) updates:

```rust
pub struct IncrementalMerkleTree {
    /// Leaf store: file_id → {resolution → hash}
    leaves: HashMap<String, HashMap<String, String>>,
    /// Cached intermediate nodes: node_path → hash
    cache: HashMap<Vec<u8>, String>,
    /// Height of the tree (ceil(log2(capacity)))
    height: usize,
}

impl IncrementalMerkleTree {
    pub fn new(capacity: usize) -> Self { ... }

    /// O(log n) update: update one file's r0/r1/r2/r3 hashes, recompute path to root
    pub fn update_file(&mut self, file_id: &str, resolution_hashes: &HashMap<String, String>) {
        let leaf_idx = self.file_id_to_leaf_index(file_id);
        self.leaves.insert(file_id.to_string(), resolution_hashes.clone());
        self.recompute_path(leaf_idx); // O(height) = O(log n)
    }

    pub fn root(&self) -> String { ... }

    /// Generate O(log n) proof for one file's resolution
    pub fn prove_file_resolution(&self, file_id: &str, resolution: &str) -> MerkleProof { ... }
}
```

Store the incremental tree in the redb `kv_store` table as a compact binary blob. `root()` changes every time a file is added — the new root goes into `RootPayload.revocation_merkle_root` (which should really be called `library_merkle_root`).

---

## 20. CLI — TUI Shard Browser

**Priority:** 🟡 Medium  
**Crates affected:** `crates/cli/src/`

### Problem

The CLI TUI (`ratatui`) currently shows a file browser and sync progress. It has no way to inspect shard files, view shard health, or manually trigger recovery.

### New TUI Panel: Shard Inspector

```rust
// crates/cli/src/shard_browser.rs
pub struct ShardBrowserState {
    shards: Vec<ShardSummary>,
    selected: usize,
    detail_view: Option<ShardDetail>,
}

pub struct ShardSummary {
    pub shard_id: String,
    pub backend: String,
    pub file_count: u32,
    pub size_bytes: u64,
    pub health: ShardHealth,  // Healthy | Degraded | Missing
}
```

Ratatui layout:

```
┌─ Shards (12 total) ──────────────────────┐ ┌─ Shard Detail ────────────────────┐
│ shard_0001  github    345 files  48MB  ✓  │ │ ID:      shard_0001               │
│ shard_0002  github    312 files  42MB  ✓  │ │ Backend: GitHub (hautlythird211)   │
│ shard_0003  gitlab    289 files  39MB  !  │ │ Files:   345                       │
│ shard_0004  mega      401 files  61MB  ✗  │ │ Health:  DEGRADED (r2 missing)     │
│ parity_A    github    --         12MB  ✓  │ │ MAC:     ✓ valid                   │
└──────────────────────────────────────────┘ │ Actions: [R]ecover [V]erify [D]ump │
                                              └────────────────────────────────────┘
```

Key bindings: `r` → trigger recovery from parity, `v` → verify shard MAC, `d` → dump plaintext index (requires key input), `e` → export shard to local path.

---

## 21. Compression — Content-Aware Algorithm Selector

**Priority:** 🟠 High  
**Crates affected:** `crates/compression/src/triple.rs`

### Problem

`compress_triple()` always runs LZ4 → ZSTD → Brotli even for content types where the pipeline is counterproductive. JPEG files already use DCT compression; WebP files use VP8 compression. Running Brotli-11 on a JPEG reduces speed by 3x and produces larger output.

### Solution

Add `compress_adaptive()` that inspects MIME type and content structure before choosing algorithm:

```rust
pub fn compress_adaptive(data: &[u8], mime: Option<&str>) -> Result<(Vec<u8>, CompressionStats)> {
    let mime = mime.unwrap_or("application/octet-stream");

    match mime {
        // Already compressed: skip all layers
        m if m.starts_with("image/jpeg") ||
             m.starts_with("image/webp") ||
             m.starts_with("image/gif") ||
             m.starts_with("video/") ||
             m.starts_with("audio/") ||
             m == "application/zip" ||
             m == "application/gzip" => {
            // Just return as-is with an identity CompressionStats
            identity_stats(data)
        }

        // Text/code: Brotli is best
        m if m.starts_with("text/") ||
             m == "application/json" ||
             m == "application/javascript" => {
            self.compress_brotli(data)
        }

        // Large binary blobs (> 10MB): prefer ZSTD for speed
        _ if data.len() > 10_000_000 => {
            self.compress_zstd(data)
        }

        // Default: triple
        _ => self.compress_triple(data),
    }
}
```

Add `infer` crate (already in workspace via `src-tauri`) to sniff magic bytes when MIME is unknown.

---

## 22. Compression — Zstd Dictionary Training Pipeline

**Priority:** 🟡 Medium  
**Crates affected:** `crates/compression/`, `crates/cli/`

### Problem

Zstd dictionary training can improve compression ratio by 20-40% for collections of similar files (e.g., 10,000 photos with similar EXIF metadata patterns). The workspace currently uses default Zstd with no shared dictionary.

### Solution

Add a `train_zstd_dictionary()` command for CLI and Tauri:

```rust
// crates/compression/src/dictionary.rs
pub fn train_dictionary(samples: &[Vec<u8>], dict_size: usize) -> Result<Vec<u8>> {
    // zstd crate provides zstd::dict::from_samples()
    let dict = zstd::dict::from_samples(samples, dict_size)?;
    Ok(dict)
}

pub fn compress_with_dict(data: &[u8], dict: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut encoder = zstd::Encoder::with_dictionary(Vec::new(), level, dict)?;
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}
```

Workflow: when creating a new shard with 100+ files of the same MIME type, sample the first 1000 files (max 10MB total), train a 112KB dictionary, store the dictionary in the shard's `ErasureMeta` blob (itself compressed), and use it for all subsequent ZSTD-level encodes in that shard.

CLI command: `cybermanju train-dict --shard shard_0001.cybermanju --output dict.zstd`

---

## 23. Face Clustering — ONNX Model Auto-Download

**Priority:** 🟠 High  
**Crates affected:** `crates/faces/src/lib.rs`

### Problem

The ONNX face detection/embedding path (`feature = "onnx-face"`) requires `~/.cache/cybermanju/scrfd_2.5g.onnx` and `arcface_mfacenet.onnx` to exist. If they don't, it silently falls back to BLAKE3 pseudo-embeddings (which produce meaningless face groups — all faces cluster randomly). Users never know why face grouping doesn't work.

### Fix

Add auto-download with SHA256 verification:

```rust
const SCRFD_URL: &str = "https://huggingface.co/cybermanju/face-models/resolve/main/scrfd_2.5g.onnx";
const SCRFD_SHA256: &str = "...";
const ARCFACE_URL: &str = "https://huggingface.co/cybermanju/face-models/resolve/main/arcface_mfacenet.onnx";
const ARCFACE_SHA256: &str = "...";

pub async fn ensure_onnx_models(progress: impl Fn(DownloadProgress)) -> Result<OnnxModelPaths> {
    let cache = dirs::cache_dir().unwrap().join("cybermanju");
    let scrfd = cache.join("scrfd_2.5g.onnx");
    let arcface = cache.join("arcface_mfacenet.onnx");

    if !scrfd.exists() {
        download_and_verify(&SCRFD_URL, &scrfd, SCRFD_SHA256, &progress).await?;
    }
    if !arcface.exists() {
        download_and_verify(&ARCFACE_URL, &arcface, ARCFACE_SHA256, &progress).await?;
    }
    Ok(OnnxModelPaths { scrfd, arcface })
}
```

New Tauri command: `download_face_models()` → triggers `ensure_onnx_models()`, emits progress events to frontend. `FaceGroupingPanel.vue` shows "AI models required — download now? (44MB)" before first use.

Host models on Codeberg LFS (community-controlled alternative to HuggingFace).

---

## 24. SimHash Index — Persistent Disk Cache

**Priority:** 🟡 Medium  
**Crates affected:** `crates/faces/src/lib.rs`

### Problem

`SimHashIndex::new()` in the face crate builds the 64-bit SimHash table for all n faces at startup: O(n × d × B) = O(n × 512 × 64). For 100K faces, this is ~3.3 billion operations and takes ~2-5 seconds every startup.

### Solution

Persist the SimHash index to the redb `kv_store`:

```rust
pub fn load_or_build_simhash_index(
    embeddings: &[(String, [f32; 512])],  // (file_id, embedding)
    db: &Database,
) -> Result<SimHashIndex> {
    // Try to load cached index
    let cache_key = "simhash_index_v1";
    let cache_hash = embeddings_hash(embeddings);  // BLAKE3 of all embeddings

    if let Ok(Some(cached)) = db.get_kv(cache_key) {
        let entry: CachedSimHash = serde_json::from_str(&cached)?;
        if entry.embeddings_hash == cache_hash {
            return Ok(entry.index); // Hit — no rebuild needed
        }
    }

    // Miss — build and cache
    let index = SimHashIndex::new(embeddings);
    let entry = CachedSimHash { embeddings_hash: cache_hash, index: index.clone() };
    db.set_kv(cache_key, &serde_json::to_string(&entry)?)?;
    Ok(index)
}
```

Invalidate cache whenever `face_groups` table is modified (new import, face reassignment).

---

## 25. Key Hierarchy — Hardware Security Key (FIDO2)

**Priority:** 🟡 Medium  
**Crates affected:** `crates/preview-keys/src/key_derivation.rs`, `src-tauri/`

### Concept

The `master_key: [u8; 32]` currently lives only in memory (derived from password via Argon2id). For users who want physical security: allow binding the master key to a FIDO2 hardware token (YubiKey, SoloKey, etc.) using the HMAC-Secret extension.

```rust
// crates/preview-keys/src/fido2.rs (new, optional feature)
pub struct Fido2KeyBinding {
    pub credential_id: Vec<u8>,
    pub rp_id: String,  // e.g. "cybermanju.drive"
    pub salt: [u8; 32], // random per-device salt stored in redb
}

pub fn derive_master_key_from_fido2(
    binding: &Fido2KeyBinding,
    authenticator: &dyn Fido2Authenticator,
) -> Result<[u8; 32]> {
    // HMAC-Secret extension: authenticator computes HMAC(credential_secret, salt)
    // Result is 32 bytes → becomes master_key
    let hmac_output = authenticator.get_hmac_secret(&binding.credential_id, &binding.salt)?;
    Ok(hmac_output.try_into().unwrap())
}
```

Use `ctap-hid-fido2` or `fido2luks` crate for FIDO2 HID communication. FIDO2-bound keys never leave the hardware token; even if the `.cybermanju` file is copied, it cannot be decrypted without physical token presence.

---

## 26. Token Revocation — Merkle Accumulator CRL

**Priority:** 🟡 Medium  
**Crates affected:** `crates/preview-keys/src/view_token.rs`, `crates/resolutions/src/root.rs`

### Problem

`TokenStore::revoke_token()` stores revoked token IDs in a `HashSet<String>` in memory only. If the app restarts, all revocations are lost — any previously revoked token becomes valid again.

### Solution

Persist revocations using the `root.cybermanju` revocation Merkle tree (`RootPayload.revocation_merkle_root`). Use a Merkle accumulator: each revocation adds a new leaf, root changes, re-signed. Verifiers check: is token_id in the accumulator?

```rust
pub struct MerkleAccumulatorCRL {
    revoked_ids: Vec<String>,  // append-only
    root: String,               // BLAKE3 Merkle root
}

impl MerkleAccumulatorCRL {
    /// Append a revocation — O(log n) root update
    pub fn revoke(&mut self, token_id: &str) -> Result<RevocationProof> { ... }

    /// Check membership — O(log n) via Merkle proof
    pub fn is_revoked(&self, token_id: &str, proof: &RevocationProof) -> bool { ... }
}
```

Store in redb `kv_store["crl_v1"]`. Include in `RootPayload.revocation_merkle_root` so that any shard holder can verify revocation status without access to the full CRL list.

---

## 27. Backends — Headscale / Tailscale LAN Sync

**Priority:** 🟠 High  
**Crates affected:** `crates/backends/src/lan.rs` (new)

### Concept

Many users have multiple devices on the same LAN or VPN (Tailscale mesh). A `LanBackend` discovers other Cybermanju instances via mDNS (`mdns-sd` crate), authenticates via ML-DSA signature (already implemented), and syncs shards peer-to-peer without touching the internet.

```rust
pub struct LanBackend {
    service_name: String,      // mDNS service: "_cybermanju._tcp"
    device_signing_key: [u8; 32],
    peers: Vec<DiscoveredPeer>,
}

#[derive(Clone)]
pub struct DiscoveredPeer {
    pub name: String,
    pub addr: SocketAddr,
    pub public_key: Vec<u8>,   // ML-DSA verifying key
    pub capabilities: Vec<String>,
}
```

P2P transfer: use a minimal HTTP/1.1 server (same stack as `web_dashboard`) on a random LAN port, authenticated with a session token derived from both parties' keys (key agreement via X25519). No cloud relay needed.

```toml
mdns-sd = "0.11"
```

This enables the classic use case: take photos on phone → Cybermanju mobile (future) auto-syncs shards to the laptop over LAN before you even plug it in.

---

## 28. Backends — Rclone Abstraction Layer

**Priority:** 🟡 Medium  
**Crates affected:** `crates/backends/src/rclone.rs` (new)

### Concept

Rather than implementing every cloud provider natively, create a `RcloneBackend` that shells out to `rclone` (if installed). This immediately adds 70+ backends (S3, Dropbox, OneDrive, Backblaze B2, Azure Blob, Wasabi, etc.) with zero Rust code per backend.

```rust
pub struct RcloneBackend {
    remote_name: String,  // rclone config name, e.g. "myb2:mybucket"
    rclone_path: PathBuf, // path to rclone binary
}

impl StorageBackend for RcloneBackend {
    fn upload_file(&self, path: &str, remote_path: &str, content: &[u8]) -> Result<RemoteFile> {
        // Write content to temp file
        // rclone copy tempfile.bin remote_name:path/to/destination
        // Return RemoteFile with size and path
    }
}
```

Detection: check `which rclone` on startup, offer as optional backend in `SyncPanel.vue` only if found. Show all configured rclone remotes as available backends.

**Punk value:** This converts any rclone-supported platform into a Cybermanju backend without Anthropic, Google, or any company's approval — users plug in their own S3-compatible MinIO, Backblaze, or even a home NAS with SMB.

---

## 29. Web Dashboard — WebSocket Live Push

**Priority:** 🟠 High  
**Crates affected:** `src-tauri/src/web_dashboard/`

### Problem

`web_dashboard/mod.rs` implements HTTP/1.1 polling. The Vue frontend polls `/api/files` every few seconds to detect changes. This wastes bandwidth and adds latency on Docker/ZimaOS deployments.

### Solution

Add WebSocket upgrade support to the embedded HTTP server:

```rust
// Detect "Upgrade: websocket" in request headers
if is_websocket_upgrade(&req) {
    return handle_websocket_upgrade(stream, &state).await;
}

async fn handle_websocket_upgrade(stream: TcpStream, state: &AppState) {
    let ws = WebSocketHandshake::new(stream).upgrade().await?;
    let mut sub = state.event_bus.subscribe();
    loop {
        let event = sub.recv().await?;
        let json = serde_json::to_string(&event)?;
        ws.send_text(&json).await?;
    }
}
```

`AppState` gains an `event_bus: broadcast::Sender<AppEvent>`. All write commands (import, delete, encrypt, sync) publish events. The Vue `useTauri.ts` opens a WebSocket connection in web mode instead of polling:

```ts
function openEventStream() {
  const ws = new WebSocket(`ws://localhost:3456/ws`)
  ws.onmessage = (e) => {
    const event = JSON.parse(e.data)
    store.handleServerEvent(event)
  }
}
```

Use `tungstenite` or implement a minimal RFC 6455 handshake directly to avoid adding a heavy dependency to the embedded server.

---

## 30. Vue Frontend — Offline PWA Mode

**Priority:** 🟡 Medium  
**Crates affected:** `src/`, `public/`, `vite.config.wasm.ts`

### Problem

The WASM build (`npm run build:wasm`) produces a static SPA but has no Service Worker — every page load re-fetches all assets. In offline mode (Docker container on a local NAS with no internet), asset fetches fail silently.

### Solution

Add `vite-plugin-pwa` for automatic Service Worker generation:

```ts
// vite.config.wasm.ts
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    vue(),
    VitePWA({
      registerType: 'autoUpdate',
      workbox: {
        globPatterns: ['**/*.{js,css,html,wasm}'],
        maximumFileSizeToCacheInBytes: 50_000_000,  // 50MB for WASM
      },
      manifest: {
        name: 'Cybermanju Drive',
        short_name: 'Cybermanju',
        theme_color: '#1a1a2e',
        icons: [{ src: '/tauri.svg', sizes: 'any', type: 'image/svg+xml' }],
      }
    })
  ]
})
```

Cache strategy: `CacheFirst` for WASM binary, `NetworkFirst` for API calls with fallback to last known data. This makes Cybermanju Drive function as a full offline application on ZimaOS without internet access.

---

## 31. Vue Frontend — Drag-Drop Shard Visualizer

**Priority:** 🟢 Exploratory  
**Crates affected:** `src/components/StorageDashboard.vue`

### Concept

Replace the current `StorageDashboard.vue` pie chart with an interactive shard map: each shard appears as a draggable card, colored by backend type, sized by storage usage. Users can drag-drop shards between backend slots to reorganize distribution.

```
┌──────────────────── Shard Distribution ────────────────────────┐
│  [GitHub]            [GitLab]           [Mega]    [Local]      │
│  ┌──────────┐       ┌──────────┐       ┌──────┐   ┌──────┐    │
│  │ shard_01 │       │ shard_03 │       │ sh04 │   │ sh05 │    │
│  │ 48MB ✓  │       │ 39MB !  │       │ 61MB │   │ 22MB │    │
│  │ shard_02 │       │ parity_B │       │      │   │      │    │
│  │ 42MB ✓  │       │ 12MB ✓  │       │      │   │      │    │
│  └──────────┘       └──────────┘       └──────┘   └──────┘    │
│  [Drag shard to backend to migrate it]                          │
└────────────────────────────────────────────────────────────────┘
```

Implemented in Vue with native `draggable` attribute + `useWindowManager` for drag logic. Dropping a shard onto a different backend card triggers `migrate_shard(shard_id, new_backend_id)` Tauri command which re-uploads via `transfer_files()`.

---

## 32. System Monitor — Real sysinfo Integration

**Priority:** 🟠 High  
**Crates affected:** `src-tauri/src/`, `src/components/SystemMonitor.vue`

### Problem

`SystemMonitor.vue` displays CPU, RAM, and disk usage but `sysinfo` crate is listed in `src-tauri/Cargo.toml` without any Tauri command actually calling it. The component shows mock data or zeros.

### Fix

Create `src-tauri/src/commands/system.rs`:

```rust
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

#[tauri::command]
pub fn get_system_stats() -> SystemStats {
    let mut sys = System::new_all();
    sys.refresh_all();

    SystemStats {
        cpu_usage_percent: sys.global_cpu_info().cpu_usage(),
        memory_used_bytes: sys.used_memory(),
        memory_total_bytes: sys.total_memory(),
        disks: sys.disks().iter().map(|d| DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            mount: d.mount_point().to_string_lossy().to_string(),
            used_bytes: d.total_space() - d.available_space(),
            total_bytes: d.total_space(),
            fs_type: String::from_utf8_lossy(d.file_system()).to_string(),
        }).collect(),
        process_count: sys.processes().len() as u32,
        uptime_seconds: System::uptime(),
    }
}
```

Register in `lib.rs` `invoke_handler`. `SystemMonitor.vue` calls `get_system_stats()` every 2 seconds via `useIntervalFn` composable (VueUse).

---

## 33. Audit Log — Tamper-Evident Chaining

**Priority:** 🟡 Medium  
**Crates affected:** `crates/db/src/database.rs`, `crates/types/src/schema.rs`

### Problem

`AuditEntry` in the types crate has `id`, `action`, `file_id`, `user_id`, `timestamp`, `details` fields — but no chain linkage. An attacker with redb write access can delete or modify audit entries without detection.

### Solution

Add `prev_hash: String` to `AuditEntry`:

```rust
pub struct AuditEntry {
    pub id: String,
    pub action: String,
    pub file_id: Option<String>,
    pub user_id: String,
    pub timestamp: String,
    pub details: Option<serde_json::Value>,
    pub prev_hash: String,  // BLAKE3 of previous entry's serialized JSON
    pub entry_hash: String, // BLAKE3 of this entry (without entry_hash field)
}
```

On each write: load last entry, compute its hash, set as `prev_hash` of new entry, compute `entry_hash` of the new entry. On verification: scan from entry 0, recompute chain, detect any break.

New Tauri command: `verify_audit_chain() -> AuditChainResult { is_valid, broken_at, entry_count }`. Show in `ActivityPanel.vue` with a "Chain Integrity" indicator.

---

## 34. Plugin SDK — WASM Sandbox for Extensions

**Priority:** 🟢 Exploratory  
**Crates affected:** `src-tauri/`, `src/components/PluginCreator.vue`

### Concept

`PluginCreator.vue` exists in the frontend but has no Rust backend. The vision: users write Cybermanju plugins as WASM modules that can transform files (image filters, document converters, custom compressors) without full app access.

Use `wasmtime` (Bytecode Alliance runtime) to sandbox plugin execution:

```rust
// src-tauri/src/plugin_host/mod.rs
use wasmtime::{Engine, Store, Module, Linker};

pub struct PluginHost {
    engine: Engine,
    plugins: HashMap<String, LoadedPlugin>,
}

impl PluginHost {
    pub fn load_plugin(&mut self, wasm_bytes: &[u8], manifest: PluginManifest) -> Result<String> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        // Expose safe APIs: read_file(id) → bytes, write_file(id, bytes)
        // Block: filesystem access, network, system calls
        ...
    }

    pub fn run_plugin(&self, plugin_id: &str, file_id: &str) -> Result<Vec<u8>> { ... }
}
```

Plugin ABI: WASM modules export `transform(input_ptr, input_len) -> output_ptr`. The host provides `read_input()` and `write_output()` imports. Plugins are sandboxed with no filesystem or network access — they only see the bytes of the file they're processing.

`PluginCreator.vue` becomes a full plugin IDE: edit AssemblyScript/Rust code in CodeMirror, compile to WASM via API endpoint or local `wasm-pack`, test against sample files, publish to community plugin registry on Codeberg.

---

## 35. Paranoid Mode — Plausible Deniability Volumes

**Priority:** 🟡 Medium  
**Crates affected:** `crates/crypto/src/pqc.rs`, `crates/portable-db/`

### Concept

Inspired by VeraCrypt hidden volumes: a single `.cybermanju` file contains two independent encrypted sections. The outer section (decrypted with password A) reveals boring files. The inner section (decrypted with password B) reveals the sensitive files. An adversary forcing password disclosure never knows the inner section exists.

Implementation using ChaCha20 keystream collision:

```
.cybermanju deniable format:
├── [outer encrypted section]  → decrypted with master_key_A (Argon2id(password_A))
│     contains: vacation photos, documents, plausible content
└── [inner encrypted section]  → occupies "random padding" of outer section
      decrypted with master_key_B (Argon2id(password_B))
      contains: sensitive files
```

Key constraint: `master_key_B` must be completely independent of `master_key_A`. The outer section's "padding" bytes are in fact the inner section's ciphertext — ChaCha20 is a stream cipher, so random-looking outer padding = valid inner ciphertext.

New flag on `PortableDatabase::create()`: `DeniabilityMode::Enabled { inner_size_bytes }`.

---

## 36. Benchmark Suite — Criterion.rs Harness

**Priority:** 🟠 High  
**Crates affected:** `crates/tests/`

### Problem

`crates/tests/` has integration tests but no benchmarks. There is no way to detect performance regressions in the compression pipeline, face clustering, or crypto operations across commits.

### Solution

Create `crates/tests/benches/` with Criterion.rs benchmarks:

```toml
# crates/tests/Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "compression"
harness = false

[[bench]]
name = "crypto"
harness = false

[[bench]]
name = "face_clustering"
harness = false

[[bench]]
name = "shard_io"
harness = false
```

```rust
// benches/compression.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use cybermanju_compression::TripleCompressor;

fn bench_compress(c: &mut Criterion) {
    let compressor = TripleCompressor::new();
    let sizes = [1024, 65536, 1_048_576, 10_485_760];

    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        c.bench_with_input(BenchmarkId::new("triple_compress", size), &data, |b, data| {
            b.iter(|| compressor.compress_triple(data).unwrap())
        });
    }
}

criterion_group!(benches, bench_compress);
criterion_main!(benches);
```

Benchmark targets:

| Benchmark | Target | Why |
|-----------|--------|-----|
| `triple_compress/1MB` | < 200ms | Desktop import speed |
| `triple_decompress/1MB` | < 50ms | File open latency |
| `blake3_hash/10MB` | < 30ms | Import pipeline hash |
| `ml_kem_1024_keygen` | < 5ms | Key creation UX |
| `ml_kem_1024_encap` | < 5ms | Encryption latency |
| `face_cluster/1000` | < 200ms | Face grouping startup |
| `simhash_build/10000` | < 2s | Large library startup |
| `shard_write/100files` | < 1s | Import throughput |

Add to CI: `cargo bench --workspace -- --output-format bencher | tee bench.txt && critcmp bench.txt`

---

## Cross-Cutting Notes

### crates/recovery/ must be added to workspace (Task 1) before Tasks 6, 8

### Shard write pipeline (Task 2) unlocks Tasks 3, 4, 8, 10, 17, 18, 19

### Key hierarchy is solid; do NOT refactor `crates/preview-keys/` until Task 15 (FIDO2) is decided — the interface will change

### WASM bridge persistence (Task 4) is prerequisite for Task 30 (PWA offline mode)

### Nostr/IPFS/Torrent backends (Tasks 11-13) share a common pattern — create `crates/backends/src/p2p_common.rs` with shared utils (CID encoding, chunking for large files, progress tracking) before implementing all three individually

### The punk-anarchist principle is preserved throughout: every "big tech" backend (GitHub, GDrive, Telegram) is a **dumb encrypted blob store**. Tasks 11-13 add backends that nobody controls. Task 35 (deniability) and Task 25 (FIDO2) protect against coercion. The community owns the keys, the format, and increasingly the infrastructure.
