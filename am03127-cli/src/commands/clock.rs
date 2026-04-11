use am03127_client::PanelClient;
use am03127_commands::realtime_clock::DateTime;
use anyhow::{Context, Result};
use console::style;
use time::OffsetDateTime;

use crate::{
    config::Panel,
    console::{SpinnerGroup, print_title},
};

fn datetime_from_offset(dt: OffsetDateTime) -> DateTime {
    DateTime {
        year: (dt.year() % 100) as u8,
        week: dt.iso_week(),
        month: dt.month() as u8,
        day: dt.day(),
        hour: dt.hour(),
        minute: dt.minute(),
        second: dt.second(),
    }
}

pub async fn run(panels: &[&Panel]) -> Result<()> {
    let now = OffsetDateTime::now_local().context("failed to get local time")?;
    let dt = datetime_from_offset(now);

    print_title(&format!(
        "Setting clock to {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        2000u16 + dt.year as u16,
        dt.month,
        dt.day,
        dt.hour,
        dt.minute,
        dt.second,
    ));

    let label_width = super::label_width(panels);
    let spinners = SpinnerGroup::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = PanelClient::new(&panel.address);
        let name = panel.name.clone();
        let pb = spinners.add(&name);
        set.spawn(async move {
            let result = client.set_clock(&dt).await;
            (name, result, pb)
        });
    }

    let mut success = true;
    while let Some(res) = set.join_next().await {
        let (name, result, pb) = res.context("panel task panicked")?;
        match result {
            Ok(_) => pb.finish_with_message(format!(
                "{} {name:<label_width$}  clock updated",
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
        anyhow::bail!("one or more panels failed to set clock");
    }

    Ok(())
}
