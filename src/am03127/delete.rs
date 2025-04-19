use super::{DEFAULT_ID, DEFAULT_LINE, DEFAULT_PAGE, DEFAULT_SCHEDULE, STRING_SIZE, wrap_command};
use core::fmt::Write;
use heapless::String;

pub struct DeleteAll {
    id: u8,
}

impl DeleteAll {
    pub fn new(id: u8) -> Self {
        Self { id }
    }
    
    pub fn id(mut self, id: u8) -> Self {
        self.id = id;
        self
    }
    
    pub fn command(&self) -> String<STRING_SIZE> {
        wrap_command(self.id, "<D*>")
    }
}

impl Default for DeleteAll {
    fn default() -> Self {
        Self { id: DEFAULT_ID }
    }
}

pub struct DeletePage {
    id: u8,
    line: u8,
    page: char,
}

impl DeletePage {
    pub fn new(id: u8, line: u8, page: char) -> Self {
        Self { id, line, page }
    }
    
    pub fn id(mut self, id: u8) -> Self {
        self.id = id;
        self
    }
    
    pub fn line(mut self, line: u8) -> Self {
        self.line = line;
        self
    }
    
    pub fn page(mut self, page: char) -> Self {
        self.page = page;
        self
    }
    
    pub fn command(&self) -> String<STRING_SIZE> {
        let mut buffer = String::<STRING_SIZE>::new();
        write!(&mut buffer, "<DL{}P{}", self.line, self.page).unwrap();
        wrap_command(self.id, &buffer)
    }
}

impl Default for DeletePage {
    fn default() -> Self {
        Self {
            id: DEFAULT_ID,
            line: DEFAULT_LINE,
            page: DEFAULT_PAGE,
        }
    }
}

pub struct DeleteSchedule {
    id: u8,
    schedule: char,
}

impl DeleteSchedule {
    pub fn new(id: u8, schedule: char) -> Self {
        Self { id, schedule }
    }
    
    pub fn id(mut self, id: u8) -> Self {
        self.id = id;
        self
    }
    
    pub fn schedule(mut self, schedule: char) -> Self {
        self.schedule = schedule;
        self
    }
    
    pub fn command(&self) -> String<STRING_SIZE> {
        let mut buffer = String::<STRING_SIZE>::new();
        write!(&mut buffer, "<DT{}>", self.schedule).unwrap();
        wrap_command(self.id, &buffer)
    }
}

impl Default for DeleteSchedule {
    fn default() -> Self {
        Self {
            id: DEFAULT_ID,
            schedule: DEFAULT_SCHEDULE,
        }
    }
}
