//! Wasm sandbox for untrusted connectors (wasmtime).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("wasm runtime disabled; enable feature `runtime`")]
    Disabled,
    #[cfg(feature = "runtime")]
    #[error("wasmtime: {0}")]
    Engine(#[from] wasmtime::Error),
}

pub struct WasmSandbox {
    #[cfg(feature = "runtime")]
    engine: wasmtime::Engine,
    memory_limit: usize,
}

impl WasmSandbox {
    pub fn new(memory_limit_mb: usize) -> Result<Self, WasmError> {
        #[cfg(feature = "runtime")]
        {
            let mut config = wasmtime::Config::new();
            config.wasm_memory64(false);
            let engine = wasmtime::Engine::new(&config)?;
            return Ok(Self {
                engine,
                memory_limit: memory_limit_mb * 1024 * 1024,
            });
        }
        #[cfg(not(feature = "runtime"))]
        {
            let _ = memory_limit_mb;
            Err(WasmError::Disabled)
        }
    }

    pub fn load_module(&self, wasm: &[u8]) -> Result<(), WasmError> {
        #[cfg(feature = "runtime")]
        {
            let _module = wasmtime::Module::new(&self.engine, wasm)?;
            return Ok(());
        }
        #[cfg(not(feature = "runtime"))]
        {
            let _ = wasm;
            Err(WasmError::Disabled)
        }
    }
}
