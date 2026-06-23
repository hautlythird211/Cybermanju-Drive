use anyhow::Result;

use crate::triple::TripleCompressor;
use crate::types::{CompressionStats, LayerDetail};

/// Content-aware compression that selects the best algorithm based on MIME type.
pub fn compress_adaptive(
    compressor: &TripleCompressor,
    data: &[u8],
    mime: Option<&str>,
) -> Result<(Vec<u8>, CompressionStats)> {
    let mime = mime.unwrap_or("application/octet-stream");
    let original_size = data.len() as u64;
    let start = std::time::Instant::now();

    match mime {
        // Already compressed formats — skip all layers
        m if m.starts_with("image/jpeg")
            || m.starts_with("image/webp")
            || m.starts_with("image/gif")
            || m.starts_with("video/")
            || m.starts_with("audio/")
            || m == "application/zip"
            || m == "application/gzip" =>
        {
            let hash = blake3::hash(data);
            let duration_ms = start.elapsed().as_millis() as u64;
            Ok((
                data.to_vec(),
                CompressionStats {
                    original_size,
                    compressed_size: original_size,
                    ratio: 1.0,
                    layer: "identity".into(),
                    layer_details: vec![LayerDetail {
                        name: "Identity (already compressed)".into(),
                        algorithm: "none".into(),
                        input_size: original_size,
                        output_size: original_size,
                        ratio: 1.0,
                        color: "#6B7280".into(),
                    }],
                    blake3_hash: hash.to_hex().to_string(),
                    duration_ms,
                },
            ))
        }

        // Text/code: Brotli is best
        m if m.starts_with("text/") || m == "application/json" || m == "application/javascript" => {
            compressor.compress_brotli(data).map(|compressed| {
                let hash = blake3::hash(data);
                let duration_ms = start.elapsed().as_millis() as u64;
                let compressed_size = compressed.len() as u64;
                (
                    compressed,
                    CompressionStats {
                        original_size,
                        compressed_size,
                        ratio: if original_size > 0 {
                            compressed_size as f64 / original_size as f64
                        } else {
                            1.0
                        },
                        layer: "brotli".into(),
                        layer_details: vec![LayerDetail {
                            name: "Brotli (text-optimized)".into(),
                            algorithm: "brotli level 11".into(),
                            input_size: original_size,
                            output_size: compressed_size,
                            ratio: compressed_size as f64 / original_size as f64,
                            color: "#FFB800".into(),
                        }],
                        blake3_hash: hash.to_hex().to_string(),
                        duration_ms,
                    },
                )
            })
        }

        // Large binary blobs (> 10MB): prefer ZSTD for speed
        _ if data.len() > 10_000_000 => compressor.compress_zstd(data).map(|compressed| {
            let hash = blake3::hash(data);
            let duration_ms = start.elapsed().as_millis() as u64;
            let compressed_size = compressed.len() as u64;
            (
                compressed,
                CompressionStats {
                    original_size,
                    compressed_size,
                    ratio: if original_size > 0 {
                        compressed_size as f64 / original_size as f64
                    } else {
                        1.0
                    },
                    layer: "zstd".into(),
                    layer_details: vec![LayerDetail {
                        name: "Zstandard (large binary)".into(),
                        algorithm: "zstd level 15".into(),
                        input_size: original_size,
                        output_size: compressed_size,
                        ratio: compressed_size as f64 / original_size as f64,
                        color: "#00FF41".into(),
                    }],
                    blake3_hash: hash.to_hex().to_string(),
                    duration_ms,
                },
            )
        }),

        // Default: triple compression
        _ => compressor.compress_triple(data),
    }
}

/// Sniff MIME type from magic bytes if MIME is unknown.
pub fn sniff_mime(data: &[u8]) -> &'static str {
    if data.len() < 4 {
        return "application/octet-stream";
    }
    match &data[..4] {
        [0xFF, 0xD8, 0xFF, _] => "image/jpeg",
        [0x89, 0x50, 0x4E, 0x47] => "image/png",
        b"GIF8" => "image/gif",
        b"RIFF" => "image/webp",
        [0x1F, 0x8B, _, _] => "application/gzip",
        b"PK\x03\x04" => "application/zip",
        b"%PDF" => "application/pdf",
        b"\x00\x00\x00\x1C" => "video/mp4",
        b"\x1A\x45\xDF\xA3" => "video/webm",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniff_mime() {
        assert_eq!(sniff_mime(&[0xFF, 0xD8, 0xFF, 0xE0]), "image/jpeg");
        assert_eq!(sniff_mime(&[0x89, 0x50, 0x4E, 0x47]), "image/png");
        assert_eq!(sniff_mime(b"GIF89a"), "image/gif");
        assert_eq!(sniff_mime(b"\x00\x00\x00\x1C"), "video/mp4");
        assert_eq!(sniff_mime(&[0x00]), "application/octet-stream");
    }
}
