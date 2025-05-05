use crate::{
    am03127::{page_content::Page, schedule::Schedule},
    error::Error,
};
use core::{fmt::Debug, marker::PhantomData, ops::Range};
use embassy_embedded_hal::adapter::BlockingAsync;
use esp_storage::FlashStorage;
use heapless::Vec;
use sequential_storage::{
    cache::NoCache,
    map::{self, SerializationError, Value},
};

const LOGGER_NAME: &str = "NvsStorage";

pub const PAGE_STORAGE_BEGIN: u32 = 0x9000;
pub const PAGE_STORAGE_SIZE: u32 = 0x3000;
pub const SCHEDULE_STORAGE_BEGIN: u32 = 0xc000;
pub const SCHEDULE_STORAGE_SIZE: u32 = 0x1000;

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

impl<'a> Value<'a> for Schedule {
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
            Ok(schedule) => Ok(schedule),
            Err(_) => Err(SerializationError::InvalidData),
        }
    }
}

pub struct NvsStorageSection<T, const S: usize> {
    flash: BlockingAsync<FlashStorage>,
    flash_range: Range<u32>,
    _type: PhantomData<T>,
}

impl<T: for<'a> Value<'a> + Debug, const S: usize> NvsStorageSection<T, S> {
    pub fn new(flash_begin: u32, flash_size: u32) -> Self {
        let flash = BlockingAsync::new(FlashStorage::new());
        let flash_end = flash_begin + flash_size;
        let flash_range = flash_begin..flash_end;

        NvsStorageSection {
            flash,
            flash_range,
            _type: PhantomData,
        }
    }

    pub async fn read(&mut self, key: char) -> Result<Option<T>, Error> {
        log::info!("{LOGGER_NAME}: Reading page \"{key}\"");

        let mut data_buffer = [0; S];

        let page = map::fetch_item::<u8, T, _>(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(key as u8),
        )
        .await?;

        log::debug!("{LOGGER_NAME}: read {:?}", page);
        Ok(page)
    }

    pub async fn read_all<const N: usize>(&mut self) -> Result<Vec<T, N>, Error> {
        log::info!("{LOGGER_NAME}: Reading all pages");

        let mut cache = NoCache::new();
        let mut data_buffer = [0; S];

        let mut pages_iterator = map::fetch_all_items::<u8, _, _>(
            &mut self.flash,
            self.flash_range.clone(),
            &mut cache,
            &mut data_buffer,
        )
        .await?;

        let mut pages = Vec::<T, N>::new();

        while let Some((_, page)) = pages_iterator.next::<u8, T>(&mut data_buffer).await? {
            pages.push(page).expect("Failed to fill pages");
        }
        Ok(pages)
    }

    pub async fn write(&mut self, key: char, value: T) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Writing page \"{key}\"");

        let mut data_buffer = [0; S];
        map::store_item(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(key as u8),
            &value,
        )
        .await?;

        Ok(())
    }

    pub async fn delete(&mut self, key: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting page \"{key}\"");

        let mut data_buffer = [0; S];

        map::remove_item(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(key as u8),
        )
        .await?;

        Ok(())
    }
}
