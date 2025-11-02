extern crate alloc;
use crate::{
    SharedStorage,
    am03127::{page_content::Page, schedule::Schedule},
    error::Error,
};
use alloc::vec::Vec;
use core::{fmt::Debug, marker::PhantomData, ops::Range};
use embassy_embedded_hal::adapter::BlockingAsync;
use sequential_storage::{
    cache::NoCache,
    erase_all,
    map::{self, SerializationError, Value},
};

/// Logger name for storage-related log messages
const LOGGER_NAME: &str = "NvsStorage";
/// Starting address for page storage in flash memory
pub const PAGE_STORAGE_BEGIN: u32 = 0x9000;
/// Size of the page storage area in flash memory
pub const PAGE_STORAGE_SIZE: u32 = 0x3000;
/// Starting address for schedule storage in flash memory
pub const SCHEDULE_STORAGE_BEGIN: u32 = 0xc000;
/// Size of the schedule storage area in flash memory
pub const SCHEDULE_STORAGE_SIZE: u32 = 0x3000;

pub trait IdAble {
    fn get_id(&self) -> char;
}

impl IdAble for Page {
    fn get_id(&self) -> char {
        self.id
    }
}

impl IdAble for Schedule {
    fn get_id(&self) -> char {
        self.id
    }
}

/// Implementation of Value trait for Page to enable serialization/deserialization
impl<'a> Value<'a> for Page {
    /// Serializes a Page into a byte buffer
    ///
    /// # Arguments
    /// * `buffer` - The buffer to serialize into
    ///
    /// # Returns
    /// * `Ok(usize)` - The number of bytes written
    /// * `Err(SerializationError)` - If serialization failed
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, map::SerializationError> {
        if buffer.len() < core::mem::size_of::<Self>() {
            return Err(SerializationError::BufferTooSmall);
        }
        match postcard::to_slice(&self, buffer) {
            Ok(used) => Ok(used.len()),
            Err(_) => Err(SerializationError::InvalidData),
        }
    }

    /// Deserializes a Page from a byte buffer
    ///
    /// # Arguments
    /// * `buffer` - The buffer containing serialized data
    ///
    /// # Returns
    /// * `Ok(Self)` - The deserialized Page
    /// * `Err(SerializationError)` - If deserialization failed
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

/// Implementation of Value trait for Schedule to enable serialization/deserialization
impl<'a> Value<'a> for Schedule {
    /// Serializes a Schedule into a byte buffer
    ///
    /// # Arguments
    /// * `buffer` - The buffer to serialize into
    ///
    /// # Returns
    /// * `Ok(usize)` - The number of bytes written
    /// * `Err(SerializationError)` - If serialization failed
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, map::SerializationError> {
        if buffer.len() < core::mem::size_of::<Self>() {
            return Err(SerializationError::BufferTooSmall);
        }
        match postcard::to_slice(&self, buffer) {
            Ok(used) => Ok(used.len()),
            Err(_) => Err(SerializationError::InvalidData),
        }
    }

    /// Deserializes a Schedule from a byte buffer
    ///
    /// # Arguments
    /// * `buffer` - The buffer containing serialized data
    ///
    /// # Returns
    /// * `Ok(Self)` - The deserialized Schedule
    /// * `Err(SerializationError)` - If deserialization failed
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

/// Storage section for persistent data in flash memory
///
/// This generic struct provides methods to read, write, and delete items
/// from a specific section of flash memory.
pub struct NvsStorageSection<T, const S: usize> {
    /// Flash storage driver
    flash: SharedStorage,
    /// Range of flash memory addresses for this section
    flash_range: Range<u32>,
    /// Phantom data to track the type stored in this section
    _type: PhantomData<T>,
}

impl<T: for<'a> Value<'a> + IdAble + Clone + Debug, const S: usize>
    NvsStorageSection<Option<T>, S>
{
    /// Creates a new storage section in flash memory
    ///
    /// # Arguments
    /// * `flash_begin` - Starting address of the flash section
    /// * `flash_size` - Size of the flash section in bytes
    ///
    /// # Returns
    /// * A new NvsStorageSection instance
    pub fn new(flash_storage: SharedStorage, flash_begin: u32, flash_size: u32) -> Self {
        let flash_end = flash_begin + flash_size;
        let flash_range = flash_begin..flash_end;

        NvsStorageSection {
            flash: flash_storage,
            flash_range,
            _type: PhantomData,
        }
    }

    /// Reads an item from storage by its key
    ///
    /// # Arguments
    /// * `key` - Character key to identify the item
    ///
    /// # Returns
    /// * `Ok(Some(T))` - The item if found
    /// * `Ok(None)` - If the item doesn't exist
    /// * `Err(Error)` - If reading failed
    pub async fn read(&self, key: char) -> Result<Option<T>, Error> {
        log::info!("{LOGGER_NAME}: Reading \"{key}\"");

        let mut data_buffer = [0; S];
        let flash = &mut *self.flash.lock().await;
        let mut flash = BlockingAsync::new(flash);

        let page = map::fetch_item::<u8, Option<T>, _>(
            &mut flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(key as u8),
        )
        .await?;

        match page {
            Some(page) => {
                log::debug!("{LOGGER_NAME}: read {:?}", page);
                Ok(page)
            }
            None => Ok(None),
        }
    }

    /// Reads all items from storage
    ///
    /// # Type Parameters
    /// * `N` - Maximum number of items to read
    ///
    /// # Returns
    /// * `Ok(Vec<T, N>)` - Vector of all items
    /// * `Err(Error)` - If reading failed
    pub async fn read_all(&self) -> Result<Vec<T>, Error> {
        log::info!("{LOGGER_NAME}: Reading all");

        let mut cache = NoCache::new();
        let mut data_buffer = [0; S];

        let flash = &mut *self.flash.lock().await;
        let mut flash = BlockingAsync::new(flash);

        let mut item_iterator = map::fetch_all_items::<u8, _, _>(
            &mut flash,
            self.flash_range.clone(),
            &mut cache,
            &mut data_buffer,
        )
        .await?;

        let mut values = Vec::new();
        while let Some((_, item)) = item_iterator.next::<Option<T>>(&mut data_buffer).await? {
            if let Some(valid_item) = item {
                values.push(valid_item);
            }
        }
        Ok(values)
    }

    /// Writes an item to storage
    ///
    /// # Arguments
    /// * `key` - Character key to identify the item
    /// * `value` - The item to write
    ///
    /// # Returns
    /// * `Ok(())` - If writing was successful
    /// * `Err(Error)` - If writing failed
    pub async fn write(&self, key: char, value: T) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Writing \"{key}\"");

        let mut data_buffer = [0; S];
        let flash = &mut *self.flash.lock().await;
        let mut flash = BlockingAsync::new(flash);

        map::store_item(
            &mut flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(key as u8),
            &Some(value),
        )
        .await?;

        Ok(())
    }

    /// Deletes an item from storage
    ///
    /// # Arguments
    /// * `key` - Character key identifying the item to delete
    ///
    /// # Returns
    /// * `Ok(())` - If deletion was successful
    /// * `Err(Error)` - If deletion failed
    pub async fn delete(&self, key: char) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting \"{key}\"");

        let mut data_buffer = [0; S];
        let flash = &mut *self.flash.lock().await;
        let mut flash = BlockingAsync::new(flash);

        map::store_item::<_, Option<T>, _>(
            &mut flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(key as u8),
            &None,
        )
        .await?;

        Ok(())
    }

    pub async fn delete_all(&self) -> Result<(), Error> {
        log::info!("{LOGGER_NAME}: Deleting all");
        // Todo consider using remove_all_items
        let flash = &mut *self.flash.lock().await;
        let mut flash = BlockingAsync::new(flash);

        erase_all(&mut flash, self.flash_range.clone()).await?;
        Ok(())
    }
}
