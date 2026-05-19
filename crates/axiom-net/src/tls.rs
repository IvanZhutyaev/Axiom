//! TLS 1.3 session upgrade (rustls, phase 0 stub).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("tls not negotiated in this build path")]
    NotNegotiated,
}

/// Placeholder for post-handshake TLS upgrade per TZ §3.5.4.
pub struct TlsSession;

impl TlsSession {
    pub fn upgrade_from_handshake() -> Result<Self, TlsError> {
        Err(TlsError::NotNegotiated)
    }
}
