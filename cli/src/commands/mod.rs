pub mod list;
pub mod open;
pub mod status;
pub mod update;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "AM03127 panel controller CLI")]
pub struct Args {
    /// Path to the TOML config file
    #[arg(short, long, default_value = "panel-config.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// List all known panels
    List {},

    /// Open a panel's address in the browser
    Open {
        /// Panel to open
        panel: String,
    },
}
