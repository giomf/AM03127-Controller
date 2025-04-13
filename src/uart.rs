use anyhow::{Result, bail};
use embedded_io_async::Write;
use esp_hal::{
    Async,
    gpio::interconnect::{PeripheralInput, PeripheralOutput},
    peripheral::Peripheral,
    peripherals::UART1,
    uart::{Config, DataBits, Parity, Uart as UartDriver},
};

const BAUD_RATE: u32 = 9600;
const READ_BUFFER_SIZE: usize = 32;

pub struct Uart<'a> {
    uart: UartDriver<'a, Async>,
}

impl<'a> Uart<'a> {
    pub fn new(
        uart: UART1,
        tx: impl Peripheral<P = impl PeripheralOutput> + 'a,
        rx: impl Peripheral<P = impl PeripheralInput> + 'a,
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

    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        self.uart.write_all(data).await?;
        let mut buffer = [0u8; READ_BUFFER_SIZE];
        let bytes_read = self.uart.read_async(&mut buffer).await?;

        log::info!("Receiving {bytes_read} bytes");
        let response = core::str::from_utf8(&buffer[..bytes_read])?;
        log::info!("Interpreting response as: {}", response);

        if response.starts_with("ACK") {
            return Ok(());
        } else if response.starts_with("NACK") {
            bail!("NACK");
        }

        Ok(())
    }
}
