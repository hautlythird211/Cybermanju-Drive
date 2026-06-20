use anyhow::{anyhow, Result};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub color_type: String,
    pub has_alpha: bool,
    pub bits_per_channel: u16,
    pub exif: Option<ExifData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_taken: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    pub exposure_time: Option<String>,
    pub f_number: Option<f64>,
    pub iso: Option<u32>,
    pub focal_length: Option<f64>,
    pub orientation: Option<u32>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageTransform {
    RotateCW,
    RotateCCW,
    Rotate180,
    FlipH,
    FlipV,
    Resize { width: u32, height: u32 },
    Crop { x: u32, y: u32, w: u32, h: u32 },
}

pub fn extract_image_info(data: &[u8]) -> Result<ImageInfo> {
    let img = image::load_from_memory(data)?;
    let (w, h) = (img.width(), img.height());

    let format = match img {
        DynamicImage::ImageLuma8(_) => "grayscale",
        DynamicImage::ImageLumaA8(_) => "grayscale_alpha",
        DynamicImage::ImageRgb8(_) => "rgb",
        DynamicImage::ImageRgba8(_) => "rgba",
        DynamicImage::ImageBgr8(_) => "bgr",
        DynamicImage::ImageBgra8(_) => "bgra",
        DynamicImage::ImageLuma16(_) => "luma16",
        DynamicImage::ImageLumaA16(_) => "luma_alpha16",
        DynamicImage::ImageRgb16(_) => "rgb16",
        DynamicImage::ImageRgba16(_) => "rgba16",
        _ => "unknown",
    };

    let has_alpha = matches!(
        img,
        DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageBgra8(_)
            | DynamicImage::ImageLumaA8(_)
            | DynamicImage::ImageRgba16(_)
            | DynamicImage::ImageLumaA16(_)
    );

    let exif = extract_exif_data(data);

    Ok(ImageInfo {
        width: w,
        height: h,
        format: format.to_string(),
        color_type: format!("{:?}", img.color()),
        has_alpha,
        bits_per_channel: 8,
        exif,
    })
}

fn extract_exif_data(data: &[u8]) -> Option<ExifData> {
    let mut buf = Cursor::new(data);
    let reader = exif::Reader::new();
    let exif = reader.read_from_container(&mut buf).ok()?;

    let get_str = |tag: exif::Tag| -> Option<String> {
        exif.get_field(tag, exif::In::PRIMARY)
            .and_then(|f| f.display_value().to_string().into())
    };

    let get_f64 = |tag: exif::Tag| -> Option<f64> {
        exif.get_field(tag, exif::In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse().ok())
    };

    let get_u32 = |tag: exif::Tag| -> Option<u32> {
        exif.get_field(tag, exif::In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse().ok())
    };

    let (gps_lat, gps_lon) = extract_gps(&exif);

    Some(ExifData {
        camera_make: get_str(exif::Tag::Make),
        camera_model: get_str(exif::Tag::Model),
        date_taken: get_str(exif::Tag::DateTimeOriginal),
        gps_lat,
        gps_lon,
        exposure_time: get_str(exif::Tag::ExposureTime),
        f_number: get_f64(exif::Tag::FNumber),
        iso: get_u32(exif::Tag::ISOSpeedRatings),
        focal_length: get_f64(exif::Tag::FocalLength),
        orientation: get_u32(exif::Tag::Orientation),
        image_width: get_u32(exif::Tag::PixelXDimension),
        image_height: get_u32(exif::Tag::PixelYDimension),
    })
}

fn extract_gps(exif: &exif::Exif) -> (Option<f64>, Option<f64>) {
    let lat = exif
        .get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
        .and_then(|f| {
            let val = f.display_value().to_string();
            parse_dms_to_decimal(&val)
        });

    let lon = exif
        .get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)
        .and_then(|f| {
            let val = f.display_value().to_string();
            parse_dms_to_decimal(&val)
        });

    (lat, lon)
}

fn parse_dms_to_decimal(dms: &str) -> Option<f64> {
    let parts: Vec<&str> = dms.split(',').map(|s| s.trim()).collect();
    if parts.len() < 3 {
        return None;
    }
    let deg: f64 = parts[0].parse().ok()?;
    let min: f64 = parts[1].parse().ok()?;
    let sec_part: Vec<&str> = parts[2].split(' ').collect();
    let sec: f64 = sec_part[0].parse().ok()?;
    Some(deg + min / 60.0 + sec / 3600.0)
}

pub fn transform_image(data: &[u8], transform: &ImageTransform) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data)?;

    let transformed = match transform {
        ImageTransform::RotateCW => img.rotate90(),
        ImageTransform::RotateCCW => img.rotate270(),
        ImageTransform::Rotate180 => img.rotate180(),
        ImageTransform::FlipH => img.fliph(),
        ImageTransform::FlipV => img.flipv(),
        ImageTransform::Resize { width, height } => {
            img.resize_exact(*width, *height, image::imageops::FilterType::Lanczos3)
        }
        ImageTransform::Crop { x, y, w, h } => {
            img.crop(*x, *y, *w, *h)
        }
    };

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    transformed.write_to(&mut cursor, image::ImageOutputFormat::Png)?;
    Ok(buf)
}

pub fn resize_image(data: &[u8], max_width: u32, max_height: u32, quality: u8) -> Result<Vec<u8>> {
    let img = image::load_from_memory(data)?;
    let (w, h) = (img.width(), img.height());

    let ratio = (max_width as f64 / w as f64).min(max_height as f64 / h as f64);
    if ratio >= 1.0 {
        return Ok(data.to_vec());
    }

    let new_w = (w as f64 * ratio) as u32;
    let new_h = (h as f64 * ratio) as u32;
    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    resized.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(quality))?;
    Ok(buf)
}

pub fn get_image_dimensions(data: &[u8]) -> Result<(u32, u32)> {
    let img = image::load_from_memory(data)?;
    Ok((img.width(), img.height()))
}
