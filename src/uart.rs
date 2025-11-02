extern crate alloc;

use crate::error::Error;
use embassy_time::{Duration, with_timeout};
use embedded_io_async::Write;
use esp_hal::{
    Async,
    gpio::interconnect::{PeripheralInput, PeripheralOutput},
    peripherals::UART1,
    uart::{Config, DataBits, Parity, Uart as UartDriver},
};

/// Baud rate for UART communication with the LED panel
const BAUD_RATE: u32 = 9600;
/// Size of the buffer for reading responses from the LED panel
const READ_BUFFER_SIZE: usize = 32;
/// Logger name for UART-related log messages
const LOGGER_NAME: &str = "UART";
/// Uart timeout in seconds
const UART_TIMEOUT_SECS: u64 = 5;

/// UART communication interface for the LED panel
///
/// This struct handles the low-level communication with the AM03127 LED panel
/// over a UART interface.
pub struct Uart<'a> {
    /// The underlying UART driver
    uart: UartDriver<'a, Async>,
}

impl<'a> Uart<'a> {
    /// Creates a new UART interface for communicating with the LED panel
    ///
    /// # Arguments
    /// * `uart` - The UART1 peripheral
    /// * `tx` - The TX pin peripheral
    /// * `rx` - The RX pin peripheral
    ///
    /// # Returns
    /// * A new Uart instance configured for communication with the LED panel
    pub fn new(
        uart: UART1<'a>,
        tx: impl PeripheralOutput<'a>,
        rx: impl PeripheralInput<'a>,
    ) -> Self {
        let config = Config::default()
            .with_baudrate(BAUD_RATE)
            .with_data_bits(DataBits::_8)
            .with_stop_bits(esp_hal::uart::StopBits::_1)
            .with_parity(Parity::None);

        let uart = UartDriver::new(uart, config)
            .unwrap()
            .with_rx(rx)
            .with_tx(tx)
            .into_async();

        Self { uart }
    }

    /// Writes data to the LED panel and processes the response
    ///
    /// # Arguments
    /// * `data` - The data to write to the LED panel
    ///
    /// # Returns
    /// * `Ok(())` if the write was successful and the panel acknowledged it
    /// * `Err(Error)` if the write failed or the panel rejected the command
    pub async fn write(&mut self, data: &str) -> Result<(), Error> {
        log::debug!("{LOGGER_NAME}: Sending {data}");

        let timeout = Duration::from_secs(UART_TIMEOUT_SECS);
        with_timeout(timeout, self.uart.write_all(data.as_bytes()))
            .await
            .map_err(|_| Error::Uart("Write timeout".try_into().unwrap()))??;

        let mut buffer = [0u8; READ_BUFFER_SIZE];
        let bytes_read = with_timeout(timeout, self.uart.read_async(&mut buffer))
            .await
            .map_err(|_| Error::Uart("Read timeout".try_into().unwrap()))??;

        log::debug!("{LOGGER_NAME}: Receiving {bytes_read} bytes");
        let response = core::str::from_utf8(&buffer[..bytes_read]).unwrap();

        log::debug!("{LOGGER_NAME}: Interpreting response as: {}", response);

        if response.starts_with("NACK") {
            log::error!("Failed get positive response from uart")
        }

        Ok(())
    }
}
