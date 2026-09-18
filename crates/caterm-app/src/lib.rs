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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Unrecoverable: if the Tauri runtime itself fails to start, the process has no
    // useful state to continue in. This is the one sanctioned exception to the
    // zero-panic policy (REQ-04) outside #[cfg(test)] — see ledger-v2.md KA-05.
    // `disallowed_methods` is also allowed here: `tauri::generate_context!()` expands to
    // code that calls `std::process::exit` internally (Tauri's own codegen, not ours) —
    // the lint attributes that call to this statement's span.
    #[allow(clippy::expect_used, clippy::disallowed_methods)]
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
