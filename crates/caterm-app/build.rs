// Every #[tauri::command] registered in `invoke_handler!` (src/lib.rs) must be listed here too
// — Tauri v2 only auto-generates the `allow-<command>`/`deny-<command>` permissions that
// `capabilities/default.json` references for commands named in this list.
const COMMANDS: &[&str] = &[
    "list_hosts",
    "save_host",
    "delete_host",
    "list_groups",
    "save_group",
    "delete_group",
    "list_snippets",
    "save_snippet",
    "delete_snippet",
    "validate_vault_password",
    "is_vault_initialized",
    "reset_vault",
    "ssh_connect",
    "ssh_write",
    "ssh_resize",
    "ssh_disconnect",
    "get_ai_settings",
    "save_ai_settings",
    "ai_generate_plan",
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
