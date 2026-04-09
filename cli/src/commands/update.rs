use std::{fs, path::Path};

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::stream::{self, StreamExt};

use crate::{
    config::Panel,
    console::{ProgressGroup, print_title},
};

const CHUNK_SIZE: usize = 4096;

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
            git_hash: FirmwareBuildInfo::read_cstr(data, FW_GIT_HASH_OFFSET, FW_GIT_HASH_LEN),
            build_date: FirmwareBuildInfo::read_cstr(data, FW_BUILD_DATE_OFFSET, FW_BUILD_DATE_LEN),
            build_time: FirmwareBuildInfo::read_cstr(data, FW_BUILD_TIME_OFFSET, FW_BUILD_TIME_LEN),
        })
    }

    fn read_cstr(data: &[u8], offset: usize, max_len: usize) -> String {
        let slice = &data[offset..(offset + max_len).min(data.len())];
        let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        String::from_utf8_lossy(&slice[..end]).into_owned()
    }
}

pub async fn run(panels: &[&Panel], firmware_path: &Path) -> Result<()> {
    let firmware = fs::read(firmware_path)
        .with_context(|| format!("failed to read '{}'", firmware_path.display()))?;
    let firmware_len = firmware.len() as u64;

    if let Some(info) = FirmwareBuildInfo::parse(&firmware) {
        print_title(&format!(
            "Firmware: {} ({} {})",
            info.git_hash, info.build_date, info.build_time,
        ));
    }

    print_title(&format!(
        "Uploading {} bytes to {} panel(s)",
        firmware_len,
        panels.len()
    ));

    let firmware = Bytes::from(firmware);

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
