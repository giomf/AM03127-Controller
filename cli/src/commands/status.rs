use am03127_client::PanelClient;
use anyhow::{Context, Result};

use crate::{
    config::Panel,
    console::{SpinnerGroup, print_title},
};

pub async fn run(panels: &[&Panel]) -> Result<()> {
    print_title("Checking panel status");
    let spinners = SpinnerGroup::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = PanelClient::new(&panel.address);
        let name = panel.name.clone();
        let pb = spinners.add(&name);
        set.spawn(async move {
            let result = client.get_status().await;
            (name, result, pb)
        });
    }

    while let Some(res) = set.join_next().await {
        let (name, result, pb) = res.context("panel task panicked")?;
        match result {
            Err(_) => pb.finish_with_message(format!(
                "{} {name} {}",
                console::style("✗").red(),
                console::style("offline").red(),
            )),
            Ok(info) => pb.finish_with_message(format!(
                "{} {name} {} {} {}",
                console::style("✓").green(),
                console::style("online").green(),
                console::style(&info.version).cyan(),
                console::style(format!("{} {}", info.build_date, info.build_time)).dim(),
            )),
        }
    }

    Ok(())
}
