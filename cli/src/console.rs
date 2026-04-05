use std::time::Duration;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

const TICK_STRINGS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""];
const TICK_INTERVAL: Duration = Duration::from_millis(80);

/// Prints a bold section title to stdout.
pub fn print_title(title: &str) {
    println!("{}", style(title).bold());
}

/// A group of per-task spinners rendered together.
pub struct SpinnerGroup {
    mp: MultiProgress,
    style: ProgressStyle,
}

impl SpinnerGroup {
    pub fn new() -> Self {
        let style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(TICK_STRINGS);
        Self {
            mp: MultiProgress::new(),
            style,
        }
    }

    /// Add a spinning entry with the given label. The returned [`ProgressBar`]
    /// should be finished (e.g. via [`ProgressBar::finish_with_message`]) once
    /// the task completes.
    pub fn add(&self, label: impl Into<String>) -> ProgressBar {
        let pb = self.mp.add(ProgressBar::new_spinner());
        pb.set_style(self.style.clone());
        pb.set_message(label.into());
        pb.enable_steady_tick(TICK_INTERVAL);
        pb
    }
}
