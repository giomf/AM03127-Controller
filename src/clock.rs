use core::net::SocketAddr;
use embassy_net::Stack as NetworkStack;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_time::{Duration, Timer};
use sntpc::{NtpContext, NtpTimestampGenerator, get_time};
use time::OffsetDateTime;

use crate::am03127::realtime_clock::DateTime;
use crate::server::SharedPanel;

const SNTP_ADDRESS: [u8; 4] = [188, 174, 253, 188];
const SNTP_PORT: u16 = 123;
const UPDATE_INTERVAL_SECS: u64 = 3600;
const LOGGER_NAME: &str = "Clock";

#[derive(Copy, Clone, Default)]
struct DummyTimestampGenerator;
impl NtpTimestampGenerator for DummyTimestampGenerator {
    fn init(&mut self) {}

    fn timestamp_sec(&self) -> u64 {
        0
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        0
    }
}

impl From<OffsetDateTime> for DateTime {
    fn from(value: OffsetDateTime) -> Self {
        Self {
            year: (value.year() % 100) as u8,
            week: value.iso_week(),
            month: value.month() as u8,
            day: value.day(),
            hour: value.hour(),
            minute: value.minute(),
            second: value.second(),
        }
    }
}

#[embassy_executor::task]
pub async fn timing_task(network_stack: NetworkStack<'static>, shared_panel: SharedPanel) {
    log::info!("{LOGGER_NAME}: Start clock task. Executing every {UPDATE_INTERVAL_SECS} seconds.");
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];
    let mut socket = UdpSocket::new(
        network_stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(SNTP_PORT).unwrap();
    let context = NtpContext::new(DummyTimestampGenerator::default());
    let sntp_address = SocketAddr::from((SNTP_ADDRESS, SNTP_PORT));

    loop {
        network_stack.wait_config_up().await;
        log::info!("{LOGGER_NAME}: Getting current date");
        match get_time(sntp_address, &socket, context).await {
            Ok(result) => {
                log::info!("{LOGGER_NAME}: Setting curernt date to panel");
                let timestamp = result.sec();
                let datetime = OffsetDateTime::from_unix_timestamp(timestamp as i64).unwrap();
                let mut panel = shared_panel.lock().await;
                match panel.set_clock(datetime.into()).await {
                    Ok(_) => {
                        log::info!("{LOGGER_NAME}: Updated panel to current date: {datetime}")
                    }
                    Err(e) => {
                        log::error!("{LOGGER_NAME}: Failed to send current date to panel. {e}")
                    }
                }
            }
            Err(e) => log::error!("{LOGGER_NAME}: Failed to get current time {:?}", e),
        };
        Timer::after(Duration::from_secs(UPDATE_INTERVAL_SECS)).await;
    }
}
