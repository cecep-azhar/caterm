# CATerm v2 Ledger & Audit

## Audit Realita (19 Sep 2026)
| # | Pertanyaan | Status Aktual |
|---|---|---|
| A-1 | Vault Argon2id sungguhan? | `[SELESAI]` Modul KEK menggunakan Argon2id dan Zeroize diterapkan. |
| A-2 | `ssh.rs` memakai `russh` atau `ssh2`? | `[SELESAI]` Tetap memakai `ssh2` untuk rilis 2.x demi stabilitas modul lain (tunnels/monitor/SFTP) yang bergantung padanya. Interface asinkron disimulasikan lewat tokio mpsc & thread worker. |
| A-3 | TOFU host-key verification? | `[SELESAI]` Diimplementasikan di `caterm-core/src/ssh.rs` dengan pencatatan & verifikasi `known_hosts` otomatis! |
| A-4 | `catermctl` CLI commands? | `[REGRESI]` Belum ada subcommands untuk host/vault/connect/tunnel/monitor/key/audit. |
| A-5 | SFTP (Fase 4)? | `[SELESAI]` Ditambahkan via `crates/caterm-core/src/sftp.rs` beserta antarmuka UI. |

## Utang Teknis & Task Selesai (19 Sep 2026)
- [x] **Clippy Cleanup**: 14 titik `.unwrap()` di `crates/caterm-app/src/commands.rs` diatasi dengan `run_blocking` helper. `cargo clippy --workspace` kini lulus 0 error/0 warning!
- [x] **`T2-SSH-04` — TOFU Host Key Verification**: Menambahkan verifikasi `known_hosts` otomatis pada `ssh::connect`. Celah MITM berhasil ditutup.
- [x] **`T2-TOOL-05` — SSH Keys Manager**: Implementasi backend Rust (`caterm-core/src/keys.rs`) & UI SvelteKit (`/ssh-keys`). Private key disimpan terenkripsi AES-256-GCM di SQLite database lokal.
- [x] **`T2-TOOL-06` — Deploy public key**: Fitur `Deploy to Server` di UI SSH Keys. Deploy idempoten ke `~/.ssh/authorized_keys` menggunakan backend `ssh2` channel exec.
- [x] **`T2-TOOL-02` — Port Forwarding**: Manajer tunneling dengan mode *Local*, *Remote*, dan *Dynamic*. Terikat ke ID host. Otomatis membersihkan socket *direct-tcpip* saat ditutup. Backend Rust `caterm-core/src/tunnels.rs` + UI `/port-forwarding`.
- [x] **`T2-TOOL-07` — Server Monitoring**: Menampilkan pemakaian real-time (CPU, RAM, Root Disk, OS, Uptime) pada tab Monitoring untuk seluruh host aktif. Menggunakan skrip *agentless* Unix yang di-_poll_ via SSH `exec`. Backend Rust `caterm-core/src/monitor.rs` + UI SvelteKit `/monitoring`.
- [x] **`T2-SFTP-01` — SFTP / Remote File Manager**: Implementasi di `sftp.rs` untuk baca/tulis/hapus file jarak jauh secara efisien lewat PTY koneksi SSH yang sedang aktif. UI disediakan di tab SvelteKit `/sftp`.
- [x] **`T2-CORE-01` — Argon2id Vault (Phase 1)**: Modul KEK `vault.rs` menggunakan algoritma sandi `argon2` m=64MB, t=3, p=4. Kunci memori dihapus otomatis dengan `zeroize` saat app di-lock. Celah fallback `vault.key` plaintext dihapus.
- [x] **`T2-EXIM-01` — Ekspor/Impor Kredensial**: Encrypted JSON Vault Backup (`backup.rs`). Kunci turunan diturunkan (Argon2id) dari `Backup Passphrase`. Import mendeskripsi format secara aman. Telah diuji dengan real data untuk memastikan tidak ada plaintext leakage (T2-EXIM-01 roundtrip test). Didukung UI Settings > *Backup & Restore*. Minimum password 8 karakter diterapkan pada UI & Backend.

## Keputusan Desain (Membutuhkan Konfirmasi Pemilik)
* **DX-1 (ssh2 vs russh)**: Default -> **DIPUTUSKAN**. Kita tetap menggunakan `ssh2` pada rilis 2.x. Thread worker dan mpsc channels sudah memberikan antarmuka asinkron yang cukup stabil untuk modul lain (Agent 4/5/7: tunnels, monitor, sftp, logs) tanpa menyebabkan breaking changes pada rilis ini. TOFU berjalan baik via libssh2 native `known_hosts`.
* **DX-2 (Kerjakan TOFU sebelum Power Tools?)**: Default -> **SELESAI DULUAN**. Verifikasi host key TOFU (`REQ-18`/`T2-SSH-04`) telah selesai dikerjakan sebelum SSH Keys Manager.- [x] **Audit & Fix (21 Sep 2026)**: Verified that Hosts, Groups, Snippets, and SSH Keys fully persist to SQLite vault via caterm-core and work across restarts. Updated Hosts UI toolbar to include select mode, connect button, and details panel button.
