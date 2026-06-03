use am03127_client::PanelClient;
use am03127_commands::schedule::{Schedule, ScheduleDateTime};
use anyhow::{Context, Result, bail};
use console::style;
use time::{Date, PrimitiveDateTime, Time, macros::format_description};

use crate::{config::Panel, console::Console};

/// Parses a datetime string in `YYYY-MM-DDTHH:MM` or `YYYY-MM-DD` (midnight) format.
pub fn parse_schedule_datetime(s: &str) -> Result<ScheduleDateTime, String> {
    let dt = if s.contains('T') {
        PrimitiveDateTime::parse(s, format_description!("[year]-[month]-[day]T[hour]:[minute]"))
            .map_err(|e| format!("invalid datetime '{s}': {e} (expected YYYY-MM-DDTHH:MM)"))?
    } else {
        let date = Date::parse(s, format_description!("[year]-[month]-[day]"))
            .map_err(|e| format!("invalid date '{s}': {e} (expected YYYY-MM-DD)"))?;
        PrimitiveDateTime::new(date, Time::MIDNIGHT)
    };

    let year = dt.year();
    if !(2000..=2099).contains(&year) {
        return Err(format!("year must be between 2000 and 2099, got {year}"));
    }

    Ok(ScheduleDateTime {
        year: (year % 100) as u8,
        month: dt.month() as u8,
        day: dt.day(),
        hour: dt.hour(),
        minute: dt.minute(),
    })
}

pub async fn run(
    panels: &[&Panel],
    id: char,
    page: char,
    from: ScheduleDateTime,
    to: ScheduleDateTime,
    console: &Console,
) -> Result<()> {
    if !id.is_ascii_uppercase() {
        bail!("schedule id must be an uppercase ASCII letter (A-Z), got '{id}'");
    }
    if !page.is_ascii_uppercase() {
        bail!("page id must be an uppercase ASCII letter (A-Z), got '{page}'");
    }

    let schedule = Schedule::new(id, from, to, page.to_string());

    console.print_title(&format!("Sending schedule '{id}' to panels"));

    let label_width = super::label_width(panels);
    let spinners = console.spinner_group();
    let mut set = tokio::task::JoinSet::new();

    for panel in panels {
        let client = PanelClient::new(&panel.address);
        let name = panel.name.clone();
        let pb = spinners.add(&name);
        let schedule = schedule.clone();
        set.spawn(async move {
            let result = client.set_schedule(&schedule).await;
            (name, result, pb)
        });
    }

    let mut success = true;
    while let Some(res) = set.join_next().await {
        let (name, result, pb) = res.context("panel task panicked")?;
        match result {
            Ok(_) => pb.finish(
                format!("{name}: ok"),
                format!(
                    "{} {name:<label_width$}  schedule '{id}' sent",
                    style("✓").green()
                ),
            ),
            Err(e) => {
                pb.finish(
                    format!("{name}: error: {e}"),
                    format!("{} {name:<label_width$}  {e}", style("✗").red()),
                );
                success = false;
            }
        }
    }

    if !success {
        anyhow::bail!("one or more panels failed to receive the schedule");
    }

    Ok(())
}
