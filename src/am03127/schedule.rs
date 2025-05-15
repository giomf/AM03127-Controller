use super::{CommandAble, DEFAULT_SCHEDULE};
use core::fmt::Display;
use heapless::String;
use serde::{Deserialize, Serialize};

/// Maximum number of characters allowed in the pages field
const MAX_SCHEDULES_PAGES: usize = 31;

/// Represents a schedule for displaying pages on the LED panel
///
/// A schedule defines when specific pages should be displayed based on time ranges.
/// Each schedule has an ID, a start time, an end time, and a list of page IDs to display.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Schedule {
    /// Unique identifier for the schedule (A-Z)
    pub id: char,
    /// Start time for the schedule
    from: ScheduleDateTime,
    /// End time for the schedule
    to: ScheduleDateTime,
    /// List of page IDs to display during this schedule
    pages: String<MAX_SCHEDULES_PAGES>,
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
    pub fn from(mut self, from: ScheduleDateTime) -> Self {
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
    pub fn to(mut self, to: ScheduleDateTime) -> Self {
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
    pub fn pages(mut self, pages: String<MAX_SCHEDULES_PAGES>) -> Self {
        self.pages = pages;
        self
    }
}

impl Display for Schedule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<T{}>{}{}{}", self.id, self.from, self.to, self.pages)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ScheduleDateTime {
    /// Year (0-99)
    year: u8,
    /// Month (1-12)
    month: u8,
    /// Day of the month (1-31)
    day: u8,
    /// Hour (0-23)
    hour: u8,
    /// Minute (0-59)
    minute: u8,
}

impl Display for ScheduleDateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02}{:02}{:02}{:02}{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
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
