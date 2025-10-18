use crate::server::AppState;
use embedded_storage::nor_flash::NorFlash;
use esp_storage::FlashStorage;
use picoserve::{io::Read, response::IntoResponse};

pub struct OverTheAirUpdate;

/// Logger name for router-related log messages
const LOGGER_NAME: &str = "OTA";

impl picoserve::routing::RequestHandlerService<AppState, ()> for OverTheAirUpdate {
    async fn call_request_handler_service<
        R: Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        _state: &AppState,
        (): (),
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        log::info!("{LOGGER_NAME}: Starting Over The Air Update");

        let mut flash = FlashStorage::new();
        let mut buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
        let pt = esp_bootloader_esp_idf::partitions::read_partition_table(&mut flash, &mut buffer)
            .unwrap();

        log::info!(
            "{LOGGER_NAME}: Currently booted partition {:?}",
            pt.booted_partition().unwrap().unwrap().label_as_str()
        );

        let mut ota =
            esp_bootloader_esp_idf::ota_updater::OtaUpdater::new(&mut flash, &mut buffer).unwrap();
        // ota.with_next_partition(|next_ota_partition, part_type| {
        //     log::info!("Flashing image to {:?}", part_type);

        //     // write to the app partition
        //     // for (sector, chunk) in OTA_IMAGE.chunks(4096).enumerate() {
        //     //     log::info!("Writing sector {sector}...");

        //     //     next_ota_partition
        //     //         .write((sector * 4096) as u32, chunk)
        //     //         .unwrap();
        //     // }

        //     log::info!("Changing OTA slot and setting the state to NEW");

        //     ota.activate_next_partition().unwrap();
        //     ota.set_current_ota_state(esp_bootloader_esp_idf::ota::OtaImageState::New)
        //         .unwrap();
        // });

        // let current = ota.selected_partition().unwrap();
        // log::info!(
        //     "current image state {:?} (only relevant if the bootloader was built with auto-rollback support)",
        //     ota.current_ota_state()
        // );
        // log::info!("currently selected partition {:?}", current);

        todo!();
        let mut reader = request.body_connection.body().reader();
        let mut buffer = [0; 1024];
        let mut total_size = 0;

        loop {
            let read_size = reader.read(&mut buffer).await?;
            if read_size == 0 {
                break;
            }
            total_size += read_size;
        }

        log::info!("Total Size: {total_size}\r\n");
        let connection = request.body_connection.finalize().await.unwrap();
        Ok("TEST".write_to(connection, response_writer).await.unwrap())
    }
}
