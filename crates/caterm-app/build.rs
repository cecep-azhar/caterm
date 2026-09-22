// Every #[tauri::command] registered in `invoke_handler!` (src/lib.rs) must be listed here too
// — Tauri v2 only auto-generates the `allow-<command>`/`deny-<command>` permissions that
// `capabilities/default.json` references for commands named in this list.
//
// A command missing from this list still compiles and still registers: it fails only at
// runtime, as `Command <name> not allowed by ACL`, and only on the code path that calls it.
// `caterm-core/tests/arch.rs::tauri_command_registry_build_script_and_acl_agree` keeps this
// list, the invoke handler, and capabilities/default.json in lockstep so that can't recur.
const COMMANDS: &[&str] = &[
    // Hosts
    "list_hosts",
    "save_host",
    "delete_host",
    // Groups
    "list_groups",
    "save_group",
    "delete_group",
    // Teams
    "list_teams",
    "save_team",
    "delete_team",
    // Investigations
    "list_investigations",
    "save_investigation",
    "delete_investigation",
    // Snippets
    "list_snippets",
    "save_snippet",
    "delete_snippet",
    // Port forwarding
    "list_tunnels",
    "save_tunnel",
    "delete_tunnel",
    "start_tunnel",
    "stop_tunnel",
    // Monitoring
    "poll_active_metrics",
    // SFTP / remote files
    "list_remote_dir",
    "read_remote_file",
    "write_remote_file",
    "mkdir_remote_dir",
    "delete_remote_file",
    "sftp_stat",
    "sftp_chmod",
    "sftp_upload",
    "sftp_download",
    "sftp_cancel",
    "sftp_rename",
    "sftp_copy",
    // Local FS
    "local_list_dir",
    "local_stat",
    "local_mkdir",
    "local_delete",
    "local_rename",
    "local_read_file",
    "local_write_file",
    // Window control
    "window_minimize",
    "window_maximize",
    "window_close",
    "window_start_dragging",
    "open_external_url",
    // Backup & restore
    "export_encrypted_backup",
    "import_encrypted_backup",
    // SSH keys
    "list_keys",
    "generate_key",
    "import_key",
    "delete_key",
    "deploy_public_key",
    // Vault
    "validate_vault_password",
    "is_vault_initialized",
    "reset_vault",
    // Audit
    "get_command_logs",
    // SSH terminal
    "ssh_connect",
    "ssh_write",
    "ssh_read",
    "ssh_resize",
    "ssh_disconnect",
    "detect_host_os",
    // AI / Prompt Studio
    "get_ai_settings",
    "save_ai_settings",
    "ai_generate_plan",
    "ai_chat",
    "ai_execute_step",
];

fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("gagal menjalankan tauri-build");
}
