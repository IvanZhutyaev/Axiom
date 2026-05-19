//! `.axc` binary container (versioned, backward-compatible header).

use crate::opcode::Instruction;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use thiserror::Error;

pub const AXC_MAGIC: &[u8; 4] = b"AXC\x01";
pub const AXC_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperatorMeta {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AxcModule {
    pub version: u16,
    pub pipeline_name: String,
    pub operators: Vec<OperatorMeta>,
    pub code: Vec<Instruction>,
    pub constants: Vec<String>,
    pub sources: Vec<String>,
    pub sinks: Vec<String>,
}

impl Default for AxcModule {
    fn default() -> Self {
        Self {
            version: AXC_VERSION,
            pipeline_name: "main".into(),
            operators: Vec::new(),
            code: Vec::new(),
            constants: Vec::new(),
            sources: Vec::new(),
            sinks: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AxcError {
    #[error("invalid magic")]
    BadMagic,
    #[error("unsupported version {0}")]
    UnsupportedVersion(u16),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn save_axc_v1(module: &AxcModule, w: &mut impl Write) -> Result<(), AxcError> {
    w.write_all(AXC_MAGIC)?;
    w.write_all(&module.version.to_le_bytes())?;
    let payload = serde_json::to_vec(module)?;
    let len = payload.len() as u32;
    w.write_all(&len.to_le_bytes())?;
    let crc = crc32fast::hash(&payload);
    w.write_all(&crc.to_le_bytes())?;
    w.write_all(&payload)?;
    Ok(())
}

pub fn load_axc_v1(r: &mut impl Read) -> Result<AxcModule, AxcError> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != AXC_MAGIC {
        return Err(AxcError::BadMagic);
    }
    let mut ver_buf = [0u8; 2];
    r.read_exact(&mut ver_buf)?;
    let version = u16::from_le_bytes(ver_buf);
    if version > AXC_VERSION {
        return Err(AxcError::UnsupportedVersion(version));
    }
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut crc_buf = [0u8; 4];
    r.read_exact(&mut crc_buf)?;
    let expected_crc = u32::from_le_bytes(crc_buf);
    let mut payload = vec![0u8; len];
    r.read_exact(&mut payload)?;
    if crc32fast::hash(&payload) != expected_crc {
        return Err(AxcError::BadMagic);
    }
    let module: AxcModule = serde_json::from_slice(&payload)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode::{Instruction, Opcode};

    #[test]
    fn roundtrip_axc() {
        let mut m = AxcModule::default();
        m.pipeline_name = "sensor".into();
        m.code.push(Instruction::new(Opcode::Halt));
        let mut buf = Vec::new();
        save_axc_v1(&m, &mut buf).unwrap();
        let decoded = load_axc_v1(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded.pipeline_name, "sensor");
    }
}
