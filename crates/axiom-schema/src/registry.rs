//! Central schema registry with evolution rules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaType {
    Null,
    Boolean,
    Int,
    Long,
    Float,
    Double,
    Bytes,
    String,
    Array { items: Box<SchemaType> },
    Map { values: Box<SchemaType> },
    Record {
        name: String,
        fields: Vec<Field>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: SchemaType,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEntry {
    pub id: u32,
    pub json: String,
    pub ty: SchemaType,
}

#[derive(Debug, Default)]
pub struct SchemaRegistry {
    schemas: HashMap<u32, SchemaEntry>,
    next_id: u32,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("schema {0} not found")]
    NotFound(u32),
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_json(&mut self, json: &str) -> Result<u32, RegistryError> {
        let ty: SchemaType = serde_json::from_str(json)?;
        let id = self.next_id;
        self.next_id += 1;
        self.schemas.insert(
            id,
            SchemaEntry {
                id,
                json: json.to_string(),
                ty,
            },
        );
        Ok(id)
    }

    pub fn get(&self, id: u32) -> Result<&SchemaEntry, RegistryError> {
        self.schemas.get(&id).ok_or(RegistryError::NotFound(id))
    }

    pub fn register_record(&mut self, name: &str, fields: Vec<Field>) -> u32 {
        let ty = SchemaType::Record {
            name: name.into(),
            fields,
        };
        let json = serde_json::to_string(&ty).unwrap_or_default();
        let id = self.next_id;
        self.next_id += 1;
        self.schemas.insert(
            id,
            SchemaEntry {
                id,
                json,
                ty,
            },
        );
        id
    }
}
