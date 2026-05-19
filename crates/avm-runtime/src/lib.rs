//! AVM stack interpreter with event loop (`EMIT`, `NEXT_EVENT`).

pub mod disasm;
pub mod interpreter;
pub mod job;
pub mod pipeline_exec;
pub mod value;
pub mod window;

pub use disasm::disassemble;
pub use interpreter::{Interpreter, RunError, RunResult};
pub use job::JobContext;
pub use pipeline_exec::{run_batch, PipelineExecutor, WatermarkState};
pub use value::Value;
