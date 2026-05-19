//! WebSocket LSP bridge for UI.

use crate::{diagnostics, LspDiagnostic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct DidChangeParams {
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticNotification {
    pub diagnostics: Vec<LspDiagnostic>,
}

pub fn handle_message(payload: &str) -> Option<String> {
    let req: DidChangeParams = serde_json::from_str(payload).ok()?;
    let diags = diagnostics(&req.source);
    serde_json::to_string(&DiagnosticNotification { diagnostics: diags }).ok()
}
