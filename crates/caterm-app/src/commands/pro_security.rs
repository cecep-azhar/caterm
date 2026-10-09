// Tauri IPC commands for CATerm Pro Security Sentinel Suite
// Modules: Blast Shield, Paste Sentinel, Laptop Posture Scanner, Integrity Tripwire,
// Ephemeral Keys, Bastion Jump Proxy, Docker Security Probe, Cryptographic Session Recorder.

use caterm_core::error::CatermError;
use caterm_pro::{
    blast_shield::{check_command_safety, BlastShieldCheckResult},
    docker_probe::{run_docker_security_probe, DockerProbeReport},
    ephemeral_keys::{
        generate_ephemeral_key, list_ephemeral_keys, revoke_ephemeral_key,
        CreateEphemeralKeyRequest, EphemeralKeyRecord, EphemeralKeyStatus,
    },
    integrity_tripwire::{
        create_baseline_snapshot, verify_integrity_against_baseline, BaselineFileRecord,
        IntegrityTripwireReport,
    },
    jump_proxy::{probe_jump_chain, JumpChainConfig, JumpChainProbeResult},
    laptop_audit::{run_laptop_posture_scan, LaptopPostureReport},
    paste_sentinel::{check_paste_content, PasteCheckResult},
    session_audit::{verify_session_audit_chain, CastChunk},
};

#[tauri::command]
pub async fn pro_blast_shield_check(
    command: String,
) -> Result<BlastShieldCheckResult, CatermError> {
    tokio::task::spawn_blocking(move || Ok(check_command_safety(&command)))
        .await
        .map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(format!(
                "Task error: {e}"
            )))
        })?
}

#[tauri::command]
pub async fn pro_paste_sentinel_check(text: String) -> Result<PasteCheckResult, CatermError> {
    tokio::task::spawn_blocking(move || Ok(check_paste_content(&text)))
        .await
        .map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(format!(
                "Task error: {e}"
            )))
        })?
}

#[tauri::command]
pub async fn pro_laptop_posture_scan() -> Result<LaptopPostureReport, CatermError> {
    tokio::task::spawn_blocking(|| Ok(run_laptop_posture_scan()))
        .await
        .map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(format!(
                "Task error: {e}"
            )))
        })?
}

#[tauri::command]
pub async fn pro_integrity_create_baseline(
    target_name: String,
    paths: Option<Vec<String>>,
) -> Result<Vec<BaselineFileRecord>, CatermError> {
    tokio::task::spawn_blocking(move || Ok(create_baseline_snapshot(&target_name, paths)))
        .await
        .map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(format!(
                "Task error: {e}"
            )))
        })?
}

#[tauri::command]
pub async fn pro_integrity_verify(
    target_name: String,
    baseline: Vec<BaselineFileRecord>,
) -> Result<IntegrityTripwireReport, CatermError> {
    tokio::task::spawn_blocking(move || {
        Ok(verify_integrity_against_baseline(&target_name, &baseline))
    })
    .await
    .map_err(|e| {
        CatermError::from(caterm_core::error::IoError::Generic(format!(
            "Task error: {e}"
        )))
    })?
}

// ======================== BATCH 2 IPC COMMANDS ========================

#[tauri::command]
pub async fn pro_ephemeral_key_create(
    request: CreateEphemeralKeyRequest,
) -> Result<EphemeralKeyRecord, CatermError> {
    tokio::task::spawn_blocking(move || {
        generate_ephemeral_key(request).map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(e))
        })
    })
    .await
    .map_err(|e| {
        CatermError::from(caterm_core::error::IoError::Generic(format!(
            "Task error: {e}"
        )))
    })?
}

#[tauri::command]
pub async fn pro_ephemeral_keys_list() -> Result<Vec<EphemeralKeyStatus>, CatermError> {
    tokio::task::spawn_blocking(|| Ok(list_ephemeral_keys()))
        .await
        .map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(format!(
                "Task error: {e}"
            )))
        })?
}

#[tauri::command]
pub async fn pro_ephemeral_key_revoke(id: String) -> Result<bool, CatermError> {
    tokio::task::spawn_blocking(move || {
        revoke_ephemeral_key(&id).map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(e))
        })
    })
    .await
    .map_err(|e| {
        CatermError::from(caterm_core::error::IoError::Generic(format!(
            "Task error: {e}"
        )))
    })?
}

#[tauri::command]
pub async fn pro_jump_proxy_probe(
    config: JumpChainConfig,
) -> Result<JumpChainProbeResult, CatermError> {
    tokio::task::spawn_blocking(move || Ok(probe_jump_chain(&config)))
        .await
        .map_err(|e| {
            CatermError::from(caterm_core::error::IoError::Generic(format!(
                "Task error: {e}"
            )))
        })?
}

#[tauri::command]
pub async fn pro_docker_probe_scan(
    remote_socket_info: Option<(String, u32)>,
    containers: Option<Vec<serde_json::Value>>,
) -> Result<DockerProbeReport, CatermError> {
    tokio::task::spawn_blocking(move || {
        Ok(run_docker_security_probe(remote_socket_info, containers))
    })
    .await
    .map_err(|e| {
        CatermError::from(caterm_core::error::IoError::Generic(format!(
            "Task error: {e}"
        )))
    })?
}

#[tauri::command]
pub async fn pro_session_audit_verify(
    genesis_hash: String,
    chunks: Vec<CastChunk>,
) -> Result<bool, CatermError> {
    tokio::task::spawn_blocking(move || {
        Ok(verify_session_audit_chain(&genesis_hash, &chunks))
    })
    .await
    .map_err(|e| {
        CatermError::from(caterm_core::error::IoError::Generic(format!(
            "Task error: {e}"
        )))
    })?
}
