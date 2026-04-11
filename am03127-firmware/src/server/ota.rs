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

/// Streams the firmware body into the target flash partition in 4-byte aligned chunks.
async fn flash_firmware<BR: Read, F: NorFlash>(
    body_reader: &mut BR,
    target_partition: &mut F,
    content_length: usize,
) -> Result<(), &'static str> {
    let mut body_buffer = [0u8; OTA_BUFFER_SIZE];
    // Needs ALIGNMENT - 1 extra bytes: up to (ALIGNMENT - 1) unwritten remainder bytes
    // from the previous iteration may sit at the front before a full OTA_BUFFER_SIZE read
    // is appended, so the buffer must be large enough to hold both at the same time.
    let mut write_buffer = [0u8; OTA_BUFFER_SIZE + ALIGNMENT - 1];
    let mut bytes_written: usize = 0;
    let mut last_printed_percent = 0;
    let mut write_buffer_pos = 0;

    // Erase the region to be written before any writes. NorFlash bits can only transition
    // 1→0; without a prior erase, old 0-bits from a previous firmware corrupt the new image.
    log::info!("{LOGGER_NAME}: Erase partition");
    let erase_size = F::ERASE_SIZE as u32;
    let erase_end = (content_length as u32)
        .div_ceil(erase_size)
        .saturating_mul(erase_size);
    target_partition
        .erase(0, erase_end)
        .map_err(|_| "Failed to erase OTA partition")?;

    log::info!("{LOGGER_NAME}: Update status 0%");
    loop {
        match body_reader.read(&mut body_buffer).await {
            Ok(0) => {
                // Write any remaining data
                if write_buffer_pos > 0 {
                    if write_buffer_pos % ALIGNMENT != 0 {
                        return Err("Firmware size is not 4-byte aligned");
                    }
                    target_partition
                        .write(bytes_written as u32, &write_buffer[..write_buffer_pos])
                        .map_err(|_| "Failed to write final chunk to flash")?;
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
                        .map_err(|_| "Failed to write chunk to flash")?;
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
            Err(err) => {
                log::error!("{LOGGER_NAME}: Failed reading image: {:?}", err);
                return Err("Failed reading firmware body");
            }
        }
    }

    if bytes_written != content_length {
        return Err("Could not read whole body");
    }

    Ok(())
}

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

        let content_type = request.parts.headers().get("Content-Type");
        if content_type.is_none_or(|ct| ct != "application/octet-stream") {
            let connection = request.body_connection.finalize().await?;
            return response_writer
                .write_response(
                    connection,
                    Response::new(StatusCode::BAD_REQUEST, "Wrong content type"),
                )
                .await;
        }

        let content_length = body_reader.content_length();
        if content_length == 0 {
            let connection = request.body_connection.finalize().await?;
            return response_writer
                .write_response(
                    connection,
                    Response::new(
                        StatusCode::BAD_REQUEST,
                        "Content length does not match body size",
                    ),
                )
                .await;
        }

        let flash = &mut *state.storage.lock().await;
        let mut pt_buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];

        let ota_result: Result<(), &'static str> = async {
            let mut ota = OtaUpdater::new(flash, &mut pt_buffer)
                .map_err(|_| "Failed to initialize OTA updater")?;

            let current_partition = ota
                .selected_partition()
                .map_err(|_| "Failed to read selected partition")?;
            log::info!(
                "{LOGGER_NAME}: Currently selected partition {:?}",
                current_partition
            );

            let (mut target_partition, target_partition_type) = ota
                .next_partition()
                .map_err(|_| "No next OTA partition available")?;
            log::info!(
                "{LOGGER_NAME}: Selecting next OTA partition {:?}",
                target_partition_type
            );

            if content_length > target_partition.partition_size() {
                return Err("Image does not fit into OTA partition");
            }

            log::info!(
                "{LOGGER_NAME}: Flashing image to {:?}",
                target_partition_type
            );
            flash_firmware(&mut body_reader, &mut target_partition, content_length).await?;

            log::info!(
                "{LOGGER_NAME}: Select {:?} as next bootable partition",
                target_partition_type
            );
            ota.activate_next_partition()
                .map_err(|_| "Failed to activate next partition")?;
            ota.set_current_ota_state(esp_bootloader_esp_idf::ota::OtaImageState::New)
                .map_err(|_| "Failed to set OTA state")?;

            Ok(())
        }
        .await;

        let response = match ota_result {
            Ok(()) => Response::new(StatusCode::OK, "Update successful"),
            Err(msg) => {
                log::error!("{LOGGER_NAME}: {msg}");
                Response::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };
        let connection = request.body_connection.finalize().await?;
        let sent = response_writer.write_response(connection, response).await?;

        if ota_result.is_ok() {
            log::info!("{LOGGER_NAME}: Rebooting into new firmware");
            esp_hal::system::software_reset()
        }

        Ok(sent)
    }
}
