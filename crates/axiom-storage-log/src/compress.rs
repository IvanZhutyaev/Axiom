//! Segment payload compression (lz4 / snappy).

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None,
    Lz4,
    Snappy,
}

#[derive(Debug, Error)]
pub enum CompressError {
    #[error("lz4: {0}")]
    Lz4(String),
    #[error("snappy: {0}")]
    Snappy(String),
}

pub fn compress(data: &[u8], algo: Compression) -> Result<Vec<u8>, CompressError> {
    match algo {
        Compression::None => Ok(data.to_vec()),
        Compression::Lz4 => lz4_flex::compress_prepend_size(data)
            .map_err(|e| CompressError::Lz4(e.to_string())),
        Compression::Snappy => Ok(snap::raw::Encoder::new()
            .compress_vec(data)
            .map_err(|e| CompressError::Snappy(e.to_string()))?),
    }
}

pub fn decompress(data: &[u8], algo: Compression) -> Result<Vec<u8>, CompressError> {
    match algo {
        Compression::None => Ok(data.to_vec()),
        Compression::Lz4 => lz4_flex::decompress_size_prepended(data)
            .map_err(|e| CompressError::Lz4(e.to_string())),
        Compression::Snappy => snap::raw::Decoder::new()
            .decompress_vec(data)
            .map_err(|e| CompressError::Snappy(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lz4_roundtrip() {
        let raw = b"hello axiom log compression";
        let c = compress(raw, Compression::Lz4).unwrap();
        let d = decompress(&c, Compression::Lz4).unwrap();
        assert_eq!(d, raw);
    }
}
