//! In-memory sorted table (BTreeMap).

use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct MemTable {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    tombstones: BTreeMap<Vec<u8>, ()>,
}

impl MemTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.tombstones.remove(&key);
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        if self.tombstones.contains_key(key) {
            return None;
        }
        self.data.get(key).map(|v| v.as_slice())
    }

    pub fn delete(&mut self, key: Vec<u8>) {
        self.data.remove(&key);
        self.tombstones.insert(key, ());
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_full(&self, threshold: usize) -> bool {
        self.data.len() >= threshold
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.data
            .iter()
            .filter(|(k, _)| !self.tombstones.contains_key(*k))
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
    }
}
