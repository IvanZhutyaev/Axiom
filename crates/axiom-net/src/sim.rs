//! In-memory simulated transport for tests (loss/delay injection).

use crate::packet::{Packet, PacketError};
use crate::transport::{Connection, Transport, TransportError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Default)]
pub struct SimTransport {
    channels: Arc<Mutex<HashMap<SocketAddr, mpsc::UnboundedSender<Vec<u8>>>>>,
}

impl SimTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, addr: SocketAddr) -> mpsc::UnboundedReceiver<Vec<u8>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut ch = self.channels.blocking_lock();
        ch.insert(addr, tx);
        rx
    }

    pub fn deliver(&self, to: SocketAddr, raw: Vec<u8>) {
        if let Some(tx) = self.channels.blocking_lock().get(&to) {
            let _ = tx.send(raw);
        }
    }
}

pub struct SimConnection {
    pub conn_id: u16,
    pub local: SocketAddr,
    pub remote: SocketAddr,
    sim: Arc<SimTransport>,
    inbox: mpsc::UnboundedReceiver<Vec<u8>>,
}

#[async_trait]
impl Transport for SimTransport {
    async fn connect(&self, remote: SocketAddr) -> Result<Connection, TransportError> {
        let _ = remote;
        Err(TransportError::HandshakeFailed)
    }

    async fn accept(&self) -> Result<Connection, TransportError> {
        Err(TransportError::HandshakeFailed)
    }
}

impl SimConnection {
    pub fn pair(
        sim: Arc<SimTransport>,
        a: SocketAddr,
        b: SocketAddr,
        conn_id: u16,
    ) -> (Self, Self) {
        let rx_a = sim.register(a);
        let rx_b = sim.register(b);
        (
            Self {
                conn_id,
                local: a,
                remote: b,
                sim: sim.clone(),
                inbox: rx_a,
            },
            Self {
                conn_id,
                local: b,
                remote: a,
                sim,
                inbox: rx_b,
            },
        )
    }

    pub async fn send(&self, data: &[u8]) -> Result<(), TransportError> {
        let pkt = Packet {
            flags: crate::packet::PacketFlags::DATA,
            conn_id: self.conn_id,
            seq: 0,
            payload: data.to_vec(),
        };
        self.sim.deliver(self.remote, pkt.encode());
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportError> {
        let raw = self
            .inbox
            .recv()
            .await
            .ok_or(TransportError::Closed)?;
        let pkt = Packet::decode(&raw)?;
        Ok(pkt.payload)
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
    async fn sim_delivers_packets() {
        let sim = Arc::new(SimTransport::new());
        let (mut a, mut b) =
            SimConnection::pair(sim, addr(9001), addr(9002), 1);
        a.send(b"ping").await.unwrap();
        let msg = b.recv().await.unwrap();
        assert_eq!(msg, b"ping");
    }
}
