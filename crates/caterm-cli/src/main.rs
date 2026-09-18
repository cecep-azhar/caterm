use clap::{Args, Parser, Subcommand};
use serde::Serialize;

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
    match cli.command {
        Some(Commands::Env(flags)) => print_env(flags.json),
        Some(Commands::Errors {
            action: ErrorsAction::List,
        }) => print_errors_list(),
        None => {}
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
