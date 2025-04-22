use core::ops::Range;

use crate::am03127::page_content::Page;
use embassy_embedded_hal::adapter::BlockingAsync;
use esp_storage::FlashStorage;
use heapless::Vec;
use sequential_storage::{cache::NoCache, map};

const LOGGER_NAME: &str = "NvsStorage";
const NVS_FLASH_BEGIN: u32 = 0x9000;
const NVS_FLASH_SIZE: u32 = 0x4000;
const ITEM_SIZE: usize = core::mem::size_of::<Option<Page>>();

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

    pub async fn read(&mut self, page_id: char) -> Option<Page> {
        log::info!("{LOGGER_NAME}: Reading page {page_id}");

        let mut data_buffer = [0; ITEM_SIZE];
        map::fetch_item::<u8, [u8; ITEM_SIZE], _>(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(page_id as u8),
        )
        .await
        .unwrap();
        postcard::from_bytes(&data_buffer).expect("Failed to deserialize page")
    }
    pub async fn write(&mut self, page_id: char, page: Page) {
        log::info!("{LOGGER_NAME}: Writing page {page_id}");

        let data: Vec<u8, ITEM_SIZE> =
            postcard::to_vec(&Some(page)).expect("Failed to serialize page");
        let mut data_buffer = [0u8; ITEM_SIZE];

        map::store_item(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(page_id as u8),
            &data.as_slice(),
        )
        .await
        .unwrap();
    }

    pub async fn delete(&mut self, page_id: char) {
        log::info!("{LOGGER_NAME}: Deleting page {page_id}");

        let mut data_buffer = [0u8; ITEM_SIZE];

        map::remove_item(
            &mut self.flash,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &(page_id as u8),
        )
        .await
        .unwrap();
    }
}
