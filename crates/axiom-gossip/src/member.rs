//! Cluster member state.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberState {
    Alive,
    Suspect,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub addr: SocketAddr,
    pub incarnation: u64,
    #[serde(skip)]
    pub state: MemberState,
    #[serde(skip)]
    pub last_heard: Option<Instant>,
}

impl Member {
    pub fn new(id: impl Into<String>, addr: SocketAddr) -> Self {
        Self {
            id: id.into(),
            addr,
            incarnation: 0,
            state: MemberState::Alive,
            last_heard: Some(Instant::now()),
        }
    }

    pub fn mark_suspect(&mut self) {
        self.state = MemberState::Suspect;
        self.incarnation += 1;
    }

    pub fn mark_dead(&mut self) {
        self.state = MemberState::Dead;
    }

    pub fn is_alive(&self) -> bool {
        self.state == MemberState::Alive
    }

    pub fn stale(&self, timeout: Duration) -> bool {
        self.last_heard
            .map(|t| t.elapsed() > timeout)
            .unwrap_or(true)
    }
}
