# CATerm Component Gap Analysis — v1.2.0

**Date:** 2026-10-11
**Author:** Cecep Saeful Azhar Hidayat, ST

---

## Executive Summary

CATerm frontend 55 komponen UI diimpor dari `$lib/components/`.
CAUI v1.2.0 menyediakan 19 di antaranya.
**36 komponen masih belum ada di CAUI** — harus dibuat atau tetap lokal.

---

## Kategori Gap

### 🔴 P0 — App-Specific (JANGAN pindahkan ke CAUI)
Konten spesifik CATerm, bukan reusable UI pattern:

| Komponen | Lines | Keterangan |
|----------|-------|------------|
| `TerminalPane` | 837 | Terminal SSH core — fitur khusus |
| `SessionViewport` | 213 | Layout session terminal |
| `SessionFileManager` | 368 | SFTP file manager |
| `NetworkAuditModal` | 790 | Security audit modal |
| `HostDetailPanel` | 211 | Host detail display |
| `WorkspaceMenu` | 520 | Workspace navigation |
| `DirectorySync` | 483 | SFTP sync engine |
| `LockScreen` | 574 | Zero-knowledge lock screen |
| `TerminalAutocomplete` | 126 | SSH autocomplete |
| `OsIcon` | 505 | OS detection icons |
| `TotpBadge` | 199 | 2FA badge |

→ **Tetap lokal di CATerm.** Tidak perlu migrasi.

---

### 🟡 P1 — Potensial CAUI (UI Pattern Umum)
Komponen yang bisa reuse di app CA lain (CACash, CAStudio, CAMark, dll):

| Komponen | Lines | Use Case | Rekomendasi |
|----------|-------|----------|-------------|
| `NotificationCenter` | 83 | Toast + notification center | **Tambah CAUI** |
| `FeedbackModal` | 310 | User feedback modal | **Tambah CAUI** |
| `AboutModal` | 100 | About dialog | **Tambah CAUI** |
| `CrashReportModal` | 142 | Crash report | Pertimbangkan CAUI |
| `Pagination` | 71 | Table pagination | **Tambah CAUI** |
| `AvatarPicker` | 115 | Avatar selection | **Tambah CAUI** |
| `ProfileAvatar` | 367 | User profile avatar | **Tambah CAUI** |
| `ProfileMenu` | 139 | User menu dropdown | **Tambah CAUI** |
| `BottomNav` | 90 | Mobile bottom navigation | **Tambah CAUI** |
| `AllMenusSheet` | 272 | Mobile menus overlay | **Tambah CAUI** |
| `SponsorWall` | 88 | Sponsor/contributor list | **Tambah CAUI** |
| `VpsRecommendation` | 32 | VPS recommendation card | Tunda (CATerm-specific) |
| `ProGate` | 71 | Pro feature gate | **Tambah CAUI** |
| `ProLoginForm` | 188 | Subscription login | Tunda |
| `ProTeamPanel` | 257 | Team management | Tunda |
| `AmbientGlow` | 173 | Ambient background effect | **Tambah CAUI** |
| `HeaderQuickControls` | 180 | Header action buttons | **Tambah CAUI** |
| `FloatingAiAssistant` | 300 | AI chat panel | **Tambah CAUI** |
| `AiNavTabs` | 50 | AI settings tabs | Tunda |
| `AiSettingsForm` | 199 | AI configuration form | Tunda |
| `AmbientSettingsCard` | 571 | Settings card | Tunda |
| `BlastShieldModal` | 82 | Security warning modal | Tunda |
| `PasteSentinelModal` | 85 | Paste safety modal | Tunda |
| `RemoteFileEditor` | 231 | Remote file editor | Tunda |
| `PageContainer` | 15 | Wrapper page container | **Tambah CAUI** |

→ **Prioritas: Notification, Feedback, About, Pagination, Avatar, Profile, BottomNav**

---

### ✅ SUDAH ADA di CAUI v1.2.0

| Komponen | Status |
|----------|--------|
| `Button`, `Badge`, `Alert`, `Card`, `Modal`, `Input` | ✅ |
| `Table`, `Select`, `Tabs`, `Switch`, `Checkbox` | ✅ |
| `LanguageSwitcher`, `ThemeSwitcher`, `Icon` | ✅ |
| `Sidebar`, `SidebarItem`, `CommandPalette` | ✅ |
| `PageHeader`, `Logo` | ✅ (root level) |
| `DashboardView`, `LoginScreen`, `SplashScreen`, `AiChatPanel` | ✅ (templates) |

---

## Migrasi ke CAUI — Recommendation

### Phase 1: High Reuse (buat sekarang)
1. `NotificationCenter` → `NotificationCenter.svelte`
2. `FeedbackModal` → `FeedbackModal.svelte`
3. `AboutModal` → `AboutModal.svelte`
4. `Pagination` → `Pagination.svelte`
5. `AvatarPicker` → `AvatarPicker.svelte`
6. `ProfileAvatar` → `ProfileAvatar.svelte`
7. `ProfileMenu` → `ProfileMenu.svelte`
8. `BottomNav` → `MobileBottomNav.svelte` (sudah ada?)

### Phase 2: Medium Reuse
9. `AllMenusSheet` → `MobileBottomNav.svelte` (merge?)
10. `SponsorWall` → `SponsorWall.svelte`
11. `ProGate` → `ProGate.svelte`
12. `AmbientGlow` → `AmbientGlow.svelte`
13. `HeaderQuickControls` → `HeaderQuickControls.svelte`
14. `FloatingAiAssistant` → `FloatingAiAssistant.svelte`
15. `PageContainer` → `PageContainer.svelte`

### Phase 3: App-Specific (tetap lokal)
Semua Terminal*, Session*, Host*, Workspace*, Directory*, Lock* → tetap di CATerm

---

## File Locations

- **CAUI:** `~/Product/caui/src/lib/components/ui/`
- **CATerm:** `~/Product/caterm/frontend/src/lib/components/`
- **CAUI barrel:** `~/Product/caui/src/lib/components/ui/index.ts`

---

*Cecep Saeful Azhar Hidayat, ST*
*cecepazhar.com • hi@cecepazhar.com*