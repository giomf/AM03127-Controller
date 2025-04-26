use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

use crate::server::dto::ScheduleDto;

use super::{CommandAble, DEFAULT_SCHEDULE, realtime_clock::DateTime};
use core::fmt::Display;

const PAGES_MAX_CHARS: usize = 32;

#[derive(Debug, Deserialize, Serialize)]
pub struct Schedule {
    id: char,
    from: DateTime,
    to: DateTime,
    pages: Vec<char, PAGES_MAX_CHARS>,
}

impl CommandAble for Schedule {}

impl Schedule {
    pub fn id(mut self, schedule_id: char) -> Self {
        self.id = schedule_id;
        self
    }
    pub fn from(mut self, from: DateTime) -> Self {
        self.from = from;
        self
    }
    pub fn to(mut self, to: DateTime) -> Self {
        self.to = to;
        self
    }
    pub fn pages(mut self, pages: Vec<char, PAGES_MAX_CHARS>) -> Self {
        self.pages = pages;
        self
    }

    pub fn from_dto_with_id(dto: ScheduleDto, id: char) -> Self {
        Self::default()
            .id(id)
            .from(dto.from.into())
            .to(dto.to.into())
            .pages(dto.pages)
    }
}

impl Display for Schedule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut schedule = String::<PAGES_MAX_CHARS>::new();
        for page_id in &self.pages {
            schedule.push(page_id.clone()).unwrap();
        }
        write!(f, "<T{}>{}{}{}", self.id, self.from, self.to, schedule)
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            id: DEFAULT_SCHEDULE,
            from: Default::default(),
            to: Default::default(),
            pages: Default::default(),
        }
    }
}
