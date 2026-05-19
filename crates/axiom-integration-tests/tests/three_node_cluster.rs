//! Phase 0: three swim nodes see each other via gossip messages.

use axiom_gossip::network::{encode_message, sim_recv, sim_send, GossipMesh};
use axiom_gossip::{GossipMessage, Member, SwimNode};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

fn addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

#[tokio::test]
async fn three_node_gossip_mesh() {
    let mesh = Arc::new(GossipMesh::new(addr(9000), 1));
    let n1 = SwimNode::new("n1", addr(9001));
    let n2 = SwimNode::new("n2", addr(9002));
    let n3 = SwimNode::new("n3", addr(9003));

    n1.join(Member::new("n2", addr(9002))).await;
    n1.join(Member::new("n3", addr(9003))).await;
    n2.join(Member::new("n1", addr(9001))).await;
    n3.join(Member::new("n1", addr(9001))).await;

    let (mut c12, mut c21) = mesh.connect_peer(addr(9002));
    let ping = GossipMessage::Ping {
        from: "n1".into(),
        seq: 1,
    };
    sim_send(&c12, &ping).await.unwrap();
    let got = sim_recv(&mut c21).await.unwrap();
    n2.handle_message(got).await;

    assert_eq!(n1.alive_count().await, 3);
    assert_eq!(n2.alive_count().await, 2);

    let bytes = encode_message(&GossipMessage::Ack {
        from: "n2".into(),
        seq: 1,
    })
    .unwrap();
    assert!(!bytes.is_empty());
}
