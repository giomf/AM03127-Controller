use std::{fs, path::Path};

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::stream::{self, StreamExt};

use crate::{
    config::Panel,
    console::{ProgressGroup, print_title},
};

const CHUNK_SIZE: usize = 4096;

pub async fn run(panels: &[&Panel], firmware_path: &Path) -> Result<()> {
    let firmware = fs::read(firmware_path)
        .with_context(|| format!("failed to read '{}'", firmware_path.display()))?;
    let firmware_len = firmware.len() as u64;
    let firmware = Bytes::from(firmware);

    print_title(&format!(
        "Uploading firmware ({} bytes) to {} panel(s)",
        firmware_len,
        panels.len()
    ));

    let client = reqwest::Client::new();
    let bars = ProgressGroup::new(firmware_len);
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
            Ok(_) => pb.finish_with_message(format!("{name}: ✓ done, rebooting")),
            Err(e) => {
                pb.finish_with_message(format!("{name}: ✗ {e}"));
                success = false;
            }
        }
    }

    if !success {
        anyhow::bail!("one or more panels failed to update");
    }

    Ok(())
}
