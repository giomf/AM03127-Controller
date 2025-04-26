use crate::{
    am03127::{
        CommandAble,
        delete::{DeletePage, DeleteSchedule},
        page_content::{
            Page,
            formatting::{Clock, ColumnStart, Font},
        },
        realtime_clock::DateTime,
        schedule::Schedule,
    },
    storage::NvsStorage,
    uart::Uart,
};
use anyhow::Result;
use core::fmt::Write;
use heapless::String;

pub const DEFAULT_PANEL_ID: u8 = 1;
const LOGGER_NAME: &str = "Panel";

pub struct Panel<'a> {
    uart: Uart<'a>,
    storage: NvsStorage,
}

impl<'a> Panel<'a> {
    pub fn new(uart: Uart<'a>, storage: NvsStorage) -> Self {
        Self { uart, storage }
    }

    pub async fn init(&mut self) -> Result<()> {
        self.uart.init(DEFAULT_PANEL_ID).await?;

        Ok(())
    }

    pub async fn display_clock(&mut self, page_id: char) -> Result<()> {
        let mut message = String::<32>::new();
        write!(
            &mut message,
            "{}{}{}{}",
            Clock::Time,
            Font::Narrow,
            ColumnStart(41),
            Clock::Date
        )?;

        let page = Page::default().message(&message.as_str());
        self.set_page(page_id, page).await?;

        Ok(())
    }

    pub async fn set_clock(&mut self, date_time: DateTime) -> Result<()> {
        log::info!("{LOGGER_NAME}: Setting clock");
        let command = date_time.command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await
    }

    pub async fn set_page(&mut self, page_id: char, page: Page) -> Result<()> {
        log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", page);

        let command = page.command(DEFAULT_PANEL_ID);

        self.uart.write(command.as_bytes()).await?;
        self.storage.write(page_id, page).await?;

        Ok(())
    }

    pub async fn get_page(&mut self, page_id: char) -> Result<Option<Page>> {
        self.storage.read(page_id).await
    }

    pub async fn delete_page(&mut self, page_id: char) -> Result<()> {
        let command = DeletePage::default()
            .page_id(page_id)
            .command(DEFAULT_PANEL_ID);

        self.uart.write(command.as_bytes()).await?;
        self.storage.delete(page_id).await?;

        Ok(())
    }

    pub async fn set_schedule(&mut self, schedule_id: char, schedule: Schedule) -> Result<()> {
        let command = schedule.command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await?;
        //TODO store schedue

        Ok(())
    }

    pub async fn get_schedule(&mut self, schedule_id: char) -> Result<Option<Schedule>> {
        todo!()
    }

    pub async fn delete_schedule(&mut self, schedule_id: char) -> Result<()> {
        let command = DeleteSchedule::new(schedule_id).command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await?;
        //TODO store schedue

        Ok(())
    }
}
