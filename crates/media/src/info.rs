use anyhow::Result;
use infer::Infer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub mime_type: String,
    pub extension: String,
    pub is_image: bool,
    pub is_video: bool,
    pub is_audio: bool,
    pub supported_resolutions: Vec<String>,
    pub estimated_file_type: String,
}

pub fn detect_media_type(data: &[u8], filename: &str) -> Result<MediaInfo> {
    let matcher = Infer::new();

    let mime = if let Some(kind) = matcher.get(data) {
        kind.mime_type().to_string()
    } else {
        mime_guess::from_path(filename)
            .first_or_octet_stream()
            .to_string()
    };

    let ext = mime_guess::from_path(filename)
        .first()
        .map(|m| {
            m.suffix()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "bin".to_string())
        })
        .unwrap_or_else(|| "bin".to_string());

    let is_image = mime.starts_with("image/");
    let is_video = mime.starts_with("video/");
    let is_audio = mime.starts_with("audio/");

    let supported_resolutions = if is_image {
        vec![
            "r0".to_string(),
            "r1".to_string(),
            "r2".to_string(),
            "r3".to_string(),
        ]
    } else if is_video {
        vec!["r0".to_string(), "r1".to_string(), "r2".to_string()]
    } else {
        vec!["r3".to_string()]
    };

    let estimated_file_type = if is_image {
        "image".to_string()
    } else if is_video {
        "video".to_string()
    } else if is_audio {
        "audio".to_string()
    } else {
        "file".to_string()
    };

    Ok(MediaInfo {
        mime_type: mime,
        extension: ext,
        is_image,
        is_video,
        is_audio,
        supported_resolutions,
        estimated_file_type,
    })
}

pub fn is_media_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    matches!(
        ext.as_str(),
        "jpg"
            | "jpeg"
            | "png"
            | "gif"
            | "webp"
            | "bmp"
            | "svg"
            | "tiff"
            | "avif"
            | "heic"
            | "heif"
            | "ico"
            | "mp4"
            | "mov"
            | "avi"
            | "mkv"
            | "webm"
            | "wmv"
            | "flv"
            | "m4v"
            | "3gp"
            | "ogv"
            | "mp3"
            | "wav"
            | "ogg"
            | "flac"
            | "aac"
            | "m4a"
            | "wma"
            | "opus"
            | "mid"
            | "midi"
    )
}

pub fn is_image_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    matches!(
        ext.as_str(),
        "jpg"
            | "jpeg"
            | "png"
            | "gif"
            | "webp"
            | "bmp"
            | "svg"
            | "tiff"
            | "avif"
            | "heic"
            | "heif"
            | "ico"
    )
}

pub fn is_video_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    matches!(
        ext.as_str(),
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "wmv" | "flv" | "m4v" | "3gp" | "ogv"
    )
}

pub fn is_audio_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    matches!(
        ext.as_str(),
        "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" | "wma" | "opus" | "mid" | "midi"
    )
}
