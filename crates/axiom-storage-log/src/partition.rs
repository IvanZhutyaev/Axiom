//! Multi-partition append-only log with replication factor metadata.

use crate::log::{EventLog, LogError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PartitionConfig {
    pub id: u32,
    pub replication_factor: u8,
}

pub struct PartitionedLog {
    base: PathBuf,
    partitions: HashMap<u32, EventLog>,
    replication_factor: u8,
}

impl PartitionedLog {
    pub fn open(base: impl AsRef<Path>, partition_count: u32, replication_factor: u8) -> Result<Self, LogError> {
        let base = base.as_ref().to_path_buf();
        std::fs::create_dir_all(&base)?;
        let mut partitions = HashMap::new();
        for id in 0..partition_count {
            let dir = base.join(format!("partition-{id}"));
            partitions.insert(id, EventLog::open(dir)?);
        }
        Ok(Self {
            base,
            partitions,
            replication_factor,
        })
    }

    pub fn append(&mut self, partition: u32, payload: Vec<u8>) -> Result<u64, LogError> {
        let log = self
            .partitions
            .get_mut(&partition)
            .ok_or_else(|| LogError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "partition")))?;
        log.append(payload)
    }

    pub fn read(&self, partition: u32, offset: u64) -> Option<&crate::record::LogRecord> {
        self.partitions.get(&partition)?.read(offset)
    }

    pub fn replication_factor(&self) -> u8 {
        self.replication_factor
    }

    pub fn partition_count(&self) -> u32 {
        self.partitions.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn multi_partition_append() {
        let dir = tempdir().unwrap();
        let mut log = PartitionedLog::open(dir.path(), 3, 3).unwrap();
        let o0 = log.append(0, b"a".to_vec()).unwrap();
        let o1 = log.append(1, b"b".to_vec()).unwrap();
        assert_eq!(o0, 0);
        assert_eq!(o1, 0);
        assert_eq!(log.read(0, 0).unwrap().payload, b"a");
    }
}
