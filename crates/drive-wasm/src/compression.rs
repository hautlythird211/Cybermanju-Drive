use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compress_lz4(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

#[wasm_bindgen]
pub fn decompress_lz4(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    lz4_flex::decompress_size_prepended(data)
        .map_err(|e| JsValue::from_str(&format!("LZ4 decompression failed: {}", e)))
}

#[wasm_bindgen]
pub fn compress_brotli(data: &[u8], quality: u32) -> Vec<u8> {
    let mut output = Vec::new();
    let mut writer = brotli::CompressorWriter::new(&mut output, 4096, quality, 22);
    use std::io::Write;
    writer.write_all(data).expect("Brotli write failed");
    drop(writer);
    output
}

#[wasm_bindgen]
pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    use std::io::Read;
    let mut output = Vec::new();
    let mut reader = brotli::Decompressor::new(data, 4096);
    reader
        .read_to_end(&mut output)
        .map_err(|e| JsValue::from_str(&format!("Brotli decompression failed: {}", e)))?;
    Ok(output)
}

#[wasm_bindgen]
pub fn compress_lz4_probe_ratio(data: &[u8]) -> f64 {
    let compressed = lz4_flex::compress_prepend_size(data);
    if data.is_empty() {
        1.0
    } else {
        compressed.len() as f64 / data.len() as f64
    }
}
