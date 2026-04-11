use crate::{config::Panel, console::print_title};

pub fn run(panels: &[Panel]) {
    print_title("Available Panels");
    let label_width = panels.iter().map(|p| p.name.len()).max().unwrap_or(0);
    for panel in panels {
        println!(
            "{:<label_width$}  {}",
            panel.name,
            console::style(&panel.address).dim()
        );
    }
}
