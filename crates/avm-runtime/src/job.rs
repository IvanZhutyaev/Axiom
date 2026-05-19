//! Isolated execution context per job (TZ §3.2.1).

use crate::interpreter::Interpreter;
use avm_bytecode::module::AxcModule;
use avm_gc::{Heap, GcConfig};

pub struct JobContext {
    pub job_id: String,
    pub module: AxcModule,
    interpreter: Interpreter,
    pub heap: Heap,
}

impl JobContext {
    pub fn new(job_id: impl Into<String>, module: AxcModule) -> Self {
        let interpreter = Interpreter::new(module.clone());
        Self {
            job_id: job_id.into(),
            module,
            interpreter,
            heap: Heap::new(GcConfig::default()),
        }
    }

    pub fn alloc(&mut self, size: usize) -> Option<usize> {
        avm_gc::with_tlab(size.max(64), |tlab| {
            tlab.allocate(size).or_else(|| self.heap.allocate(size))
        })
    }

    pub fn interpreter_mut(&mut self) -> &mut Interpreter {
        &mut self.interpreter
    }
}
