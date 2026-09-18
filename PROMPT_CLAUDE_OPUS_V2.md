# PROMPT UNTUK CLAUDE OPUS — GENERATE PRD-V2.MD & TASK-V2.MD CATERM

```markdown
# PROMPT — GENERATE PRD v2.0 & TASK MATRIX CATerm v2

> **Instruksi Penggunaan:**
> Berikan prompt ini secara utuh kepada Claude Opus (atau AI Agent Senior Architect) untuk menghasilkan dokumen `prd-v2.md` dan `task-v2.md` tingkat produksi bagi proyek **CATerm v2.0**.
> **Author:** Cecep Azhar

---

## ROLE & METODOLOGI

Anda adalah **Senior Principal Systems Architect & Lead Rust Developer**. Anda ditugaskan oleh **Cecep Azhar** untuk menyusun spesifikasi kebutuhan teknis menyeluruh (`prd-v2.md`) dan daftar tugas implementasi terperinci (`task-v2.md`) untuk generasi kedua aplikasi desktop **CATerm v2.0**.

Anda bekerja dengan filosofi:
- **Zero False Completion:** Tidak ada centang `[x]` yang diberikan tanpa unit test/integration test riil yang membuktikan fungsinya.
- **Local-First & Zero-Knowledge Security:** Kredensial tidak pernah diproses di luar perangkat secara unencrypted. Ketiadaan koneksi cloud atau berakhirnya masa langganan TIDAK BOLEH mengunci atau menghapus data lokal pengguna.
- **Extreme Resource Efficiency:** Arsitektur wajib berjalan dengan jejak memori < 35MB RAM.

---

## 1. LATAR BELAKANG & GENESIS (MASALAH TERMIUS & TERMIQUE)

Berdasarkan pengalaman bertahun-tahun menggunakan Termius / Termique di lingkungan multi-device/multi-platform:
1. **Memory Bloat:** Penggunaan RAM mencapai 400MB–1GB+ (Electron engine), membebankan laptop dan baterai.
2. **Subscription Lock-in & Data Revocation:** Saat langganan habis, akses ke seluruh konfigurasi, grup host, snippet, dan preferensi lokal hilang atau dikunci secara sepihak.
3. **Incomplete Backup:** Ekspor lokal tidak pernah mengover seluruh konteks lingkungan (known_hosts, key bindings, environment variables).
4. **Kebocoran Privasi:** Metadata dan kredensial disinkronkan ke cloud dengan enkripsi proprietary yang tidak dapat diaudit.

CATerm v2 dibangun oleh **Cecep Azhar** untuk menghancurkan model lama tersebut dengan **5 Pilar Utama**:
- **Very Light:** Rust + Tauri 2.0 + System Webview (< 35MB RAM, < 8MB binary).
- **Stable:** SQLite WAL mode + Rust async PTY engine (`russh`). Zero panic, 100% thread-safe.
- **Privacy First:** Zero-Knowledge E2EE (Argon2id + AES-256-GCM). Data lokal tersimpan 100% aman.
- **Collaboration Next:** E2EE Team Vault sharing via ECDH (Curve25519). Tambah anggota tim +$1/bulan.
- **Sync Next:** Managed / Self-hostable E2EE Sync Cloud ($1 bulan ke-1, $3/bulan berikutnya). **Jika langganan habis, DATA LOKAL TETAP 100% UTUH DAN AKTIF SELAMANYA.**

---

## 2. DAFTAR FITUR LENGKAP CATERM V2 (ULTRA-DETAIL)

Aplikasi harus mencakup seluruh modul berikut di dalam PRD v2.0:
1. **Hosts & Groups Management:** Hierarki grup, tag, port/user, password/private key terenkripsi.
2. **Interactive PTY Terminal:** xterm.js v5 + Rust `russh` tunnel, 256-color, truecolor, tab, split pane (horizontal/vertikal), bracketed paste.
3. **TOFU Anti-MITM Verification:** Host key verification persisten di SQLite (`host_keys`).
4. **Files Manager (SFTP):** Explorer dua panel (lokal vs remote), drag-and-drop, resume transfer, permission editor.
5. **Snippets Engine:** Skrip CLI dengan variabel dinamis (`{{host}}`, `{{user}}`), auto-enter / inspeksi.
6. **Port Forwarding (Tunnels):** Local, Remote, dan Dynamic (SOCKS5 Proxy) tunneling.
7. **Server Real-time Monitoring:** Real-time CPU, RAM, Disk, Traffic, Load via SSH background stream.
8. **Command Audit Logs:** Centralized command & output logs, auto-truncated ke 5KB.
9. **SSH Keys Manager:** Generator RSA/Ed25519, import/export, `ssh-copy-id` internal.
10. **AI Terminal Copilot:** Opt-in consent, custom provider endpoint, encrypted API key.
11. **E2EE Multi-Device Sync:** *(DISABLED / NEXT FEATURE / UI PLACEHOLDER)* - Siap dihubungkan ke Managed Cloud / Self-hosted Relay.
12. **Team Vault Collaboration:** *(DISABLED / NEXT FEATURE / UI PLACEHOLDER)* - E2EE ECDH sharing.

---

## 3. CATATAN KHUSUS PHASE 2 (SYNC CLOUD & TEAM INFRASTRUCTURE)

Di dalam `prd-v2.md`, dokumentasikan strategi Phase 2 Sync Cloud & Collaboration:
- **Status di Core App:** Fitur Sync dan Team berada dalam kondisi **DISABLED / NEXT FEATURE** (UI Badge *"Coming Soon / Managed Sync"*).
- **Skema Harga Sync Cloud:** 
  - Bulan ke-1: **$1** (Onboarding & card verification).
  - Bulan ke-2 dst: **$3 / bulan**.
  - Team Seats: **+$1 / bulan per anggota tambahan**.
- **Desain Infrastruktur Cloud (FathCloud / Coolify):**
  - Backend Microservice (Go/Rust) terpasang di atas Coolify / Hostinger VPS.
  - Zero-Knowledge Storage Engine: Server HANYA menyimpan encrypted SQLite blob (AES-256-GCM). Kunci tidak pernah bocor ke server.
  - Asymmetric Team Key Exchange (ECDH Curve25519).
  - Opsi Self-Hosted (Docker Container) vs Official Managed Sync Cloud.

---

## 4. FORMAT OUTPUT UNTUK CLAUDE OPUS

Susun dua berkas terpisah secara hiper-detail:
1. **`prd-v2.md`** (Product Requirements Document v2.0 - REQ-01 s/d REQ-35, DDL SQLite lengkap, Keamanan, Monetisasi).
2. **`task-v2.md`** (Implementation Checklist Matrix - Fase 0 s/d Fase 8, Kode unik task `[ ] T2-XXX-01`, Evidence Gate terukur).

Silakan buatkan `prd-v2.md` dan `task-v2.md` sekarang juga secara lengkap, profesional, dan siap langsung digunakan oleh pengembang.
```
