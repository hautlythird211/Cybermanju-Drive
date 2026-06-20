# Cybermanju Drive — Resolution Tree Architecture

> Machine-readable architecture context for `.cybermanju` v2: a distributed,
> indexed, multi-shard container system with resolution-based file decomposition,
> cross-backend erasure coding, fully encrypted at rest, and byte-level recovery.
> All layers (index, preview, content) are encrypted. Without the index key,
> shards are opaque — indistinguishable from random bytes.

---

## Table of Contents

1. [Core Architecture Model](#1-core-architecture-model)
2. [`.cybermanju` Shard Format](#2-cybermanju-shard-format)
3. [`root.cybermanju` Master Index](#3-rootcybermanju-master-index)
4. [Index Layer: Encrypted But Separate](#4-index-layer-encrypted-but-separate)
5. [Content Layer: Encrypted Blob Store](#5-content-layer-encrypted-blob-store)
6. [Resolution Merkle Tree](#6-resolution-merkle-tree)
7. [Three-Tier Key System](#7-three-tier-key-system)
8. [Cross-Backend Distribution](#8-cross-backend-distribution)
9. [Erasure Coding Strategies](#9-erasure-coding-strategies)
10. [Compression Pipeline](#10-compression-pipeline)
11. [Shared Preview Library](#11-shared-preview-library)
12. [Recovery Pipeline](#12-recovery-pipeline)
13. [Technology Stack](#13-technology-stack)
14. [Implementation Roadmap](#14-implementation-roadmap)
15. [Design Decisions Log](#15-design-decisions-log)
16. [Crate Migration Guide](#16-crate-migration-guide)

---

## 1. Core Architecture Model

### The Shard Model

A Cybermanju Drive library is NOT a single file. It is **hundreds of `.cybermanju`
shard files** distributed across multiple backends, unified by a single
`root.cybermanju` master index.

```
My Library (10,000 files, 50GB)
├── root.cybermanju                    # Master index (1-5MB, knows everything)
├── shard_0001.cybermanju              # Backend: GitHub     (chunk of files)
├── shard_0002.cybermanju              # Backend: GitHub     (chunk of files)
├── shard_0003.cybermanju              # Backend: GitLab     (chunk of files)
├── shard_0004.cybermanju              # Backend: Google Drive (chunk of files)
├── shard_0005.cybermanju              # Backend: Local      (chunk of files)
├── shard_0006.cybermanju              # Backend: MEGA       (chunk of files)
├── ...                                # (hundreds more)
├── parity_shard_A.cybermanju          # Backend: GitHub     (parity for reconstruction)
├── parity_shard_B.cybermanju          # Backend: GitLab     (parity for reconstruction)
└── preview_shard.cybermanju           # Backend: ALL        (ultra-compressed previews)
```

### What Each Shard Contains

Each `.cybermanju` file is a **fully opaque, encrypted container**.
Without the index key, the entire file is indistinguishable from random bytes:

```
.shard.cybermanju  (to an attacker: opaque blob)
├── [HEADER]           Magic + minimal routing metadata (plaintext, 64 bytes)
├── [INDEX LAYER]      File manifest, blob map, resolution map — ENCRYPTED
├── [CONTENT LAYER]    Encrypted blobs: r0, r1, r2, r3, parity — ENCRYPTED
└── [FOOTER]           BLAKE3 checksums, signature — ENCRYPTED
```

**Security model**: ALL layers inside `.cybermanju` are encrypted.
The only plaintext information is:
- Magic bytes (file identification, 32 bytes)
- Shard ID (routing, 16 bytes)
- Total size (for allocation, 8 bytes)
- Root hash backlink (for verification, 32 bytes)

Total plaintext: **~88 bytes** out of potentially gigabytes.
An attacker sees: random bytes with a header. Cannot parse index, cannot
extract previews, cannot read file names, cannot reconstruct content.

### Recovery Model

Each shard contains **minimized recovery bytes**. Any sufficient subset of
shards can reconstruct the entire library — but ONLY with the proper keys:

```
Recovery threshold:
├── Index reconstruction:  root.cybermanju + index_key + any 1 shard
├── File reconstruction:   root.cybermanju + index_key + content_key + k shards
├── Preview reconstruction: shard + index_key + preview_key
└── Full library:          root.cybermanju + index_key + content_key + enough shards
```

Without keys, an attacker cannot:
- Identify which shard contains which file (index encrypted)
- Extract previews from any shard (preview blobs encrypted)
- Reconstruct original files (content blobs encrypted)
- Determine file names, sizes, or structure (index encrypted)

### Access Patterns

| Action | Keys Required | What App Does |
|--------|--------------|---------------|
| Browse file list | index_key | Decrypts shard index, parses manifest |
| See file metadata | index_key | Reads from decrypted index |
| View thumbnail (r0) | index_key + preview_key | Decrypts index → finds r0 offset → decrypts r0 blob |
| View preview (r1) | index_key + preview_key | Decrypts index → finds r1 offset → decrypts r1 blob |
| Stream video (r2) | index_key + content_key | Decrypts index → finds r2 offset → decrypts chunks |
| Download original (r3) | index_key + content_key | Decrypts index → finds r3 offset → decrypts all chunks |
| Recover lost shard | index_key + content_key + k shards | Reconstructs from erasure coding |

### What an Attacker Sees

```
Without ANY key:
├── Shard file on GitHub/MEGA/etc.
├── Reads: 88 bytes of header (magic, shard_id, size, root_hash)
├── Rest: encrypted bytes (looks like random data)
├── Cannot: parse index, find files, extract previews, read names
├── Cannot: determine what type of files are stored
└── Verdict: OPAQUE — useless without index_key

With index_key only:
├── Can: parse shard index, see file names, metadata, structure
├── Cannot: decrypt preview blobs (need preview_key)
├── Cannot: decrypt content blobs (need content_key)
├── Can: see file names which may be sensitive ("passport_scan.pdf")
└── Verdict: METADATA EXPOSED — but content safe

With index_key + preview_key:
├── Can: see file names, metadata, thumbnails, previews
├── Cannot: decrypt r2/r3 content (need content_key)
├── Can: view 640x480 previews of photos
├── Cannot: download originals
└── Verdict: VISUAL CONTENT EXPOSED — but originals safe

With index_key + content_key:
├── Can: see everything, download everything
├── Can: reconstruct from erasure coding
├── Verdict: FULL COMPROMISE
```

---

## 2. `.cybermanju` Shard Format

### Binary Layout

```
Offset    Size      Field                           Encryption
──────────────────────────────────────────────────────────────
[0..32)   32B       Magic: "CYBSHARD_V2..."         PLAINTEXT
[32..36)  4B        header_len (u32 LE)             PLAINTEXT
[36..+h)  variable  header_json (ShardHeader)       PLAINTEXT (minimal)
[h..+i)   4B        encrypted_index_len (u32 LE)    PLAINTEXT
[+i..+j)  variable  encrypted_index_blob            ENCRYPTED (index_key)
[j..+c)   4B        encrypted_content_map_len       PLAINTEXT
[+c..+e)  variable  encrypted_content_map_blob      ENCRYPTED (index_key)
[e..+f)   4B        encrypted_erasure_len           PLAINTEXT
[+f..+g)  variable  encrypted_erasure_blob          ENCRYPTED (index_key)
[g..+s)   4B        signature_len (u32 LE)          PLAINTEXT
[+s..+k)  variable  shard_signature                 PLAINTEXT (ML-DSA-65)
[k..)     variable  content_blobs                   ENCRYPTED (content/preview keys)

Total plaintext: ~88 bytes (magic + lengths + signature)
Everything else: encrypted at rest
```

### ShardHeader

```json
{
  "magic": "CYBSHARD_V2",
  "version": "2.0",
  "shard_id": "shard_0042",
  "root_hash_backlink": "blake3:root_cybermanju_hash...",
  "created_at": "2026-06-20T00:00:00Z",
  "modified_at": "2026-06-20T12:00:00Z",
  "app_version": "0.2.0",
  "shard_type": "content",
  "encrypted_index_len": 15360,
  "encrypted_content_map_len": 4096,
  "encrypted_erasure_len": 2048,
  "content_algorithm": "ml-kem-1024+chacha20poly1305",
  "index_algorithm": "aes-256-gcm",
  "compression": "lz4+zstd15+brotli11",
  "erasure_codec": "clay-codes",
  "erasure_params": { "k": 3, "m": 1, "d": 4 },
  "platform_origin": "linux"
}
```

**Note**: Header is intentionally minimal — only routing/identification data.
No file names, no sizes, no metadata. An attacker cannot determine what's
inside from the header alone. Total plaintext per shard: ~88 bytes.

### ShardIndex (Encrypted — Needs index_key)

This section is ENCRYPTED with the index_key. Without it, this is
indistinguishable from random bytes. The app decrypts it on load.

```json
{
  "shard_id": "shard_0042",
  "files": {
    "file_abc123": {
      "name": "vacation_photo.jpg",
      "mime": "image/jpeg",
      "folder": "/photos/2026/italy",
      "tags": ["vacation", "italy", "beach"],
      "original_size": 8388608,
      "original_blake3": "blake3:def456...",
      "created_at": "2026-06-15T10:30:00Z",
      "modified_at": "2026-06-15T10:30:00Z",
      "gps": { "lat": 41.9028, "lon": 12.4964 },
      "face_groups": ["face_alice", "face_bob"],
      "versions": 3,
      "current_version": 3,
      "resolutions": {
        "r0": {
          "blake3": "blake3:thumb...",
          "size": 3072,
          "format": "webp",
          "width": 200,
          "height": 150,
          "content_offset": 1024,
          "content_length": 3072,
          "encrypted": true,
          "encryption_key_tier": "preview"
        },
        "r1": {
          "blake3": "blake3:preview...",
          "size": 46080,
          "format": "jpeg",
          "width": 640,
          "height": 480,
          "content_offset": 4096,
          "content_length": 46080,
          "encrypted": true,
          "encryption_key_tier": "preview"
        },
        "r2": {
          "blake3": "blake3:medium...",
          "size": 460800,
          "format": "jpeg",
          "width": 1920,
          "height": 1080,
          "content_offset": 50176,
          "content_length": 460800,
          "encrypted": true,
          "encryption_key_tier": "content"
        },
        "r3": {
          "blake3": "blake3:original...",
          "size": 8388608,
          "format": "encrypted",
          "content_offset": 510976,
          "content_length": 8388608,
          "encrypted": true,
          "encryption_key_tier": "content",
          "chunk_count": 8,
          "chunk_size": 1048576
        }
      },
      "parity": {
        "codec": "clay-codes",
        "shard_indices": [0, 1, 2],
        "parity_indices": [3],
        "parity_in_shards": ["shard_0043", "shard_0044"]
      }
    },
    "file_def789": {
      "name": "beach_sunset.mp4",
      "mime": "video/mp4",
      "folder": "/videos/2026/italy",
      "original_size": 157286400,
      "resolutions": {
        "r0": { "content_offset": 8894464, "content_length": 4096, "encrypted": true, "encryption_key_tier": "preview" },
        "r1": { "content_offset": 8898560, "content_length": 61440, "encrypted": true, "encryption_key_tier": "preview" },
        "r2": { "content_offset": 8960000, "content_length": 3145728, "encrypted": true, "encryption_key_tier": "content" },
        "r3": { "content_offset": 12105728, "content_length": 157286400, "encrypted": true, "encryption_key_tier": "content", "chunk_count": 150, "chunk_size": 1048576 }
      }
    }
  },
  "sprite_sheets": {
    "batch_0": { "content_offset": 1701920768, "content_length": 30720, "grid": "4x4", "thumb_count": 16 }
  },
  "erasure_map": {
    "clay_shards": [
      { "shard_index": 0, "content_offset": 1701951488, "content_length": 3355443 },
      { "shard_index": 1, "content_offset": 1705305931, "content_length": 3355443 }
    ],
    "fountain_packets": [
      { "packet_id": 0, "content_offset": 1708660374, "content_length": 1040 },
      { "packet_id": 1, "content_offset": 1708661414, "content_length": 1040 }
    ]
  },
  "merkle_root": "blake3:shard_merkle_root..."
}
```

### ContentMap (Byte-Range Access)

The ContentMap tells you exactly where each blob lives in the content section:

```json
{
  "blob_regions": [
    { "id": "r0_file_abc123", "offset": 1024, "length": 3072, "key_tier": "preview", "compression": "webp-lossy" },
    { "id": "r1_file_abc123", "offset": 4096, "length": 46080, "key_tier": "preview", "compression": "mozjpeg-q75" },
    { "id": "r2_file_abc123", "offset": 50176, "length": 460800, "key_tier": "content", "compression": "mozjpeg-q90" },
    { "id": "r3_file_abc123_chunk_0", "offset": 510976, "length": 1048576, "key_tier": "content", "compression": "lz4+zstd15+brotli11" },
    { "id": "r3_file_abc123_chunk_1", "offset": 1559552, "length": 1048576, "key_tier": "content", "compression": "lz4+zstd15+brotli11" },
    { "id": "r0_file_def789", "offset": 8894464, "length": 4096, "key_tier": "preview", "compression": "webp-lossy" }
  ],
  "content_total_bytes": 1712000000,
  "compression_ratio": 0.62,
  "encrypted_ratio": 1.0
}
```

### ErasureMeta (Recovery Info)

```json
{
  "shard_id": "shard_0042",
  "erasure_codec": "clay-codes",
  "erasure_params": { "k": 3, "m": 1, "d": 4 },
  "this_shard_role": "data_shard_0",
  "parity_distributed_to": ["shard_0043", "shard_0044", "shard_0045"],
  "recovery_threshold": {
    "data_shards_needed": 3,
    "total_shards_available": 6,
    "can_recover_with": ["shard_0043", "shard_0044", "shard_0045"]
  },
  "fountain_config": {
    "symbol_size": 1024,
    "source_symbols": 8192,
    "repair_symbols_per_shard": 2048,
    "min_packets_for_recovery": 8192
  },
  "shard_blake3": "blake3:entire_shard_content_hash..."
}
```

---

## 3. `root.cybermanju` Master Index

The root file is the **single source of truth** that knows about every shard,
every file, every resolution, every backend. It is small (1-5MB for 10K files)
and can be replicated everywhere.

**Security**: The root file is also encrypted. Only the magic bytes, version,
and library_id are plaintext. Everything else (library name, file counts,
shard distribution, file manifest) is encrypted with the index_key.

### Binary Layout

```
Offset    Size      Field                           Encryption
──────────────────────────────────────────────────────────────
[0..32)   32B       Magic: "CYBROOT__V2..."         PLAINTEXT
[32..36)  4B        header_len (u32 LE)             PLAINTEXT
[36..+h)  variable  header_json (RootHeader)        PLAINTEXT (minimal)
[h..+i)   4B        encrypted_shard_index_len      PLAINTEXT
[+i..+j)  variable  encrypted_shard_index_blob     ENCRYPTED (index_key)
[j..+k)   4B        encrypted_file_manifest_len    PLAINTEXT
[+k..+m)  variable  encrypted_file_manifest_blob   ENCRYPTED (index_key)
[m..+n)   4B        encrypted_distribution_len     PLAINTEXT
[+n..+p)  variable  encrypted_distribution_blob    ENCRYPTED (index_key)
[p..+q)   4B        revocation_merkle_root_len     PLAINTEXT
[q..+r)   32B       revocation_merkle_root         PLAINTEXT (for verification)
[r..+s)   4B        signature_len (u32 LE)         PLAINTEXT
[s..+t)   variable  root_signature                 PLAINTEXT (ML-DSA-65)

Total plaintext: ~128 bytes (magic + lengths + revocation root + signature)
Shard index, file manifest, distribution: ALL ENCRYPTED
```

### RootHeader

```json
{
  "magic": "CYBROOT__V2",
  "version": "2.0",
  "library_id": "lib_001",
  "encrypted_payload_len": 1048576,
  "signature_len": 3200
}
```

**Note**: Header is intentionally minimal — only identification and routing data.
Library name, file counts, shard distribution, encryption keys — all encrypted.
An attacker cannot determine library contents from the header.

### RootPayload (Encrypted — Needs index_key)

```json
{
  "library_name": "My Photo Library",
  "created_at": "2026-01-01T00:00:00Z",
  "modified_at": "2026-06-20T12:00:00Z",
  "app_version": "0.2.0",
  "total_files": 12345,
  "total_shards": 156,
  "total_size_bytes": 53687091200,
  "total_preview_size": 15728640,
  "total_parity_size": 8933376000,
  "shard_distribution": {
    "github": 42,
    "gitlab": 18,
    "google_drive": 35,
    "local": 28,
    "mega": 33
  },
  "erasure_codec": "clay-codes",
  "erasure_params": { "k": 3, "m": 1, "d": 4 },
  "encryption": {
    "algorithm": "ml-kem-1024+chacha20poly1305",
    "index_key_id": "index-key-001",
    "content_key_id": "content-key-001",
    "preview_key_id": "preview-key-001"
  },
  "sync": {
    "crdt": "delta-state",
    "vector_clock": true,
    "last_sync_hash": "blake3:..."
  },
  "revocation_merkle_root": "blake3:revocation_tree_root..."
}
```

### ShardIndex (Encrypted — Needs index_key)

This section is ENCRYPTED with the index_key. Without it, this is
indistinguishable from random bytes.

```json
{
  "shards": {
    "shard_0001": {
      "shard_type": "content",
      "backend": "github",
      "remote_path": "cybermanju/shards/shard_0001.cybermanju",
      "remote_url": "https://api.github.com/repos/.../contents/...",
      "file_count": 47,
      "size_bytes": 268435456,
      "blake3": "blake3:shard_hash...",
      "shard_signature": "ml_dsa65:...",
      "last_verified": "2026-06-20T12:00:00Z"
    },
    "shard_0002": {
      "shard_type": "content",
      "backend": "github",
      "remote_path": "cybermanju/shards/shard_0002.cybermanju",
      "file_count": 52,
      "size_bytes": 301989888,
      "blake3": "blake3:shard_hash...",
      "shard_signature": "ml_dsa65:..."
    },
    "parity_shard_A": {
      "shard_type": "parity",
      "backend": "gitlab",
      "remote_path": "cybermanju/parity/parity_shard_A.cybermanju",
      "file_count": 0,
      "size_bytes": 893337600,
      "blake3": "blake3:parity_hash...",
      "covers_shards": ["shard_0001", "shard_0002", "shard_0003"]
    },
    "preview_shard": {
      "shard_type": "preview",
      "backend": "ALL",
      "remote_path": "cybermanju/preview/preview_shard.cybermanju",
      "file_count": 12345,
      "size_bytes": 15728640,
      "blake3": "blake3:preview_hash..."
    }
  },
  "shard_merkle_root": "blake3:root_of_all_shard_hashes"
}
```

### FileManifest (Encrypted — Needs index_key)

```json
{
  "files": {
    "file_abc123": {
      "name": "vacation_photo.jpg",
      "mime": "image/jpeg",
      "folder": "/photos/2026/italy",
      "original_size": 8388608,
      "original_blake3": "blake3:def456...",
      "created_at": "2026-06-15T10:30:00Z",
      "tags": ["vacation", "italy"],
      "face_groups": ["face_alice"],
      "shard_assignments": {
        "r0": ["shard_0001", "shard_0002", "parity_shard_A"],
        "r1": ["shard_0001", "shard_0002"],
        "r2": ["shard_0001"],
        "r3": ["shard_0001", "shard_0003", "parity_shard_A"]
      },
      "merkle_root": "blake3:file_merkle_root..."
    },
    "file_def789": {
      "name": "beach_sunset.mp4",
      "mime": "video/mp4",
      "folder": "/videos/2026/italy",
      "original_size": 157286400,
      "shard_assignments": {
        "r0": ["shard_0002", "parity_shard_A"],
        "r1": ["shard_0002", "shard_0004"],
        "r2": ["shard_0002"],
        "r3": ["shard_0002", "shard_0005", "parity_shard_A"]
      }
    }
  },
  "folders": {
    "/photos/2026/italy": { "file_ids": ["file_abc123"], "subfolders": [] },
    "/videos/2026/italy": { "file_ids": ["file_def789"], "subfolders": [] }
  },
  "tags_index": {
    "vacation": ["file_abc123"],
    "italy": ["file_abc123", "file_def789"]
  },
  "face_index": {
    "face_alice": ["file_abc123"]
  }
}
```

### DistributionPolicy (Encrypted — Needs index_key)

```json
{
  "resolution_distribution": {
    "r0": {
      "backends": ["ALL"],
      "redundancy": "max",
      "erasure": "none",
      "priority": "instant"
    },
    "r1": {
      "backends": ["github", "google_drive", "local"],
      "redundancy": 2,
      "erasure": "none",
      "priority": "fast"
    },
    "r2": {
      "backends": ["local", "mega"],
      "redundancy": 1,
      "erasure": "optional",
      "priority": "normal"
    },
    "r3": {
      "backends": ["local", "mega", "github"],
      "redundancy": 0,
      "erasure": {
        "codec": "clay-codes",
        "params": { "k": 3, "m": 1, "d": 4 }
      },
      "priority": "background"
    },
    "parity": {
      "backends": ["github", "gitlab"],
      "redundancy": 0,
      "erasure": {
        "codec": "fountain-raptorq",
        "params": { "symbol_size": 1024, "redundancy": 0.33 }
      },
      "priority": "background"
    }
  },
  "shard_size_target_bytes": 268435456,
  "shard_split_strategy": "by_folder",
  "cost_model": {
    "storage_cost_per_gb_month": {
      "github": 0.0,
      "gitlab": 0.0,
      "google_drive": 0.02,
      "local": 0.0,
      "mega": 0.005
    }
  }
}
```

---

## 4. Index Layer: Encrypted But Separate

The index layer is encrypted with the index_key, separate from the content
layer. This provides **key separation** — compromising one key doesn't
compromise everything. But the shard is still opaque without the index_key.

### How It Works

```
┌─────────────────────────────────────────────────────┐
│  .cybermanju SHARD                                   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  HEADER (88 bytes, plaintext)                 │   │
│  │                                               │   │
│  │  Magic, shard_id, size, root_hash_backlink    │   │
│  │  (minimal — no file names, no metadata)       │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  INDEX LAYER (encrypted with INDEX KEY)        │   │
│  │                                               │   │
│  │  • File names, sizes, dates, folders          │   │
│  │  • Tags, face groups, GPS coordinates         │   │
│  │  • Thumbnail byte ranges + key tier           │   │
│  │  • Resolution metadata (width, height, mime)  │   │
│  │  • Erasure coding map                         │   │
│  │  • Shard cross-references                     │   │
│  │                                               │   │
│  │  Encrypted with: INDEX KEY (AES-256-GCM)      │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  CONTENT LAYER (encrypted with separate keys) │   │
│  │                                               │   │
│  │  • r0 thumbnails: encrypted with PREVIEW KEY  │   │
│  │  • r1 previews: encrypted with PREVIEW KEY    │   │
│  │  • r2 medium: encrypted with CONTENT KEY      │   │
│  │  • r3 original: encrypted with CONTENT KEY    │   │
│  │  • Parity shards: encrypted with CONTENT KEY  │   │
│  │                                               │   │
│  │  Each chunk independently decryptable          │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  FOOTER (encrypted with INDEX KEY)            │   │
│  │                                               │   │
│  │  BLAKE3 checksums, erasure metadata           │   │
│  │  ML-DSA-65 signature (plaintext for verify)   │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Index Encryption (Always Encrypted)

The index is ALWAYS encrypted. There is no plaintext option.
The index_key is separate from the content_key and preview_key.

```rust
struct IndexEncryption {
    key_id: String,
    algorithm: String,  // "aes-256-gcm"
}
```

**Why always encrypted**: File names alone are sensitive. "passport_scan.pdf",
"divorce_paperwork.docx", "medical_record.pdf" — these reveal everything
without seeing the file content. The index must be opaque to attackers.

### Browse Flow (Requires index_key)

```rust
fn browse_shard(shard_path: &Path, index_key: &Key) -> ShardIndex {
    // 1. Read shard header (always plaintext, 88 bytes)
    let header = read_shard_header(shard_path);

    // 2. Read encrypted index blob
    let encrypted_index = read_encrypted_index(shard_path, &header)?;

    // 3. Decrypt index with INDEX KEY (not content key, not preview key)
    let index = decrypt_index(&encrypted_index, index_key)?;

    index
}

fn browse_all_files(
    root: &RootCybermanju,
    index_key: &Key,
) -> Vec<FileManifest> {
    let mut all_files = Vec::new();

    // 1. Read root index (small, maybe cached)
    let root_index = read_root_index(root, index_key)?;

    // 2. For each shard, decrypt its index
    for shard_info in root_index.shards.values() {
        let shard_index = browse_shard(&shard_info.local_path, index_key)?;
        all_files.extend(shard_index.files.values().cloned());
    }

    all_files
}
```

### Preview Flow (Requires index_key + preview_key)

```rust
fn preview_file(
    root: &RootCybermanju,
    file_id: &str,
    index_key: &Key,
    preview_key: &Key,
    target_resolution: ResolutionLevel,
) -> Result<Vec<u8>> {
    // 1. Find which shard contains this file's preview
    let file_entry = root.file_manifest.get(file_id)?;
    let shard_id = &file_entry.shard_assignments[&target_resolution][0];
    let shard_info = root.shard_index.get(shard_id)?;

    // 2. Decrypt shard index (requires index_key)
    let shard_index = browse_shard(&shard_info.local_path, index_key)?;

    // 3. Find the byte range for this resolution
    let resolution = shard_index.files[file_id].resolutions[&target_resolution];
    let content_offset = resolution.content_offset;
    let content_length = resolution.content_length;

    // 4. Read encrypted bytes from shard
    let encrypted_bytes = read_content_bytes(
        &shard_info.local_path,
        content_offset,
        content_length,
    )?;

    // 5. Decrypt with PREVIEW KEY (not content key!)
    let decrypted = decrypt_preview(&encrypted_bytes, preview_key, file_id)?;

    Ok(decrypted)
}
```

### Search Flow (Requires index_key)

```rust
fn search_files(
    root: &RootCybermanju,
    query: &str,
    index_key: &Key,
) -> Vec<FileManifest> {
    let mut results = Vec::new();

    // 1. Decrypt root index
    let root_index = read_root_index(root, index_key)?;

    // 2. Search root index (tags, folders, names)
    for (file_id, file) in root_index.file_manifest.files.iter() {
        if file.name.contains(query)
            || file.tags.iter().any(|t| t.contains(query))
            || file.folder.contains(query)
        {
            results.push(file.clone());
        }
    }

    // 3. Search shard-level indexes for deeper metadata
    for shard_info in root_index.shard_index.shards.values() {
        let shard_index = browse_shard(&shard_info.local_path, index_key)?;
        for (file_id, file) in shard_index.files.iter() {
            if file.face_groups.iter().any(|f| f.contains(query)) {
                results.push(file.clone());
            }
        }
    }

    results
}
```

---

## 5. Content Layer: Encrypted Blob Store

### Chunk-Level Encryption

Each content blob is independently encrypted. This enables:
- Read r0 without touching r3
- Stream r2 without decrypting entire file
- Seek to any chunk in r3

```rust
struct ChunkEncryption {
    /// Each chunk gets its own nonce (96-bit for ChaCha20-Poly1305)
    chunk_index: u32,
    nonce: [u8; 12],
    /// Key tier determines which key encrypts this chunk
    key_tier: KeyTier,
    /// Auth tag (16 bytes for Poly1305)
    auth_tag: [u8; 16],
}

enum KeyTier {
    /// Preview key — encrypts r0, r1 thumbnails/previews
    Preview,
    /// Content key — encrypts r2, r3, parity
    Content,
}
```

### Chunked r3 for Streaming

Large files (r3) are split into independently encrypted chunks:

```
r3 original (8MB file, 1MB chunks)
├── chunk_0: [encrypted: 1MB + 16B auth tag]  → ChaCha20(key=content_key, nonce=chunk_0_nonce)
├── chunk_1: [encrypted: 1MB + 16B auth tag]  → ChaCha20(key=content_key, nonce=chunk_1_nonce)
├── chunk_2: [encrypted: 1MB + 16B auth tag]  → ChaCha20(key=content_key, nonce=chunk_2_nonce)
├── ...
└── chunk_7: [encrypted: 1MB + 16B auth tag]  → ChaCha20(key=content_key, nonce=chunk_7_nonce)
```

### Streaming Without Full Decrypt

```rust
fn stream_video_chunk(
    shard_path: &Path,
    file_id: &str,
    chunk_index: u32,
    content_key: &Key,
) -> Result<Vec<u8>> {
    // 1. Read shard index (no key needed)
    let shard_index = browse_shard(shard_path, None)?;

    // 2. Find chunk byte range
    let resolution = shard_index.files[file_id].resolutions[&ResolutionLevel::R2];
    let chunk_offset = resolution.content_offset + (chunk_index as u64 * resolution.chunk_size);
    let chunk_length = resolution.chunk_size;

    // 3. Read only this chunk's bytes
    let encrypted_chunk = read_content_bytes(shard_path, chunk_offset, chunk_length + 16)?;

    // 4. Decrypt only this chunk
    let nonce = compute_chunk_nonce(file_id, chunk_index);
    let plaintext = chacha20poly1305_decrypt(&encrypted_chunk, content_key, &nonce)?;

    Ok(plaintext)
}
```

### Compression Before Encryption

Each blob is compressed BEFORE encryption, so the encrypted output is opaque
but the plaintext is compressed:

```
Original file → Compress (triple-layer) → Encrypt (chunk-level) → Store in shard

Reading:
Shard → Read byte range → Decrypt chunk → Decompress → Original file
```

### Random-Access Decompression

For r3 with chunk-level compression, you can seek to any chunk:

```rust
fn read_r3_chunk(
    shard_path: &Path,
    file_id: &str,
    chunk_index: u32,
    content_key: &Key,
) -> Result<Vec<u8>> {
    // 1. Read encrypted chunk
    let encrypted = read_content_bytes(shard_path, chunk_offset, chunk_len + 16)?;

    // 2. Decrypt
    let compressed = chacha20poly1305_decrypt(&encrypted, content_key, &nonce)?;

    // 3. Decompress (only this chunk)
    let decompressed = triple_decompress(&compressed)?;

    Ok(decompressed)
}
```

---

## 6. Resolution Merkle Tree

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

## 7. Three-Tier Key System

### Key Hierarchy

```
Master Key (user-controlled, permanent, 256-bit)
├── Derives → Index Key      (encrypts shard indexes)
├── Derives → Content Key    (encrypts r2, r3 — full resolution data)
├── Derives → Preview Key    (encrypts r0, r1 — thumbnails/previews)
└── Derives → View Token Key (time-limited, view-limited, encrypts ONLY r0)
```

### Key Derivation

```rust
use hkdf::Hkdf;
use sha2::Sha256;

fn derive_index_key(master_key: &[u8], library_id: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(
        Some(b"cybermanju-index-v1"),
        master_key
    );
    let mut key = [0u8; 32];
    hk.expand(library_id.as_bytes(), &mut key).unwrap();
    key
}

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
    hk.expand(file_id.as_bytes(), &mut key).unwrap();
    key
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
  5. Store token metadata in root index (for revocation)
  6. Return encrypted r0 + token

Validate Token:
  1. Parse PASETO V4 token
  2. Check expiry (exp claim)
  3. Check revocation list (jti claim against revocation Merkle tree)
  4. Derive token_key = HKDF(preview_key, file_id, token_id)
  5. Decrypt r0 data with token_key
  6. Increment view count (if < vcn, else revoke)

Revoke Token:
  1. Add jti to revocation Merkle tree
  2. Update revocation root in root.cybermanju header
  3. Token becomes invalid on next validation
```

### Access Control Matrix

| Layer | Master Key | Index Key | Content Key | Preview Key | View Token |
|-------|-----------|-----------|-------------|-------------|------------|
| Shard header | Read-only | Read-only | Read-only | Read-only | Read-only |
| Index (metadata) | Full access | Full access | No access | No access | No access |
| r0 (thumb) | Full access | Full access | Full access | Full access | Time-limited, view-limited |
| r1 (preview) | Full access | Full access | Full access | Full access | No access |
| r2 (medium) | Full access | Full access | Full access | No access | No access |
| r3 (original) | Full access | Full access | Full access | No access | No access |
| Parity shards | Full access | Full access | Full access | No access | No access |

**Key insight**: Without index_key, you can't even find where r0/r1/r2/r3
are stored inside the shard. The index is the map — without it, the shard
is just encrypted bytes with no structure.

---

## 8. Cross-Backend Distribution

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
    Specific(String),
    Matching(fn(&dyn StorageBackend) -> bool),
    Random { pool: Vec<String>, count: usize },
    RoundRobin { pool: Vec<String> },
}
```

### Default Distribution Policy

```yaml
resolution_distribution:
  r0:
    backends: [ALL]
    redundancy: max
    erasure: none
    priority: instant

  r1:
    backends: [github, google_drive, local]
    redundancy: 2
    erasure: none
    priority: fast

  r2:
    backends: [local, mega]
    redundancy: 1
    erasure: optional
    priority: normal

  r3:
    backends: [local, mega, github]
    redundancy: 0
    erasure:
      codec: clay-codes
      params: { k: 3, m: 1, d: 4 }
    priority: background

  parity:
    backends: [github, gitlab]
    redundancy: 0
    erasure:
      codec: fountain-raptorq
      params: { symbol_size: 1024, redundancy: 0.33 }
    priority: background
```

### Cost-Aware Placement

```rust
struct CostModel {
    storage_cost: HashMap<String, f64>,
    download_cost: HashMap<String, f64>,
    upload_cost: HashMap<String, f64>,
}

impl CostModel {
    fn optimize_placement(
        &self,
        file: &FileResolutions,
        policy: &DistributionPolicy,
    ) -> PlacementPlan {
        // r3 (large): cheapest backends with sufficient bandwidth
        // r0 (tiny): ignore cost, maximize availability
        // parity: cheapest backends with sufficient storage
    }
}
```

---

## 9. Erasure Coding Strategies

### Tier 1: Reed-Solomon (Proven, Fast)

**Use case**: r3 distribution when backend count is fixed and known.

```toml
[dependencies]
reed-solomon-simd = "3.1.0"
```

```rust
use reed_solomon_simd::ReedSolomon;

let rs = ReedSolomon::new(3, 1)?;
let shards = rs.encode(&data)?;
let recovered = rs.reconstruct(&available_shards)?;
```

**Performance**: 10.2 GiB/s encode, 1.0 GiB/s decode (Ryzen 5 3600).

### Tier 2: Fountain Codes (Rateless)

**Use case**: Parity distribution where backend count is variable.

```toml
[dependencies]
fountain_scheme = "1.0.1"
```

```rust
use fountain_scheme::{Encoder, Decoder, RaptorQ};

let encoder = RaptorQ::new(symbol_size: 1024, source_symbols: &data)?;
let packet = encoder.encode(sequence_number);

let mut decoder = Decoder::new(symbol_size: 1024, source_len: data.len());
for packet in received_packets {
    decoder.add_packet(packet)?;
}
let recovered = decoder.decode()?;
```

### Tier 3: MSR Codes (Minimum Storage Regenerating)

**Use case**: r3 distribution where repair bandwidth matters.

```toml
[dependencies]
clay-codes = "0.1.1"
```

```rust
use clay_codes::{ClayCodec, Shard};

let codec = ClayCodec::new(k: 3, m: 1, d: 4)?;
let shards = codec.encode(&data)?;
let repaired = codec.repair(lost_shard: 2, available: &shards)?;
```

### Tier 4: Shamir Secret Sharing (Key Splitting)

**Use case**: Splitting master key across devices.

```toml
[dependencies]
shamir-zero = "0.1.10"
```

```rust
use shamir_zero::Shamir;

let shares = Shamir::split(&master_key, threshold: 3, shares: 5)?;
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

## 10. Compression Pipeline

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

let mut cdc = FastCDC::new(data, min_size: 2048, max_size: 65536, avg_size: 32768);

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

let deduped: HashMap<blake3::Hash, Vec<u8>> = chunks.into_iter().collect();
```

---

## 11. Shared Preview Library

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
        let mut sheet = ImageBuffer::new(800, 800);

        for (i, thumb) in thumbnails.iter().enumerate() {
            let x = (i % 4) * 204;
            let y = (i / 4) * 204;
            let img = image::load_from_memory(thumb).unwrap();
            sheet.copy_from(&img, x, y).unwrap();
        }

        let mut buf = Cursor::new(Vec::new());
        sheet.write_to(&mut buf, ImageFormat::Png).unwrap();
        buf.into_inner()
    }
}
```

---

## 12. Recovery Pipeline

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
    ErasureReconstruct {
        shards: Vec<Shard>,
        codec: ClayCodes,
    },
    FountainReconstruct {
        packets: Vec<FountainPacket>,
        codec: RaptorQ,
    },
    UpscaleFromLower {
        source: ResolutionLevel,
        target: ResolutionLevel,
        upscaler: UpscalerEngine,
    },
    FastLanczos {
        source: ResolutionLevel,
        target: ResolutionLevel,
        resize: FastImageResize,
    },
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
    LanczosClassic,                     // fast_image_resize 6.0
    OnnxRealEsrGan {
        model_path: PathBuf,
        execution_provider: GpuProvider,
    },
    BurnEsrGan {
        model: BurnModel,
    },
    FourierRecover,
}
```

### Recovery Pipeline

```rust
impl RecoveryPipeline {
    async fn recover(
        &self,
        file_id: &str,
        target: ResolutionLevel,
        available: &AvailableData,
    ) -> Result<RecoveryResult> {
        if let Some(data) = self.try_exact_recovery(file_id, target, available) {
            return Ok(RecoveryResult::Exact(data));
        }

        if available.has_enough_shards(target) {
            let data = self.erasure_reconstruct(available)?;
            return Ok(RecoveryResult::Exact(data));
        }

        if available.has_enough_packets() {
            let data = self.fountain_reconstruct(available)?;
            return Ok(RecoveryResult::Exact(data));
        }

        if let Some(lower) = target.lower_resolution() {
            if let Some(lower_data) = self.load_resolution(file_id, lower) {
                let upscaled = self.upscaler.upscale(&lower_data, target)?;
                let confidence = self.estimate_confidence(lower, target);
                return Ok(RecoveryResult::Approximate(upscaled, confidence));
            }
        }

        if available.has_partial_data() {
            let data = self.hybrid_reconstruct(available, target)?;
            let confidence = self.estimate_hybrid_confidence(available, target);
            return Ok(RecoveryResult::Approximate(data, confidence));
        }

        Err(RecoveryError::InsufficientData {
            file_id: file_id.to_string(),
            target,
            available: available.summary(),
        })
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
        let img = image::load_from_memory(input)?;
        let tensor = img_to_tensor(&img)?;
        let output = self.session.run(ort::inputs![tensor]?)?;
        let output_img = tensor_to_img(&output[0])?;
        Ok(output_img)
    }
}
```

---

## 13. Technology Stack

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
reed-solomon-simd = "3.1.0"
clay-codes = "0.1.1"
fountain_scheme = "1.0.1"
shamir-zero = "0.1.10"
gf256 = "0.3.1"

# crates/preview-keys/Cargo.toml
[dependencies]
rusty_paseto = "0.10.0"
hkdf = "0.13.0"
ml-kem = "0.3.2"
ml-dsa = "0.1.1"

# crates/recovery/Cargo.toml
[dependencies]
fast_image_resize = "6.0"
ort = { version = "2.0.0-rc.12", features = ["load-dynamic"] }
image = "0.25.10"
ravif = "0.13.0"
mozjpeg-rs = "0.9.2"
pdfium-render = "0.8"
ffmpeg-next = "7"

# crates/sync-crdt/Cargo.toml
[dependencies]
abyo-crdt = "0.4"
vclock = "0.4.4"
iroh = "1.0.0"
iroh-blobs = "1.0.0"
fastcdc = "4.0.1"
```

### Existing Dependencies to Upgrade

```toml
lz4_flex = "0.13.1"
zstd = "0.13.3"
brotli = "8.0.4"

# Remove (archived)
# pqcrypto-mlkem = "0.19.x"       # → ml-kem
# pqcrypto-mlkem768 = "0.19.x"    # → ml-kem
# pqcrypto-ml-dsa = "0.19.x"      # → ml-dsa
# pqcrypto-falcon = "0.4.x"       # → falcon-rs
# pqcrypto-sphincsplus = "0.7.x"  # → slh-dsa
```

---

## 14. Implementation Roadmap

### Phase 0: Crypto Migration (Urgent — before June 2026)

| Task | Crate | Effort |
|------|-------|--------|
| Replace `pqcrypto-*` → `ml-kem` 0.3.2 | `cybermanju-crypto` | 2-3 days |
| Replace `pqcrypto-ml-dsa` → `ml-dsa` 0.1.1 | `cybermanju-crypto` | 2-3 days |
| Add `hpke` 0.13.0 for hybrid KEM | `cybermanju-crypto` | 1-2 days |
| Add `rusty_paseto` 0.10.0 for tokens | New crate | 2-3 days |
| Add `frost-*` for threshold key splitting | `cybermanju-crypto` | 3-5 days |
| Add `shamir-zero` for key splitting | `cybermanju-erasure` | 1-2 days |
| Update WASM crypto bindings | `cybermanju-drive-wasm` | 2-3 days |
| Update all tests | All crates | 2-3 days |

**Total**: ~2-3 weeks

### Phase 1: Shard Format + Root Index

| Task | Crate | Effort |
|------|-------|--------|
| Implement `.cybermanju` shard format | `portable-db` | 5-7 days |
| Implement `root.cybermanju` master index | `portable-db` | 5-7 days |
| Implement index layer (read without decrypt) | `portable-db` | 3-5 days |
| Implement content layer (chunk encryption) | `crypto` | 3-5 days |
| Add byte-range access API | `portable-db` | 2-3 days |
| Update Tauri commands | `src-tauri` | 3-5 days |
| Update WASM bridge | `drive-wasm` | 2-3 days |

**Total**: ~4-5 weeks

### Phase 2: Erasure Coding

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-erasure` crate | New | 1 day |
| Integrate `reed-solomon-simd` 3.1.0 | `erasure` | 2-3 days |
| Integrate `clay-codes` 0.1.1 | `erasure` | 3-5 days |
| Integrate `fountain_scheme` 1.0.1 | `erasure` | 3-5 days |
| Implement `DistributionPolicy` | `erasure` | 3-5 days |
| Modify sync pipeline for shard distribution | `src-tauri/sync` | 5-7 days |
| Add cost-aware placement | `erasure` | 2-3 days |

**Total**: ~3-4 weeks

### Phase 3: Preview Keys

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-preview-keys` crate | New | 1 day |
| Implement PASETO V4 token generation | `preview-keys` | 3-5 days |
| Implement token validation + revocation | `preview-keys` | 3-5 days |
| Add HKDF key derivation hierarchy | `preview-keys` | 1-2 days |
| Add Merkle tree for revocation list | `preview-keys` | 2-3 days |
| Integrate with Tauri commands | `src-tauri` | 2-3 days |

**Total**: ~2-3 weeks

### Phase 4: Shared Preview Library

| Task | Crate | Effort |
|------|-------|--------|
| Implement content-addressed preview store | `resolutions` | 3-5 days |
| Implement deduplication engine | `resolutions` | 2-3 days |
| Implement sprite sheet generation | `resolutions` | 2-3 days |
| Add `fastcdc` chunking for large previews | `resolutions` | 1-2 days |
| Integrate with sync pipeline | `src-tauri/sync` | 2-3 days |

**Total**: ~2-3 weeks

### Phase 5: Recovery Pipeline

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-recovery` crate | New | 1 day |
| Integrate `fast_image_resize` 6.0 | `recovery` | 1-2 days |
| Integrate `ort` + Real-ESRGAN ONNX | `recovery` | 3-5 days |
| Add `ffmpeg-next` for video thumbnails | `recovery` | 2-3 days |
| Add `pdfium-render` for PDF previews | `recovery` | 2-3 days |
| Implement recovery gradient logic | `recovery` | 3-5 days |
| Integrate with Tauri commands | `src-tauri` | 2-3 days |

**Total**: ~3-4 weeks

### Phase 6: Distributed Sync CRDT

| Task | Crate | Effort |
|------|-------|--------|
| Create `cybermanju-sync-crdt` crate | New | 1 day |
| Integrate `abyo-crdt` or `saorsa` | `sync-crdt` | 3-5 days |
| Implement delta-state sync protocol | `sync-crdt` | 5-7 days |
| Add vector clock causal ordering | `sync-crdt` | 2-3 days |
| Integrate `iroh` for P2P transport | `sync-crdt` | 5-7 days |
| Add conflict resolution UI | Vue frontend | 3-5 days |

**Total**: ~4-5 weeks

---

## 15. Design Decisions Log

### Decision 1: Shard Model (Hundreds of `.cybermanju` files)

**Chosen**: Distributed shards, each a self-contained `.cybermanju` file.

**Alternatives considered**:
- Single `.cybermanju` file: Too large, single point of failure
- Sidecar directories: Not portable, not self-contained
- Pure database (redb): No random-access byte ranges

**Rationale**: Each shard is portable, self-contained, and independently
verifiable. A shard can be copied, moved, or recovered independently.

### Decision 2: Index Always Encrypted (No Plaintext Option)

**Chosen**: Index is ALWAYS encrypted with index_key. No plaintext mode.

**Alternatives considered**:
- Plaintext index: Convenient but leaks metadata (file names, GPS, faces)
- Partial encryption: Public metadata visible, private hidden — complex
- Encrypted index with separate key: Best balance

**Rationale**: File names alone are sensitive ("passport_scan.pdf"). The index
must be opaque to attackers. The index_key is separate from content_key and
preview_key, providing defense in depth.

### Decision 3: Fixed 4 Resolution Levels

**Chosen**: 4 fixed levels (r0-r3) + parity.

**Alternatives considered**:
- 2 levels: Too simple, no recovery gradient
- 8 levels: Diminishing returns, complexity
- Dynamic: Maximum flexibility but hard to reason about

**Rationale**: 4 levels provide gallery access, share preview, detail view,
and original — covering all common use cases.

### Decision 4: Clay Codes for r3, Fountain Codes for Parity

**Chosen**: Clay codes (MSR) for r3, fountain codes (RaptorQ) for parity.

**Rationale**: Clay codes optimize repair bandwidth for large r3 data.
Fountain codes are rateless, ideal for parity where backend count varies.

### Decision 5: PASETO V4 for View Tokens

**Chosen**: PASETO V4 Local tokens.

**Alternatives considered**:
- JWT: Complex, security issues
- age: File encryption, not token standard
- Custom: Reinventing the wheel

**Rationale**: PASETO is a modern token standard designed to replace JWT.
V4 Local uses AES-256-CTR + BLAKE3-MACT, simpler and more secure.

### Decision 6: BLAKE3 for All Hashing

**Chosen**: BLAKE3 for Merkle trees, content addressing, and integrity.

**Rationale**: BLAKE3 is 14× faster than SHA-256, has built-in tree hashing
for parallelism, and is used by iroh, Cargo, and many content-addressed systems.

### Decision 7: `redb` for Metadata, Optional `fjall` for Blob Index

**Chosen**: Keep `redb` as primary DB, optionally add `fjall` for blob indexing.

**Rationale**: `redb` is proven, ACID, stable format. `fjall` is better for
write-heavy blob indexing but adds complexity. Keep it optional.

---

## 16. Crate Migration Guide

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

let signing_key = MlDsa65::generate(&mut rng);
let signature = signing_key.sign_randomized(&mut rng, &message)?;
let verified = signing_key.verify(&message, &signature)?;
```

### PASETO V4 Token Example

```rust
use rusty_paseto::prelude::*;

let preview_key = PasetoSymmetricKey::<V4, Local>::new();

let token = PasetoBuilder::<V4, Local>::new()
    .set_claim("sub", "file_id_001")?
    .set_claim("res", "r0")?
    .set_claim("vcn", 10u32)?
    .set_claim("exp", Expiration::try_from("24h")?)?
    .set_claim("jti", Uuid::new_v4().to_string())?
    .build(&preview_key)?;

let claims = Paseto::<V4, Local>::parse(&token, &preview_key)?;
```

### HPKE Hybrid Key Exchange

```rust
use hpke::{Hpke, DhKem};

let hpke = Hpke::<DhKem>::new();
let (enc, shared_secret) = hpke.encapsulate(&recipient_public_key)?;

let key = hkdf_sha256(shared_secret, "cybermanju-content-v1", file_id);
```

---

## Appendix A: File Size Estimates

### Per-Shard Storage

| Component | Size | Notes |
|-----------|------|-------|
| Header | 512B | Fixed metadata |
| Index layer | 50KB per 100 files | File manifest + blob map |
| Content layer | Variable | Actual file data |
| Erasure metadata | 1KB | Shard mapping |
| Signature | 3KB | ML-DSA-65 |
| **Overhead per shard** | **~55KB** | For 100 files |

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
| Browse file list | <100ms | root.cybermanju + shard indexes |
| Gallery load (100 thumbs) | <100ms | r0 from any shard |
| Share preview | <200ms | r0+r1 with view token |
| Detail view | <500ms | r1+r2 |
| Stream video chunk | <50ms | r2 chunk seek |
| Full download | <5s | r3 reconstruction |
| Recovery (Lanczos) | <100ms | fast_image_resize SIMD |
| Recovery (neural) | <2s | ort + Real-ESRGAN |
| Recovery (erasure) | <10s | clay-codes decode |
| Sync (per file) | <30s | Resolution decomposition |
| Token generation | <10ms | PASETO V4 |
| Token validation | <1ms | PASETO V4 |

---

## Appendix C: Security Considerations

### Threat Model

| Attacker | What They Have | What They Can Do | What Stops Them |
|----------|---------------|-----------------|-----------------|
| Casual | Shard file from backend | Nothing — 88 bytes of plaintext header only | Encryption |
| With brute force | Shard + computing power | Nothing — AES-256-GCM is unbreakable | Key size |
| With index_key | Index key only | See file names, metadata, structure | Content key still protects r2/r3 |
| With preview_key | Preview key only | Cannot find previews without index_key | Index encryption |
| With index+preview | Both keys | See thumbnails, previews (640x480) | Content key protects originals |
| With content_key | Content key only | Cannot find content without index_key | Index encryption |
| With all keys | All three keys | Full access — can read everything | Key management |
| With master key | Master key | Can derive all keys — full compromise | Shamir split |

### Security Properties

1. **Shard opacity**: `.cybermanju` files are opaque without index_key
   - 88 bytes of plaintext (magic, shard_id, size, root_hash)
   - Rest is encrypted — indistinguishable from random bytes
   - Cannot determine file types, names, or structure

2. **Key separation**: Three independent keys for three purposes
   - index_key: Controls metadata access
   - preview_key: Controls preview access
   - content_key: Controls full content access
   - Compromising one doesn't compromise others

3. **Preview safety**: Previews are encrypted inside the shard
   - Cannot extract r0/r1 without both index_key AND preview_key
   - Even with preview_key, you need index_key to find the byte ranges
   - Previews are safe at rest on backends

4. **Erasure coding safety**: Parity shards are encrypted
   - Cannot reconstruct without content_key
   - Cannot determine which shards have parity without index_key

5. **Tamper detection**: ML-DSA-65 signatures on every shard
   - Any modification is detectable
   - Signature is plaintext for verification

### What's NOT Protected

1. **File names in plaintext header**: The shard_id is visible — this is
   necessary for routing but reveals nothing about content.

2. **Shard size**: Visible in header — reveals how much data is in the shard
   but not what type of data.

3. **Access patterns**: If an attacker monitors which shards you download,
   they can infer usage patterns. Mitigate with: constant-size shards,
   dummy downloads, Tor routing.

4. **Timing attacks**: Decryption time may vary based on key correctness.
   Mitigate with: constant-time operations, padding.

### Recommendations

1. **Never store master key on the same backend as shards**
2. **Split master key via Shamir (3-of-5) across physical devices**
3. **Rotate keys periodically** — re-encrypt shards with new keys
4. **Use hardware security module (HSM) for key storage** if possible
5. **Enable shard signatures** — detect tampering
6. **Monitor shard integrity** — verify BLAKE3 hashes regularly

---

*Document version: 2.2.0*
*Last updated: 2026-06-20*
*Status: Architecture proposal — pending review*
