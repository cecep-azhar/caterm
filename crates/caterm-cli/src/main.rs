use clap::Parser;

/// catermctl — headless verification CLI for CATerm v2.
#[derive(Parser, Debug)]
#[command(name = "catermctl", version = caterm_core::CORE_VERSION)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
