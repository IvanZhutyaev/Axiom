//! AVM bytecode and `.axc` container format (v1).

pub mod binary_axc;
pub mod module;
pub mod opcode;

pub use binary_axc::{load_axc, save_axc, AXC_VERSION_V2};
pub use module::{load_axc_v1, save_axc_v1, AxcModule, AXC_MAGIC, AXC_VERSION};
pub use opcode::{Instruction, Opcode, Operand};
