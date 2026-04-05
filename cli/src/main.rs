mod commands;
mod config;
mod console;

use std::path::PathBuf;

use anyhow::Result;
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
    Status {
        /// Panels to target, comma-separated (default: all)
        #[arg(short, long, value_delimiter = ',')]
        panels: Vec<String>,
    },
    /// Upload a firmware binary to panels via OTA
    Update {
        /// Path to the firmware .bin file
        firmware: PathBuf,
        /// Panels to target, comma-separated (default: all)
        #[arg(short, long, value_delimiter = ',')]
        panels: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    let config = Config::from_file(&args.config)?;

    match args.command {
        Commands::Status { panels } => {
            let targets = config.select_panels(&panels)?;
            commands::status::run(&targets).await?;
        }
        Commands::Update { firmware, panels } => {
            let targets = config.select_panels(&panels)?;
            commands::update::run(&targets, &firmware).await?;
        }
    }

    Ok(())
}
