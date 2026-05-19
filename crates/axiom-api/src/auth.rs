//! Dev JWT (HMAC-SHA256) — replace with OAuth2 provider in production.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub secret: Vec<u8>,
}

impl AuthConfig {
    pub fn dev_default() -> Self {
        Self {
            secret: b"axiom-dev-secret-change-me".to_vec(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: u64,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid token")]
    Invalid,
}

pub fn mint_token(cfg: &AuthConfig, sub: &str, role: &str) -> String {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let claims = Claims {
        sub: sub.into(),
        role: role.into(),
        exp: 4_102_444_800, // far future
    };
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());
    let sig = sign(cfg, &format!("{header}.{payload}"));
    format!("{header}.{payload}.{sig}")
}

pub fn authorize(cfg: &AuthConfig, token: &str) -> Result<Claims, AuthError> {
    let mut parts = token.split('.');
    let header = parts.next().ok_or(AuthError::Invalid)?;
    let payload = parts.next().ok_or(AuthError::Invalid)?;
    let sig = parts.next().ok_or(AuthError::Invalid)?;
    let expected = sign(cfg, &format!("{header}.{payload}"));
    if sig != expected {
        return Err(AuthError::Invalid);
    }
    let json = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AuthError::Invalid)?;
    serde_json::from_slice(&json).map_err(|_| AuthError::Invalid)
}

fn sign(cfg: &AuthConfig, data: &str) -> String {
    let mut h = Sha256::new();
    h.update(&cfg.secret);
    h.update(data.as_bytes());
    URL_SAFE_NO_PAD.encode(h.finalize())
}
