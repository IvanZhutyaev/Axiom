//! Tracing JIT (LLVM backend optional via feature `llvm`).

use avm_bytecode::module::AxcModule;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JitError {
    #[error("jit not enabled")]
    Disabled,
    #[error("trace too cold")]
    Cold,
}

#[derive(Debug, Default)]
pub struct TraceProfile {
    counters: HashMap<u32, AtomicU64>,
    pub hot_threshold: u64,
}

impl TraceProfile {
    pub fn bump(&self, trace_id: u32) -> u64 {
        let c = self
            .counters
            .entry(trace_id)
            .or_insert_with(|| AtomicU64::new(0));
        c.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn is_hot(&self, trace_id: u32) -> bool {
        self.counters
            .get(&trace_id)
            .map(|c| c.load(Ordering::Relaxed) >= self.hot_threshold)
            .unwrap_or(false)
    }
}

pub struct JitCompiler {
    pub profile: TraceProfile,
    pub compiled: HashMap<u32, Vec<u8>>,
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self {
            profile: TraceProfile {
                hot_threshold: 1000,
                ..Default::default()
            },
            compiled: HashMap::new(),
        }
    }
}

impl JitCompiler {
    pub fn record_and_maybe_compile(
        &mut self,
        module: &AxcModule,
        trace_id: u32,
    ) -> Result<Option<Vec<u8>>, JitError> {
        let count = self.profile.bump(trace_id);
        if count < self.profile.hot_threshold {
            return Ok(None);
        }
        if let Some(code) = self.compiled.get(&trace_id) {
            return Ok(Some(code.clone()));
        }
        let native = self.compile_trace(module, trace_id)?;
        self.compiled.insert(trace_id, native.clone());
        Ok(Some(native))
    }

    fn compile_trace(&self, _module: &AxcModule, trace_id: u32) -> Result<Vec<u8>, JitError> {
        #[cfg(feature = "llvm")]
        {
            let _ = trace_id;
            return Err(JitError::Disabled);
        }
        // Portable stub: pseudo-native blob with trace id header
        Ok(vec![0x4A, 0x49, 0x54, (trace_id & 0xff) as u8])
    }

    pub fn deoptimize(&mut self, trace_id: u32) {
        self.compiled.remove(&trace_id);
    }
}
