# CATerm System Architecture

## 1. High-Level Architecture
CATerm is a high-performance, zero-knowledge, multi-platform SSH terminal and DevOps management desktop application.

```
┌─────────────────────────────────────────────────────────────┐
│                 Frontend (SvelteKit + xterm.js)             │
│   - UI Shell, Ambient Glow, Theme & CADS Design Tokens      │
│   - Terminal View, Split Panes, VFS/SFTP Dual-Pane          │
│   - AI Smart Assistant, Prompt Studio, Network Diagnostics  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Tauri v2 IPC (Events/Commands)
┌──────────────────────────────▼──────────────────────────────┐
│                    Tauri Native App Shell                   │
│   - `caterm-app`: Window lifecycle, system tray, IPC router │
│   - Menu management, security capabilities, OS permissions  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Rust Internal Crate API
┌──────────────────────────────▼──────────────────────────────┐
│                    Rust Core & Pro Crates                   │
│  ┌────────────────────────┐    ┌─────────────────────────┐  │
│  │     `caterm-core`      │    │      `caterm-pro`       │  │
│  │ - PTY Engine (ssh2)    │    │ - Ed25519 GCC License   │  │
│  │ - VFS Multi-protocol   │    │ - Monotonic Clock Guard │  │
│  │ - SQLCipher Vault      │    │ - Zero-Knowledge Cloud  │  │
│  │ - Master AI Router     │    │   Sync Engine           │  │
│  │ - FTS5 Habit Memory    │    │                         │  │
│  │ - TOTP Authenticator   │    │                         │  │
│  └────────────────────────┘    └─────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Encrypted File I/O
┌──────────────────────────────▼──────────────────────────────┐
│        Encrypted Local Storage (SQLCipher + Argon2id)       │
│   - Master Vault, Hosts, Snippets, Groups, Keys, Audit      │
└─────────────────────────────────────────────────────────────┘
```

## 2. Workspace Crates Hierarchy
- `crates/caterm-core`: Pure Rust domain core.
  - `ssh.rs`: Raw PTY streaming, multi-session Tokio async execution, TOFU host key pinning.
  - `vault.rs` & `db.rs`: SQLCipher encrypted storage, Argon2id derivation, key management.
  - `vfs.rs` & `sftp.rs`: Dual-pane virtual filesystem with SFTP, Local, SCP, FTP, WebDAV, S3.
  - `ai/`: Master AI Router, PII scrubber, context builder, FTS5 habit learner, custom skills (`@k8s-triage`).
  - `totp.rs`: RFC 6238 2FA vault authenticator.
  - `monitor.rs`: Remote telemetry polling (CPU, RAM, Disk, Network I/O).
  - `tunnels.rs`: Local, Remote, Dynamic SOCKS5 port forwarder.
  - `network_audit.rs`: Mesh latency matrix and security diagnostic suite.
- `crates/caterm-app`: Desktop shell (Tauri v2).
  - `commands.rs`: Registration and sanitization of Tauri IPC commands.
  - `lib.rs`: Plugin initialization, tray setup, window management.
- `crates/caterm-pro`: Commercial extension.
  - `license.rs`: Cryptographic Ed25519 verification against GCC licensing server.
  - `vault_sync.rs`: E2EE encrypted vault sync serialization.
  - `gcc_unlock.rs`: Hardware ID and online handshake authorization.
- `crates/caterm-cli`: Standalone CLI (`catermctl`) for headless terminal commands and migration harness.

## 3. Data & Zero-Knowledge Security Model
- **Encryption**: AES-256-GCM through SQLCipher. Master passkey never saved in plaintext; derived using Argon2id.
- **AI Privacy & Sanitization**: Regex scrubber replaces passwords, private keys, and session tokens before dispatch to LLM providers.
- **Local-First**: All hosts, snippets, logs, and habit memory reside locally on user device by default. Cloud sync requires opt-in Pro license and end-to-end client envelope encryption.
