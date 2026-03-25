use lz4_flex::{compress_prepend_size, decompress_size_prepended};

const COMPRESSED_PREFIX: u8 = 0x01;
const UNCOMPRESSED_PREFIX: u8 = 0x00;

/// Minimum size to bother compressing (small messages have overhead)
const MIN_COMPRESS_SIZE: usize = 256;

/// Compress data with LZ4 if it's large enough.
/// Returns prefixed data: [prefix_byte] + [payload]
pub fn compress(data: &[u8]) -> Vec<u8> {
    if data.len() < MIN_COMPRESS_SIZE {
        let mut result = Vec::with_capacity(1 + data.len());
        result.push(UNCOMPRESSED_PREFIX);
        result.extend_from_slice(data);
        return result;
    }

    let compressed = compress_prepend_size(data);

    // Only use compressed if it's actually smaller
    if compressed.len() >= data.len() {
        let mut result = Vec::with_capacity(1 + data.len());
        result.push(UNCOMPRESSED_PREFIX);
        result.extend_from_slice(data);
        result
    } else {
        let mut result = Vec::with_capacity(1 + compressed.len());
        result.push(COMPRESSED_PREFIX);
        result.extend_from_slice(&compressed);
        result
    }
}

/// Decompress data. Reads prefix byte to determine format.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Err("Empty data".to_string());
    }

    match data[0] {
        UNCOMPRESSED_PREFIX => Ok(data[1..].to_vec()),
        COMPRESSED_PREFIX => {
            decompress_size_prepended(&data[1..])
                .map_err(|e| format!("LZ4 decompress error: {}", e))
        }
        prefix => Err(format!("Unknown compression prefix: 0x{:02x}", prefix)),
    }
}