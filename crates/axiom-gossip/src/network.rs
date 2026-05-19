//! Gossip message encoding and delivery over `axiom-net`.

use crate::swim::GossipMessage;
use axiom_net::packet::{Packet, PacketFlags};
use axiom_net::sim::{SimConnection, SimTransport};
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GossipNetError {
    #[error("encode: {0}")]
    Encode(String),
    #[error("net: {0}")]
    Net(#[from] axiom_net::transport::TransportError),
    #[error("packet: {0}")]
    Packet(#[from] axiom_net::packet::PacketError),
    #[error("closed")]
    Closed,
}

pub fn encode_message(msg: &GossipMessage) -> Result<Vec<u8>, GossipNetError> {
    serde_json::to_vec(msg).map_err(|e| GossipNetError::Encode(e.to_string()))
}

pub fn decode_message(bytes: &[u8]) -> Result<GossipMessage, GossipNetError> {
    serde_json::from_slice(bytes).map_err(|e| GossipNetError::Encode(e.to_string()))
}

pub fn wrap_packet(conn_id: u16, payload: &[u8]) -> Vec<u8> {
    Packet {
        flags: PacketFlags::DATA,
        conn_id,
        seq: 0,
        payload: payload.to_vec(),
    }
    .encode()
}

/// Send gossip payload over a sim connection (tests / loopback).
pub async fn sim_send(conn: &SimConnection, msg: &GossipMessage) -> Result<(), GossipNetError> {
    let bytes = encode_message(msg)?;
    conn.send(&bytes).await?;
    Ok(())
}

/// Receive one gossip message from sim connection.
pub async fn sim_recv(conn: &mut SimConnection) -> Result<GossipMessage, GossipNetError> {
    let bytes = conn.recv().await?;
    decode_message(&bytes)
}

pub struct GossipMesh {
    pub sim: Arc<SimTransport>,
    pub local: SocketAddr,
    pub conn_id: u16,
}

impl GossipMesh {
    pub fn new(local: SocketAddr, conn_id: u16) -> Self {
        Self {
            sim: Arc::new(SimTransport::new()),
            local,
            conn_id,
        }
    }

    pub fn connect_peer(
        &self,
        peer: SocketAddr,
    ) -> (SimConnection, SimConnection) {
        SimConnection::pair(self.sim.clone(), self.local, peer, self.conn_id)
    }
}
