//! AQL compiler: symbols, type inference, logical plan, bytecode emission.

pub mod compile;
pub mod logical;
pub mod types;

pub use compile::{compile, CompileError, CompileResult};
