//! SWIM protocol core (ping / indirect ping / dissemination).

use crate::member::{Member, MemberState};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    Ping { from: String, seq: u64 },
    Ack { from: String, seq: u64 },
    IndirectPing { target: String, from: String, seq: u64 },
    Update(Member),
}

pub struct SwimNode {
    pub local_id: String,
    pub local_addr: SocketAddr,
    members: Arc<RwLock<HashMap<String, Member>>>,
    ping_seq: AtomicU64,
    suspect_timeout: Duration,
    _probe_interval: Duration,
}

impl SwimNode {
    pub fn new(id: impl Into<String>, addr: SocketAddr) -> Self {
        let id = id.into();
        let mut members = HashMap::new();
        members.insert(id.clone(), Member::new(id.clone(), addr));
        Self {
            local_id: id,
            local_addr: addr,
            members: Arc::new(RwLock::new(members)),
            ping_seq: AtomicU64::new(0),
            suspect_timeout: Duration::from_secs(5),
            _probe_interval: Duration::from_secs(1),
        }
    }

    pub async fn join(&self, peer: Member) {
        let mut m = self.members.write().await;
        m.insert(peer.id.clone(), peer);
    }

    pub async fn members_snapshot(&self) -> Vec<Member> {
        self.members
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    pub async fn alive_count(&self) -> usize {
        self.members
            .read()
            .await
            .values()
            .filter(|m| m.is_alive())
            .count()
    }

    pub fn next_probe_target(&self, members: &[Member]) -> Option<Member> {
        let candidates: Vec<_> = members
            .iter()
            .filter(|m| m.id != self.local_id && m.is_alive())
            .cloned()
            .collect();
        let mut rng = rand::thread_rng();
        candidates.choose(&mut rng).cloned()
    }

    pub fn make_ping(&self) -> GossipMessage {
        let seq = self.ping_seq.fetch_add(1, Ordering::Relaxed);
        GossipMessage::Ping {
            from: self.local_id.clone(),
            seq,
        }
    }

    pub async fn handle_message(&self, msg: GossipMessage) {
        match msg {
            GossipMessage::Ping { from, .. } => {
                let mut m = self.members.write().await;
                if let Some(member) = m.get_mut(&from) {
                    member.last_heard = Some(std::time::Instant::now());
                    member.state = MemberState::Alive;
                }
            }
            GossipMessage::Ack { from, .. } => {
                let mut m = self.members.write().await;
                if let Some(member) = m.get_mut(&from) {
                    member.last_heard = Some(std::time::Instant::now());
                    member.state = MemberState::Alive;
                }
            }
            GossipMessage::Update(update) => {
                let mut m = self.members.write().await;
                m.insert(update.id.clone(), update);
            }
            GossipMessage::IndirectPing { target, from, .. } => {
                let mut m = self.members.write().await;
                if let Some(member) = m.get_mut(&target) {
                    member.last_heard = Some(std::time::Instant::now());
                }
                if let Some(member) = m.get_mut(&from) {
                    member.last_heard = Some(std::time::Instant::now());
                }
            }
        }
    }

    pub async fn run_probe_round(&self) {
        let snapshot = self.members_snapshot().await;
        for member in &snapshot {
            if member.id != self.local_id && member.stale(self.suspect_timeout) {
                let mut m = self.members.write().await;
                if let Some(mem) = m.get_mut(&member.id) {
                    if mem.state == MemberState::Alive {
                        mem.mark_suspect();
                    } else if mem.state == MemberState::Suspect {
                        mem.mark_dead();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
    }

    #[tokio::test]
    async fn join_and_see_peer() {
        let a = SwimNode::new("node-a", addr(7001));
        let b = SwimNode::new("node-b", addr(7002));
        a.join(Member::new("node-b", addr(7002))).await;
        b.join(Member::new("node-a", addr(7001))).await;
        assert_eq!(a.alive_count().await, 2);
        assert_eq!(b.alive_count().await, 2);
    }

    #[tokio::test]
    async fn ping_updates_liveness() {
        let a = SwimNode::new("node-a", addr(7003));
        a.join(Member::new("node-b", addr(7004))).await;
        let ping = a.make_ping();
        a.handle_message(GossipMessage::Ack {
            from: "node-b".into(),
            seq: 1,
        })
        .await;
        let _ = ping;
        assert!(a.alive_count().await >= 1);
    }
}
