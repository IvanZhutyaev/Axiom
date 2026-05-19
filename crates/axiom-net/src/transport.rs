//! Connection-oriented reliable transport over UDP.

use crate::packet::{Packet, PacketFlags, PacketError};
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("packet: {0}")]
    Packet(#[from] PacketError),
    #[error("connection closed")]
    Closed,
    #[error("handshake failed")]
    HandshakeFailed,
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&self, addr: SocketAddr) -> Result<Connection, TransportError>;
    async fn accept(&self) -> Result<Connection, TransportError>;
}

pub struct Connection {
    pub conn_id: u16,
    pub remote: SocketAddr,
    socket: Arc<UdpSocket>,
    send_seq: Arc<Mutex<u32>>,
    recv_buf: Arc<Mutex<VecDeque<Vec<u8>>>>,
    unacked: Arc<Mutex<HashMap<u32, Vec<u8>>>>,
}

pub struct UdpTransport {
    socket: Arc<UdpSocket>,
    next_conn: Arc<Mutex<u16>>,
}

impl UdpTransport {
    pub async fn bind(addr: SocketAddr) -> Result<Self, TransportError> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
            next_conn: Arc::new(Mutex::new(1)),
        })
    }
}

#[async_trait]
impl Transport for UdpTransport {
    async fn connect(&self, remote: SocketAddr) -> Result<Connection, TransportError> {
        let mut id_guard = self.next_conn.lock().await;
        let conn_id = *id_guard;
        *id_guard = id_guard.wrapping_add(1);

        let syn = Packet {
            flags: PacketFlags::SYN,
            conn_id,
            seq: 0,
            payload: vec![0x01, 0x00], // max window placeholder
        };
        self.socket
            .send_to(&syn.encode(), remote)
            .await?;

        let mut buf = vec![0u8; 65535];
        let (len, _) = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            self.socket.recv_from(&mut buf),
        )
        .await
        .map_err(|_| TransportError::HandshakeFailed)??;

        let ack = Packet::decode(&buf[..len])?;
        if !ack.flags.contains(PacketFlags::SYN) || !ack.flags.contains(PacketFlags::ACK) {
            return Err(TransportError::HandshakeFailed);
        }

        Ok(Connection {
            conn_id,
            remote,
            socket: self.socket.clone(),
            send_seq: Arc::new(Mutex::new(1)),
            recv_buf: Arc::new(Mutex::new(VecDeque::new())),
            unacked: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn accept(&self) -> Result<Connection, TransportError> {
        let mut buf = vec![0u8; 65535];
        loop {
            let (len, remote) = self.socket.recv_from(&mut buf).await?;
            let pkt = Packet::decode(&buf[..len])?;
            if pkt.flags.contains(PacketFlags::SYN) {
                let reply = Packet {
                    flags: PacketFlags::SYN | PacketFlags::ACK,
                    conn_id: pkt.conn_id,
                    seq: 0,
                    payload: vec![0x01, 0x00],
                };
                self.socket
                    .send_to(&reply.encode(), remote)
                    .await?;
                return Ok(Connection {
                    conn_id: pkt.conn_id,
                    remote,
                    socket: self.socket.clone(),
                    send_seq: Arc::new(Mutex::new(1)),
                    recv_buf: Arc::new(Mutex::new(VecDeque::new())),
                    unacked: Arc::new(Mutex::new(HashMap::new())),
                });
            }
        }
    }
}

impl Connection {
    pub async fn send(&self, data: &[u8]) -> Result<(), TransportError> {
        let mut seq = self.send_seq.lock().await;
        let packet = Packet {
            flags: PacketFlags::DATA,
            conn_id: self.conn_id,
            seq: *seq,
            payload: data.to_vec(),
        };
        let raw = packet.encode();
        self.socket.send_to(&raw, self.remote).await?;
        self.unacked.lock().await.insert(*seq, raw);
        *seq += 1;
        Ok(())
    }

    pub async fn recv(&self) -> Result<Vec<u8>, TransportError> {
        {
            let mut buf = self.recv_buf.lock().await;
            if let Some(d) = buf.pop_front() {
                return Ok(d);
            }
        }
        let mut sock_buf = vec![0u8; 65535];
        loop {
            let (len, _) = self.socket.recv_from(&mut sock_buf).await?;
            let pkt = Packet::decode(&sock_buf[..len])?;
            if pkt.conn_id != self.conn_id {
                continue;
            }
            if pkt.flags.contains(PacketFlags::ACK) {
                self.unacked.lock().await.remove(&pkt.seq);
            }
            if pkt.flags.contains(PacketFlags::DATA) {
                let ack = Packet {
                    flags: PacketFlags::ACK,
                    conn_id: self.conn_id,
                    seq: pkt.seq,
                    payload: Vec::new(),
                };
                self.socket
                    .send_to(&ack.encode(), self.remote)
                    .await?;
                return Ok(pkt.payload);
            }
        }
    }

    pub async fn close(&self) -> Result<(), TransportError> {
        let fin = Packet {
            flags: PacketFlags::FIN,
            conn_id: self.conn_id,
            seq: 0,
            payload: Vec::new(),
        };
        self.socket
            .send_to(&fin.encode(), self.remote)
            .await?;
        Ok(())
    }
}
