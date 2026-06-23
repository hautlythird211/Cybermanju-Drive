use serde::{Deserialize, Serialize};

use crate::errors::RecoveryError;
use crate::image_utils;
use crate::neural_upscaler::{NeuralUpscaler, UpscaleModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRequest {
    pub file_id: String,
    pub target_resolution: String,
    pub quality: u8,
    pub upscale: bool,
    pub upscale_model: String,
    pub output_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
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

pub struct RecoveryPipeline;

impl Default for RecoveryPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryPipeline {
    pub fn new() -> Self {
        Self
    }

    /// Recover file from chunks, apply upscaling if requested
    pub fn recover_from_chunks(
        &self,
        chunks: &[Vec<u8>],
        request: &RecoveryRequest,
    ) -> Result<RecoveryResult, RecoveryError> {
        let start = std::time::Instant::now();

        // Reconstruct data from chunks
        let mut data = Vec::new();
        for chunk in chunks {
            data.extend_from_slice(chunk);
        }

        // Decode the image to get dimensions
        let (rgba_data, width, height, _channels) = image_utils::from_bytes(&data)?;

        let mut result_data = rgba_data.clone();
        let mut method = "direct".to_string();

        // Apply upscaling if requested
        if request.upscale {
            let model = match request.upscale_model.as_str() {
                "real-esrgan-x2" => UpscaleModel::RealEsrganX2,
                "real-esrgan-x4" => UpscaleModel::RealEsrganX4,
                "real-cugan" => UpscaleModel::RealCugan,
                _ => UpscaleModel::LanczosFallback,
            };
            let upscaler = NeuralUpscaler::new(model);
            let scale = self.determine_scale(&request.target_resolution);
            result_data = upscaler.upscale(&result_data, width, height, scale)?;
            method = format!("upscale-{}", request.upscale_model);
        }

        // Encode to output format
        let (output_data, out_w, out_h) = match request.output_format.as_str() {
            "webp" => {
                let encoded = image_utils::to_webp(&result_data, width, height, request.quality)?;
                (encoded, width, height)
            }
            "jpeg" | "jpg" => {
                let encoded = image_utils::to_jpeg(&result_data, width, height, request.quality)?;
                (encoded, width, height)
            }
            _ => (result_data.clone(), width, height),
        };

        // Calculate quality score using PSNR if we have original
        let quality_score = if !rgba_data.is_empty() && !output_data.is_empty() {
            let (recovered_rgba, _, _, _) =
                image_utils::from_bytes(&output_data).unwrap_or_default();
            if !recovered_rgba.is_empty() {
                image_utils::calculate_psnr(&rgba_data, &recovered_rgba)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(RecoveryResult {
            file_id: request.file_id.clone(),
            recovered_data: output_data.clone(),
            width: out_w,
            height: out_h,
            format: request.output_format.clone(),
            size_bytes: output_data.len() as u64,
            method,
            quality_score,
            processing_time_ms: elapsed,
        })
    }

    /// Recover from a preview by upscaling to target resolution
    pub fn recover_from_preview(
        &self,
        preview_data: &[u8],
        request: &RecoveryRequest,
    ) -> Result<RecoveryResult, RecoveryError> {
        let start = std::time::Instant::now();

        let (rgba_data, width, height, _) = image_utils::from_bytes(preview_data)?;

        let scale = self.determine_scale(&request.target_resolution);
        let upscaler = NeuralUpscaler::new(UpscaleModel::LanczosFallback);
        let upscaled = upscaler.upscale(&rgba_data, width, height, scale)?;

        let output_data = match request.output_format.as_str() {
            "webp" => {
                image_utils::to_webp(&upscaled, width * scale, height * scale, request.quality)?
            }
            "jpeg" | "jpg" => {
                image_utils::to_jpeg(&upscaled, width * scale, height * scale, request.quality)?
            }
            _ => upscaled,
        };

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(RecoveryResult {
            file_id: request.file_id.clone(),
            recovered_data: output_data.clone(),
            width: width * scale,
            height: height * scale,
            format: request.output_format.clone(),
            size_bytes: output_data.len() as u64,
            method: "preview-upscale".to_string(),
            quality_score: 0.0,
            processing_time_ms: elapsed,
        })
    }

    /// Reconstruct missing shards using erasure coding
    pub fn reconstruct_missing_shards(
        &self,
        available: &[Vec<u8>],
        total_shards: usize,
        data_shards: usize,
    ) -> Result<Vec<Vec<u8>>, RecoveryError> {
        let parity_shards = total_shards - data_shards;
        let codec = cybermanju_erasure::ReedSolomonCodec::new(data_shards, parity_shards)
            .map_err(|e| RecoveryError::ReconstructionError(e.to_string()))?;

        let mut shards: Vec<Option<Vec<u8>>> = Vec::with_capacity(total_shards);
        for i in 0..total_shards {
            if i < available.len() {
                shards.push(Some(available[i].clone()));
            } else {
                shards.push(None);
            }
        }

        codec
            .reconstruct(&mut shards)
            .map_err(|e| RecoveryError::ReconstructionError(e.to_string()))?;

        Ok(shards.into_iter().map(|s| s.unwrap_or_default()).collect())
    }

    /// Generate a quick low-quality preview for recovery verification
    pub fn generate_recovery_preview(
        &self,
        file_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, RecoveryError> {
        let thumbnail = image_utils::create_thumbnail(file_data, width, height, 200)?;
        image_utils::to_jpeg(&thumbnail, 200, 200, 50)
    }

    /// Batch recover multiple files
    pub fn batch_recover(
        &self,
        requests: &[RecoveryRequest],
    ) -> Result<Vec<RecoveryResult>, RecoveryError> {
        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            let chunks = Vec::new(); // Placeholder — in production, load from shards
            let result = self.recover_from_chunks(&chunks, request)?;
            results.push(result);
        }
        Ok(results)
    }

    /// Determine upscale scale factor from target resolution string
    fn determine_scale(&self, target_resolution: &str) -> u32 {
        match target_resolution {
            "r0" => 1,
            "r1" => 2,
            "r2" => 3,
            "r3" => 4,
            _ => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_scale() {
        let pipeline = RecoveryPipeline::new();
        assert_eq!(pipeline.determine_scale("r0"), 1);
        assert_eq!(pipeline.determine_scale("r1"), 2);
        assert_eq!(pipeline.determine_scale("r2"), 3);
        assert_eq!(pipeline.determine_scale("r3"), 4);
    }
}
