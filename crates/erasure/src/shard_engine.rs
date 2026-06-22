use crate::errors::ErasureError;
use crate::fountain::FountainEncoder;
use crate::reed_solomon::ReedSolomonCodec;

/// Erasure coding engine for shard-level encoding/decoding.
pub struct ShardErasureEngine {
    pub codec: ErasureCodecType,
    pub k: u32,
    pub m: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErasureCodecType {
    ReedSolomon,
    Fountain,
}

impl ShardErasureEngine {
    pub fn new_reed_solomon(k: u32, m: u32) -> Result<Self, ErasureError> {
        if k == 0 || m == 0 {
            return Err(ErasureError::InvalidParameters(
                "k and m must be > 0".into(),
            ));
        }
        Ok(Self {
            codec: ErasureCodecType::ReedSolomon,
            k,
            m,
        })
    }

    pub fn new_fountain(k: u32, m: u32) -> Result<Self, ErasureError> {
        if k == 0 || m == 0 {
            return Err(ErasureError::InvalidParameters(
                "k and m must be > 0".into(),
            ));
        }
        Ok(Self {
            codec: ErasureCodecType::Fountain,
            k,
            m,
        })
    }

    /// Encode data into k data shards + m parity shards.
    pub fn encode(&self, data: &[u8]) -> Result<Vec<Vec<u8>>, ErasureError> {
        match self.codec {
            ErasureCodecType::ReedSolomon => {
                let codec = ReedSolomonCodec::new(self.k as usize, self.m as usize)?;
                codec.encode(data)
            }
            ErasureCodecType::Fountain => {
                let encoder = FountainEncoder::new(data, 1024);
                let mut shards = encoder.encode();
                // Generate additional repair packets
                let repair = encoder.repair_packets(self.m as usize);
                shards.extend(repair);
                Ok(shards)
            }
        }
    }

    /// Reconstruct original data from available shards (some may be None).
    pub fn decode(&self, shards: &[Option<Vec<u8>>]) -> Result<Vec<u8>, ErasureError> {
        match self.codec {
            ErasureCodecType::ReedSolomon => {
                let codec = ReedSolomonCodec::new(self.k as usize, self.m as usize)?;
                let mut shards = shards.to_vec();
                codec.reconstruct(&mut shards)?;
                let data_shards: Vec<Vec<u8>> = shards
                    .into_iter()
                    .take(self.k as usize)
                    .map(|s| s.unwrap_or_default())
                    .collect();
                let total_len: usize = data_shards.iter().map(|s| s.len()).sum();
                let mut result = Vec::with_capacity(total_len);
                for shard in &data_shards {
                    result.extend_from_slice(shard);
                }
                Ok(result)
            }
            ErasureCodecType::Fountain => {
                // Fountain codes: for systematic symbols (first source_symbols from encode()),
                // any k of them suffice. For repair packets, proper LT decoding via
                // Gaussian elimination over GF(2) is needed but not yet implemented.
                let available: Vec<Vec<u8>> = shards
                    .iter()
                    .filter_map(|s| s.as_ref().cloned())
                    .filter(|s| !s.is_empty())
                    .collect();
                if available.len() < self.k as usize {
                    return Err(ErasureError::ReconstructionFailed(format!(
                        "need {} shards, have {}",
                        self.k,
                        available.len()
                    )));
                }
                // Take the first k available shards (systematic symbols decode correctly;
                // repair packets require proper LT decoding which is not yet implemented)
                let mut result = Vec::new();
                for shard in available.iter().take(self.k as usize) {
                    result.extend_from_slice(shard);
                }
                Ok(result)
            }
        }
    }

    /// Generate parity shards from data shards.
    pub fn parity_shards(&self, data_shards: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, ErasureError> {
        if data_shards.len() < self.k as usize {
            return Err(ErasureError::InvalidParameters(format!(
                "need {} data shards, got {}",
                self.k,
                data_shards.len()
            )));
        }
        let combined: Vec<u8> = data_shards.iter().flat_map(|s| s.iter().copied()).collect();
        let encoded = self.encode(&combined)?;
        Ok(encoded.into_iter().skip(self.k as usize).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reed_solomon_encode_decode() {
        let engine = ShardErasureEngine::new_reed_solomon(4, 2).unwrap();
        let data = b"Hello, erasure coding test data that is long enough for multiple shards!!!";
        let encoded = engine.encode(data).unwrap();
        assert_eq!(encoded.len(), 6); // 4 data + 2 parity

        // Simulate losing 2 shards
        let mut shards: Vec<Option<Vec<u8>>> = encoded.into_iter().map(Some).collect();
        shards[4] = None;
        shards[5] = None;

        let decoded = engine.decode(&shards).unwrap();
        assert_eq!(&decoded[..data.len()], data.as_slice());
    }

    #[test]
    fn test_parity_shards() {
        let engine = ShardErasureEngine::new_reed_solomon(4, 2).unwrap();
        let data_shards: Vec<Vec<u8>> = (0..4).map(|i| vec![i; 100]).collect();
        let parity = engine.parity_shards(&data_shards).unwrap();
        assert_eq!(parity.len(), 2);
    }
}
