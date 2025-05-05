use super::{CommandAble, DEFAULT_LINE, DEFAULT_PAGE, DEFAULT_SCHEDULE};
use core::fmt::Display;

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
    /// Line number (usually 1)
    line: u8,
    /// ID of the page to delete (A-Z)
    page_id: char,
}

impl CommandAble for DeletePage {}

impl DeletePage {
    /// Sets the line number for the delete command
    ///
    /// # Arguments
    /// * `line` - The line number
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn line(mut self, line: u8) -> Self {
        self.line = line;
        self
    }

    /// Sets the page ID for the delete command
    ///
    /// # Arguments
    /// * `page_id` - The ID of the page to delete
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn page_id(mut self, page_id: char) -> Self {
        self.page_id = page_id;
        self
    }
}

impl Display for DeletePage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<DL{}P{}>", self.line, self.page_id)
    }
}

impl Default for DeletePage {
    fn default() -> Self {
        Self {
            page_id: DEFAULT_PAGE,
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
