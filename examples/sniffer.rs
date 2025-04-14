//! Example of Configuring Sniffer Mode for CSI Collection
//!
//! This configuration allows the collection of CSI data by sniffing WiFi networks.
//! Only one device is needed in this configuration. No SSID or Password need to be defined.
//!
//! This is also the default configuraion in `WiFiConfig`. Though Sniffer mode can also be explicitly defined.

#![no_std]
#![no_main]

use defmt::println;
use embassy_executor::Spawner;
use esp_csi_rs::{
    config::{CSIConfig, TrafficConfig, TrafficType, WiFiConfig},
    CSICollector, NetworkArchitechture,
};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
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
    esp_alloc::heap_allocator!(72 * 1024);

    // Initialize Embassy
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    // Instantiate peripherals necessary to set up  WiFi
    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let wifi = peripherals.WIFI;
    let timer = timer1.timer0;
    let mut rng = Rng::new(peripherals.RNG);
    let radio_clk = peripherals.RADIO_CLK;

    // Initialize WiFi Controller
    let init = &*mk_static!(
        EspWifiController<'static>,
        init(timer, rng, radio_clk,).unwrap()
    );

    // Obtain a random seed value
    let seed = rng.random() as u64;

    println!("WiFi Controller Initialized");

    // Create a CSI collector configuration
    // Device configured to default as Sniffer
    // Traffic generation, although default, is ignored
    // Network Architechture is Sniffer
    let csi_collector = CSICollector::new(
        WiFiConfig::default(),
        esp_csi_rs::WiFiMode::Sniffer,
        CSIConfig::default(),
        TrafficConfig::default(),
        false,
        NetworkArchitechture::Sniffer,
    );

    // Initalize CSI collector
    csi_collector.init(wifi, init, seed, &spawner).unwrap();

    // Collect CSI for 10 seconds
    csi_collector.start(10).await;

    loop {}
}
