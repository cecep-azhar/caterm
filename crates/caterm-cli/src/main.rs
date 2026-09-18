use clap::{Parser, Subcommand};

/// catermctl — headless verification CLI for CATerm v2.
#[derive(Parser, Debug)]
#[command(name = "catermctl", version = caterm_core::CORE_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Error code registry (REQ-04)
    Errors {
        #[command(subcommand)]
        action: ErrorsAction,
    },
}

#[derive(Subcommand, Debug)]
enum ErrorsAction {
    /// List every currently-defined error code, domain, and message.
    List,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Errors {
            action: ErrorsAction::List,
        }) => print_errors_list(),
        None => {}
    }
}

fn print_errors_list() {
    for err in caterm_core::CatermError::all_known_for_registry() {
        println!("{}\t{}\t{}", err.code(), err.domain(), err.user_message());
    }
}
