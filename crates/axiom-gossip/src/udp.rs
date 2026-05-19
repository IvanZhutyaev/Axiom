//! UDP gossip transport over `axiom-net`.

use crate::network::{decode_message, encode_message, GossipNetError};
use crate::swim::GossipMessage;
use axiom_net::transport::{UdpTransport, TransportError};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct UdpGossip {
    transport: Arc<Mutex<UdpTransport>>,
    local: SocketAddr,
}

impl UdpGossip {
    pub async fn bind(addr: SocketAddr) -> Result<Self, TransportError> {
        let transport = UdpTransport::bind(addr).await?;
        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
            local: addr,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local
    }

    pub async fn send_to(&self, peer: SocketAddr, msg: &GossipMessage) -> Result<(), GossipNetError> {
        let bytes = encode_message(msg)?;
        let t = self.transport.lock().await;
        t.send_to(peer, &bytes).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<(SocketAddr, GossipMessage), GossipNetError> {
        let mut t = self.transport.lock().await;
        let (from, bytes) = t.recv_from().await?;
        Ok((from, decode_message(&bytes)?))
    }
}
