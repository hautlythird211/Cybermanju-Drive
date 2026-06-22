use tauri::command;
use crate::AppState;
use tauri::State;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RecoveryRequest {
    pub file_id: String,
    pub chunks: Vec<Vec<u8>>,
    pub target_resolution: String,
    pub quality: u8,
    pub upscale: bool,
    pub upscale_model: String,
    pub output_format: String,
}

#[derive(serde::Serialize)]
pub struct RecoveryResponse {
    pub file_id: String,
    pub recovered_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: u64,
    pub method: String,
    pub quality_score: f64,
    pub processing_time_ms: u64,
}

pub async fn recover_file(
    file_id: String,
    chunks: Vec<Vec<u8>>,
    request: RecoveryRequest,
) -> Result<RecoveryResponse, String> {
    let pipeline = cybermanju_recovery::RecoveryPipeline::new();
    let recovery_request = cybermanju_recovery::RecoveryRequest {
        file_id: request.file_id,
        target_resolution: request.target_resolution,
        quality: request.quality,
        upscale: request.upscale,
        upscale_model: request.upscale_model,
        output_format: request.output_format,
    };
    let result = pipeline
        .recover_from_chunks(&chunks, &recovery_request)
        .map_err(|e| e.to_string())?;
    Ok(RecoveryResponse {
        file_id: result.file_id,
        recovered_data: result.recovered_data,
        width: result.width,
        height: result.height,
        format: result.format,
        size_bytes: result.size_bytes,
        method: result.method,
        quality_score: result.quality_score,
        processing_time_ms: result.processing_time_ms,
    })
}

#[command]
pub async fn upscale_file(
    _state: State<'_, AppState>,
    file_id: String,
    data: Vec<u8>,
    width: u32,
    height: u32,
    model: String,
    scale: u32,
) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Err(format!("file {}: empty data", file_id));
    }
    if scale == 0 || scale > 8 {
        return Err(format!("file {}: scale must be 1-8, got {}", file_id, scale));
    }
    let upscale_model = match model.as_str() {
        "real-esrgan-x2" => cybermanju_recovery::UpscaleModel::RealEsrganX2,
        "real-esrgan-x4" => cybermanju_recovery::UpscaleModel::RealEsrganX4,
        "real-cugan" => cybermanju_recovery::UpscaleModel::RealCugan,
        _ => cybermanju_recovery::UpscaleModel::LanczosFallback,
    };
    log::info!("Upscaling file {} ({}x{}) with {:?} x{}", file_id, width, height, upscale_model, scale);
    let upscaler = cybermanju_recovery::NeuralUpscaler::new(upscale_model);
    upscaler
        .upscale(&data, width, height, scale)
        .map_err(|e| format!("file {}: {}", file_id, e))
}

#[command]
pub async fn upscale_region(
    _state: State<'_, AppState>,
    file_id: String,
    data: Vec<u8>,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    region_w: u32,
    region_h: u32,
    model: String,
    scale: u32,
) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Err(format!("file {}: empty data", file_id));
    }
    if scale == 0 || scale > 8 {
        return Err(format!("file {}: scale must be 1-8, got {}", file_id, scale));
    }
    if x + region_w > width || y + region_h > height {
        return Err(format!("file {}: region ({},{},{},{}) exceeds image bounds ({}x{})", file_id, x, y, region_w, region_h, width, height));
    }
    let upscale_model = match model.as_str() {
        "real-esrgan-x2" => cybermanju_recovery::UpscaleModel::RealEsrganX2,
        "real-esrgan-x4" => cybermanju_recovery::UpscaleModel::RealEsrganX4,
        "real-cugan" => cybermanju_recovery::UpscaleModel::RealCugan,
        _ => cybermanju_recovery::UpscaleModel::LanczosFallback,
    };
    log::info!("Upscaling region of file {} ({},{},{},{}) with {:?} x{}", file_id, x, y, region_w, region_h, upscale_model, scale);
    let upscaler = cybermanju_recovery::NeuralUpscaler::new(upscale_model);
    upscaler
        .upscale_region(&data, width, height, x, y, region_w, region_h, scale)
        .map_err(|e| format!("file {}: {}", file_id, e))
}
