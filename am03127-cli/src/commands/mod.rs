pub mod clock;
pub mod info;
pub mod list;
pub mod open;
pub mod status;
pub mod update;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::Panel;

const FW_GIT_HASH_OFFSET: usize = 0x30;
const FW_GIT_HASH_LEN: usize = 16;
const FW_BUILD_TIME_OFFSET: usize = 0x70;
const FW_BUILD_TIME_LEN: usize = 16;
const FW_BUILD_DATE_OFFSET: usize = 0x80;
const FW_BUILD_DATE_LEN: usize = 16;
const FW_HEADER_MIN_LEN: usize = FW_BUILD_DATE_OFFSET + FW_BUILD_DATE_LEN;

pub struct FirmwareBuildInfo {
    pub git_hash: String,
    pub build_date: String,
    pub build_time: String,
}

impl FirmwareBuildInfo {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < FW_HEADER_MIN_LEN {
            return None;
        }
        Some(Self {
            git_hash: Self::read_cstr(data, FW_GIT_HASH_OFFSET, FW_GIT_HASH_LEN),
            build_date: Self::read_cstr(data, FW_BUILD_DATE_OFFSET, FW_BUILD_DATE_LEN),
            build_time: Self::read_cstr(data, FW_BUILD_TIME_OFFSET, FW_BUILD_TIME_LEN),
        })
    }

    fn read_cstr(data: &[u8], offset: usize, max_len: usize) -> String {
        let slice = &data[offset..(offset + max_len).min(data.len())];
        let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        String::from_utf8_lossy(&slice[..end]).into_owned()
    }
}

pub fn label_width(panels: &[&Panel]) -> usize {
    panels.iter().map(|p| p.name.len()).max().unwrap_or(0)
}

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
    /// Show firmware build info (git hash, date, time)
    Info {
        /// Path to the firmware .bin file
        firmware: PathBuf,
    },
    /// Sync the clock on panels to the current local time
    Clock {
        /// Panels to target, comma-separated (default: all)
        #[arg(short, long, value_delimiter = ',')]
        panels: Vec<String>,
    },
}
