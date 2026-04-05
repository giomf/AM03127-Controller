mod commands;
mod config;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use config::Config;

#[derive(Parser)]
#[command(about = "AM03127 panel controller CLI")]
struct Args {
    /// Path to the TOML config file
    #[arg(short, long, default_value = "panel-config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check connectivity status of all panels
    Status,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = Config::from_file(&args.config).unwrap_or_else(|e| {
        eprintln!("Error reading config '{}': {e}", args.config.display());
        std::process::exit(1);
    });

    match args.command {
        Commands::Status => commands::status::run(&config).await,
    }
}
