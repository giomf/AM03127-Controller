#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::server::dto::DateTimeDto;

use super::CommandAble;
use core::fmt::Display;

#[derive(Default, Deserialize, Serialize)]
pub struct DateTime {
    year: u8,
    week: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl From<DateTimeDto> for DateTime {
    fn from(value: DateTimeDto) -> Self {
        DateTime::default()
            .year(value.year)
            .month(value.month)
            .week(value.week)
            .day(value.day)
            .hour(value.hour)
            .minute(value.minute)
            .second(value.second)
    }
}

impl CommandAble for DateTime {}

impl DateTime {
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
}

impl Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "<SC>{:02}{:02}{:02}{:02}{:02}{:02}{:02}",
            self.year, self.week, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}
