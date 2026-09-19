//! Thin Tauri command bindings (K2-1). Each command below only
//! deserializes its arguments, calls exactly one `caterm_core` function, and
//! returns its result — no business logic here. See `tests/arch.rs` in
//! `caterm-core` for the guard that enforces this.

use caterm_core::{groups, keys, snippets, ssh, store, vault, CatermError};

async fn run_blocking<F, R>(f: F) -> Result<R, CatermError>
where
    F: FnOnce() -> Result<R, CatermError> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| caterm_core::error::IoError::Generic(format!("Task join error: {e}")))?
}

#[tauri::command]
pub async fn list_keys() -> Result<Vec<keys::KeyRecord>, CatermError> {
    run_blocking(keys::list_keys).await
}

#[tauri::command]
pub async fn generate_key(input: keys::KeyInput) -> Result<keys::KeyRecord, CatermError> {
    run_blocking(move || keys::generate_key(input)).await
}

#[tauri::command]
pub async fn import_key(name: String, private_key_pem: String, passphrase: Option<String>) -> Result<keys::KeyRecord, CatermError> {
    run_blocking(move || keys::import_key(&name, &private_key_pem, passphrase.as_deref())).await
}

#[tauri::command]
pub async fn delete_key(id: String) -> Result<(), CatermError> {
    run_blocking(move || keys::delete_key(&id)).await
}

#[tauri::command]
pub async fn list_hosts() -> Result<Vec<store::HostRecord>, CatermError> {
    run_blocking(store::list_hosts).await
}

#[tauri::command]
pub async fn save_host(input: store::HostInput) -> Result<store::HostRecord, CatermError> {
    run_blocking(move || store::save_host(input)).await
}

#[tauri::command]
pub async fn delete_host(id: String) -> Result<(), CatermError> {
    run_blocking(move || store::delete_host(&id)).await
}

#[tauri::command]
pub async fn list_groups() -> Result<Vec<groups::GroupRecord>, CatermError> {
    run_blocking(groups::list_groups).await
}

#[tauri::command]
pub async fn save_group(input: groups::GroupInput) -> Result<groups::GroupRecord, CatermError> {
    run_blocking(move || groups::save_group(input)).await
}

#[tauri::command]
pub async fn delete_group(id: String) -> Result<(), CatermError> {
    run_blocking(move || groups::delete_group(&id)).await
}

#[tauri::command]
pub async fn list_snippets() -> Result<Vec<snippets::SnippetRecord>, CatermError> {
    run_blocking(snippets::list_snippets).await
}

#[tauri::command]
pub async fn save_snippet(input: snippets::SnippetInput) -> Result<snippets::SnippetRecord, CatermError> {
    run_blocking(move || snippets::save_snippet(input)).await
}

#[tauri::command]
pub async fn delete_snippet(id: String) -> Result<(), CatermError> {
    run_blocking(move || snippets::delete_snippet(&id)).await
}

#[tauri::command]
pub async fn validate_vault_password(password: String) -> Result<(), CatermError> {
    run_blocking(move || vault::validate_master_password(&password)).await
}

#[tauri::command]
pub async fn ssh_connect(host_id: String) -> Result<ssh::SshSession, CatermError> {
    run_blocking(move || ssh::connect(&host_id)).await
}

#[tauri::command]
pub async fn ssh_write(session_id: String, data: String) -> Result<String, CatermError> {
    run_blocking(move || ssh::write(&session_id, &data)).await
}

#[tauri::command]
pub async fn ssh_read(session_id: String) -> Result<String, CatermError> {
    run_blocking(move || ssh::read(&session_id)).await
}

#[tauri::command]
pub async fn ssh_resize(session_id: String, cols: u16, rows: u16) -> Result<(), CatermError> {
    run_blocking(move || ssh::resize(&session_id, cols, rows)).await
}

#[tauri::command]
pub async fn ssh_disconnect(session_id: String) -> Result<(), CatermError> {
    run_blocking(move || ssh::disconnect(&session_id)).await
}