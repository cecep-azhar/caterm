//! Thin Tauri command bindings (K2-1). Each command below only
//! deserializes its arguments, calls exactly one `caterm_core` function, and
//! returns its result — no business logic here. See `tests/arch.rs` in
//! `caterm-core` for the guard that enforces this.

use caterm_core::{groups, ssh, store, vault, CatermError};

#[tauri::command]
pub fn list_hosts() -> Result<Vec<store::HostRecord>, CatermError> {
    store::list_hosts()
}

#[tauri::command]
pub fn save_host(input: store::HostInput) -> Result<store::HostRecord, CatermError> {
    store::save_host(input)
}

#[tauri::command]
pub fn delete_host(id: String) -> Result<(), CatermError> {
    store::delete_host(&id)
}

#[tauri::command]
pub fn list_groups() -> Result<Vec<groups::GroupRecord>, CatermError> {
    groups::list_groups()
}

#[tauri::command]
pub fn save_group(input: groups::GroupInput) -> Result<groups::GroupRecord, CatermError> {
    groups::save_group(input)
}

#[tauri::command]
pub fn delete_group(id: String) -> Result<(), CatermError> {
    groups::delete_group(&id)
}

#[tauri::command]
pub fn validate_vault_password(password: String) -> Result<(), CatermError> {
    vault::validate_master_password(&password)
}

#[tauri::command]
pub fn ssh_connect(request: ssh::SshConnectRequest) -> Result<ssh::SshSession, CatermError> {
    ssh::connect(request)
}

#[tauri::command]
pub fn ssh_write(session_id: String, data: String) -> Result<String, CatermError> {
    ssh::write(&session_id, &data)
}

#[tauri::command]
pub fn ssh_resize(session_id: String, cols: u16, rows: u16) -> Result<(), CatermError> {
    ssh::resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn ssh_disconnect(session_id: String) -> Result<(), CatermError> {
    ssh::disconnect(&session_id)
}
