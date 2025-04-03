#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#[deny(clippy::mem_forget)]

mod server;
mod am03127;
mod uart;

use embassy_executor::Spawner;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_println::println;
use esp_wifi::{
    EspWifiController, init,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
};
use picoserve::{AppBuilder, AppRouter, make_static};
use server::{AppProps, web_task};
use uart::Uart;

const WEB_TASK_POOL_SIZE: usize = 2;
const STACK_RESSOURCE_SIZE: usize = WEB_TASK_POOL_SIZE + 1;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut rng = Rng::new(peripherals.RNG);


    let uart = Uart::new(peripherals.UART1, peripherals.GPIO2, peripherals.GPIO3);

    let esp_wifi_ctrl = &*make_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
    );

    let (wifi_controller, interfaces) =
        esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();
    let wifi_interface = interfaces.sta;

    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let config = embassy_net::Config::dhcpv4(Default::default());
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (network_stack, network_runner) = embassy_net::new(
        wifi_interface,
        config,
        make_static!(StackResources<STACK_RESSOURCE_SIZE>, StackResources::new()),
        seed,
    );

    spawner.spawn(net_task(network_runner)).ok();
    spawner.spawn(wifi_connection(wifi_controller)).ok();

    network_stack.wait_link_up().await;
    network_stack.wait_config_up().await;

    match network_stack.config_v4() {
        Some(config) => println!("DHCP IP: {}", config.address),
        None => println!("Failed to receive DHCP IP address"),
    }
    let app = make_static!(AppRouter<AppProps>, AppProps.build_app());
    let config = make_static!(
        picoserve::Config<Duration>,
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(1)),
            write: Some(Duration::from_secs(1)),
        })
        .keep_connection_alive()
    );

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(id, network_stack, app, config));
    }
}

#[embassy_executor::task]
async fn wifi_connection(mut wifi_controller: WifiController<'static>) {
    println!("Start wifi connection task");
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
            println!("Starting wifi");
            wifi_controller.start_async().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect to wifi...");

        match wifi_controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
