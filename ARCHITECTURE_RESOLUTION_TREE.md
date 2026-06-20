# Cybermanju Drive — Resolution Tree Architecture

> Machine-readable architecture context for `.cybermanju` v2, resolution-based
> file decomposition, cross-backend distribution, preview key system, and
> byte-level recovery with neural upscaling.

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Core Insight: Resolution as First-Class Citizen](#2-core-insight-resolution-as-first-class-citizen)
3. [`.cybermanju` v2 Binary Format](#3-cybermanju-v2-binary-format)
4. [Resolution Merkle Tree](#4-resolution-merkle-tree)
5. [Three-Tier Key System](#5-three-tier-key-system)
6. [Cross-Backend Distribution](#6-cross-backend-distribution)
7. [Erasure Coding Strategies](#7-erasure-coding-strategies)
8. [Compression Pipeline](#8-compression-pipeline)
9. [Shared Preview Library](#9-shared-preview-library)
10. [Recovery Pipeline](#10-recovery-pipeline)
11. [Technology Stack](#11-technology-stack)
12. [Implementation Roadmap](#12-implementation-roadmap)
13. [Design Decisions Log](#13-design-decisions-log)
14. [Crate Migration Guide](#14-crate-migration-guide)

---

## 1. Problem Statement

The current architecture treats a file as **one blob** — compress it, encrypt it,
ship it to one backend. The portable DB stores recovery data in flat sidecar
directories. Versioning only snapshots metadata (hash + size), not actual content
decomposition.

### Limitations

| Problem | Current State | Impact |
|---------|--------------|--------|
| No resolution awareness | Full decrypt+decompress for preview | Slow gallery loads, wasted bandwidth |
| Flat resilience | All-or-nothing per backend | One backend death = total loss or zero loss |
| No preview access control | Same key for everything | Cannot share previews without exposing originals |
| No dedup across resolutions | Each resolution stored independently | Wasted storage for same content at different sizes |
| No byte-level recovery | Binary success/fail | Partial data loss = total loss |
| Legacy crypto | `pqcrypto-*` C wrappers | Archiving June 2026, no pure-Rust path |

---

## 2. Core Insight: Resolution as First-Class Citizen

A "file" is not a single entity — it is a **Merkle tree of resolutions**,
each atomically stored, independently encrypted, potentially distributed across
different backends.

```
File "photo.jpg" (4032x3024, 8MB)
├── r0: 200x200 WebP (3KB)     → ALL backends (redundant, tiny)
├── r1: 640x480 JPEG (45KB)    → GitHub + Google Drive (2+ backends)
├── r2: 1920x1080 JPEG (450KB) → Local + MEGA (1-2 backends)
├── r3: Original (8MB)          → Split across Local + MEGA (MSR coded)
└── parity: MSR shards (2MB)    → GitHub + GitLab (parity distribution)
```

Each resolution node has its own BLAKE3 hash. The root hash is the file's
identity. This is the "git tree" for resolutions.

### Resolution Levels

| Level | Name | Typical Size | Typical Use | Distribution |
|-------|------|-------------|-------------|-------------|
| r0 | Root/Thumbnail | 2-5KB | Gallery, face grid, instant preview | ALL backends |
| r1 | Preview | 40-60KB | Share link, card view, quick glance | 2+ backends |
| r2 | Medium | 400-600KB | Detail view, editing preview | 1-2 backends |
| r3 | Full/Original | Variable | Download, archival, print | Split/MSR across backends |
| rp | Parity | ~33% of r3 | Recovery when shards lost | 2+ cheap backends |

### Access Patterns

| Scenario | Data Needed | Latency Target | Backend Hit |
|----------|------------|----------------|-------------|
| Gallery thumbnail | r0 (3KB) | <50ms | 1 (any) |
| Share preview | r0 + r1 (48KB) | <200ms | 1 (any with r0) |
| Detail view | r1 + r2 (495KB) | <500ms | 1-2 |
| Full download | r3 (8MB) | <5s | Split reconstruction |
| Disaster recovery | Parity + available shards | <30s | 2+ |
| Approximate recovery | Lower res + upscaler | <5s | 1 (any with lower res) |

---

## 3. `.cybermanju` v2 Binary Format

### File Structure

```
.cybermanju
├── [0..32)      magic: "CYBERMANJU_V2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
├── [32..36)     header_json_len (u32 LE)
├── [36..+h)     header_json (PortableHeaderV2)
├── [+h..+m)     merkle_manifest_json (ResolutionManifest)
├── [+m..+p)     erasure_config_json (ErasureConfig)
├── [+p..+k)     preview_access_config_json (PreviewAccessConfig)
├── [+k..+s)     sync_state_json (SyncState)
└── [+s..)       encrypted_compressed_redb (triple-layer, optionally PQC encrypted)
```

### Sidecar Directory Structure

```
.cybermanju.cyb3/
├── r0/                          # Resolution 0 — thumbnails
│   ├── {file_id}.webp           # 200x200 WebP, ~2-5KB
│   └── {file_id}.meta           # {w, h, mime, original_blake3}
├── r1/                          # Resolution 1 — previews
│   ├── {file_id}.jpg            # 640x480 JPEG, ~45KB
│   └── {file_id}.meta
├── r2/                          # Resolution 2 — medium
│   ├── {file_id}.jpg            # 1920x1080, ~450KB
│   └── {file_id}.meta
├── r3/                          # Resolution 3 — original
│   ├── {file_id}.enc            # Encrypted original
│   └── {file_id}.meta
├── parity/                      # Erasure coding shards
│   ├── {file_id}.clay.shard.0   # MSR parity shard 0
│   ├── {file_id}.clay.shard.1   # MSR parity shard 1
│   ├── {file_id}.fountain pkt   # Fountain code packets
│   └── {file_id}.fountain pkt
├── sprites/                     # Sprite sheets for gallery view
│   └── {batch_id}.png           # 16 thumbnails per sheet (4x4 grid)
├── previews/                    # Ultra-compressed shared library
│   └── {blake3_prefix}/
│       └── {hash}.webp          # Deduplicated, content-addressed
└── keys/                        # Encrypted key material
    ├── {file_id}.master.enc     # Master-encrypted content key
    └── {file_id}.preview.enc    # Preview-encrypted preview key
```

### PortableHeaderV2

```json
{
  "version": "2.0",
  "created_at": "2026-06-20T00:00:00Z",
  "last_modified_at": "2026-06-20T00:00:00Z",
  "app_version": "0.2.0",
  "db_hash": "blake3:abcdef...",
  "encryption": {
    "algorithm": "ml-kem-1024+chacha20poly1305",
    "key_id": "master-key-001",
    "kdf": "hkdf-sha256"
  },
  "compression": {
    "algorithm": "lz4+zstd15+brotli11",
    "fast_path": "lz4+zstd3"
  },
  "resolution_config": {
    "levels": 4,
    "level_specs": [
      { "level": 0, "name": "thumbnail",  "max_dim": 200,  "format": "webp",  "quality": 80 },
      { "level": 1, "name": "preview",    "max_dim": 640,  "format": "jpeg",  "quality": 75 },
      { "level": 2, "name": "medium",     "max_dim": 1920, "format": "jpeg",  "quality": 90 },
      { "level": 3, "name": "original",   "max_dim": 0,    "format": "native","quality": 100 }
    ]
  },
  "erasure": {
    "r3_codec": "clay-codes",
    "r3_params": { "k": 3, "m": 1, "d": 4 },
    "parity_codec": "fountain-raptorq",
    "parity_params": { "symbol_size": 1024, "redundancy": 0.33 }
  },
  "preview_access": {
    "token_standard": "paseto-v4",
    "key_derivation": "hkdf-sha256",
    "max_view_tokens": 100,
    "token_expiry_hours": 24,
    "revocable": true
  },
  "sync": {
    "crdt": "delta-state",
    "vector_clock": true,
    "last_sync_hash": "blake3:..."
  },
  "stats": {
    "total_files": 1234,
    "total_previews": 1234,
    "total_relations": 5678,
    "total_deletions": 12,
    "db_size_bytes": 1048576,
    "content_store_size": 52428800,
    "preview_store_size": 2097152,
    "parity_store_size": 17301504
  },
  "platform_origin": "linux",
  "synced_platforms": ["github", "google_drive", "mega"]
}
```

### ResolutionManifest

```json
{
  "files": {
    "file_id_001": {
      "name": "photo.jpg",
      "mime": "image/jpeg",
      "original_size": 8388608,
      "original_blake3": "blake3:abcdef1234567890...",
      "resolutions": {
        "r0": {
          "hash": "blake3:thumb_hash...",
          "size": 3072,
          "format": "webp",
          "width": 200,
          "height": 150,
          "backends": ["github", "gitlab", "google_drive", "local", "mega"],
          "blob_path": "r0/{file_id}.webp"
        },
        "r1": {
          "hash": "blake3:preview_hash...",
          "size": 46080,
          "format": "jpeg",
          "width": 640,
          "height": 480,
          "backends": ["github", "google_drive"],
          "blob_path": "r1/{file_id}.jpg"
        },
        "r2": {
          "hash": "blake3:medium_hash...",
          "size": 460800,
          "format": "jpeg",
          "width": 1920,
          "height": 1080,
          "backends": ["local", "mega"],
          "blob_path": "r2/{file_id}.jpg"
        },
        "r3": {
          "hash": "blake3:original_hash...",
          "size": 8388608,
          "format": "encrypted",
          "backends": ["local", "mega"],
          "blob_path": "r3/{file_id}.enc",
          "erasure": {
            "codec": "clay-codes",
            "shards": 3,
            "parity": 1,
            "shard_size": 2796203,
            "shard_backend_map": {
              "shard_0": "local",
              "shard_1": "mega",
              "shard_2": "local",
              "parity_0": "github"
            }
          }
        }
      },
      "parity": {
        "hash": "blake3:parity_hash...",
        "size": 2796203,
        "codec": "fountain-raptorq",
        "packets_stored": 8,
        "min_packets_needed": 5,
        "backends": ["github", "gitlab"],
        "blob_path": "parity/{file_id}.fountain"
      },
      "sprite": {
        "hash": "blake3:sprite_hash...",
        "size": 30720,
        "grid": "4x4",
        "backends": ["github", "gitlab", "google_drive", "local", "mega"]
      }
    }
  },
  "merkle_root": "blake3:root_hash_of_all_file_merkle_roots"
}
```

---

## 4. Resolution Merkle Tree

Each file has a Merkle tree where the root is the file's identity and leaves
are individual resolutions.

### Tree Structure

```
                    File Merkle Root (BLAKE3)
                   /                          \
          BLAKE3(r0_hash || r1_hash)    BLAKE3(r2_hash || r3_hash)
           /            \                /              \
        r0_hash      r1_hash         r2_hash          r3_hash
        (200x200)   (640x480)       (1920x1080)     (original)
```

### Hash Computation

```rust
fn compute_file_merkle_root(resolutions: &ResolutionMap) -> blake3::Hash {
    let r0_hash = blake3::hash(&resolutions.r0.data);
    let r1_hash = blake3::hash(&resolutions.r1.data);
    let r2_hash = blake3::hash(&resolutions.r2.data);
    let r3_hash = blake3::hash(&resolutions.r3.data);

    let left = blake3::hashv(&[r0_hash.as_bytes(), r1_hash.as_bytes()]);
    let right = blake3::hashv(&[r2_hash.as_bytes(), r3_hash.as_bytes()]);

    blake3::hashv(&[left.as_bytes(), right.as_bytes()])
}
```

### Integrity Verification

```rust
fn verify_resolution(
    file_root: blake3::Hash,
    level: ResolutionLevel,
    data: &[u8],
    proof: MerkleProof,
) -> bool {
    let leaf_hash = blake3::hash(data);
    proof.verify(file_root, leaf_hash, level.as_index())
}
```

### Properties

- **Per-resolution integrity**: Each resolution independently verifiable
- **Deduplication**: Same content at same resolution = same hash = shared blob
- **Tamper evidence**: Any modification changes the root hash
- **Selective disclosure**: Share r0 hash without revealing r3

---

## 5. Three-Tier Key System

### Key Hierarchy

```
Master Key (user-controlled, permanent, 256-bit)
├── Derives → Content Key    (encrypts r2, r3 — full resolution data)
├── Derives → Preview Key    (encrypts r0, r1 — thumbnails/previews)
└── Derives → View Token Key (time-limited, view-limited, encrypts ONLY r0)
```

### Key Derivation

```rust
use hkdf::Hkdf;
use sha2::Sha256;

fn derive_content_key(master_key: &[u8], file_id: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(
        Some(b"cybermanju-content-v1"),
        master_key
    );
    let mut key = [0u8; 32];
    hk.expand(file_id.as_bytes(), &mut key).unwrap();
    key
}

fn derive_preview_key(master_key: &[u8], file_id: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(
        Some(b"cybermanju-preview-v1"),
        master_key
    );
    let mut key = [0u8; 32];
    hk.expand(file_id.as_bytes(), &mut key).key
}

fn derive_view_token_key(
    master_key: &[u8],
    file_id: &str,
    token_id: &str,
) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(
        Some(b"cybermanju-view-token-v1"),
        master_key
    );
    let mut key = [0u8; 32];
    let info = format!("{}:{}", file_id, token_id);
    hk.expand(info.as_bytes(), &mut key).unwrap();
    key
}
```

### View Token (PASETO V4)

```
PASETO V4 Local Token:
├── Footer: {"kid": "preview-key-001", "alk": "ML-KEM-1024"}
├── Claims:
│   ├── sub: "file_id_001"
│   ├── res: "r0"              (max resolution allowed)
│   ├── vcn: 10                (max view count)
│   ├── exp: "2026-06-21T00:00:00Z"  (expiry)
│   └── jti: "token_uuid"     (for revocation)
└── Encryption: AES-256-CTR + BLAKE3-MACT
    key = HKDF(preview_key, file_id, token_id)
```

### Token Lifecycle

```
Generate Token:
  1. Generate token_id (UUID)
  2. Derive token_key = HKDF(preview_key, file_id, token_id)
  3. Encrypt r0 data with token_key
  4. Create PASETO V4 token with claims
  5. Store token metadata in DB (for revocation)
  6. Return encrypted r0 + token

Validate Token:
  1. Parse PASETO V4 token
  2. Check expiry (exp claim)
  3. Check revocation list (jti claim)
  4. Derive token_key = HKDF(preview_key, file_id, token_id)
  5. Decrypt r0 data with token_key
  6. Increment view count (if < vcn, else revoke)

Revoke Token:
  1. Add jti to revocation Merkle tree
  2. Update revocation root in .cybermanju header
  3. Token becomes invalid on next validation
```

### Access Control Matrix

| Resolution | Master Key | Content Key | Preview Key | View Token |
|-----------|-----------|-------------|-------------|------------|
| r0 (thumb) | Full access | Full access | Full access | Time-limited, view-limited |
| r1 (preview) | Full access | Full access | Full access | No access |
| r2 (medium) | Full access | Full access | No access | No access |
| r3 (original) | Full access | Full access | No access | No access |
| Parity shards | Full access | Full access | No access | No access |

---

## 6. Cross-Backend Distribution

### Backend Types (Existing)

| Backend | Protocol | Auth | Max Size | LFS Support |
|---------|----------|------|----------|-------------|
| Local | std::fs | None | Unlimited | No |
| GitHub | REST API | PAT | 2GB (Releases) | Yes (1MB threshold) |
| GitLab | REST API | PAT | 10MB (API) | Yes |
| Codeberg | REST API | PAT | 10MB (API) | Yes |
| Gitea | REST API | PAT | 10MB (API) | Yes |
| Google Drive | Drive API v3 | OAuth | 5TB | No |
| Google Photos | Photos API | OAuth | N/A | No |
| Telegram | Bot API | Token | 2GB | No |
| MEGA | MegaSDK | Email+Pass | 1TB | No |

### Distribution Policy

```rust
struct DistributionPolicy {
    /// Which resolution level → which backends
    placement: HashMap<ResolutionLevel, Vec<BackendSelector>>,

    /// Minimum redundancy per resolution
    min_redundancy: HashMap<ResolutionLevel, u32>,

    /// Erasure coding configuration
    erasure: ErasureConfig,

    /// Cost-aware placement (prefer cheaper backends for larger data)
    cost_model: Option<CostModel>,
}

enum BackendSelector {
    /// Specific backend by ID
    Specific(String),
    /// Any backend matching predicate
    Matching(fn(&dyn StorageBackend) -> bool),
    /// Random selection from pool
    Random { pool: Vec<String>, count: usize },
    /// Round-robin across pool
    RoundRobin { pool: Vec<String> },
}
```

### Default Distribution Policy

```yaml
resolution_distribution:
  r0:
    backends: [ALL]
    redundancy: max        # all available backends
    erasure: none          # full copies, too small for EC
    priority: instant      # must be fastest to access

  r1:
    backends: [github, google_drive, local]
    redundancy: 2          # at least 2 copies
    erasure: none          # small enough for full copies
    priority: fast         # <200ms access

  r2:
    backends: [local, mega]
    redundancy: 1          # at least 1 copy
    erasure: optional      # RS if backend supports it
    priority: normal       # <500ms access

  r3:
    backends: [local, mega, github]
    redundancy: 0          # no full copies
    erasure:
      codec: clay-codes
      params: { k: 3, m: 1, d: 4 }  # 33% overhead, 2.9× less repair BW
    priority: background   # reconstructed on demand

  parity:
    backends: [github, gitlab]
    redundancy: 0
    erasure:
      codec: fountain-raptorq
      params: { symbol_size: 1024, redundancy: 0.33 }
    priority: background   # used only for recovery
```

### Cost-Aware Placement

```rust
struct CostModel {
    /// Cost per GB per month for each backend
    storage_cost: HashMap<String, f64>,
    /// Cost per GB for download
    download_cost: HashMap<String, f64>,
    /// Cost per GB for upload
    upload_cost: HashMap<String, f64>,
}

impl CostModel {
    fn optimize_placement(
        &self,
        file: &FileResolutions,
        policy: &DistributionPolicy,
    ) -> PlacementPlan {
        // For r3 (large): use cheapest backends with sufficient bandwidth
        // For r0 (tiny): ignore cost, maximize availability
        // For parity: use cheapest backends with sufficient storage
    }
}
```

---

## 7. Erasure Coding Strategies

### Tier 1: Reed-Solomon (Proven, Fast)

**Use case**: r3 distribution when backend count is fixed and known.

```toml
[dependencies]
reed-solomon-simd = "3.1.0"  # 10.2 GiB/s encode, O(n log n)
```

```rust
use reed_solomon_simd::ReedSolomon;

// RS(3,4) — 3 data shards, 1 parity, tolerates 1 loss
let rs = ReedSolomon::new(3, 1)?;

let shards = rs.encode(&data)?;
// shards: [data0, data1, data2, parity0]

// Distribute across backends
for (i, shard) in shards.iter().enumerate() {
    backend.upload(&format!("shard_{}", i), shard)?;
}

// Reconstruct from any 3 of 4 shards
let recovered = rs.reconstruct(&available_shards)?;
```

**Performance** (256:256 shards on Ryzen 5 3600):
- Encode: 10.2 GiB/s
- Decode: 1.0 GiB/s
- SIMD: SSSE3, AVX2, NEON (runtime dispatch)

### Tier 2: Fountain Codes (Rateless)

**Use case**: Parity distribution where backend count is variable.

```toml
[dependencies]
fountain_scheme = "1.0.1"
```

```rust
use fountain_scheme::{Encoder, Decoder, RaptorQ};

// Create fountain encoder
let encoder = RaptorQ::new(symbol_size: 1024, source_symbols: &data)?;

// Generate infinite stream of encoded packets
let packet = encoder.encode(sequence_number);

// Distribute packets across backends (any order, any count)
for packet in encoder.take(10) {
    let backend = select_backend();
    backend.upload(&packet.id(), &packet.encode())?;
}

// Decode from ANY k packets (regardless of which ones)
let mut decoder = Decoder::new(symbol_size: 1024, source_len: data.len());
for packet in received_packets {
    decoder.add_packet(packet)?;
}
let recovered = decoder.decode()?;
```

**Properties**:
- Rateless: generate infinite packets from k source symbols
- Any k-of-n packets reconstruct the data
- No fixed shard assignment needed
- Ideal for variable-backend-count scenarios

### Tier 3: MSR Codes (Minimum Storage Regenerating)

**Use case**: r3 distribution where repair bandwidth matters.

```toml
[dependencies]
clay-codes = "0.1.1"
```

```rust
use clay_codes::{ClayCodec, Shard};

// Clay(k=3, m=1, d=4) — same storage overhead as RS(3,4)
// but 2.9× less repair bandwidth when one shard is lost
let codec = ClayCodec::new(k: 3, m: 1, d: 4)?;

let shards = codec.encode(&data)?;

// When shard_2 is lost:
// RS: download 3 full shards (24MB) to reconstruct 1 shard (8MB)
// Clay: download β sub-chunks from d=4 helpers (4MB) to reconstruct
// → 2.9× less bandwidth
let repaired = codec.repair(lost_shard: 2, available: &shards)?;
```

### Tier 4: Shamir Secret Sharing (Key Splitting)

**Use case**: Splitting master key across devices.

```toml
[dependencies]
shamir-zero = "0.1.10"  # 2.8× faster than Go, zero-unsafe
```

```rust
use shamir_zero::Shamir;

// Split master key into 5 shares, any 3 can reconstruct
let shares = Shamir::split(&master_key, threshold: 3, shares: 5)?;

// Distribute shares across devices
device_1.store(&shares[0]);
device_2.store(&shares[1]);
device_3.store(&shares[2]);
device_4.store(&shares[3]);
device_5.store(&shares[4]);

// Reconstruct from any 3 shares
let reconstructed = Shamir::combine(&[share_0, share_1, share_2])?;
```

### Combined Strategy

```rust
fn distribute_file(file: &FileResolutions, policy: &DistributionPolicy) {
    // 1. r0: Full copies to ALL backends
    for backend in all_backends() {
        backend.upload("r0", &file.r0_data);
    }

    // 2. r1: Full copies to 2+ backends
    let r1_backends = select_backends(policy.r1, 2);
    for backend in r1_backends {
        backend.upload("r1", &file.r1_data);
    }

    // 3. r2: Full copy to 1-2 backends
    let r2_backends = select_backends(policy.r2, 1);
    for backend in r2_backends {
        backend.upload("r2", &file.r2_data);
    }

    // 4. r3: Clay codes across 3+ backends
    let clay_shards = ClayCodec::encode(&file.r3_data, k: 3, m: 1)?;
    let r3_backends = select_backends(policy.r3, 4);
    for (shard, backend) in clay_shards.iter().zip(r3_backends) {
        backend.upload("r3_shard", shard);
    }

    // 5. Parity: Fountain codes across 2+ backends
    let fountain_packets = RaptorQ::encode(&file.r3_data, symbol_size: 1024)?;
    let parity_backends = select_backends(policy.parity, 2);
    for packet in fountain_packets.take(8) {
        let backend = parity_backends.next();
        backend.upload("parity", &packet.encode());
    }

    // 6. Master key: Shamir split across 5 devices
    let key_shares = Shamir::split(&master_key, threshold: 3, shares: 5)?;
    distribute_key_shares(key_shares);
}
```

---

## 8. Compression Pipeline

### Current Triple-Layer (LZ4 → Zstd → Brotli)

```
COMPRESS: data → LZ4 compress → if ratio > 0.98: skip → Zstd(level=15) → Brotli(level=11) → output
DECOMPRESS: data → Brotli decompress → Zstd decompress → LZ4 decompress → original
```

### Recommended Upgrade

| Layer | Current | Upgrade | Why |
|-------|---------|---------|-----|
| L1 | `lz4_flex` 0.12 | `lz4_flex` 0.13.1 | Pure Rust, no C deps, `no_std` |
| L2 | `zstd` (unknown) | `zstd` 0.13.3 (zstd 1.5.7) | +30% compression speed on small data |
| L3 | `brotli` (unknown) | `brotli` 8.0.4 | Panic-safe, no_std |
| NEW | — | `fastcdc` 4.0.1 | Content-defined chunking for dedup |
| NEW | — | `ravif` 0.13.0 | AVIF encoding (better than JPEG) |
| NEW | — | `mozjpeg-rs` 0.9.2 | Pure safe-Rust JPEG, trellis quantization |

### Benchmark Ratios (2026)

| Algorithm | Weighted Ratio | Speed Profile | Best For |
|-----------|---------------|---------------|----------|
| lzma2 9e | **1.667x** | Very slow compress, fast decompress | Max ratio text archives |
| zstd 22 | **1.633x** | Slow compress, fast decompress | Best all-around |
| brotli 11 | **1.632x** | Slow compress, fast decompress | Web text assets |
| zstd 3 | ~1.4x | Very fast compress | **Everyday default** |
| lz4 | ~1.3x | Extremely fast both | Real-time, local |
| gzip 4 | ~1.3x | Medium | Legacy compatibility |

### Per-Resolution Compression

| Resolution | Format | Compression | Level | Rationale |
|-----------|--------|-------------|-------|-----------|
| r0 (thumb) | WebP | Lossy only | q80 | Tiny, quality matters more than ratio |
| r1 (preview) | JPEG | mozjpeg trellis | q75 | Small, balance quality/size |
| r2 (medium) | JPEG | mozjpeg trellis | q90 | Medium, good quality |
| r3 (original) | Native | Triple-layer | LZ4→Zstd15→Brotli11 | Maximum ratio |
| Parity | Raw | None | — | Already encoded data |

### Content-Defined Chunking for Dedup

```toml
[dependencies]
fastcdc = "4.0.1"
```

```rust
use fastcdc::v2020::FastCDC;

// Chunk file content-defined boundaries
let mutcdc = FastCDC::new(data, min_size: 2048, max_size: 65536, avg_size: 32768);

let chunks: Vec<(u64, Vec<u8>)> = cdc
    .filter_map(|chunk| {
        if chunk.length > 0 {
            let hash = blake3::hash(&data[chunk.offset..chunk.offset + chunk.length]);
            Some((hash.into(), data[chunk.offset..chunk.offset + chunk.length].to_vec()))
        } else {
            None
        }
    })
    .collect();

// Dedup: same content = same hash = shared blob
let deduped: HashMap<blake3::Hash, Vec<u8>> = chunks.into_iter().collect();
```

---

## 9. Shared Preview Library

### Structure

```
SharedPreviewLibrary/
├── index.blake3                    # Merkle root of all previews
├── config.json                     # Library metadata
├── by_hash/                        # Content-addressed storage
│   ├── ab/
│   │   └── cd/
│   │       └── abcdef1234...webp   # Deduplicated preview blob
├── by_file/                        # File → preview mapping
│   └── {file_id}.json             # {r0_hash, r1_hash, sprite_hash}
├── sprite_sheets/                  # Batch preview sheets
│   └── batch_{n}.png              # 4x4 grid of 200x200 thumbs = 800x800px
└── metadata/                       # Preview metadata
    └── {hash}.meta                # {width, height, mime, original_size}
```

### Sprite Sheet Generation

```rust
struct SpriteSheet {
    grid_size: (u32, u32),     // 4x4 = 16 thumbnails
    thumb_size: (u32, u32),    // 200x200 each
    padding: u32,              // 4px between thumbnails
}

impl SpriteSheet {
    fn generate(thumbnails: &[Vec<u8>]) -> Vec<u8> {
        // Create 800x800 RGBA image
        let mut sheet = ImageBuffer::new(800, 800);

        for (i, thumb) in thumbnails.iter().enumerate() {
            let x = (i % 4) * 204;  // 200 + 4px padding
            let y = (i / 4) * 204;
            let img = image::load_from_memory(thumb).unwrap();
            sheet.copy_from(&img, x, y).unwrap();
        }

        // Encode as optimized PNG
        let mut buf = Cursor::new(Vec::new());
        sheet.write_to(&mut buf, ImageFormat::Png).unwrap();
        buf.into_inner()
    }
}
```

### Benefits

- **Deduplication**: Same image content shares the same preview hash
- **Ultra-compression**: 200x200 WebP is typically 2-5KB (1/1600 of 8MB photo)
- **Sprite sheets**: 16 thumbnails in one 800x800 PNG (~30KB) for gallery loading
- **Content-addressed**: Same content = same hash = automatic dedup
- **Portable**: Syncs to ALL backends since it's so small

---

## 10. Recovery Pipeline

### Recovery Gradient

| Available Data | Strategy | Result Quality |
|---------------|----------|---------------|
| r0 only (3KB) | Lanczos upscale | ~r1 quality (480p) |
| r0 + parity (3KB+1MB) | RS reconstruct + upscale | Exact r1 + approx r2 |
| r1 only (45KB) | Real-ESRGAN upscale | ~r2 quality (1080p) |
| r1 + r2 (45KB+450KB) | Partial decode + fill | Exact r2 + approx r3 |
| r2 + parity (450KB+2MB) | RS reconstruct | Exact r3 (original) |
| r3 shards (3/4 available) | Reed-Solomon decode | Exact r3 (original) |
| r3 packets (k/∞ available) | Fountain decode | Exact r3 (original) |
| ALL data lost | Metadata + hash only | Proof of existence |

### Recovery Strategies

```rust
enum RecoveryStrategy {
    /// Exact reconstruction from Clay/RS shards
    ErasureReconstruct {
        shards: Vec<Shard>,
        codec: ClayCodes,               // clay-codes 0.1.1
    },

    /// Exact reconstruction from fountain packets
    FountainReconstruct {
        packets: Vec<FountainPacket>,
        codec: RaptorQ,                 // fountain_scheme 1.0.1
    },

    /// Approximate reconstruction via neural upscaling
    UpscaleFromLower {
        source: ResolutionLevel,
        target: ResolutionLevel,
        upscaler: UpscalerEngine,
    },

    /// Fast traditional upscaling (no AI)
    FastLanczos {
        source: ResolutionLevel,
        target: ResolutionLevel,
        resize: FastImageResize,        // fast_image_resize 6.0
    },

    /// Hybrid: partial reconstruction + upscaling
    HybridReconstruct {
        partial_shards: Vec<Shard>,
        partial_data: Vec<u8>,
        upscaler: UpscalerEngine,
    },
}
```

### Upscaler Engines

```rust
enum UpscalerEngine {
    /// Built-in SIMD Lanczos3 — fast, no model needed
    LanczosClassic,                     // fast_image_resize 6.0

    /// Neural upscaling via ONNX Runtime
    OnnxRealEsrGan {
        model_path: PathBuf,            // ~60MB ONNX model
        execution_provider: GpuProvider, // CUDA/Metal/Vulkan/CPU
    },                                  // ort 2.0 + Real-ESRGAN ONNX

    /// Neural upscaling via Burn framework
    BurnEsrGan {
        model: BurnModel,               // burn 0.21
    },

    /// Frequency-domain recovery
    FourierRecover,                     // Custom implementation
}
```

### Recovery Pipeline Implementation

```rust
impl RecoveryPipeline {
    async fn recover(
        &self,
        file_id: &str,
        target: ResolutionLevel,
        available: &AvailableData,
    ) -> Result<RecoveryResult> {
        // 1. Check if exact data available
        if let Some(data) = self.try_exact_recovery(file_id, target, available) {
            return Ok(RecoveryResult::Exact(data));
        }

        // 2. Try erasure reconstruction
        if available.has_enough_shards(target) {
            let data = self.erasure_reconstruct(available)?;
            return Ok(RecoveryResult::Exact(data));
        }

        // 3. Try fountain reconstruction
        if available.has_enough_packets() {
            let data = self.fountain_reconstruct(available)?;
            return Ok(RecoveryResult::Exact(data));
        }

        // 4. Try upscaling from lower resolution
        if let Some(lower) = target.lower_resolution() {
            if let Some(lower_data) = self.load_resolution(file_id, lower) {
                let upscaled = self.upscaler.upscale(&lower_data, target)?;
                let confidence = self.estimate_confidence(lower, target);
                return Ok(RecoveryResult::Approximate(upscaled, confidence));
            }
        }

        // 5. Try hybrid reconstruction
        if available.has_partial_data() {
            let data = self.hybrid_reconstruct(available, target)?;
            let confidence = self.estimate_hybrid_confidence(available, target);
            return Ok(RecoveryResult::Approximate(data, confidence));
        }

        // 6. Nothing available
        Err(RecoveryError::InsufficientData {
            file_id: file_id.to_string(),
            target,
            available: available.summary(),
        })
    }

    fn try_exact_recovery(
        &self,
        file_id: &str,
        target: ResolutionLevel,
        available: &AvailableData,
    ) -> Option<Vec<u8>> {
        // Check if we have the exact resolution blob
        available.get_resolution(file_id, target)
    }

    fn estimate_confidence(
        &self,
        source: ResolutionLevel,
        target: ResolutionLevel,
    ) -> f64 {
        // Lower resolution → lower confidence
        let ratio = source.size_ratio_to(target);
        match ratio {
            r if r > 0.5 => 0.9,   // Close resolutions, high confidence
            r if r > 0.25 => 0.7,  // Medium gap, moderate confidence
            r if r > 0.1 => 0.5,   // Large gap, lower confidence
            _ => 0.3,              // Very large gap, low confidence
        }
    }
}
```

### Neural Upscaling Integration

```toml
[dependencies]
ort = { version = "2.0.0-rc.12", features = ["load-dynamic"] }
fast_image_resize = "6.0"
image = "0.25.10"
```

```rust
use ort::{Session, SessionBuilder, GraphOptimizationLevel};

struct OnnxUpscaler {
    session: Session,
}

impl OnnxUpscaler {
    fn new(model_path: &Path, provider: GpuProvider) -> Result<Self> {
        let session = SessionBuilder::new()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cpus::get() as i64)?;

        let session = match provider {
            GpuProvider::Cuda => session.with_cuda()?,
            GpuProvider::Metal => session.with_coreml()?,
            GpuProvider::Vulkan => session.with_vulkan()?,
            GpuProvider::Cpu => session,
        };

        let session = session.commit_from_file(model_path)?;

        Ok(Self { session })
    }

    fn upscale(&self, input: &[u8], target: ResolutionLevel) -> Result<Vec<u8>> {
        // Decode input image to tensor
        let img = image::load_from_memory(input)?;
        let tensor = img_to_tensor(&img)?;

        // Run inference
        let output = self.session.run(ort::inputs![tensor]?)?;

        // Decode output tensor to image
        let output_img = tensor_to_img(&output[0])?;

        Ok(output_img)
    }
}
```

---

## 11. Technology Stack

### Core Dependencies

```toml
[workspace]
members = [
    "crates/types",
    "crates/crypto",
    "crates/compression",
    "crates/search",
    "crates/db",
    "crates/portable-db",
    "crates/web",
    "crates/faces",
    "crates/backends",
    "crates/cli",
    "crates/drive-wasm",
    "crates/resolutions",      # NEW
    "crates/erasure",          # NEW
    "crates/preview-keys",     # NEW
    "crates/recovery",         # NEW
    "crates/sync-crdt",        # NEW
]
```

### New Crate Dependencies

```toml
# crates/resolutions/Cargo.toml
[dependencies]
blake3 = "1.5"
redb = "4.1.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# crates/erasure/Cargo.toml
[dependencies]
reed-solomon-simd = "3.1.0"        # RS O(n log n), 10 GiB/s
clay-codes = "0.1.1"               # MSR codes, 2.9× less repair BW
fountain_scheme = "1.0.1"          # LT/RaptorQ fountain codes
shamir-zero = "0.1.10"             # Shamir secret sharing, zero-unsafe
gf256 = "0.3.1"                    # GF(256) toolkit: RS+RAID+Shamir

# crates/preview-keys/Cargo.toml
[dependencies]
rusty_paseto = "0.10.0"            # PASETO V4 tokens
hkdf = "0.13.0"                    # Key derivation
ml-kem = "0.3.2"                   # ML-KEM-1024 (FIPS 203)
ml-dsa = "0.1.1"                   # ML-DSA-65 (FIPS 204)

# crates/recovery/Cargo.toml
[dependencies]
fast_image_resize = "6.0"          # SIMD Lanczos3/Bicubic
ort = { version = "2.0.0-rc.12", features = ["load-dynamic"] }
image = "0.25.10"
ravif = "0.13.0"                   # AVIF encoding
mozjpeg-rs = "0.9.2"               # Pure safe-Rust JPEG
pdfium-render = "0.8"              # PDF rendering
ffmpeg-next = "7"                   # Video thumbnails

# crates/sync-crdt/Cargo.toml
[dependencies]
abyo-crdt = "0.4"                  # Delta-state CRDTs
vclock = "0.4.4"                   # Vector clocks
iroh = "1.0.0"                     # P2P content-addressed networking
iroh-blobs = "1.0.0"               # BLAKE3 verified blob transfer
fastcdc = "4.0.1"                  # Content-defined chunking
```

### Existing Dependencies to Upgrade

```toml
# Upgrade these
lz4_flex = "0.13.1"                # was 0.12
zstd = "0.13.3"                    # was unknown
brotli = "8.0.4"                   # was unknown

# Remove these (archived)
# pqcrypto-mlkem = "0.19.x"       # → ml-kem
# pqcrypto-mlkem768 = "0.19.x"    # → ml-kem
# pqcrypto-ml-dsa = "0.19.x"      # → ml-dsa
# pqcrypto-falcon = "0.4.x"       # → falcon-rs (experimental)
# pqcrypto-sphincsplus = "0.7.x"  # → slh-dsa
```

---

## 12. Implementation Roadmap

### Phase 0: Crypto Migration (Urgent — before June 2026)

**Goal**: Migrate from archived `pqcrypto-*` to pure-Rust `ml-kem`/`ml-dsa`/`slh-dsa`.

| Task | Crate | Effort |
|------|-------|--------|
| Replace `pqcrypto-mlkem` → `ml-kem` 0.3.2 | `cybermanju-crypto` | 2-3 days |
| Replace `pqcrypto-ml-dsa` → `ml-dsa` 0.1.1 | `cybermanju-crypto` | 2-3 days |
| Add `hpke` 0.13.0 for hybrid KEM | `cybermanju-crypto` | 1-2 days |
| Add `rusty_paseto` 0.10.0 for tokens | New crate | 2-3 days |
| Add `frost-*` for threshold key splitting | `cybermanju-crypto` | 3-5 days |
| Add `shamir-zero` for key splitting | `cybermanju-erasure` | 1-2 days |
| Update WASM crypto bindings | `cybermanju-drive-wasm` | 2-3 days |
| Update all tests | All crates | 2-3 days |

**Total**: ~2-3 weeks

### Phase 1: Resolution Tree (Foundation)

**Goal**: Add resolution awareness to the file model.

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-resolutions` crate | New | 1 day |
| Implement `ResolutionTree` struct | `resolutions` | 3-5 days |
| Implement `ResolutionManifest` | `resolutions` | 2-3 days |
| Modify `FileNode` with resolution metadata | `types` | 1-2 days |
| Modify portable DB to v2 format | `portable-db` | 3-5 days |
| Add resolution generation pipeline | `resolutions` | 3-5 days |
| Update Tauri commands | `src-tauri` | 2-3 days |
| Update WASM bridge | `drive-wasm` | 2-3 days |

**Total**: ~3-4 weeks

### Phase 2: Erasure Coding

**Goal**: Distribute r3 and parity across backends with erasure coding.

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-erasure` crate | New | 1 day |
| Integrate `reed-solomon-simd` 3.1.0 | `erasure` | 2-3 days |
| Integrate `clay-codes` 0.1.1 | `erasure` | 3-5 days |
| Integrate `fountain_scheme` 1.0.1 | `erasure` | 3-5 days |
| Implement `DistributionPolicy` | `erasure` | 3-5 days |
| Modify sync pipeline for resolution distribution | `src-tauri/sync` | 5-7 days |
| Add cost-aware placement | `erasure` | 2-3 days |
| Add repair bandwidth optimization | `erasure` | 2-3 days |

**Total**: ~3-4 weeks

### Phase 3: Preview Keys

**Goal**: PASETO V4 view tokens with resolution-limited access.

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-preview-keys` crate | New | 1 day |
| Implement PASETO V4 token generation | `preview-keys` | 3-5 days |
| Implement token validation + revocation | `preview-keys` | 3-5 days |
| Add HKDF key derivation hierarchy | `preview-keys` | 1-2 days |
| Add Merkle tree for revocation list | `preview-keys` | 2-3 days |
| Integrate with Tauri commands | `src-tauri` | 2-3 days |
| Update web dashboard API | `src-tauri/web_dashboard` | 2-3 days |

**Total**: ~2-3 weeks

### Phase 4: Shared Preview Library

**Goal**: Content-addressed, deduplicated preview store with sprite sheets.

| Task | Crate | Effort |
|------|-------|--------|
| Implement content-addressed preview store | `resolutions` | 3-5 days |
| Implement deduplication engine | `resolutions` | 2-3 days |
| Implement sprite sheet generation | `resolutions` | 2-3 days |
| Add `fastcdc` chunking for large previews | `resolutions` | 1-2 days |
| Integrate with sync pipeline | `src-tauri/sync` | 2-3 days |
| Add preview library sync to all backends | `src-tauri/sync` | 2-3 days |

**Total**: ~2-3 weeks

### Phase 5: Recovery Pipeline

**Goal**: Byte-level recovery with neural upscaling.

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-recovery` crate | New | 1 day |
| Integrate `fast_image_resize` 6.0 | `recovery` | 1-2 days |
| Integrate `ort` + Real-ESRGAN ONNX | `recovery` | 3-5 days |
| Add `ffmpeg-next` for video thumbnails | `recovery` | 2-3 days |
| Add `pdfium-render` for PDF previews | `recovery` | 2-3 days |
| Implement recovery gradient logic | `recovery` | 3-5 days |
| Add confidence scoring | `recovery` | 1-2 days |
| Integrate with Tauri commands | `src-tauri` | 2-3 days |

**Total**: ~3-4 weeks

### Phase 6: Distributed Sync CRDT

**Goal**: Delta-state CRDT sync with vector clocks.

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-sync-crdt` crate | New | 1 day |
| Integrate `abyo-crdt` or `saorsa` | `sync-crdt` | 3-5 days |
| Implement delta-state sync protocol | `sync-crdt` | 5-7 days |
| Add vector clock causal ordering | `sync-crdt` | 2-3 days |
| Integrate `iroh` for P2P transport | `sync-crdt` | 5-7 days |
| Add conflict resolution UI | Vue frontend | 3-5 days |
| Test multi-device sync | Testing | 3-5 days |

**Total**: ~4-5 weeks

---

## 13. Design Decisions Log

### Decision 1: Fixed 4 Resolution Levels

**Chosen**: 4 fixed levels (r0-r3) + parity

**Alternatives considered**:
- 2 levels (thumb + full): Too simple, no recovery gradient
- 8 levels: Too much complexity, diminishing returns
- Dynamic levels: Maximum flexibility but hard to reason about

**Rationale**: 4 levels provide a good balance:
- r0 (200x200): Instant gallery access
- r1 (640x480): Share preview
- r2 (1920x1080): Detail view
- r3 (original): Full resolution
- Plus parity for recovery

### Decision 2: Clay Codes for r3, Fountain Codes for Parity

**Chosen**: Clay codes (MSR) for r3 distribution, fountain codes (RaptorQ) for parity

**Alternatives considered**:
- Reed-Solomon for both: Fixed shard count, higher repair bandwidth
- Fountain codes for both: No fixed assignment, but less repair optimization
- Clay codes for both: Fixed shard count, complex implementation

**Rationale**: Clay codes optimize repair bandwidth for the large r3 data.
Fountain codes are rateless, ideal for parity where backend count varies.

### Decision 3: PASETO V4 for View Tokens

**Chosen**: PASETO V4 Local tokens

**Alternatives considered**:
- JWT: Complex, security issues, JWK complexity
- age: File encryption, not token standard
- Custom: Reinventing the wheel

**Rationale**: PASETO is a modern token standard designed to replace JWT.
V4 Local uses AES-256-CTR + BLAKE3-MACT, simpler and more secure than JWT.

### Decision 4: BLAKE3 for All Hashing

**Chosen**: BLAKE3 for Merkle trees, content addressing, and integrity

**Alternatives considered**:
- SHA-256: Slower, NIST standard
- SHA-3: Slower, NIST standard
- Keccak: Same family as SHA-3

**Rationale**: BLAKE3 is 14× faster than SHA-256, has built-in tree hashing
for parallelism, and is used by iroh, Cargo, and many content-addressed systems.

### Decision 5: `redb` for Metadata, `fjall` Optional for Blob Index

**Chosen**: Keep `redb` as primary DB, optionally add `fjall` for blob indexing

**Alternatives considered**:
- Replace `redb` with `fjall`: LSM-tree is better for write-heavy workloads
- Keep only `redb`: Simple, but blob indexing is write-heavy
- Use `iroh` for everything: Too much coupling

**Rationale**: `redb` is proven, ACID, stable format. `fjall` is better for
write-heavy blob indexing but adds complexity. Keep it optional.

---

## 14. Crate Migration Guide

### `pqcrypto-*` → RustCrypto Migration

```rust
// OLD (archived June 2026)
use pqcrypto_kyber::mlkem1024;
use pqcrypto_dilithium::mldsa65;

let keypair = mlkem1024::keypair();
let (ct, ss) = mlkem1024::encapsulate(&keypair.0);
let shared = mlkem1024::decapsulate(&ct, &keypair.1);

// NEW (pure Rust, production-ready)
use ml_kem::{MlKem1024, Encapsulate, Decapsulate};
use ml_dsa::{MlDsa65, SignRandomized};

let mut rng = OsRng;
let keypair = MlKem1024::generate(&mut rng);
let (ct, ss) = keypair.encapsulate(&mut rng)?;
let shared = keypair.decapsulate(&ct)?;

// Signatures
let signing_key = MlDsa65::generate(&mut rng);
let signature = signing_key.sign_randomized(&mut rng, &message)?;
let verified = signing_key.verify(&message, &signature)?;
```

### PASETO V4 Token Example

```rust
use rusty_paseto::prelude::*;

// Generate preview key
let preview_key = PasetoSymmetricKey::<V4, Local>::new();

// Create view token
let token = PasetoBuilder::<V4, Local>::new()
    .set_claim("sub", "file_id_001")?
    .set_claim("res", "r0")?
    .set_claim("vcn", 10u32)?
    .set_claim("exp", Expiration::try_from("24h")?)?
    .set_claim("jti", Uuid::new_v4().to_string())?
    .build(&preview_key)?;

// Validate token
let claims = Paseto::<V4, Local>::parse(&token, &preview_key)?;

// Check expiry, view count, revocation
```

### HPKE Hybrid Key Exchange

```rust
use hpke::{Hpke, DhKem};

// X-Wing: ML-KEM-768 + X25519 (hybrid)
let hpke = Hpke::<DhKem>::new();

// Encapsulate (for encrypting to a public key)
let (enc, shared_secret) = hpke.encapsulate(&recipient_public_key)?;

// The shared_secret is quantum-resistant even if X25519 is broken
let key = hkdf_sha256(shared_secret, "cybermanju-content-v1", file_id);
```

---

## Appendix A: File Size Estimates

### Per-File Storage Overhead

| Resolution | Format | Size | Cumulative |
|-----------|--------|------|------------|
| r0 | WebP q80 | 3KB | 3KB |
| r1 | JPEG q75 | 45KB | 48KB |
| r2 | JPEG q90 | 450KB | 498KB |
| r3 (encrypted) | Triple-layer | 2.5MB | 2.998MB |
| Parity (fountain) | RaptorQ | 830KB | 3.828MB |
| Sprite sheet | PNG | 30KB | 3.858MB |
| Metadata | JSON | 2KB | 3.860MB |
| **Total overhead** | | | **~3.86MB per 8MB file** |
| **Overhead ratio** | | | **48.25% of original** |

### For 1000 Files (8MB average)

| Data | Size |
|------|------|
| r0 thumbnails | 3MB |
| r1 previews | 45MB |
| r2 medium | 450MB |
| r3 originals | 8GB |
| Parity shards | 830MB |
| Sprite sheets | 30MB |
| Metadata | 2MB |
| **Total** | **~9.36GB** |
| **Without EC** | **8GB** |
| **EC overhead** | **~1.36GB (17%)** |

---

## Appendix B: Performance Targets

| Operation | Target | Implementation |
|-----------|--------|---------------|
| Gallery load (100 thumbs) | <100ms | r0 from any backend |
| Share preview | <200ms | r0+r1 with view token |
| Detail view | <500ms | r1+r2 |
| Full download | <5s | r3 reconstruction |
| Recovery (Lanczos) | <100ms | fast_image_resize SIMD |
| Recovery (neural) | <2s | ort + Real-ESRGAN |
| Recovery (erasure) | <10s | clay-codes decode |
| Sync (per file) | <30s | Resolution decomposition |
| Token generation | <10ms | PASETO V4 |
| Token validation | <1ms | PASETO V4 |

---

## Appendix C: Security Considerations

| Threat | Mitigation |
|--------|-----------|
| Quantum computer breaks ML-KEM | HPKE hybrid (ML-KEM + X25519) |
| Master key compromise | Shamir split (3-of-5) across devices |
| View token leak | Time-limited + view-limited + revocable |
| Backend compromise | r3 encrypted, only parity on cheap backends |
| Man-in-the-middle | All transfers over TLS + content verification |
| Data corruption | BLAKE3 Merkle tree integrity verification |
| Ransomware | r0 on ALL backends, parity for reconstruction |
| Insider threat | Multi-user ACL with audit trail |

---

*Document version: 2.0.0*
*Last updated: 2026-06-20*
*Status: Architecture proposal — pending review*
