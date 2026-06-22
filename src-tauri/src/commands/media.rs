use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionInfo {
    pub level: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub size_bytes: Option<u64>,
    pub key_tier: String,
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMediaData {
    pub file_id: String,
    pub filename: String,
    pub mime_type: String,
    pub is_image: bool,
    pub is_video: bool,
    pub is_audio: bool,
    pub image_info: Option<cybermanju_media::ImageInfo>,
    pub video_info: Option<cybermanju_media::VideoInfo>,
    pub available_resolutions: Vec<ResolutionInfo>,
    pub selected_resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionData {
    pub file_id: String,
    pub level: String,
    pub data_base64: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[tauri::command]
pub async fn get_media_info(
    file_id: String,
    filename: String,
    data: Vec<u8>,
    _state: State<'_, AppState>,
) -> Result<FileMediaData, String> {
    let media_info =
        cybermanju_media::detect_media_type(&data, &filename).map_err(|e| e.to_string())?;

    let image_info = if media_info.is_image {
        cybermanju_media::extract_image_info(&data).ok()
    } else {
        None
    };

    let video_info = if media_info.is_video {
        Some(cybermanju_media::VideoInfo {
            duration_secs: 0.0,
            width: 0,
            height: 0,
            codec: "unknown".to_string(),
            fps: 0.0,
            bitrate: 0,
            audio_codec: None,
            audio_sample_rate: None,
            audio_channels: None,
            container: media_info.extension.clone(),
            total_frames: None,
        })
    } else {
        None
    };

    let available_resolutions = vec![
        ResolutionInfo {
            level: "r0".to_string(),
            width: Some(200),
            height: Some(150),
            format: Some("webp".to_string()),
            size_bytes: Some(3072),
            key_tier: "preview".to_string(),
            encrypted: true,
        },
        ResolutionInfo {
            level: "r1".to_string(),
            width: Some(640),
            height: Some(480),
            format: Some("jpeg".to_string()),
            size_bytes: Some(46080),
            key_tier: "preview".to_string(),
            encrypted: true,
        },
        ResolutionInfo {
            level: "r2".to_string(),
            width: Some(1920),
            height: Some(1080),
            format: Some("jpeg".to_string()),
            size_bytes: Some(460800),
            key_tier: "content".to_string(),
            encrypted: true,
        },
        ResolutionInfo {
            level: "r3".to_string(),
            width: image_info.as_ref().map(|i| i.width),
            height: image_info.as_ref().map(|i| i.height),
            format: Some(media_info.extension.clone()),
            size_bytes: Some(data.len() as u64),
            key_tier: "content".to_string(),
            encrypted: true,
        },
    ];

    Ok(FileMediaData {
        file_id,
        filename,
        mime_type: media_info.mime_type.clone(),
        is_image: media_info.is_image,
        is_video: media_info.is_video,
        is_audio: media_info.is_audio,
        image_info,
        video_info,
        available_resolutions,
        selected_resolution: "r3".to_string(),
    })
}

#[tauri::command]
pub async fn get_resolution_data(
    file_id: String,
    level: String,
    data: Vec<u8>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<ResolutionData, String> {
    let max_dim = match level.as_str() {
        "r0" => 200,
        "r1" => 640,
        "r2" => 1920,
        "r3" => {
            return Ok(ResolutionData {
                file_id,
                level,
                data_base64: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &data,
                ),
                width: width.unwrap_or(0),
                height: height.unwrap_or(0),
                format: "original".to_string(),
            })
        }
        _ => 640,
    };

    let target_w = width.unwrap_or(max_dim);
    let target_h = height.unwrap_or(max_dim);

    let resized =
        cybermanju_media::resize_image(&data, target_w, target_h, 85).map_err(|e| e.to_string())?;

    let dims = cybermanju_media::get_image_dimensions(&resized).unwrap_or((target_w, target_h));

    Ok(ResolutionData {
        file_id,
        level,
        data_base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &resized),
        width: dims.0,
        height: dims.1,
        format: "jpeg".to_string(),
    })
}

#[tauri::command]
pub async fn transform_image_file(data: Vec<u8>, transform: String) -> Result<Vec<u8>, String> {
    let t = match transform.as_str() {
        "rotate_cw" => cybermanju_media::ImageTransform::RotateCW,
        "rotate_ccw" => cybermanju_media::ImageTransform::RotateCCW,
        "rotate_180" => cybermanju_media::ImageTransform::Rotate180,
        "flip_h" => cybermanju_media::ImageTransform::FlipH,
        "flip_v" => cybermanju_media::ImageTransform::FlipV,
        _ => return Err(format!("Unknown transform: {}", transform)),
    };

    cybermanju_media::transform_image(&data, &t).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_thumbnail_cmd(
    data: Vec<u8>,
    max_size: u32,
    format: String,
    quality: u8,
) -> Result<cybermanju_media::ThumbnailResult, String> {
    cybermanju_media::generate_image_thumbnail(&data, max_size, &format, quality)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn detect_media_type_cmd(
    data: Vec<u8>,
    filename: String,
) -> Result<cybermanju_media::MediaInfo, String> {
    cybermanju_media::detect_media_type(&data, &filename).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_image_dimensions_cmd(data: Vec<u8>) -> Result<(u32, u32), String> {
    cybermanju_media::get_image_dimensions(&data).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_generate_thumbnails_cmd(
    items: Vec<(String, Vec<u8>)>,
    max_size: u32,
    format: String,
    quality: u8,
) -> Result<Vec<(String, Option<cybermanju_media::ThumbnailResult>)>, String> {
    let results = cybermanju_media::batch_generate_thumbnails(&items, max_size, &format, quality);
    Ok(results.into_iter().map(|(id, r)| (id, r.ok())).collect())
}
