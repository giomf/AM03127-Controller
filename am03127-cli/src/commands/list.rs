use am03127_client::config::Panel;

use crate::console::Console;

pub fn run(panels: &[Panel], console: &Console) {
    console.print_title("Available Panels");
    let label_width = panels.iter().map(|p| p.name.len()).max().unwrap_or(0);
    for panel in panels {
        let hidden_tag = if panel.hidden { " [hidden]" } else { "" };
        let floor = panel
            .floor
            .as_deref()
            .map(|f| format!(" ({f})"))
            .unwrap_or_default();
        if console.plain {
            println!(
                "{:<label_width$}  {}{}{}",
                panel.name, panel.address, floor, hidden_tag
            );
        } else {
            println!(
                "{:<label_width$}  {}{}{}",
                panel.name,
                console::style(&panel.address).dim(),
                floor,
                console::style(hidden_tag).yellow()
            );
        }
    }
}
