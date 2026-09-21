//! Thin Tauri command bindings (K2-1). Each command below only
//! deserializes its arguments, calls exactly one `caterm_core` function, and
//! returns its result — no business logic here. See `tests/arch.rs` in
//! `caterm-core` for the guard that enforces this.

use caterm_core::{
    CatermError, ai, audit, backup, groups, investigations, keys, monitor, sftp, snippets, ssh,
    store, teams, tunnels, vault,
};

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
pub async fn window_minimize(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub async fn window_maximize(window: tauri::Window) {
    if let Ok(max) = window.is_maximized() {
        if max {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
pub async fn window_close(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
pub async fn list_tunnels() -> Result<Vec<tunnels::TunnelRecord>, CatermError> {
    run_blocking(tunnels::list_tunnels).await
}

#[tauri::command]
pub async fn save_tunnel(
    input: tunnels::TunnelInput,
) -> Result<tunnels::TunnelRecord, CatermError> {
    run_blocking(move || tunnels::save_tunnel(input)).await
}

#[tauri::command]
pub async fn delete_tunnel(id: String) -> Result<(), CatermError> {
    run_blocking(move || tunnels::delete_tunnel(&id)).await
}

#[tauri::command]
pub async fn start_tunnel(id: String) -> Result<(), CatermError> {
    run_blocking(move || tunnels::start_tunnel(&id)).await
}

#[tauri::command]
pub async fn stop_tunnel(id: String) -> Result<(), CatermError> {
    run_blocking(move || tunnels::stop_tunnel(&id)).await
}
#[tauri::command]
pub async fn list_remote_dir(
    host_id: String,
    remote_path: String,
) -> Result<Vec<sftp::SftpFileEntry>, CatermError> {
    run_blocking(move || sftp::list_remote_dir(&host_id, &remote_path)).await
}

#[tauri::command]
pub async fn read_remote_file(
    host_id: String,
    remote_path: String,
) -> Result<Vec<u8>, CatermError> {
    run_blocking(move || sftp::read_remote_file(&host_id, &remote_path)).await
}

#[tauri::command]
pub async fn write_remote_file(
    host_id: String,
    remote_path: String,
    data: Vec<u8>,
)| -> Result<(), CatermError> {
    run_blocking(move || sftp::write_remote_file(&host_id, &remote_path, &data)).await
}

#[tauri::command]
pub async fn mkdir_remote_dir(host_id: String, remote_path: String) -> Result<(), CatermError> {
    run_blocking(move || sftp::mkdir_remote_dir(&host_id, &remote_path)).await
}

#[tauri::command]
pub async fn delete_remote_file(host_id: String, remote_path: String) -> Result<(), CatermError> {
    run_blocking(move || sftp::delete_remote_file(&host_id, &remote_path)).await
}

#[tauri::command]
pub async fn sftp_rename(
    host_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), CatermError> {
    run_blocking(move || sftp::rename_remote_file(&host_id, &old_path, &new_path)).await
}

#[tauri::command]
pub async fn sftp_copy(
    host_id: String,
    src_path: String,
    dst_path: String,
) -> Result<(), CatermError> {
    run_blocking(move || sftp::copy_remote_file(&host_id, &src_path, &dst_path)).await
}
#[tauri::command]
pub async fn export_encrypted_backup(passphrase: String) -> Result<String, CatermError> {
    run_blocking(move || backup::export_encrypted_backup(&passphrase)).await
}

#[tauri::command]
pub async fn import_encrypted_backup(
    encrypted_b64: String,
    passphrase: String,
) -> Result<usize, CatermError> {
    run_blocking(move || backup::import_encrypted_backup(&encrypted_b64, &passphrase)).await
}
#[tauri::command]
pub async fn list_keys() -> Result<Vec<keys::KeyRecord>, CatermError> {
    run_blocking(keys::list_keys).await
}

#[tauri::command]
pub async fn poll_active_metrics() -> Result<Vec<monitor::HostMetrics>, CatermError> {
    run_blocking(monitor::poll_active_metrics).await
}

#[tauri::command]
pub async fn generate_key(input: keys::KeyInput) -> Result<keys::KeyRecord, CatermError> {
    run_blocking(move || keys::generate_key(input)).await
}

#[tauri::command]
pub async fn import_key(
    name: String,
    private_key_pem: String,
    passphrase: Option<String>,
) -> Result<keys::KeyRecord, CatermError> {
    run_blocking(move || keys::import_key(&name, &private_key_pem, passphrase.as_deref())).await
}

#[tauri::command]
pub async fn delete_key(id: String) -> Result<(), CatermError> {
    run_blocking(move || keys::delete_key(&id)).await
}

#[tauri::command]
pub async fn deploy_public_key(host_id: String, key_id: String) -> Result<(), CatermError> {
    run_blocking(move || keys::deploy_public_key(&host_id, &key_id)).await
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
pub async fn save_snippet(
    input: snippets::SnippetInput,
) -> Result<snippets::SnippetRecord, CatermError> {
    run_blocking(move || snippets::save_snippet(input)).await
}

#[tauri::command]
pub async fn delete_snippet(id: String) -> Result<(), CatermError> {
    run_blocking(move || snippets::delete_snippet(&id)).await
}

#[tauri::command]
pub async fn validate_vault_password(password: String) -> Result<(), CatermError> {
    run_blocking(move || vault::unlock_vault(&password)).await
}

#[tauri::command]
pub async fn get_command_logs(
    host_id: Option<String>,
    search: Option<String>,
) -> Result<Vec<audit::CommandLog>, CatermError> {
    run_blocking(move || audit::get_logs(host_id.as_deref(), search.as_deref())).await
}

#[tauri::command]
pub async fn is_vault_initialized() -> Result<bool, CatermError> {
    run_blocking(vault::is_vault_initialized).await
}

#[tauri::command]
pub async fn reset_vault() -> Result<(), CatermError> {
    run_blocking(vault::reset_vault).await
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

#[tauri::command]
pub async fn list_teams() -> Result<Vec<teams::TeamRecord>, CatermError> {
    run_blocking(teams::list_teams).await
}

#[tauri::command]
pub async fn save_team(input: teams::TeamInput) -> Result<teams::TeamRecord, CatermError> {
    run_blocking(move || teams::save_team(input)).await
}

#[tauri::command]
pub async fn delete_team(id: String) -> Result<(), CatermError> {
    run_blocking(move || teams::delete_team(&id)).await
}

#[tauri::command]
pub async fn list_investigations() -> Result<Vec<investigations::InvestigationRecord>, CatermError>
{
    run_blocking(investigations::list_investigations).await
}

#[tauri::command]
pub async fn save_investigation(
    input: investigations::InvestigationInput,
) -> Result<investigations::InvestigationRecord, CatermError> {
    run_blocking(move || investigations::save_investigation(input)).await
}

#[tauri::command]
pub async fn delete_investigation(id: String) -> Result<(), CatermError> {
    run_blocking(move || investigations::delete_investigation(&id)).await
}

#[tauri::command]
pub async fn get_ai_settings() -> Result<ai::AiSettings, CatermError> {
    run_blocking(ai::get_ai_settings).await
}

#[tauri::command]
pub async fn save_ai_settings(
    settings: Option<ai::AiSettings>,
    input: Option<ai::AiSettings>,
) -> Result<ai::AiSettings, CatermError> {
    let s = settings
        .or(input)
        .unwrap_or_else(|| ai::AiSettings::default());
    run_blocking(move || ai::save_ai_settings(s)).await
}

#[tauri::command]
pub async fn ai_generate_plan(
    goal: String,
    host_id: Option<String>,
) -> Result<ai::AiExecutionPlan, CatermError> {
    run_blocking(move || ai::generate_plan(&goal, host_id.as_deref())).await
}

#[tauri::command]
pub async fn ai_chat(
    messages: Vec<ai::AiChatMessage>,
    host_label: Option<String>,
) -> Result<ai::AiChatReply, CatermError> {
    run_blocking(move || ai::chat(messages, host_label.as_deref())).await
}

#[tauri::command]
pub async fn ai_execute_step(
    host_id: String,
    command: String,
) -> Result<ai::AiExecutionResult, CatermError> {
    run_blocking(move || ai::execute_plan_step(&host_id, &command)).await
}
