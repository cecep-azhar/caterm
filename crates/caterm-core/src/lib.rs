//! CATerm v2 core library. Pure Rust — no Tauri, no GUI toolkit, no WebView.
//! Every public function here must be callable and testable without a display server.

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
