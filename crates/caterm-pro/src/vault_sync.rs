use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Checksum mismatch: expected {0}, got {1}")]
    ChecksumMismatch(String, String),
    #[error("Network sync error: {0}")]
    Network(String),
    #[error("Corrupted envelope payload")]
    CorruptedEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedVaultEnvelope {
    pub version: u32,
    pub nonce_hex: String,
    pub ciphertext_base64: String,
    pub checksum_sha256: String,
    pub timestamp: i64,
}

pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Encrypts vault plaintext with 256-bit zero-knowledge DEK using AES-GCM-256.
pub fn encrypt_vault_payload(
    plaintext: &[u8],
    dek: &[u8; 32],
    timestamp: i64,
) -> Result<EncryptedVaultEnvelope, SyncError> {
    let cipher =
        Aes256Gcm::new_from_slice(dek).map_err(|e| SyncError::EncryptionFailed(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| SyncError::EncryptionFailed(e.to_string()))?;

    use base64::{Engine, engine::general_purpose::STANDARD};
    let ciphertext_base64 = STANDARD.encode(&ciphertext);
    let checksum = compute_sha256(plaintext);

    Ok(EncryptedVaultEnvelope {
        version: 1,
        nonce_hex: hex::encode(nonce_bytes),
        ciphertext_base64,
        checksum_sha256: checksum,
        timestamp,
    })
}

/// Decrypts vault envelope using the local zero-knowledge DEK and verifies payload integrity.
pub fn decrypt_vault_payload(
    envelope: &EncryptedVaultEnvelope,
    dek: &[u8; 32],
) -> Result<Vec<u8>, SyncError> {
    let nonce_bytes = hex::decode(&envelope.nonce_hex).map_err(|_| SyncError::CorruptedEnvelope)?;
    if nonce_bytes.len() != 12 {
        return Err(SyncError::CorruptedEnvelope);
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    use base64::{Engine, engine::general_purpose::STANDARD};
    let ciphertext = STANDARD
        .decode(envelope.ciphertext_base64.trim())
        .map_err(|_| SyncError::CorruptedEnvelope)?;

    let cipher =
        Aes256Gcm::new_from_slice(dek).map_err(|e| SyncError::DecryptionFailed(e.to_string()))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| SyncError::DecryptionFailed(e.to_string()))?;

    let actual_checksum = compute_sha256(&plaintext);
    if actual_checksum != envelope.checksum_sha256 {
        return Err(SyncError::ChecksumMismatch(
            envelope.checksum_sha256.clone(),
            actual_checksum,
        ));
    }

    Ok(plaintext)
}
