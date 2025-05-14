use super::{CommandAble, DEFAULT_SCHEDULE, realtime_clock::DateTime};
use core::fmt::Display;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

/// Maximum number of characters allowed in the pages field
const PAGES_MAX_CHARS: usize = 32;

/// Represents a schedule for displaying pages on the LED panel
///
/// A schedule defines when specific pages should be displayed based on time ranges.
/// Each schedule has an ID, a start time, an end time, and a list of page IDs to display.
#[derive(Debug, Deserialize, Serialize)]
pub struct Schedule {
    /// Unique identifier for the schedule (A-Z)
    pub id: char,
    /// Start time for the schedule
    from: DateTime,
    /// End time for the schedule
    to: DateTime,
    /// List of page IDs to display during this schedule
    pages: Vec<char, PAGES_MAX_CHARS>,
}

impl CommandAble for Schedule {}

impl Schedule {
    /// Sets the schedule ID
    ///
    /// # Arguments
    /// * `schedule_id` - A character identifier for the schedule (A-Z)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn id(mut self, schedule_id: char) -> Self {
        self.id = schedule_id;
        self
    }

    /// Sets the start time for the schedule
    ///
    /// # Arguments
    /// * `from` - The DateTime when the schedule should start
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn from(mut self, from: DateTime) -> Self {
        self.from = from;
        self
    }

    /// Sets the end time for the schedule
    ///
    /// # Arguments
    /// * `to` - The DateTime when the schedule should end
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn to(mut self, to: DateTime) -> Self {
        self.to = to;
        self
    }

    /// Sets the pages to be displayed during this schedule
    ///
    /// # Arguments
    /// * `pages` - Vector of page IDs to display
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn pages(mut self, pages: Vec<char, PAGES_MAX_CHARS>) -> Self {
        self.pages = pages;
        self
    }
}

impl Display for Schedule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut schedule = String::<PAGES_MAX_CHARS>::new();
        for page_id in &self.pages {
            schedule.push(page_id.clone()).unwrap();
        }
        write!(f, "<T{}>{}{}{}", self.id, self.from, self.to, schedule)
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            id: DEFAULT_SCHEDULE,
            from: Default::default(),
            to: Default::default(),
            pages: Default::default(),
        }
    }
}
