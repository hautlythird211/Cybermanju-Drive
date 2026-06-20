use serde::{Deserialize, Serialize};

/// Resolution levels for the file decomposition
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ResolutionLevel {
    R0,
    R1,
    R2,
    R3,
}

impl ResolutionLevel {
    pub fn as_index(&self) -> usize {
        match self {
            ResolutionLevel::R0 => 0,
            ResolutionLevel::R1 => 1,
            ResolutionLevel::R2 => 2,
            ResolutionLevel::R3 => 3,
        }
    }

    pub fn max_dimension(&self) -> u32 {
        match self {
            ResolutionLevel::R0 => 200,
            ResolutionLevel::R1 => 640,
            ResolutionLevel::R2 => 1920,
            ResolutionLevel::R3 => u32::MAX,
        }
    }

    pub fn lower_resolution(&self) -> Option<ResolutionLevel> {
        match self {
            ResolutionLevel::R0 => None,
            ResolutionLevel::R1 => Some(ResolutionLevel::R0),
            ResolutionLevel::R2 => Some(ResolutionLevel::R1),
            ResolutionLevel::R3 => Some(ResolutionLevel::R2),
        }
    }

    pub fn size_ratio_to(&self, target: ResolutionLevel) -> f64 {
        let self_dim = self.max_dimension() as f64;
        let target_dim = target.max_dimension() as f64;
        if target_dim == 0.0 || target_dim == f64::INFINITY {
            1.0
        } else {
            self_dim / target_dim
        }
    }
}

/// Key tier determines which encryption key encrypts a blob
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum KeyTier {
    Preview,
    Content,
}

/// Shard type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ShardType {
    Content,
    Parity,
    Preview,
}

/// Erasure coding codec type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErasureCodecType {
    #[serde(rename = "reed-solomon")]
    ReedSolomon,
    #[serde(rename = "clay-codes")]
    ClayCodes,
    #[serde(rename = "fountain-raptorq")]
    FountainRaptorq,
    #[serde(rename = "shamir")]
    Shamir,
}

/// Parameters for erasure coding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErasureParams {
    pub k: u32,
    pub m: u32,
    pub d: u32,
}

/// Fountain code configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FountainConfig {
    pub symbol_size: u32,
    pub source_symbols: u32,
    pub repair_symbols_per_shard: u32,
    pub min_packets_for_recovery: u32,
}

/// Recovery threshold information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryThreshold {
    pub data_shards_needed: u32,
    pub total_shards_available: u32,
    pub can_recover_with: Vec<String>,
}
