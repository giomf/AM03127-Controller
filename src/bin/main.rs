#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::{Runner,  StackResources};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::{
    EspWifiController, init,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
};

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

extern crate alloc;

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

    let esp_wifi_ctrl = &*mk_static!(
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
        mk_static!(StackResources<3>, StackResources::<3>::new()),
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
