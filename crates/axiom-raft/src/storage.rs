use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

pub trait RaftStorage {
    fn current_term(&self) -> u64;
    fn set_current_term(&mut self, term: u64);
    fn voted_for(&self) -> Option<u64>;
    fn set_voted_for(&mut self, id: Option<u64>);
    fn last_log_index(&self) -> u64;
    fn last_log_term(&self) -> u64;
    fn append_entries(&mut self, entries: Vec<LogEntry>);
    fn entry_at(&self, index: u64) -> Option<LogEntry>;
    fn commit_index(&self) -> u64;
    fn set_commit_index(&mut self, idx: u64);
}

#[derive(Debug, Default)]
pub struct MemRaftStorage {
    term: u64,
    voted_for: Option<u64>,
    logs: Vec<LogEntry>,
    commit_index: u64,
    snapshots: HashMap<u64, Vec<u8>>,
}

impl MemRaftStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self, index: u64) -> Option<&[u8]> {
        self.snapshots.get(&index).map(|v| v.as_slice())
    }

    pub fn install_snapshot(&mut self, index: u64, data: Vec<u8>) {
        self.snapshots.insert(index, data);
        self.logs.retain(|e| e.index <= index);
        self.commit_index = self.commit_index.max(index);
    }
}

impl RaftStorage for MemRaftStorage {
    fn current_term(&self) -> u64 {
        self.term
    }

    fn set_current_term(&mut self, term: u64) {
        self.term = term;
    }

    fn voted_for(&self) -> Option<u64> {
        self.voted_for
    }

    fn set_voted_for(&mut self, id: Option<u64>) {
        self.voted_for = id;
    }

    fn last_log_index(&self) -> u64 {
        self.logs.last().map(|e| e.index).unwrap_or(0)
    }

    fn last_log_term(&self) -> u64 {
        self.logs.last().map(|e| e.term).unwrap_or(0)
    }

    fn append_entries(&mut self, entries: Vec<LogEntry>) {
        for e in entries {
            if let Some(pos) = self.logs.iter().position(|x| x.index == e.index) {
                self.logs[pos] = e;
            } else {
                self.logs.push(e);
            }
        }
    }

    fn entry_at(&self, index: u64) -> Option<LogEntry> {
        self.logs.iter().find(|e| e.index == index).cloned()
    }

    fn commit_index(&self) -> u64 {
        self.commit_index
    }

    fn set_commit_index(&mut self, idx: u64) {
        self.commit_index = idx;
    }
}
