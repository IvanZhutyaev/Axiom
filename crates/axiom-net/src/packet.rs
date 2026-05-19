//! Wire packet format per docs/formats/wire-protocol.md.

use thiserror::Error;

pub const HEADER_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PacketFlags(u8);

impl PacketFlags {
    pub const SYN: Self = Self(0b0000_0001);
    pub const ACK: Self = Self(0b0000_0010);
    pub const FIN: Self = Self(0b0000_0100);
    pub const DATA: Self = Self(0b0000_1000);
    pub const NACK: Self = Self(0b0001_0000);

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for PacketFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub flags: PacketFlags,
    pub conn_id: u16,
    pub seq: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("truncated packet")]
    Truncated,
    #[error("crc mismatch")]
    Crc,
}

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        let len = self.payload.len() as u16;
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        buf.push(self.flags.bits());
        buf.extend_from_slice(&self.conn_id.to_le_bytes());
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&len.to_le_bytes());
        let crc = crc32fast::hash(&self.payload);
        buf.extend_from_slice(&crc.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, PacketError> {
        if data.len() < HEADER_SIZE {
            return Err(PacketError::Truncated);
        }
        let flags = PacketFlags::from_bits_truncate(data[0]);
        let conn_id = u16::from_le_bytes(data[1..3].try_into().unwrap());
        let seq = u32::from_le_bytes(data[3..7].try_into().unwrap());
        let len = u16::from_le_bytes(data[7..9].try_into().unwrap()) as usize;
        let expected_crc = u32::from_le_bytes(data[9..13].try_into().unwrap());
        if data.len() < HEADER_SIZE + len {
            return Err(PacketError::Truncated);
        }
        let payload = data[HEADER_SIZE..HEADER_SIZE + len].to_vec();
        if crc32fast::hash(&payload) != expected_crc {
            return Err(PacketError::Crc);
        }
        Ok(Packet {
            flags,
            conn_id,
            seq,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_roundtrip() {
        let p = Packet {
            flags: PacketFlags::DATA | PacketFlags::ACK,
            conn_id: 7,
            seq: 42,
            payload: b"hello".to_vec(),
        };
        let enc = p.encode();
        let dec = Packet::decode(&enc).unwrap();
        assert_eq!(dec, p);
    }
}
