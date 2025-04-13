#![allow(dead_code)]

use super::{DEFAULT_ID, STRING_SIZE, wrap_command};
use core::fmt::Write;
use heapless::String;

pub struct RealTimeClock {
    id: u8,
    year: u8,
    week: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl RealTimeClock {
    pub fn id(mut self, id: u8) -> Self {
        self.id = id;
        self
    }
    pub fn year(mut self, year: u8) -> Self {
        self.year = year;
        self
    }
    pub fn week(mut self, week: u8) -> Self {
        self.week = week;
        self
    }
    pub fn month(mut self, month: u8) -> Self {
        self.month = month;
        self
    }
    pub fn day(mut self, day: u8) -> Self {
        self.day = day;
        self
    }
    pub fn hour(mut self, hour: u8) -> Self {
        self.hour = hour;
        self
    }
    pub fn minute(mut self, minute: u8) -> Self {
        self.minute = minute;
        self
    }
    pub fn second(mut self, second: u8) -> Self {
        self.second = second;
        self
    }

    pub fn command(&self) -> String<STRING_SIZE> {
        let mut command = String::<STRING_SIZE>::new();
        write!(
            &mut command,
            "<SC>{:02}{:02}{:02}{:02}{:02}{:02}{:02}",
            self.year, self.week, self.month, self.day, self.hour, self.minute, self.second
        )
        .unwrap();

        wrap_command(self.id, &command)
    }
}

impl Default for RealTimeClock {
    fn default() -> Self {
        Self {
            id: DEFAULT_ID,
            year: 0,
            week: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}
