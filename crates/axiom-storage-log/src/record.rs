//! Log record format: offset (8) + length (4) + crc (4) + payload.

use thiserror::Error;

pub const RECORD_HEADER_SIZE: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    pub offset: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum RecordError {
    #[error("crc mismatch")]
    CrcMismatch,
    #[error("truncated record")]
    Truncated,
}

impl LogRecord {
    pub fn encode(&self) -> Vec<u8> {
        let len = self.payload.len() as u32;
        let crc = crc32fast::hash(&self.payload);
        let mut buf = Vec::with_capacity(RECORD_HEADER_SIZE + self.payload.len());
        buf.extend_from_slice(&self.offset.to_le_bytes());
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&crc.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn decode(mut data: &[u8]) -> Result<Self, RecordError> {
        if data.len() < RECORD_HEADER_SIZE {
            return Err(RecordError::Truncated);
        }
        let offset = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let expected_crc = u32::from_le_bytes(data[12..16].try_into().unwrap());
        data = &data[16..];
        if data.len() < len {
            return Err(RecordError::Truncated);
        }
        let payload = data[..len].to_vec();
        if crc32fast::hash(&payload) != expected_crc {
            return Err(RecordError::CrcMismatch);
        }
        Ok(LogRecord { offset, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_roundtrip() {
        let r = LogRecord {
            offset: 1,
            payload: b"event".to_vec(),
        };
        let enc = r.encode();
        let dec = LogRecord::decode(&enc).unwrap();
        assert_eq!(dec, r);
    }
}
