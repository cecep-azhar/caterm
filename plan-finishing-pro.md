# CATerm Pro — Master Plan Finishing & Monetization Readiness (v1.0)

> **Dokumen:** Master Execution Plan Finishing Monetisasi CATerm Pro  
> **Target Rilis:** CATerm Pro Commercial Launch  
> **Status:** Ready for Autopilot AI Execution  
> **Audience:** Core Dev, Sub-Agents (Rust Core, Tauri, Frontend Svelte 5, Backend GCC)

---

## 1. Executive Summary & Monetization Strategy

### 1.1 Model Bisnis & Value Proposition
CATerm mengadopsi model **Open-Core**:
- **Community Edition (100% Free & Open-Source / MIT):** Semua fungsionalitas lokal (SSH PTY, SFTP Dual-Pane, Zero-Knowledge Local Vault, Local Terminal Monitoring, Command Logs/Audit, Local Snippets, TOFU Anti-MITM, CLI `catermctl`, serta BYO AI API Key unlimited).
- **CATerm Pro (Commercial Tier):** Fitur yang membutuhkan infrastruktur server berbiaya dan kolaborasi:
  1. **Cloud Sync E2EE** (Zero-Knowledge sync vault, host list, snippets lintas perangkat).
  2. **AI Ops Copilot Hosted** (300 request/bulan pool bersama tanpa perlu API key sendiri).
  3. **Ruang Tim & Kolaborasi** (1 akun Pro pemilik dapat mengundang hingga 6 rekan tim = total 7 anggota).
  4. **Multi-Device Activation** (Hingga 5 device pribadi untuk Solo, hingga 7 device total untuk Team).
  5. **Founder Perks & Polish** (Custom Ambient Glow Theme presets, priority support).

### 1.2 Strategi Pricing & Dual Payment Gateway
Untuk melayani pasar domestik (Indonesia) dan internasional secara optimal:
- **Pasar Domestik (IDR) — Mayar Gateway:**
  - Integrasi QRIS, Virtual Account (BCA, Mandiri, BRI, BNI), dan E-Wallet.
  - Bulanan: **Rp 99.000 / bulan** (Promo Tahun Pertama: **Rp 59.000 / bulan**).
  - Tahunan: **Rp 990.000 / tahun** (Promo Tahun Pertama: **Rp 594.000 / tahun**).
- **Pasar Global (USD) — Ko-fi / Lemon Squeezy Gateway:**
  - Integrasi Kartu Kredit, PayPal, Apple Pay, Google Pay.
  - Bulanan: **$10 / bulan** (Promo Tahun Pertama: **$6 / bulan** — diskon 40%).
  - Tahunan: **$108 / tahun** (Diskon permanen 10% + Promo Tahun Pertama: **$64.80 / tahun**).
- **Trial 7 Hari Tanpa Kartu Kredit:**
  - Instant activation via desktop client saat registrasi akun baru.
  - Terikat pada HWID unik untuk mencegah trial spamming / abuse.

---

## 2. Gap Analysis & Current State

### 2.1 Rust Core (`caterm-core`)
- **Sudah Ada:**
  - Modul `pro.rs` lengkap dengan evaluasi offline cryptographic Ed25519 token.
  - Anti-tampering monotonic clock ratchet (`CLOCK_DRIFT_TOLERANCE_SECS = 300`).
  - Network client menggunakan `ureq` (blocking, lightweight, no reqwest overhead).
  - Model `Entitlement`, `ProAccount`, `SignedToken`, dan secure local cache di SQLCipher.
- **Kekurangan / To Do:**
  - Public key Ed25519 production injection check (`CATERM_PRO_LICENSE_PUBKEY_V1`).
  - Verifikasi enforcement point pada fitur E2EE Cloud Sync & Hosted AI Dispatcher.

### 2.2 Tauri App Bridge (`caterm-app`)
- **Sudah Ada:**
  - Tauri IPC commands untuk registrasi, login, logout, trial, heartbeat, revoke device, team invite.
  - Autostart / background periodic license heartbeat.
- **Kekurangan / To Do:**
  - Tauri command `open_checkout_url(plan, gateway, email)` untuk seamless deep-linking ke Mayar/Ko-fi dengan parameter email otomatis terisi.

### 2.3 Svelte 5 Frontend (`frontend`)
- **Sudah Ada:**
  - `pro.ts` API client wrapper.
  - `ProLoginForm.svelte`, `ProTeamPanel.svelte`, `AmbientSettingsCard.svelte` dengan gatekeeper check.
  - Pro status badge & reactive state di `pro.svelte.ts`.
- **Kekurangan / To Do:**
  - **Purchase / Upgrade Modal Dialog** yang interaktif dengan pilihan Mayar (IDR) & Ko-fi (USD).
  - Banner status trial countdown ("Sisa Trial: 5 Hari") & Tombol "Upgrade ke Pro".
  - Ambient Glow Theme Lock hardening (mencegah bypass UI state saat entitlement expired).
  - Quota meter visual untuk AI Hosted (e.g. "Sisa Kuota Tim: 280/300 req").

### 2.4 GCC Server (`gcc/apps/gcc/internal/catermpro`)
- **Sudah Ada:**
  - Endpoint Autentikasi (`/auth/register`, `/auth/login`, `/auth/refresh`, `/auth/logout`, `/auth/password/*`).
  - Licensing & Trial endpoints (`/license/trial`, `/license/heartbeat`, `/license/devices/:id/revoke`).
  - Team Collaboration API (`/team`, `/team/invites`, `/team/members/*`).
  - Ed25519 token signing logic di `crypto.go`.
- **Kekurangan / To Do:**
  - Verifikasi kesesuaian Private Key signing GCC dengan Public Key default Rust client.
  - Endpoint Webhook Receiver untuk Mayar (QRIS/VA Settlement) & Ko-fi Payment Webhook.
  - Middleware rate-limiting & quota tracking di `ai_usage` table.

---

## 3. Detailed Action Plan

### Phase 1: Frontend Checkout Wireup & Purchase Modal
1. **Komponen `ProUpgradeModal.svelte`:**
   - Tabs pilihan mata uang: **IDR (Mayar)** vs **USD (Ko-fi)**.
   - Pilihan interval: Bulanan vs Tahunan (Badge Diskon 40% Tahun Pertama).
   - Input prefill email otomatis dari akun CATerm yang sedang login.
   - Tombol "Bayar Sekarang" yang membuka URL checkout browser default via Tauri shell plugin:
     - Mayar: `https://mayar.link/caterm-pro?email={email}&plan={plan}`
     - Ko-fi: `https://ko-fi.com/fathforce/?email={email}&tier={tier}`
2. **Trial Banner & Header Indicator:**
   - Tampilkan badge status di Header bar: `PRO Active`, `PRO Trial (Xd left)`, atau `Free Edition (Upgrade)`.
   - Pop-up peringatan halus pada H-2 dan H-0 sebelum masa trial habis.

### Phase 2: GCC Server Licensing & Key Matching
1. **Ed25519 Key Pair Synchronization:**
   - Validasi key pair Ed25519 GCC Server (`CATERM_PRO_PRIVATE_KEY_V1`) menghasilkan tanda tangan yang lolos verifikasi public key `4aed277ac7b92ee58778d3c5233ebda741c652c753e3dae7127f367dbb58d92f` di `caterm-core/src/pro.rs`.
2. **Mayar & Ko-fi Webhook Processor:**
   - Route `POST /api/caterm/pro/v1/webhook/mayar`:
     - Verifikasi signature HMAC Mayar.
     - Parse customer email & payment status `settled`/`paid`.
     - Upsert customer & buat/perpanjang baris lisensi status `active`, plan `monthly`/`yearly`, `max_devices=7`.
   - Route `POST /api/caterm/pro/v1/webhook/kofi`:
     - Verifikasi token verifikasi Ko-fi.
     - Perpanjang entitlement akun yang bersangkutan.

### Phase 3: Gatekeeper Enforcement Hardening
1. **Cloud Sync Gatekeeper:**
   - Blokir request sync jika payload token client tidak memuat feature flag `cloud_sync`.
2. **Hosted AI Metering & Guard:**
   - Endpoint proxy AI mengecek tabel `ai_usage` (kuota 300 req/bulan per pool license).
   - Balas HTTP 429 `QUOTA_EXCEEDED` jika limit tercapai dan sarankan beralih ke BYO API Key.
3. **Ambient Theme & Founder Styling Lock:**
   - Kustomisasi RGB spectrum dan custom preset ambient glow dikunci secara reaktif jika `entitlement.state !== 'valid'`.

### Phase 4: End-to-End Verification Suite
1. **Test Case 1: Trial 7-Day Lifecycle:**
   - Akun baru registrasi → request trial → verifikasi token Ed25519 valid 7 hari → simulasi clock drift → validasi error handling.
2. **Test Case 2: Multi-Device Limit & Revocation:**
   - Aktivasi 5 device berturut-turut → device ke-6 mendapat error `DEVICE_LIMIT_EXCEEDED` → lakukan revoke device lama via UI → aktivasi device ke-6 berhasil.
3. **Test Case 3: Offline Grace Period (14 Hari):**
   - Disconnect internet → jalankan aplikasi → verifikasi entitlement tetap `valid` hingga batas token offline `exp`.

---

## 4. Task Checklist untuk AI Agent Autopilot Execution

### 🤖 Sub-Agent 1: Frontend Svelte 5 & UI Polish
- [ ] Buat file `frontend/src/lib/components/ProUpgradeModal.svelte` (Dual-gateway Mayar & Ko-fi selector).
- [ ] Integrasikan `ProUpgradeModal` ke `HeaderQuickControls.svelte`, `AmbientSettingsCard.svelte`, dan `ProTeamPanel.svelte`.
- [ ] Tambahkan countdown bar trial interaktif pada `Settings` dan `PageHeader`.
- [ ] Verifikasi seluruh string pesan error dan tombol memiliki translasi lengkap di `frontend/src/lib/i18n/`.

### 🤖 Sub-Agent 2: Backend GCC & Webhook Integration
- [ ] Verifikasi dan uji unit test `crypto_test.go` untuk Ed25519 token signer v1.
- [ ] Implementasikan handler webhook Mayar di `apps/gcc/internal/catermpro/mayar.go`.
- [ ] Implementasikan handler webhook Ko-fi di `apps/gcc/internal/catermpro/kofi.go`.
- [ ] Tambahkan DDL tabel `ai_usage` pada migrasi database GCC PostgreSQL/Turso.
- [ ] Jalankan `go test ./...` pada `gcc/apps/gcc/internal/catermpro` untuk memastikan zero regressions.

### 🤖 Sub-Agent 3: Rust Core & Tauri Bridge
- [ ] Pastikan compile flag pubkey default sinkron dengan production environment.
- [ ] Tambahkan Tauri command `open_checkout_external(url: String)` pada `crates/caterm-app/src/commands.rs`.
- [ ] Jalankan `cargo test -p caterm-core --lib pro` untuk memvalidasi evaluasi offline token.
- [ ] Lakukan build test binary Linux x86_64 (`cargo check --all-targets`).

---

## 5. Deployment & Release Readiness Checklist

- [ ] **DNS & Reverse Proxy:** Subdomain `caterm.fathforce.com/api/pro/v1` terarah stabil ke instance GCC via Cloudflare Tunnel / Traefik.
- [ ] **Production Secrets Setup:**
  - `CATERM_PRO_ED25519_PRIVKEY_V1` terpasang di environment GCC server.
  - `MAYAR_WEBHOOK_SECRET` & `KOFI_VERIFICATION_TOKEN` terkonfigurasi.
- [ ] **Landing Page Parity:**
  - Pricing table di web landing mencantumkan harga promo 40% OFF dan link direct checkout.
- [ ] **Cross-Platform Build CI:**
  - Linux `.AppImage` / `.deb` / `.tar.gz`
  - Windows `.msi` / `.exe`
  - macOS `.dmg` (Universal / Apple Silicon)
- [ ] **Post-Release Monitoring:**
  - Log audit aktivasi token dan monitoring error rate webhook di dashboard GCC.
