#![recursion_limit = "256"]
#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![deny(clippy::mem_forget)]
#[cfg(feature = "sntp")]
mod clock;
mod error;
mod panel;
mod server;
mod storage;
mod uart;

use embassy_executor::Spawner;
use embassy_net::{Runner, Stack as NetworkStack, StackResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer, with_timeout};
use esp_alloc as _;
use esp_backtrace as _;
use esp_bootloader_esp_idf::{esp_app_desc, ota_updater::OtaUpdater};
use esp_hal::{clock::CpuClock, peripherals::WIFI, ram, rng::Rng, timer::timg::TimerGroup};
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, ControllerConfig, Interface, WifiController,
    sta::StationConfig,
};
use esp_storage::FlashStorage;
use panel::Panel;
use picoserve::{AppRouter, AppWithStateBuilder, Config as ServerConfig, Router, make_static};
use server::{AppState, ServerProperties, web_task};
use uart::Uart;

#[cfg(feature = "sntp")]
use crate::clock::timing_task;

const WEB_TASK_POOL_SIZE: usize = 2;
// Webtask poolsize + sntp socket + one extra
#[cfg(feature = "sntp")]
const STACK_RESSOURCE_SIZE: usize = WEB_TASK_POOL_SIZE + 1 + 1;
// Webtask poolsize + one extra
#[cfg(not(feature = "sntp"))]
const STACK_RESSOURCE_SIZE: usize = WEB_TASK_POOL_SIZE + 1;
const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");
const PANEL_INIT_DELAY_SECS: u64 = 10;
const WIFI_DELAY_SECS: u64 = 5;
/// Max wait for link and DHCP after connecting; a drop in that window would otherwise hang the task.
const WIFI_CONFIG_TIMEOUT_SECS: u64 = 30;

pub type SharedUart = &'static Mutex<CriticalSectionRawMutex, Uart<'static>>;
pub type SharedStorage = &'static Mutex<CriticalSectionRawMutex, FlashStorage<'static>>;

esp_app_desc!(
    env!("BUILD_VERSION"), // version
    "AM03127-Controller",  // project name
    env!("BUILD_TIME"),    // build time
    env!("BUILD_DATE"),    // build date
    "5.2",                 // ESP-IDF version
    64 * 1024,             // MMU page size
    0,                     // min eFuse block revision
    u16::MAX,              // max eFuse block revision
    esp_bootloader_esp_idf::SECURE_VERSION
);

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let esp_config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(esp_config);

    // 100 KB Heap
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let shared_uart = make_static!(
        Mutex<CriticalSectionRawMutex, Uart>, Mutex::new(Uart::new(peripherals.UART1, peripherals.GPIO3, peripherals.GPIO2))
    );
    let shared_storage = make_static!(Mutex<CriticalSectionRawMutex, FlashStorage>, Mutex::new(FlashStorage::new(peripherals.FLASH)));
    let panel = make_static!(Panel, Panel::new(shared_uart, shared_storage));

    let mut ota_buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    let (selected_partition, booted_partition) = {
        let flash = &mut *shared_storage.lock().await;
        let mut pt_buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
        let pt = esp_bootloader_esp_idf::partitions::read_partition_table(flash, &mut pt_buffer)
            .unwrap();
        let booted = pt
            .booted_partition()
            .ok()
            .flatten()
            .map(|p| p.partition_type());
        let mut ota = OtaUpdater::new(flash, &mut ota_buffer).unwrap();
        let selected = ota.selected_partition().unwrap();
        (selected, booted)
    };

    log::info!("==========================================");
    log::info!("\tAM03127-Controller");
    log::info!("\tVersion:\t\t{}", env!("BUILD_VERSION"));
    log::info!(
        "\tBuilt:\t\t\t{} {}",
        env!("BUILD_DATE"),
        env!("BUILD_TIME")
    );
    match booted_partition {
        Some(p) => log::info!("\tBooted partition:\t{:?}", p),
        None => log::info!("\tBooted partition:\tUnknown"),
    }
    log::info!("\tSelected partition:\t{:?}", selected_partition);
    log::info!("==========================================");

    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let (wifi_controller, wifi_interface) = init_wifi(peripherals.WIFI);
    let (network_stack, network_runner) = init_network(wifi_interface);
    let (server_app, server_config) = init_server();

    spawner.spawn(wifi_task(wifi_controller, network_stack).unwrap());
    spawner.spawn(network_task(network_runner).unwrap());
    spawner.spawn(panel_init_task(panel).unwrap());
    #[cfg(feature = "sntp")]
    spawner.spawn(timing_task(network_stack, panel).unwrap());

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.spawn(web_task(
            id,
            network_stack,
            server_app,
            server_config,
            AppState {
                panel: panel,
                storage: shared_storage,
            },
        ).unwrap());
    }
}

fn init_wifi(wifi: WIFI<'static>) -> (WifiController<'static>, Interface) {
    let station = StationConfig::default()
        .with_ssid(SSID.try_into().unwrap())
        .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
            PASSWORD.try_into().unwrap(),
        ));
    let config = ControllerConfig::default().with_initial_config(WifiConfig::Station(station));
    let controller = WifiController::new(wifi, config).unwrap();

    (controller, Interface::station())
}

fn init_network(wifi_device: Interface) -> (NetworkStack<'static>, Runner<'static, Interface>) {
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
    &'static mut Router<<ServerProperties as AppWithStateBuilder>::PathRouter, AppState>,
    &'static mut ServerConfig,
) {
    let server_app = make_static!(AppRouter<ServerProperties>, ServerProperties.build_app());
    let server_config = make_static!(
        ServerConfig,
        ServerConfig::new(picoserve::Timeouts {
            persistent_start_read_request: Duration::from_secs(5),
            start_read_request: Duration::from_secs(5),
            read_request: Duration::from_secs(60),
            write: Duration::from_secs(2),
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
        if wifi_controller.is_connected() {
            // wait until we're no longer connected
            let _ = wifi_controller.wait_for_disconnect_async().await;
            log::info!("{LOGGER_NAME}: Wifi disconnected");
            Timer::after(Duration::from_secs(WIFI_DELAY_SECS)).await
        }

        match wifi_controller.connect_async().await {
            Ok(_) => {
                log::info!("{LOGGER_NAME}: Wifi connected to {SSID}");
                let config_up = async {
                    network_stack.wait_link_up().await;
                    log::info!("{LOGGER_NAME}: Getting DHCP IP address");
                    network_stack.wait_config_up().await;
                };
                let timeout = Duration::from_secs(WIFI_CONFIG_TIMEOUT_SECS);
                if with_timeout(timeout, config_up).await.is_err() {
                    log::error!(
                        "{LOGGER_NAME}: No DHCP IP address after {WIFI_CONFIG_TIMEOUT_SECS} s, reconnecting"
                    );
                    let _ = wifi_controller.disconnect_async().await;
                    Timer::after(Duration::from_secs(WIFI_DELAY_SECS)).await;
                    continue;
                }

                match network_stack.config_v4() {
                    Some(config) => {
                        log::info!("{LOGGER_NAME}: Received DHCP IP address {}", config.address)
                    }
                    None => log::error!("{LOGGER_NAME}: Failed to receive DHCP IP address"),
                }
            }
            Err(e) => {
                log::info!(
                    "{LOGGER_NAME}: Failed to connect to wifi \"{SSID}\": {:?}",
                    e
                );
                Timer::after(Duration::from_secs(WIFI_DELAY_SECS)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, Interface>) -> ! {
    const LOGGER_NAME: &str = "Network";
    log::info!("{LOGGER_NAME}: Start network task");
    runner.run().await
}

#[embassy_executor::task]
async fn panel_init_task(panel: &'static Panel) {
    Timer::after(Duration::from_secs(PANEL_INIT_DELAY_SECS)).await;

    match panel.init().await {
        Ok(_) => log::info!("Panel initialized"),
        Err(e) => log::error!("Failed to initialize panel. {e}"),
    }
}
