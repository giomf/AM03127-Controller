use std::{fs, path::Path};

use anyhow::{Context, Result};
use bytes::Bytes;
use console::style;
use futures::stream::{self, StreamExt};

use super::FirmwareBuildInfo;
use crate::{
    config::Panel,
    console::{ProgressGroup, print_title},
};

const CHUNK_SIZE: usize = 4096;

pub async fn run(panels: &[&Panel], firmware_path: &Path) -> Result<()> {
    let firmware = fs::read(firmware_path)
        .with_context(|| format!("failed to read '{}'", firmware_path.display()))?;
    let firmware_len = firmware.len() as u64;

    if let Some(info) = FirmwareBuildInfo::parse(&firmware) {
        print_title(&format!(
            "Updating {} panel(s) to {} built at {}",
            panels.len(),
            style(&info.git_hash).cyan(),
            style(format!("{} {}", info.build_date, info.build_time)).dim(),
        ));
    }
    let firmware = Bytes::from(firmware);

    let client = reqwest::Client::new();
    let label_width = panels.iter().map(|p| p.name.len()).max().unwrap_or(0);
    let bars = ProgressGroup::new(firmware_len, label_width);
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = client.clone();
        let name = panel.name.clone();
        let url = format!("http://{}/ota", panel.address);
        let firmware = firmware.clone();
        let pb = bars.add(&name);

        set.spawn(async move {
            let pb_stream = pb.clone();
            let body_stream = stream::iter(
                firmware
                    .chunks(CHUNK_SIZE)
                    .map(|c| Bytes::copy_from_slice(c))
                    .collect::<Vec<_>>(),
            )
            .map(move |chunk| {
                pb_stream.inc(chunk.len() as u64);
                Ok::<_, std::io::Error>(chunk)
            });

            let result = client
                .put(&url)
                .header("Content-Type", "application/octet-stream")
                .header("Content-Length", firmware_len)
                .body(reqwest::Body::wrap_stream(body_stream))
                .send()
                .await
                .and_then(|r| r.error_for_status());

            (name, result, pb)
        });
    }

    let mut success = true;
    while let Some(res) = set.join_next().await {
        let (name, result, pb) = res.context("panel task panicked")?;
        match result {
            Ok(_) => {
                pb.println(format!("✓ {name} done, rebooting..."));
                pb.finish_and_clear();
            }
            Err(e) => {
                pb.println(format!("✗ {name} {e}"));
                pb.finish_and_clear();
                success = false;
            }
        }
    }

    if !success {
        anyhow::bail!("one or more panels failed to update");
    }

    Ok(())
}
