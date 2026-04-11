use am03127_client::PanelClient;
use am03127_commands::page::{Lagging, Leading, Page, WaitingModeAndSpeed, WaitingTime};
use anyhow::{Context, Result, bail};
use console::style;

use crate::{
    config::Panel,
    console::{SpinnerGroup, print_title},
};

pub fn parse_leading(s: &str) -> Result<Leading, String> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|_| format!("unknown leading effect '{s}'"))
}

pub fn parse_lagging(s: &str) -> Result<Lagging, String> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|_| format!("unknown lagging effect '{s}'"))
}

pub async fn run(
    panels: &[&Panel],
    id: char,
    message: String,
    leading: Leading,
    lagging: Lagging,
    waiting_time: u8,
) -> Result<()> {
    if !id.is_ascii_uppercase() {
        bail!("page id must be an uppercase ASCII letter (A-Z), got '{id}'");
    }

    let page = Page::new(
        id,
        message,
        leading,
        lagging,
        WaitingModeAndSpeed::default(),
        WaitingTime::new(waiting_time),
    );

    print_title(&format!("Sending page '{id}' to panels"));

    let label_width = super::label_width(panels);
    let spinners = SpinnerGroup::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = PanelClient::new(&panel.address);
        let name = panel.name.clone();
        let pb = spinners.add(&name);
        let page = page.clone();
        set.spawn(async move {
            let result = client.set_page(&page).await;
            (name, result, pb)
        });
    }

    let mut success = true;
    while let Some(res) = set.join_next().await {
        let (name, result, pb) = res.context("panel task panicked")?;
        match result {
            Ok(_) => pb.finish_with_message(format!(
                "{} {name:<label_width$}  page '{id}' sent",
                style("✓").green(),
            )),
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
        anyhow::bail!("one or more panels failed to receive the page");
    }

    Ok(())
}
