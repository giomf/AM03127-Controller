use super::{CommandAble, DEFAULT_LINE, DEFAULT_PAGE, DEFAULT_SCHEDULE};
use core::fmt::Display;

pub struct DeleteAll {}
impl CommandAble for DeleteAll {}
impl Display for DeleteAll {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "<D*>")
    }
}

pub struct DeletePage {
    line: u8,
    page_id: char,
}

impl CommandAble for DeletePage {}

impl DeletePage {
    pub fn line(mut self, line: u8) -> Self {
        self.line = line;
        self
    }

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

pub struct DeleteSchedule {
    schedule_id: char,
}

impl CommandAble for DeleteSchedule {}

impl DeleteSchedule {
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
