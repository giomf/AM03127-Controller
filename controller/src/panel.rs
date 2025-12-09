extern crate alloc;
use alloc::vec::Vec;

use am03127::{
    CommandAble,
    delete::{DeleteAll, DeletePage, DeleteSchedule},
    page::Page,
    realtime_clock::DateTime,
    schedule::Schedule,
    set_id,
};

use crate::{
    SharedStorage, SharedUart,
    error::Error,
    storage::{
        NvsStorageSection, PAGE_STORAGE_BEGIN, PAGE_STORAGE_SIZE, PageWrapper,
        SCHEDULE_STORAGE_BEGIN, SCHEDULE_STORAGE_SIZE, ScheduleWrapper,
    },
};

/// Logger name for panel-related log messages
const LOGGER_NAME: &str = "Panel";
/// Default ID for the LED panel
const DEFAULT_PANEL_ID: u8 = 1;
/// Size of a key in memory
const KEY_MEMORY_SIZE: usize = core::mem::size_of::<u8>();
/// Size of a Page struct in memory
const PAGE_MEMORY_SIZE: usize = core::mem::size_of::<Option<PageWrapper>>();
/// Size of a Schedule struct in memory
const SCHEDULE_MEMORY_SIZE: usize = core::mem::size_of::<Option<ScheduleWrapper>>();
/// Size of a estimated longest String
const ESTIMATED_STRING_SIZE: usize = 32;
/// Total size needed for a page entry (key + data)
const PAGE_ENTRY_SIZE: usize = KEY_MEMORY_SIZE + PAGE_MEMORY_SIZE + ESTIMATED_STRING_SIZE;
/// Total size needed for a schedule entry (key + data)
const SCHEDULE_ENTRY_SIZE: usize = KEY_MEMORY_SIZE + SCHEDULE_MEMORY_SIZE + ESTIMATED_STRING_SIZE;

/// Main controller for the LED panel
///
/// This struct provides high-level methods to interact with the LED panel,
/// including displaying content, managing pages and schedules, and setting the clock.
pub struct Panel {
    /// UART interface for communicating with the panel
    uart: SharedUart,
    /// Storage for pages
    page_storage: NvsStorageSection<PageWrapper, { PAGE_ENTRY_SIZE }>,
    /// Storage for schedules
    schedule_storage: NvsStorageSection<ScheduleWrapper, { SCHEDULE_ENTRY_SIZE }>,
}

impl Panel {
    /// Creates a new Panel instance
    ///
    /// # Arguments
    /// * `uart` - Shared UART interface for communicating with the LED panel
    /// * `flash_storage` - Shared flash storage for persisting pages and schedules
    ///
    /// # Returns
    /// * A new Panel instance with initialized storage
    pub fn new(uart: SharedUart, flash_storage: SharedStorage) -> Self {
        log::info!(
            "{LOGGER_NAME}: Creating page storage beginning at {PAGE_STORAGE_BEGIN} size of {PAGE_STORAGE_SIZE} and data buffer size of {PAGE_ENTRY_SIZE}"
        );
        let page_storage =
            NvsStorageSection::new(flash_storage, PAGE_STORAGE_BEGIN, PAGE_STORAGE_SIZE);
        log::info!(
            "{LOGGER_NAME}: Creating schedule storage beginning at {SCHEDULE_STORAGE_BEGIN} size of {SCHEDULE_STORAGE_SIZE} and data buffer size of {SCHEDULE_ENTRY_SIZE}"
        );
        let schedule_storage =
            NvsStorageSection::new(flash_storage, SCHEDULE_STORAGE_BEGIN, SCHEDULE_STORAGE_SIZE);
        Self {
            uart,
            page_storage,
            schedule_storage,
        }
    }

    /// Initializes the LED panel
    ///
    /// Sets the panel ID and restores all previously saved pages and schedules
    /// from flash storage to the panel.
    ///
    /// # Returns
    /// * `Ok(())` if initialization was successful
    /// * `Err(Error)` if initialization failed
    pub async fn init(&self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Initialize panel");
        let command = set_id(DEFAULT_PANEL_ID);
        self.uart.lock().await.write(&command).await?;
        self.init_pages().await?;
        self.init_schedules().await?;
        Ok(())
    }

    /// Initializes pages by loading them from storage and sending to the panel
    ///
    /// # Returns
    /// * `Ok(())` if all pages were initialized successfully
    /// * `Err(Error)` if initialization failed
    async fn init_pages(&self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Init pages");

        let pages: Vec<Page> = self
            .page_storage
            .read_all()
            .await?
            .into_iter()
            .map(|page_wrapper| page_wrapper.0)
            .collect();

        for page in pages {
            let command = page.command(DEFAULT_PANEL_ID);
            self.uart.lock().await.write(&command).await?;
        }

        Ok(())
    }

    /// Initializes schedules by loading them from storage and sending to the panel
    ///
    /// # Returns
    /// * `Ok(())` if all schedules were initialized successfully
    /// * `Err(Error)` if initialization failed
    async fn init_schedules(&self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Init schedules");

        let schedules: Vec<Schedule> = self
            .schedule_storage
            .read_all()
            .await?
            .into_iter()
            .map(|schedule_wrapper| schedule_wrapper.0)
            .collect();

        for schedule in schedules {
            let command = schedule.command(DEFAULT_PANEL_ID);
            self.uart.lock().await.write(&command).await?;
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
    pub async fn set_clock(&self, date_time: &DateTime) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting clock");
        let command = date_time.command(DEFAULT_PANEL_ID);
        self.uart.lock().await.write(&command).await?;

        Ok(())
    }

    /// Sets a page on the panel
    ///
    /// Sends the page to the LED panel and persists it to flash storage.
    ///
    /// # Arguments
    /// * `page_id` - The ID of the page to set (A-Z)
    /// * `page` - The page content
    ///
    /// # Returns
    /// * `Ok(())` if the page was set successfully
    /// * `Err(Error)` if setting the page failed
    pub async fn set_page(&self, page_id: char, page: Page) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting page \"{page_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", page);

        let command = page.command(DEFAULT_PANEL_ID);

        self.uart.lock().await.write(&command).await?;
        self.page_storage.write(page_id, PageWrapper(page)).await?;

        Ok(())
    }

    /// Retrieves a page from storage
    ///
    /// # Arguments
    /// * `page_id` - The ID of the page to retrieve (A-Z)
    ///
    /// # Returns
    /// * `Ok(Some(Page))` if the page was found
    /// * `Ok(None)` if the page doesn't exist
    /// * `Err(Error)` if retrieving the page failed
    pub async fn get_page(&self, page_id: char) -> Result<Option<Page>, Error> {
        log::info!("{LOGGER_NAME}: Getting page \"{page_id}\"");
        self.page_storage
            .read(page_id)
            .await
            .map(|opt| opt.map(|page_wrapper| page_wrapper.0))
    }

    /// Retrieves all pages from storage
    ///
    /// # Returns
    /// * `Ok(Vec<Page>)` - A vector of all stored pages
    /// * `Err(Error)` if retrieving the pages failed
    pub async fn get_pages(&self) -> Result<Vec<Page>, Error> {
        log::info!("{LOGGER_NAME}: Getting pages");
        Ok(self
            .page_storage
            .read_all()
            .await?
            .into_iter()
            .map(|page_wrapper| page_wrapper.0)
            .collect())
    }

    /// Deletes a page from the panel and storage
    ///
    /// Removes the page from both the LED panel and flash storage.
    ///
    /// # Arguments
    /// * `page_id` - The ID of the page to delete (A-Z)
    ///
    /// # Returns
    /// * `Ok(())` if the page was deleted successfully
    /// * `Err(Error)` if deleting the page failed
    pub async fn delete_page(&self, page_id: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting page \"{page_id}\"");

        let command = DeletePage::new(page_id).command(DEFAULT_PANEL_ID);

        self.uart.lock().await.write(&command).await?;
        self.page_storage.delete(page_id).await?;

        Ok(())
    }

    /// Sets a schedule on the panel
    ///
    /// Sends the schedule to the LED panel and persists it to flash storage.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to set (A-E)
    /// * `schedule` - The schedule content
    ///
    /// # Returns
    /// * `Ok(())` if the schedule was set successfully
    /// * `Err(Error)` if setting the schedule failed
    pub async fn set_schedule(&self, schedule_id: char, schedule: Schedule) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Setting schedule \"{schedule_id}\"");
        log::debug!("{LOGGER_NAME}: {:?}", schedule);

        let command = schedule.command(DEFAULT_PANEL_ID);
        self.uart.lock().await.write(&command).await?;
        self.schedule_storage
            .write(schedule_id, schedule.into())
            .await?;

        Ok(())
    }

    /// Retrieves a schedule from storage
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to retrieve (A-E)
    ///
    /// # Returns
    /// * `Ok(Some(Schedule))` if the schedule was found
    /// * `Ok(None)` if the schedule doesn't exist
    /// * `Err(Error)` if retrieving the schedule failed
    pub async fn get_schedule(&self, schedule_id: char) -> Result<Option<Schedule>, Error> {
        log::info!("{LOGGER_NAME}: Getting schedule \"{schedule_id}\"");
        self.schedule_storage
            .read(schedule_id)
            .await
            .map(|opt| opt.map(|schedule_wrapper| schedule_wrapper.0))
    }

    /// Retrieves all schedules from storage
    ///
    /// # Returns
    /// * `Ok(Vec<Schedule>)` - A vector of all stored schedules
    /// * `Err(Error)` if retrieving the schedules failed
    pub async fn get_schedules(&self) -> Result<Vec<Schedule>, Error> {
        log::info!("{LOGGER_NAME}: Getting schedules");
        Ok(self
            .schedule_storage
            .read_all()
            .await?
            .into_iter()
            .map(|schedule_wrapper| schedule_wrapper.0)
            .collect())
    }

    /// Deletes a schedule from the panel and storage
    ///
    /// Removes the schedule from both the LED panel and flash storage.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to delete (A-E)
    ///
    /// # Returns
    /// * `Ok(())` if the schedule was deleted successfully
    /// * `Err(Error)` if deleting the schedule failed
    pub async fn delete_schedule(&self, schedule_id: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting schedule \"{schedule_id}\"");

        let command = DeleteSchedule::new(schedule_id).command(DEFAULT_PANEL_ID);
        self.uart.lock().await.write(&command).await?;
        self.schedule_storage.delete(schedule_id).await?;

        Ok(())
    }

    /// Deletes all pages and schedules from the panel and storage
    ///
    /// Sends a delete all command to the LED panel and erases all
    /// pages and schedules from flash storage.
    ///
    /// # Returns
    /// * `Ok(())` if all data was deleted successfully
    /// * `Err(Error)` if deleting failed
    pub async fn delete_all(&self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting all");

        let command = DeleteAll {}.command(DEFAULT_PANEL_ID);
        self.uart.lock().await.write(&command).await?;
        self.page_storage.delete_all().await?;
        self.schedule_storage.delete_all().await?;
        Ok(())
    }
}
