//! Tauri binding layer. Deliberately empty of business logic (K2-1): every
//! `#[tauri::command]` added here must do nothing but deserialize -> call
//! one `caterm_core` function -> serialize. No demo commands (anti-regresi T-19).

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod commands;

/// `start` should be captured as close to `main()`'s first line as possible by the
/// caller, so the printed duration approximates true cold start (REQ-02). Prints
/// `CATERM_COLD_START_MS=<n>` once the backend considers itself ready (main window
/// built). This is a *backend-ready* proxy for now — Fase 0 has no frontend IPC yet
/// (K2-1) and no unlock screen (Fase 3), so it is not yet REQ-02's literal
/// "cold start -> jendela unlock". `T2-BOOT-06` evidence documents this scope honestly;
/// the harness will be repointed at a real frontend-ready signal once Fase 3 ships one.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(start: std::time::Instant) {
    // Unrecoverable: if the Tauri runtime itself fails to start, the process has no
    // useful state to continue in. This is the one sanctioned exception to the
    // zero-panic policy (REQ-04) outside #[cfg(test)] — see ledger-v2.md KA-05.
    // `disallowed_methods` is also allowed here: `tauri::generate_context!()` expands to
    // code that calls `std::process::exit` internally (Tauri's own codegen, not ours) —
    // the lint attributes that call to this statement's span.
    #[allow(clippy::expect_used, clippy::disallowed_methods)]
    tauri::Builder::default()
        .setup(move |_app| {
            println!("CATERM_COLD_START_MS={}", start.elapsed().as_millis());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_hosts,
            commands::save_host,
            commands::delete_host,
            commands::list_groups,
            commands::save_group,
            commands::delete_group,
            commands::list_teams,
            commands::save_team,
            commands::delete_team,
            commands::list_snippets,
            commands::save_snippet,
            commands::delete_snippet,
            commands::list_tunnels,
            commands::save_tunnel,
            commands::delete_tunnel,
            commands::start_tunnel,
            commands::stop_tunnel,
            commands::poll_active_metrics,
            commands::list_remote_dir,
            commands::read_remote_file,
            commands::write_remote_file,
            commands::delete_remote_file,
            commands::export_encrypted_backup,
            commands::import_encrypted_backup,
            commands::list_keys,
            commands::generate_key,
            commands::import_key,
            commands::delete_key,
            commands::deploy_public_key,
            commands::validate_vault_password,
            commands::is_vault_initialized,
            commands::reset_vault,
            commands::get_command_logs,
            commands::ssh_connect,
            commands::ssh_write,
            commands::ssh_read,
            commands::ssh_resize,
            commands::ssh_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
