use crate::errors::ErasureError;

// GF(2^8) with primitive polynomial 0x11b
const GF_POLY: u16 = 0x11b;

fn build_gf_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0u8; 512];
    let mut log = [0u8; 256];
    let mut x: u16 = 1;
    for i in 0..255 {
        exp[i] = x as u8;
        log[x as usize] = i as u8;
        x <<= 1;
        if x & 256 != 0 {
            x ^= GF_POLY;
        }
    }
    for i in 255..512 {
        exp[i] = exp[i - 255];
    }
    (exp, log)
}

static GF_TABLES: once_cell::sync::Lazy<([u8; 512], [u8; 256])> =
    once_cell::sync::Lazy::new(build_gf_tables);

fn gf_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let (exp, log) = &*GF_TABLES;
    let idx = log[a as usize] as usize + log[b as usize] as usize;
    exp[idx % 255]
}

fn gf_pow(mut a: u8, mut exp: u32) -> u8 {
    if exp == 0 {
        return 1;
    }
    let mut result = 1u8;
    let (gf_exp, gf_log) = &*GF_TABLES;
    while exp > 0 {
        if exp & 1 == 1 {
            if result == 0 || a == 0 {
                result = 0;
            } else {
                let idx = gf_log[result as usize] as usize + gf_log[a as usize] as usize;
                result = gf_exp[idx % 255];
            }
        }
        exp >>= 1;
        if exp > 0 {
            if a == 0 {
                a = 0;
            } else {
                let idx = gf_log[a as usize] as usize * 2;
                a = gf_exp[idx % 255];
            }
        }
    }
    result
}

fn gf_inv(a: u8) -> Result<u8, ErasureError> {
    if a == 0 {
        return Err(ErasureError::EncodingFailed("cannot invert zero".into()));
    }
    Ok(gf_pow(a, 254))
}

fn gf_div(a: u8, b: u8) -> Result<u8, ErasureError> {
    if b == 0 {
        return Err(ErasureError::EncodingFailed(
            "division by zero in GF(2^8)".into(),
        ));
    }
    Ok(gf_mul(a, gf_inv(b)?))
}

/// Reed-Solomon codec over GF(2^8)
pub struct ReedSolomonCodec {
    data_shards: usize,
    parity_shards: usize,
    total_shards: usize,
    /// Generator polynomial coefficients (excluding leading x^n term)
    gen_poly: Vec<u8>,
}

impl ReedSolomonCodec {
    pub fn new(data_shards: usize, parity_shards: usize) -> Result<Self, ErasureError> {
        if data_shards == 0 || parity_shards == 0 {
            return Err(ErasureError::InvalidParameters(
                "data_shards and parity_shards must be > 0".into(),
            ));
        }
        let total = data_shards + parity_shards;
        if total > 255 {
            return Err(ErasureError::InvalidParameters(
                "total shards cannot exceed 255 for GF(2^8)".into(),
            ));
        }

        // Build generator polynomial: g(x) = (x - α^0)(x - α^1)...(x - α^(m-1))
        let mut gen = vec![1u8];
        for i in 0..parity_shards {
            let alpha_i = {
                let (exp, _) = &*GF_TABLES;
                exp[i]
            };
            // Multiply gen by (x - alpha_i)
            let mut new_gen = vec![0u8; gen.len() + 1];
            for j in 0..gen.len() {
                new_gen[j] ^= gen[j];
                new_gen[j + 1] ^= gf_mul(gen[j], alpha_i);
            }
            gen = new_gen;
        }

        Ok(Self {
            data_shards,
            parity_shards,
            total_shards: total,
            gen_poly: gen,
        })
    }

    /// Split data into equal-sized data shards (pad last with zeros)
    fn split_data(data: &[u8]) -> Vec<Vec<u8>> {
        // This is called externally, we keep it as a utility
        let chunk_size =
            (data.len() + Self::num_shards_hint(data) - 1) / Self::num_shards_hint(data);
        Self::split_data_with_count(data, Self::num_shards_hint(data), chunk_size)
    }

    fn num_shards_hint(data: &[u8]) -> usize {
        // We need the caller to specify; this is a helper
        // In practice, split_data_with_count is used
        (data.len() / 1024).max(4).min(255)
    }

    /// Split data into `num_shards` chunks of `chunk_size` bytes each
    pub fn split_data_with_count(
        data: &[u8],
        num_shards: usize,
        chunk_size: usize,
    ) -> Vec<Vec<u8>> {
        let mut shards: Vec<Vec<u8>> = (0..num_shards)
            .map(|i| {
                let start = i * chunk_size;
                let end = ((i + 1) * chunk_size).min(data.len());
                let mut shard = if start < data.len() {
                    data[start..end].to_vec()
                } else {
                    vec![]
                };
                shard.resize(chunk_size, 0);
                shard
            })
            .collect();
        shards
    }

    /// Concatenate shards back into original data, stripping padding
    pub fn concat_data(shards: &[Vec<u8>], original_len: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(original_len);
        for shard in shards {
            for &byte in shard {
                result.push(byte);
            }
        }
        result.truncate(original_len);
        result
    }

    /// Encode data: split into data shards and compute parity shards
    pub fn encode(&self, data: &[u8]) -> Result<Vec<Vec<u8>>, ErasureError> {
        let chunk_size = (data.len() + self.data_shards - 1) / self.data_shards;
        let data_shards = Self::split_data_with_count(data, self.data_shards, chunk_size);
        let mut result = data_shards.clone();

        // Compute parity shards
        for p in 0..self.parity_shards {
            let mut parity = vec![0u8; chunk_size];
            for i in 0..self.data_shards {
                let alpha_ip = {
                    let (exp, _) = &*GF_TABLES;
                    exp[(i * p) % 255]
                };
                for j in 0..chunk_size {
                    parity[j] ^= gf_mul(data_shards[i][j], alpha_ip);
                }
            }
            result.push(parity);
        }

        Ok(result)
    }

    /// Reconstruct missing shards from available ones
    pub fn reconstruct(&self, shards: &mut [Option<Vec<u8>>]) -> Result<(), ErasureError> {
        let n = shards.len();
        if n != self.total_shards {
            return Err(ErasureError::InvalidParameters(format!(
                "expected {} shards, got {}",
                self.total_shards, n
            )));
        }

        let chunk_size = shards
            .iter()
            .filter_map(|s| s.as_ref())
            .next()
            .map(|s| s.len())
            .unwrap_or(0);

        // Find missing indices
        let missing: Vec<usize> = shards
            .iter()
            .enumerate()
            .filter_map(|(i, s)| if s.is_none() { Some(i) } else { None })
            .collect();

        if missing.is_empty() {
            return Ok(());
        }

        // For each missing shard, reconstruct using Gaussian elimination over GF(2^8)
        let (exp, _) = &*GF_TABLES;

        for &miss_idx in &missing {
            // Build the system: for each byte position in the chunk
            for byte_pos in 0..chunk_size {
                // Build decoding matrix for this byte position
                let available: Vec<(usize, Vec<u8>)> = shards
                    .iter()
                    .enumerate()
                    .filter_map(|(i, s)| s.as_ref().map(|data| (i, data.to_vec())))
                    .collect();

                if available.len() < self.data_shards {
                    return Err(ErasureError::ReconstructionFailed(
                        "not enough shards to reconstruct".into(),
                    ));
                }

                // Vandermonde matrix: row i corresponds to shard at index available[i].0
                // M[i][j] = alpha^(available[i].0 * j) for j=0..data_shards-1
                // rhs[i] = shard_data[i][byte_pos]
                let mut matrix: Vec<Vec<u8>> = Vec::new();
                let mut rhs: Vec<u8> = Vec::new();

                for (shard_idx, data) in &available {
                    let mut row = Vec::new();
                    for j in 0..self.data_shards {
                        row.push(exp[(*shard_idx * j) % 255]);
                    }
                    matrix.push(row);
                    rhs.push(data[byte_pos]);
                }

                // Gaussian elimination
                let rows = matrix.len();
                let cols = self.data_shards;

                // Forward elimination
                for col in 0..cols.min(rows) {
                    // Find pivot
                    let pivot_row =
                        (col..rows).find(|&r| matrix[r][col] != 0).ok_or_else(|| {
                            ErasureError::ReconstructionFailed("singular matrix".into())
                        })?;

                    // Swap
                    matrix.swap(col, pivot_row);
                    rhs.swap(col, pivot_row);

                    // Eliminate below
                    let pivot_val = matrix[col][col];
                    for row in (col + 1)..rows {
                        if matrix[row][col] != 0 {
                            let factor = gf_div(matrix[row][col], pivot_val)?;
                            for c in col..cols {
                                matrix[row][c] ^= gf_mul(factor, matrix[col][c]);
                            }
                            rhs[row] ^= gf_mul(factor, rhs[col]);
                        }
                    }
                }

                // Back substitution
                let mut solution = vec![0u8; cols];
                for i in (0..cols.min(rows)).rev() {
                    let mut sum = rhs[i];
                    for j in (i + 1)..cols {
                        sum ^= gf_mul(solution[j], matrix[i][j]);
                    }
                    solution[i] = gf_div(sum, matrix[i][i])?;
                }

                // The missing byte is solution[?] — we need to figure out which coefficient
                // corresponds to the missing shard index
                // Actually: solution[j] is the coefficient for the j-th data shard
                // We need to evaluate the polynomial at miss_idx
                let mut missing_byte = 0u8;
                for j in 0..self.data_shards {
                    missing_byte ^= gf_mul(solution[j], exp[(miss_idx * j) % 255]);
                }

                // Set the byte in the missing shard
                if let Some(ref mut data) = shards[miss_idx] {
                    if byte_pos < data.len() {
                        data[byte_pos] = missing_byte;
                    }
                } else {
                    let mut data = vec![0u8; chunk_size];
                    data[byte_pos] = missing_byte;
                    shards[miss_idx] = Some(data);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf_arithmetic() {
        let a = 5u8;
        let b = 3u8;
        let c = gf_mul(a, b);
        assert_eq!(gf_div(c, b).unwrap(), a);
    }

    #[test]
    fn test_encode_decode() {
        let rs = ReedSolomonCodec::new(4, 2).unwrap();
        let data = b"Hello, Reed-Solomon erasure coding!";
        let encoded = rs.encode(data).unwrap();
        assert_eq!(encoded.len(), 6);

        // Simulate losing 2 shards
        let mut shards: Vec<Option<Vec<u8>>> = encoded.into_iter().map(Some).collect();
        shards[4] = None;
        shards[5] = None;

        rs.reconstruct(&mut shards).unwrap();

        let recovered: Vec<Vec<u8>> = shards.into_iter().map(|s| s.unwrap()).collect();
        let result = ReedSolomonCodec::concat_data(&recovered[..4], data.len());
        assert_eq!(&result[..data.len()], data.as_slice());
    }
}
