//! Distributed execution engine: scheduler, barriers, exactly-once.

pub mod barrier;
pub mod barrier;
pub mod checkpoint;
pub mod exactly_once;
pub mod pipeline;
pub mod scheduler;

pub use barrier::DistributedBarrier;
pub use checkpoint::{BarrierCoordinator, CheckpointId};
pub use exactly_once::{EventKey, IdempotencyStore};
pub use pipeline::PipelineRunner;
pub use scheduler::{Scheduler, TaskId, TaskSpec};
