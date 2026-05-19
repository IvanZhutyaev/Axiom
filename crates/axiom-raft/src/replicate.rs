//! Leader-side log replication with quorum commit.

use crate::node::{RaftNode, Role};
use crate::rpc::{AppendEntriesRequest, AppendEntriesResponse};
use crate::storage::RaftStorage;

pub struct ReplicationResult {
    pub committed_index: u64,
    pub replicated: usize,
}

/// Replicate entry to all peers; commit when majority acks.
pub fn replicate_and_commit(
    leader: &mut RaftNode,
    data: Vec<u8>,
    send_append: impl Fn(u64, AppendEntriesRequest) -> AppendEntriesResponse,
) -> Option<ReplicationResult> {
    if leader.state.role != Role::Leader {
        return None;
    }
    let index = leader.propose(data)?;
    let term = leader.state.current_term;
    let prev_index = index.saturating_sub(1);
    let prev_term = leader.storage.last_log_term();
    let req = AppendEntriesRequest {
        term,
        leader_id: leader.id,
        prev_log_index: prev_index,
        prev_log_term: prev_term,
        entries: vec![(
            term,
            leader
                .storage
                .entry_at(index)
                .map(|e| e.data)
                .unwrap_or_default(),
        )],
        leader_commit: leader.state.commit_index,
    };

    let mut acks = 1usize;
    for (i, peer) in leader.peers.iter().enumerate() {
        let resp = send_append(*peer, req.clone());
        if resp.success {
            leader.match_index[i] = resp.match_index;
            leader.next_index[i] = resp.match_index + 1;
            acks += 1;
        }
    }

    let quorum = leader.peers.len() / 2 + 1;
    if acks >= quorum {
        leader.storage.set_commit_index(index);
        leader.state.commit_index = index;
        Some(ReplicationResult {
            committed_index: index,
            replicated: acks,
        })
    } else {
        None
    }
}
