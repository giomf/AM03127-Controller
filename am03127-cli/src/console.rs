use std::time::Duration;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

const TICK_STRINGS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""];
const TICK_INTERVAL: Duration = Duration::from_millis(80);

/// Thin wrapper around the `plain` flag that drives all output decisions.
pub struct Console {
    pub plain: bool,
}

impl Console {
    pub fn new(plain: bool) -> Self {
        Self { plain }
    }

    /// Print a bold section title. No-op in plain mode.
    pub fn print_title(&self, title: &str) {
        if !self.plain {
            println!("{}", style(title).bold());
        }
    }

    /// Create a spinner group whose entries respect the current output mode.
    pub fn spinner_group(&self) -> SpinnerGroup {
        SpinnerGroup::new(self.plain)
    }
}

/// Returned by [`SpinnerGroup::add`]; finish it once the corresponding task
/// completes.  In plain mode it simply prints to stdout.
pub enum TaskHandle {
    Fancy(ProgressBar),
    Plain,
}

impl TaskHandle {
    /// Finish the task.  `plain_msg` is used in plain mode; `fancy_msg`
    /// (which may contain ANSI styling) is used in interactive mode.
    pub fn finish(&self, plain_msg: impl Into<String>, fancy_msg: impl Into<String>) {
        match self {
            TaskHandle::Fancy(pb) => pb.finish_with_message(fancy_msg.into()),
            TaskHandle::Plain => println!("{}", plain_msg.into()),
        }
    }
}

/// A group of per-task spinners rendered together.
pub struct SpinnerGroup {
    plain: bool,
    mp: MultiProgress,
    style: ProgressStyle,
}

impl SpinnerGroup {
    fn new(plain: bool) -> Self {
        let style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(TICK_STRINGS);
        Self {
            plain,
            mp: MultiProgress::new(),
            style,
        }
    }

    /// Add an entry for `label`.  In interactive mode a spinner is shown; in
    /// plain mode the handle will simply print when finished.
    pub fn add(&self, label: impl Into<String>) -> TaskHandle {
        if self.plain {
            TaskHandle::Plain
        } else {
            let pb = self.mp.add(ProgressBar::new_spinner());
            pb.set_style(self.style.clone());
            pb.set_message(label.into());
            pb.enable_steady_tick(TICK_INTERVAL);
            TaskHandle::Fancy(pb)
        }
    }
}

/// A group of per-task byte-count progress bars rendered together.
pub struct _ProgressGroup {
    mp: MultiProgress,
    style: ProgressStyle,
    total: u64,
}

impl _ProgressGroup {
    pub fn _new(total_bytes: u64, label_width: usize) -> Self {
        let template = format!("{{msg:{label_width}}} [{{bar:40.cyan/blue}}] {{percent}}%");
        let style = ProgressStyle::with_template(&template).unwrap();
        Self {
            mp: MultiProgress::new(),
            style,
            total: total_bytes,
        }
    }

    /// Add a progress bar entry for the given label. The returned [`ProgressBar`]
    /// tracks bytes and should be finished once the upload completes.
    pub fn _add(&self, label: impl Into<String>) -> ProgressBar {
        let pb = self.mp.add(ProgressBar::new(self.total));
        pb.set_style(self.style.clone());
        pb.set_message(label.into());
        pb
    }
}
