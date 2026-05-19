//! Node configuration.

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Master,
    Worker,
    Storage,
    AllInOne,
}

impl NodeRole {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "master" => Self::Master,
            "worker" => Self::Worker,
            "storage" => Self::Storage,
            _ => Self::AllInOne,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub role: NodeRole,
    pub api_bind: String,
    pub gossip_bind: String,
    pub data_dir: PathBuf,
    /// Peer gossip addresses to join (multi-node dev cluster).
    pub seeds: Vec<SocketAddr>,
    pub grpc_bind: String,
}
