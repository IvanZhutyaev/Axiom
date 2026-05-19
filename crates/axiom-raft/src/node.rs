use crate::rpc::{
    AppendEntriesRequest, AppendEntriesResponse, RequestVoteRequest, RequestVoteResponse,
};
use crate::storage::{LogEntry, MemRaftStorage, RaftStorage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftState {
    pub role: Role,
    pub current_term: u64,
    pub leader_id: Option<u64>,
    pub commit_index: u64,
}

pub struct RaftNode {
    pub id: u64,
    pub state: RaftState,
    pub storage: MemRaftStorage,
    pub peers: Vec<u64>,
    pub votes_received: usize,
    pub next_index: Vec<u64>,
    pub match_index: Vec<u64>,
}

impl RaftNode {
    pub fn new(id: u64, peers: Vec<u64>) -> Self {
        Self {
            id,
            state: RaftState {
                role: Role::Follower,
                current_term: 0,
                leader_id: None,
                commit_index: 0,
            },
            storage: MemRaftStorage::new(),
            peers,
            votes_received: 0,
            next_index: Vec::new(),
            match_index: Vec::new(),
        }
    }

    pub fn start_election(&mut self) {
        self.state.role = Role::Candidate;
        self.state.current_term += 1;
        self.storage.set_current_term(self.state.current_term);
        self.storage.set_voted_for(Some(self.id));
        self.votes_received = 1;
    }

    pub fn become_leader(&mut self) {
        self.state.role = Role::Leader;
        self.state.leader_id = Some(self.id);
        let last = self.storage.last_log_index() + 1;
        self.next_index = vec![last; self.peers.len()];
        self.match_index = vec![0; self.peers.len()];
    }

    pub fn handle_request_vote(&mut self, req: RequestVoteRequest) -> RequestVoteResponse {
        if req.term < self.state.current_term {
            return RequestVoteResponse {
                term: self.state.current_term,
                vote_granted: false,
            };
        }
        if req.term > self.state.current_term {
            self.state.current_term = req.term;
            self.storage.set_current_term(req.term);
            self.state.role = Role::Follower;
            self.storage.set_voted_for(None);
        }
        let up_to_date = req.last_log_term > self.storage.last_log_term()
            || (req.last_log_term == self.storage.last_log_term()
                && req.last_log_index >= self.storage.last_log_index());
        let grant = self.storage.voted_for().is_none() && up_to_date;
        if grant {
            self.storage.set_voted_for(Some(req.candidate_id));
        }
        RequestVoteResponse {
            term: self.state.current_term,
            vote_granted: grant,
        }
    }

    pub fn handle_append_entries(&mut self, req: AppendEntriesRequest) -> AppendEntriesResponse {
        if req.term < self.state.current_term {
            return AppendEntriesResponse {
                term: self.state.current_term,
                success: false,
                match_index: 0,
            };
        }
        self.state.role = Role::Follower;
        self.state.current_term = req.term;
        self.state.leader_id = Some(req.leader_id);
        self.storage.set_current_term(req.term);

        let mut entries = Vec::new();
        let mut idx = req.prev_log_index;
        for (term, data) in req.entries {
            idx += 1;
            entries.push(LogEntry {
                term,
                index: idx,
                data,
            });
        }
        self.storage.append_entries(entries);
        if req.leader_commit > self.storage.commit_index() {
            self.storage
                .set_commit_index(req.leader_commit.min(self.storage.last_log_index()));
        }
        self.state.commit_index = self.storage.commit_index();
        AppendEntriesResponse {
            term: self.state.current_term,
            success: true,
            match_index: self.storage.last_log_index(),
        }
    }

    pub fn propose(&mut self, data: Vec<u8>) -> Option<u64> {
        if self.state.role != Role::Leader {
            return None;
        }
        let index = self.storage.last_log_index() + 1;
        let term = self.state.current_term;
        self.storage.append_entries(vec![LogEntry {
            term,
            index,
            data,
        }]);
        Some(index)
    }

    pub fn tick_election(&mut self, won: bool) {
        if won && self.votes_received > self.peers.len() / 2 {
            self.become_leader();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leader_propose_replicates() {
        let mut leader = RaftNode::new(1, vec![2, 3]);
        leader.become_leader();
        let idx = leader.propose(b"meta".to_vec()).unwrap();
        assert_eq!(idx, 1);
    }
}
