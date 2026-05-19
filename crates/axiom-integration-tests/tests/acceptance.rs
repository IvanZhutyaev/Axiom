//! Acceptance-oriented smoke tests (TZ §9 subset).

use aql_compile::compile;
use axiom_engine::{BarrierCoordinator, IdempotencyStore, Scheduler};
use axiom_raft::RaftNode;
use avm_bytecode::{load_axc, save_axc};
use std::io::Cursor;

const PIPELINE: &str = include_str!("../../../examples/sensor.aql");

#[test]
fn tz_pipeline_compiles_to_axc() {
    let c = compile(PIPELINE).unwrap();
    let mut buf = Vec::new();
    save_axc(&c.module, &mut Cursor::new(&mut buf)).unwrap();
    let loaded = load_axc(&mut Cursor::new(&buf)).unwrap();
    assert!(!loaded.code.is_empty());
}

#[test]
fn exactly_once_dedup() {
    let mut store = IdempotencyStore::new();
    let key = axiom_engine::EventKey {
        checkpoint_id: uuid::Uuid::new_v4(),
        sequence: 1,
    };
    assert!(store.should_process(key.clone()));
    assert!(!store.should_process(key));
}

#[test]
fn raft_leader_proposes() {
    let mut n = RaftNode::new(1, vec![2, 3]);
    n.become_leader();
    assert!(n.propose(b"x".to_vec()).is_some());
}

#[test]
fn barrier_checkpoint() {
    let mut b = BarrierCoordinator::new(2);
    let id = b.inject_barrier();
    b.ack(id, "a");
    b.ack(id, "b");
    assert!(b.finalize(id).is_some());
}

#[test]
fn scheduler_assigns() {
    let mut s = Scheduler::default();
    s.submit(vec![1, 2, 3], 2);
    s.assign_tasks(&["n1".into(), "n2".into()]);
    assert_eq!(s.list().len(), 1);
}
