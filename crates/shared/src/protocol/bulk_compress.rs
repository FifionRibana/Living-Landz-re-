/// Compress data with LZ4. All bulk channel payloads MUST use this.
pub fn compress(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

/// Decompress LZ4 data.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    lz4_flex::decompress_size_prepended(data).map_err(|e| format!("LZ4 decompress error: {}", e))
}
