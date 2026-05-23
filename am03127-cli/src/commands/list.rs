use crate::{config::Panel, console::Console};

pub fn run(panels: &[Panel], console: &Console) {
    console.print_title("Available Panels");
    let label_width = panels.iter().map(|p| p.name.len()).max().unwrap_or(0);
    for panel in panels {
        if console.plain {
            println!("{:<label_width$}  {}", panel.name, panel.address);
        } else {
            println!(
                "{:<label_width$}  {}",
                panel.name,
                console::style(&panel.address).dim()
            );
        }
    }
}
