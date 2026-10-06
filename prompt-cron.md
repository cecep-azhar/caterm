# 🚀 Autopilot Master Prompt — CATerm Scheduled Tasks, Playbooks & SFTP Auto-Backup (REQ-38)

> **Dokumen Terkait:** `cron.md` (Spesifikasi Arsitektur & Skema DB), `prd-v2.md` (REQ-38).  
> **Target Repositori:** `/home/cecepazhar/Product/caterm`  
> **Tujuan:** Mengimplementasikan modul **Agentless Scheduled Tasks, Remote SSH Action Playbooks, dan SFTP Auto-Backup Engine** di CATerm v2 secara otonom penuh (*autopilot*) melalui 4 sub-agent paralel terkoordinasi.

---

## 🎯 Ringkasan Tujuan Teknis

Membangun modul penjadwalan (*scheduler*) di CATerm yang mampu mengeksekusi script SSH remote dan menarik file backup server via SFTP ke penyimpanan lokal PC secara otomatis di latar belakang (System Tray), dengan keamanan brankas terenkripsi SQLCipher.

---

## 🤖 Pembagian Tugas & Instruksi 4 Sub-Agent

```
┌────────────────────────────────────────────────────────────────────────┐
│                          ORCHESTRATOR AGENT                            │
│            (Supervisi, Integrasi IPC, & Verifikasi Gates)              │
└───────┬───────────────────┬───────────────────┬────────────────┬───────┘
        │                   │                   │                │
        ▼                   ▼                   ▼                ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐   ┌────────────┐
│  SUB-AGENT 1  │   │  SUB-AGENT 2  │   │  SUB-AGENT 3  │   │ SUB-AGENT 4│
│ Tokio Cron    │   │ SSH & SFTP    │   │  Svelte 5 UI  │   │  QA & Tray │
│ Core Engine   │   │ Backup Runner │   │ Task Manager  │   │ Integration│
└───────────────┘   └───────────────┘   └───────────────┘   └────────────┘
```

---

### 🔹 **SUB-AGENT 1: Tokio Async Cron Engine & Database Migrations**
* **Target Direktori:** `crates/caterm-core/`
* **Tugas:**
  1. Tambahkan dependency parser cron di `crates/caterm-core/Cargo.toml` (misal: `cron = "0.12"`).
  2. Buat modul `crates/caterm-core/src/scheduler/`:
     - `engine.rs`: Scheduler loop berbasis `tokio::time::interval` yang mengevaluasi ekspresi cron (`schedule_expr`) dan interval (`every Nh`, `every Nm`).
     - `task.rs`: Model data `ScheduledTaskRecord`, `TaskInput`, `TaskExecutionLog`.
  3. Migrasi Skema SQLCipher di `crates/caterm-core/src/store.rs`:
     - Buat tabel `scheduled_tasks` dan `task_execution_logs` sesuai spesifikasi `cron.md §4`.
     - Fungsi CRUD: `list_tasks`, `save_task`, `delete_task`, `get_task_logs`, `log_execution_result`.
  4. Expose Tauri IPC Commands di `crates/caterm-app/src/commands.rs`:
     - `list_scheduled_tasks() -> Vec<ScheduledTaskRecord>`
     - `save_scheduled_task(input: TaskInput) -> ScheduledTaskRecord`
     - `delete_scheduled_task(id: String) -> ()`
     - `trigger_task_run_now(task_id: String) -> ()`
     - `get_task_execution_logs(task_id: String) -> Vec<TaskExecutionLog>`
* **Gate 1:** `cargo check -p caterm-core -p caterm-app` PASS.

---

### 🔹 **SUB-AGENT 2: SSH Remote Action & SFTP Backup Pull Pipeline**
* **Target Direktori:** `crates/caterm-core/src/scheduler/runner.rs`
* **Tugas:**
  1. Implementasi **SSH Command Task Runner**:
     - Membuka koneksi SSH ke target `host_id` menggunakan kredensial terdekripsi dari vault.
     - Mengeksekusi `command_script` via SSH exec channel dengan timeout handling.
     - Menangkap output `stdout`, `stderr`, dan `exit_code`.
  2. Implementasi **SFTP Auto-Backup Pull Pipeline**:
     - *Langkah 1 (Pre-cmd):* Eksekusi `remote_pre_cmd` di server (misal: `mysqldump ... | gzip > /tmp/db.sql.gz`).
     - *Langkah 2 (SFTP Download):* Unduh file `remote_src_path` ke `local_dest_dir` dengan penamaan berbasis timestamp (`db_20261006_020000.sql.gz`). Menggunakan file sementara `.part` selama proses download.
     - *Langkah 3 (Post-cmd):* Eksekusi `remote_post_cmd` (misal: `rm -f /tmp/db.sql.gz`) untuk membersihkan file di server.
     - *Langkah 4 (Retention Policy):* Hapus file backup lokal tertua jika jumlah file di `local_dest_dir` melebihi `retention_count`.
  3. Catat hasil eksekusi secara otomatis ke tabel `task_execution_logs`.
* **Gate 2:** `cargo test -p caterm-core --test scheduler*` PASS.

---

### 🔹 **SUB-AGENT 3: Frontend Svelte 5 UI, Task Editor & Log Viewer**
* **Target Direktori:** `frontend/src/`
* **Tugas:**
  1. Buat halaman utama tugas terjadwal `frontend/src/routes/tasks/+page.svelte`:
     - Header status scheduler aktif (`● Scheduler Running`).
     - Tabel daftar tugas: Status badge (Active/Paused/Running), Nama & Tipe tugas, Target Host, Jadwal, Last Run Status, Next Run countdown, tombol *Run Now*.
  2. Buat modal editor tugas `frontend/src/lib/components/TaskModal.svelte`:
     - Tab tipe tugas: *SSH Script Playbook* vs *SFTP Auto-Backup Pull*.
     - Dropdown pemilihan Target Host dari daftar server tersimpan di CATerm.
     - Generator ekspresi cron dengan preview teks ramah pengguna (contoh: `0 2 * * *` -> *"Setiap hari jam 02:00 WIB"*).
     - Form folder tujuan lokal (*Local destination folder*).
  3. Buat modal inspektur log `frontend/src/lib/components/TaskLogModal.svelte`:
     - Daftar riwayat eksekusi kronologis.
     - Log viewer bergaya terminal gelap dengan format waktu, durasi (ms), exit code, dan ukuran transfer data.
  4. Tambahkan menu navigasi **"Scheduled Tasks"** di sidebar utama CATerm.
* **Gate 3:** `npm run build` dan `npm test` di `frontend/` PASS tanpa error.

---

### 🔹 **SUB-AGENT 4: System Tray Lifecycle, OS Notifications & E2E Testing**
* **Target Direktori:** `crates/caterm-app/`, `tests/`
* **Tugas:**
  1. Integrasi **Tauri v2 System Tray & Background Worker**:
     - Saat jendela utama ditutup (*close to tray*), pastikan scheduler loop Tokio tetap berjalan di thread latar belakang.
     - Menu tray: Status tugas terdekat, opsi *"Buka CATerm"*, *"Jalankan Backup Sekarang"*, *"Keluar"*.
  2. Integrasi **OS Desktop Notifications**:
     - Kirim notifikasi desktop native saat backup selesai (`✅ Backup VPS Selesai (14.5 MB)`) atau jika terjadi kegagalan koneksi (`❌ Backup Gagal: Host unreachable`).
  3. Unit & Integration Tests:
     - Test parsing cron & interval evaluation.
     - Test simulasi SFTP download dengan mock SFTP server.
     - Test retention cleanup (memverifikasi file terlama terhapus sesuai batas retensi).
* **Gate 4:** `cargo test --workspace` 100% PASS & `cargo build --release` berhasil.

---

## 📋 Acceptance Criteria & Quality Gates

1. **Agentless Guarantee:** Tidak ada software/skrip tambahan yang perlu diinstal di server remote.
2. **Zero Plaintext Credentials:** Semua password/kunci SSH didekripsi langsung dari SQLCipher vault saat runtime dan di-*zeroize* setelah sesi selesai.
3. **Tray Idle Efficiency:** Pemakaian CPU 0,0% saat idle, RAM < 25 MB.
4. **Data Integrity:** Penarikan file backup menggunakan format atomik `.part` untuk mencegah file korup saat koneksi terputus.

---

## 🛠️ Perintah Eksekusi Rilis & Pemasangan

Setelah seluruh tahapan selesai dan terverifikasi:
```bash
cargo build --release --manifest-path crates/caterm-app/Cargo.toml
npx --prefix frontend tauri build --config crates/caterm-app/tauri.conf.json
cp target/release/caterm ~/.local/bin/caterm && chmod +x ~/.local/bin/caterm
```
Laporkan bukti output terminal hasil eksekusi dan verifikasi kepada Prof. Cecep.
