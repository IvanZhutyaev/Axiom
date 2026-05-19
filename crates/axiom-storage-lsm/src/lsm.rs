//! LSM store orchestrating MemTable flushes and reads.

use crate::memtable::MemTable;
use crate::sstable::{SsError, SsTable};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MEMTABLE_THRESHOLD: usize = 1024;

#[derive(Debug, Error)]
pub enum LsmError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sstable: {0}")]
    Ss(#[from] SsError),
}

pub struct LsmStore {
    path: PathBuf,
    memtable: MemTable,
    tables: Vec<SsTable>,
    table_files: Vec<PathBuf>,
}

impl LsmStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LsmError> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;
        let mut store = Self {
            path,
            memtable: MemTable::new(),
            tables: Vec::new(),
            table_files: Vec::new(),
        };
        store.load_existing()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self, LsmError> {
        Self::open(std::env::temp_dir().join(format!("axiom-lsm-{}", uuid_simple())))
    }

    fn load_existing(&mut self) -> Result<(), LsmError> {
        let mut files: Vec<_> = std::fs::read_dir(&self.path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|x| x == "sst")
            })
            .map(|e| e.path())
            .collect();
        files.sort();
        for f in files {
            self.tables.push(SsTable::open(&f)?);
            self.table_files.push(f);
        }
        Ok(())
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), LsmError> {
        self.memtable.put(key, value);
        if self.memtable.is_full(MEMTABLE_THRESHOLD) {
            self.flush()?;
        }
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        if let Some(v) = self.memtable.get(key) {
            return Some(v.to_vec());
        }
        for table in self.tables.iter().rev() {
            if let Some(v) = table.get(key) {
                return Some(v.to_vec());
            }
        }
        None
    }

    pub fn delete(&mut self, key: Vec<u8>) -> Result<(), LsmError> {
        self.memtable.delete(key);
        Ok(())
    }

    pub fn scan_range(&self, start: &[u8], end: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut result = BTreeMap::new();
        for table in &self.tables {
            for (k, v) in &table.entries {
                if k.as_slice() >= start && k.as_slice() < end {
                    result.insert(k.clone(), v.clone());
                }
            }
        }
        for (k, v) in self.memtable.iter() {
            if k >= start && k < end {
                result.insert(k.to_vec(), v.to_vec());
            }
        }
        result.into_iter().collect()
    }

    pub fn flush(&mut self) -> Result<(), LsmError> {
        let entries: Vec<_> = self
            .memtable
            .iter()
            .map(|(k, v)| (k.to_vec(), v.to_vec()))
            .collect();
        if entries.is_empty() {
            return Ok(());
        }
        let id = self.table_files.len();
        let file = self.path.join(format!("{id:06}.sst"));
        let table = SsTable::from_memtable(entries.into_iter());
        table.write_to_file(&file)?;
        self.tables.push(table);
        self.table_files.push(file);
        self.memtable = MemTable::new();
        if self.table_files.len() >= 4 {
            let inputs = self.table_files.clone();
            let _ = crate::compaction::compact_level(&self.path, 1, &inputs);
            self.tables.clear();
            self.table_files.clear();
            self.load_existing()?;
        }
        Ok(())
    }

    pub fn checkpoint(&mut self) -> Result<(), LsmError> {
        self.flush()?;
        let marker = self.path.join("CHECKPOINT");
        std::fs::write(&marker, b"ok")?;
        Ok(())
    }
}

use std::collections::BTreeMap;

fn uuid_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn put_get_flush() {
        let dir = tempdir().unwrap();
        let mut store = LsmStore::open(dir.path()).unwrap();
        store.put(b"k1".to_vec(), b"v1".to_vec()).unwrap();
        assert_eq!(store.get(b"k1"), Some(b"v1".to_vec()));
        store.flush().unwrap();
        let store2 = LsmStore::open(dir.path()).unwrap();
        assert_eq!(store2.get(b"k1"), Some(b"v1".to_vec()));
    }
}
