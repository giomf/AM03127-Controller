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
    storage::{
        NvsStorageSection, PAGE_STORAGE_BEGIN, PAGE_STORAGE_SIZE, SCHEDULE_STORAGE_BEGIN,
        SCHEDULE_STORAGE_SIZE,
    },
    uart::Uart,
};
use anyhow::Result;
use core::fmt::Write;
use heapless::{String, Vec};

const LOGGER_NAME: &str = "Panel";
const DEFAULT_PANEL_ID: u8 = 1;
const MAX_PAGES: usize = 24; // A - Z
const MAX_SCHEDULES: usize = 5; // A - E

pub struct Panel<'a> {
    uart: Uart<'a>,
    page_storage: NvsStorageSection<Page>,
    schedule_storage: NvsStorageSection<Schedule>,
}

impl<'a> Panel<'a> {
    pub fn new(uart: Uart<'a>) -> Self {
        let page_storage = NvsStorageSection::new(PAGE_STORAGE_BEGIN, PAGE_STORAGE_SIZE);
        let schedule_storage =
            NvsStorageSection::new(SCHEDULE_STORAGE_BEGIN, SCHEDULE_STORAGE_SIZE);
        Self {
            uart,
            page_storage,
            schedule_storage,
        }
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
        self.uart.write(command.as_bytes()).await?;

        Ok(())
    }

    pub async fn set_page(&mut self, page_id: char, page: Page) -> Result<()> {
        log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", page);

        let command = page.command(DEFAULT_PANEL_ID);

        self.uart.write(command.as_bytes()).await?;
        self.page_storage.write(page_id, page).await?;

        Ok(())
    }

    pub async fn get_page(&mut self, page_id: char) -> Result<Option<Page>> {
        log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");
        self.page_storage.read(page_id).await
    }

    pub async fn get_pages(&mut self) -> Result<Vec<Page, MAX_PAGES>> {
        log::info!("{LOGGER_NAME}: Getting pages");
        self.page_storage.read_all().await
    }

    pub async fn delete_page(&mut self, page_id: char) -> Result<()> {
        log::info!("{LOGGER_NAME}: Deleting page \"{page_id}\"");

        let command = DeletePage::default()
            .page_id(page_id)
            .command(DEFAULT_PANEL_ID);

        self.uart.write(command.as_bytes()).await?;
        self.page_storage.delete(page_id).await?;

        Ok(())
    }

    pub async fn set_schedule(&mut self, schedule_id: char, schedule: Schedule) -> Result<()> {
        log::info!("{LOGGER_NAME}: Setting schedule \"{schedule_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", schedule);

        let command = schedule.command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await?;
        self.schedule_storage.write(schedule_id, schedule).await?;

        Ok(())
    }

    pub async fn get_schedule(&mut self, schedule_id: char) -> Result<Option<Schedule>> {
        log::info!("{LOGGER_NAME}: Getting schedule \"{schedule_id}\"");
        self.schedule_storage.read(schedule_id).await
    }

    pub async fn get_schedules(&mut self) -> Result<Vec<Schedule, MAX_SCHEDULES>> {
        log::info!("{LOGGER_NAME}: Getting schedules");
        self.schedule_storage.read_all().await
    }

    pub async fn delete_schedule(&mut self, schedule_id: char) -> Result<()> {
        log::info!("{LOGGER_NAME}: Deleting page \"{schedule_id}\"");

        let command = DeleteSchedule::new(schedule_id).command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await?;
        self.schedule_storage.delete(schedule_id).await?;

        Ok(())
    }
}
