# CATerm PRD Audit — prd-v2.md (REQ-01..39) vs Implementasi

> Audit: 2026-10-11 · Base: `/home/cecepazhar/Product/caterm` · Metode: statis

## Hasil: 35/39 REQ (90%) · Design Compliance: PASS

### Gap / Deviasi

| Req | Gap | Detail | Severity |
|---|---|---|---|
| REQ-13 | Stack deviasi | PRD: `russh`; kode: `ssh2` 0.9.6 (Cargo.toml). Fungsional setara (PTY, exec pool, TOFU di `ssh.rs:288`). | P1 |
| REQ-32 | Placeholder by-design | Sync E2EE: schema `sync.rs` + UI route, jaringan mati. Sesuai PRD §3.2. | Info |
| REQ-34 | Placeholder by-design | Team vault: `teams.rs` + `ProTeamPanel.svelte` badge "Coming Soon". Sesuai §3.2. | Info |
| REQ-37 | Auto-inject TOTP belum terbukti | `totp.rs`/`totp_store.rs` ada; tautan langsung TOTP→prompt SSH interaktif tidak ditemukan via grep. Perlu verifikasi manual. | P0 |
| REQ-29 | Coverage test tipis | `ai/scrubber.rs` mature (bearer/JWT/AWS/ANSI) tapi unit test terlihat 1 (test_scrub_bearer_token). | P2 |

### Bukti Implementasi (key)
- 160 `#[tauri::command]` di `caterm-app`.
- Core modules 30+ file: `vault.rs` (KEK/DEK, Argon2, canary, legacy salt migrate), `ssh.rs` (PTY+exec pool, TOFU), `sftp.rs`, `tunnels.rs`, `monitor.rs`, `audit.rs`, `keys.rs`, `snippets.rs`, `totp*.rs`, `scheduler/{engine,runner,store,task}.rs` (cron 5-field + `every Nh` + `at HH:MM`), `pro.rs`, `ai/{scrubber,dispatcher,context_builder,habit_learner,skills}.rs`.
- Pro crate: `blast_shield`, `paste_sentinel`, `integrity_tripwire`, `jump_proxy`, `gcc_unlock`, `ephemeral_keys`, `session_audit`, `laptop_audit`, `vault_sync`, `license`.
- Frontend: 87 svelte files; routes: command-logs, contribution, design-system, devops, diagnostics, groups, investigations, monitoring, port-forwarding, prompt-studio, security, session, settings/* (ai-routing, ai-personas, ai-habits, ai-skills), sftp, snippets, ssh-keys, tasks, teams, totp.
- CAUI templates dogfood: `lib/components/caui/templates/{LoginScreen,DashboardView,AiChatPanel,SplashScreen}`.

### Rekomendasi
1. P0: Verifikasi & wire REQ-37 auto-inject TOTP ke prompt SSH (atau amend PRD → manual paste).
2. P1: Amend PRD REQ-13: `ssh2` menggantikan `russh`.
3. P2: Tambah unit test scrubber (JWT, AWS, ANSI, multi-line).