//! Two-process gossip over UDP localhost.

use axiom_gossip::{GossipMessage, UdpGossip};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

fn addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

#[tokio::test]
async fn udp_gossip_ping_pong() {
    let a = UdpGossip::bind(addr(19100)).await.unwrap();
    let b = UdpGossip::bind(addr(19101)).await.unwrap();

    let ping = GossipMessage::Ping {
        from: "a".into(),
        seq: 1,
    };
    a.send_to(addr(19101), &ping).await.unwrap();

    let recv = tokio::time::timeout(Duration::from_secs(2), b.recv())
        .await
        .expect("timeout")
        .unwrap();
    assert_eq!(recv.0.port(), 19100);
    assert!(matches!(recv.1, GossipMessage::Ping { .. }));
}
