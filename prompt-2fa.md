# 🚀 Autopilot Master Prompt — CATerm 2FA / TOTP Authenticator & SSH Auto-Inject (REQ-37)

> **Dokumen Terkait:** `Notes/4 - Product/2 CATerm/prd-v2.md` (REQ-37), `caterm` repository (`/home/cecepazhar/Product/caterm`).  
> **Tujuan:** Mengimplementasikan fitur **Zero-Knowledge 2FA / TOTP Authenticator Vault (RFC 6238)** dan **SSH Auto-Inject** di CATerm v2 secara otonom penuh (*autopilot*) dengan pembagian tugas sub-agent yang terkoordinasi dan terverifikasi di setiap gerbang (*gate*).

---

## 🎯 Ringkasan Eksekutif & Arsitektur Fitur (REQ-37)

Fitur ini menyelesaikan masalah developer/sysadmin yang harus membuka HP untuk menyalin kode 2FA setiap kali login SSH ke server produksi, jump host, atau bastion.

### 3 Skenario Utama:
1. **Host 2FA Slot & SSH Auto-Inject:**
   - Kunci secret Base32 disimpan terenkripsi di SQLCipher per Host.
   - Saat sesi SSH menerima prompt `keyboard-interactive` (`Verification code:`, `OTP:`, `2FA:`), backend Rust otomatis mengkalkulasi token 6 digit dan menyuntikkannya ke socket SSH atau menampilkan badge 1-klik di atas terminal.
2. **Dedicated 2FA Authenticator Panel (Sidebar Tab):**
   - Panel pengelola seluruh token 2FA server & akun cloud (AWS, Cloudflare, GitHub, Email).
   - Visual UI: 6 digit kode besar (`358 630`), lingkaran countdown 30 detik (SVG stroke-dashoffset), dan tombol 1-Click Copy.
3. **Zero-Knowledge & Zero-Overhead:**
   - Komputasi RFC 6238 instan (< 1 mikrodetik), zero-allocation, RAM < 10 KB, secret key selalu terenkripsi dengan Argon2id + SQLCipher DEK.

---

## 🤖 Pembagian Tugas Sub-Agent (Workstreams)

```
┌────────────────────────────────────────────────────────────────────────┐
│                          ORCHESTRATOR AGENT                            │
│           (Koordinasi, Verifikasi Gate & Pengujian E2E)                │
└───────┬───────────────────┬───────────────────┬────────────────┬───────┘
        │                   │                   │                │
        ▼                   ▼                   ▼                ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐   ┌────────────┐
│  SUB-AGENT 1  │   │  SUB-AGENT 2  │   │  SUB-AGENT 3  │   │ SUB-AGENT 4│
│  Core Engine  │   │  SSH Protocol │   │  Svelte 5 UI  │   │  QA & Test │
│ & Kripto DB   │   │ & Auto-Inject │   │ & Visual Timer│   │  Vectors   │
└───────────────┘   └───────────────┘   └───────────────┘   └────────────┘
```

---

### 🔹 **SUB-AGENT 1: Core Engine RFC 6238 & Database Vault Schema**
* **Target Direktori:** `crates/caterm-core/`
* **Tugas:**
  1. Buat modul `crates/caterm-core/src/totp.rs`:
     - Implementasi RFC 6238 TOTP (HMAC-SHA1, default interval 30 detik, 6 digit).
     - Parser Base32 secret string (mengabaikan spasi/tanda hubung, sanitasi case-insensitive).
     - Parser standard `otpauth://totp/...` URI.
     - Fungsi: `generate_totp(secret: &str, time_offset_sec: i64) -> Result<(String, u64), CatermError>` (mengembalikan 6 digit string & sisa detik).
  2. Database Migrations di `crates/caterm-core/src/store.rs`:
     - Tambahkan kolom `totp_secret_enc TEXT` pada tabel `hosts`.
     - Buat tabel baru `totp_entries`:
       ```sql
       CREATE TABLE IF NOT EXISTS totp_entries (
         id TEXT PRIMARY KEY,
         label TEXT NOT NULL,
         issuer TEXT,
         secret_enc TEXT NOT NULL,
         created_at TEXT NOT NULL DEFAULT (datetime('now')),
         updated_at TEXT NOT NULL DEFAULT (datetime('now'))
       );
       ```
  3. Expose IPC Commands di `crates/caterm-app/src/commands.rs`:
     - `generate_current_totp(secret_or_id: String)`
     - `list_totp_entries() -> Vec<TotpEntryRecord>`
     - `save_totp_entry(input: TotpEntryInput) -> TotpEntryRecord`
     - `delete_totp_entry(id: String)`
* **Gate 1:** `cargo check -p caterm-core -p caterm-app` PASS.

---

### 🔹 **SUB-AGENT 2: SSH Keyboard-Interactive Interceptor & Auto-Inject**
* **Target Direktori:** `crates/caterm-core/src/ssh.rs`
* **Tugas:**
  1. Pada handler `keyboard-interactive` autentikasi SSH di `ssh.rs`:
     - Deteksi prompt instruksi server (regex case-insensitive: `verification code|one-time|otp|2fa|authenticator`).
     - Jika host memiliki `totp_secret` yang terdaftar:
       - Hitung token TOTP 6 digit saat itu.
       - Bila opsi `auto_inject_totp = true`: langsung jawab prompt server dengan token tersebut.
       - Kirim event Tauri `ssh:totp_prompt` ke frontend berisi ID host dan sisa waktu aktif.
* **Gate 2:** `cargo test -p caterm-core --test ssh*` PASS.

---

### 🔹 **SUB-AGENT 3: Frontend Svelte 5 UI, Visual Countdown & Authenticator Tab**
* **Target Direktori:** `frontend/src/`
* **Tugas:**
  1. Buat komponen timer melingkar `frontend/src/lib/components/TotpBadge.svelte`:
     - SVG radial circle dengan `stroke-dashoffset` CSS smooth transition.
     - Tampilan 6 digit format terpisah `358 630`.
     - Tombol 1-Click Copy ke clipboard dengan feedback visual ("Tersalin!").
  2. Buat halaman/tab pengelola `frontend/src/routes/totp/+page.svelte`:
     - Daftar kartu token 2FA (nama server/akun, penerbit/issuer, sisa detik, tombol salin, tombol edit/hapus).
     - Tombol "Tambah Token 2FA" (mendukung input manual Base32 secret atau paste URL `otpauth://`).
  3. Integrasi pada Form Host (`HostModal.svelte` / `HostForm.svelte`):
     - Slot input baru: `2FA / TOTP Secret Key (Opsional)` dengan toggle test generate langsung untuk memverifikasi kunci sebelum disimpan.
  4. Floating Action Prompt di Terminal Pane:
     - Saat prompt SSH 2FA terdeteksi, munculkan badge mengambang di pojok terminal: `[⚡ Isi 2FA: 358 630 (22s)]` yang otomatis mengetikkan token saat diklik.
* **Gate 3:** `npm run build` dan `npm test` di `frontend/` PASS tanpa TypeScript / lint error.

---

### 🔹 **SUB-AGENT 4: Unit Testing RFC Vectors, Zeroization & Security Audit**
* **Target Direktori:** `crates/caterm-core/tests/`
* **Tugas:**
  1. Buat test suite `crates/caterm-core/tests/totp_rfc6238_test.rs`:
     - Uji kecocokan hasil terhadap RFC 6238 Test Vectors resmi (pada timestamp 59s, 1111111109s, 1111111111s, 1234567890s, 2000000000s).
     - Uji sanitasi spasi, tanda strip, dan lower/upper case Base32.
  2. Uji Keamanan Zero-Knowledge:
     - Pastikan `totp_secret` di-*zeroize* dari memori setelah token digenerate (`zeroize` crate).
     - Pastikan `totp_secret` tersimpan dalam kolom ciphertext `_enc` di SQLCipher.
* **Gate 4:** `cargo test --workspace` 100% PASS.

---

## 🚦 Aturan Eksekusi Otonom (DoD & Verification Gates)

1. **Strict No-Regression:**
   - Seluruh 77+ unit test dan guard arsitektur yang sudah ada tidak boleh ada yang rusak.
2. **Kompilasi Rilis & Paket:**
   - Setelah Sub-agent 1–4 selesai, jalankan build rilis desktop:
     ```bash
     cargo build --release --manifest-path crates/caterm-app/Cargo.toml
     npx --prefix frontend tauri build --config crates/caterm-app/tauri.conf.json
     ```
   - Salin binary ke `~/.local/bin/caterm`.
3. **Laporan Akhir:**
   - Laporkan bukti eksekusi berupa raw output terminal, daftar perubahan file, dan status verifikasi dalam Bahasa Indonesia kepada Prof. Cecep.
