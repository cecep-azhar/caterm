# CATerm Open-Core vs Pro Extension Architecture

## 1. Executive Summary

CATerm mengadopsi model pemisahan arsitektur **Open-Core**:
- **Community Edition (Open-Core / MIT):** Bebas dan terbuka untuk umum. Seluruh fitur lokal (SSH PTY, SFTP Dual-Pane, Zero-Knowledge Local Vault, Local Terminal Monitoring, Command Logs/Audit, Local Snippets, TOFU Anti-MITM, CLI `catermctl`, serta BYO AI API Key unlimited) berjalan tanpa batasan dan tanpa ketergantungan server.
- **Pro Extension (Commercial Tier / Closed/Proprietary Module):** Fitur yang mengonsumsi resource cloud, fleet synchronization, atau enkripsi online enterprise diisolasi dalam modul terpisah (`caterm-pro`).

Tujuan utama pemisahan ini adalah **mencegah bypass lisensi hanya dengan mengubah `isPro = true` di source code lokal**.

---

## 2. Struktur Crates & Cargo Workspace

Workspace Cargo (`Cargo.toml`) mengatur modul-modul berikut:

```toml
[workspace]
resolver = "2"
members = [
    "crates/caterm-core",
    "crates/caterm-cli",
    "crates/caterm-app",
    "crates/caterm-pro",
]
```

### Modul `crates/caterm-pro`:
1. `license.rs`:
   - Validasi kriptografis Ed25519 token yang diterbitkan oleh backend GCC.
   - Pengecekan hardware ID (HWID), expiry timestamp, anti-tampering monotonic clock ratchet, dan claims fitur.
2. `vault_sync.rs`:
   - Enkripsi dan dekripsi zero-knowledge vault menggunakan kunci lokal DEK (256-bit AES-GCM) dengan integritas SHA-256.
   - Payload sinkronisasi dienkripsi sebelum dikirim ke endpoint cloud GCC.
3. `gcc_unlock.rs`:
   - Protokol online runtime handshake untuk ephemeral token/key dengan backend GCC sebelum modul fleet armada dibuka.

---

## 3. Cargo Feature Flags (`caterm-app`)

Konfigurasi feature flags di `crates/caterm-app/Cargo.toml`:

```toml
[dependencies]
caterm-core = { path = "../caterm-core" }
caterm-pro = { path = "../caterm-pro", optional = true }
tauri = { version = "2", features = [] }

[features]
default = ["pro"]
community = []
pro = ["dep:caterm-pro"]
```

### Mekanisme Isolasi Binary:
- **Build Community Edition:**
  ```bash
  cargo build -p caterm-app --no-default-features --features community
  ```
  Pada mode ini, crate `caterm-pro` sama sekali **tidak di-compile dan tidak di-link** ke dalam binary binary hasil build. Mengubah variabel frontend di mode community tidak akan membuka fungsionalitas Pro karena kode backend-nya tidak ada dalam binary.
- **Build Commercial Pro Edition:**
  ```bash
  cargo build -p caterm-app --features pro
  ```
  Mengikutsertakan modul kriptografi `caterm-pro` dan mengaktifkan fungsionalitas cloud sync & runtime handshake.

---

## 4. UI Pro Gate Guard (`frontend`)

Di frontend, perlindungan visual dan reaktif disediakan oleh komponen `ProGate.svelte` (`frontend/src/lib/components/ProGate.svelte`):
- Membungkus komponen konten yang sensitif Pro via slot/snippet.
- Jika `!pro.isPro`:
  - Menampilkan overlay gelap dengan backdrop blur (`backdrop-blur-md`).
  - Ikon gembok emas `🔒` dengan efek subtle pulse.
  - Judul "Fitur Pro Eksklusif", deskripsi fitur, tombol aktivasi ($10/bulan), dan navigasi ke pengaturan akun.
- Titik integrasi UI:
  1. `Network Diagnostics & Server Benchmark Pro` (Tab Benchmark di `NetworkAuditModal.svelte`).
  2. `Mesh Latency Matrix & Diagnostics Lab Pro` (Tab Mesh & Tools di `NetworkAuditModal.svelte`).
  3. `Directory & Broadcast Sync Pro` (`DirectorySync.svelte` di SFTP modal dan shortcut toggle Broadcast input di `+layout.svelte`).

---

## 5. Matriks Keamanan & Anti-Tampering

| Vektor Serangan | Mitigasi Arsitektur |
| :--- | :--- |
| Clone repo & set `isPro = true` di JS/TS | Modul backend Pro tidak ada di binary Community; request RPC akan gagal atau kode tidak ter-link. |
| Fake Token di local storage/SQLite | Token diverifikasi kriptografis Ed25519 menggunakan compiled-in public key; token tanpa signature valid ditolak. |
| Mundurkan jam sistem (Clock Rollback) | Monotonic clock ratchet menolak token jika `now` mundur melampaui toleransi jitter (300 detik). |
| MITM Proxy / Local GCC server palsu | Handshake ephemeral key dan public key pinning mencegah server palsu menerbitkan token tanpa private key resmi. |
