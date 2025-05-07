//! Example of Station Mode for CSI Collection
//!
//! This configuration allows the collection of CSI data by connecting to a WiFi router or ESP Access Point.
//!
//! At least two devices are needed in this configuration.
//!
//! Connection Options:
//! - Option 1: Connect to an existing commercial router
//! - Option 2: Connect to another ESP programmed in AP Mode or AP/STA Mode
//!
//! The SSID and Password defined is for the Access Point or Router the ESP Station will be connecting to.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_csi_rs::{
    config::{CSIConfig, TrafficConfig, TrafficType, WiFiConfig},
    CSICollector, NetworkArchitechture,
};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use esp_println::println;
use esp_wifi::{init, EspWifiController};

extern crate alloc;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Configure System Clock
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    // Take Peripherals
    let peripherals = esp_hal::init(config);

    // Allocate some heap space
    esp_alloc::heap_allocator!(size: 72 * 1024);

    println!("Embassy Init");
    // Initialize Embassy
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    // Instantiate peripherals necessary to set up  WiFi
    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let wifi = peripherals.WIFI;
    let timer = timer1.timer0;
    let mut rng = Rng::new(peripherals.RNG);
    let radio_clk = peripherals.RADIO_CLK;

    println!("Controller Init");
    // Initialize WiFi Controller
    let init = &*mk_static!(
        EspWifiController<'static>,
        init(timer, rng, radio_clk,).unwrap()
    );

    // Obtain a random seed value
    let seed = rng.random() as u64;

    println!("WiFi Controller Initialized");

    // Create a CSI collector configuration
    // Device configured as a Station
    // Traffic is enabled with UDP packets
    // Traffic (UDP packets) is generated every 1 second
    // Network Architechture is Station-Router (enables NTP time collection)
    let csi_collector = CSICollector::new(
        WiFiConfig {
            ssid: "esp".try_into().unwrap(),
            password: "12345678".try_into().unwrap(),
            ..Default::default()
        },
        esp_csi_rs::WiFiMode::Station,
        CSIConfig::default(),
        TrafficConfig {
            traffic_type: TrafficType::UDP,
            traffic_interval_ms: 1000,
        },
        true,
        NetworkArchitechture::RouterStation,
    );

    // Initalize CSI collector
    csi_collector.init(wifi, init, seed, &spawner).unwrap();

    // Collect CSI for 10 seconds
    csi_collector.start(10).await;

    loop {}
}
