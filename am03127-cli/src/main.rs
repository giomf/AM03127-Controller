mod commands;
mod config;
mod console;

use anyhow::Result;
use clap::Parser;
use config::Config;

use crate::commands::{Args, Commands};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Status { panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            commands::status::run(&targets).await?;
        }
        Commands::Update { firmware, panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            commands::update::run(&targets, &firmware).await?;
        }
        Commands::List {} => {
            let config = Config::from_file(&args.config)?;
            commands::list::run(&config.panels);
        }
        Commands::Open { panel } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&[panel])?;
            commands::open::run(targets[0])?;
        }
        Commands::Info { firmware } => {
            commands::info::run(&firmware)?;
        }
        Commands::Clock { panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            commands::clock::run(&targets).await?;
        }
        Commands::Page { id, message, leading, lagging, waiting_time, panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            let leading = commands::page::parse_leading(&leading)
                .map_err(|e| anyhow::anyhow!(e))?;
            let lagging = commands::page::parse_lagging(&lagging)
                .map_err(|e| anyhow::anyhow!(e))?;
            commands::page::run(&targets, id, message, leading, lagging, waiting_time).await?;
        }
    }

    Ok(())
}
