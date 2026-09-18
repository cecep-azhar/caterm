//! CATerm v2 core library. Pure Rust — no Tauri, no GUI toolkit, no WebView.
//! Every public function here must be callable and testable without a display server.

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

pub mod error;
pub mod paths;

pub use error::CatermError;

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
