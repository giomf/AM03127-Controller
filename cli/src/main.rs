mod config;

use std::path::PathBuf;

use clap::Parser;
use config::Config;

#[derive(Parser)]
#[command(about = "AM03127 panel controller CLI")]
struct Args {
    /// Path to the TOML config file
    #[arg(short, long, default_value = "panel-config.toml")]
    config: PathBuf,
}

fn main() {
    let args = Args::parse();

    let config = Config::from_file(&args.config).unwrap_or_else(|e| {
        eprintln!("Error reading config '{}': {e}", args.config.display());
        std::process::exit(1);
    });

    for panel in &config.panels {
        println!("Panel: {} @ {}", panel.name, panel.address);
    }
}
