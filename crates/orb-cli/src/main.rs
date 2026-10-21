//! Headless orb: scanning, indexing, searching and benchmarking without the desktop UI.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orb", version = orb_core::VERSION, about = "orb media indexer (headless)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print build and environment information.
    Info,
}

fn main() -> anyhow::Result<()> {
    orb_core::init_tracing();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Info => {
            let paths = orb_core::paths::AppPaths::discover()?;
            println!("orb {}", orb_core::VERSION);
            println!("data:   {}", paths.data);
            println!("cache:  {}", paths.cache);
            println!("models: {}", paths.models);
        }
    }
    Ok(())
}
