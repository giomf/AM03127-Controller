use anyhow::{Context, Result};

use crate::config::Panel;

pub async fn run(panels: &[&Panel]) -> Result<()> {
    let client = reqwest::Client::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = client.clone();
        let name = panel.name.clone();
        let url = format!("http://{}", panel.address);
        set.spawn(async move {
            let reachable = client.get(&url).send().await.is_ok();
            (name, reachable)
        });
    }

    while let Some(res) = set.join_next().await {
        let (name, reachable) = res.context("panel task panicked")?;
        if reachable {
            println!("{name}: online");
        } else {
            println!("{name}: offline");
        }
    }

    Ok(())
}
