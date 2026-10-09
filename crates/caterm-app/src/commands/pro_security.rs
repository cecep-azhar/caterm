// Tauri IPC commands for CATerm Pro Security Sentinel Suite
// Modules: Blast Shield, Paste Sentinel, Laptop Posture Scanner, Integrity Tripwire.

use caterm_core::error::CatermError;
use caterm_pro::{
    blast_shield::{BlastShieldCheckResult, check_command_safety},
    integrity_tripwire::{
        BaselineFileRecord, IntegrityTripwireReport, create_baseline_snapshot,
        verify_integrity_against_baseline,
    },
    laptop_audit::{LaptopPostureReport, run_laptop_posture_scan},
    paste_sentinel::{PasteCheckResult, check_paste_content},
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
