use am03127_client::{PanelClient, config::Panel};
use anyhow::{Context, Result};

use crate::console::Console;

pub async fn run(panels: &[&Panel], console: &Console) -> Result<()> {
    console.print_title("Checking panel status");
    let label_width = super::label_width(panels);
    let spinners = console.spinner_group();
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
            Err(_) => pb.finish(
                format!("{name}: offline"),
                format!(
                    "{} {name:<label_width$}  {}",
                    console::style("✗").red(),
                    console::style("offline").red(),
                ),
            ),
            Ok(info) => {
                let details = info.details().join(", ");
                let (mark, state) = match info.panel_ok {
                    Some(false) => (console::style("!").yellow(), console::style("online").yellow()),
                    _ => (console::style("✓").green(), console::style("online").green()),
                };
                pb.finish(
                    format!(
                        "{name}: online {} {} {} {details}",
                        info.version, info.build_date, info.build_time
                    ),
                    format!(
                        "{mark} {name:<label_width$}  {state} {} {}  {}",
                        console::style(&info.version).cyan(),
                        console::style(format!("{} {}", info.build_date, info.build_time)).dim(),
                        details,
                    ),
                )
            }
        }
    }

    Ok(())
}
