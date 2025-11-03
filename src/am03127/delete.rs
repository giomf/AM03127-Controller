use core::fmt::Display;

use super::{CommandAble, DEFAULT_LINE, DEFAULT_PAGE, DEFAULT_SCHEDULE};

/// Command to delete all pages and schedules from the LED panel
pub struct DeleteAll {}
impl CommandAble for DeleteAll {}
impl Display for DeleteAll {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<D*>")
    }
}

/// Command to delete a specific page from the LED panel
pub struct DeletePage {
    /// ID of the page to delete (A-Z)
    id: char,
    /// Line number (usually 1)
    line: u8,
}

impl CommandAble for DeletePage {}

impl DeletePage {
    pub fn new(id: char) -> Self {
        DeletePage {
            id,
            line: DEFAULT_LINE,
        }
    }
}

impl Display for DeletePage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<DL{}P{}>", self.line, self.id)
    }
}

impl Default for DeletePage {
    fn default() -> Self {
        Self {
            id: DEFAULT_PAGE,
            line: DEFAULT_LINE,
        }
    }
}

/// Command to delete a specific schedule from the LED panel
pub struct DeleteSchedule {
    /// ID of the schedule to delete (A-E)
    schedule_id: char,
}

impl CommandAble for DeleteSchedule {}

impl DeleteSchedule {
    /// Creates a new DeleteSchedule command
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to delete
    ///
    /// # Returns
    /// * A new DeleteSchedule instance
    pub fn new(schedule_id: char) -> Self {
        Self { schedule_id }
    }
}

impl Display for DeleteSchedule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<DT{}>", self.schedule_id)
    }
}

impl Default for DeleteSchedule {
    fn default() -> Self {
        Self {
            schedule_id: DEFAULT_SCHEDULE,
        }
    }
}
