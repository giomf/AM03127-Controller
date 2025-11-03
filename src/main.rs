#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#[deny(clippy::mem_forget)]
mod am03127;
mod clock;
mod error;
mod panel;
mod server;
mod storage;
mod uart;

use embassy_executor::Spawner;
use embassy_net::{Runner, Stack as NetworkStack, StackResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::{
    clock::CpuClock, interrupt::software::SoftwareInterruptControl, peripherals::WIFI, ram,
    rng::Rng, timer::timg::TimerGroup,
};
use esp_radio::{
    Controller, init,
    wifi::{ClientConfig, ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStaState},
};
use esp_storage::FlashStorage;
use panel::Panel;
use picoserve::{AppRouter, AppWithStateBuilder, Config as ServerConfig, Router, make_static};
use server::{AppProps, AppState, web_task};
use uart::Uart;

use crate::clock::timing_task;

const WEB_TASK_POOL_SIZE: usize = 2;
// Webtask poolsize + sntp socket + one extra
const STACK_RESSOURCE_SIZE: usize = WEB_TASK_POOL_SIZE + 1 + 1;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

pub type SharedUart = &'static Mutex<CriticalSectionRawMutex, Uart<'static>>;
pub type SharedStorage = &'static Mutex<CriticalSectionRawMutex, FlashStorage<'static>>;

esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let esp_config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(esp_config);

    // 256 KB Heap
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 192 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let shared_uart = make_static!(
        Mutex<CriticalSectionRawMutex, Uart>, Mutex::new(Uart::new(peripherals.UART1, peripherals.GPIO3, peripherals.GPIO2))
    );
    let shared_storage = make_static!(Mutex<CriticalSectionRawMutex, FlashStorage>, Mutex::new(FlashStorage::new(peripherals.FLASH)));
    let panel = make_static!(Panel, Panel::new(shared_uart, shared_storage));

    #[cfg(target_arch = "riscv32")]
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        sw_int.software_interrupt0,
    );

    let (wifi_controller, wifi_interface) = init_wifi(peripherals.WIFI);
    let (network_stack, network_runner) = init_network(wifi_interface);
    let (server_app, server_config) = init_server();

    spawner.must_spawn(wifi_task(wifi_controller, network_stack));
    spawner.must_spawn(network_task(network_runner));
    spawner.must_spawn(panel_init_task(panel));
    spawner.must_spawn(timing_task(network_stack, panel));
    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(
            id,
            network_stack,
            server_app,
            server_config,
            AppState {
                panel: panel,
                storage: shared_storage,
            },
        ));
    }
}

fn init_wifi(wifi: WIFI<'static>) -> (WifiController<'static>, WifiDevice<'static>) {
    let esp_radio_ctrl = &*make_static!(Controller<'static>, init().unwrap());
    let (controller, interfaces) =
        esp_radio::wifi::new(&esp_radio_ctrl, wifi, Default::default()).unwrap();
    let device = interfaces.sta;

    return (controller, device);
}

fn init_network(
    wifi_device: WifiDevice<'static>,
) -> (NetworkStack<'static>, Runner<'static, WifiDevice<'static>>) {
    let rng = Rng::new();
    let config = embassy_net::Config::dhcpv4(Default::default());
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    embassy_net::new(
        wifi_device,
        config,
        make_static!(StackResources<STACK_RESSOURCE_SIZE>, StackResources::new()),
        seed,
    )
}

fn init_server() -> (
    &'static mut Router<<AppProps as AppWithStateBuilder>::PathRouter, AppState>,
    &'static mut ServerConfig<Duration>,
) {
    let server_app = make_static!(AppRouter<AppProps>, AppProps.build_app());
    let server_config = make_static!(
        ServerConfig<Duration>,
        ServerConfig::new(picoserve::Timeouts {
            persistent_start_read_request: Some(Duration::from_secs(5)),
            start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(2)),
            write: Some(Duration::from_secs(2)),
        })
        .keep_connection_alive()
    );

    (server_app, server_config)
}

#[embassy_executor::task]
async fn wifi_task(
    mut wifi_controller: WifiController<'static>,
    network_stack: NetworkStack<'static>,
) {
    const LOGGER_NAME: &str = "WIFI";
    log::info!("{LOGGER_NAME}: Start wifi connection task");

    loop {
        if matches!(esp_radio::wifi::sta_state(), WifiStaState::Connected) {
            // wait until we're no longer connected
            wifi_controller
                .wait_for_event(WifiEvent::StaDisconnected)
                .await;
            log::info!("{LOGGER_NAME}: Wifi disconnected");
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(wifi_controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(SSID.try_into().unwrap())
                    .with_password(PASSWORD.try_into().unwrap()),
            );
            wifi_controller.set_config(&client_config).unwrap();
            log::info!("{LOGGER_NAME}: Starting wifi");
            wifi_controller.start_async().await.unwrap();
            log::info!("{LOGGER_NAME}: Wifi started");
        }
        log::info!("{LOGGER_NAME}: Connect to wifi");
        match wifi_controller.connect_async().await {
            Ok(_) => {
                log::info!("{LOGGER_NAME}: Wifi connected");
                network_stack.wait_link_up().await;
                log::info!("{LOGGER_NAME}: Getting DHCP IP address");
                network_stack.wait_config_up().await;

                match network_stack.config_v4() {
                    Some(config) => {
                        log::info!("{LOGGER_NAME}: Received DHCP IP address {}", config.address)
                    }
                    None => log::error!("{LOGGER_NAME}: Failed to receive DHCP IP address"),
                }
            }
            Err(e) => {
                log::info!("{LOGGER_NAME}: Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    const LOGGER_NAME: &str = "Network";
    log::info!("{LOGGER_NAME}: Start network task");
    runner.run().await
}

#[embassy_executor::task]
async fn panel_init_task(panel: &'static Panel) {
    match panel.init().await {
        Ok(_) => log::info!("Panel initialized"),
        Err(e) => log::error!("Failed to initialize panel. {e}"),
    }
}
