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

impl Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "<SC>{:02}{:02}{:02}{:02}{:02}{:02}{:02}",
            self.year, self.week, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}
