//! Three-node exactly-once simulation: barrier + idempotency, no dup on retry.

use axiom_engine::barrier::DistributedBarrier;
use axiom_raft::node::{RaftNode, Role};
use axiom_raft::replicate::replicate_and_commit;
use axiom_raft::rpc::AppendEntriesRequest;
use std::sync::{Arc, Mutex};

#[test]
fn three_node_quorum_commit_and_exactly_once() {
    let mut n1 = RaftNode::new(1, vec![2, 3]);
    let mut n2 = RaftNode::new(2, vec![1, 3]);
    let mut n3 = RaftNode::new(3, vec![1, 2]);
    n1.start_election();
    n1.become_leader();
    assert_eq!(n1.state.role, Role::Leader);

    let nodes: Arc<Mutex<Vec<RaftNode>>> =
        Arc::new(Mutex::new(vec![n1, n2, n3]));

    let result = {
        let mut guard = nodes.lock().unwrap();
        replicate_and_commit(&mut guard[0], b"job-config".to_vec(), |peer, req| {
            let idx = match peer {
                2 => 1,
                3 => 2,
                _ => 0,
            };
            guard[idx].handle_append_entries(req)
        })
    };
    assert!(result.is_some());
    assert!(result.unwrap().replicated >= 2);

    let mut barrier = DistributedBarrier::new(3);
    let b = barrier.inject();
    barrier.operator_ack(b, "op1");
    barrier.operator_ack(b, "op2");
    let cp = barrier
        .operator_ack(b, "op3")
        .expect("checkpoint after 3 acks");

    let checkpoint = cp.0;
    let mut delivered = 0;
    for _ in 0..3 {
        if barrier.process_at_checkpoint(checkpoint, b"evt") {
            delivered += 1;
        }
    }
    assert_eq!(delivered, 1, "exactly-once: only one delivery per sequence");
}
