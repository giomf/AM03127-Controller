use heapless::String;
use serde::Deserialize;

use crate::am03127::page_content::{Lagging, Leading, WaitingModeAndSpeed};

#[derive(Default, Deserialize, Debug, Clone)]
pub struct DateTimeDto {
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub month: u8,
    pub second: u8,
    pub year: u8,
    pub week: u8,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PageDto {
    pub text: String<32>,
    #[serde(default)]
    pub leading: Leading,
    #[serde(default)]
    pub lagging: Lagging,
    #[serde(default)]
    pub waiting_mode_and_speed: WaitingModeAndSpeed,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ScheduleDto {
    pub from: DateTimeDto,
    pub to: DateTimeDto,
    pub pages: heapless::Vec<char, 32>,
}
