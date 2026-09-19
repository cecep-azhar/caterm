//! Thin Tauri command bindings (K2-1). Each command below only
//! deserializes its arguments, calls exactly one `caterm_core` function, and
//! returns its result — no business logic here. See `tests/arch.rs` in
//! `caterm-core` for the guard that enforces this.

use caterm_core::{groups, snippets, ssh, store, vault, CatermError};

#[tauri::command]
pub async fn list_hosts() -> Result<Vec<store::HostRecord>, CatermError> {
    tokio::task::spawn_blocking(store::list_hosts).await.unwrap()
}

#[tauri::command]
pub async fn save_host(input: store::HostInput) -> Result<store::HostRecord, CatermError> {
    tokio::task::spawn_blocking(move || store::save_host(input)).await.unwrap()
}

#[tauri::command]
pub async fn delete_host(id: String) -> Result<(), CatermError> {
    tokio::task::spawn_blocking(move || store::delete_host(&id)).await.unwrap()
}

#[tauri::command]
pub async fn list_groups() -> Result<Vec<groups::GroupRecord>, CatermError> {
    tokio::task::spawn_blocking(groups::list_groups).await.unwrap()
}

#[tauri::command]
pub async fn save_group(input: groups::GroupInput) -> Result<groups::GroupRecord, CatermError> {
    tokio::task::spawn_blocking(move || groups::save_group(input)).await.unwrap()
}

#[tauri::command]
pub async fn delete_group(id: String) -> Result<(), CatermError> {
    tokio::task::spawn_blocking(move || groups::delete_group(&id)).await.unwrap()
}

#[tauri::command]
pub async fn list_snippets() -> Result<Vec<snippets::SnippetRecord>, CatermError> {
    tokio::task::spawn_blocking(snippets::list_snippets).await.unwrap()
}

#[tauri::command]
pub async fn save_snippet(input: snippets::SnippetInput) -> Result<snippets::SnippetRecord, CatermError> {
    tokio::task::spawn_blocking(move || snippets::save_snippet(input)).await.unwrap()
}

#[tauri::command]
pub async fn delete_snippet(id: String) -> Result<(), CatermError> {
    tokio::task::spawn_blocking(move || snippets::delete_snippet(&id)).await.unwrap()
}

#[tauri::command]
pub async fn validate_vault_password(password: String) -> Result<(), CatermError> {
    tokio::task::spawn_blocking(move || vault::validate_master_password(&password)).await.unwrap()
}

#[tauri::command]
pub async fn ssh_connect(host_id: String) -> Result<ssh::SshSession, CatermError> {
    tokio::task::spawn_blocking(move || ssh::connect(&host_id)).await.unwrap()
}

#[tauri::command]
pub async fn ssh_write(session_id: String, data: String) -> Result<String, CatermError> {
    tokio::task::spawn_blocking(move || ssh::write(&session_id, &data)).await.unwrap()
}

#[tauri::command]
pub async fn ssh_read(session_id: String) -> Result<String, CatermError> {
    tokio::task::spawn_blocking(move || ssh::read(&session_id)).await.unwrap()
}

#[tauri::command]
pub async fn ssh_resize(session_id: String, cols: u16, rows: u16) -> Result<(), CatermError> {
    tokio::task::spawn_blocking(move || ssh::resize(&session_id, cols, rows)).await.unwrap()
}

#[tauri::command]
pub async fn ssh_disconnect(session_id: String) -> Result<(), CatermError> {
    tokio::task::spawn_blocking(move || ssh::disconnect(&session_id)).await.unwrap()
}