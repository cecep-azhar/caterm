# CATerm v2

> **High-Performance, Zero-Knowledge, Multi-Platform SSH Terminal & DevOps Client**  
> Built primarily with **Rust** (Tauri v2 + Tokio + SQLCipher) and SvelteKit.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS%20%7C%20Android-brightgreen.svg)](#supported-platforms)
[![Rust](https://img.shields.io/badge/Core-Rust%202024-orange.svg)](https://www.rust-lang.org/)
[![GitHub Issues](https://img.shields.io/badge/Issues-GitHub-blue)](https://github.com/cecep-azhar/caterm/issues)
[![Support on Ko-fi](https://img.shields.io/badge/Support-Ko--fi-FF5E5B?logo=ko-fi&logoColor=white)](https://ko-fi.com/cecepazhar)

---

## ⚡ Overview

**CATerm** adalah desktop terminal emulator dan server management workspace generasi modern yang direkayasa dengan **Rust (Edition 2024)** untuk performa maksimal, latensi ultra rendah, dan privasi tanpa kompromi (Zero-Knowledge). Menggabungkan PTY SSH client byte-by-byte streaming, WinSCP-parity dual-pane SFTP manager, remote system monitoring real-time, zero-knowledge encrypted vault (SQLCipher + Argon2id), master multi-agent AI routing engine, serta extensible custom skills.

CATerm dioptimalkan berjalan di native binary dengan footprint memori sangat hemat (<180MB RSS) menggunakan `mimalloc` dan headless WebKit rendering, tanpa bloatware web iframe wrapper biasa.

---

## 🖥️ Supported Platforms

| Platform | Packages / Bundles | Status |
| :--- | :--- | :--- |
| **Linux (Fedora / RHEL)** | `.rpm` (x86_64, aarch64) | ✅ Active & Tested |
| **Linux (Debian / Ubuntu)** | `.deb` (x86_64, aarch64) | ✅ Active & Tested |
| **Linux (Universal)** | `.AppImage`, `.tar.gz` | ✅ Active & Tested |
| **Windows 10 / 11** | Native `.exe` Installer & Portable (x64) | ✅ Active & Tested |
| **macOS** | `.dmg` (Apple Silicon & Intel) | 🔄 Universal build ready |
| **Android** | `.apk` (ARM64 debug/release ready) | ✅ Available & Tested |

---

## 🚀 Key Features

### 1. ⚡ Zero-Delay SSH PTY Terminal
- Arsitektur non-blocking async streaming berbasis Rust `ssh2` dan Tokio.
- Byte-by-byte immediate flush — bebas input delay pada pengetikan cepat dan throughput streaming tinggi.
- xterm.js WebGL renderer dengan 256 colors, true color 24-bit, clipboard copy/paste aman.

### 2. 🔄 Multi-Session Persistence & Split Panes
- Tab dan split grid multi-terminal simultan (hingga 4 pane aktif).
- Background connection persistence: penambahan host baru atau beralih sesi tidak memutuskan proses remote yang sedang berjalan.

### 3. 📁 Dual-Pane SFTP / VFS Manager (WinSCP Parity)
- Dual-pane side-by-side local filesystem vs remote server.
- Stream chunked 64KB upload/download dengan progress tracking dan cancellation token.
- Keyboard shortcuts (`F2` Rename, `F5` Copy, `F7` New Folder, `F8` Delete, `Ctrl+R` Refresh).
- Visual permissions viewer dan chmod editor.
- Dukungan protokol VFS multi-storage (SFTP, Local, SCP, FTP/FTPS, WebDAV, S3-compatible).

### 4. 🔒 Zero-Knowledge Encrypted Vault
- Enkripsi database lokal menggunakan **SQLCipher (AES-256-GCM)** dengan kunci turunan **Argon2id**.
- Perlindungan kredensial: Host passwords, SSH Private Keys, TOTP 2FA tokens, dan API key AI tersimpan aman secara offline.
- Zero telemetry, zero key upload. Format backup/restore terenkripsi `.catb`.

### 5. 🤖 Hana AI DevOps Co-Pilot & Master Router
- Master Multi-Agent Router yang mengarahkan tugas DevOps (Chat, Diagnostic, Script Generator, Security Review) ke provider paling optimal.
- Dukungan model luas: CATerm Pro Hosted AI (OmniRoute/9Router failover) serta Bring Your Own Key (Claude, OpenAI, Gemini, DeepSeek, Local Ollama).
- PII Scrubber & Secret Filter otomatis menyaring token, password, dan kunci privat sebelum diteruskan ke upstream API.
- Custom Skills framework (e.g. `@k8s-triage`, `@laravel-deploy`) dan Persistent Habit Memory berbasis SQLite FTS5.

### 6. 📊 Real-Time Server Monitoring & Diagnostics
- Polling telemetri non-interactive via SSH streaming langsung (CPU, RAM, Disk, Load, Network I/O) tanpa perlu instalasi agent di target server.
- Diagnostics Lab & Network Benchmark Suite (latensi mesh, port scanning, audit keamanan).

### 7. 🌐 Port Forwarding & SSH Tunnels
- Local, Remote, dan Dynamic SOCKS5 proxy tunnels dengan visual status indicator.

### 8. 🛡️ TOTP 2FA Authenticator & Security Suite
- Terintegrasi authenticator TOTP RFC 6238 langsung di dalam vault CATerm.
- TOFU (Trust-On-First-Use) Host Key verification untuk pencegahan Man-in-the-Middle (MITM).

---

## 🛠️ Workspace Architecture

```text
caterm/
├── crates/
│   ├── caterm-core/      # Rust logic murni: Vault (SQLCipher), SSH/SFTP, VFS, AI Router/Memory, TOTP, Audit
│   ├── caterm-cli/       # catermctl utility: benchmark, terminal harness, vault migration
│   ├── caterm-app/       # Tauri v2 native desktop wrapper (IPC handlers, system tray, window lifecycle)
│   └── caterm-pro/       # Commercial extension: Ed25519 GCC license verification, E2EE sync payload
├── frontend/             # SvelteKit + Tailwind CSS + xterm.js desktop UI
├── docs/                 # Panduan teknis arsitektur, signing, lisensi, dan peluncuran
└── scripts/              # Build, release packaging, dan budget validation
```

---

## 📦 Building from Source

### Prerequisites
- **Rust toolchain 1.85+ (Edition 2024)**: `rustup update`
- **Node.js 20+** & **npm**
- **Sistem Dependency**:
  - *Fedora / RHEL*: `sudo dnf install webkit2gtk4.1-devel openssl-devel libssh2-devel libappindicator-gtk3-devel`
  - *Ubuntu / Debian*: `sudo apt install libwebkit2gtk-4.1-dev libssl-dev libssh2-1-dev libayatana-appindicator3-dev`
  - *Windows*: Visual Studio C++ Build Tools & WebView2 runtime

### 1. Build Frontend
```bash
cd frontend
npm install
npm run build
cd ..
```

### 2. Run Desktop App (Dev Mode)
```bash
cargo tauri dev
```

### 3. Build Production Binary
```bash
# Community Open-Core Build
cargo build -p caterm-app --no-default-features --features community --release

# Pro Commercial Build
cargo build -p caterm-app --features pro --release
```

---

## 📄 License & Attribution
- Core / Community Edition dilisensikan di bawah [MIT License](LICENSE).
- Komponen sinkronisasi armada dan hosting AI diatur di bawah lisensi komersial CATerm Pro.
