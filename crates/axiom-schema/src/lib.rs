//! Schema registry and binary wire format (Avro-like).

pub mod codec;
pub mod registry;

pub use codec::{decode, encode, DecodeError, EncodeError};
pub use registry::{SchemaEntry, SchemaRegistry, SchemaType};
