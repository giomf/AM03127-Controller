use crate::{
    am03127::{
        CommandAble,
        delete::{DeleteAll, DeletePage, DeleteSchedule},
        page_content::Page,
        realtime_clock::DateTime,
        schedule::Schedule,
        set_id,
    },
    error::Error,
    storage::{
        NvsStorageSection, PAGE_STORAGE_BEGIN, PAGE_STORAGE_SIZE, SCHEDULE_STORAGE_BEGIN,
        SCHEDULE_STORAGE_SIZE,
    },
    uart::Uart,
};
use heapless::Vec;

/// Logger name for panel-related log messages
const LOGGER_NAME: &str = "Panel";
/// Default ID for the LED panel
const DEFAULT_PANEL_ID: u8 = 1;
/// Maximum number of pages that can be stored (A-Z)
/// Has to be a power of 2
const MAX_PAGES: usize = 32;
/// Maximum number of schedules that can be stored (A-E)
/// Has to be a power of 2
const MAX_SCHEDULES: usize = 8;
/// Size of a key in memory
const KEY_MEMORY_SIZE: usize = core::mem::size_of::<u8>();
/// Size of a Page struct in memory
const PAGE_MEMORY_SIZE: usize = core::mem::size_of::<Page>();
/// Size of a Schedule struct in memory
const SCHEDULE_MEMORY_SIZE: usize = core::mem::size_of::<Schedule>();
/// Total size needed for a page entry (key + data)
const PAGE_ENTRY_SIZE: usize = KEY_MEMORY_SIZE + PAGE_MEMORY_SIZE;
/// Total size needed for a schedule entry (key + data)
const SCHEDULE_ENTRY_SIZE: usize = KEY_MEMORY_SIZE + SCHEDULE_MEMORY_SIZE;

/// Type alias for a collection of pages
pub type Pages = Vec<Page, MAX_PAGES>;
/// Type alias for a collection of schedules
pub type Schedules = Vec<Schedule, MAX_SCHEDULES>;

/// Main controller for the LED panel
///
/// This struct provides high-level methods to interact with the LED panel,
/// including displaying content, managing pages and schedules, and setting the clock.
pub struct Panel<'a> {
    /// UART interface for communicating with the panel
    uart: Uart<'a>,
    /// Storage for pages
    page_storage: NvsStorageSection<Page, { PAGE_ENTRY_SIZE }>,
    /// Storage for schedules
    schedule_storage: NvsStorageSection<Schedule, { SCHEDULE_ENTRY_SIZE }>,
}

impl<'a> Panel<'a> {
    /// Creates a new Panel instance
    ///
    /// # Arguments
    /// * `uart` - UART interface for communicating with the LED panel
    ///
    /// # Returns
    /// * A new Panel instance with initialized storage
    pub fn new(uart: Uart<'a>) -> Self {
        log::info!(
            "{LOGGER_NAME}: Creating page storage beginning at {PAGE_STORAGE_BEGIN} size of {PAGE_STORAGE_SIZE} and data buffer size of {PAGE_ENTRY_SIZE}"
        );
        let page_storage = NvsStorageSection::new(PAGE_STORAGE_BEGIN, PAGE_STORAGE_SIZE);
        log::info!(
            "{LOGGER_NAME}: Creating schedule storage beginning at {SCHEDULE_STORAGE_BEGIN} size of {SCHEDULE_STORAGE_SIZE} and data buffer size of {SCHEDULE_ENTRY_SIZE}"
        );
        let schedule_storage =
            NvsStorageSection::new(SCHEDULE_STORAGE_BEGIN, SCHEDULE_STORAGE_SIZE);
        Self {
            uart,
            page_storage,
            schedule_storage,
        }
    }

    /// Initializes the LED panel
    ///
    /// # Returns
    /// * `Ok(())` if initialization was successful
    /// * `Err(Error)` if initialization failed
    pub async fn init(&mut self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Initialize panel");
        let command = set_id(DEFAULT_PANEL_ID);
        self.uart.write(command).await?;
        self.init_pages().await?;
        self.init_schedules().await?;
        Ok(())
    }

    async fn init_pages(&mut self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Init pages");

        let pages: Pages = self.page_storage.read_all().await?;
        for page in pages {
            let command = page.command(DEFAULT_PANEL_ID);
            self.uart.write(command).await?;
        }

        Ok(())
    }

    async fn init_schedules(&mut self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Init schedules");

        let schedules: Schedules = self.schedule_storage.read_all().await?;
        for schedule in schedules {
            let command = schedule.command(DEFAULT_PANEL_ID);
            self.uart.write(command).await?;
        }

        Ok(())
    }

    /// Sets the panel's internal clock
    ///
    /// # Arguments
    /// * `date_time` - The date and time to set
    ///
    /// # Returns
    /// * `Ok(())` if the clock was set successfully
    /// * `Err(Error)` if setting the clock failed
    pub async fn set_clock(&mut self, date_time: DateTime) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting clock");
        let command = date_time.command(DEFAULT_PANEL_ID);
        self.uart.write(command).await?;

        Ok(())
    }

    /// Sets a page on the panel
    ///
    /// # Arguments
    /// * `page_id` - The ID of the page to set
    /// * `page` - The page content
    ///
    /// # Returns
    /// * `Ok(())` if the page was set successfully
    /// * `Err(Error)` if setting the page failed
    pub async fn set_page(&mut self, page_id: char, page: Page) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", page);

        let command = page.command(DEFAULT_PANEL_ID);

        self.uart.write(command).await?;
        self.page_storage.write(page_id, page).await?;

        Ok(())
    }

    /// Retrieves a page from storage
    ///
    /// # Arguments
    /// * `page_id` - The ID of the page to retrieve
    ///
    /// # Returns
    /// * `Ok(Some(Page))` if the page was found
    /// * `Ok(None)` if the page doesn't exist
    /// * `Err(Error)` if retrieving the page failed
    pub async fn get_page(&mut self, page_id: char) -> Result<Option<Page>, Error> {
        log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");
        self.page_storage.read(page_id).await
    }

    /// Retrieves all pages from storage
    ///
    /// # Returns
    /// * `Ok(Pages)` - A vector of all pages
    /// * `Err(Error)` if retrieving the pages failed
    pub async fn get_pages(&mut self) -> Result<Pages, Error> {
        log::info!("{LOGGER_NAME}: Getting pages");
        self.page_storage.read_all().await
    }

    /// Deletes a page from the panel and storage
    ///
    /// # Arguments
    /// * `page_id` - The ID of the page to delete
    ///
    /// # Returns
    /// * `Ok(())` if the page was deleted successfully
    /// * `Err(Error)` if deleting the page failed
    pub async fn delete_page(&mut self, page_id: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting page \"{page_id}\"");

        let command = DeletePage::new(page_id).command(DEFAULT_PANEL_ID);

        self.uart.write(command).await?;
        self.page_storage.delete(page_id).await?;

        Ok(())
    }

    /// Sets a schedule on the panel
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to set
    /// * `schedule` - The schedule content
    ///
    /// # Returns
    /// * `Ok(())` if the schedule was set successfully
    /// * `Err(Error)` if setting the schedule failed
    pub async fn set_schedule(
        &mut self,
        schedule_id: char,
        schedule: Schedule,
    ) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting schedule \"{schedule_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", schedule);

        let command = schedule.command(DEFAULT_PANEL_ID);
        self.uart.write(command).await?;
        self.schedule_storage.write(schedule_id, schedule).await?;

        Ok(())
    }

    /// Retrieves a schedule from storage
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to retrieve
    ///
    /// # Returns
    /// * `Ok(Some(Schedule))` if the schedule was found
    /// * `Ok(None)` if the schedule doesn't exist
    /// * `Err(Error)` if retrieving the schedule failed
    pub async fn get_schedule(&mut self, schedule_id: char) -> Result<Option<Schedule>, Error> {
        log::info!("{LOGGER_NAME}: Getting schedule \"{schedule_id}\"");
        self.schedule_storage.read(schedule_id).await
    }

    /// Retrieves all schedules from storage
    ///
    /// # Returns
    /// * `Ok(Schedules)` - A vector of all schedules
    /// * `Err(Error)` if retrieving the schedules failed
    pub async fn get_schedules(&mut self) -> Result<Schedules, Error> {
        log::info!("{LOGGER_NAME}: Getting schedules");
        self.schedule_storage.read_all().await
    }

    /// Deletes a schedule from the panel and storage
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to delete
    ///
    /// # Returns
    /// * `Ok(())` if the schedule was deleted successfully
    /// * `Err(Error)` if deleting the schedule failed
    pub async fn delete_schedule(&mut self, schedule_id: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting page \"{schedule_id}\"");

        let command = DeleteSchedule::new(schedule_id).command(DEFAULT_PANEL_ID);
        self.uart.write(command).await?;
        self.schedule_storage.delete(schedule_id).await?;

        Ok(())
    }

    pub async fn delete_all(&mut self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting all");

        let command = DeleteAll {}.command(DEFAULT_PANEL_ID);
        self.uart.write(command).await?;
        self.page_storage.delete_all().await?;
        self.schedule_storage.delete_all().await?;
        Ok(())
    }
}
