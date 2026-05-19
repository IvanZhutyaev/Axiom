# Raft metadata log (phase 1)

Raft entries wrap cluster metadata and partition leadership:

- Term (u64), index (u64), type (enum), payload bytes
- Snapshot: last included index/term + cluster config + LSM checkpoint refs

See `axiom-raft` crate for the in-memory simulation API; full wire format is aligned with the event log CRC framing.
