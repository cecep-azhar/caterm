# 🚀 Master Autopilot Prompts Index — CATerm v2

Kumpulan master prompt eksekusi otonom untuk sub-agent coding CATerm:

1. **`prompt-2fa.md`** — *Zero-Knowledge 2FA / TOTP Authenticator Vault & SSH Interactive Auto-Inject (REQ-37)*:
   - Implementasi RFC 6238 TOTP Engine di `caterm-core`.
   - Penyimpanan terenkripsi SQLCipher dan auto-inject prompt SSH `keyboard-interactive`.
   - UI Svelte 5: Tab 2FA Authenticator dengan visual timer countdown 30 detik (SVG stroke-dashoffset).

2. **`prompt-cron.md`** — *Agentless Scheduled Tasks, Remote Action Playbooks & SFTP Auto-Backup Engine (REQ-38)*:
   - Implementasi Tokio Async Cron Scheduler di Rust.
   - Pipeline SFTP Auto-Backup Pull (remote dump -> download lokal -> cleanup remote -> retention rotation).
   - System Tray Daemon ultra-ringan & Notifikasi Desktop OS.
   - UI Svelte 5: Tab Scheduled Tasks, Task Modal & Terminal Log Inspector.

3. **Dokumen Arsitektur Terkait:**
   - `cron.md` — Spesifikasi teknis & skema database scheduler.
   - `prd-v2.md` — PRD resmi CATerm v2.0 (mencakup REQ-01 s/d REQ-38).
