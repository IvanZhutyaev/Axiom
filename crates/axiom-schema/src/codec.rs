//! Binary encoding: schema id prefix + varint/zigzag payload.

use crate::registry::SchemaType;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("unsupported type")]
    Unsupported,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("truncated input")]
    Truncated,
    #[error("unsupported type")]
    Unsupported,
}

pub fn encode(schema_id: u32, value: &Value) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&schema_id.to_le_bytes());
    encode_value(value, &mut buf)?;
    Ok(buf)
}

pub fn decode(data: &[u8]) -> Result<(u32, Value), DecodeError> {
    if data.len() < 4 {
        return Err(DecodeError::Truncated);
    }
    let schema_id = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let (value, _) = decode_value(&data[4..])?;
    Ok((schema_id, value))
}

fn encode_value(v: &Value, buf: &mut Vec<u8>) -> Result<(), EncodeError> {
    match v {
        Value::Null => {
            buf.push(0);
        }
        Value::Bool(b) => {
            buf.push(1);
            buf.push(u8::from(*b));
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                buf.push(2);
                write_varint(zigzag(i), buf);
            } else if let Some(f) = n.as_f64() {
                buf.push(3);
                buf.extend_from_slice(&f.to_le_bytes());
            } else {
                return Err(EncodeError::Unsupported);
            }
        }
        Value::String(s) => {
            buf.push(4);
            let b = s.as_bytes();
            write_varint(b.len() as u64, buf);
            buf.extend_from_slice(b);
        }
        Value::Array(arr) => {
            buf.push(5);
            write_varint(arr.len() as u64, buf);
            for item in arr {
                encode_value(item, buf)?;
            }
        }
        Value::Object(obj) => {
            buf.push(6);
            write_varint(obj.len() as u64, buf);
            for (k, v) in obj {
                let kb = k.as_bytes();
                write_varint(kb.len() as u64, buf);
                buf.extend_from_slice(kb);
                encode_value(v, buf)?;
            }
        }
    }
    Ok(())
}

fn decode_value(data: &[u8]) -> Result<(Value, usize), DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::Truncated);
    }
    let mut pos = 1;
    let v = match data[0] {
        0 => Value::Null,
        1 => {
            if data.len() < 2 {
                return Err(DecodeError::Truncated);
            }
            pos = 2;
            Value::Bool(data[1] != 0)
        }
        2 => {
            let (n, used) = read_varint(&data[1..])?;
            pos = 1 + used;
            Value::Number(serde_json::Number::from(unzigzag(n)))
        }
        3 => {
            if data.len() < 1 + 8 {
                return Err(DecodeError::Truncated);
            }
            let f = f64::from_le_bytes(data[1..9].try_into().unwrap());
            pos = 9;
            Value::Number(
                serde_json::Number::from_f64(f).ok_or(DecodeError::Unsupported)?,
            )
        }
        4 => {
            let (len, u1) = read_varint(&data[1..])?;
            let start = 1 + u1;
            let end = start + len as usize;
            if data.len() < end {
                return Err(DecodeError::Truncated);
            }
            let s = std::str::from_utf8(&data[start..end]).map_err(|_| DecodeError::Unsupported)?;
            pos = end;
            Value::String(s.to_string())
        }
        5 => {
            let (len, u1) = read_varint(&data[1..])?;
            let mut off = 1 + u1;
            let mut items = Vec::new();
            for _ in 0..len {
                let (item, used) = decode_value(&data[off..])?;
                off += used;
                items.push(item);
            }
            pos = off;
            Value::Array(items)
        }
        6 => {
            let (len, u1) = read_varint(&data[1..])?;
            let mut off = 1 + u1;
            let mut map = serde_json::Map::new();
            for _ in 0..len {
                let (klen, u2) = read_varint(&data[off..])?;
                off += u2;
                let k = std::str::from_utf8(&data[off..off + klen as usize])
                    .map_err(|_| DecodeError::Unsupported)?
                    .to_string();
                off += klen as usize;
                let (val, u3) = decode_value(&data[off..])?;
                off += u3;
                map.insert(k, val);
            }
            pos = off;
            Value::Object(map)
        }
        _ => return Err(DecodeError::Unsupported),
    };
    Ok((v, pos))
}

fn write_varint(mut n: u64, buf: &mut Vec<u8>) {
    while n >= 0x80 {
        buf.push((n as u8) | 0x80);
        n >>= 7;
    }
    buf.push(n as u8);
}

fn read_varint(data: &[u8]) -> Result<(u64, usize), DecodeError> {
    let mut result = 0u64;
    let mut shift = 0;
    for (i, &b) in data.iter().enumerate() {
        result |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            return Ok((result, i + 1));
        }
        shift += 7;
        if shift > 63 {
            break;
        }
    }
    Err(DecodeError::Truncated)
}

fn zigzag(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn unzigzag(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

/// Encode a typed record field map using schema metadata.
pub fn encode_record(schema_id: u32, fields: &[(&str, Value)]) -> Result<Vec<u8>, EncodeError> {
    let mut obj = serde_json::Map::new();
    for (k, v) in fields {
        obj.insert((*k).to_string(), v.clone());
    }
    encode(schema_id, &Value::Object(obj))
}

#[allow(dead_code)]
pub fn default_value(ty: &SchemaType) -> Value {
    match ty {
        SchemaType::Null => Value::Null,
        SchemaType::Boolean => Value::Bool(false),
        SchemaType::Int | SchemaType::Long => Value::Number(0.into()),
        SchemaType::Float | SchemaType::Double => {
            Value::Number(serde_json::Number::from_f64(0.0).unwrap())
        }
        SchemaType::String => Value::String(String::new()),
        SchemaType::Bytes => Value::String(String::new()),
        SchemaType::Array { .. } => Value::Array(vec![]),
        SchemaType::Map { .. } => Value::Object(serde_json::Map::new()),
        SchemaType::Record { fields, .. } => {
            let mut m = serde_json::Map::new();
            for f in fields {
                m.insert(
                    f.name.clone(),
                    f.default.clone().unwrap_or_else(|| default_value(&f.ty)),
                );
            }
            Value::Object(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_json() {
        let v = serde_json::json!({"temperature": 30.5, "sensor": "a"});
        let enc = encode(42, &v).unwrap();
        let (id, dec) = decode(&enc).unwrap();
        assert_eq!(id, 42);
        assert_eq!(dec, v);
    }
}
