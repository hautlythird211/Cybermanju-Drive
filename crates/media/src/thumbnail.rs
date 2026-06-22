use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailResult {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: usize,
}

pub fn generate_image_thumbnail(
    data: &[u8],
    max_size: u32,
    output_format: &str,
    quality: u8,
) -> Result<ThumbnailResult> {
    let img = image::load_from_memory(data)?;
    let (w, h) = (img.width(), img.height());

    let scale = if w > h {
        max_size as f64 / w as f64
    } else {
        max_size as f64 / h as f64
    };
    let new_w = (w as f64 * scale) as u32;
    let new_h = (h as f64 * scale) as u32;

    let thumbnail = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);

    match output_format {
        "jpeg" | "jpg" => {
            let encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
            thumbnail.write_with_encoder(encoder)?;
        }
        "png" => {
            thumbnail.write_to(&mut cursor, ImageFormat::Png)?;
        }
        _ => {
            thumbnail.write_to(&mut cursor, ImageFormat::Png)?;
        }
    }

    Ok(ThumbnailResult {
        data: buf.clone(),
        width: new_w,
        height: new_h,
        format: output_format.to_string(),
        size_bytes: buf.len(),
    })
}

pub fn generate_video_thumbnail_placeholder(width: u32, height: u32) -> Result<ThumbnailResult> {
    let img = DynamicImage::new_rgb8(width, height);
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, ImageFormat::Png)?;

    Ok(ThumbnailResult {
        data: buf,
        width,
        height,
        format: "png".to_string(),
        size_bytes: 0,
    })
}

pub fn batch_generate_thumbnails(
    items: &[(String, Vec<u8>)],
    max_size: u32,
    output_format: &str,
    quality: u8,
) -> Vec<(String, Result<ThumbnailResult>)> {
    items
        .iter()
        .map(|(id, data)| {
            let result = generate_image_thumbnail(data, max_size, output_format, quality);
            (id.clone(), result)
        })
        .collect()
}
