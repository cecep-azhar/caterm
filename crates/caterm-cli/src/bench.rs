//! `catermctl bench run` (T2-BOOT-06). Measures the four Fase-0-measurable REQ-02 metrics
//! against a *release* build: binary size, cold start, and RSS of the main process **plus
//! every descendant WebView process** — measuring only the Rust process is explicitly
//! forbidden by task-v2.md, because it would hide exactly the cost WebView adds.
//!
//! `db_query_ms` is reported as `null`/unavailable: there is no `hosts` table yet (that
//! schema is normative in Fase 1, T2-CORE-01). Fabricating a number against a table that
//! does not exist would be the "data palsu" anti-pattern this whole project exists to avoid.

use serde::Serialize;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

#[derive(Serialize)]
pub struct BenchReport {
    pub binary_size_bytes: u64,
    pub cold_start_ms: u128,
    pub rss_bytes: u64,
    pub process_count: usize,
    pub db_query_ms: Option<u64>,
    pub db_query_note: &'static str,
}

pub fn default_exe_path() -> Result<PathBuf, String> {
    let mine = std::env::current_exe().map_err(|e| format!("current_exe gagal: {e}"))?;
    let dir = mine
        .parent()
        .ok_or_else(|| "current_exe tidak punya parent dir".to_string())?;
    Ok(dir.join(format!("caterm-app{}", std::env::consts::EXE_SUFFIX)))
}

pub fn run(exe: &Path, settle_secs: u64, timeout_secs: u64) -> Result<BenchReport, String> {
    let binary_size_bytes = std::fs::metadata(exe)
        .map_err(|e| format!("gagal stat {}: {e}", exe.display()))?
        .len();

    let (cold_start_ms, rss_bytes, process_count) =
        spawn_and_measure(exe, settle_secs, timeout_secs)?;

    Ok(BenchReport {
        binary_size_bytes,
        cold_start_ms,
        rss_bytes,
        process_count,
        db_query_ms: None,
        db_query_note: "belum tersedia: tabel `hosts` belum ada (Fase 1, T2-CORE-01). Bukan 0ms - memang belum terukur.",
    })
}

fn spawn_and_measure(
    exe: &Path,
    settle_secs: u64,
    timeout_secs: u64,
) -> Result<(u128, u64, usize), String> {
    let mut child = Command::new(exe)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("gagal spawn {}: {e}", exe.display()))?;

    let pid = child.id();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "child stdout tidak tersedia".to_string())?;

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(value) = line.strip_prefix("CATERM_COLD_START_MS=") {
                let _ = tx.send(value.to_string());
                return;
            }
        }
    });

    let cold_start_ms: u128 = match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(value) => value.parse().unwrap_or(0),
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "timeout {timeout_secs}s menunggu CATERM_COLD_START_MS dari {}",
                exe.display()
            ));
        }
    };

    std::thread::sleep(Duration::from_secs(settle_secs));

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let root = sysinfo::Pid::from_u32(pid);
    let pids = descendant_pids(&sys, root);
    let rss_bytes: u64 = pids
        .iter()
        .filter_map(|p| sys.process(*p))
        .map(|p| p.memory())
        .sum();
    let process_count = pids.len();

    let _ = child.kill();
    let _ = child.wait();

    Ok((cold_start_ms, rss_bytes, process_count))
}

/// `root` plus every process transitively parented by it (WebView2's browser/renderer/GPU
/// process tree on Windows, or the WebKitGTK helper processes on Linux).
fn descendant_pids(sys: &sysinfo::System, root: sysinfo::Pid) -> Vec<sysinfo::Pid> {
    let mut found = vec![root];
    let mut frontier = vec![root];
    while let Some(parent) = frontier.pop() {
        for (pid, process) in sys.processes() {
            if process.parent() == Some(parent) && !found.contains(pid) {
                found.push(*pid);
                frontier.push(*pid);
            }
        }
    }
    found
}
