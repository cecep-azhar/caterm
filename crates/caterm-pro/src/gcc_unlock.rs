use crate::license::{LicenseError, VerifiedLicense, verify_token_ed25519};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnlockError {
    #[error("License error: {0}")]
    License(#[from] LicenseError),
    #[error("Handshake request failed: {0}")]
    Http(String),
    #[error("Module '{0}' is not permitted by license tier '{1}'")]
    ModuleNotEntitled(String, String),
    #[error("Backend rejected ephemeral session handshake: {0}")]
    HandshakeRejected(String),
    #[error("Cryptographic exchange failure")]
    CryptoFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub client_session_nonce: String,
    pub hwid: String,
    pub requested_module: String,
    pub token_payload: String,
    pub token_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub success: bool,
    pub ephemeral_token: String,
    pub granted_module: String,
    pub valid_until: i64,
}

#[derive(Debug, Clone)]
pub struct ProRuntimeSession {
    pub module_name: String,
    pub ephemeral_key: [u8; 32],
    pub valid_until: i64,
}

/// Handshake protocol: validates token and performs ephemeral decryption key handshake with GCC.
pub fn verify_and_unlock_module(
    pubkey: &VerifyingKey,
    payload_str: &str,
    signature_base64: &str,
    current_hwid: &str,
    requested_module: &str,
    now: i64,
) -> Result<VerifiedLicense, UnlockError> {
    let verified = verify_token_ed25519(pubkey, payload_str, signature_base64, current_hwid, now)?;

    // Check if requested module is in allowed features or entitlement tier
    let has_entitlement = verified
        .features
        .iter()
        .any(|f| f == requested_module || f == "all")
        || verified.tier == "pro"
        || verified.tier == "enterprise"
        || verified.tier == "team";

    if !has_entitlement {
        return Err(UnlockError::ModuleNotEntitled(
            requested_module.to_string(),
            verified.tier,
        ));
    }

    Ok(verified)
}

/// Perform online handshake with GCC backend endpoint before opening fleet/pro modules.
pub fn perform_gcc_ephemeral_handshake(
    gcc_api_url: &str,
    auth_header: &str,
    req: &HandshakeRequest,
) -> Result<HandshakeResponse, UnlockError> {
    let url = format!("{}/license/handshake", gcc_api_url.trim_end_matches('/'));

    let res = ureq::post(&url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .send_json(req)
        .map_err(|e| UnlockError::Http(e.to_string()))?;

    if res.status().as_u16() != 200 {
        return Err(UnlockError::HandshakeRejected(format!(
            "HTTP status {}",
            res.status()
        )));
    }

    let handshake_resp: HandshakeResponse = res
        .into_body()
        .read_json()
        .map_err(|e| UnlockError::Http(e.to_string()))?;

    if !handshake_resp.success {
        return Err(UnlockError::HandshakeRejected(
            "Handshake success flag is false".into(),
        ));
    }

    Ok(handshake_resp)
}
