use axiom_storage_log::EventLog;
use axiom_storage_lsm::LsmStore;
use tempfile::tempdir;

#[test]
fn log_and_lsm_survive_restart() {
    let dir = tempdir().unwrap();
    let log_path = dir.path().join("log");
    let lsm_path = dir.path().join("lsm");

    let offset = {
        let mut log = EventLog::open(&log_path).unwrap();
        let mut lsm = LsmStore::open(&lsm_path).unwrap();
        let off = log.append(b"event-1".to_vec()).unwrap();
        lsm.put(b"state".to_vec(), b"v1".to_vec()).unwrap();
        lsm.checkpoint().unwrap();
        log.flush_checkpoint_marker(&off.to_le_bytes()).unwrap();
        off
    };

    let log2 = EventLog::open(&log_path).unwrap();
    let lsm2 = LsmStore::open(&lsm_path).unwrap();
    assert_eq!(log2.len(), 1);
    assert_eq!(lsm2.get(b"state"), Some(b"v1".to_vec()));
    assert_eq!(log2.latest_offset(), offset);
}
