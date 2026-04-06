use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{
    config::Panel,
    console::{SpinnerGroup, print_title},
};

#[derive(Deserialize)]
struct BuildInfo {
    version: String,
    build_time: String,
    build_date: String,
}

pub async fn run(panels: &[&Panel]) -> Result<()> {
    print_title("Checking panel status");
    let client = reqwest::Client::new();
    let spinners = SpinnerGroup::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = client.clone();
        let name = panel.name.clone();
        let url = format!("http://{}/status", panel.address);
        let pb = spinners.add(&name);
        set.spawn(async move {
            let result = client.get(&url).send().await;
            (name, result, pb)
        });
    }

    while let Some(res) = set.join_next().await {
        let (name, result, pb) = res.context("panel task panicked")?;
        match result {
            Err(_) => pb.finish_with_message(format!("✗ {name}: offline")),
            Ok(response) => match response.json::<BuildInfo>().await {
                Ok(info) => pb.finish_with_message(format!(
                    "✓ {name}: online {} {} {}",
                    info.version, info.build_date, info.build_time,
                )),
                Err(_) => pb.finish_with_message(format!("✓ {name}: online")),
            },
        }
    }

    Ok(())
}
