use anyhow::Result;

/// Train a Zstd dictionary from sample data for improved compression.
pub fn train_dictionary(samples: &[Vec<u8>], dict_size: usize) -> Result<Vec<u8>> {
    if samples.is_empty() {
        return Err(anyhow::anyhow!("need at least one sample"));
    }
    let dict = zstd::dict::from_samples(samples, dict_size)?;
    Ok(dict)
}

/// Compress data using a pre-trained Zstd dictionary.
pub fn compress_with_dict(data: &[u8], dict: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut encoder = zstd::Encoder::with_dictionary(Vec::new(), level, dict)?;
    std::io::Write::write_all(&mut encoder, data)?;
    Ok(encoder.finish()?)
}

/// Decompress data using a pre-trained Zstd dictionary.
pub fn decompress_with_dict(data: &[u8], dict: &[u8]) -> Result<Vec<u8>> {
    let cursor = std::io::Cursor::new(data);
    let mut decoder = zstd::Decoder::with_dictionary(cursor, dict)?;
    let mut output = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut output)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_train_and_compress() {
        let samples: Vec<Vec<u8>> = (0..10)
            .map(|i| format!("sample data {} with repeated content and common patterns", i).into_bytes())
            .collect();
        let dict = train_dictionary(&samples, 1024).unwrap();
        assert!(!dict.is_empty());

        let data = b"sample data with repeated content and common patterns";
        let compressed = compress_with_dict(data, &dict, 3).unwrap();
        let decompressed = decompress_with_dict(&compressed, &dict).unwrap();
        assert_eq!(&decompressed, data);
    }
}
