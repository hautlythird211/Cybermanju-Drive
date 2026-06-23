use crate::errors::ErasureError;
use rand::Rng;

// GF(2^8) with primitive polynomial 0x11b
const GF_POLY: u16 = 0x11b;

fn build_gf_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0u8; 512];
    let mut log = [0u8; 256];
    let mut x: u16 = 1;
    for (i, item) in exp.iter_mut().enumerate().take(255) {
        *item = x as u8;
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

#[allow(dead_code)]
fn gf_inv(a: u8) -> Result<u8, ErasureError> {
    if a == 0 {
        return Err(ErasureError::ShamirError("cannot invert zero".into()));
    }
    let (exp, _) = &*GF_TABLES;
    Ok(exp[255 - log_lookup(a)])
}

#[allow(dead_code)]
fn log_lookup(a: u8) -> usize {
    let (_, log) = &*GF_TABLES;
    log[a as usize] as usize
}

fn gf_div(a: u8, b: u8) -> Result<u8, ErasureError> {
    if b == 0 {
        return Err(ErasureError::ShamirError("division by zero".into()));
    }
    if a == 0 {
        return Ok(0);
    }
    let (exp, log) = &*GF_TABLES;
    let idx = (log[a as usize] as i32 - log[b as usize] as i32 + 255) % 255;
    Ok(exp[idx as usize])
}

/// Shamir Secret Sharing over GF(2^8)
pub struct ShamirScheme {
    threshold: usize,
    total_shards: usize,
}

impl ShamirScheme {
    pub fn new(threshold: usize, total_shards: usize) -> Result<Self, ErasureError> {
        if threshold == 0 || threshold > 255 {
            return Err(ErasureError::ShamirError("threshold must be 1..255".into()));
        }
        if total_shards < threshold || total_shards > 255 {
            return Err(ErasureError::ShamirError(
                "total_shards must be >= threshold and <= 255".into(),
            ));
        }
        Ok(Self {
            threshold,
            total_shards,
        })
    }

    /// Split secret bytes into `total_shards` shares, any `threshold` can reconstruct
    pub fn split(&self, secret: &[u8]) -> Result<Vec<Vec<u8>>, ErasureError> {
        let mut rng = rand::thread_rng();
        let mut shares: Vec<Vec<u8>> = (0..self.total_shards)
            .map(|_| Vec::with_capacity(secret.len()))
            .collect();

        for &byte in secret {
            // Generate random polynomial: f(x) = byte + c1*x + c2*x^2 + ... + c(t-1)*x^(t-1)
            let mut coeffs: Vec<u8> = Vec::with_capacity(self.threshold);
            coeffs.push(byte);
            for _ in 1..self.threshold {
                coeffs.push(rng.gen());
            }

            // Evaluate at x = 1, 2, ..., total_shards
            for x in 1..=self.total_shards {
                let x_val = x as u8;
                let mut y = 0u8;
                let mut x_power = 1u8;
                for &coeff in &coeffs {
                    y ^= gf_mul(coeff, x_power);
                    x_power = gf_mul(x_power, x_val);
                }
                shares[x - 1].push(y);
            }
        }

        Ok(shares)
    }

    /// Combine `threshold` shares to reconstruct the secret
    pub fn combine(&self, shares: &[Vec<u8>]) -> Result<Vec<u8>, ErasureError> {
        if shares.len() < self.threshold {
            return Err(ErasureError::ShamirError(format!(
                "need {} shares, got {}",
                self.threshold,
                shares.len()
            )));
        }

        let secret_len = shares[0].len();
        for s in shares.iter().take(self.threshold) {
            if s.len() != secret_len {
                return Err(ErasureError::ShamirError(
                    "shares have inconsistent lengths".into(),
                ));
            }
        }

        let mut secret = Vec::with_capacity(secret_len);
        let points: Vec<(u8, &Vec<u8>)> = (0..self.threshold)
            .map(|i| ((i + 1) as u8, &shares[i]))
            .collect();

        for byte_idx in 0..secret_len {
            // Lagrange interpolation at x=0
            let mut y = 0u8;
            for (i, (xi, share_i)) in points.iter().enumerate() {
                let mut basis = 1u8;
                for (j, (xj, _)) in points.iter().enumerate() {
                    if i != j {
                        // basis *= (0 - xj) / (xi - xj) = xj / (xi - xj) in GF(2^8)
                        let num = *xj;
                        // Actually: (0 - xj) = xj in GF(2^8) since -1 = 1
                        // (xi - xj) = xi + xj in GF(2^8)
                        let diff = xi ^ xj;
                        let frac = gf_div(num, diff)?;
                        basis = gf_mul(basis, frac);
                    }
                }
                y ^= gf_mul(share_i[byte_idx], basis);
            }
            secret.push(y);
        }

        Ok(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_combine() {
        let scheme = ShamirScheme::new(3, 5).unwrap();
        let secret = b"Hello, Shamir Secret Sharing!";
        let shares = scheme.split(secret).unwrap();
        assert_eq!(shares.len(), 5);

        // Any 3 shares should reconstruct
        let reconstructed = scheme.combine(&shares[..3]).unwrap();
        assert_eq!(&reconstructed, secret);
    }

    #[test]
    fn test_different_combinations() {
        let scheme = ShamirScheme::new(3, 5).unwrap();
        let secret = b"test secret";
        let shares = scheme.split(secret).unwrap();

        // Try different combinations
        let r1 = scheme
            .combine(&[shares[0].clone(), shares[2].clone(), shares[4].clone()])
            .unwrap();
        assert_eq!(&r1, secret);

        let r2 = scheme
            .combine(&[shares[1].clone(), shares[3].clone(), shares[0].clone()])
            .unwrap();
        assert_eq!(&r2, secret);
    }

    #[test]
    fn test_insufficient_shares() {
        let scheme = ShamirScheme::new(3, 5).unwrap();
        let shares = scheme.split(b"test").unwrap();
        assert!(scheme.combine(&shares[..2]).is_err());
    }
}
