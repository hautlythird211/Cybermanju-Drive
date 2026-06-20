use serde::{Deserialize, Serialize};

use crate::errors::RecoveryError;
use crate::image_utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpscaleModel {
    RealEsrganX2,
    RealEsrganX4,
    RealCugan,
    LanczosFallback,
}

pub struct NeuralUpscaler {
    model: UpscaleModel,
    model_path: Option<String>,
}

impl NeuralUpscaler {
    pub fn new(model: UpscaleModel) -> Self {
        Self { model, model_path: None }
    }

    pub fn with_model_path(model: UpscaleModel, path: &str) -> Self {
        Self { model, model_path: Some(path.to_string()) }
    }

    /// Upscale image data by the given scale factor
    pub fn upscale(&self, input: &[u8], width: u32, height: u32, scale: u32) -> Result<Vec<u8>, RecoveryError> {
        let target_w = width * scale;
        let target_h = height * scale;

        match &self.model {
            UpscaleModel::LanczosFallback => {
                image_utils::resize_lanczos(input, width, height, target_w, target_h, 4)
            }
            UpscaleModel::RealEsrganX2 | UpscaleModel::RealEsrganX4 => {
                // In production, this would load ONNX model and run inference
                // For now, fall back to Lanczos with a quality hint
                if let Some(ref path) = self.model_path {
                    self.run_onnx_model(input, width, height, scale, path)
                } else {
                    // No model file — fall back to Lanczos
                    image_utils::resize_lanczos(input, width, height, target_w, target_h, 4)
                }
            }
            UpscaleModel::RealCugan => {
                if let Some(ref path) = self.model_path {
                    self.run_onnx_model(input, width, height, scale, path)
                } else {
                    image_utils::resize_lanczos(input, width, height, target_w, target_h, 4)
                }
            }
        }
    }

    /// Upscale only a specific region (tile-based processing)
    pub fn upscale_region(
        &self,
        input: &[u8],
        width: u32,
        height: u32,
        x: u32,
        y: u32,
        region_w: u32,
        region_h: u32,
        scale: u32,
    ) -> Result<Vec<u8>, RecoveryError> {
        // Extract the region
        let mut region_data = Vec::with_capacity((region_w * region_h * 4) as usize);
        for row in y..(y + region_h).min(height) {
            let offset = (row * width + x) * 4;
            let end = (offset + region_w * 4).min(input.len() as u32);
            if (offset as usize) < input.len() {
                let slice_end = (end as usize).min(input.len());
                region_data.extend_from_slice(&input[offset as usize..slice_end]);
            }
        }

        self.upscale(&region_data, region_w, region_h, scale)
    }

    /// Get supported scale factors for this model
    pub fn get_supported_scales(&self) -> Vec<u32> {
        match &self.model {
            UpscaleModel::RealEsrganX2 => vec![2],
            UpscaleModel::RealEsrganX4 => vec![4],
            UpscaleModel::RealCugan => vec![2, 3, 4],
            UpscaleModel::LanczosFallback => vec![2, 3, 4, 8],
        }
    }

    /// Estimate processing time in milliseconds
    pub fn estimate_processing_time(&self, width: u32, height: u32, scale: u32) -> u64 {
        let pixels = (width * height) as u64;
        let scaled_pixels = pixels * scale as u64 * scale as u64;

        match &self.model {
            UpscaleModel::LanczosFallback => {
                // Lanczos is fast: ~1ms per megapixel
                scaled_pixels / 1_000_000
            }
            UpscaleModel::RealEsrganX2 | UpscaleModel::RealEsrganX4 => {
                // Neural upscale: ~50ms per megapixel on CPU
                scaled_pixels / 20_000
            }
            UpscaleModel::RealCugan => {
                // Real-CUGAN: ~80ms per megapixel on CPU
                scaled_pixels / 12_500
            }
        }
    }

    /// Run ONNX model inference (placeholder for actual implementation)
    fn run_onnx_model(
        &self,
        input: &[u8],
        width: u32,
        height: u32,
        scale: u32,
        model_path: &str,
    ) -> Result<Vec<u8>, RecoveryError> {
        // In production, this would use ort crate to load and run the ONNX model
        // For now, fall back to Lanczos
        let _ = (model_path, width, height, scale);
        image_utils::resize_lanczos(input, width, height, width * scale, height * scale, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_scales() {
        let upscaler = NeuralUpscaler::new(UpscaleModel::LanczosFallback);
        assert_eq!(upscaler.get_supported_scales(), vec![2, 3, 4, 8]);

        let upscaler = NeuralUpscaler::new(UpscaleModel::RealEsrganX4);
        assert_eq!(upscaler.get_supported_scales(), vec![4]);
    }

    #[test]
    fn test_estimate_processing_time() {
        let upscaler = NeuralUpscaler::new(UpscaleModel::LanczosFallback);
        let time = upscaler.estimate_processing_time(1920, 1080, 2);
        assert!(time < 1000); // Should be fast
    }
}
