use heapless::String;
use serde::{Deserialize, Serialize};

use crate::am03127::{
    MESSAGE_STRING_SIZE,
    page_content::{Lagging, Leading, Page, WaitingModeAndSpeed},
};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DateTimeDto {
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub month: u8,
    pub second: u8,
    pub year: u8,
    pub week: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PageDto {
    pub text: String<MESSAGE_STRING_SIZE>,
    #[serde(default)]
    pub leading: Leading,
    #[serde(default)]
    pub lagging: Lagging,
    #[serde(default)]
    pub waiting_mode_and_speed: WaitingModeAndSpeed,
}

impl From<Page> for PageDto {
    fn from(page: Page) -> Self {
        PageDto {
            text: page.message,
            leading: page.leading,
            lagging: page.lagging,
            waiting_mode_and_speed: page.waiting_mode_and_speed,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ScheduleDto {
    pub from: DateTimeDto,
    pub to: DateTimeDto,
    pub pages: heapless::Vec<char, 32>,
}
