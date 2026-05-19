//! AQL language server helpers for Monaco integration.

pub mod ws;

use aql_compile::compile;
use aql_syntax::parse;

pub fn diagnostics(source: &str) -> Vec<LspDiagnostic> {
    let mut diags = Vec::new();
    if let Err(e) = parse(source) {
        diags.push(LspDiagnostic {
            line: 0,
            message: e.to_string(),
            severity: "error",
        });
        return diags;
    }
    if let Err(e) = compile(source) {
        diags.push(LspDiagnostic {
            line: 0,
            message: e.to_string(),
            severity: "error",
        });
    }
    diags
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LspDiagnostic {
    pub line: u32,
    pub message: String,
    pub severity: &'static str,
}
