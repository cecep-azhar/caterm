# PRD v2.0 — CATerm v2 (Rust / Tauri 2.0 Rewrite)

> **Product Requirements Document — Generasi Kedua**
> **Produk:** CATerm (Cecep Azhar + Terminal) — Zero-Knowledge, Local-First SSH Connection Manager
> **Author & Owner:** Cecep Saeful Azhar Hidayat (Fathforce / PT Fath Synergy Group)
> **Versi dokumen:** 2.0 · **Tanggal:** 18 September 2026 · **Status:** BINDING
> **Lisensi core engine:** MIT · **Stack:** Tauri 2.0 + Rust + Svelte 5 + TailwindCSS + xterm.js v5

---

## 0. Status Dokumen & Hubungan dengan Dokumen Lama

Dokumen ini adalah **spesifikasi tunggal yang mengikat** untuk CATerm v2. Ia menggantikan (*supersede*) dokumen berikut:

| Dokumen lama | Isi | Status setelah dokumen ini |
|---|---|---|
| `prd.md` (Bagian B "Usulan PRD v2.0") | REQ-01 s/d REQ-12, stack Go/Wails | **SUPERSEDED.** Tetap disimpan sebagai arsip keputusan arsitektur; nomor REQ-nya **tidak lagi berlaku**. |
| `prd-v3.md` | REQ-13 s/d REQ-20 (ekstensi cakupan) | **DISERAP.** Seluruh isinya dipetakan ulang ke REQ-baru di §4. Lihat tabel pemetaan §4.0. |
| `task.md` (27 task, Fase F–Z) | Implementasi Go/Wails | **ARSIP.** Digantikan `task-v2.md`. |
| `audit-ulang.md` (T-01 s/d T-21) | Temuan audit Fase 0 | **TETAP MENGIKAT** sebagai daftar anti-regresi. Lihat §12. |
| `blueprint_v2.html` / `CATerm_Blueprint_v2.pdf` | Blueprint arsitektur & feature matrix | **SUMBER UTAMA** dokumen ini. |
| `blueprint-v2.3.md` | Blueprint besar: peta produk, CLI, mobile, store, dokumentasi, monetisasi, BEP, marketing, sosial media, konten video | **DOKUMEN PENDAMPING** (18 September 2026). Mengatur keputusan **bisnis & ekosistem**; tidak mengubah satu REQ pun. Kalau bertentangan soal perilaku aplikasi, **dokumen ini (`prd-v2.md`) yang menang**. Fitur baru F-01..F-08 di sana wajib melewati revisi PRD sebelum dikoding. |
| `proposal.html` / `CATerm_Sync_Cloud_Proposal.pdf` | Strategi monetisasi & GTM | **SUMBER UTAMA** untuk §10. |

> ⚠️ **Penomoran REQ di dokumen ini dimulai ulang dari REQ-01 dan berakhir di REQ-36.** Setiap referensi "REQ-xx" di `task-v2.md` merujuk ke dokumen ini, bukan ke `prd.md` atau `prd-v3.md`.

### 0.1 Keputusan stack — kenapa rewrite, bukan lanjut

CATerm v1 berjalan di **Go + Wails v2**, 27 task selesai, test hijau. Keputusan untuk **rewrite penuh ke Rust + Tauri 2.0** diambil sadar dengan konsekuensi berikut:

| Aspek | Go + Wails v2 (v1) | Rust + Tauri 2.0 (v2) | Alasan menentukan |
|---|---|---|---|
| Baseline RSS | ~55–90 MB (Go runtime + GC + WebKitGTK) | Target **< 35 MB** | Pilar #1 "Very Light". Go runtime + GC arena membuat target 35 MB tidak realistis tanpa trik yang merusak maintainability. |
| Ukuran binary | ~12 MB (terukur di repo v1) | Target **< 8 MB** | `opt-level="z"` + LTO + `panic=abort` + strip. |
| PTY/SSH engine | `golang.org/x/crypto/ssh` | `russh` (async, Tokio) | Kontrol backpressure per-channel lebih eksplisit; menutup T-14 (batching per byte). |
| Keamanan memori | GC — DEK sulit di-*zeroize* secara deterministik | `zeroize` + `secrecy` crate, `Drop` deterministik | Pilar #3. Di Go, `Lock()` tidak menjamin DEK hilang dari heap (T-05 membuktikan masalah ini nyata). |
| Panic safety | `recover()` per-goroutine, mudah terlewat | `#![deny(clippy::unwrap_used)]` ditegakkan compiler | Pilar #2 "Zero panic". T-01 adalah panic nyata di v1. |
| Biaya | — | **Rewrite 8 fase, estimasi 10–14 minggu part-time** | Diterima sebagai investasi satu kali. |

**Kode Go v1 tidak dibuang.** Ia berstatus *reference implementation*: skema SQLite, format ciphertext `c1.<nonce>.<ct>`, aturan AAD, dan test vector kripto **wajib kompatibel biner** agar vault v1 bisa dibuka v2 (REQ-03 §Migrasi).

---

## 1. Genesis — Masalah Nyata yang Dipecahkan

Berdasarkan pemakaian Termius / Termique bertahun-tahun di lingkungan multi-device dan multi-platform, empat cacat berikut adalah alasan produk ini ada. Setiap cacat dipetakan ke requirement yang mematikannya.

### 1.1 Memory Bloat
**Masalah.** Klien SSH berbasis Electron memakan 400 MB – 1 GB+ RAM per instance. Di laptop kerja yang juga menjalankan IDE, Docker, dan browser, ini bukan angka statistik — ini kipas yang menyala dan baterai yang habis dalam 3 jam.
**Dimatikan oleh:** REQ-02 (Resource Budget — hard gate ~~< 35 MB RSS~~ → **≤ 90 MB memori privat**, revisi-2 terukur 2026-09-18 T2-BOOT-06b; baseline aplikasi kosong terukur **70,8 MB** private working set. Revisi pertama hari itu sempat menetapkan 380 MB karena alat ukur menjumlahkan working set dan menghitung ulang halaman Chromium yang dipakai bersama — angka itu sudah dicabut. Klaim pembanding yang benar: **4–6× lebih hemat dari klien berbasis Electron** yang lazim memakai 400 MB – 1 GB+), REQ-14 (flow control PTY), REQ-35 (build profile).

### 1.2 Subscription Lock-in & Data Revocation
**Masalah.** Saat langganan habis, akses ke konfigurasi, grup host, snippet, dan preferensi lokal hilang atau dikunci sepihak. Data yang **diketik sendiri oleh user di mesin sendiri** disandera oleh status pembayaran.
**Dimatikan oleh:** **REQ-09 (Local Data Sovereignty Guarantee)** dan REQ-33 (Entitlement Client). Ini adalah requirement paling ideologis di dokumen ini dan satu-satunya yang berstatus **INVIOLABLE** — tidak boleh dilanggar oleh keputusan bisnis apa pun di masa depan.

### 1.3 Incomplete Backup
**Masalah.** Ekspor lokal tidak pernah mengover seluruh konteks lingkungan: `known_hosts`, key bindings, environment variables per-host, konfigurasi AI, tema, urutan tab.
**Dimatikan oleh:** REQ-31 (Full Environment Backup & Restore — *whole-vault*, bukan *hosts-only*).

### 1.4 Kebocoran Privasi
**Masalah.** Metadata dan kredensial disinkronkan ke cloud dengan enkripsi proprietary yang tidak dapat diaudit pihak ketiga.
**Dimatikan oleh:** REQ-05..REQ-08 (kripto terbuka & terdokumentasi), REQ-32 (Zero-Knowledge Sync — server hanya menyimpan blob), REQ-29 (AI Data Minimization).

---

## 2. Lima Pilar (Definisi Terukur)

Pilar bukan slogan. Setiap pilar punya **angka** dan **gerbang verifikasi** yang bisa gagal di CI.

| # | Pilar | Definisi terukur | Gerbang CI | REQ pengikat |
|---|---|---|---|---|
| 1 | **Very Light** | RSS puncak ~~≤ 35 MB~~ → **≤ 160 MB memori privat** *(provisional, revisi-2 2026-09-18 — lihat REQ-02)* dengan 3 sesi SSH aktif + 1 panel SFTP terbuka selama 10 menit. Idle **≤ 90 MB** *(terukur: 70,8 MB)*. Binary rilis (stripped, per-platform) ≤ 8 MB *(terukur: 2,48 MB)*. Cold start ke jendela unlock ≤ 800 ms *(terukur proxy: ~352–419 ms)*. | `T2-REL-03` gagalkan build kalau terlampaui | REQ-02, REQ-35 |
| 2 | **Stable** | 0 `panic!` di jalur runtime. 0 `unwrap()`/`expect()` di luar `#[cfg(test)]`. Soak test 8 jam tanpa pertumbuhan RSS > 5%. `cargo test` + `cargo miri` (modul kripto) hijau. | `T2-CORE-01`, `T2-REL-02` | REQ-04 |
| 3 | **Privacy First** | Tidak ada byte kredensial plaintext yang pernah ditulis ke disk. Argon2id (t=3, m=64 MiB, p=4) + AES-256-GCM dengan AAD. DEK di-*zeroize* saat lock. 0 telemetri keluar tanpa opt-in eksplisit. | `T2-CORE-06`, `T2-REL-05` (scan disk) | REQ-05..REQ-09, REQ-29 |
| 4 | **Collaboration Next** | Team Vault via ECDH X25519. Ter-*ship* dalam kondisi **DISABLED / UI PLACEHOLDER** di v2.0. Kode kripto-nya tetap ditulis dan diuji. | `T2-NEXT-04` | REQ-34 |
| 5 | **Sync Next** | E2EE Sync managed/self-hosted. Ter-*ship* **DISABLED / UI PLACEHOLDER**. **Berakhirnya langganan tidak pernah mengurangi fungsi lokal.** | `T2-NEXT-02`, `T2-NEXT-05` | REQ-32, REQ-33, REQ-09 |

---

## 3. Cakupan

### 3.1 Dalam cakupan v2.0 GA — AKTIF LOKAL

1. Hosts & Groups Management (hierarki, tag, inheritance)
2. Interactive PTY Terminal (tabs, split pane H/V, truecolor, bracketed paste)
3. TOFU Anti-MITM Host Key Verification
4. Files Manager (SFTP dual-pane, drag-drop, resume, permission editor)
5. Snippets Engine (variabel dinamis, auto-enter / inspect)
6. Port Forwarding (Local / Remote / Dynamic SOCKS5)
7. Server Real-time Monitoring (CPU, RAM, Disk, Traffic, Load)
8. Command Audit Logs (truncate 5 KB, terenkripsi)
9. SSH Keys Manager (generate RSA-4096/Ed25519, import/export, deploy public key)
10. AI Terminal Copilot (opt-in, provider kustom multi-profil + validasi, API key terenkripsi)
11. Full Environment Backup & Restore
12. Settings, Theming, Keybindings

### 3.2 Dalam cakupan v2.0 GA — DISABLED / NEXT FEATURE (UI Placeholder)

13. E2EE Multi-Device Sync — UI lengkap, badge *"Coming Soon · Managed Sync"*, tombol non-aktif, **tanpa** koneksi jaringan apa pun.
14. Team Vault Collaboration — idem, badge *"Coming Soon · Team"*.

> Kedua fitur ini **wajib ter-*ship*** dalam bentuk placeholder. Alasannya bukan pemasaran: keberadaan skema tabel, tipe data, dan layar kosongnya sejak awal mencegah rewrite kedua saat backend cloud siap. Kode kriptografinya (§9) **ditulis dan diuji penuh** di v2.0 meskipun jalur jaringannya mati.

### 3.3 Di luar cakupan (eksplisit, v2.0)

- Mobile (iOS/Android) — Tauri 2.0 mendukungnya, tapi tidak untuk rilis ini.
- Telnet, serial console, RDP, VNC, Kubernetes exec.
- Mosh.
- Agent forwarding (`ssh -A`) — ditunda ke v2.1 karena permukaan risikonya besar dan butuh desain terpisah.
- Plugin/extension marketplace.
- Backend Sync Cloud itu sendiri (repo terpisah: `caterm-sync-relay`).
- Multi-user lokal / profil OS terpisah dalam satu instalasi.

---

## 4. Requirement Register (REQ-01 s/d REQ-36)

**Format.** Setiap REQ berisi: Deskripsi · Aturan Bisnis · Kriteria Penerimaan (GIVEN/WHEN/THEN) · Prioritas. Prioritas: **P0** = blocker rilis · **P1** = wajib GA · **P2** = boleh menyusul di patch minor.

### 4.0 Pemetaan dari dokumen lama

| REQ lama | Sumber | REQ baru di dokumen ini |
|---|---|---|
| REQ-01 (Master Password) | `prd.md` | REQ-05, REQ-08 |
| REQ-02 (CRUD Host/Group) | `prd.md` | REQ-10, REQ-11 |
| REQ-03 (SSH/PTY/Resize) | `prd.md` | REQ-13, REQ-14, REQ-15 |
| REQ-04 (Tabs & Split) | `prd.md` | REQ-16 |
| REQ-05 (Snippet) | `prd.md` | REQ-22 |
| REQ-06/07 (Export/Import) | `prd.md` | REQ-31, REQ-32 |
| REQ-08 (TOFU) | `prd.md` | REQ-18 |
| REQ-09 (Key passphrase) | `prd.md` | REQ-26 |
| REQ-10 (Paste safety) | `prd.md` | REQ-17 |
| REQ-11 (Auto-lock) | `prd.md` | REQ-07 |
| REQ-12 (Settings) | `prd.md` | REQ-30 |
| REQ-13 (TOFU) | `prd-v3.md` | REQ-18 |
| REQ-14 (SFTP) | `prd-v3.md` | REQ-19, REQ-20, REQ-21 |
| REQ-15 (Port Forwarding) | `prd-v3.md` | REQ-23 |
| REQ-16 (Monitoring) | `prd-v3.md` | REQ-24 |
| REQ-17 (SSH Key Manager) | `prd-v3.md` | REQ-26, REQ-27 |
| REQ-18 (Audit Log) | `prd-v3.md` | REQ-25 |
| REQ-19 (Teams — "DICOPOT BERSIH") | `prd-v3.md` | **DIBALIK** → REQ-34 (lihat catatan di bawah) |
| REQ-20 (AI Copilot) | `prd-v3.md` | REQ-28, REQ-29 |

> **Catatan pembalikan keputusan REQ-19.** `prd-v3.md` mencopot Teams dengan alasan "PKI berada di luar arsitektur v2.0 dan akan menghancurkan zero-knowledge". Alasan itu **valid untuk arsitektur Git-sync v1**, tapi tidak lagi berlaku: blueprint v2 mengganti transport sync dari Git-folder menjadi relay blob, dan ECDH X25519 justru **memperkuat** zero-knowledge karena server tidak pernah memegang kunci simetris bersama. Teams dihidupkan kembali sebagai REQ-34 dengan status **DISABLED di v2.0** — kriptonya ditulis & diuji, jalur jaringannya mati.

---

## BAGIAN A — FONDASI PLATFORM

### REQ-01 — Arsitektur Tauri 2.0 + Rust Core Terisolasi
**Prioritas:** P0

**Deskripsi.** Seluruh logika bisnis hidup di crate Rust yang **tidak bergantung pada Tauri**, sehingga dapat dibangun, dijalankan, dan diuji tanpa GUI.

**Aturan bisnis.**
- Workspace Cargo dua crate: `caterm-core` (pure Rust, library) dan `caterm-app` (`src-tauri`, thin binding layer).
- `caterm-core` **dilarang** meng-*import* `tauri`, `tauri-plugin-*`, `webkit`, atau crate GUI apa pun. Ditegakkan oleh test arsitektur yang gagal di CI.
- Binding Tauri (`#[tauri::command]`) hanya boleh berisi: deserialisasi argumen → panggil satu fungsi `caterm-core` → serialisasi hasil. **Dilarang ada `if`/`match` logika bisnis di dalam binding.**
- CLI headless `catermctl` (crate ketiga, `caterm-cli`) memanggil `caterm-core` yang **persis sama** dengan yang dipanggil GUI. Ini alat verifikasi utama di seluruh `task-v2.md`.

**Alasan.** Aplikasi desktop tidak bisa di-`curl`. Tanpa aturan ini, satu-satunya cara membuktikan sebuah fitur hidup adalah membuka GUI dan melihat — dan "melihat" bukan bukti. Ini pelajaran langsung dari `audit-ulang.md`: T-03 (split pane fiktif) dan T-04 (SFTP UI tanpa backend) lolos QA justru karena verifikasinya visual.

**Kriteria penerimaan.**
- **GIVEN** seseorang menambahkan `use tauri::...` ke `caterm-core`, **WHEN** CI berjalan, **THEN** job `arch-guard` gagal dengan pesan yang menyebut file pelanggarnya.
- **GIVEN** GUI belum pernah dijalankan, **WHEN** `catermctl` dipanggil, **THEN** seluruh operasi vault, host, snippet, key, dan sync-export dapat dieksekusi penuh dari terminal.
- **GIVEN** `cargo build -p caterm-core` dijalankan di mesin tanpa WebKitGTK/libsoup terpasang, **WHEN** build selesai, **THEN** sukses.

---

### REQ-02 — Resource Budget & Local Performance Telemetry
**Prioritas:** P0

**Deskripsi.** Efisiensi sumber daya adalah **requirement fungsional**, bukan aspirasi. Ia diukur otomatis dan bisa menggagalkan rilis.

**Aturan bisnis.**

> **[REVISI-2 2026-09-18 — T2-BOOT-06b. MENCABUT revisi pertama hari ini.]**
> Revisi pertama menetapkan ambang **380 MB** berdasarkan RSS terukur ≈ 330,4 MiB.
> **Angka itu salah, dan ambang 380 MB dicabut.** Penyebabnya bukan penalaran, melainkan
> alat ukurnya: harness menjumlahkan `sysinfo::Process::memory()` — di Windows itu
> **working set**, yang menyertakan halaman memori *yang dipakai bersama*. WebView2 adalah
> Chromium; keenam prosesnya memetakan `msedge.dll` (~200 MB kode) yang sama. Menjumlahkan
> working set menghitung kode bersama itu **satu kali per proses**. Jebakan yang sama ada di
> Linux dengan WebKitGTK dan `/proc/<pid>/statm`.
>
> **Pengukuran ulang dengan metode yang benar** (aplikasi kosong, Tauri 2.11 + WebView2,
> Windows 10, 60 detik setelah start, 7 proses — kondisi persis sama):
>
> | Metrik | Nilai | Keterangan |
> |---|---|---|
> | Jumlah working set (metode lama) | **315,7 MB** | ❌ salah — halaman bersama terhitung ulang |
> | **Private working set** | **70,8 MB** | ✅ angka Task Manager; ini yang dibandingkan orang |
> | Private commit (`PrivateUsage`) | **141–156 MB** | gerbang CI; konservatif, mencakup halaman ter-*page-out* |
> | Proses Rust (`caterm-app.exe`) saja | **2,9 MB** private | kode CATerm sendiri nyaris tak berbiaya |
>
> **Kesimpulan Q-01/D-01 yang diperbarui.** Target lama ≤ 18/35 MB memang tidak realistis
> dengan WebView2 — jawaban Q-01 tetap **TIDAK**. Tetapi biaya sebenarnya adalah **~71 MB**,
> bukan 330 MB. Arsitektur **TIDAK diubah** (K-1 final). Ambang di bawah memakai
> **memori privat** sebagai metrik normatif; working set **dilarang** dipakai sebagai angka
> budget. Baris `RSS aktif` tetap provisional — wajib diukur ulang dengan 3 sesi SSH + SFTP
> nyata di Fase 2/8. **Butuh konfirmasi pemilik — tidak memblokir kelanjutan kerja.**
> Bukti: `evidence/v2/T2-BOOT-06.log` (pengukuran pertama, disimpan apa adanya sebagai
> catatan kesalahan) dan `evidence/v2/T2-BOOT-06b.log` (koreksi + pembuktian).

**Metrik memori bersifat normatif: "memori privat" = halaman bersama TIDAK dihitung.**
Menjumlahkan working set lintas proses WebView dilarang eksplisit — ia menggelembungkan
angka ~4,5× dan itulah persis kesalahan yang dicabut di atas.

| Metrik | Batas keras | Kondisi pengukuran |
|---|---|---|
| RSS idle (vault terkunci) | ~~≤ 18 MB~~ → **≤ 90 MB** private working set *(revisi-2 2026-09-18; terukur 70,8 MB)* | 60 detik setelah start |
| RSS aktif | ~~≤ 35 MB~~ → **≤ 160 MB** private working set *(provisional — wajib diukur ulang di Fase 2/8 dengan kondisi asli)* | 3 sesi SSH + 1 panel SFTP terbuka, 10 menit |
| Gerbang CI (private commit) | **≤ 180 MB** idle *(proksi konservatif yang dapat diukur otomatis lintas-OS)* | `scripts/budget-check.sh` |
| RSS drift (soak) | ≤ +5% | 8 jam, 1 sesi dengan output kontinu |
| Binary rilis (stripped) | ≤ 8 MB *(terukur: 2,48 MB — lolos jauh di bawah ambang)* | per-platform, tanpa installer wrapper |
| Cold start → jendela unlock | ≤ 800 ms *(terukur proxy backend-ready: ~380 ms — lolos)* | mesin referensi, disk warm |
| Latensi keystroke → render lokal | ≤ 16 ms p95 | echo lokal xterm.js |
| Query `hosts` 1.000 baris | ≤ 20 ms | SQLite, index terpasang |

- Pengukuran RSS memakai *resident set size* proses utama **plus seluruh proses anak WebView**. Mengukur hanya proses Rust adalah kecurangan dan dilarang eksplisit.
- Panel "Resource Monitor" internal (Settings → About) menampilkan RSS, jumlah task Tokio aktif, jumlah koneksi SSH terbuka, dan ukuran DB. **Data ini tidak pernah keluar dari mesin.**

**Kriteria penerimaan.**
- **GIVEN** build rilis, **WHEN** harness benchmark dijalankan di CI, **THEN** laporan JSON berisi seluruh metrik di atas dan job gagal jika ada yang melampaui batas.
- **GIVEN** soak test 8 jam dengan `yes` berjalan di remote, **WHEN** selesai, **THEN** RSS akhir ≤ 105% RSS pada menit ke-5.

---

### REQ-03 — Storage Engine: SQLite WAL, Migrasi Berversi, ULID
**Prioritas:** P0

**Deskripsi.** Satu file SQLite adalah sumber kebenaran tunggal untuk seluruh state aplikasi.

**Aturan bisnis.**
- Driver `rusqlite` dengan fitur `bundled` (SQLite ter-*link* statis — hindari ketergantungan versi OS).
- PRAGMA wajib saat open: `journal_mode=WAL`, `foreign_keys=ON`, `busy_timeout=5000`, `synchronous=NORMAL`, `secure_delete=ON`.
- **Seluruh primary key bertipe `TEXT` berisi ULID.** Dilarang `INTEGER PRIMARY KEY AUTOINCREMENT` pada tabel mana pun yang bisa di-sync. ULID dipilih karena monotonik-waktu (ramah B-tree) dan 26 karakter Crockford base32 (lebih pendek & bebas ambiguitas dibanding UUID hex).
- Migrasi **bernomor dan bertahap**, tercatat di `schema_migrations(version, applied_at, checksum)`. Dilarang migrasi "idempoten `IF NOT EXISTS` dengan versi statis" — ini persis temuan **T-12**.
- Setiap migrasi menyimpan `checksum` SHA-256 dari SQL-nya. Jika checksum migrasi yang sudah diterapkan berubah di build berikutnya, aplikasi **menolak start** dengan pesan eksplisit.
- Seluruh timestamp: `TEXT` ISO-8601 UTC presisi milidetik (`2026-09-18T07:12:33.482Z`). Dilarang epoch integer, dilarang waktu lokal.
- **Backward compatibility:** migrasi `v1_import` mampu membaca file DB CATerm v1 (Go) dan mengangkatnya ke skema v2 **tanpa meminta re-enkripsi** — format ciphertext identik (§8.2).

**Kriteria penerimaan.**
- **GIVEN** DB kosong, **WHEN** aplikasi start pertama kali, **THEN** seluruh tabel §5 terbentuk dan `schema_migrations` berisi seluruh versi.
- **GIVEN** migrasi dijalankan 2×, **WHEN** selesai, **THEN** tidak error dan `schema_migrations` tidak duplikat.
- **GIVEN** file DB dari CATerm v1, **WHEN** dibuka v2 dengan Master Password yang sama, **THEN** seluruh host, snippet, dan host key terbaca utuh dan kredensial berhasil didekripsi.
- **GIVEN** seseorang mengubah SQL migrasi v3 yang sudah dirilis, **WHEN** aplikasi start di DB yang sudah menerapkannya, **THEN** aplikasi menolak start dengan error `MIGRATION_CHECKSUM_MISMATCH`.

---

### REQ-04 — Zero-Panic Policy & Taksonomi Error
**Prioritas:** P0

**Deskripsi.** Aplikasi tidak boleh *crash*. Kegagalan harus menjadi pesan, bukan proses yang mati.

**Aturan bisnis.**
- Lint level crate: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::integer_arithmetic_overflow)]` di `caterm-core` dan `caterm-app`, kecuali di dalam `#[cfg(test)]`.
- Seluruh fungsi publik `caterm-core` mengembalikan `Result<T, CatermError>`. `CatermError` adalah `enum` `thiserror` dengan varian bertingkat per-domain (`Vault`, `Db`, `Ssh`, `Sftp`, `Tunnel`, `Ai`, `Sync`, `Io`).
- Setiap varian punya **kode stabil** (`CAT-VAULT-003`) yang dirender di UI dan di log. Kode ini kontrak publik: dilarang berubah arti antar versi.
- **Aturan redaksi log (mengikat, menutup T-02):** tidak ada password, passphrase, private key, API key, isi ciphertext, atau DEK yang boleh muncul di log level mana pun — termasuk `debug` dan `trace`. Tipe rahasia dibungkus `secrecy::Secret<T>` yang implementasi `Debug`-nya mencetak `[REDACTED]`.
- `panic = "abort"` di profil rilis. Panic yang lolos = *crash* yang terlihat, bukan kondisi tersembunyi. Panic hook menulis laporan lokal (tanpa isi memori) ke `crash/`.

**Kriteria penerimaan.**
- **GIVEN** `cargo clippy --workspace --all-targets -- -D warnings`, **WHEN** dijalankan, **THEN** 0 warning.
- **GIVEN** test `test_no_secret_in_debug_output`, **WHEN** `format!("{:?}", secret_struct)` dipanggil pada seluruh struct yang memuat rahasia, **THEN** tidak ada satu pun nilai rahasia muncul. Test ini **berisi assertion nyata** — badan test kosong adalah pelanggaran (T-02).
- **GIVEN** fuzz test 10.000 iterasi pada parser ciphertext, parser fingerprint, dan parser output monitoring, **WHEN** selesai, **THEN** 0 panic.

---

## BAGIAN B — KRIPTOGRAFI & VAULT

### REQ-05 — Master Password & Hierarki Kunci KEK/DEK
**Prioritas:** P0

**Deskripsi.** Satu Master Password membuka seluruh vault. Kunci enkripsi data tidak pernah diturunkan langsung dari password.

**Aturan bisnis.**
- Hierarki dua lapis:
  ```
  MasterPassword + salt(16B) ──Argon2id(t=3, m=64MiB, p=4, len=32)──▶ KEK   [hanya di memori]
  DEK(32B, CSPRNG, sekali seumur vault) ──AES-256-GCM(KEK)──▶ wrapped_dek  [disimpan]
  kredensial ──AES-256-GCM(DEK, AAD)──▶ ciphertext                          [disimpan & di-sync]
  ```
- **Tidak ada kolom `master_hash`.** Verifikasi password = percobaan *unwrap* `wrapped_dek`; tag GCM gagal berarti password salah. Menyimpan dua artefak turunan dari satu rahasia memperbesar permukaan brute-force offline tanpa manfaat.
- Parameter Argon2id disimpan di `vault_meta` (`kdf_algo`, `kdf_t`, `kdf_m_kib`, `kdf_p`, `kdf_salt`) sehingga dapat dinaikkan di masa depan tanpa memecah vault lama.
- Kebijakan password minimum: 12 karakter. Indikator kekuatan memakai `zxcvbn`; skor < 3 menampilkan peringatan tapi **tidak memblokir** — ini mesin pengguna sendiri, bukan layanan kami.
- Gagal unlock: *backoff* eksponensial mulai 1 detik, maksimum 30 detik. **Tidak ada penghapusan data setelah N kegagalan** — kebijakan "wipe after 10 tries" adalah vektor sabotase, bukan keamanan.
- Pesan error unlock **tidak boleh** membedakan "vault tidak ada" dari "password salah" di permukaan UI yang sama.

**Kriteria penerimaan.**
- **GIVEN** vault baru, **WHEN** Master Password di-set, **THEN** `vault_meta` berisi `kdf_salt` acak 16 byte, `wrapped_dek`, parameter KDF — dan **tidak** berisi hash password dalam bentuk apa pun.
- **GIVEN** password salah 3×, **WHEN** percobaan ke-4, **THEN** ada jeda ≥ 4 detik dan tidak ada data yang tersentuh.
- **GIVEN** `wrapped_dek` dirusak 1 byte, **WHEN** unlock dengan password benar, **THEN** error `CAT-VAULT-002` (integritas), bukan panic dan bukan DEK sampah.

---

### REQ-06 — Field-Level Encryption AES-256-GCM dengan AAD
**Prioritas:** P0

**Deskripsi.** Kredensial dienkripsi per-field, bukan per-database, sehingga sync blob dan backup tidak pernah membawa plaintext.

**Aturan bisnis.**
- Format field terenkripsi (kompatibel biner dengan CATerm v1):
  ```
  c1.<b64url_nopad(nonce 12B)>.<b64url_nopad(ciphertext || tag 16B)>
  ```
  `c1` adalah versi format. Parser wajib menolak prefix tak dikenal, bukan menebak.
- **AAD wajib = `"<record_id>|<table>|<field>"`** (UTF-8). Ini mengikat ciphertext ke lokasinya: ciphertext password host A tidak dapat dipindahkan ke host B oleh penyerang yang punya akses tulis ke DB atau ke blob sync.
- Nonce 12 byte dari CSPRNG, **unik per operasi enkripsi**. Dilarang nonce deterministik, dilarang counter yang di-persist.
- **Ciphertext adalah bentuk kanonik.** Re-enkripsi hanya terjadi saat plaintext benar-benar berubah. Membuka form lalu menekan Batal tidak boleh mengubah satu byte pun di DB. (Tanpa aturan ini, setiap sync menghasilkan diff palsu pada seluruh baris.)
- Field yang **wajib** terenkripsi: `hosts.password_enc`, `hosts.private_key_enc`, `hosts.key_passphrase_enc`, `ssh_keys.private_key_enc`, `ssh_keys.passphrase_enc`, `settings.ai_api_key_enc`, `audit_log.output_enc`, `audit_log.command_enc`, `team_identity.private_key_enc`, `ai_providers.api_key_enc`, `ai_providers.headers_enc`.
- Field yang **sengaja tidak** terenkripsi (metadata operasional, dibutuhkan untuk query/index): `hosts.label`, `hosts.hostname`, `hosts.port`, `hosts.username`, `groups.label`, tag, timestamp. **Konsekuensi ini wajib dinyatakan ke user** di Settings → Security: *"Nama host dan alamat server disimpan tanpa enkripsi agar pencarian instan. Password, private key, dan API key selalu terenkripsi."*

**Kriteria penerimaan.**
- **GIVEN** roundtrip encrypt→decrypt, **WHEN** dijalankan, **THEN** plaintext identik byte-per-byte.
- **GIVEN** ciphertext milik `(host_A, hosts, password)`, **WHEN** didekripsi dengan AAD `(host_B, hosts, password)`, **THEN** gagal dengan error integritas. *(Ini inti perlindungan terhadap penyerang dengan akses tulis.)*
- **GIVEN** 1 byte ciphertext dibalik, **WHEN** dekripsi, **THEN** gagal tag GCM — bukan mengembalikan sampah.
- **GIVEN** host disimpan tanpa mengubah password, **WHEN** disimpan 100×, **THEN** `password_enc` tidak berubah satu byte pun.
- **GIVEN** 1.000 enkripsi berturut-turut, **WHEN** nonce dikumpulkan, **THEN** 0 duplikat.

---

### REQ-07 — Auto-Lock, Zeroization & Higiene Memori
**Prioritas:** P0

**Deskripsi.** DEK hidup di memori hanya selama vault terbuka, dan benar-benar hilang saat terkunci.

**Aturan bisnis.**
- Auto-lock idle: default **10 menit**, dapat diatur 1–120 menit, atau dimatikan dengan konfirmasi risiko eksplisit.
- Pemicu lock tambahan: lock manual (`Ctrl+Shift+L`), sistem masuk sleep/suspend, layar terkunci OS (bila API tersedia), jendela ditutup.
- Idle dihitung dari input user terakhir di **jendela aplikasi**. Output terminal yang mengalir **bukan** aktivitas user — server yang cerewet tidak boleh menahan vault tetap terbuka.
- Saat lock: DEK dan KEK di-*zeroize* (`zeroize::Zeroizing`), seluruh handle koneksi SSH ditutup, panel SFTP ditutup, transfer berjalan dibatalkan dengan status `cancelled_by_lock`, tunnel ditutup.
- Sesi terminal yang terputus karena lock **tidak** dihidupkan ulang otomatis saat unlock. Sesi zombie lebih berbahaya daripada merepotkan.
- **Auto-lock wajib terpasang pada startup.** Ini temuan **T-05**: di v1, `StartIdleTimer` ada tapi tidak pernah dipanggil. Test integrasi wajib membuktikan timer benar-benar terpasang, bukan sekadar ada.
- Memory hygiene: `mlock`/`VirtualLock` pada halaman DEK bila OS mengizinkan (best-effort, kegagalan tidak fatal tapi dicatat). Core dump dinonaktifkan di rilis (`RLIMIT_CORE=0` di Unix).

**Kriteria penerimaan.**
- **GIVEN** vault terbuka dan idle timeout 1 menit, **WHEN** 61 detik tanpa input, **THEN** vault terkunci, seluruh sesi SSH tertutup, dan operasi berikutnya menuntut password.
- **GIVEN** `Lock()` dipanggil, **WHEN** memory dump proses diperiksa dalam test, **THEN** byte DEK tidak ditemukan.
- **GIVEN** aplikasi baru start, **WHEN** test integrasi memeriksa registry timer, **THEN** idle timer terpasang dan callback auto-lock ter-*register* — bukan `None`.

---

### REQ-08 — Ganti Master Password & Recovery Kit
**Prioritas:** P1

**Deskripsi.** Password bisa diganti tanpa menyentuh data; kehilangan password harus punya jalan keluar yang sadar.

**Aturan bisnis.**
- `ChangePassword(old, new)` = *rewrap* DEK saja. **Ciphertext field tidak boleh berubah satu byte pun.** Operasi ini O(1), bukan O(n).
- Operasi rewrap atomik: tulis `wrapped_dek` baru dalam satu transaksi bersama `kdf_salt` baru.
- **Recovery Kit** (opt-in, ditawarkan sekali saat setup dan selalu tersedia di Settings): DEK di-*wrap* dengan kunci turunan dari *recovery code* 24 kata (BIP-39 wordlist) dan disimpan di kolom `recovery_wrapped_dek`. User diminta mencetak/menyalin kode itu. Tanpa Recovery Kit, **lupa password = data hilang permanen** — ini dinyatakan terang-terangan di UI, bukan di footer.
- Recovery Kit dapat dicabut (`recovery_wrapped_dek = NULL`) kapan saja.
- **Tidak ada backdoor, tidak ada "reset password" berbasis email, tidak ada escrow.** Ini konsekuensi langsung dari zero-knowledge dan harus dinyatakan ke user.
- `ResetVault()` (hapus seluruh data) **wajib memverifikasi Master Password saat ini** sebelum eksekusi, plus konfirmasi ketik ulang kata `HAPUS`. Ini temuan **T-10**.

**Kriteria penerimaan.**
- **GIVEN** 50 host tersimpan, **WHEN** `ChangePassword` sukses, **THEN** seluruh 50 kredensial tetap terbaca, `wrapped_dek` berubah, dan hash SHA-256 dari seluruh kolom `*_enc` **identik** dengan sebelum operasi.
- **GIVEN** Recovery Kit aktif, **WHEN** user memasukkan 24 kata yang benar, **THEN** vault terbuka dan user dipaksa menetapkan Master Password baru.
- **GIVEN** `ResetVault` dipanggil tanpa password valid, **WHEN** dieksekusi, **THEN** ditolak dan **tidak ada** tabel yang tersentuh.

---

### REQ-09 — Local Data Sovereignty Guarantee ⚖️ **INVIOLABLE**
**Prioritas:** P0

**Deskripsi.** Data lokal pengguna adalah milik pengguna, tanpa syarat, selamanya. Ini requirement yang mendefinisikan produk.

**Aturan bisnis (mengikat permanen, tidak dapat di-*override* oleh keputusan produk/bisnis di masa depan).**
1. Aplikasi **tidak pernah** memerlukan koneksi internet untuk membuka vault, terhubung ke server, atau menjalankan fitur lokal mana pun yang tercantum di §3.1.
2. Berakhirnya langganan, kegagalan pembayaran, pencabutan akun, atau **matinya perusahaan** tidak boleh:
   - mengunci, menghapus, mengenkripsi-ulang, atau membatasi akses ke data lokal;
   - menonaktifkan fitur mana pun di §3.1;
   - memunculkan nag-screen yang memblokir pemakaian.
3. Efek berakhirnya langganan **hanya satu**: sinkronisasi ke Managed Cloud berhenti. Itu saja. Data lokal terakhir tetap utuh dan dapat diedit.
4. **Dilarang** ada *kill switch*, *remote wipe*, *license check* daring wajib, *time bomb*, atau kode apa pun yang mengubah perilaku lokal berdasarkan respons server.
5. Ekspor penuh (REQ-31) **selalu** tersedia, termasuk saat langganan mati dan saat offline.
6. Core engine berlisensi **MIT**. Jika layanan cloud berhenti, komunitas dapat men-*self-host* relay (§9.4) atau tetap memakai aplikasi sepenuhnya offline.

**Kriteria penerimaan.**
- **GIVEN** mesin tanpa jaringan sama sekali (interface dimatikan), **WHEN** aplikasi dijalankan, **THEN** seluruh fitur §3.1 berfungsi penuh tanpa pesan error jaringan dan tanpa penundaan startup.
- **GIVEN** entitlement di-*mock* menjadi `expired`, **WHEN** seluruh suite fungsional dijalankan, **THEN** 100% test §3.1 tetap lulus.
- **GIVEN** audit kode otomatis mencari pola *kill switch* (`remote_wipe`, `license_valid`, `disable_if_`, `subscription_required`), **WHEN** CI berjalan, **THEN** 0 hasil di jalur yang mempengaruhi fitur lokal.
- **GIVEN** build rilis, **WHEN** trafik jaringan dipantau selama 30 menit pemakaian normal tanpa fitur AI/Sync, **THEN** **0 byte** keluar dari aplikasi.

---

## BAGIAN C — DATA DOMAIN

### REQ-10 — Groups: Hierarki & Konfigurasi Terwaris
**Prioritas:** P1

**Deskripsi.** Group bukan sekadar folder; ia membawa konfigurasi yang diwarisi host di dalamnya.

**Aturan bisnis.**
- Hierarki *n*-level lewat `parent_id` (self-referencing). Kedalaman maksimum 8 — melindungi UI dan query rekursif.
- **Deteksi siklus wajib.** Memindahkan group A ke dalam keturunannya sendiri ditolak dengan `CAT-DB-011`.
- Atribut terwaris ke host anak: `username`, `port`, `ssh_key_id`, `proxy_jump_host_id`, `env_vars`, `charset`, `keepalive_interval`. Host boleh *override* per-field; nilai `NULL` di host berarti "warisi".
- Resolusi warisan bersifat *nearest-ancestor-wins*, dihitung di `caterm-core`, bukan di UI.
- Menghapus group: soft-delete (`deleted_at`). Host anak **tidak ikut terhapus** — dipindah ke group induk, atau ke root jika tidak ada induk. Penghapusan diam-diam atas host adalah kehilangan data.
- Warna & ikon per-group untuk identifikasi visual cepat (mis. merah = produksi).

**Kriteria penerimaan.**
- **GIVEN** group `Prod` dengan `username=deploy`, **WHEN** host di dalamnya tidak menetapkan username, **THEN** koneksi memakai `deploy` dan UI menampilkan badge *"diwarisi dari Prod"*.
- **GIVEN** group A induk dari B, **WHEN** user mencoba memindahkan A ke dalam B, **THEN** ditolak dengan pesan jelas.
- **GIVEN** group dengan 5 host dihapus, **WHEN** selesai, **THEN** 5 host tetap ada dan berpindah ke induk.

---

### REQ-11 — Hosts: CRUD, Tag, Konfigurasi Koneksi Lengkap
**Prioritas:** P0

**Deskripsi.** Entitas inti aplikasi.

**Aturan bisnis.**
- Field: `label`, `hostname`, `port` (default 22), `username`, `auth_method` (`password` | `key` | `agent` | `interactive`), kredensial terenkripsi, `group_id`, `tags[]`, `color`, `notes`, `env_vars` (JSON), `startup_snippet_id`, `proxy_jump_host_id`, `keepalive_interval`, `compression`, `charset`, `sort_order`.
- **Jump host / bastion (`ProxyJump`)** didukung berantai maksimum 3 hop. Loop terdeteksi dan ditolak.
- Tag many-to-many lewat tabel `host_tags`. Tag bersifat global dan dapat diberi warna.
- Duplikat host (`Clone`) menyalin seluruh konfigurasi **termasuk kredensial terenkripsi** — yang berarti **re-enkripsi dengan AAD baru** (record_id berbeda), bukan menyalin ciphertext mentah. Ini konsekuensi langsung REQ-06 dan mudah terlewat.
- Soft-delete dengan `deleted_at`; Trash dapat dipulihkan selama 30 hari lalu di-*purge*.
- Impor dari `~/.ssh/config`: parse `Host`, `HostName`, `Port`, `User`, `IdentityFile`, `ProxyJump`. Hasilnya masuk ke group `Imported` dan **selalu** ditampilkan sebagai pratinjau untuk dikonfirmasi — tidak pernah langsung ditulis.

**Kriteria penerimaan.**
- **GIVEN** host dengan password, **WHEN** disimpan, **THEN** `password_enc` berformat `c1.*` dan **tidak ada** plaintext password yang bisa ditemukan di file DB (dibuktikan dengan pemindaian byte mentah file `.db`, `.db-wal`, dan `.db-shm`).
- **GIVEN** host di-*clone*, **WHEN** clone dibuka, **THEN** password terbaca benar DAN `password_enc` clone **berbeda** dari aslinya (AAD berbeda).
- **GIVEN** `~/.ssh/config` dengan 20 entri, **WHEN** diimpor, **THEN** pratinjau menampilkan 20 host dan tidak ada yang tertulis sebelum konfirmasi.

---

### REQ-12 — Pencarian Instan & Command Palette
**Prioritas:** P1

**Deskripsi.** Menemukan dan menghubungi server dalam < 2 detik tanpa menyentuh mouse.

**Aturan bisnis.**
- `Ctrl+K` / `Cmd+K` membuka Command Palette: pencarian *fuzzy* lintas host, group, snippet, SSH key, dan aksi aplikasi.
- Pencarian host mencakup `label`, `hostname`, `username`, `tags`, `notes`. **Tidak pernah** mencari di dalam field terenkripsi (mustahil dan tidak diinginkan).
- Hasil terurut: kecocokan awalan > kecocokan fuzzy > frekuensi pakai (`connect_count`) > terakhir dipakai (`last_connected_at`).
- Target performa: ≤ 30 ms untuk 1.000 host (REQ-02).
- `Enter` pada host = langsung buka sesi di tab baru. `Ctrl+Enter` = buka di split pane.

**Kriteria penerimaan.**
- **GIVEN** 1.000 host di DB, **WHEN** user mengetik 3 karakter, **THEN** hasil tampil ≤ 30 ms (diukur, bukan dirasakan).
- **GIVEN** palette terbuka, **WHEN** user mengetik nama host lalu Enter, **THEN** sesi SSH terbuka tanpa klik mouse.

---

## BAGIAN D — TERMINAL & SSH

### REQ-13 — Koneksi SSH via `russh` & Metode Autentikasi
**Prioritas:** P0

**Deskripsi.** Mesin koneksi berbasis `russh` async di atas Tokio.

**Aturan bisnis.**
- Metode auth didukung: `password`, `publickey` (RSA, ECDSA, Ed25519, dengan/tanpa passphrase), `keyboard-interactive` (untuk 2FA/OTP), dan `agent` (SSH_AUTH_SOCK / Pageant).
- Urutan percobaan mengikuti konfigurasi host, bukan tebakan. Jika `auth_method=key` gagal, **jangan** diam-diam jatuh ke password.
- **Kredensial tidak pernah dikirim sebelum host key terverifikasi (REQ-18).** Ini urutan yang tidak boleh dibalik.
- Algoritma: preferensi modern (`curve25519-sha256`, `ssh-ed25519`, `chacha20-poly1305`, `aes256-gcm`). Algoritma warisan (`ssh-rsa` SHA-1, `diffie-hellman-group1-sha1`) **nonaktif secara default**, dapat diaktifkan per-host lewat toggle "Legacy Compatibility" dengan peringatan.
- Timeout: TCP connect 10 detik, handshake 15 detik, auth 30 detik (keyboard-interactive butuh waktu manusia).
- Keepalive: `ServerAliveInterval` default 30 detik, 3× gagal = putus dengan pesan jelas.
- Reconnect otomatis: opsional per-host, maksimum 5 percobaan dengan backoff eksponensial, **hanya** jika sesi putus karena jaringan — bukan karena server menolak auth atau user menutup.
- `ProxyJump` berantai diimplementasikan sebagai channel `direct-tcpip` bersarang, bukan spawn proses `ssh`.

**Kriteria penerimaan.**
- **GIVEN** server test dengan Ed25519 key ber-passphrase, **WHEN** koneksi dibuat, **THEN** sukses dan passphrase didekripsi dari vault (menutup **T-11**).
- **GIVEN** server menuntut keyboard-interactive (OTP), **WHEN** koneksi dibuat, **THEN** UI menampilkan prompt dinamis sesuai teks server.
- **GIVEN** host key belum diverifikasi, **WHEN** koneksi diinisiasi, **THEN** **0 byte** kredensial terkirim (dibuktikan dengan capture trafik di test fixture).
- **GIVEN** bastion → target, **WHEN** koneksi via ProxyJump, **THEN** sesi terbuka dan tidak ada proses `ssh` eksternal yang di-*spawn*.

---

### REQ-14 — PTY Stream, Batching & Flow Control Berbasis ACK
**Prioritas:** P0

**Deskripsi.** Aliran data terminal harus cepat, tidak membanjiri IPC, dan tidak menumpuk di memori.

**Aturan bisnis.**
- **Tiga lapis mitigasi wajib** (v1 hanya menerapkan nol dari tiga — temuan **T-14**):
  1. **Tanpa antrean tak terbatas di Rust.** Satu loop `read → batch → emit`. Dilarang channel berbuffer besar; biarkan SSH/TCP window melakukan backpressure alami ke server.
  2. **Batching emit.** Kumpulkan output, *flush* tiap **16 ms atau 4 KB**, mana yang lebih dulu. Emit per-byte akan membunuh IPC Tauri.
  3. **Flow control berbasis ACK.** Frontend memakai `term.write(data, callback)`; Rust berhenti membaca setelah **256 KB** belum di-ACK, dan lanjut setelah ACK diterima. Ini pola resmi xterm.js untuk kasus ini.
- Setiap pane punya `CancellationToken`. Menutup pane membatalkan token; task Tokio wajib selesai ≤ 500 ms. Diverifikasi dengan membandingkan jumlah task hidup sebelum/sesudah.
- Scrollback default 10.000 baris (dapat diatur 1.000–100.000). **Dinyatakan eksplisit:** scrollback adalah buffer frontend dan bukan mitigasi memori backend.
- Input user dikirim *raw*, tanpa batching (latensi lebih penting daripada throughput pada arah ini).

**Kriteria penerimaan.**
- **GIVEN** `yes` berjalan di remote selama 60 detik, **WHEN** dipantau, **THEN** RSS aplikasi tidak tumbuh > 5 MB dan UI tetap responsif (frame time p95 ≤ 32 ms).
- **GIVEN** `cat` file 500 MB, **WHEN** berjalan, **THEN** tidak ada OOM, tidak ada freeze, dan `Ctrl+C` tetap direspons ≤ 200 ms.
- **GIVEN** 10 pane dibuka lalu ditutup, **WHEN** selesai, **THEN** jumlah task Tokio kembali ke baseline ± 2.
- **GIVEN** frontend sengaja berhenti mengirim ACK, **WHEN** 300 KB output mengalir, **THEN** Rust berhenti membaca dari socket (dibuktikan lewat metrik `bytes_unacked`).

---

### REQ-15 — Resize & Window Change
**Prioritas:** P0

**Deskripsi.** PTY punya dimensi; dimensi itu harus mengikuti jendela.

**Aturan bisnis.**
- `FitAddon` xterm.js → event resize → *debounce* 100 ms → perintah Tauri → `channel.window_change(cols, rows, 0, 0)`.
- Resize dipicu oleh: jendela di-*resize*, split pane dibuat/ditutup, divider digeser, sidebar di-*toggle*, panel SFTP dibuka/ditutup, font size diubah.
- Ukuran terakhir per-pane disimpan agar restore layout konsisten.

**Kriteria penerimaan.**
- **GIVEN** `htop` berjalan, **WHEN** jendela di-*resize* dan split dibuat, **THEN** render tetap rapi tanpa artefak.
- **GIVEN** resize digeser cepat 50× dalam 2 detik, **WHEN** selesai, **THEN** jumlah `window_change` yang terkirim ≤ 20 (debounce bekerja) dan dimensi akhir benar.

---

### REQ-16 — Tabs, Split Pane & Session Lifecycle
**Prioritas:** P1

**Deskripsi.** Beberapa sesi dalam satu jendela, masing-masing adalah sesi SSH sungguhan.

**Aturan bisnis.**
- Tab tak terbatas (dibatasi RAM). Setiap tab memuat satu *pane tree*.
- Split horizontal & vertikal, bersarang, maksimum 4 pane per tab (batas praktis untuk keterbacaan; dapat dinaikkan di Settings dengan peringatan).
- **Setiap pane adalah sesi SSH independen dengan channel PTY-nya sendiri.** Placeholder statis seperti `[Secondary Terminal Pane Split Ready]` adalah **pelanggaran requirement**, bukan tahap sementara — ini temuan **T-03** dan menjadi test anti-regresi wajib.
- *Broadcast input*: satu toggle mengirim keystroke ke seluruh pane di tab aktif. Saat aktif, UI menampilkan indikator merah menyala — mengetik `rm -rf` ke 4 server produksi sekaligus harus terasa berbahaya.
- Drag-and-drop tab untuk mengurutkan; tab dapat diberi nama & warna manual.
- Menutup tab dengan sesi aktif meminta konfirmasi (dapat dimatikan di Settings).
- Restore layout saat start: struktur tab/pane dipulihkan, **sesi SSH tidak otomatis tersambung** — user menekan "Reconnect". Auto-connect saat startup ke server produksi adalah perilaku berbahaya.

**Kriteria penerimaan.**
- **GIVEN** split pane dibuat, **WHEN** `hostname` diketik di pane kedua, **THEN** output berasal dari server sungguhan, dan test integrasi membuktikan ada **2 channel SSH terbuka**.
- **GIVEN** broadcast aktif ke 3 pane, **WHEN** user mengetik `echo x`, **THEN** ketiga server menerima perintah itu.
- **GIVEN** 4 tab × 2 pane, **WHEN** aplikasi ditutup dan dibuka, **THEN** struktur pulih dan 0 sesi tersambung otomatis.

---

### REQ-17 — Rendering, Truecolor, Clipboard & Paste Safety
**Prioritas:** P1

**Deskripsi.** Terminal harus terasa seperti terminal native.

**Aturan bisnis.**
- xterm.js v5 dengan WebGL renderer (fallback canvas). Dukungan 256-color dan truecolor (24-bit).
- `TERM=xterm-256color`, `COLORTERM=truecolor`.
- Bracketed paste mode aktif. Dukungan ligature font, Nerd Font glyph, dan CJK width.
- **Paste safety (mengikat temuan I-3 dari `prd.md`):** menempel teks yang mengandung newline **atau** > 500 karakter memunculkan dialog pratinjau dengan jumlah baris & karakter, plus tombol Batal. Dapat dimatikan per-user di Settings, **tidak** dapat dimatikan untuk host bertag `production`.
- `Ctrl+C`: jika ada seleksi aktif → copy; jika tidak → kirim SIGINT ke remote. `Ctrl+Shift+C`/`Ctrl+Shift+V` selalu copy/paste.
- URL di output dapat diklik (`Ctrl+Click`), dibuka lewat `tauri-plugin-opener` — **bukan** `window.open()` (temuan I-9).
- Pencarian dalam scrollback (`Ctrl+F`) dengan highlight & navigasi hasil.
- Zoom font `Ctrl+=` / `Ctrl+-` / `Ctrl+0`.

**Kriteria penerimaan.**
- **GIVEN** skrip uji truecolor 24-bit, **WHEN** dijalankan, **THEN** gradasi warna tampil benar.
- **GIVEN** teks 5 baris di-*paste*, **WHEN** ditempel, **THEN** dialog konfirmasi muncul menyebut "5 baris".
- **GIVEN** host bertag `production`, **WHEN** user mencoba mematikan paste safety, **THEN** toggle terkunci dengan penjelasan.

---

### REQ-18 — TOFU Host Key Verification (Anti-MITM)
**Prioritas:** P0

**Deskripsi.** Server harus membuktikan identitasnya sebelum menerima rahasia apa pun.

**Aturan bisnis.**
- **`InsecureIgnoreHostKey` dan padanannya di `russh` dilarang keras.** Test arsitektur CI memindai pola ini dan menggagalkan build.
- Kunci identitas entri: `(hostname, port, key_type)`. Satu server boleh punya beberapa tipe kunci.
- Koneksi pertama: tampilkan fingerprint **SHA-256 base64** + *randomart* ASCII, tahan koneksi sampai user menekan "Terima". Tombol "Terima" tidak boleh menjadi default yang ter-*focus* (mencegah Enter refleks).
- Koneksi berikutnya: fingerprint cocok → lanjut diam-diam.
- **Fingerprint berubah → blokir mutlak.** Layar peringatan merah penuh, **tidak ada tombol "Lanjutkan"**. Satu-satunya jalan adalah membuka Settings → Known Hosts dan menghapus entri lama secara sadar, dengan konfirmasi ketik ulang hostname.
- Kredensial **tidak pernah** dikirim sebelum verifikasi selesai.
- Impor dari `~/.ssh/known_hosts` didukung (termasuk format ter-*hash*).
- `host_keys` berisi `first_seen_at`, `last_seen_at`, `verified_by` (`user` | `imported` | `pinned`), dan `revoked_at`.
- **Implementasi wajib benar-benar terpasang**, bukan sekadar ada. Ini temuan **T-01**: di v1, `ConnectTerminal` memanggil `NewTOFU(ctx, nil)` — store `nil` — yang membuat verifikasi mati total sekaligus rawan panic. Test integrasi wajib membuktikan store terpasang.

**Kriteria penerimaan.**
- **GIVEN** host baru, **WHEN** koneksi diinisiasi, **THEN** dialog fingerprint muncul dan koneksi tertahan sampai user memutuskan.
- **GIVEN** host tersimpan dengan fingerprint X, **WHEN** server menyajikan fingerprint Y, **THEN** koneksi diblokir, peringatan MITM tampil, dan **0 byte** kredensial terkirim.
- **GIVEN** pemindaian kode di CI, **WHEN** mencari pola bypass verifikasi, **THEN** 0 hasil di luar `#[cfg(test)]`.
- **GIVEN** test integrasi `test_tofu_store_is_wired`, **WHEN** koneksi dibuat lewat jalur produksi, **THEN** handler host key yang terpasang adalah `TofuVerifier` dengan store non-null.

---

## BAGIAN E — FILES MANAGER (SFTP)

### REQ-19 — SFTP Dual-Pane Explorer
**Prioritas:** P1

**Deskripsi.** Manajemen file remote tanpa meninggalkan aplikasi.

**Aturan bisnis.**
- Panel kiri = filesystem lokal, panel kanan = remote. Keduanya punya breadcrumb, sort (nama/ukuran/tanggal/permission), filter, dan toggle file tersembunyi.
- SFTP memakai subsistem SSH yang sama dengan sesi terminal — **satu koneksi, channel terpisah**. Dilarang membuka koneksi TCP kedua ke server yang sama untuk SFTP.
- Operasi: navigasi, buat folder, rename, hapus (dengan konfirmasi), copy path, refresh, dan buka file teks di editor internal (≤ 1 MB, baca-tulis).
- Direktori besar (> 1.000 entri) dimuat bertahap (*virtual scroll*), tidak memblokir UI.
- Symlink ditampilkan dengan ikon berbeda dan target-nya; mengikuti symlink adalah aksi eksplisit.
- Panel remote menampilkan pemilik, grup, permission (oktal + rwx), ukuran manusiawi, dan mtime.

**Kriteria penerimaan.**
- **GIVEN** sesi SSH aktif, **WHEN** panel SFTP dibuka, **THEN** direktori home remote tampil ≤ 1 detik dan **tidak ada** koneksi TCP baru ke server (dibuktikan dengan hitungan socket).
- **GIVEN** direktori berisi 5.000 file, **WHEN** dibuka, **THEN** UI tetap responsif dan entri dimuat bertahap.
- **GIVEN** panel SFTP terbuka, **WHEN** backend diperiksa, **THEN** operasi benar-benar memanggil subsistem SFTP — bukan UI kosong (menutup **T-04**).

---

### REQ-20 — Transfer Engine: Queue, Progress, Cancel, Resume
**Prioritas:** P1

**Deskripsi.** Transfer file harus dapat dipercaya pada koneksi yang tidak dapat dipercaya.

**Aturan bisnis.**
- Drag-and-drop dua arah (OS → panel remote, panel remote → OS, dan antar panel).
- Antrean transfer global dengan konkurensi maksimum 3 (dapat diatur 1–8). Antrean bertahan lintas restart aplikasi lewat tabel `sftp_transfers`.
- Progress per-file: persen, byte terkirim, kecepatan, ETA. Progress agregat di status bar.
- **Cancel** membatalkan `CancellationToken` dan **menghapus file parsial di tujuan**, kecuali transfer dapat di-*resume*.
- **Resume**: untuk file > 10 MB, state (`bytes_done`, `mtime`, `size`, `sha256_partial`) disimpan. Melanjutkan memakai `pread`/`pwrite` dari offset. Sebelum melanjutkan, **validasi** bahwa ukuran & mtime sumber belum berubah; jika berubah, mulai dari nol dengan pemberitahuan.
- Transfer direktori rekursif dengan hitungan file & total byte di muka.
- Konflik nama: dialog Overwrite / Skip / Rename / Apply-to-all.
- Preservasi mtime dan permission (opsional, default aktif).
- Batas: file tunggal hingga 50 GB (diuji dengan file sparse).

**Kriteria penerimaan.**
- **GIVEN** upload file 1 GB, **WHEN** koneksi diputus di 40%, **THEN** state tersimpan dan melanjutkan dari ~40% (bukan dari 0) setelah reconnect.
- **GIVEN** transfer dibatalkan di 50%, **WHEN** dicek, **THEN** file parsial terhapus dari tujuan.
- **GIVEN** aplikasi ditutup saat 3 transfer mengantre, **WHEN** dibuka lagi, **THEN** antrean tampil dengan status `paused` dan dapat dilanjutkan.
- **GIVEN** file sumber berubah di tengah pause, **WHEN** resume dicoba, **THEN** sistem mendeteksi dan memulai ulang dengan pemberitahuan.

---

### REQ-21 — Permission Editor & Path Safety
**Prioritas:** P1

**Deskripsi.** Mengubah permission remote tanpa mengetik `chmod`, dan tanpa membiarkan path berbahaya lolos.

**Aturan bisnis.**
- Editor permission visual: matriks checkbox owner/group/other × read/write/execute, dengan pratinjau nilai oktal langsung. Dukungan `setuid`/`setgid`/`sticky` di mode lanjutan.
- `chown`/`chgrp` tersedia; kegagalan karena kurang privilese ditampilkan sebagai pesan jelas, bukan error mentah.
- Terapkan rekursif ke direktori dengan konfirmasi eksplisit dan hitungan objek terdampak.
- **Path safety (wajib):** seluruh path remote dinormalisasi dan divalidasi. Segmen `..` yang keluar dari root operasi ditolak. Nama file dari server yang mengandung `../`, newline, null byte, atau kontrol ANSI **disanitasi saat dirender** — output server tidak boleh menyuntik escape sequence ke UI kita.
- Operasi hapus di path sensitif (`/`, `/etc`, `/usr`, `/bin`, `/boot`, `/var`, home root) memerlukan konfirmasi ketik ulang path.

**Kriteria penerimaan.**
- **GIVEN** file 644, **WHEN** user mencentang execute-owner, **THEN** pratinjau menampilkan `744` dan setelah Apply `stat` remote mengonfirmasi.
- **GIVEN** server mengembalikan nama file `../../etc/passwd`, **WHEN** dirender & dioperasikan, **THEN** ditolak dan dicatat, tidak pernah menulis di luar direktori tujuan.
- **GIVEN** nama file mengandung escape ANSI, **WHEN** ditampilkan, **THEN** ditampilkan literal tanpa mengubah warna/kursor UI.

---

## BAGIAN F — PRODUKTIVITAS & OPERASI

### REQ-22 — Snippets Engine
**Prioritas:** P1

**Deskripsi.** Perpustakaan perintah yang dapat dijalankan ke sesi aktif.

**Aturan bisnis.**
- Snippet: `label`, `body` (multi-baris), `description`, `tags[]`, `group_id` opsional, `auto_enter` (bool), `requires_confirmation` (bool).
- **Variabel dinamis** dengan sintaks `{{nama}}`:
  - Built-in: `{{host}}`, `{{hostname}}`, `{{port}}`, `{{user}}`, `{{group}}`, `{{date}}`, `{{datetime}}`, `{{uuid}}`.
  - Custom: variabel tak dikenal memunculkan form isian sebelum eksekusi, dengan nilai default & riwayat.
- **Dilarang** menyubstitusi variabel dengan isi field terenkripsi (mis. `{{password}}`). Menempelkan password ke buffer terminal berarti menuliskannya ke history shell remote.
- Mode eksekusi: **Insert** (tulis ke prompt, tunggu user menekan Enter) atau **Run** (tulis + Enter otomatis). Default **Insert** — Run harus dipilih sadar per-snippet.
- `requires_confirmation` memunculkan pratinjau perintah final (setelah substitusi) sebelum dikirim. Wajib aktif otomatis bila body mengandung pola berbahaya (`rm -rf`, `mkfs`, `dd of=`, `> /dev/sd`, `shutdown`, `reboot`).
- Snippet startup per-host (`startup_snippet_id`) dijalankan otomatis setelah koneksi berhasil.
- Eksekusi ke seluruh pane aktif jika broadcast menyala.

**Kriteria penerimaan.**
- **GIVEN** snippet `ssh-keygen -f {{filename}}`, **WHEN** dijalankan, **THEN** form isian `filename` muncul sebelum apa pun dikirim.
- **GIVEN** snippet berisi `rm -rf /tmp/x`, **WHEN** dijalankan, **THEN** pratinjau konfirmasi muncul meskipun `requires_confirmation` tidak dicentang manual.
- **GIVEN** snippet berisi `{{password}}`, **WHEN** disimpan, **THEN** ditolak dengan penjelasan.

---

### REQ-23 — Port Forwarding (Local, Remote, Dynamic SOCKS5)
**Prioritas:** P1

**Deskripsi.** Tunneling sebagai objek yang dikelola, bukan perintah yang diketik.

**Aturan bisnis.**
- Tiga tipe: **Local** (`-L`), **Remote** (`-R`), **Dynamic/SOCKS5** (`-D`).
- Aturan tersimpan per-host di tabel `port_forwards`, dapat dinyalakan/dimatikan dengan satu klik, dan opsional `auto_start` saat host tersambung.
- **Default bind address = `127.0.0.1`.** Bind ke `0.0.0.0` memerlukan checkbox persetujuan eksplisit dengan peringatan bahwa layanan akan terekspos ke jaringan lokal.
- Deteksi port bentrok sebelum bind, dengan saran port bebas terdekat.
- Indikator status: `stopped` | `starting` | `active` | `error`, plus hitungan koneksi aktif dan byte lewat.
- **Saat sesi SSH ditutup atau vault terkunci, seluruh listener terkait wajib ditutup bersih.** Listener yatim adalah lubang keamanan. Diverifikasi dengan pemeriksaan socket.
- SOCKS5 mendukung CONNECT; resolusi DNS terjadi di sisi remote (mencegah kebocoran DNS).

**Kriteria penerimaan.**
- **GIVEN** aturan Local `8080 → localhost:80`, **WHEN** diaktifkan, **THEN** `curl localhost:8080` mengembalikan konten dari server remote.
- **GIVEN** SOCKS5 di 1080 aktif, **WHEN** `curl --socks5-hostname localhost:1080 http://internal.host`, **THEN** berhasil dan DNS diresolusi di remote.
- **GIVEN** 3 tunnel aktif, **WHEN** sesi ditutup, **THEN** 0 listener tersisa (dibuktikan dengan enumerasi socket proses).
- **GIVEN** user mencoba bind `0.0.0.0` tanpa mencentang persetujuan, **WHEN** Apply, **THEN** ditolak.

---

### REQ-24 — Server Real-time Monitoring
**Prioritas:** P2

**Deskripsi.** Telemetri server tanpa memasang agent apa pun.

**Aturan bisnis.**
- Metrik: CPU (%, per-core opsional), RAM (used/total/cached), Swap, Disk (per-mount, used/total), Network (rx/tx bytes/s), Load average (1/5/15), Uptime, jumlah proses, top-5 proses berdasarkan CPU & RAM.
- Pengumpulan lewat **channel `exec` terpisah**, bukan menulis ke PTY interaktif user. Perintah monitoring **tidak boleh** muncul di layar terminal user maupun di history shell.
- Interval polling dapat dikonfigurasi, default **5 detik**, minimum 1 detik, dengan peringatan beban pada interval rendah.
- Sumber data berurutan: `/proc` (Linux, disukai — paling murah) → `sysctl`/`vm_stat` (macOS/BSD) → fallback `top`/`vmstat`. **Degradasi anggun:** jika perintah tidak ada (mis. `free` di BSD), metrik itu ditampilkan `n/a` dan **tidak merusak** panel lain.
- Riwayat disimpan di `monitor_samples` dengan retensi 24 jam (rolling), lalu diringkas per-5-menit selama 7 hari. Retensi dapat diatur atau dimatikan.
- Monitoring **opt-in per-host**, default mati. Ia memakan bandwidth dan menambah proses di server orang lain.
- Grafik sparkline real-time di sidebar host + panel penuh di tab terpisah.

**Kriteria penerimaan.**
- **GIVEN** monitoring aktif, **WHEN** sesi terminal berjalan, **THEN** tidak ada satu pun perintah monitoring yang muncul di layar user atau di `history`.
- **GIVEN** host FreeBSD tanpa `free`, **WHEN** monitoring aktif, **THEN** RAM `n/a` dan CPU/Load tetap terisi.
- **GIVEN** monitoring 1 jam pada interval 5 detik, **WHEN** DB diperiksa, **THEN** pertumbuhan ukuran DB ≤ 2 MB per host.

---

### REQ-25 — Command Audit Logs
**Prioritas:** P1

**Deskripsi.** Jejak audit perintah, dengan default yang menghormati privasi.

**Aturan bisnis.**
- **Default: hanya perintah yang dicatat, output TIDAK.** Perekaman output adalah opt-in eksplisit per-host.
- Perintah dideteksi dari input user yang diakhiri Enter, dengan rekonstruksi buffer edit baris (menangani backspace, arrow, `Ctrl+U`, `Ctrl+W`).
- **Redaksi otomatis sebelum disimpan:** pola yang cocok dengan `password=`, `--token`, `-p <x>`, `export *_KEY=`, `AWS_SECRET*`, `Authorization:`, string base64 panjang di posisi argumen, dan seluruh baris yang diawali spasi (konvensi shell "jangan catat") → disimpan sebagai `[REDACTED]`.
- Perintah dan output disimpan **terenkripsi dengan DEK** (`command_enc`, `output_enc`). Ini menutup **T-20** (v1 menyimpan perintah polos tanpa batas ukuran).
- **Output dipotong otomatis di 5 KB per perintah** (kepala 3 KB + ekor 2 KB, dengan penanda `[... N bytes omitted ...]`).
- Retensi default 90 hari, dapat diatur 1–3.650 hari atau "selamanya". Purge otomatis saat start.
- Viewer audit: filter per-host, per-rentang-waktu, pencarian teks (didekripsi di memori, **tidak** ada index plaintext di DB), dan ekspor ke JSON/CSV terenkripsi opsional.
- Exit code dan durasi dicatat bila dapat diketahui.

**Kriteria penerimaan.**
- **GIVEN** user mengetik `mysql -u root -psecret123`, **WHEN** dicatat, **THEN** baris tersimpan adalah `mysql -u root -p[REDACTED]`.
- **GIVEN** output 50 KB, **WHEN** perekaman output aktif, **THEN** yang tersimpan ≤ 5 KB dengan penanda pemotongan.
- **GIVEN** file DB dipindai byte mentah, **WHEN** mencari perintah yang dicatat, **THEN** 0 hasil plaintext.

---

### REQ-26 — SSH Keys Manager
**Prioritas:** P1

**Deskripsi.** Mengelola kunci privat di dalam vault, bukan berserakan di disk.

**Aturan bisnis.**
- **Generate:** Ed25519 (default, direkomendasikan) dan RSA-4096. ECDSA didukung untuk impor, **tidak** untuk generate.
- Passphrase pada kunci baru **sangat dianjurkan**; melewatinya memerlukan konfirmasi sadar.
- **Import:** tempel isi atau pilih file; format OpenSSH dan PKCS#8 (PEM/DER). Deteksi otomatis tipe & bit. Kunci ber-passphrase dibuka sekali saat impor untuk validasi, lalu **passphrase disimpan terenkripsi DEK** (opsional) atau diminta setiap koneksi.
- **Penyimpanan:** `private_key_enc` selalu terenkripsi dengan DEK. Kunci **tidak pernah** ditulis ke disk dalam bentuk plaintext, termasuk ke file sementara. Saat dibutuhkan `russh`, kunci didekripsi **ke memori saja**.
- **Export:** memerlukan konfirmasi Master Password ulang. Dialog menampilkan peringatan bahwa file hasil ekspor **tidak terenkripsi**.
- Tampilkan fingerprint SHA-256, tipe, panjang bit, komentar, tanggal dibuat, dan daftar host yang memakainya. Menghapus kunci yang masih dipakai host ditolak dengan daftar pemakainya.
- Menutup **T-11**: jalur passphrase wajib benar-benar terhubung dari UI hingga dialer.

**Kriteria penerimaan.**
- **GIVEN** generate Ed25519 dengan passphrase, **WHEN** dipakai untuk koneksi, **THEN** sukses dan passphrase terbaca dari vault.
- **GIVEN** kunci diimpor, **WHEN** disk dipindai (termasuk `$TMPDIR` dan file DB), **THEN** 0 kemunculan header `-----BEGIN OPENSSH PRIVATE KEY-----` dalam bentuk plaintext.
- **GIVEN** kunci dipakai 3 host, **WHEN** dihapus, **THEN** ditolak dengan menyebut ketiga host.

---

### REQ-27 — Deploy Public Key (`ssh-copy-id` internal)
**Prioritas:** P2

**Deskripsi.** Memasang public key ke server dalam satu klik, dengan aman.

**Aturan bisnis.**
- Alur: pilih host → pilih kunci → autentikasi (password atau kunci lain) → append ke `~/.ssh/authorized_keys`.
- Operasi wajib **idempoten**: jika kunci sudah ada, jangan duplikat — laporkan "sudah terpasang".
- Buat `~/.ssh` (mode 700) dan `authorized_keys` (mode 600) jika belum ada. Permission salah diperbaiki, dan perbaikannya dilaporkan.
- **Backup** `authorized_keys` ke `authorized_keys.caterm.bak` sebelum modifikasi.
- Setelah sukses, tawarkan uji koneksi memakai kunci baru **sebelum** menyarankan mematikan auth password. Menyarankan pengerasan sebelum terbukti bekerja adalah cara mengunci diri sendiri di luar server.
- Dukungan `sudo` untuk memasang kunci ke user lain — **di luar cakupan v2.0**.

**Kriteria penerimaan.**
- **GIVEN** host tanpa `~/.ssh`, **WHEN** deploy, **THEN** direktori dibuat 700, file 600, kunci terpasang, koneksi uji sukses.
- **GIVEN** deploy dijalankan 2×, **WHEN** selesai, **THEN** `authorized_keys` berisi kunci itu tepat satu kali.

---

## BAGIAN G — AI COPILOT

### REQ-28 — AI Terminal Copilot (Opt-In)
**Prioritas:** P2

**Deskripsi.** Asisten kontekstual untuk menjelaskan perintah, menganalisis error, dan menyusun skrip.

**Aturan bisnis.**
- `ai_enabled` default **`false`**. Fitur muncul sebagai ajakan, bukan sebagai sesuatu yang sudah menyala.
- **Consent gate:** saat pertama diaktifkan, tampilkan layar penuh yang menyatakan dengan jelas: data apa yang dikirim, ke endpoint mana, bahwa pihak ketiga akan memprosesnya, dan bahwa fitur ini dapat dimatikan kapan saja. Wajib centang "Saya mengerti" — tidak bisa dilewati dengan Enter.
- **Tidak ada endpoint default yang di-*hardcode*.** Ini temuan **T-08**: v1 mengarah ke IP privat internal (lihat `audit-ulang.md` T-08). Di v2, `base_url` **wajib diisi user**; field kosong = fitur mati. Aplikasi menyarankan contoh publik yang umum dikenal (api.anthropic.com, api.openai.com, endpoint Ollama lokal) sebagai teks bantuan, bukan sebagai nilai terisi.
- Protokol penyedia dipilih user secara eksplisit: OpenAI-style `/chat/completions`, Anthropic Messages API, Ollama, atau gateway OpenAI-compatible. **Dilarang menebak protokol dari bentuk URL** (REQ-36).
- **`api_key` disimpan terenkripsi DEK** di `ai_providers.api_key_enc` dengan AAD per-record (REQ-36). Menutup **T-09** (v1 menyimpannya plaintext di `settings`).
- **Konfigurasi penyedia memakai profil ber-validasi (REQ-36).** Beberapa profil boleh tersimpan, tepat satu aktif, dan profil yang belum lulus uji koneksi tidak dapat dijadikan default.
- Mode — seluruhnya bekerja **hanya** di atas data yang sudah diizinkan whitelist REQ-29 (teks terpilih, N baris terakhir buffer, perintah terakhir, pertanyaan user):
  - **Explain** — jelaskan perintah atau output yang dipilih.
  - **Fix** — analisis pesan error terakhir dan usulkan perbaikan.
  - **Generate** — susun perintah dari deskripsi bahasa alami.
  - **Chat** — panel tanya-jawab bebas.
  - **Summarize** — ringkas output panjang yang dipilih user (log panjang, stack trace, keluaran `journalctl`) menjadi beberapa poin. Berguna justru karena output terminal sering terlalu panjang untuk dibaca cepat.
  - **Diagnose** — dari **perintah terakhir + exit code + N baris buffer terakhir**, usulkan penyebab dan langkah berikutnya. **Dilarang membaca `audit_log`, daftar host, atau isi vault** — sumbernya tetap whitelist REQ-29, tanpa pengecualian.
  - **To Snippet** — ubah hasil Generate menjadi snippet tersimpan (REQ-22) lengkap dengan variabel `{{var}}` yang terdeteksi. Penyimpanan tetap menunggu klik user; AI tidak pernah menulis ke vault sendiri.
- Hasil Generate, To Snippet, dan Diagnose **tidak pernah dieksekusi otomatis.** Selalu ditempatkan di prompt dalam mode Insert, menunggu Enter dari user.
- Semua panggilan AI punya timeout 60 detik dan dapat dibatalkan. Kegagalan jaringan tidak pernah mempengaruhi sesi SSH.
- Penanda biaya: tampilkan jumlah token yang terkirim per-permintaan, agar pemakaian terasa.

**Kriteria penerimaan.**
- **GIVEN** instalasi baru, **WHEN** kode dipindai, **THEN** 0 URL endpoint AI ter-*hardcode* (test otomatis mencari pola `http://` dan `https://` di modul AI).
- **GIVEN** `ai_enabled=false`, **WHEN** aplikasi berjalan 30 menit, **THEN** 0 koneksi jaringan keluar.
- **GIVEN** API key tersimpan, **WHEN** file DB dipindai, **THEN** 0 kemunculan plaintext key.
- **GIVEN** AI menghasilkan `rm -rf /`, **WHEN** ditampilkan, **THEN** hanya masuk buffer input, tidak pernah terkirim.
- **GIVEN** profil penyedia belum pernah lulus uji koneksi, **WHEN** user mencoba menjadikannya default, **THEN** ditolak dan UI menyebut lapis validasi yang gagal beserta kode `CAT-AI-0xx` (REQ-36).

---

### REQ-29 — AI Data Minimization & Redaction Guard
**Prioritas:** P0 *(P0 meskipun REQ-28 P2 — jika AI menyala, guard ini wajib sudah benar)*

**Deskripsi.** Apa yang keluar dari mesin harus sesedikit mungkin dan tidak pernah berisi rahasia.

**Aturan bisnis.**
- **Whitelist konteks, bukan blacklist.** Hanya yang berikut boleh dikirim: teks yang **diseleksi user secara eksplisit**, N baris terakhir buffer terminal (default 50, maksimum 200, dapat diatur), perintah terakhir, dan pertanyaan user.
- **Tidak pernah dikirim, dalam kondisi apa pun:** isi vault, password, private key, passphrase, isi field `*_enc`, daftar host, hostname/IP, username, isi file SFTP, API key lain, atau audit log.
- **Redaction Guard berjalan sebelum setiap permintaan** dan memindai payload untuk: private key block, string mirip API key (`sk-`, `ghp_`, `AKIA`, JWT), `password=`, `Authorization:`, alamat IP privat, dan hostname dari vault user. Yang cocok diganti `[REDACTED]`.
- **Pratinjau payload wajib tersedia:** tombol "Lihat yang akan dikirim" menampilkan payload final persis apa adanya, sebelum dikirim. Pada 3 pemakaian pertama, pratinjau ini **dipaksa tampil**.
- Percakapan AI tidak disimpan ke disk secara default; jika disimpan (opt-in), terenkripsi DEK.
- Tidak ada telemetri, analytics, crash reporter daring, atau "improve the product" data collection — **titik**.

**Kriteria penerimaan.**
- **GIVEN** buffer terminal berisi private key, **WHEN** user meminta Explain, **THEN** payload yang terkirim berisi `[REDACTED]` di posisi itu, dibuktikan oleh test yang memeriksa body HTTP.
- **GIVEN** 200 kasus uji berisi pola rahasia, **WHEN** guard dijalankan, **THEN** ≥ 99% terdeteksi, dan setiap kegagalan tercatat sebagai bug P0.
- **GIVEN** pemakaian AI pertama kali, **WHEN** permintaan dikirim, **THEN** pratinjau payload tampil dan menunggu persetujuan.

---

### REQ-36 — Profil Provider AI Kustom & Validasi Konfigurasi
**Prioritas:** P1 *(P1 meskipun REQ-28 berstatus P2 — kalau AI dipakai sama sekali, konfigurasinya wajib bisa dibuktikan benar sebelum dipercaya)*

> **Catatan penomoran.** REQ ini ditambahkan setelah REQ-35 (18 September 2026) sehingga bernomor 36, tetapi ditempatkan di BAGIAN G karena isinya adalah AI. Penomoran mengikuti urutan penambahan, bukan urutan posisi — nomor REQ tidak pernah digeser agar referensi di `task-v2.md` dan commit lama tetap sahih.

**Deskripsi.** Pengguna bebas memakai penyedia AI mana pun — komersial, self-hosted, atau model lokal. Konsekuensinya: aplikasi tidak boleh menebak. Ia wajib menyediakan cara **memvalidasi** konfigurasi itu secara berlapis, dan menolak memakai profil yang belum terbukti hidup.

**Aturan bisnis.**

- **Multi-profil.** Konfigurasi AI tidak lagi berupa satu set field di `settings`, melainkan tabel `ai_providers` (§5.1) yang menampung banyak profil. Tepat **satu** profil berstatus default; sisanya tersimpan siap pakai. Ini kebutuhan nyata: model murah untuk Explain, model kuat untuk Generate, model lokal saat offline.
- **Protokol yang didukung:** `openai_chat` (`/chat/completions`), `anthropic_messages`, `ollama`, dan `custom_openai_compatible` untuk gateway/proxy yang meniru format OpenAI. Protokol dipilih user secara eksplisit — **dilarang mendeteksi otomatis dari bentuk URL**, karena tebakan yang salah mengirim payload ke endpoint yang salah.
- **Tetap tidak ada endpoint bawaan** (REQ-28, T-08). Daftar contoh penyedia boleh ditampilkan sebagai teks bantuan yang dapat disalin, **bukan** sebagai nilai terisi dan bukan sebagai tombol "pakai ini".

**Validasi berlapis.** Profil baru bisa dijadikan default hanya setelah lapis 1–3 lulus.

| Lapis | Nama | Jaringan? | Yang diperiksa |
|---|---|---|---|
| 1 | **Bentuk** | ❌ tidak | `base_url` absolut & berskema `http`/`https`; **`https` wajib** kecuali host loopback/RFC1918 **dan** user mencentang "Saya mengerti lalu lintas ini tidak terenkripsi"; tanpa query string rahasia; `model` tidak kosong; `timeout_sec` 5–300; `context_lines` 1–200; `max_tokens` 16–32768; `api_key` tidak kosong kecuali protokol lokal tanpa auth |
| 2 | **Jangkauan** | ✅ ya, 1 permintaan | Satu permintaan minimal (prompt konstanta ≤ 16 token) ke endpoint yang diisi user |
| 3 | **Kontrak** | — | Respons di-*parse* ke struct kanonik; field wajib tidak ada = gagal, **bukan** ditebak, **bukan** panic |
| 4 | **Katalog model** *(opsional)* | ✅ ya | `ai_models_list` mengisi dropdown model bila provider mendukungnya. **Kegagalan lapis ini tidak memblokir** — user selalu boleh mengetik nama model manual |

- **Kode error stabil untuk hasil validasi** (kontrak publik, REQ-04). Pesan UI wajib menyebut lapis mana yang gagal dan apa langkah perbaikannya:

| Kode | Arti |
|---|---|
| `CAT-AI-001` | URL tidak valid / skema ditolak (lapis 1) |
| `CAT-AI-002` | DNS gagal — host tidak ditemukan |
| `CAT-AI-003` | TLS gagal — sertifikat tidak tepercaya atau kedaluwarsa |
| `CAT-AI-004` | Timeout |
| `CAT-AI-005` | 401/403 — API key ditolak |
| `CAT-AI-006` | 404 — path endpoint salah |
| `CAT-AI-007` | 429 — rate limit |
| `CAT-AI-008` | 5xx — kegagalan sisi penyedia |
| `CAT-AI-009` | Respons bukan JSON atau formatnya tidak dikenali |
| `CAT-AI-010` | Model tidak tersedia di penyedia ini |

- **Uji koneksi tidak pernah otomatis.** Ia berjalan hanya saat user menekan "Uji koneksi", atau sekali saat menyimpan profil bila user mencentang "uji sekarang". Tidak ada *health check* berkala, tidak ada validasi saat startup, tidak ada *retry* di latar. Ini konsekuensi langsung REQ-09: **0 byte tanpa izin**.
- **Prompt uji adalah konstanta.** Ia tidak boleh memuat satu byte pun data user — bukan hostname, bukan isi buffer, bukan nama profil. Tetap melewati Redaction Guard yang sama (REQ-29), karena guard yang dilewati "untuk kasus khusus" adalah guard yang akan dilewati lagi nanti.
- **Rahasia profil** (`api_key`, header tambahan bernilai sensitif) dienkripsi DEK dengan AAD `<provider_id>|ai_providers|<field>`. Setelah disimpan, **tidak pernah ditampilkan kembali** — UI hanya menampilkan 4 karakter terakhir dan tombol "Ganti".
- Profil yang belum pernah lulus validasi diberi badge **"Belum diverifikasi"** dan **tidak dapat dijadikan default**. Ditegakkan di level skema (`CHECK`), bukan hanya di UI.
- **Menghapus profil aktif mematikan AI** (`ai_enabled=false`), bukan diam-diam jatuh ke profil lain. Berpindah penyedia adalah keputusan sadar.
- Berganti profil aktif **tidak membawa riwayat percakapan** lintas profil.
- Ekspor penuh (REQ-31) menyertakan seluruh profil dengan rahasia **tetap terenkripsi apa adanya** — tidak di-enkripsi ulang, sesuai REQ-06.

**Kriteria penerimaan.**
- **GIVEN** `base_url` berskema `http://` menuju host publik, **WHEN** profil disimpan, **THEN** ditolak di lapis 1 dengan `CAT-AI-001` dan **0 byte** keluar dari mesin.
- **GIVEN** `base_url` `http://127.0.0.1:11434` dan checkbox risiko dicentang, **WHEN** disimpan, **THEN** diterima.
- **GIVEN** API key salah, **WHEN** "Uji koneksi" ditekan, **THEN** hasilnya `CAT-AI-005` yang spesifik — bukan pesan "gagal terhubung" yang menyamaratakan semua kegagalan.
- **GIVEN** uji koneksi dijalankan, **WHEN** body permintaan diperiksa di test lewat mock server, **THEN** body **tidak memuat** hostname, username, isi buffer, atau string apa pun yang berasal dari vault.
- **GIVEN** profil dengan `last_validation_status` bukan `ok`, **WHEN** dicoba dijadikan default, **THEN** ditolak oleh constraint skema.
- **GIVEN** 10 profil tersimpan dengan API key berbeda, **WHEN** file `.db`, `.db-wal`, dan `.db-shm` dipindai byte mentah, **THEN** **0** kemunculan plaintext key.
- **GIVEN** profil default dihapus, **WHEN** selesai, **THEN** `ai_enabled=false` dan tidak ada profil lain yang naik otomatis.
- **GIVEN** `ai_enabled=false` dan tidak ada profil, **WHEN** aplikasi berjalan 30 menit, **THEN** 0 koneksi jaringan keluar (REQ-09).

---

### REQ-37 — Zero-Knowledge 2FA / TOTP Authenticator Vault & SSH Interactive Auto-Inject
**Prioritas:** P1 *(Killer Feature — Pembebasan Sysadmin dari Ketergantungan Ponsel saat Login SSH)*
**Kategori:** Autentikasi & Keamanan (RFC 6238)

**Deskripsi.** Server produksi modern sering kali menerapkan autentikasi dua faktor berbasis waktu (TOTP) seperti `pam_google_authenticator` pada akses SSH. CATerm menyediakan engine 2FA/TOTP terintegrasi dengan brankas Zero-Knowledge terenkripsi SQLCipher dan fitur penyuntikan otomatis (*auto-inject*) saat menerima prompt verifikasi SSH.

**Aturan bisnis.**
1. **RFC 6238 TOTP Engine:**
   - Menghasilkan token 6 digit standar berbasis waktu dengan interval 30 detik dan algoritma HMAC-SHA1.
   - Mendukung parsing format Base32 secret murni dan standard `otpauth://totp/...` URI.
2. **Penyimpanan Terenkripsi Zero-Knowledge:**
   - Kunci rahasia 2FA (`totp_secret_enc`) disimpan terenkripsi di tabel `hosts` untuk server spesifik, serta tabel mandiri `totp_entries` untuk akun cloud umum (AWS, Cloudflare, GitHub, Email).
   - Seluruh secret key dienkripsi menggunakan DEK SQLCipher dengan AAD. Tidak ada plaintext secret di disk.
3. **SSH Interactive Hook & Auto-Inject:**
   - Driver SSH mendeteksi prompt server `keyboard-interactive` yang cocok dengan pola regex: `[Vv]erification code|[Oo][Tt][Pp]|[2][Ff][Aa]|[Aa]uthenticator`.
   - Jika host memiliki secret 2FA terdaftar dan `auto_inject_totp = true`, token 6 digit langsung dikirimkan ke server secara non-blocking.
   - Jika mode manual aktif, terminal menampilkan badge mengambang di pojok kanan atas: `[⚡ Isi OTP: 358 630 (22s)]` yang otomatis menyuntikkan token saat diklik.
4. **Dedicated 2FA Authenticator Tab (Sidebar):**
   - Panel khusus di antarmuka CATerm yang menampilkan seluruh akun 2FA dengan kartu token, 6 digit berukuran besar, lingkaran countdown visual 30 detik (SVG stroke-dashoffset), dan tombol 1-Click Copy.
5. **Zero-Overhead & Keamanan Memori:**
   - Komputasi TOTP instan (< 1 mikrodetik), zero dynamic allocation, RAM < 10 KB.
   - Secret key di-*zeroize* dari RAM segera setelah kalkulasi selesai.

**Kriteria penerimaan.**
- **GIVEN** server SSH meminta `Verification code:`, **WHEN** `auto_inject_totp` aktif pada host tersebut, **THEN** CATerm menghitung 6 digit TOTP saat itu dan menyuntikkannya ke socket SSH dalam < 1 ms tanpa delay input pengguna.
- **GIVEN** tab 2FA Authenticator dibuka, **WHEN** waktu melewati batas 30 detik (epoch rollover), **THEN** token 6 digit diperbarui otomatis dan lingkaran countdown mereset halus ke 30s tanpa flicker.
- **GIVEN** secret Base32 `UOJH07NNTRXYHEQQ`, **WHEN** digenerate pada timestamp yang sama, **THEN** menghasilkan token 6 digit identik dengan Google Authenticator / `2fa.suite.my.id`.

---

### REQ-38 — Agentless Scheduled Tasks, Action Playbooks & SFTP Auto-Backup Engine
**Prioritas:** P1 *(High-Utility DevOps Automation — Stasiun Otomasi Server & Backup Terpusat)*
**Kategori:** Otomasi, Penjadwalan & Backup Terenkripsi

**Deskripsi.** Modul eksekusi tugas terjadwal (*scheduled tasks*) dan penarikan cadangan otomatis (*auto-backup pull*) secara mandiri dari PC/laptop lokal ke server-server remote tanpa perlu menginstal agent/daemon tambahan di server target.

**Aturan bisnis.**
1. **Tokio Async Cron Scheduler:**
   - Mendukung sintaks cron standar (`0 2 * * *`), interval (`every 6h`, `every 30m`), dan one-shot (`at <ISO_TIMESTAMP>`).
   - Berjalan pada background thread terisolasi saat aplikasi diminimize ke System Tray dengan pemakaian CPU 0,0% saat idle dan RAM < 25 MB.
2. **SFTP Auto-Backup Pull Pipeline:**
   - Mendukung 4 tahapan atomik: (1) Remote Pre-Command (e.g. `mysqldump ... | gzip`), (2) SFTP Atomic Download (`.part` file), (3) Remote Post-Command Cleanup (`rm -f /tmp/...`), (4) Local Retention Rotation (menghapus backup lokal yang lebih tua dari $N$ file/hari).
3. **Remote SSH Playbooks & Health Watchdog:**
   - Menjalankan script bash multi-line terjadwal di target host dengan penanganan timeout, streaming log output, dan penangkapan exit code.
   - Pemicu notifikasi OS native dan webhook saat tugas berhasil atau gagal.
4. **Zero-Knowledge Security:**
   - Kredensial SSH didekripsi langsung dari SQLCipher vault saat runtime dan di-*zeroize* segera setelah koneksi selesai. Log riwayat stdout/stderr tersimpan terenkripsi di tabel `task_execution_logs`.

**Kriteria penerimaan.**
- **GIVEN** jadwal backup harian `0 2 * * *`, **WHEN** waktu jam 02:00 tiba saat CATerm berada di system tray, **THEN** CATerm menjalankan dump di remote, mengunduh file hasil kompresi via SFTP ke folder lokal, dan memunculkan notifikasi desktop OS.
- **GIVEN** penarikan backup SFTP terputus di tengah jalan, **WHEN** koneksi terputus, **THEN** file parsial `.part` tidak menimpa file backup valid sebelumnya.

---

## BAGIAN H — SETTINGS, BACKUP & PORTABILITAS

### REQ-30 — Settings, Theming & Keybindings
**Prioritas:** P1

**Deskripsi.** Aplikasi mengikuti kebiasaan penggunanya.

**Aturan bisnis.**
- Kategori: General, Appearance, Terminal, Security, Keys, AI, Sync *(disabled)*, Team *(disabled)*, Advanced, About.
- Appearance: tema gelap (default, `#0b0f19` + aksen `#58a6ff`), tema terang, ikut sistem, plus tema terminal kustom (impor format Windows Terminal / iTerm2). Font keluarga & ukuran, tinggi baris, bentuk & kedip kursor, opacity.
- Terminal: scrollback, bell, `TERM`, encoding, shell integration, perilaku copy-on-select.
- Security: timeout auto-lock, kebijakan clipboard (auto-clear setelah N detik), paste safety, retensi audit, kebijakan known hosts.
- **Keybindings dapat dikonfigurasi penuh** dengan deteksi konflik, dan preset (Default, Termius-like, iTerm2-like, tmux-like). Ini bagian dari "konteks lingkungan" yang gagal di-backup Termius (§1.3) — karenanya keybindings **wajib** ikut dalam ekspor REQ-31.
- Advanced: lokasi direktori data, mode portable, level log, reset aplikasi.
- Seluruh setting tersimpan di tabel `settings` (key-value bertipe), bukan file eksternal — agar ikut terbawa backup dan (kelak) sync.

**Kriteria penerimaan.**
- **GIVEN** keybinding diubah, **WHEN** aplikasi restart, **THEN** perubahan bertahan.
- **GIVEN** dua aksi diberi kombinasi sama, **WHEN** disimpan, **THEN** konflik terdeteksi dan ditunjukkan.
- **GIVEN** tema iTerm2 diimpor, **WHEN** diterapkan, **THEN** 16 warna ANSI + foreground/background/cursor sesuai.

---

### REQ-31 — Full Environment Backup & Restore
**Prioritas:** P0

**Deskripsi.** Jawaban langsung atas cacat §1.3. Backup berarti **seluruh lingkungan**, bukan daftar host.

**Aturan bisnis.**
- Satu perintah menghasilkan satu file `.catermvault` yang berisi **seluruhnya**: hosts, groups, tags, snippets, ssh_keys, host_keys (known hosts), port_forwards, settings, keybindings, tema kustom, konfigurasi AI (key tetap terenkripsi), layout tab/pane, statistik pemakaian, dan opsional audit log.
- Format: arsip **Zstandard** berisi manifest JSON (deterministik: kunci terurut, indentasi tetap, LF) + data. **Ciphertext disalin verbatim** — tidak di-enkripsi ulang (REQ-06).
- Dua mode:
  1. **Encrypted (default):** seluruh arsip dienkripsi AES-256-GCM dengan kunci turunan Argon2id dari passphrase ekspor (boleh sama dengan Master Password). `kdf_salt` + `wrapped_dek` disertakan sehingga restore di mesin lain hanya butuh passphrase. *(Ini persis yang membuat sync v1 tidak jalan lintas mesin — tanpa menyertakan material kunci, ciphertext tidak bisa dibuka di mana pun.)*
  2. **Plaintext-metadata (opt-in, dengan peringatan keras):** struktur terbaca, tapi field kredensial **tetap terenkripsi** dan tidak dapat dibuka tanpa DEK. Berguna untuk audit dan diff.
- **Restore** punya tiga mode: **Replace** (ganti total, backup DB lama dulu ke `.bak`), **Merge** (Last-Write-Wins per-record berbasis `updated_at`), dan **Selective** (pilih entitas).
- **Restore tidak pernah otomatis.** Selalu tampilkan pratinjau: N host baru, N diperbarui, N tidak berubah, N konflik. User menekan Apply.
- **Absennya record ≠ penghapusan.** Record yang ada di lokal tapi tidak di backup **dibiarkan**. Penghapusan hanya dirambatkan lewat *tombstone* (`deleted_at` terisi).
- Backup otomatis terjadwal (harian/mingguan) ke folder pilihan user, dengan rotasi N salinan. Ini **file lokal**, bukan cloud.
- **Tersedia saat langganan mati dan saat offline** (REQ-09).

**Kriteria penerimaan.**
- **GIVEN** vault dengan 50 host, 20 snippet, 5 key, tema kustom, dan keybinding kustom, **WHEN** diekspor lalu di-*restore* di instalasi bersih di mesin lain, **THEN** seluruh 50 host tersambung dengan kredensial benar DAN tema DAN keybinding identik.
- **GIVEN** ekspor dijalankan 2× tanpa perubahan data, **WHEN** kedua file dibandingkan, **THEN** manifest JSON-nya **byte-identik** (determinisme).
- **GIVEN** restore mode Merge, **WHEN** lokal punya host yang tidak ada di backup, **THEN** host itu tetap ada setelah restore.
- **GIVEN** entitlement `expired`, **WHEN** ekspor dijalankan, **THEN** berhasil penuh.

---

## BAGIAN I — PHASE 2: SYNC & TEAM (DISABLED DI v2.0)

### REQ-32 — E2EE Multi-Device Sync *(DISABLED / NEXT FEATURE)*
**Prioritas:** P1 *(untuk bagian UI placeholder & skema)* · P2 *(untuk jalur jaringan, Phase 2)*

**Deskripsi.** Sinkronisasi antar-perangkat di mana server tidak pernah dapat membaca apa pun.

**Aturan bisnis — status di v2.0 GA.**
- Layar Sync **ada dan lengkap secara visual**: daftar perangkat, status sinkronisasi terakhir, pilihan Managed Cloud vs Self-hosted, tombol "Aktifkan Sync".
- Seluruh kontrol dalam keadaan **non-aktif**, dengan badge **"Coming Soon · Managed Sync"**.
- **Tidak ada kode jaringan yang aktif.** Modul `sync` ter-*compile* di belakang feature flag `sync-net` yang **mati** di build rilis v2.0. Test membuktikan build rilis tidak memuat klien HTTP sync.
- Skema tabel (`sync_state`, `sync_queue`, `devices`) **dibuat sejak v2.0** agar tidak perlu migrasi besar saat diaktifkan.
- Mesin resolusi konflik (**LWW-Element-Set CRDT** per-record dengan `updated_at` + `device_id` sebagai tiebreaker) **ditulis dan diuji penuh** di v2.0 sebagai fungsi murni — hanya transport-nya yang mati.

**Aturan bisnis — desain Phase 2 (mengikat implementasi mendatang).**
- Unit sync = **record**, bukan seluruh file DB. Blob per-record memungkinkan merge granular dan delta kecil.
- Payload yang diunggah: `{ record_id, entity, updated_at, device_id, ciphertext_blob, blob_nonce, hmac }`. **Server melihat:** ID acak, timestamp, ukuran. **Server tidak pernah melihat:** hostname, label, isi apa pun.
- **Metadata yang di v2.0 tidak terenkripsi (REQ-06) WAJIB dienkripsi sebelum diunggah.** Blob sync mengenkripsi *seluruh baris* dengan DEK, termasuk `label` dan `hostname`. Ini perbedaan penting antara "aman di disk sendiri" dan "aman di server orang lain".
- Otentikasi ke relay memakai token perangkat; **Master Password tidak pernah dikirim**, dan kredensial akun cloud terpisah dari Master Password vault.
- Deteksi konflik memunculkan UI perbandingan; **tidak ada auto-overwrite senyap**.
- `wrapped_dek` + `kdf_salt` disinkronkan agar perangkat kedua bisa membuka data dengan Master Password yang sama. Tanpa ini, sync mati total lintas mesin.
- Transport: HTTPS + certificate pinning opsional untuk Managed Cloud.
- **Sync dapat dimatikan kapan saja; mematikannya tidak menghapus apa pun secara lokal.**

**Kriteria penerimaan (v2.0).**
- **GIVEN** build rilis v2.0, **WHEN** biner dipindai untuk simbol klien HTTP sync, **THEN** tidak ditemukan.
- **GIVEN** layar Sync dibuka, **WHEN** user menekan "Aktifkan Sync", **THEN** muncul penjelasan "Coming Soon" dan **0 permintaan jaringan** terjadi.
- **GIVEN** test unit CRDT dengan 500 skenario konflik, **WHEN** dijalankan, **THEN** hasilnya deterministik dan konvergen terlepas dari urutan penerapan.

---

### REQ-33 — Subscription & Entitlement Client *(DISABLED / NEXT FEATURE)*
**Prioritas:** P2

**Deskripsi.** Status langganan mengatur **sinkronisasi**, dan tidak pernah mengatur hal lain.

**Aturan bisnis.**
- Model harga (§10): Bulan ke-1 **$1**, bulan ke-2 dst **$3/bulan**, Team seat **+$1/bulan per anggota**.
- Entitlement di-*cache* lokal dengan masa berlaku 30 hari. **Offline tidak pernah berarti "expired".**
- Status: `none` | `active` | `grace` (14 hari setelah gagal bayar) | `expired`.
- **Efek `expired` tepat satu: unggahan sync berhenti.** Yang **tidak** terjadi: pengurangan fitur lokal, penghapusan data, nag-screen pemblokir, pembatasan ekspor, pembatasan jumlah host.
- Notifikasi status langganan tampil satu kali di area status bar, dapat ditutup permanen.
- Pembayaran ditangani **penuh di browser eksternal** lewat payment provider. Aplikasi **tidak pernah** meminta, menyentuh, atau menyimpan nomor kartu.
- Self-hosted relay **tidak memerlukan langganan sama sekali** dan tidak melakukan pengecekan entitlement apa pun.

**Kriteria penerimaan.**
- **GIVEN** status `expired`, **WHEN** seluruh suite fungsional §3.1 dijalankan, **THEN** 100% lulus (sama dengan REQ-09).
- **GIVEN** offline 60 hari, **WHEN** aplikasi dipakai, **THEN** tidak ada fitur lokal yang terkunci.
- **GIVEN** mode self-hosted, **WHEN** kode dijalankan, **THEN** 0 pemanggilan endpoint entitlement.

---

### REQ-34 — Team Vault Collaboration via ECDH *(DISABLED / NEXT FEATURE)*
**Prioritas:** P2

**Deskripsi.** Berbagi kredensial ke anggota tim tanpa server pernah dapat membacanya.

**Aturan bisnis — status di v2.0 GA.**
- Layar Team lengkap secara visual (daftar anggota, peran, vault bersama), seluruhnya non-aktif, badge **"Coming Soon · Team"**.
- Primitif kripto (X25519 ECDH + HKDF-SHA256 + AES-256-GCM) **ditulis dan diuji penuh** di v2.0, termasuk test vector.
- Tabel `team_identity`, `team_members`, `team_vaults`, `team_grants` **dibuat sejak v2.0**.

**Aturan bisnis — desain Phase 2.**
- Setiap pengguna memiliki **identity keypair X25519**; private key disimpan terenkripsi DEK (`team_identity.private_key_enc`).
- Vault bersama punya **Vault Key (VK)** simetris 32 byte acak.
- Memberi akses: `shared_secret = X25519(my_priv, their_pub)` → `wrapping_key = HKDF-SHA256(shared_secret, salt, info="caterm-team-v1")` → `VK` di-*wrap* dengan `wrapping_key` → disimpan sebagai `team_grants.wrapped_vk`. **Server hanya menyimpan hasil wrap dan public key.**
- Mencabut akses: hapus grant **dan** rotasi VK, lalu re-wrap untuk anggota tersisa. Mencabut tanpa rotasi adalah keamanan teater — anggota lama masih punya VK lama.
- **Verifikasi kunci out-of-band wajib:** sebelum berbagi, kedua pihak membandingkan *safety number* (fingerprint public key) lewat kanal terpisah. Tanpa ini, server dapat melakukan MITM dengan menyodorkan public key palsu.
- Peran: `owner` (kelola anggota & rotasi), `editor` (ubah kredensial), `viewer` (pakai untuk konek, tidak dapat mengungkap kredensial di UI — *catatan jujur:* ini mitigasi UX, bukan jaminan kriptografis, karena viewer tetap memegang VK).
- Audit trail keanggotaan (siapa menambah/mencabut siapa, kapan) disimpan lokal dan ditandatangani.

**Kriteria penerimaan (v2.0).**
- **GIVEN** test vector ECDH, **WHEN** dijalankan, **THEN** `shared_secret` cocok dengan nilai referensi RFC 7748.
- **GIVEN** A membagikan vault ke B lalu mencabutnya, **WHEN** simulasi dijalankan, **THEN** VK dirotasi dan grant lama tidak dapat membuka data baru.
- **GIVEN** layar Team, **WHEN** tombol apa pun ditekan, **THEN** 0 permintaan jaringan.

---

## BAGIAN J — RILIS

### REQ-35 — Packaging, Signing, Auto-Update & CI/CD
**Prioritas:** P0

**Deskripsi.** Produk yang tidak bisa dipasang dengan mudah bukan produk.

**Aturan bisnis.**
- Target rilis v2.0: **Linux** (`.deb`, `.rpm`, `.AppImage`, tarball portable), **Windows** (`.msi`, `.exe` portable), **macOS** (`.dmg` universal — Intel + Apple Silicon).
- Reproducible build sejauh mungkin; setiap artefak disertai **SHA-256 checksum** dan **signature** (minisign/GPG) yang dipublikasikan di release page.
- Code signing: Windows Authenticode & macOS notarization **jika sertifikat tersedia**; jika tidak, dokumentasikan langkah bypass Gatekeeper/SmartScreen secara jujur di README, tanpa menyuruh user mematikan proteksi secara permanen.
- **Auto-update opsional dan default MATI.** Jika dinyalakan, update diverifikasi signature-nya sebelum dipasang, dan changelog ditampilkan sebelum apply. Tidak ada update senyap.
- CI/CD GitHub Actions (menutup **T-21**): pada setiap PR jalankan `fmt`, `clippy -D warnings`, `test`, `test --release`, arch-guard, secret-scan, audit dependensi (`cargo audit` + `cargo deny`), dan benchmark budget REQ-02. Pada tag `v*`: build 3 platform, tanda tangani, buat GitHub Release dengan changelog.
- **Higiene repo (menutup T-15, T-16, T-17):** `.gitignore` menapis seluruh binary, artefak build, dan `*.db`. Dilarang path absolut di skrip mana pun. Dilarang commit biner.
- `README.md` sungguhan (menutup **T-18**): nilai jual, screenshot/GIF, instalasi per-platform, arsitektur keamanan, panduan build, kontribusi, lisensi MIT. Bukan template bawaan framework.
- Dokumen keamanan: `SECURITY.md` (cara melaporkan kerentanan, SLA respons) dan `THREAT_MODEL.md` (§8.6).

**Kriteria penerimaan.**
- **GIVEN** tag `v2.0.0` di-*push*, **WHEN** CI selesai, **THEN** 6+ artefak terpublikasi dengan checksum dan signature.
- **GIVEN** VM bersih tiap OS target, **WHEN** paket dipasang, **THEN** aplikasi berjalan tanpa dependensi manual tambahan.
- **GIVEN** `cargo audit`, **WHEN** dijalankan, **THEN** 0 kerentanan level high/critical.
- **GIVEN** `git ls-files`, **WHEN** diperiksa, **THEN** 0 file `.exe`, `.deb`, `.rpm`, `.db`, atau biner > 1 MB.

---

## 5. Model Data — Skema SQLite Lengkap (DDL)

Seluruh DDL di bawah adalah **normatif**. Implementasi yang menyimpang harus memperbarui dokumen ini terlebih dahulu.

**Konvensi mengikat:**
- Semua PK: `TEXT` berisi **ULID** (26 char Crockford base32).
- Semua timestamp: `TEXT` ISO-8601 UTC milidetik (`YYYY-MM-DDTHH:MM:SS.sssZ`).
- Kolom berakhiran `_enc`: **selalu** berformat `c1.<b64url_nopad(nonce)>.<b64url_nopad(ct||tag)>`.
- Semua tabel yang dapat di-sync memiliki `created_at`, `updated_at`, `deleted_at` (NULL = hidup), dan `rev` (counter integer untuk deteksi konflik).
- Boolean disimpan sebagai `INTEGER` 0/1 dengan `CHECK`.

### 5.1 Meta & Infrastruktur

```sql
-- Versi skema; checksum mencegah migrasi yang diam-diam diubah setelah rilis.
CREATE TABLE schema_migrations (
    version      INTEGER PRIMARY KEY,
    name         TEXT    NOT NULL,
    checksum     TEXT    NOT NULL,          -- SHA-256 hex dari SQL migrasi
    applied_at   TEXT    NOT NULL
);

-- Satu baris seumur hidup vault (id = 'singleton').
CREATE TABLE vault_meta (
    id                   TEXT PRIMARY KEY CHECK (id = 'singleton'),
    schema_version       INTEGER NOT NULL,
    kdf_algo             TEXT    NOT NULL DEFAULT 'argon2id',
    kdf_t                INTEGER NOT NULL DEFAULT 3,        -- iterations
    kdf_m_kib            INTEGER NOT NULL DEFAULT 65536,    -- 64 MiB
    kdf_p                INTEGER NOT NULL DEFAULT 4,        -- parallelism
    kdf_salt             BLOB    NOT NULL,                  -- 16 bytes CSPRNG
    wrapped_dek          TEXT    NOT NULL,                  -- c1.<nonce>.<ct> (DEK di-wrap KEK)
    recovery_wrapped_dek TEXT,                              -- NULL = Recovery Kit tidak aktif
    recovery_salt        BLOB,
    cipher_suite         TEXT    NOT NULL DEFAULT 'aes-256-gcm',
    device_id            TEXT    NOT NULL,                  -- ULID, identitas instalasi ini
    device_label         TEXT    NOT NULL DEFAULT 'this-device',
    created_at           TEXT    NOT NULL,
    updated_at           TEXT    NOT NULL
);
-- CATATAN: TIDAK ADA kolom master_hash. Verifikasi password = unwrap wrapped_dek (REQ-05).

-- Key-value bertipe. Seluruh preferensi ada di sini agar ikut backup & sync.
CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    value_type  TEXT NOT NULL CHECK (value_type IN ('string','int','bool','json','enc')),
    updated_at  TEXT NOT NULL
);
-- value_type='enc' berarti value berformat c1.* (mis. key='ai.api_key').
```

```sql
-- REQ-36. Profil penyedia AI. Banyak profil, tepat satu default.
-- api_key_enc & headers_enc terenkripsi DEK, AAD = '<provider_id>|ai_providers|<field>'.
CREATE TABLE ai_providers (
    id                     TEXT PRIMARY KEY,
    label                  TEXT NOT NULL,
    protocol               TEXT NOT NULL
                           CHECK (protocol IN ('openai_chat','anthropic_messages','ollama','custom_openai_compatible')),
    base_url               TEXT NOT NULL,
    model                  TEXT NOT NULL,
    api_key_enc            TEXT,                  -- NULL hanya untuk penyedia lokal tanpa auth
    headers_enc            TEXT,                  -- JSON header tambahan, terenkripsi
    max_tokens             INTEGER NOT NULL DEFAULT 1024 CHECK (max_tokens BETWEEN 16 AND 32768),
    temperature            REAL    NOT NULL DEFAULT 0.2 CHECK (temperature BETWEEN 0.0 AND 2.0),
    timeout_sec            INTEGER NOT NULL DEFAULT 60  CHECK (timeout_sec BETWEEN 5 AND 300),
    context_lines          INTEGER NOT NULL DEFAULT 50  CHECK (context_lines BETWEEN 1 AND 200),
    allow_insecure_http    INTEGER NOT NULL DEFAULT 0 CHECK (allow_insecure_http IN (0,1)),
    is_default             INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0,1)),
    enabled                INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0,1)),
    last_validated_at      TEXT,
    last_validation_status TEXT CHECK (last_validation_status IS NULL
                                       OR last_validation_status IN ('ok','failed')),
    last_validation_code   TEXT,                  -- CAT-AI-0xx (REQ-36)
    created_at             TEXT NOT NULL,
    updated_at             TEXT NOT NULL,
    deleted_at             TEXT,
    rev                    INTEGER NOT NULL DEFAULT 1,
    -- Profil hanya boleh jadi default kalau pernah lulus validasi (REQ-36).
    CHECK (is_default = 0 OR last_validation_status = 'ok'),
    -- Skema http hanya boleh dengan persetujuan risiko eksplisit dari user.
    CHECK (allow_insecure_http = 1 OR base_url LIKE 'https://%')
);
-- Tepat satu default yang hidup.
CREATE UNIQUE INDEX idx_ai_providers_default ON ai_providers(is_default)
    WHERE is_default = 1 AND deleted_at IS NULL;
CREATE INDEX idx_ai_providers_label ON ai_providers(label) WHERE deleted_at IS NULL;
```

> Kolom `settings.ai_api_key_enc` tetap ada untuk kompatibilitas migrasi vault v1 (REQ-03), tetapi **tidak dipakai lagi** oleh jalur produksi v2 — migrasi memindahkannya menjadi satu baris `ai_providers` berstatus `Belum diverifikasi`.

### 5.2 Hosts, Groups, Tags

```sql
CREATE TABLE groups (
    id                  TEXT PRIMARY KEY,
    parent_id           TEXT REFERENCES groups(id) ON DELETE SET NULL,
    label               TEXT    NOT NULL,
    description         TEXT,
    color               TEXT,                       -- hex, mis. '#dc2626'
    icon                TEXT,
    sort_order          INTEGER NOT NULL DEFAULT 0,
    -- Konfigurasi terwaris (REQ-10). NULL pada host = warisi dari sini.
    inherit_username    TEXT,
    inherit_port        INTEGER CHECK (inherit_port IS NULL OR (inherit_port BETWEEN 1 AND 65535)),
    inherit_ssh_key_id  TEXT REFERENCES ssh_keys(id) ON DELETE SET NULL,
    inherit_jump_host_id TEXT REFERENCES hosts(id) ON DELETE SET NULL,
    inherit_env_vars    TEXT,                       -- JSON object
    inherit_keepalive   INTEGER,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT,
    rev                 INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_groups_parent  ON groups(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_groups_updated ON groups(updated_at);

CREATE TABLE hosts (
    id                  TEXT PRIMARY KEY,
    group_id            TEXT REFERENCES groups(id) ON DELETE SET NULL,
    label               TEXT    NOT NULL,           -- TIDAK terenkripsi (index pencarian)
    hostname            TEXT    NOT NULL,           -- TIDAK terenkripsi
    port                INTEGER NOT NULL DEFAULT 22 CHECK (port BETWEEN 1 AND 65535),
    username            TEXT,                       -- NULL = warisi dari group
    auth_method         TEXT    NOT NULL DEFAULT 'password'
                        CHECK (auth_method IN ('password','key','agent','interactive')),
    -- Kredensial: SELALU terenkripsi dengan DEK, AAD = '<host_id>|hosts|<field>'
    password_enc        TEXT,
    ssh_key_id          TEXT REFERENCES ssh_keys(id) ON DELETE SET NULL,
    key_passphrase_enc  TEXT,                       -- override passphrase milik ssh_keys
    private_key_enc     TEXT,                       -- key inline (tidak di keys manager)
    -- Perilaku koneksi
    jump_host_id        TEXT REFERENCES hosts(id) ON DELETE SET NULL,
    keepalive_interval  INTEGER DEFAULT 30,
    compression         INTEGER NOT NULL DEFAULT 0 CHECK (compression IN (0,1)),
    charset             TEXT    NOT NULL DEFAULT 'utf-8',
    env_vars            TEXT,                       -- JSON object
    startup_snippet_id  TEXT REFERENCES snippets(id) ON DELETE SET NULL,
    allow_legacy_algos  INTEGER NOT NULL DEFAULT 0 CHECK (allow_legacy_algos IN (0,1)),
    auto_reconnect      INTEGER NOT NULL DEFAULT 0 CHECK (auto_reconnect IN (0,1)),
    monitoring_enabled  INTEGER NOT NULL DEFAULT 0 CHECK (monitoring_enabled IN (0,1)),
    monitoring_interval INTEGER NOT NULL DEFAULT 5,
    audit_output_enabled INTEGER NOT NULL DEFAULT 0 CHECK (audit_output_enabled IN (0,1)),
    -- Presentasi & statistik
    color               TEXT,
    notes               TEXT,
    sort_order          INTEGER NOT NULL DEFAULT 0,
    connect_count       INTEGER NOT NULL DEFAULT 0,
    last_connected_at   TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT,
    rev                 INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_hosts_group    ON hosts(group_id)  WHERE deleted_at IS NULL;
CREATE INDEX idx_hosts_label    ON hosts(label)     WHERE deleted_at IS NULL;
CREATE INDEX idx_hosts_hostname ON hosts(hostname)  WHERE deleted_at IS NULL;
CREATE INDEX idx_hosts_recent   ON hosts(last_connected_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_hosts_updated  ON hosts(updated_at);

CREATE TABLE tags (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL UNIQUE,
    color       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT,
    rev         INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE host_tags (
    host_id     TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    tag_id      TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    created_at  TEXT NOT NULL,
    PRIMARY KEY (host_id, tag_id)
);
CREATE INDEX idx_host_tags_tag ON host_tags(tag_id);
```

### 5.3 Host Keys (TOFU) & SSH Keys

```sql
-- REQ-18. Identitas entri = (hostname, port, key_type).
CREATE TABLE host_keys (
    id              TEXT PRIMARY KEY,
    hostname        TEXT    NOT NULL,
    port            INTEGER NOT NULL DEFAULT 22,
    key_type        TEXT    NOT NULL,      -- ssh-ed25519, ssh-rsa, ecdsa-sha2-nistp256, ...
    public_key      TEXT    NOT NULL,      -- base64 blob mentah
    fingerprint_sha256 TEXT NOT NULL,      -- SHA256:<base64>
    verified_by     TEXT    NOT NULL DEFAULT 'user'
                    CHECK (verified_by IN ('user','imported','pinned')),
    first_seen_at   TEXT    NOT NULL,
    last_seen_at    TEXT    NOT NULL,
    revoked_at      TEXT,                  -- diisi saat user mencabut kepercayaan
    notes           TEXT,
    created_at      TEXT    NOT NULL,
    updated_at      TEXT    NOT NULL,
    rev             INTEGER NOT NULL DEFAULT 1,
    UNIQUE (hostname, port, key_type)
);
CREATE INDEX idx_host_keys_lookup ON host_keys(hostname, port);

-- REQ-26. Private key SELALU terenkripsi DEK, tidak pernah menyentuh disk sebagai plaintext.
CREATE TABLE ssh_keys (
    id                 TEXT PRIMARY KEY,
    label              TEXT NOT NULL,
    key_type           TEXT NOT NULL CHECK (key_type IN ('ed25519','rsa','ecdsa')),
    bits               INTEGER,
    public_key         TEXT NOT NULL,      -- 'ssh-ed25519 AAAA... comment'
    private_key_enc    TEXT NOT NULL,      -- AAD = '<key_id>|ssh_keys|private_key'
    passphrase_enc     TEXT,               -- NULL = minta setiap koneksi
    fingerprint_sha256 TEXT NOT NULL,
    comment            TEXT,
    source             TEXT NOT NULL DEFAULT 'generated'
                       CHECK (source IN ('generated','imported')),
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    deleted_at         TEXT,
    rev                INTEGER NOT NULL DEFAULT 1
);
CREATE UNIQUE INDEX idx_ssh_keys_fp ON ssh_keys(fingerprint_sha256) WHERE deleted_at IS NULL;
```

### 5.4 Snippets & Port Forwards

```sql
CREATE TABLE snippets (
    id                    TEXT PRIMARY KEY,
    group_id              TEXT REFERENCES groups(id) ON DELETE SET NULL,
    label                 TEXT NOT NULL,
    body                  TEXT NOT NULL,          -- boleh berisi {{var}}
    description           TEXT,
    auto_enter            INTEGER NOT NULL DEFAULT 0 CHECK (auto_enter IN (0,1)),
    requires_confirmation INTEGER NOT NULL DEFAULT 0 CHECK (requires_confirmation IN (0,1)),
    is_dangerous          INTEGER NOT NULL DEFAULT 0 CHECK (is_dangerous IN (0,1)),
    var_defaults          TEXT,                   -- JSON: {"filename":"id_ed25519"}
    use_count             INTEGER NOT NULL DEFAULT 0,
    last_used_at          TEXT,
    sort_order            INTEGER NOT NULL DEFAULT 0,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL,
    deleted_at            TEXT,
    rev                   INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_snippets_label ON snippets(label) WHERE deleted_at IS NULL;

CREATE TABLE snippet_tags (
    snippet_id TEXT NOT NULL REFERENCES snippets(id) ON DELETE CASCADE,
    tag_id     TEXT NOT NULL REFERENCES tags(id)     ON DELETE CASCADE,
    PRIMARY KEY (snippet_id, tag_id)
);

-- REQ-23
CREATE TABLE port_forwards (
    id              TEXT PRIMARY KEY,
    host_id         TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    label           TEXT NOT NULL,
    forward_type    TEXT NOT NULL CHECK (forward_type IN ('local','remote','dynamic')),
    bind_address    TEXT NOT NULL DEFAULT '127.0.0.1',
    bind_port       INTEGER NOT NULL CHECK (bind_port BETWEEN 1 AND 65535),
    dest_host       TEXT,                 -- NULL untuk dynamic/SOCKS5
    dest_port       INTEGER CHECK (dest_port IS NULL OR (dest_port BETWEEN 1 AND 65535)),
    auto_start      INTEGER NOT NULL DEFAULT 0 CHECK (auto_start IN (0,1)),
    -- Bind non-loopback butuh persetujuan eksplisit user (REQ-23).
    allow_public_bind INTEGER NOT NULL DEFAULT 0 CHECK (allow_public_bind IN (0,1)),
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0,1)),
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT,
    rev             INTEGER NOT NULL DEFAULT 1,
    CHECK (forward_type = 'dynamic' OR (dest_host IS NOT NULL AND dest_port IS NOT NULL)),
    CHECK (bind_address = '127.0.0.1' OR bind_address = '::1' OR allow_public_bind = 1)
);
CREATE INDEX idx_port_forwards_host ON port_forwards(host_id) WHERE deleted_at IS NULL;
```

### 5.5 Audit Log, Monitoring, Transfers

```sql
-- REQ-25. command_enc & output_enc terenkripsi DEK. Output dipotong 5 KB SEBELUM dienkripsi.
CREATE TABLE audit_log (
    id            TEXT PRIMARY KEY,
    host_id       TEXT REFERENCES hosts(id) ON DELETE SET NULL,
    host_label    TEXT,                    -- snapshot; host bisa dihapus
    session_id    TEXT NOT NULL,
    command_enc   TEXT NOT NULL,           -- AAD = '<audit_id>|audit_log|command'
    output_enc    TEXT,                    -- NULL kecuali audit_output_enabled
    output_bytes  INTEGER,                 -- ukuran asli sebelum truncate
    truncated     INTEGER NOT NULL DEFAULT 0 CHECK (truncated IN (0,1)),
    redacted      INTEGER NOT NULL DEFAULT 0 CHECK (redacted IN (0,1)),
    exit_code     INTEGER,
    duration_ms   INTEGER,
    executed_at   TEXT NOT NULL,
    created_at    TEXT NOT NULL
);
CREATE INDEX idx_audit_host ON audit_log(host_id, executed_at DESC);
CREATE INDEX idx_audit_time ON audit_log(executed_at DESC);
-- CATATAN: sengaja TIDAK ADA index teks pada perintah — index plaintext akan
-- membocorkan isi yang justru sudah dienkripsi.

-- REQ-24. Retensi rolling; 'raw' 24 jam, 'agg5m' 7 hari.
CREATE TABLE monitor_samples (
    id            TEXT PRIMARY KEY,
    host_id       TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    sampled_at    TEXT NOT NULL,
    resolution    TEXT NOT NULL DEFAULT 'raw' CHECK (resolution IN ('raw','agg5m')),
    cpu_percent   REAL,
    mem_used_kb   INTEGER,
    mem_total_kb  INTEGER,
    swap_used_kb  INTEGER,
    disk_json     TEXT,                    -- JSON array per-mount
    net_rx_bps    INTEGER,
    net_tx_bps    INTEGER,
    load_1        REAL,
    load_5        REAL,
    load_15       REAL,
    uptime_sec    INTEGER,
    process_count INTEGER,
    top_json      TEXT                     -- JSON array top-5 proses
);
CREATE INDEX idx_monitor_host_time ON monitor_samples(host_id, sampled_at DESC);

-- REQ-20. Antrean transfer bertahan lintas restart.
CREATE TABLE sftp_transfers (
    id             TEXT PRIMARY KEY,
    host_id        TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    direction      TEXT NOT NULL CHECK (direction IN ('upload','download')),
    local_path     TEXT NOT NULL,
    remote_path    TEXT NOT NULL,
    total_bytes    INTEGER NOT NULL,
    bytes_done     INTEGER NOT NULL DEFAULT 0,
    source_mtime   TEXT,                   -- validasi sebelum resume
    source_size    INTEGER,
    status         TEXT NOT NULL DEFAULT 'queued'
                   CHECK (status IN ('queued','running','paused','completed','failed','cancelled')),
    error_code     TEXT,
    error_message  TEXT,
    preserve_mtime INTEGER NOT NULL DEFAULT 1 CHECK (preserve_mtime IN (0,1)),
    started_at     TEXT,
    finished_at    TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);
CREATE INDEX idx_transfers_status ON sftp_transfers(status, created_at);
```

### 5.6 Layout Sesi & Statistik

```sql
-- REQ-16. Struktur tab/pane dipulihkan saat start; sesi TIDAK auto-connect.
CREATE TABLE workspace_layout (
    id           TEXT PRIMARY KEY,
    tab_index    INTEGER NOT NULL,
    tab_label    TEXT,
    tab_color    TEXT,
    pane_tree    TEXT NOT NULL,            -- JSON: {"split":"h","children":[...]}
    is_active    INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0,1)),
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);
```

### 5.7 Sync & Team — *dibuat di v2.0, tidak dipakai sampai Phase 2*

```sql
-- REQ-32. Tabel ada sejak v2.0 agar aktivasi Phase 2 tidak butuh migrasi besar.
CREATE TABLE devices (
    id             TEXT PRIMARY KEY,       -- ULID device
    label          TEXT NOT NULL,
    platform       TEXT NOT NULL,
    public_key     TEXT,                   -- X25519 pub device
    last_sync_at   TEXT,
    is_current     INTEGER NOT NULL DEFAULT 0 CHECK (is_current IN (0,1)),
    revoked_at     TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE TABLE sync_state (
    id                 TEXT PRIMARY KEY CHECK (id = 'singleton'),
    enabled            INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0,1)),
    mode               TEXT    NOT NULL DEFAULT 'managed' CHECK (mode IN ('managed','selfhosted')),
    relay_url          TEXT,                -- NULL di v2.0; wajib diisi user di Phase 2
    account_id         TEXT,
    device_token_enc   TEXT,
    last_sync_at       TEXT,
    last_sync_status   TEXT,
    last_cursor        TEXT,                -- posisi perubahan terakhir dari relay
    entitlement_status TEXT NOT NULL DEFAULT 'none'
                       CHECK (entitlement_status IN ('none','active','grace','expired')),
    entitlement_checked_at TEXT,
    entitlement_expires_at TEXT,
    updated_at         TEXT NOT NULL
);

-- Outbox: perubahan lokal menunggu diunggah. Diisi oleh trigger sejak v2.0
-- (murah, dan membuat aktivasi Phase 2 tidak kehilangan riwayat).
CREATE TABLE sync_queue (
    id           TEXT PRIMARY KEY,
    entity       TEXT NOT NULL,             -- 'hosts','groups','snippets',...
    record_id    TEXT NOT NULL,
    operation    TEXT NOT NULL CHECK (operation IN ('upsert','delete')),
    rev          INTEGER NOT NULL,
    queued_at    TEXT NOT NULL,
    attempts     INTEGER NOT NULL DEFAULT 0,
    last_error   TEXT,
    UNIQUE (entity, record_id)
);

-- REQ-34. Identitas ECDH milik user ini.
CREATE TABLE team_identity (
    id              TEXT PRIMARY KEY CHECK (id = 'singleton'),
    public_key      TEXT NOT NULL,          -- X25519 pub, base64
    private_key_enc TEXT NOT NULL,          -- AAD = 'singleton|team_identity|private_key'
    safety_number   TEXT NOT NULL,          -- fingerprint untuk verifikasi out-of-band
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE team_members (
    id             TEXT PRIMARY KEY,
    display_name   TEXT NOT NULL,
    email          TEXT,
    public_key     TEXT NOT NULL,
    safety_number  TEXT NOT NULL,
    verified_at    TEXT,                    -- NULL = belum diverifikasi out-of-band
    role           TEXT NOT NULL DEFAULT 'viewer' CHECK (role IN ('owner','editor','viewer')),
    invited_at     TEXT,
    joined_at      TEXT,
    revoked_at     TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE TABLE team_vaults (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    owner_id    TEXT NOT NULL REFERENCES team_members(id),
    vk_version  INTEGER NOT NULL DEFAULT 1, -- naik setiap rotasi (pencabutan akses)
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE team_grants (
    id            TEXT PRIMARY KEY,
    vault_id      TEXT NOT NULL REFERENCES team_vaults(id)  ON DELETE CASCADE,
    member_id     TEXT NOT NULL REFERENCES team_members(id) ON DELETE CASCADE,
    vk_version    INTEGER NOT NULL,
    wrapped_vk    TEXT NOT NULL,            -- VK di-wrap dgn HKDF(ECDH(me, member))
    grant_nonce   BLOB NOT NULL,
    granted_by    TEXT NOT NULL,
    granted_at    TEXT NOT NULL,
    revoked_at    TEXT,
    UNIQUE (vault_id, member_id, vk_version)
);

CREATE TABLE team_vault_hosts (
    vault_id  TEXT NOT NULL REFERENCES team_vaults(id) ON DELETE CASCADE,
    host_id   TEXT NOT NULL REFERENCES hosts(id)       ON DELETE CASCADE,
    PRIMARY KEY (vault_id, host_id)
);
```

### 5.8 Aturan Integritas

1. `FOREIGN KEYS = ON` pada setiap koneksi, tanpa kecuali.
2. Seluruh operasi multi-tabel berjalan dalam transaksi. Kegagalan di tengah wajib *rollback* utuh.
3. `updated_at` dan `rev` diperbarui oleh **trigger**, bukan oleh kode aplikasi — kode aplikasi lupa, trigger tidak.
4. Soft delete: query aplikasi **selalu** menyertakan `WHERE deleted_at IS NULL`. Disediakan view `v_hosts_live`, `v_groups_live`, dst.
5. Purge hard-delete berjalan saat start untuk record dengan `deleted_at` lebih tua dari 30 hari.
6. `VACUUM` ditawarkan manual di Settings → Advanced (bukan otomatis — ia menulis ulang seluruh file dan mahal pada DB besar).
7. Setiap tabel yang dapat di-sync memiliki trigger `AFTER INSERT/UPDATE/DELETE` yang menulis ke `sync_queue`. Trigger ini aktif **sejak v2.0** meskipun sync mati; biayanya sangat kecil dan membuat aktivasi Phase 2 tidak kehilangan riwayat perubahan.

---

## 6. Permukaan IPC (Tauri Commands & Events)

Kontrak antara frontend Svelte dan Rust core. **Setiap command adalah pembungkus tipis** (REQ-01).

### 6.1 Commands (frontend → Rust)

| Domain | Command | Ringkasan |
|---|---|---|
| Vault | `vault_status`, `vault_init`, `vault_unlock`, `vault_lock`, `vault_change_password`, `vault_recovery_setup`, `vault_recovery_unlock`, `vault_reset` | REQ-05, 07, 08 |
| Host | `host_list`, `host_get`, `host_create`, `host_update`, `host_delete`, `host_restore`, `host_clone`, `host_import_ssh_config`, `host_resolve_config` | REQ-10, 11 |
| Group | `group_list`, `group_create`, `group_update`, `group_delete`, `group_move` | REQ-10 |
| Tag | `tag_list`, `tag_create`, `tag_assign`, `tag_unassign` | REQ-11 |
| Search | `search_global` | REQ-12 |
| Session | `session_open`, `session_close`, `session_write`, `session_ack`, `session_resize`, `session_list`, `session_reconnect` | REQ-13–16 |
| TOFU | `hostkey_pending_get`, `hostkey_accept`, `hostkey_reject`, `hostkey_list`, `hostkey_revoke`, `hostkey_import_known_hosts` | REQ-18 |
| SFTP | `sftp_open`, `sftp_list`, `sftp_mkdir`, `sftp_rename`, `sftp_remove`, `sftp_stat`, `sftp_chmod`, `sftp_chown`, `sftp_read_text`, `sftp_write_text` | REQ-19, 21 |
| Transfer | `transfer_enqueue`, `transfer_list`, `transfer_cancel`, `transfer_pause`, `transfer_resume`, `transfer_clear_finished` | REQ-20 |
| Snippet | `snippet_list`, `snippet_create`, `snippet_update`, `snippet_delete`, `snippet_render`, `snippet_execute` | REQ-22 |
| Tunnel | `tunnel_list`, `tunnel_create`, `tunnel_update`, `tunnel_delete`, `tunnel_start`, `tunnel_stop`, `tunnel_status` | REQ-23 |
| Monitor | `monitor_start`, `monitor_stop`, `monitor_latest`, `monitor_history` | REQ-24 |
| Audit | `audit_list`, `audit_search`, `audit_export`, `audit_purge` | REQ-25 |
| Keys | `key_list`, `key_generate`, `key_import`, `key_export`, `key_delete`, `key_deploy` | REQ-26, 27 |
| AI | `ai_config_get`, `ai_config_set`, `ai_consent_accept`, `ai_preview_payload`, `ai_ask`, `ai_cancel` | REQ-28, 29 |
| AI Provider | `ai_provider_list`, `ai_provider_create`, `ai_provider_update`, `ai_provider_delete`, `ai_provider_set_default`, `ai_provider_validate` (lapis 1, tanpa jaringan), `ai_provider_test` (lapis 2–3), `ai_models_list` | REQ-36 |
| Settings | `settings_get_all`, `settings_set`, `settings_reset`, `keybindings_get`, `keybindings_set`, `theme_import` | REQ-30 |
| Backup | `backup_export`, `backup_preview`, `backup_restore`, `backup_schedule_set` | REQ-31 |
| Sync *(disabled)* | `sync_status` → selalu `{enabled:false, reason:"coming_soon"}` | REQ-32 |
| Team *(disabled)* | `team_status` → selalu `{enabled:false, reason:"coming_soon"}` | REQ-34 |
| System | `app_metrics`, `app_version`, `app_open_external`, `app_data_dir` | REQ-02, 17 |

### 6.2 Events (Rust → frontend)

| Event | Payload | Catatan |
|---|---|---|
| `pty://output` | `{session_id, seq, data_b64}` | Ter-*batch* 16 ms / 4 KB (REQ-14) |
| `pty://exit` | `{session_id, code, reason}` | |
| `pty://state` | `{session_id, state}` | `connecting`/`authenticating`/`ready`/`closed` |
| `hostkey://prompt` | `{request_id, hostname, port, key_type, fingerprint, randomart, is_changed}` | Menahan koneksi (REQ-18) |
| `transfer://progress` | `{transfer_id, bytes_done, total, bps, eta_sec}` | Throttle 250 ms |
| `transfer://done` | `{transfer_id, status, error_code?}` | |
| `tunnel://state` | `{tunnel_id, state, active_conns, bytes}` | |
| `monitor://sample` | `{host_id, sample}` | |
| `vault://locked` | `{reason}` | `idle`/`manual`/`suspend` |
| `app://toast` | `{level, code, message}` | Kode error stabil (REQ-04) |

**Aturan mengikat:** setiap event yang membawa data dari server remote (`pty://output`, `sftp` listing, `monitor://sample`) diperlakukan frontend sebagai **data tidak tepercaya**. Dilarang `innerHTML`, dilarang mengeksekusi apa pun dari isinya, dan escape sequence disanitasi sebelum dirender di luar kanvas xterm.js (REQ-21).

---

## 7. Antarmuka & Alur Layar

### 7.1 Peta layar

```
┌─ Unlock ──────────────────────────────────────────────┐
│  Master Password · Recovery Kit · (Setup pertama kali)  │
└───────────────────────┬───────────────────────────────┘
                        ▼
┌─ Main Window ─────────────────────────────────────────┐
│ Sidebar        │  Workspace                            │
│  Hosts (tree)  │  ┌ Tab 1 ─ Tab 2 ─ + ───────────────┐ │
│  Snippets      │  │ ┌───────────┬───────────┐        │ │
│  Keys          │  │ │  pane A   │  pane B   │        │ │
│  Tunnels       │  │ │ (xterm)   │ (xterm)   │        │ │
│  Monitoring    │  │ └───────────┴───────────┘        │ │
│  Audit         │  │ ▸ SFTP drawer (toggle)           │ │
│  ── Next ──    │  └──────────────────────────────────┘ │
│  Sync 🔒       │  Status bar: RSS · sesi · transfer    │
│  Team 🔒       │                                       │
│  Settings      │                                       │
└───────────────────────────────────────────────────────┘
```

### 7.2 Alur kritis

**Koneksi pertama ke host baru (REQ-13 + REQ-18):**
```
Pilih host → TCP connect → SSH handshake → server sajikan host key
   → [belum dikenal] → tahan koneksi → dialog fingerprint + randomart
       → user Terima → simpan host_keys → LANJUT auth → PTY siap
       → user Tolak  → tutup koneksi (0 byte kredensial terkirim)
   → [dikenal & cocok] → langsung auth
   → [dikenal & BERBEDA] → BLOKIR. Layar merah. Tidak ada tombol lanjut.
```

**Pemakaian AI pertama kali (REQ-28 + REQ-29):**
```
Klik Copilot → layar consent (data apa, ke mana, oleh siapa)
  → centang "Saya mengerti" → isi base_url + model + api_key (tidak ada default)
  → user pilih teks → Explain → Redaction Guard → PRATINJAU PAYLOAD (wajib, 3× pertama)
  → Kirim → hasil ditampilkan → (Generate) masuk buffer input, TIDAK dieksekusi
```

**Klik Sync / Team di v2.0 (REQ-32, REQ-34):**
```
Klik → layar lengkap dengan badge "Coming Soon · Managed Sync"
  → penjelasan model harga & arsitektur zero-knowledge
  → tombol "Beri tahu saya saat rilis" → membuka halaman web di browser eksternal
  → 0 permintaan jaringan dari dalam aplikasi
```

### 7.3 Aksesibilitas & i18n

- Navigasi keyboard penuh; seluruh aksi punya jalur non-mouse.
- Kontras minimum WCAG AA pada teks UI (kanvas terminal mengikuti tema pilihan user).
- `prefers-reduced-motion` dihormati.
- String UI terpusat di file locale. Rilis v2.0: **Bahasa Indonesia + English**. Struktur siap bahasa lain.

---

## 8. Spesifikasi Keamanan

### 8.1 Hierarki kunci

```
                 ┌──────────────────┐
  Master Password│  (tidak disimpan) │
                 └────────┬─────────┘
                          │ Argon2id(t=3, m=64MiB, p=4) + salt(16B)
                          ▼
                    KEK (32B) ── hanya di memori, zeroized saat lock
                          │ AES-256-GCM
                          ▼
                 wrapped_dek (disimpan di vault_meta)
                          │ unwrap
                          ▼
                    DEK (32B) ── hanya di memori
                          │ AES-256-GCM + AAD
                          ▼
              ciphertext field (disimpan & di-sync)

  Recovery Code (24 kata) ──Argon2id──▶ RK ──▶ recovery_wrapped_dek  [opsional]
  ECDH X25519 (team)      ──HKDF────▶ wrapping_key ──▶ wrapped_vk    [Phase 2]
```

### 8.2 Format & parameter

| Item | Nilai |
|---|---|
| KDF | Argon2id, t=3, m=65536 KiB, p=4, output 32 B |
| Salt | 16 B CSPRNG, per-vault, disimpan |
| Cipher | AES-256-GCM |
| Nonce | 12 B CSPRNG, unik per operasi |
| Tag | 16 B (bagian dari ciphertext) |
| Format field | `c1.<b64url_nopad(nonce)>.<b64url_nopad(ct‖tag)>` |
| AAD | `"<record_id>\|<table>\|<field>"` UTF-8 |
| Fingerprint host key | `SHA256:<base64_nopad>` |
| Team ECDH | X25519 (RFC 7748) + HKDF-SHA256, info `"caterm-team-v1"` |
| RNG | OS CSPRNG (`getrandom`), tidak pernah PRNG userspace |

### 8.3 Konsekuensi yang wajib dinyatakan ke user

Ditampilkan di Settings → Security dan di dokumentasi, dengan bahasa yang jelas, bukan disclaimer kecil:

1. **Metadata tidak terenkripsi di disk.** `label`, `hostname`, `port`, `username`, dan tag disimpan polos agar pencarian instan. Siapa pun yang punya akses baca ke file DB dapat melihat **daftar server Anda**, meskipun tidak dapat membuka kredensialnya. *(Saat sync diaktifkan kelak, metadata ini dienkripsi sebelum diunggah — REQ-32.)*
2. **Lupa Master Password = data hilang permanen**, kecuali Recovery Kit dibuat sebelumnya. Tidak ada backdoor. Tidak ada reset.
3. **Saat vault terbuka, DEK ada di RAM.** Penyerang dengan akses root ke mesin yang sedang menyala dapat mengambilnya. Auto-lock mempersempit jendela ini; ia tidak menutupnya.
4. **Enkripsi disk penuh tetap dianjurkan.** CATerm melindungi isi vault, bukan seluruh sistem Anda.
5. **Ekspor mode plaintext-metadata** memperlihatkan struktur vault. Gunakan hanya untuk audit.

### 8.4 Aturan penanganan rahasia (mengikat kode)

| Aturan | Penegakan |
|---|---|
| Rahasia dibungkus `Secret<T>`, `Debug` = `[REDACTED]` | Test `test_no_secret_in_debug_output` |
| Tidak ada rahasia di log level mana pun | Secret-scan CI + test redaksi |
| Tidak ada rahasia di argumen CLI (terlihat di `ps`) | Review + test |
| Tidak ada rahasia di file sementara | Scan disk di `T2-REL-05` |
| Tidak ada rahasia di pesan error yang ditampilkan | Review |
| Clipboard berisi rahasia auto-clear setelah 30 detik | REQ-30 |
| DEK/KEK di-*zeroize* saat lock & saat drop | Test memory scan |
| Core dump nonaktif di rilis | `RLIMIT_CORE=0` |

### 8.5 Supply chain

- `cargo-deny`: lisensi allowlist, ban crate yang di-*yank*, ban duplikat versi mayor.
- `cargo-audit` di setiap PR dan setiap malam.
- Dependensi kriptografi dibatasi pada crate RustCrypto / `ring` yang sudah diaudit publik. **Dilarang menulis primitif kripto sendiri.**
- `Cargo.lock` di-*commit*. Pembaruan dependensi lewat PR terpisah yang dapat direview.
- Frontend: `npm ci` dari lockfile, `npm audit` di CI, dan minimalisasi dependensi — setiap paket baru harus dibenarkan.

### 8.6 Threat model ringkas

| Penyerang | Kemampuan | Yang dilindungi | Yang **tidak** dilindungi |
|---|---|---|---|
| Pencuri laptop (mati) | Baca disk | Kredensial (terenkripsi) | Daftar host (metadata polos) |
| Malware userspace (vault terkunci) | Baca file | Kredensial | Metadata |
| Malware userspace (vault terbuka) | Baca memori | — | **Semuanya.** Dinyatakan jujur. |
| MITM jaringan | Intersep TCP | Kredensial (TOFU memblokir) | — |
| Operator relay sync (Phase 2) | Baca seluruh blob | Semua isi (zero-knowledge) | Ukuran & waktu perubahan |
| Server remote jahat | Kirim output apa pun | UI (escape disanitasi), path (validasi) | — |
| Insider Fathforce | Akses infrastruktur | Semua isi (tidak ada kunci di server) | Metadata langganan |

---

## 9. Arsitektur Phase 2 — Sync Cloud & Team

> Seluruh bagian ini **DISABLED di v2.0**. Didokumentasikan sekarang agar skema, tipe data, dan UI v2.0 sudah kompatibel, sehingga aktivasinya tidak menuntut rewrite.

### 9.1 Topologi

```
  Device A                    Relay (FathCloud / self-host)            Device B
 ┌─────────┐                  ┌────────────────────────┐             ┌─────────┐
 │ vault   │  HTTPS + token   │  Zero-Knowledge Store  │             │ vault   │
 │ DEK 🔑  │ ───blob────────▶ │  - record_id           │ ◀───blob─── │ DEK 🔑  │
 │         │ ◀──blob───────── │  - updated_at          │ ────────▶   │         │
 └─────────┘                  │  - ciphertext (opaque) │             └─────────┘
                              │  ❌ TIDAK ADA kunci    │
                              └────────────────────────┘
```

### 9.2 Yang dilihat dan tidak dilihat server

| Server **melihat** | Server **tidak pernah melihat** |
|---|---|
| ID akun & ID perangkat | Master Password |
| `record_id` (ULID acak) | DEK, KEK, VK |
| `entity` (tipe: host/snippet/…) | Hostname, label, username, port |
| `updated_at`, `rev`, ukuran blob | Password, private key, passphrase |
| Alamat IP klien, waktu request | Isi snippet, isi audit log, konfigurasi AI |

### 9.3 Protokol sync (Phase 2)

1. **Push:** klien mengambil batch dari `sync_queue` → untuk setiap record, serialisasi baris **penuh** (termasuk metadata) → enkripsi AES-256-GCM dengan DEK, AAD = `"<record_id>|<entity>|sync"` → `POST /v1/records` berisi `{record_id, entity, rev, updated_at, device_id, blob}`.
2. **Pull:** `GET /v1/changes?since=<cursor>` → daftar perubahan → dekripsi lokal → resolusi konflik → terapkan.
3. **Resolusi konflik:** **LWW-Element-Set CRDT** per-record. Pemenang = `updated_at` terbesar; seri diputuskan oleh `device_id` yang lebih besar secara leksikografis (deterministik di semua perangkat). Penghapusan = tombstone dengan `deleted_at`, bukan absennya record.
4. **Absennya record ≠ penghapusan.** Melindungi dari sinkronisasi parsial atau gagal separuh.
5. **Konflik yang terlihat user** (dua perubahan pada record yang sama dalam jendela 5 menit) memunculkan UI perbandingan. Tidak ada overwrite senyap.
6. **Backup wajib sebelum apply pertama** di perangkat baru: `.db` → `.db.bak`.
7. Bootstrap perangkat kedua: unduh `vault_meta` (`kdf_salt`, `wrapped_dek`, parameter KDF) → minta Master Password → unwrap DEK → tarik seluruh record. *(Tanpa langkah ini, sync lintas mesin mustahil — pelajaran dari desain v1.)*

### 9.4 Self-hosted relay

- Repo terpisah `caterm-sync-relay`, lisensi MIT, satu image Docker, target < 30 MB.
- Stack: Go atau Rust (Axum) + PostgreSQL atau SQLite + S3-compatible opsional untuk blob besar.
- Deploy di **Coolify / Hostinger VPS** (infrastruktur FathCloud) untuk Managed Cloud; komunitas men-*deploy* sendiri dengan `docker compose up`.
- **Tidak ada pengecekan entitlement pada mode self-hosted.** Nol.
- Endpoint: `POST /v1/auth/device`, `POST /v1/records`, `GET /v1/changes`, `DELETE /v1/records/:id`, `GET /v1/health`.
- Kuota Managed Cloud: 100 MB per akun, 10 perangkat (angka awal, dapat direvisi; **tidak** mempengaruhi data lokal saat terlampaui — hanya unggahan yang berhenti).

### 9.5 Team sharing (Phase 2)

```
  Alice ingin berbagi vault "Client-X" ke Bob
  1. Alice & Bob bertukar public key X25519 lewat relay
  2. WAJIB: bandingkan safety number lewat kanal terpisah (telepon/WhatsApp/tatap muka)
  3. shared   = X25519(alice_priv, bob_pub)
  4. wrapkey  = HKDF-SHA256(shared, salt, info="caterm-team-v1")
  5. wrapped  = AES-256-GCM(wrapkey, VK)
  6. Alice unggah {vault_id, member_id: bob, vk_version, wrapped, nonce}
  7. Bob: shared = X25519(bob_priv, alice_pub) → VK → buka kredensial vault
  Pencabutan: hapus grant + ROTASI VK (vk_version+1) + re-wrap ke anggota tersisa.
```

**Kejujuran yang wajib ada di UI:** anggota yang pernah punya akses sudah pernah melihat kredensial. Rotasi VK melindungi data **ke depan**, bukan ke belakang. UI harus menyarankan **mengganti password/kunci di server** setelah mencabut akses seseorang.

---

## 10. Monetisasi & Model Bisnis

### 10.1 Struktur

| Tier | Harga | Isi |
|---|---|---|
| **Core Desktop** | **Gratis, MIT, selamanya** | Seluruh fitur §3.1. Tanpa batas host, tanpa batas key, tanpa iklan, tanpa telemetri, tanpa akun. |
| **Sync Cloud — bulan ke-1** | **$1** | Onboarding & verifikasi kartu. Menyaring pengguna serius, menutup biaya payment gateway. |
| **Sync Cloud — bulan ke-2 dst** | **$3 / bulan** | Sync E2EE multi-perangkat, 10 perangkat, 100 MB. |
| **Team Seat** | **+$1 / bulan / anggota** | Team Vault via ECDH, audit keanggotaan. |
| **Self-Hosted Relay** | **Gratis** | Docker image, tanpa batas, tanpa akun, tanpa entitlement check. |

**Pembanding:** Termius Pro ~$10+/bulan dengan data lokal yang dikunci saat langganan berakhir.

### 10.2 Prinsip yang tidak dapat dilanggar

1. **Yang dijual adalah kenyamanan sinkronisasi, bukan akses ke data sendiri.** (REQ-09)
2. Fitur lokal **tidak akan pernah** dipindahkan ke balik paywall di versi mana pun.
3. Tidak ada iklan, tidak ada telemetri, tidak ada penjualan data — tidak ada data untuk dijual.
4. Self-hosting selalu tersedia dan gratis, dengan dokumentasi yang sama baiknya dengan Managed Cloud.
5. Jika layanan cloud ditutup, pengumuman ≥ 90 hari sebelumnya + alat ekspor penuh + rilis kode server.

### 10.3 Go-to-market

| Kanal | Pesan |
|---|---|
| GitHub | README dengan GIF terminal, arsitektur keamanan, panduan build |
| Product Hunt | "The Open Source, E2EE Alternative to Termius" |
| Show HN | "CATerm — a zero-knowledge SSH manager in Rust that uses 35MB of RAM" |
| Reddit | r/devops, r/selfhosted, r/linux, r/sysadmin, r/rust — tulisan teknis, bukan iklan |
| Narasi inti | *"Stop trusting your production SSH keys to proprietary cloud syncs. Own your vault locally. Sync securely."* |

### 10.4 Metrik keberhasilan v2.0 (6 bulan pasca-GA)

| Metrik | Target | Cara ukur |
|---|---|---|
| GitHub stars | 1.000 | GitHub API |
| Unduhan rilis | 5.000 | GitHub release stats |
| RSS median terlaporkan | ≤ 35 MB | Laporan komunitas + benchmark CI |
| Crash report | < 0,1% sesi | Laporan lokal yang **dikirim manual** oleh user |
| Konversi Sync (saat Phase 2 rilis) | 3% dari pengguna aktif | Billing |
| Isu keamanan kritis | 0 | `SECURITY.md` tracker |

---

## 11. Kebutuhan Non-Fungsional

| Kode | Kategori | Kebutuhan | Verifikasi |
|---|---|---|---|
| NFR-01 | Performa | Lihat tabel REQ-02 | Benchmark CI |
| NFR-02 | Stabilitas | 0 panic; soak 8 jam; drift RSS ≤ 5% | `T2-REL-02` |
| NFR-03 | Keamanan | Seluruh aturan §8 | Audit + test |
| NFR-04 | Portabilitas | Linux (glibc 2.31+), Windows 10 1809+, macOS 12+ | Matriks CI |
| NFR-05 | Offline | 100% fitur lokal tanpa jaringan | REQ-09 |
| NFR-06 | Aksesibilitas | Navigasi keyboard penuh, kontras AA | Audit manual |
| NFR-07 | Observabilitas | Log lokal berotasi (10 MB × 5), tanpa rahasia | Test redaksi |
| NFR-08 | Maintainability | Coverage `caterm-core` ≥ 80%, modul kripto ≥ 95% | `cargo llvm-cov` |
| NFR-09 | Lokalisasi | ID + EN, struktur siap bahasa lain | Review |
| NFR-10 | Integritas data | 0 kehilangan data pada crash (WAL + transaksi) | Test kill -9 |
| NFR-11 | Startup | ≤ 800 ms ke jendela unlock | Benchmark |
| NFR-12 | Ukuran | Binary ≤ 8 MB, installer ≤ 15 MB | CI gate |

---

## 12. Daftar Anti-Regresi (dari `audit-ulang.md`)

Ke-21 temuan audit v1 berubah menjadi **test wajib** di v2. Setiap baris di bawah harus punya test otomatis yang akan **merah** jika masalahnya kembali. `task-v2.md` memetakan masing-masing ke task konkret.

| Kode | Masalah v1 | Dimatikan oleh | Bentuk test anti-regresi |
|---|---|---|---|
| T-01 | TOFU store `nil` → MITM + panic | REQ-18 | `test_tofu_store_is_wired` — periksa verifier terpasang di jalur produksi |
| T-02 | Test redaksi log berbadan kosong | REQ-04 | `test_no_secret_in_debug_output` dengan assertion nyata |
| T-03 | Split pane hanya teks hiasan | REQ-16 | `test_split_pane_opens_real_channel` — hitung channel SSH |
| T-04 | SFTP UI tanpa backend | REQ-19 | `test_sftp_list_calls_subsystem` |
| T-05 | Auto-lock tidak terpasang di startup | REQ-07 | `test_idle_timer_registered_on_boot` |
| T-06 | Auto-export tidak pernah dipicu dari app | REQ-31 | `test_backup_schedule_fires` |
| T-07 | Build gagal di clean checkout | REQ-35 | Job CI `clean-clone-build` |
| T-08 | Endpoint AI hardcoded ke IP privat | REQ-28 | `test_no_hardcoded_ai_endpoint` |
| T-09 | `ai_api_key` plaintext di DB | REQ-28 | `test_ai_key_encrypted_at_rest` |
| T-10 | `ResetVault` tanpa verifikasi password | REQ-08 | `test_reset_requires_password` |
| T-11 | Passphrase key tidak terhubung ke dialer | REQ-26 | `test_connect_with_passphrase_key` |
| T-12 | Migrasi tidak berversi | REQ-03 | `test_migration_versioned_and_checksummed` |
| T-13 | Skema DB ≠ PRD | REQ-03 | `test_schema_matches_spec` (bandingkan dengan DDL §5) |
| T-14 | PTY emit per byte | REQ-14 | `test_output_batched_16ms_or_4kb` |
| T-15 | Biner ter-commit ke Git | REQ-35 | Job CI `repo-hygiene` |
| T-16 | Direktori generated bersarang ter-commit | REQ-35 | Job CI `repo-hygiene` |
| T-17 | Path absolut di skrip build | REQ-35 | Job CI `no-absolute-paths` |
| T-18 | README template bawaan | REQ-35 | Checklist rilis + test keyword |
| T-19 | Binding demo `Greet` ter-*expose* | REQ-01 | `test_no_demo_commands_exposed` |
| T-20 | Audit log tanpa batas & tanpa enkripsi | REQ-25 | `test_audit_encrypted_and_truncated` |
| T-21 | Tidak ada CI | REQ-35 | Keberadaan workflow + status hijau |

---

## 13. Matriks Keterlacakan REQ → Fase

| REQ | Judul | Prioritas | Fase di `task-v2.md` |
|---|---|---|---|
| REQ-01 | Arsitektur Tauri + Rust Core | P0 | Fase 0 |
| REQ-02 | Resource Budget | P0 | Fase 0, 8 |
| REQ-03 | Storage SQLite + Migrasi | P0 | Fase 1 |
| REQ-04 | Zero-Panic & Error Taxonomy | P0 | Fase 0 |
| REQ-05 | Master Password & KEK/DEK | P0 | Fase 1 |
| REQ-06 | Field Encryption + AAD | P0 | Fase 1 |
| REQ-07 | Auto-Lock & Zeroization | P0 | Fase 1 |
| REQ-08 | Ganti Password & Recovery Kit | P1 | Fase 1 |
| REQ-09 | Local Data Sovereignty | P0 | Fase 0, 7, 8 |
| REQ-10 | Groups & Inheritance | P1 | Fase 3 |
| REQ-11 | Hosts CRUD | P0 | Fase 3 |
| REQ-12 | Search & Command Palette | P1 | Fase 3 |
| REQ-13 | Koneksi SSH russh | P0 | Fase 2 |
| REQ-14 | PTY Stream & Flow Control | P0 | Fase 2 |
| REQ-15 | Resize | P0 | Fase 2 |
| REQ-16 | Tabs & Split Pane | P1 | Fase 3 |
| REQ-17 | Rendering & Paste Safety | P1 | Fase 3 |
| REQ-18 | TOFU | P0 | Fase 2 |
| REQ-19 | SFTP Explorer | P1 | Fase 4 |
| REQ-20 | Transfer Engine | P1 | Fase 4 |
| REQ-21 | Permission & Path Safety | P1 | Fase 4 |
| REQ-22 | Snippets | P1 | Fase 5 |
| REQ-23 | Port Forwarding | P1 | Fase 5 |
| REQ-24 | Monitoring | P2 | Fase 5 |
| REQ-25 | Audit Logs | P1 | Fase 5 |
| REQ-26 | SSH Keys Manager | P1 | Fase 5 |
| REQ-27 | Deploy Public Key | P2 | Fase 5 |
| REQ-28 | AI Copilot | P2 | Fase 6 |
| REQ-29 | AI Redaction Guard | P0 | Fase 6 |
| REQ-30 | Settings & Keybindings | P1 | Fase 3, 7 |
| REQ-31 | Backup & Restore | P0 | Fase 7 |
| REQ-32 | Sync (disabled) | P1/P2 | Fase 7 |
| REQ-33 | Entitlement (disabled) | P2 | Fase 7 |
| REQ-34 | Team Vault (disabled) | P2 | Fase 7 |
| REQ-35 | Packaging & CI/CD | P0 | Fase 0, 8 |
| REQ-36 | Provider AI Kustom & Validasi | P1 | Fase 6 |

---

## 14. Risiko

| # | Risiko | Dampak | Probabilitas | Mitigasi |
|---|---|---|---|---|
| R-01 | Target 35 MB RSS tidak tercapai karena WebView | Pilar #1 gugur | Sedang | Ukur dari Fase 0, bukan Fase 8. Budget gate di CI sejak task pertama. Rencana cadangan: turunkan klaim ke "< 60 MB, 10× lebih ringan dari Electron" — jujur, tetap kuat. |
| R-02 | `russh` kurang matang dibanding `golang.org/x/crypto/ssh` | Jadwal molor, bug protokol | Sedang | Spike Fase 2 lebih awal. Matriks uji lawan OpenSSH 7–9, Dropbear, dan appliance. Jika gagal fatal: pertimbangkan `libssh2` binding. |
| R-03 | Rewrite tidak pernah selesai; v1 mati, v2 belum lahir | Produk hilang dari peredaran | **Tinggi** | v1 tetap dirilis & di-*maintain* untuk perbaikan keamanan sampai v2 GA. Rewrite dikerjakan di repo/branch terpisah. |
| R-04 | Migrasi vault v1 → v2 merusak data user | Kehilangan kepercayaan permanen | Rendah | Format ciphertext identik. Migrasi read-only + tulis ke file baru. Backup otomatis. Test dengan 10 vault v1 sintetis. |
| R-05 | Cakupan 36 REQ terlalu besar untuk satu orang | Burnout, kualitas turun | **Tinggi** | Fase 0–3 = MVP yang dapat dirilis sendiri. Fase 4–6 boleh menyusul di v2.1. Gerbang fase mencegah kerja setengah jadi menumpuk. |
| R-06 | Sertifikat code signing tidak tersedia | Friksi instalasi Windows/macOS | Tinggi | Rilis tetap jalan tanpa signing + dokumentasi jujur. Anggarkan sertifikat setelah ada traksi. |
| R-07 | Backend Sync Cloud tidak pernah dibangun | Pilar #5 jadi janji kosong | Sedang | Karena itu ia ter-*ship* **DISABLED** dan **tidak pernah** dijanjikan dengan tanggal. Fitur lokal berdiri sendiri tanpa cloud. |
| R-08 | Serangan pada UI-placeholder Sync/Team dianggap "fitur bohong" | Kritik komunitas | Rendah | Badge "Coming Soon" eksplisit, tanpa kolom pembayaran, tanpa janji tanggal, dan kode kriptonya terbuka untuk diaudit. |
| R-09 | Kepatuhan ekspor kriptografi (BIS/EAR) | Hambatan distribusi | Rendah | Open source + kripto standar → pengecualian TSU. Dokumentasikan di `SECURITY.md`. |
| R-10 | Perbedaan perilaku WebView antar-OS | Bug hanya di satu platform | Sedang | Matriks CI 3 OS sejak Fase 0, bukan menjelang rilis. |

---

## 15. Asumsi

1. Pengembangan dilakukan oleh **satu orang** (Cecep Azhar) secara paruh waktu, di sela pekerjaan tetap di PT Gerlink Utama Mandiri. Estimasi fase menganggap ~15 jam/minggu.
2. Mesin pengembangan utama: **Linux (CachyOS/Fedora)**; macOS diuji di CI; Windows diuji di VM atau CI.
3. Tidak ada anggaran untuk audit keamanan pihak ketiga di v2.0. Audit komunitas lewat open source adalah gantinya, dan dinyatakan apa adanya.
4. `russh` versi stabil tersedia dan dipelihara selama masa proyek.
5. Tauri 2.0 stabil pada ketiga platform target.
6. Backend Sync Cloud **tidak** dikerjakan bersamaan dengan v2.0. Ia proyek berikutnya, repo terpisah.
7. Pengguna sasaran: DevOps engineer, sysadmin, dan developer yang mengelola 10–200 server dan sudah paham konsep SSH key.
8. Tidak ada kewajiban kompatibilitas dengan format file klien SSH lain selain `~/.ssh/config` dan `~/.ssh/known_hosts`.

---

## 16. Pertanyaan Terbuka

Daftar ini **sengaja tidak kosong**. "Tidak ada pertanyaan terbuka" pada dokumen sebesar ini adalah sinyal bahaya, bukan tanda kematangan.

| # | Pertanyaan | Dampak jika salah | Perlu diputuskan sebelum |
|---|---|---|---|
| Q-01 | Apakah target 35 MB RSS realistis dengan WebKitGTK di Linux? Perlu diukur dengan prototipe Fase 0, bukan diasumsikan. | Klaim pemasaran utama | Fase 0 selesai |
| Q-02 | Apakah metadata (`hostname`, `label`) perlu dienkripsi juga di disk lokal, dengan biaya kehilangan pencarian SQL instan? | Klaim privasi vs UX | Fase 1 |
| Q-03 | Apakah Recovery Kit 24 kata terlalu rumit untuk pengguna sasaran? Alternatif: file recovery terenkripsi. | Kehilangan data pengguna | Fase 1 |
| Q-04 | Batas 4 pane per tab — terlalu kecil untuk pengguna tmux? | Adopsi power user | Fase 3 |
| Q-05 | Apakah monitoring perlu dipertahankan di v2.0 GA, atau ditunda ke v2.1 demi memperkecil cakupan? | Jadwal | Awal Fase 5 |
| Q-06 | Format backup: Zstd + JSON, atau SQLite terenkripsi apa adanya (lebih sederhana, kurang portabel)? | Portabilitas backup | Fase 7 |
| Q-07 | Relay sync: Go (lebih cepat dibangun) atau Rust (satu bahasa)? | Kecepatan Phase 2 | Setelah v2.0 GA |
| Q-08 | Apakah menampilkan UI Sync/Team yang mati akan dianggap menyesatkan oleh komunitas? Perlu uji reaksi pada rilis beta. | Reputasi | Beta |
| Q-09 | Apakah v2.0 perlu mengimpor vault v1 secara otomatis saat mendeteksinya, atau selalu manual? | UX migrasi | Fase 1 |
| Q-10 | Nama produk final di package registry — `caterm` bentrok dengan paket lain? | Distribusi | Sebelum rilis |

---

## 17. Rencana Rilis

| Fase | Isi | Estimasi | Gerbang keluar |
|---|---|---|---|
| **Fase 0** | Bootstrap, arch guard, CI, budget harness | 1 minggu | CI hijau di 3 OS; prototipe mengukur RSS |
| **Fase 1** | Storage, migrasi, kripto, vault, auto-lock | 2 minggu | Coverage kripto ≥ 95%; vault v1 terbaca |
| **Fase 2** | SSH engine, TOFU, PTY, flow control | 2 minggu | Sesi stabil 8 jam; `yes` tidak menumbuhkan RSS |
| **Fase 3** | UI shell, hosts, groups, tabs, split, palette | 2,5 minggu | **MVP dapat dirilis sebagai v2.0-beta** |
| **Fase 4** | SFTP explorer & transfer engine | 1,5 minggu | Resume 1 GB terbukti |
| **Fase 5** | Snippets, tunnels, monitoring, audit, keys | 2 minggu | Seluruh tunnel tertutup bersih saat lock |
| **Fase 6** | AI Copilot + Redaction Guard + profil provider & validasi | 1,5 minggu | 0 kebocoran di 200 kasus uji · 10 kode CAT-AI terpicu |
| **Fase 7** | Backup/restore, placeholder Sync & Team | 1 minggu | Restore lintas mesin terbukti |
| **Fase 8** | Hardening, performa, packaging, rilis | 1,5 minggu | Seluruh budget REQ-02 lolos; 6 artefak tertanda tangan |

**Total: ~15 minggu (≈ 3,5 bulan)** pada asumsi 15 jam/minggu.

**Titik rilis antara:** setelah Fase 3, aplikasi sudah layak dirilis sebagai **v2.0-beta** (host + terminal + TOFU + vault). Fase 4–6 boleh menyusul sebagai v2.0-rc dan v2.1. Ini mitigasi utama untuk R-03 dan R-05.

---

## 18. Definisi Selesai (Definition of Done) v2.0 GA

CATerm v2.0 dinyatakan GA hanya jika **seluruh** baris berikut benar:

- [ ] 38 REQ berstatus P0 dan P1 terimplementasi dan terbukti lewat test otomatis (termasuk REQ-37 2FA/TOTP dan REQ-38 Scheduled Tasks/SFTP Backup).
- [ ] Seluruh 21 anti-regresi §12 punya test yang merah bila masalahnya kembali.
- [ ] `cargo clippy --workspace -- -D warnings` = 0 warning.
- [ ] Coverage `caterm-core` ≥ 80%, modul kripto ≥ 95%.
- [ ] Seluruh budget REQ-02 lolos di 3 OS.
- [ ] Soak test 8 jam tanpa kebocoran memori.
- [ ] `cargo audit` & `cargo deny` bersih.
- [ ] 6+ artefak rilis tertanda tangan dengan checksum terpublikasi.
- [ ] `README.md`, `SECURITY.md`, `THREAT_MODEL.md`, `CONTRIBUTING.md`, `CHANGELOG.md` lengkap dan bukan template.
- [ ] Vault CATerm v1 terbukti dapat dimigrasi tanpa kehilangan data.
- [ ] Aplikasi berfungsi penuh dengan jaringan dimatikan (REQ-09).
- [ ] 0 byte trafik jaringan saat AI & Sync nonaktif.
- [ ] Layar Sync & Team ter-*ship* dalam kondisi disabled dengan badge yang jujur.

---

*PRD v2.0 · CATerm · Cecep Saeful Azhar Hidayat · Fathforce (PT Fath Synergy Group) · 18 September 2026*
*Dokumen ini mengikat. Perubahan cakupan wajib melewati revisi dokumen ini terlebih dahulu, bukan lewat keputusan di tengah koding.*
