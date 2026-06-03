mod commands;
mod config;
mod console;

use anyhow::Result;
use clap::Parser;
use config::Config;
use console::Console;

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
    let console = Console::new(args.plain);

    match args.command {
        Commands::Status { panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            commands::status::run(&targets, &console).await?;
        }
        Commands::Update { firmware, panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            commands::update::run(&targets, &firmware, &console).await?;
        }
        Commands::List {} => {
            let config = Config::from_file(&args.config)?;
            commands::list::run(&config.panels, &console);
        }
        Commands::Open { panel } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&[panel])?;
            commands::open::run(targets[0])?;
        }
        Commands::Info { firmware } => {
            commands::info::run(&firmware, &console)?;
        }
        Commands::Clock { panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            commands::clock::run(&targets, &console).await?;
        }
        Commands::Page { id, message, leading, lagging, waiting_time, panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            let leading = commands::page::parse_leading(&leading)
                .map_err(|e| anyhow::anyhow!(e))?;
            let lagging = commands::page::parse_lagging(&lagging)
                .map_err(|e| anyhow::anyhow!(e))?;
            commands::page::run(&targets, id, message, leading, lagging, waiting_time, &console).await?;
        }
        Commands::Schedule { id, page, from, to, panels } => {
            let config = Config::from_file(&args.config)?;
            let targets = config.select_panels(&panels)?;
            let from = commands::schedule::parse_schedule_datetime(&from)
                .map_err(|e| anyhow::anyhow!(e))?;
            let to = commands::schedule::parse_schedule_datetime(&to)
                .map_err(|e| anyhow::anyhow!(e))?;
            commands::schedule::run(&targets, id, page, from, to, &console).await?;
        }
    }

    Ok(())
}
