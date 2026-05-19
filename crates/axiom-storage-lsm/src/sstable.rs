//! Sorted string table on disk (simple format).

use crate::bloom::BloomFilter;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Default)]
pub struct SsTable {
    pub entries: BTreeMap<Vec<u8>, Vec<u8>>,
    pub bloom: BloomFilter,
}

impl SsTable {
    pub fn from_memtable<I>(iter: I) -> Self
    where
        I: Iterator<Item = (Vec<u8>, Vec<u8>)>,
    {
        let mut entries = BTreeMap::new();
        let mut bloom = BloomFilter::with_capacity(1024, 0.01);
        for (k, v) in iter {
            bloom.insert(&k);
            entries.insert(k, v);
        }
        Self { entries, bloom }
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), SsError> {
        let mut f = File::create(path)?;
        for (k, v) in &self.entries {
            let kl = k.len() as u32;
            let vl = v.len() as u32;
            f.write_all(&kl.to_le_bytes())?;
            f.write_all(k)?;
            f.write_all(&vl.to_le_bytes())?;
            f.write_all(v)?;
        }
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Self, SsError> {
        let mut f = File::open(path)?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        let mut entries = BTreeMap::new();
        let mut pos = 0;
        while pos + 8 <= data.len() {
            let kl = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let key = data[pos..pos + kl].to_vec();
            pos += kl;
            let vl = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let val = data[pos..pos + vl].to_vec();
            pos += vl;
            entries.insert(key, val);
        }
        let mut bloom = BloomFilter::with_capacity(entries.len().max(1), 0.01);
        for k in entries.keys() {
            bloom.insert(k);
        }
        Ok(Self { entries, bloom })
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        if !self.bloom.might_contain(key) {
            return None;
        }
        self.entries.get(key).map(|v| v.as_slice())
    }
}
