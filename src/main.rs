#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#[deny(clippy::mem_forget)]
mod am03127;
mod error;
mod panel;
mod server;
mod storage;
mod uart;

use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::peripherals::WIFI;
use esp_hal::timer::{systimer::SystemTimer, timg::TimerGroupInstance};
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_wifi::{
    EspWifiController, init,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
};
use panel::Panel;
use picoserve::{AppRouter, AppWithStateBuilder, make_static};
use server::{AppProps, AppState, SharedPanel, web_task};
use uart::Uart;

const WEB_TASK_POOL_SIZE: usize = 2;
const STACK_RESSOURCE_SIZE: usize = WEB_TASK_POOL_SIZE + 1;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let rng = Rng::new(peripherals.RNG);
    let (wifi_controller, wifi_interface) =
        init_wifi(peripherals.TIMG0, peripherals.WIFI, rng.clone());
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let (network_stack, network_runner) = init_network(rng, wifi_interface);

    spawner.spawn(network_task(network_runner)).ok();
    spawner.spawn(wifi_task(wifi_controller)).ok();

    network_stack.wait_link_up().await;
    network_stack.wait_config_up().await;

    match network_stack.config_v4() {
        Some(config) => log::info!("Network: DHCP IP {}", config.address),
        None => log::error!("Network: Failed to receive DHCP IP address"),
    }
    let app = make_static!(AppRouter<AppProps>, AppProps.build_app());
    let config = make_static!(
        picoserve::Config<Duration>,
        picoserve::Config::new(picoserve::Timeouts {
            persistent_start_read_request: Some(Duration::from_secs(5)),
            start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(2)),
            write: Some(Duration::from_secs(2)),
        })
        .keep_connection_alive()
    );

    let uart = Uart::new(peripherals.UART1, peripherals.GPIO2, peripherals.GPIO3);
    let shared_panel = SharedPanel(make_static!(
        Mutex<CriticalSectionRawMutex, Panel>, Mutex::new(Panel::new(uart))
    ));
    let mut panel = shared_panel.0.lock().await;
    panel.init().await.expect("Failed to initialze panel");

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(
            id,
            network_stack,
            app,
            config,
            AppState { shared_panel },
        ));
    }
}

fn init_wifi(
    timer_group: impl TimerGroupInstance + 'static,
    wifi: WIFI<'static>,
    rng: Rng,
) -> (WifiController<'static>, WifiDevice<'static>) {
    let timg0 = TimerGroup::new(timer_group);
    let esp_wifi_ctrl =
        &*make_static!(EspWifiController<'static>, init(timg0.timer0, rng).unwrap());

    let (wifi_controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();
    let wifi_interface = interfaces.sta;

    return (wifi_controller, wifi_interface);
}

fn init_network(
    mut rng: Rng,
    wifi_interface: WifiDevice<'static>,
) -> (Stack<'static>, Runner<'static, WifiDevice<'static>>) {
    let config = embassy_net::Config::dhcpv4(Default::default());
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    embassy_net::new(
        wifi_interface,
        config,
        make_static!(StackResources<STACK_RESSOURCE_SIZE>, StackResources::new()),
        seed,
    )
}

#[embassy_executor::task]
async fn wifi_task(mut wifi_controller: WifiController<'static>) {
    const LOGGER_NAME: &str = "WIFI";
    log::info!("{LOGGER_NAME}: Start wifi connection task");

    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                wifi_controller
                    .wait_for_event(WifiEvent::StaDisconnected)
                    .await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(wifi_controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            wifi_controller.set_configuration(&client_config).unwrap();
            log::info!("{LOGGER_NAME}: Starting wifi");
            wifi_controller.start_async().await.unwrap();
            log::info!("{LOGGER_NAME}: Wifi started");
        }
        log::info!("{LOGGER_NAME}: About to connect to wifi...");

        match wifi_controller.connect_async().await {
            Ok(_) => log::info!("{LOGGER_NAME}: Wifi connected!"),
            Err(e) => {
                log::info!("{LOGGER_NAME}: Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    const LOGGER_NAME: &str = "Network";
    log::info!("{LOGGER_NAME}: Start network task");
    runner.run().await
}
