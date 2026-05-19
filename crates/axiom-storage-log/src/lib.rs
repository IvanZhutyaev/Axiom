//! Partitioned append-only log (single-partition phase 0).

pub mod log;
pub mod record;

pub use log::{EventLog, LogError};
pub use record::{LogRecord, RECORD_HEADER_SIZE};
