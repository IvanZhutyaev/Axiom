//! `.axc` v2 — binary instruction encoding (TZ §3.1.5).

use crate::module::{load_axc_v1, save_axc_v1, AxcError, AxcModule, OperatorMeta, AXC_MAGIC};
use crate::opcode::{Instruction, Opcode, Operand};
use std::io::{Read, Write};

pub const AXC_VERSION_V2: u16 = 2;
pub const AXC_MAGIC_V2: &[u8; 4] = b"AXC\x02";

pub fn save_axc_v2(module: &AxcModule, w: &mut impl Write) -> Result<(), AxcError> {
    w.write_all(AXC_MAGIC_V2)?;
    w.write_all(&AXC_VERSION_V2.to_le_bytes())?;
    let meta = serde_json::to_vec(&AxcMetaJson {
        pipeline_name: module.pipeline_name.clone(),
        operators: module.operators.clone(),
        constants: module.constants.clone(),
        sources: module.sources.clone(),
        sinks: module.sinks.clone(),
    })?;
    write_blob(w, &meta)?;
    let code = encode_code(&module.code);
    write_blob(w, &code)?;
    Ok(())
}

fn write_blob(w: &mut impl Write, data: &[u8]) -> Result<(), AxcError> {
    w.write_all(&(data.len() as u32).to_le_bytes())?;
    w.write_all(&crc32fast::hash(data).to_le_bytes())?;
    w.write_all(data)?;
    Ok(())
}

fn read_blob(r: &mut impl Read) -> Result<Vec<u8>, AxcError> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut crc_buf = [0u8; 4];
    r.read_exact(&mut crc_buf)?;
    let expected = u32::from_le_bytes(crc_buf);
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    if crc32fast::hash(&buf) != expected {
        return Err(AxcError::BadMagic);
    }
    Ok(buf)
}

pub fn load_axc(r: &mut impl Read) -> Result<AxcModule, AxcError> {
    let mut data = Vec::new();
    r.read_to_end(&mut data)?;
    if data.starts_with(AXC_MAGIC_V2) {
        return load_axc_v2_bytes(&data);
    }
    if data.starts_with(AXC_MAGIC) {
        return load_axc_v1(&mut data.as_slice());
    }
    Err(AxcError::BadMagic)
}

fn load_axc_v2_bytes(data: &[u8]) -> Result<AxcModule, AxcError> {
    let mut cur = &data[6..];
    let meta_buf = read_blob(&mut cur)?;
    let meta: AxcMetaJson = serde_json::from_slice(&meta_buf)?;
    let code_buf = read_blob(&mut cur)?;
    Ok(AxcModule {
        version: AXC_VERSION_V2,
        pipeline_name: meta.pipeline_name,
        operators: meta.operators,
        code: decode_code(&code_buf),
        constants: meta.constants,
        sources: meta.sources,
        sinks: meta.sinks,
    })
}

pub fn save_axc(module: &AxcModule, w: &mut impl Write) -> Result<(), AxcError> {
    if module.version >= AXC_VERSION_V2 {
        save_axc_v2(module, w)
    } else {
        save_axc_v1(module, w)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AxcMetaJson {
    pipeline_name: String,
    operators: Vec<OperatorMeta>,
    constants: Vec<String>,
    sources: Vec<String>,
    sinks: Vec<String>,
}

fn encode_code(code: &[Instruction]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(code.len() as u32).to_le_bytes());
    for ins in code {
        buf.push(ins.op.to_u8());
        let (tag, payload) = encode_operand(ins.operand.as_ref());
        buf.push(tag);
        buf.extend_from_slice(&payload);
    }
    buf
}

fn decode_code(buf: &[u8]) -> Vec<Instruction> {
    if buf.len() < 4 {
        return Vec::new();
    }
    let count = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;
    let mut out = Vec::with_capacity(count);
    let mut pos = 4;
    for _ in 0..count {
        if pos >= buf.len() {
            break;
        }
        let op = Opcode::from_u8(buf[pos]).unwrap_or(Opcode::Halt);
        pos += 1;
        if pos >= buf.len() {
            out.push(Instruction::new(op));
            break;
        }
        let tag = buf[pos];
        pos += 1;
        let (operand, n) = decode_operand(tag, &buf[pos..]);
        pos += n;
        out.push(if let Some(o) = operand {
            Instruction::with_operand(op, o)
        } else {
            Instruction::new(op)
        });
    }
    out
}

fn encode_operand(op: Option<&Operand>) -> (u8, Vec<u8>) {
    match op {
        None => (0, vec![]),
        Some(Operand::I64(v)) => (1, v.to_le_bytes().to_vec()),
        Some(Operand::F64(v)) => (2, v.to_le_bytes().to_vec()),
        Some(Operand::Bool(v)) => (3, vec![u8::from(*v)]),
        Some(Operand::U32(v)) => (4, v.to_le_bytes().to_vec()),
        Some(Operand::U16(v)) => (5, v.to_le_bytes().to_vec()),
        Some(Operand::U8(v)) => (6, vec![*v]),
        Some(Operand::Str(s)) => {
            let b = s.as_bytes();
            let mut p = (b.len() as u32).to_le_bytes().to_vec();
            p.extend_from_slice(b);
            (7, p)
        }
    }
}

fn decode_operand(tag: u8, buf: &[u8]) -> (Option<Operand>, usize) {
    match tag {
        0 => (None, 0),
        1 if buf.len() >= 8 => (
            Some(Operand::I64(i64::from_le_bytes(buf[0..8].try_into().unwrap()))),
            8,
        ),
        2 if buf.len() >= 8 => (
            Some(Operand::F64(f64::from_le_bytes(buf[0..8].try_into().unwrap()))),
            8,
        ),
        3 if !buf.is_empty() => (Some(Operand::Bool(buf[0] != 0)), 1),
        4 if buf.len() >= 4 => (
            Some(Operand::U32(u32::from_le_bytes(buf[0..4].try_into().unwrap()))),
            4,
        ),
        7 if buf.len() >= 4 => {
            let len = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;
            if buf.len() < 4 + len {
                return (None, 0);
            }
            let s = std::str::from_utf8(&buf[4..4 + len])
                .unwrap_or("")
                .to_string();
            (Some(Operand::Str(s)), 4 + len)
        }
        _ => (None, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_roundtrip() {
        let mut m = AxcModule::default();
        m.version = AXC_VERSION_V2;
        m.code.push(Instruction::with_operand(Opcode::Push, Operand::F64(1.5)));
        m.code.push(Instruction::new(Opcode::Halt));
        let mut buf = Vec::new();
        save_axc_v2(&m, &mut buf).unwrap();
        let loaded = load_axc(&mut buf.as_slice()).unwrap();
        assert_eq!(loaded.code.len(), 2);
    }
}
