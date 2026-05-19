//! Partitioned append-only log (single-partition phase 0).

pub mod compress;
pub mod log;
pub mod partition;
pub mod record;

pub use compress::{compress, decompress, Compression, CompressError};
pub use log::{EventLog, LogError};
pub use partition::{PartitionConfig, PartitionedLog};
pub use record::{LogRecord, RECORD_HEADER_SIZE};
