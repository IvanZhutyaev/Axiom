//! Raft consensus core (metadata + partition log leadership).

pub mod node;
pub mod replicate;
pub mod rpc;
pub mod storage;

pub use node::{RaftNode, RaftState};
pub use rpc::{AppendEntriesRequest, AppendEntriesResponse, RequestVoteRequest, RequestVoteResponse};
pub use replicate::{replicate_and_commit, ReplicationResult};
pub use storage::{LogEntry, MemRaftStorage, RaftStorage};
