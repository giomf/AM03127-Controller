use embedded_storage::nor_flash::NorFlash;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use picoserve::{
    io::Read,
    response::{Response, StatusCode},
};

use crate::server::AppState;

const LOGGER_NAME: &str = "OTA";
const OTA_BUFFER_SIZE: usize = 4096;
const ALIGNMENT: usize = 4;

pub struct OverTheAirUpdate;

impl picoserve::routing::RequestHandlerService<AppState, ()> for OverTheAirUpdate {
    async fn call_request_handler_service<
        R: Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        state: &AppState,
        (): (),
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        log::info!("{LOGGER_NAME}: Starting Over The Air Update");

        let mut body_reader = request.body_connection.body().reader();
        let content_type_header = request.parts.headers().get("Content-Type");
        if content_type_header.is_none_or(|content_type| content_type != "application/octet-stream")
        {
            let response = Response::new(StatusCode::BAD_REQUEST, "Wrong content type");
            let connection = request.body_connection.finalize().await?;
            return response_writer.write_response(connection, response).await;
        }

        let content_length = body_reader.content_length();
        if content_length == 0 {
            let response = Response::new(
                StatusCode::BAD_REQUEST,
                "Content length does not match body size",
            );
            let connection = request.body_connection.finalize().await?;
            return response_writer.write_response(connection, response).await;
        }

        let flash = &mut *state.storage.lock().await;
        let mut pt_buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];

        let mut ota = OtaUpdater::new(flash, &mut pt_buffer).unwrap();
        let current_partition = ota.selected_partition().unwrap();
        log::info!(
            "{LOGGER_NAME}: Currently selected partition {:?}",
            current_partition
        );

        let (mut target_partition, target_partition_type) = ota.next_partition().unwrap();
        log::info!(
            "{LOGGER_NAME}: Selecting nexdt ota partition {:?}",
            target_partition_type
        );
        if content_length > target_partition.partition_size() {
            let response = Response::new(
                StatusCode::BAD_REQUEST,
                "Image does not fit into ota partition",
            );
            let connection = request.body_connection.finalize().await?;
            return response_writer.write_response(connection, response).await;
        }

        log::info!(
            "{LOGGER_NAME}: Flashing image to {:?}",
            target_partition_type
        );
        let mut body_buffer = [0u8; OTA_BUFFER_SIZE];
        let mut write_buffer = [0u8; OTA_BUFFER_SIZE];
        let mut bytes_written: usize = 0;
        let mut last_printed_percent = 0;
        let mut write_buffer_pos = 0;

        log::info!("{LOGGER_NAME}: Update status 0%");
        loop {
            match body_reader.read(&mut body_buffer).await {
                Ok(0) => {
                    // Write any remaining data
                    if write_buffer_pos > 0 {
                        // Check if firmware size is aligned
                        if write_buffer_pos % ALIGNMENT != 0 {
                            log::error!(
                                "{LOGGER_NAME}: Firmware size not aligned to 4 bytes! Size: {}",
                                write_buffer_pos
                            );
                        }
                        target_partition
                            .write(bytes_written as u32, &write_buffer[..write_buffer_pos])
                            .unwrap();
                        bytes_written += write_buffer_pos;
                    }
                    break;
                }
                Ok(bytes_read) => {
                    // Copy new data into write_buffer
                    write_buffer[write_buffer_pos..write_buffer_pos + bytes_read]
                        .copy_from_slice(&body_buffer[..bytes_read]);
                    write_buffer_pos += bytes_read;

                    // Write as much as possible in 4-byte aligned chunks
                    let writable_size = (write_buffer_pos / ALIGNMENT) * ALIGNMENT;

                    if writable_size > 0 {
                        target_partition
                            .write(bytes_written as u32, &write_buffer[..writable_size])
                            .unwrap();
                        bytes_written += writable_size;

                        // Move remaining unaligned bytes to the start of buffer
                        let remainder = write_buffer_pos - writable_size;
                        if remainder > 0 {
                            write_buffer.copy_within(writable_size..write_buffer_pos, 0);
                        }
                        write_buffer_pos = remainder;

                        let current_percent =
                            (bytes_written as f32 / content_length as f32 * 100.0) as u32;
                        if current_percent >= last_printed_percent + 10 {
                            log::info!("{LOGGER_NAME}: Update status {}%", current_percent);
                            last_printed_percent = current_percent;
                        }
                    }
                }
                Err(err) => log::error!("{LOGGER_NAME}: Failed reading image: {:?}", err),
            }
        }
        let response = if bytes_written != content_length {
            Response::new(StatusCode::BAD_REQUEST, "Could not read whole body")
        } else {
            log::info!(
                "{LOGGER_NAME}: Select {:?} as next bootable partition",
                target_partition_type
            );
            ota.activate_next_partition().unwrap();
            ota.set_current_ota_state(esp_bootloader_esp_idf::ota::OtaImageState::New)
                .unwrap();

            Response::new(StatusCode::OK, "Update successfull")
        };
        let connection = request.body_connection.finalize().await?;
        response_writer.write_response(connection, response).await
    }
}
