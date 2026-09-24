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

pub mod ai;
pub mod audit;
pub mod backup;
pub mod db;
pub mod error;
pub mod ftp;
pub mod groups;
pub mod investigations;
pub mod keys;
pub mod local_fs;
pub mod monitor;
pub mod paths;
pub mod scp;
pub mod secret;
pub mod sftp;
pub mod snippets;
pub mod ssh;
pub mod store;
pub mod sync;
pub mod teams;
pub mod tunnels;
pub mod vault;
pub mod vfs;
pub mod webdav;
pub mod s3;
pub mod feedback;
pub mod prefs;

pub use error::CatermError;
pub use vfs::RemoteFileSystem;

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
