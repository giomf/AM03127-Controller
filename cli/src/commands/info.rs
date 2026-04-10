use std::{fs, path::Path};

use anyhow::{Context, Result};
use console::style;

use super::FirmwareBuildInfo;
use crate::console::print_title;

pub fn run(firmware_path: &Path) -> Result<()> {
    let firmware = fs::read(firmware_path)
        .with_context(|| format!("failed to read '{}'", firmware_path.display()))?;

    match FirmwareBuildInfo::parse(&firmware) {
        Some(info) => {
            print_title(&format!(
                "Firmware: {} {}",
                style(&info.git_hash).cyan(),
                style(format!("{} {}", info.build_date, info.build_time)).dim(),
            ));
        }
        None => {
            anyhow::bail!(
                "file '{}' is too short to contain firmware header",
                firmware_path.display()
            );
        }
    }

    Ok(())
}
