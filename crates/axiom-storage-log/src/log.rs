//! In-memory and file-backed append-only log.

use crate::record::{LogRecord, RecordError, RECORD_HEADER_SIZE};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("record: {0}")]
    Record(#[from] RecordError),
}

pub struct EventLog {
    path: Option<PathBuf>,
    records: Vec<LogRecord>,
    next_offset: u64,
}

impl EventLog {
    pub fn in_memory() -> Self {
        Self {
            path: None,
            records: Vec::new(),
            next_offset: 0,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, LogError> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;
        let data_file = path.join("segment-0.log");
        let mut log = Self {
            path: Some(path),
            records: Vec::new(),
            next_offset: 0,
        };
        if data_file.exists() {
            log.load_file(&data_file)?;
        }
        Ok(log)
    }

    fn load_file(&mut self, file: &Path) -> Result<(), LogError> {
        let mut f = File::open(file)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let mut pos = 0;
        while pos < buf.len() {
            if buf.len() - pos < RECORD_HEADER_SIZE {
                break;
            }
            let len = u32::from_le_bytes(buf[pos + 8..pos + 12].try_into().unwrap()) as usize;
            let total = RECORD_HEADER_SIZE + len;
            if pos + total > buf.len() {
                break;
            }
            let rec = LogRecord::decode(&buf[pos..pos + total])?;
            self.next_offset = rec.offset + 1;
            self.records.push(rec);
            pos += total;
        }
        Ok(())
    }

    pub fn append(&mut self, payload: Vec<u8>) -> Result<u64, LogError> {
        let offset = self.next_offset;
        self.next_offset += 1;
        let rec = LogRecord { offset, payload };
        if let Some(ref dir) = self.path.clone() {
            let file = dir.join("segment-0.log");
            let mut f = OpenOptions::new().create(true).append(true).open(file)?;
            f.write_all(&rec.encode())?;
        }
        self.records.push(rec);
        Ok(offset)
    }

    pub fn read(&self, offset: u64) -> Option<&LogRecord> {
        self.records.iter().find(|r| r.offset == offset)
    }

    pub fn iter(&self) -> impl Iterator<Item = &LogRecord> {
        self.records.iter()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn flush_checkpoint_marker(&mut self, marker: &[u8]) -> Result<(), LogError> {
        if let Some(ref dir) = self.path.clone() {
            let mut f = File::create(dir.join("checkpoint.marker"))?;
            f.write_all(marker)?;
            f.sync_all()?;
        }
        Ok(())
    }

    pub fn latest_offset(&self) -> u64 {
        self.next_offset.saturating_sub(1)
    }

    pub fn seek_end(&mut self) -> u64 {
        let off = self.latest_offset();
        let _ = off;
        self.next_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn persist_and_reload() {
        let dir = tempdir().unwrap();
        let mut log = EventLog::open(dir.path()).unwrap();
        log.append(b"a".to_vec()).unwrap();
        log.append(b"b".to_vec()).unwrap();
        let log2 = EventLog::open(dir.path()).unwrap();
        assert_eq!(log2.len(), 2);
    }
}
