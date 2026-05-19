use axiom_gossip::{Member, SwimNode};
use axiom_net::sim::{SimConnection, SimTransport};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

fn addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

#[tokio::test]
async fn gossip_membership_two_nodes() {
    let a = SwimNode::new("n1", addr(8010));
    let b = SwimNode::new("n2", addr(8011));
    a.join(Member::new("n2", addr(8011))).await;
    b.join(Member::new("n1", addr(8010))).await;
    assert_eq!(a.alive_count().await, 2);
}

#[tokio::test]
async fn sim_transport_delivers() {
    let sim = Arc::new(SimTransport::new());
    let (mut c1, mut c2) = SimConnection::pair(sim, addr(8020), addr(8021), 9);
    c1.send(b"gossip-ping").await.unwrap();
    let msg = c2.recv().await.unwrap();
    assert_eq!(msg, b"gossip-ping");
}
