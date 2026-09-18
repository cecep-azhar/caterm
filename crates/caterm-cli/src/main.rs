mod bench;

use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// catermctl — headless verification CLI for CATerm v2.
#[derive(Parser, Debug)]
#[command(name = "catermctl", version = caterm_core::CORE_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print the resolved runtime data directory and store location (REQ-30).
    Env(JsonFlag),
    /// Error code registry (REQ-04)
    Errors {
        #[command(subcommand)]
        action: ErrorsAction,
    },
    /// Resource budget harness (REQ-02, T2-BOOT-06).
    Bench {
        #[command(subcommand)]
        action: BenchAction,
    },
}

#[derive(Subcommand, Debug)]
enum BenchAction {
    /// Build a full report: binary size, cold start, RSS (main + descendant WebView procs).
    Run {
        /// Path to the caterm-app release binary. Defaults to the sibling binary next to
        /// this catermctl executable (cargo places all workspace bins in the same dir).
        #[arg(long)]
        exe: Option<PathBuf>,
        /// Seconds to wait after cold-start before snapshotting RSS, so WebView child
        /// processes finish spawning.
        #[arg(long, default_value_t = 3)]
        settle_secs: u64,
        /// Seconds to wait for the CATERM_COLD_START_MS line before giving up.
        #[arg(long, default_value_t = 15)]
        timeout_secs: u64,
        /// Emit machine-readable JSON instead of a human table.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
struct JsonFlag {
    /// Emit machine-readable JSON (pretty-printed, one field per line) instead of plain text.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum ErrorsAction {
    /// List every currently-defined error code, domain, and message.
    List,
}

#[derive(Serialize)]
struct EnvInfo {
    data_dir: String,
    source: String,
    platform: String,
    version: &'static str,
    db_path: String,
    db_exists: bool,
    db_size_bytes: u64,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Some(Commands::Env(flags)) => {
            print_env(flags.json);
            0
        }
        Some(Commands::Errors {
            action: ErrorsAction::List,
        }) => {
            print_errors_list();
            0
        }
        Some(Commands::Bench {
            action:
                BenchAction::Run {
                    exe,
                    settle_secs,
                    timeout_secs,
                    json,
                },
        }) => run_bench(exe, settle_secs, timeout_secs, json),
        None => 0,
    };

    // Only main()'s own outermost statement exits the process - every helper above
    // returns a code instead of exiting itself. clippy::disallowed_methods bans
    // std::process::exit crate-wide with no way to express a location-based exception
    // (it fires here too, in main(), the one place the policy's own reason text sanctions
    // it) - this is that single, intentional, narrowly-scoped exemption.
    #[allow(clippy::disallowed_methods)]
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

fn run_bench(exe: Option<PathBuf>, settle_secs: u64, timeout_secs: u64, as_json: bool) -> i32 {
    let exe_path = match exe.or_else(|| bench::default_exe_path().ok()) {
        Some(p) => p,
        None => {
            eprintln!("gagal menentukan path exe caterm-app; pakai --exe");
            return 1;
        }
    };

    match bench::run(&exe_path, settle_secs, timeout_secs) {
        Ok(report) => {
            if as_json {
                match serde_json::to_string_pretty(&report) {
                    Ok(j) => println!("{j}"),
                    Err(e) => eprintln!("gagal serialize bench report: {e}"),
                }
            } else {
                println!("binary_size_bytes={}", report.binary_size_bytes);
                println!(
                    "binary_size_mb={:.2}",
                    report.binary_size_bytes as f64 / 1_048_576.0
                );
                println!("cold_start_ms={}", report.cold_start_ms);
                match report.private_bytes {
                    Some(b) => {
                        println!("private_bytes={b}   <-- ANGKA BUDGET REQ-02");
                        println!("private_mb={:.2}", b as f64 / 1_048_576.0);
                    }
                    None => {
                        println!("private_bytes=null  <-- platform ini belum didukung");
                        println!("private_mb=null");
                    }
                }
                println!(
                    "working_set_sum_bytes={}  (transparansi saja - double-count halaman bersama)",
                    report.working_set_sum_bytes
                );
                println!(
                    "working_set_sum_mb={:.2}",
                    report.working_set_sum_bytes as f64 / 1_048_576.0
                );
                println!("process_count={}", report.process_count);
                println!("db_query_ms=null");
                println!("db_query_note={}", report.db_query_note);
                println!("memory_note={}", report.memory_note);
            }
            0
        }
        Err(e) => {
            eprintln!("bench gagal: {e}");
            1
        }
    }
}

fn print_env(as_json: bool) {
    let Ok(dir_info) = caterm_core::paths::resolve_data_dir() else {
        eprintln!("gagal menentukan direktori data");
        return;
    };
    let db_path = caterm_core::paths::db_path(&dir_info.path);
    let metadata = std::fs::metadata(&db_path);

    let info = EnvInfo {
        data_dir: dir_info.path.display().to_string(),
        source: dir_info.source.as_str().to_string(),
        platform: std::env::consts::OS.to_string(),
        version: caterm_core::CORE_VERSION,
        db_path: db_path.display().to_string(),
        db_exists: metadata.is_ok(),
        db_size_bytes: metadata.map(|m| m.len()).unwrap_or(0),
    };

    if as_json {
        match serde_json::to_string_pretty(&info) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("gagal serialize env info: {e}"),
        }
        return;
    }

    println!("data_dir={}", info.data_dir);
    println!("source={}", info.source);
    println!("platform={}", info.platform);
    println!("version={}", info.version);
    println!("db_path={}", info.db_path);
    println!("db_exists={}", info.db_exists);
    println!("db_size_bytes={}", info.db_size_bytes);
}

fn print_errors_list() {
    for err in caterm_core::CatermError::all_known_for_registry() {
        println!("{}\t{}\t{}", err.code(), err.domain(), err.user_message());
    }
}
