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
    error::Error,
    storage::{
        NvsStorageSection, PAGE_STORAGE_BEGIN, PAGE_STORAGE_SIZE, SCHEDULE_STORAGE_BEGIN,
        SCHEDULE_STORAGE_SIZE,
    },
    uart::Uart,
};
use core::fmt::Write;
use heapless::{String, Vec};

const LOGGER_NAME: &str = "Panel";
const DEFAULT_PANEL_ID: u8 = 1;
const MAX_PAGES: usize = 24; // A - Z
const MAX_SCHEDULES: usize = 5; // A - E
const KEY_MEMORY_SIZE: usize = core::mem::size_of::<u8>();
const PAGE_MEMORY_SIZE: usize = core::mem::size_of::<Page>();
const SCHEDULE_MEMORY_SIZE: usize = core::mem::size_of::<Schedule>();

pub type Pages = Vec<Page, MAX_PAGES>;
pub type Schedules = Vec<Schedule, MAX_SCHEDULES>;

pub struct Panel<'a> {
    uart: Uart<'a>,
    page_storage: NvsStorageSection<Page, { KEY_MEMORY_SIZE + PAGE_MEMORY_SIZE }>,
    schedule_storage: NvsStorageSection<Schedule, { KEY_MEMORY_SIZE + SCHEDULE_MEMORY_SIZE }>,
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

    pub async fn init(&mut self) -> Result<(), Error> {
        self.uart.init(DEFAULT_PANEL_ID).await?;

        Ok(())
    }

    pub async fn display_clock(&mut self, page_id: char) -> Result<(), Error> {
        let mut message = String::<32>::new();
        write!(
            &mut message,
            "{}{}{}{}",
            Clock::Time,
            Font::Narrow,
            ColumnStart(41),
            Clock::Date
        )
        .map_err(|_| Error::Internal("Failed to write command".into()))?;

        let page = Page::default().message(&message.as_str());
        self.set_page(page_id, page).await?;

        Ok(())
    }

    pub async fn set_clock(&mut self, date_time: DateTime) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting clock");
        let command = date_time.command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await?;

        Ok(())
    }

    pub async fn set_page(&mut self, page_id: char, page: Page) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", page);

        let command = page.command(DEFAULT_PANEL_ID);

        self.uart.write(command.as_bytes()).await?;
        self.page_storage.write(page_id, page).await?;

        Ok(())
    }

    pub async fn get_page(&mut self, page_id: char) -> Result<Option<Page>, Error> {
        log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");
        self.page_storage.read(page_id).await
    }

    pub async fn get_pages(&mut self) -> Result<Pages, Error> {
        log::info!("{LOGGER_NAME}: Getting pages");
        self.page_storage.read_all().await
    }

    pub async fn delete_page(&mut self, page_id: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting page \"{page_id}\"");

        let command = DeletePage::default()
            .page_id(page_id)
            .command(DEFAULT_PANEL_ID);

        self.uart.write(command.as_bytes()).await?;
        self.page_storage.delete(page_id).await?;

        Ok(())
    }

    pub async fn set_schedule(
        &mut self,
        schedule_id: char,
        schedule: Schedule,
    ) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting schedule \"{schedule_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", schedule);

        let command = schedule.command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await?;
        self.schedule_storage.write(schedule_id, schedule).await?;

        Ok(())
    }

    pub async fn get_schedule(&mut self, schedule_id: char) -> Result<Option<Schedule>, Error> {
        log::info!("{LOGGER_NAME}: Getting schedule \"{schedule_id}\"");
        self.schedule_storage.read(schedule_id).await
    }

    pub async fn get_schedules(&mut self) -> Result<Schedules, Error> {
        log::info!("{LOGGER_NAME}: Getting schedules");
        self.schedule_storage.read_all().await
    }

    pub async fn delete_schedule(&mut self, schedule_id: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting page \"{schedule_id}\"");

        let command = DeleteSchedule::new(schedule_id).command(DEFAULT_PANEL_ID);
        self.uart.write(command.as_bytes()).await?;
        self.schedule_storage.delete(schedule_id).await?;

        Ok(())
    }
}
