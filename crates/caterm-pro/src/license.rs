use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("Invalid license signature format")]
    BadSignatureEncoding,
    #[error("Cryptographic signature verification failed")]
    VerificationFailed,
    #[error("License token payload corrupted: {0}")]
    CorruptedPayload(String),
    #[error("License expired at {0}, current timestamp {1}")]
    Expired(i64, i64),
    #[error("Hardware ID mismatch: token {0}, device {1}")]
    HwidMismatch(String, String),
    #[error("Invalid verifying key: {0}")]
    InvalidPublicKey(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseClaims {
    pub lic_id: String,
    pub account_id: String,
    pub hwid: String,
    pub tier: String,
    pub features: Vec<String>,
    pub iat: i64,
    pub exp: i64,
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedLicense {
    pub is_valid: bool,
    pub tier: String,
    pub features: Vec<String>,
    pub expires_at: i64,
    pub claims: LicenseClaims,
}

pub fn parse_public_key_hex(hex_str: &str) -> Result<VerifyingKey, LicenseError> {
    let bytes =
        hex::decode(hex_str.trim()).map_err(|e| LicenseError::InvalidPublicKey(e.to_string()))?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| LicenseError::InvalidPublicKey("Key must be 32 bytes".into()))?;
    VerifyingKey::from_bytes(&key_bytes).map_err(|e| LicenseError::InvalidPublicKey(e.to_string()))
}

/// Validates Ed25519 signed license token against trusted public key and local hardware identity.
pub fn verify_token_ed25519(
    pubkey: &VerifyingKey,
    payload_str: &str,
    signature_base64: &str,
    current_hwid: &str,
    now: i64,
) -> Result<VerifiedLicense, LicenseError> {
    let sig_bytes = STANDARD
        .decode(signature_base64.trim())
        .map_err(|_| LicenseError::BadSignatureEncoding)?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| LicenseError::BadSignatureEncoding)?;
    let signature = Signature::from_bytes(&sig_arr);

    pubkey
        .verify(payload_str.as_bytes(), &signature)
        .map_err(|_| LicenseError::VerificationFailed)?;

    let claims: LicenseClaims = serde_json::from_str(payload_str)
        .map_err(|e| LicenseError::CorruptedPayload(e.to_string()))?;

    if now > claims.exp {
        return Err(LicenseError::Expired(claims.exp, now));
    }

    if !claims.hwid.is_empty() && claims.hwid != current_hwid {
        return Err(LicenseError::HwidMismatch(
            claims.hwid,
            current_hwid.to_string(),
        ));
    }

    Ok(VerifiedLicense {
        is_valid: true,
        tier: claims.tier.clone(),
        features: claims.features.clone(),
        expires_at: claims.exp,
        claims,
    })
}
