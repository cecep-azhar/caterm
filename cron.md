# ⏰ CATerm Scheduled Tasks, Auto-Playbooks & SFTP Backup Engine (REQ-38)

> **Dokumen:** Arsitektur & Spesifikasi Fitur Scheduler CATerm v2  
> **Target Aplikasi:** CATerm (`/home/cecepazhar/Product/caterm`)  
> **Status:** Spesifikasi Resmi & Binding  
> **Owner:** Cecep Saeful Azhar Hidayat, ST (Prof. Cecep)  
> **Stack:** Rust (Tokio Cron Engine) + Tauri v2 Tray + Svelte 5 + SQLCipher Vault  

---

## 1. 🎯 Visi & Latar Belakang

Banyak sysadmin dan developer mengelola puluhan server Linux/Cloud. Saat ini, tugas-tugas rutin (seperti backup database mingguan, penarikan file backup ke laptop lokal, restart container, atau pembersihan disk) membutuhkan instalasi script crontab manual di setiap server terpisah atau aplikasi pihak ketiga.

**CATerm Scheduled Tasks (REQ-38)** menghadirkan modul **Agentless Server Automation & Auto-Backup** langsung di dalam CATerm:
1. **Agentless:** 100% berjalan melalui protokol SSH2 dan SFTP native dari PC lokal — **tanpa perlu menginstall agen/daemon tambahan apapun di server target**.
2. **Local Pull Backup:** Menjalankan dump di remote server, menarik (*download*) file hasil kompresi via SFTP ke PC lokal/NAS secara terjadwal, dan menghapus file sementara di remote.
3. **Tray Daemon Ultra-Ringan:** Tetap berjalan di latar belakang (System Tray) dengan konsumsi RAM < 25 MB dan CPU 0,0% saat idle.
4. **Zero-Knowledge Security:** Seluruh perintah, jadwal cron, kunci SSH, dan log riwayat tersimpan aman dalam database terenkripsi SQLCipher.

---

## 2. 🏗️ Arsitektur Sistem

```
┌────────────────────────────────────────────────────────────────────────┐
│                        CATerm Desktop UI                               │
│      (Tab: Scheduled Tasks · Task Editor Modal · Log Inspector)       │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ IPC Commands
┌───────────────────────────────────▼────────────────────────────────────┐
│                  CATerm App Tray Daemon (Rust/Tauri v2)                │
│                                                                        │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │                 Tokio Async Cron Scheduler                     │   │
│   │   - Standard Cron Syntax (e.g. `0 2 * * *`)                    │   │
│   │   - Interval Notation (`every 6h`, `every 30m`)                │   │
│   │   - One-shot Schedule (`at 2026-10-07T02:00:00`)               │   │
│   └───────────────┬────────────────────────────────┬───────────────┘   │
│                   │                                │                   │
│         ┌─────────▼─────────┐            ┌─────────▼─────────┐         │
│         │   SSH Action      │            │  SFTP Pull Backup │         │
│         │   Task Runner     │            │      Engine       │         │
│         └─────────┬─────────┘            └─────────┬─────────┘         │
│                   │                                │                   │
└───────────────────┼────────────────────────────────┼───────────────────┘
                    │ SSH Protocol                   │ SFTP Protocol
                    ▼                                ▼
       ┌────────────────────────┐       ┌────────────────────────┐
       │     Remote Server      │       │     Local Storage      │
       │ (mysqldump, docker, df)│       │  (e.g. ~/Backups/vps/) │
       └────────────────────────┘       └────────────────────────┘
```

---

## 3. 🧩 Tipe Tugas yang Didukung

### A. 📦 SFTP Auto-Backup Pull (Remote-to-Local)
Skenario backup otomatis tanpa membebani disk server:
1. **Pre-Command (Remote):** Menjalankan perintah pembuatan backup di server (misal: `mysqldump -u root ... | gzip > /tmp/db_backup.sql.gz`).
2. **SFTP Download:** Mengunduh file `/tmp/db_backup.sql.gz` ke folder lokal (misal: `/home/cecepazhar/Backups/vps/db_backup_20261006.sql.gz`).
3. **Post-Command (Remote Cleanup):** Menghapus file sementara di server (`rm -f /tmp/db_backup.sql.gz`).
4. **Local Retention Policy:** Otomatis merotasi dan menghapus backup lokal yang lebih tua dari $N$ hari / menyimpan maksimal $M$ file.

### B. ⚡ Remote SSH Command & Playbooks
Eksekusi rentetan script bash terjadwal di satu atau banyak host sekaligus:
* *Maintenance berkala:* `docker system prune -f && apt-get update`
* *Log cleanup:* `journalctl --vacuum-time=7d`
* *Health watchdog:* Memeriksa kapasitas disk `df -h /` dan memicu alert jika utilisasi > 90%.

### C. 🖥️ Local Actions
Menjalankan script atau backup internal data CATerm sendiri secara berkala ke folder arsip lokal.

---

## 4. 🗄️ Skema Basis Data SQLCipher

Skema tabel baru ditambahkan ke database terenkripsi CATerm (`caterm.db`):

```sql
-- 1. Tabel Definisi Tugas Terjadwal
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    description        TEXT,
    task_type          TEXT NOT NULL, -- 'ssh_command' | 'sftp_backup' | 'local_script'
    host_id            TEXT REFERENCES hosts(id) ON DELETE CASCADE,
    schedule_expr      TEXT NOT NULL, -- e.g. "0 2 * * *" atau "every 6h"
    is_enabled         BOOLEAN NOT NULL DEFAULT 1,
    
    -- Konfigurasi SSH Command
    command_script     TEXT,          -- Multi-line bash script yang dieksekusi
    timeout_seconds    INTEGER NOT NULL DEFAULT 300,
    
    -- Konfigurasi SFTP Backup Pull
    remote_src_path    TEXT,          -- e.g. "/tmp/backup.tar.gz"
    local_dest_dir     TEXT,          -- e.g. "/home/cecepazhar/Backups/vps"
    remote_pre_cmd     TEXT,          -- e.g. "tar -czf /tmp/backup.tar.gz /var/www/html"
    remote_post_cmd    TEXT,          -- e.g. "rm -f /tmp/backup.tar.gz"
    retention_count    INTEGER NOT NULL DEFAULT 7,
    
    -- Notifikasi & Status
    notify_on_success  BOOLEAN NOT NULL DEFAULT 0,
    notify_on_failure  BOOLEAN NOT NULL DEFAULT 1,
    last_run_at        TEXT,
    last_status        TEXT,          -- 'success' | 'failed' | 'running'
    next_run_at        TEXT,
    
    created_at         TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at         TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. Tabel Log Riwayat Eksekusi
CREATE TABLE IF NOT EXISTS task_execution_logs (
    id                 TEXT PRIMARY KEY,
    task_id            TEXT NOT NULL REFERENCES scheduled_tasks(id) ON DELETE CASCADE,
    started_at         TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at        TEXT,
    duration_ms        INTEGER,
    exit_code          INTEGER,
    status             TEXT NOT NULL, -- 'success' | 'failed' | 'timeout' | 'cancelled'
    stdout             TEXT,
    stderr             TEXT,
    bytes_transferred  INTEGER,       -- Ukuran file yang di-download (bila SFTP)
    error_message      TEXT
);

CREATE INDEX IF NOT EXISTS idx_task_logs_task_id ON task_execution_logs(task_id, started_at DESC);
```

---

## 5. 🎨 Desain Antarmuka & UX (Svelte 5)

### A. Tab "Scheduled Tasks" di Sidebar CATerm:
* Header dengan indikator status Daemon: `● Scheduler Active (3 Jobs Scheduled)`.
* Tombol aksi: `+ New Scheduled Task` dan `Run All Now`.
* Tabel interaktif dengan kolom:
  - **Status Badge:** `Active (Hijau)` / `Paused (Abu-abu)` / `Running (Animasi Biru)`.
  - **Task Name & Type:** Ikon tipe tugas (`⚡ SSH Script` atau `📦 SFTP Backup`).
  - **Target Server:** Nama host & IP (`VPS Hostinger (100.76.150.46)`).
  - **Schedule:** Format manusiawi (`Every day at 02:00` / `Every 6 hours`).
  - **Last Run:** Waktu eksekusi terakhir & badge status (`Success 14.2 MB · 4s ago`).
  - **Next Run:** Waktu hitung mundur (`In 3 hours`).
  - **Actions:** Tombol *Trigger Run Now*, *View Logs*, *Edit*, *Delete*.

### B. Task Editor Modal:
* Input ekspresi jadwal dengan live preview waktu jalannya (misal: ketik `0 2 * * *` -> *"Runs everyday at 02:00 AM"*).
* Editor bash script dengan syntax highlighting.
* File picker untuk folder tujuan backup lokal.

### C. Log Inspector Modal:
* Tampilan stream log berwarna ala terminal (ANSI color support).
* Ringkasan durasi eksekusi, ukuran file terunduh, dan exit code.

---

## 6. 🛡️ Toleransi Kesalahan & Kepatuhan Keamanan
1. **Network Retry Policy:** Jika koneksi SSH gagal karena jaringan terputus, scheduler mencoba kembali (*retry*) sebanyak 3 kali dengan *exponential backoff* (10s, 30s, 60s).
2. **Atomic Temp Transfer:** File SFTP diunduh dengan ekstensi `.part` (misal: `backup.sql.gz.part`), dan di-rename setelah verifikasi ukuran/checksum selesai agar tidak terjadi korupsi data.
3. **Zeroization:** Kredensial SSH host didekripsi dari vault hanya saat eksekusi dimulai dan langsung di-*zeroize* setelah sesi ditutup.
