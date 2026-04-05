use anyhow::{Context, Result};

use crate::{
    config::Panel,
    console::{SpinnerGroup, print_title},
};

pub async fn run(panels: &[&Panel]) -> Result<()> {
    print_title("Checking panel status");
    let client = reqwest::Client::new();
    let spinners = SpinnerGroup::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = client.clone();
        let name = panel.name.clone();
        let url = format!("http://{}", panel.address);
        dbg!(&url);
        let pb = spinners.add(&name);
        set.spawn(async move {
            let reachable = client.get(&url).send().await.is_ok();
            (name, reachable, pb)
        });
    }

    while let Some(res) = set.join_next().await {
        let (name, reachable, pb) = res.context("panel task panicked")?;
        if reachable {
            pb.finish_with_message(format!("✓ {name}: online"));
        } else {
            pb.finish_with_message(format!("✗ {name}: offline"));
        }
    }

    Ok(())
}
