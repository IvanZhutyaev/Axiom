//! AVM stack interpreter with event loop (`EMIT`, `NEXT_EVENT`).

pub mod disasm;
pub mod interpreter;
pub mod value;

pub use disasm::disassemble;
pub use interpreter::{Interpreter, RunError, RunResult};
pub use value::Value;
