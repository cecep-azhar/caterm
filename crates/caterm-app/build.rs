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
    "validate_vault_password",
    "ssh_connect",
    "ssh_write",
    "ssh_resize",
    "ssh_disconnect",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("gagal menjalankan tauri-build");
}
