use core::{net::SocketAddr, ops::Deref};

use am03127_commands::realtime_clock::DateTime;
use embassy_net::{
    Stack as NetworkStack,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_time::{Duration, Timer};
use sntpc::{NtpContext, NtpTimestampGenerator, get_time};
use time::{Date, Month, OffsetDateTime, UtcOffset, macros::offset, util::days_in_month};

use crate::{PANEL_INIT_DELAY_SECS, panel::Panel};

const CLOCK_INIT_DELAY_SECS: u64 = PANEL_INIT_DELAY_SECS + 5;
const CLOCK_UPDATE_INTERVAL_SECS: u64 = 86_400; // 24h
const LOGGER_NAME: &str = "Clock";
const SNTP_ADDRESS: [u8; 4] = [188, 174, 253, 188];
const SNTP_PORT: u16 = 123;

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

fn eu_utc_offset(utc: &OffsetDateTime) -> UtcOffset {
    let last_sunday_of = |month: Month| -> Date {
        let days_in_month = {
            let year = utc.year();
            days_in_month(month, year)
        };
        let last = Date::from_calendar_date(utc.year(), month, days_in_month).unwrap();
        last - time::Duration::days(last.weekday().number_days_from_sunday() as i64)
    };
    let dst_start = last_sunday_of(Month::March)
        .with_hms(1, 0, 0)
        .unwrap()
        .assume_utc();
    let dst_end = last_sunday_of(Month::October)
        .with_hms(1, 0, 0)
        .unwrap()
        .assume_utc();
    if *utc >= dst_start && *utc < dst_end {
        offset!(+2) // CEST
    } else {
        offset!(+1) // CET
    }
}

pub struct DateTimeWrapper(pub DateTime);

impl From<OffsetDateTime> for DateTimeWrapper {
    fn from(value: OffsetDateTime) -> Self {
        DateTimeWrapper(DateTime {
            year: (value.year() % 100) as u8,
            week: value.iso_week(),
            month: value.month() as u8,
            day: value.day(),
            hour: value.hour(),
            minute: value.minute(),
            second: value.second(),
        })
    }
}

impl Deref for DateTimeWrapper {
    type Target = DateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[embassy_executor::task]
pub async fn timing_task(network_stack: NetworkStack<'static>, panel: &'static Panel) {
    log::info!(
        "{LOGGER_NAME}: Start clock task. Executing every {CLOCK_UPDATE_INTERVAL_SECS} seconds."
    );
    Timer::after(Duration::from_secs(CLOCK_INIT_DELAY_SECS)).await;
    let context = NtpContext::new(DummyTimestampGenerator::default());
    let sntp_address = SocketAddr::from((SNTP_ADDRESS, SNTP_PORT));

    loop {
        network_stack.wait_config_up().await;
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
        log::info!("{LOGGER_NAME}: Getting current date");
        match get_time(sntp_address, &socket, context).await {
            Ok(result) => {
                let timestamp = result.sec();
                let mut datetime = OffsetDateTime::from_unix_timestamp(timestamp as i64).unwrap();
                datetime = datetime.to_offset(eu_utc_offset(&datetime));
                log::info!("{LOGGER_NAME}: Setting current date to panel: {datetime}");
                if let Err(e) = panel.set_clock(&DateTimeWrapper::from(datetime)).await {
                    log::error!("{LOGGER_NAME}: Failed to send current date to panel. {e}");
                }
            }
            Err(e) => log::error!("{LOGGER_NAME}: Failed to get current time {:?}", e),
        };
        Timer::after(Duration::from_secs(CLOCK_UPDATE_INTERVAL_SECS)).await;
    }
}
