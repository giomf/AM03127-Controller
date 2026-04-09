use crate::{config::Panel, console::print_title};

pub fn run(panels: &[Panel]) {
    print_title("Available Panels");
    for panel in panels {
        println!("{}: {}", panel.name, console::style(&panel.address).dim());
    }
}
