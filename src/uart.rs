use esp_hal::{
    gpio::interconnect::{PeripheralInput, PeripheralOutput},
    peripheral::Peripheral,
    peripherals::UART1,
    uart::{Config, DataBits, Parity, Uart as UartDriver},
    Async,
};

const BAUD_RATE: u32 = 9600;

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
}
