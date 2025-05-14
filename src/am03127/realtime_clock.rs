#![allow(dead_code)]

use super::CommandAble;
use core::fmt::Display;
use serde::{Deserialize, Serialize};

/// Represents a date and time for the LED panel's real-time clock
///
/// This struct is used to set or represent the current date and time
/// on the LED panel's internal clock.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DateTime {
    /// Year (0-99)
    year: u8,
    /// Week of the year (1-52)
    week: u8,
    /// Month (1-12)
    month: u8,
    /// Day of the month (1-31)
    day: u8,
    /// Hour (0-23)
    hour: u8,
    /// Minute (0-59)
    minute: u8,
    /// Second (0-59)
    second: u8,
}

impl CommandAble for DateTime {}

impl DateTime {
    /// Sets the year
    ///
    /// # Arguments
    /// * `year` - The year (0-99)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn year(mut self, year: u8) -> Self {
        self.year = year;
        self
    }

    /// Sets the week of the year
    ///
    /// # Arguments
    /// * `week` - The week (1-52)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn week(mut self, week: u8) -> Self {
        self.week = week;
        self
    }

    /// Sets the month
    ///
    /// # Arguments
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn month(mut self, month: u8) -> Self {
        self.month = month;
        self
    }

    /// Sets the day of the month
    ///
    /// # Arguments
    /// * `day` - The day (1-31)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn day(mut self, day: u8) -> Self {
        self.day = day;
        self
    }

    /// Sets the hour
    ///
    /// # Arguments
    /// * `hour` - The hour (0-23)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn hour(mut self, hour: u8) -> Self {
        self.hour = hour;
        self
    }

    /// Sets the minute
    ///
    /// # Arguments
    /// * `minute` - The minute (0-59)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
    pub fn minute(mut self, minute: u8) -> Self {
        self.minute = minute;
        self
    }

    /// Sets the second
    ///
    /// # Arguments
    /// * `second` - The second (0-59)
    ///
    /// # Returns
    /// * `Self` - Returns self for method chaining
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
