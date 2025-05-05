use heapless::String;
use serde::{Deserialize, Serialize};

use crate::am03127::{
    MESSAGE_STRING_SIZE,
    page_content::{Lagging, Leading, Page, WaitingModeAndSpeed},
};

/// Data transfer object for date and time information
///
/// This struct is used to transfer date and time information between
/// the API and the application.
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DateTimeDto {
    /// Day of the month (1-31)
    pub day: u8,
    /// Hour (0-23)
    pub hour: u8,
    /// Minute (0-59)
    pub minute: u8,
    /// Month (1-12)
    pub month: u8,
    /// Second (0-59)
    pub second: u8,
    /// Year (0-99)
    pub year: u8,
    /// Week of the year (1-52)
    pub week: u8,
}

/// Data transfer object for page information
///
/// This struct is used to transfer page information between
/// the API and the application.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PageDto {
    /// Text content of the page
    pub text: String<MESSAGE_STRING_SIZE>,
    /// Leading effect for the page (defaults to Immediate)
    #[serde(default)]
    pub leading: Leading,
    /// Lagging effect for the page (defaults to Hold)
    #[serde(default)]
    pub lagging: Lagging,
    /// Waiting mode and speed for the page (defaults to FastestNormal)
    #[serde(default)]
    pub waiting_mode_and_speed: WaitingModeAndSpeed,
}

impl From<Page> for PageDto {
    /// Converts a Page to a PageDto
    ///
    /// # Arguments
    /// * `page` - The Page to convert
    ///
    /// # Returns
    /// * A new PageDto instance
    fn from(page: Page) -> Self {
        PageDto {
            text: page.message,
            leading: page.leading,
            lagging: page.lagging,
            waiting_mode_and_speed: page.waiting_mode_and_speed,
        }
    }
}

/// Data transfer object for schedule information
///
/// This struct is used to transfer schedule information between
/// the API and the application.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ScheduleDto {
    /// Start time for the schedule
    pub from: DateTimeDto,
    /// End time for the schedule
    pub to: DateTimeDto,
    /// List of page IDs to display during this schedule
    pub pages: heapless::Vec<char, 32>,
}
