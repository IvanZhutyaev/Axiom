//! Leveled compaction for LSM SSTables.

use crate::sstable::SsTable;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompactError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sstable: {0}")]
    Ss(#[from] crate::sstable::SsError),
}

/// Merge overlapping SSTables at `level` into `level+1`.
pub fn compact_level(
    base_dir: &Path,
    level: u32,
    inputs: &[PathBuf],
) -> Result<PathBuf, CompactError> {
    let mut merged = SsTable::default();
    for path in inputs {
        let table = SsTable::open(path)?;
        for (k, v) in table.entries {
            merged.entries.insert(k, v);
        }
    }
    let out = base_dir.join(format!("L{level:02}-{:06}.sst", merged.entries.len()));
    merged.write_to_file(&out)?;
    for path in inputs {
        let _ = std::fs::remove_file(path);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn merge_two_tables() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.sst");
        let b = dir.path().join("b.sst");
        let mut t1 = SsTable::default();
        t1.entries.insert(b"k1".to_vec(), b"v1".to_vec());
        t1.write_to_file(&a).unwrap();
        let mut t2 = SsTable::default();
        t2.entries.insert(b"k2".to_vec(), b"v2".to_vec());
        t2.write_to_file(&b).unwrap();
        let out = compact_level(dir.path(), 1, &[a, b]).unwrap();
        let loaded = SsTable::open(&out).unwrap();
        assert_eq!(loaded.entries.len(), 2);
    }
}
