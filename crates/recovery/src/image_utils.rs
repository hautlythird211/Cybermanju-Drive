use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, Rgba, RgbaImage};

use crate::errors::RecoveryError;

/// Resize image data using Lanczos-3 resampling
pub fn resize_lanczos(
    input: &[u8],
    width: u32,
    height: u32,
    target_width: u32,
    target_height: u32,
    channels: u8,
) -> Result<Vec<u8>, RecoveryError> {
    let img = decode_to_rgba(input, width, height, channels)?;
    let resized = img.resize(target_width, target_height, FilterType::Lanczos3);
    Ok(resized.to_rgba8().to_vec())
}

/// Resize image data using bilinear interpolation (fallback)
pub fn resize_bilinear(
    input: &[u8],
    width: u32,
    height: u32,
    target_width: u32,
    target_height: u32,
    channels: u8,
) -> Result<Vec<u8>, RecoveryError> {
    let img = decode_to_rgba(input, width, height, channels)?;
    let resized = img.resize(target_width, target_height, FilterType::Triangle);
    Ok(resized.to_rgba8().to_vec())
}

/// Encode raw RGBA bytes to WebP format (lossless)
pub fn to_webp(
    input: &[u8],
    width: u32,
    height: u32,
    _quality: u8,
) -> Result<Vec<u8>, RecoveryError> {
    let img = RgbaImage::from_raw(width, height, input.to_vec())
        .ok_or_else(|| RecoveryError::ImageError("invalid image dimensions".into()))?;

    let dynamic = DynamicImage::ImageRgba8(img);
    let rgb = dynamic.to_rgb8();

    let mut buf = Vec::new();
    let encoder = WebPEncoder::new_lossless(&mut buf);
    encoder
        .encode(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| RecoveryError::ImageError(e.to_string()))?;
    Ok(buf)
}

/// Encode raw RGBA bytes to JPEG format
pub fn to_jpeg(
    input: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, RecoveryError> {
    let img = RgbaImage::from_raw(width, height, input.to_vec())
        .ok_or_else(|| RecoveryError::ImageError("invalid image dimensions".into()))?;

    let dynamic = DynamicImage::ImageRgba8(img);
    let rgb = dynamic.to_rgb8();

    let mut buf = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    encoder
        .encode(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| RecoveryError::ImageError(e.to_string()))?;
    Ok(buf)
}

/// Decode any image format to raw RGBA bytes
pub fn from_bytes(input: &[u8]) -> Result<(Vec<u8>, u32, u32, u8), RecoveryError> {
    let img =
        image::load_from_memory(input).map_err(|e| RecoveryError::ImageError(e.to_string()))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.to_vec(), w, h, 4))
}

/// Auto-fit resize maintaining aspect ratio, max dimension constraint
pub fn create_thumbnail(
    input: &[u8],
    width: u32,
    height: u32,
    max_dim: u32,
) -> Result<Vec<u8>, RecoveryError> {
    let img = decode_to_rgba(input, width, height, 4)?;
    let thumbnail = img.resize(max_dim, max_dim, FilterType::Lanczos3);
    Ok(thumbnail.to_rgba8().to_vec())
}

/// Rough estimate of JPEG output size in bytes
pub fn estimate_jpeg_size(width: u32, height: u32, quality: u8) -> u64 {
    let pixels = (width as u64) * (height as u64);
    // Rough heuristic: JPEG ratio varies by quality and content
    let ratio = match quality {
        0..=30 => 0.05,
        31..=60 => 0.1,
        61..=80 => 0.2,
        81..=95 => 0.4,
        _ => 0.7,
    };
    (pixels * 3 / 8) as u64 * (ratio * 100.0) as u64 / 100
}

/// Rough estimate of WebP output size in bytes
pub fn estimate_webp_size(width: u32, height: u32, quality: u8) -> u64 {
    let jpeg_est = estimate_jpeg_size(width, height, quality);
    // WebP is typically 25-35% smaller than JPEG
    jpeg_est * 70 / 100
}

/// Calculate Peak Signal-to-Noise Ratio between original and reconstructed
pub fn calculate_psnr(original: &[u8], reconstructed: &[u8]) -> f64 {
    if original.len() != reconstructed.len() || original.is_empty() {
        return 0.0;
    }

    let mut mse = 0.0f64;
    for (o, r) in original.iter().zip(reconstructed.iter()) {
        let diff = *o as f64 - *r as f64;
        mse += diff * diff;
    }
    mse /= original.len() as f64;

    if mse == 0.0 {
        return f64::INFINITY;
    }

    10.0 * (255.0_f64 * 255.0 / mse).log10()
}

/// Calculate Structural Similarity Index (simplified)
pub fn calculate_ssim(original: &[u8], reconstructed: &[u8], width: u32, height: u32) -> f64 {
    if original.len() != reconstructed.len() || original.is_empty() {
        return 0.0;
    }

    let n = original.len() as f64;
    let mean_o: f64 = original.iter().map(|&x| x as f64).sum::<f64>() / n;
    let mean_r: f64 = reconstructed.iter().map(|&x| x as f64).sum::<f64>() / n;

    let var_o: f64 = original
        .iter()
        .map(|&x| (x as f64 - mean_o).powi(2))
        .sum::<f64>()
        / n;
    let var_r: f64 = reconstructed
        .iter()
        .map(|&x| (x as f64 - mean_r).powi(2))
        .sum::<f64>()
        / n;

    let cov: f64 = original
        .iter()
        .zip(reconstructed.iter())
        .map(|(&o, &r)| (o as f64 - mean_o) * (r as f64 - mean_r))
        .sum::<f64>()
        / n;

    let c1 = (0.01_f64 * 255.0).powi(2);
    let c2 = (0.03_f64 * 255.0).powi(2);

    let ssim = ((2.0 * mean_o * mean_r + c1) * (2.0 * cov + c2))
        / ((mean_o.powi(2) + mean_r.powi(2) + c1) * (var_o + var_r + c2));

    ssim
}

/// Helper: decode input bytes to DynamicImage, handling different channel counts
fn decode_to_rgba(
    input: &[u8],
    width: u32,
    height: u32,
    channels: u8,
) -> Result<DynamicImage, RecoveryError> {
    match channels {
        4 => {
            let img = RgbaImage::from_raw(width, height, input.to_vec())
                .ok_or_else(|| RecoveryError::ImageError("invalid RGBA dimensions".into()))?;
            Ok(DynamicImage::ImageRgba8(img))
        }
        3 => {
            let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, input.to_vec())
                .ok_or_else(|| RecoveryError::ImageError("invalid RGB dimensions".into()))?;
            Ok(DynamicImage::ImageRgb8(img))
        }
        _ => Err(RecoveryError::ImageError(format!(
            "unsupported channel count: {}",
            channels
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psnr_identical() {
        let data = vec![128u8; 100];
        assert_eq!(calculate_psnr(&data, &data), f64::INFINITY);
    }

    #[test]
    fn test_ssim_identical() {
        let data = vec![128u8; 100];
        let ssim = calculate_ssim(&data, &data, 10, 10);
        assert!((ssim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_estimate_sizes() {
        let jpeg = estimate_jpeg_size(1920, 1080, 85);
        let webp = estimate_webp_size(1920, 1080, 85);
        assert!(webp < jpeg);
    }
}
