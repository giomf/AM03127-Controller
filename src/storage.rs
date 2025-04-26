use crate::am03127::page_content::Page;
use anyhow::{Result, anyhow};
use core::ops::Range;
use embassy_embedded_hal::adapter::BlockingAsync;
use esp_storage::FlashStorage;
use heapless::Vec;
use sequential_storage::{
    cache::NoCache,
    map::{self, SerializationError, Value},
};

const LOGGER_NAME: &str = "NvsStorage";
const NVS_FLASH_BEGIN: u32 = 0x9000;
const NVS_FLASH_SIZE: u32 = 0x4000;
const KEY_SIZE: usize = core::mem::size_of::<u8>();
const VALUE_SIZE: usize = core::mem::size_of::<Page>();
const ENTRY_SIZE: usize = KEY_SIZE + VALUE_SIZE;

const MAX_PAGES: usize = 24;

impl<'a> Value<'a> for Page {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, map::SerializationError> {
        if buffer.len() < core::mem::size_of::<Self>() {
            return Err(SerializationError::BufferTooSmall);
        }
        match postcard::to_slice(&self, buffer) {
            Ok(used) => Ok(used.len()),
            Err(_) => Err(SerializationError::InvalidData),
        }
    }

    fn deserialize_from(buffer: &'a [u8]) -> Result<Self, map::SerializationError>
    where
        Self: Sized,
    {
        match postcard::from_bytes::<Self>(&buffer) {
            Ok(page) => Ok(page),
            Err(_) => Err(SerializationError::InvalidData),
        }
    }
}

pub struct NvsStorage {
    flash: BlockingAsync<FlashStorage>,
    flash_range: Range<u32>,
}

impl NvsStorage {
    pub fn new() -> Self {
        let flash = BlockingAsync::new(FlashStorage::new());
        let flash_end = NVS_FLASH_BEGIN + NVS_FLASH_SIZE;
        let flash_range = NVS_FLASH_BEGIN..flash_end;

        log::info!(
            "{LOGGER_NAME}: initizialsing flash from {} to {}",
            NVS_FLASH_BEGIN,
            flash_end
        );

        NvsStorage { flash, flash_range }
    }

    pub async fn read(&mut self, page_id: char) -> Result<Option<Page>> {
        log::info!("{LOGGER_NAME}: Reading page \"{page_id}\"");

        let mut data_buffer = [0; ENTRY_SIZE];
        let page = map::fetch_item::<u8, Page, _>(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(page_id as u8),
        )
        .await
        .map_err(|_| anyhow!("Failed to read page from storage"))?;

        log::debug!("{LOGGER_NAME}: read {:?}", page);
        Ok(page)
    }

    pub async fn read_all(&mut self) -> Result<Vec<Page, MAX_PAGES>> {
        log::info!("{LOGGER_NAME}: Reading all pages");

        let mut cache = NoCache::new();

        let mut data_buffer = [0; ENTRY_SIZE];
        let mut pages_iterator = map::fetch_all_items::<u8, _, _>(
            &mut self.flash,
            self.flash_range.clone(),
            &mut cache,
            &mut data_buffer,
        )
        .await
        .map_err(|_| anyhow!("Failed to read page from storage"))?;

        let mut pages = Vec::<Page, MAX_PAGES>::new();

        while let Some((_, page)) = pages_iterator
            .next::<u8, Page>(&mut data_buffer)
            .await
            .map_err(|_| anyhow!("Failed to read page from storage"))?
        {
            pages.push(page).expect("Failed to fill pages");
        }
        Ok(pages)
    }

    pub async fn write(&mut self, page_id: char, page: Page) -> Result<()> {
        log::info!("{LOGGER_NAME}: Writing page \"{page_id}\"");

        let mut data_buffer = [0; ENTRY_SIZE];
        map::store_item(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(page_id as u8),
            &page,
        )
        .await
        .map_err(|_| anyhow!("Failed to write page to storage"))?;

        Ok(())
    }

    pub async fn delete(&mut self, page_id: char) -> Result<()> {
        log::info!("{LOGGER_NAME}: Deleting page \"{page_id}\"");

        let mut data_buffer = [0; ENTRY_SIZE];
        map::remove_item(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(page_id as u8),
        )
        .await
        .map_err(|_| anyhow!("Failed to delete page from storage"))?;

        Ok(())
    }
}
