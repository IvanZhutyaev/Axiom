//! AVM bytecode and `.axc` container format (v1).

pub mod opcode;
pub mod module;

pub use module::{load_axc, save_axc, AxcModule, AXC_MAGIC, AXC_VERSION};
pub use opcode::Opcode;
