# CATerm v2 Ledger & Audit

## Audit Realita (19 Sep 2026)
| # | Pertanyaan | Status Aktual |
|---|---|---|
| A-1 | Vault Argon2id sungguhan? | `[REGRESI]` Masih berupa fallback acak 32-byte (`vault::load_or_create_local_key`), belum KEK/DEK Argon2id. |
| A-2 | `ssh.rs` memakai `russh` atau `ssh2`? | `[REGRESI]` Masih memakai `ssh2` (sync blocking + thread worker), bukan `russh` (async native). |
| A-3 | TOFU host-key verification? | `[SELESAI]` Diimplementasikan di `caterm-core/src/ssh.rs` dengan pencatatan & verifikasi `known_hosts` otomatis! |
| A-4 | `catermctl` CLI commands? | `[REGRESI]` Belum ada subcommands untuk host/vault/connect/tunnel/monitor/key/audit. |
| A-5 | SFTP (Fase 4)? | `[REGRESI]` Belum diimplementasikan sama sekali. |

## Utang Teknis & Task Selesai (19 Sep 2026)
- [x] **Clippy Cleanup**: 14 titik `.unwrap()` di `crates/caterm-app/src/commands.rs` diatasi dengan `run_blocking` helper. `cargo clippy --workspace` kini lulus 0 error/0 warning!
- [x] **`T2-SSH-04` — TOFU Host Key Verification**: Menambahkan verifikasi `known_hosts` otomatis pada `ssh::connect`. Celah MITM berhasil ditutup.
- [x] **`T2-TOOL-05` — SSH Keys Manager**: Implementasi backend Rust (`caterm-core/src/keys.rs`) & UI SvelteKit (`/ssh-keys`). Private key disimpan terenkripsi AES-256-GCM di SQLite database lokal.
- [x] **`T2-TOOL-06` — Deploy public key**: Fitur `Deploy to Server` di UI SSH Keys. Deploy idempoten ke `~/.ssh/authorized_keys` menggunakan backend `ssh2` channel exec.
- [x] **`T2-TOOL-02` — Port Forwarding**: Manajer tunneling dengan mode *Local*, *Remote*, dan *Dynamic*. Terikat ke ID host. Otomatis membersihkan socket *direct-tcpip* saat ditutup. Backend Rust `caterm-core/src/tunnels.rs` + UI `/port-forwarding`.
- [x] **`T2-TOOL-07` — Server Monitoring**: Menampilkan pemakaian real-time (CPU, RAM, Root Disk, OS, Uptime) pada tab Monitoring untuk seluruh host aktif. Menggunakan skrip *agentless* Unix yang di-_poll_ via SSH `exec`. Backend Rust `caterm-core/src/monitor.rs` + UI SvelteKit `/monitoring`.

## Keputusan Desain (Membutuhkan Konfirmasi Pemilik)
* **DX-1 (ssh2 -> russh)**: Default -> **DILEWATI SEMENTARA**. Karena Port Forwarding dan Monitoring akan ditulis, saya membuat interface backend `TunnelManager` dan `MonitorManager` yang terisolasi dari core `ssh2::Session` sehingga nanti migrasi ke `russh` dapat dilakukan pada level konektor inti.
* **DX-2 (Kerjakan TOFU sebelum Power Tools?)**: Default -> **SELESAI DULUAN**. Verifikasi host key TOFU (`REQ-18`/`T2-SSH-04`) telah selesai dikerjakan sebelum SSH Keys Manager.