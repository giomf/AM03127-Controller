use std::{fs, path::Path};

use am03127_client::PanelClient;
use anyhow::{Context, Result};
use console::style;

use super::FirmwareBuildInfo;
use crate::{
    config::Panel,
    console::{SpinnerGroup, print_title},
};

pub async fn run(panels: &[&Panel], firmware_path: &Path) -> Result<()> {
    let firmware = fs::read(firmware_path)
        .with_context(|| format!("failed to read '{}'", firmware_path.display()))?;

    if let Some(info) = FirmwareBuildInfo::parse(&firmware) {
        print_title(&format!(
            "Updating {} panel(s) to {} built at {}",
            panels.len(),
            style(&info.git_hash).cyan(),
            style(format!("{} {}", info.build_date, info.build_time)).dim(),
        ));
    }

    let label_width = super::label_width(panels);
    let spinners = SpinnerGroup::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = PanelClient::new(&panel.address);
        let name = panel.name.clone();
        let firmware = firmware.clone();
        let pb = spinners.add(&name);

        set.spawn(async move {
            let result = client.update_firmware(&firmware).await;
            (name, result, pb)
        });
    }

    let mut success = true;
    while let Some(res) = set.join_next().await {
        let (name, result, pb) = res.context("panel task panicked")?;
        match result {
            Ok(_) => {
                pb.finish_with_message(format!(
                    "{} {name:<label_width$}  done, rebooting...",
                    style("✓").green()
                ));
            }
            Err(e) => {
                pb.finish_with_message(format!(
                    "{} {name:<label_width$}  {e}",
                    style("✗").red()
                ));
                success = false;
            }
        }
    }

    if !success {
        anyhow::bail!("one or more panels failed to update");
    }

    Ok(())
}

