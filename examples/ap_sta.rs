//! Example of Access Point + Station Mode for CSI Collection
//!
//! This configuration allows other ESP devices configured in Access Point + Station Mode to connect and collect CSI data.
//! This configuration also allows the Access Point to connect to a another router/gateway simultaneously as a Station.
//!
//! This configuration would be useful if the CSI collection network requires access to the internet.
//! For example, connecting to an NTP server for time syncronization.
//!
//! At least two devices are needed in this configuration.
//! More ESP stations connecting to the Access Point + Station can also be added to form a star topology.
//!
//! IMPORTANT NOTE: The AP + Station collects CSI data but does not generate traffic.
//!
//! Connection Options:
//! - Option 1: Allow one ESP configured as a Station to connect to the ESP Access Point + Station.
//! - Option 2: Allow multiple ESPs configured as a Station to connect to the ESP Access Point + Station.
//!
//! In both options the ESP Access Point + Station is assumed to be connected to a commercial router (as a station).
//!
//! The `ap_ssid` and `ap_password` defined are the ones the ESP Station(s) need to connect to the ESP Access Point.
//! The `ssid` and `password` defined are the ones the ESP Access Point needs to connect to a Commercial Router.
//! `max_connections` defines the maximum number of ESP Stations that can connect to the ESP Access Point.
//! The default value of `max_connections` is 1. This needs to be increased if you intend to connect more Stations.
//!
//! There is an option also to hide the ESP Access Point SSID by setting `ssid_hidden` to `true`.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_csi_rs::{
    config::{CSIConfig, TrafficConfig, WiFiConfig},
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
    esp_alloc::heap_allocator!(size:72 * 1024);

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

    // Instantiate WiFi controller and interfaces
    let (controller, interfaces) = esp_wifi::wifi::new(&init, wifi).unwrap();

    // Obtain a random seed value
    let seed = rng.random() as u64;

    println!("WiFi Controller Initialized");

    // Create a CSI collector configuration
    // Device configured as a Access Point + Router
    // Traffic is not enabled, so configuration is ignored. Connected stations are expected to generate traffic.
    // Any traffic generated in this device mode would be between the AP and the Router
    // Network Architechture is AccessPoint-Station (NTP time is collected)
    let csi_collector = CSICollector::new(
        WiFiConfig {
            ap_ssid: "SSID".try_into().unwrap(),
            ap_password: "PASSWORD".try_into().unwrap(),
            ssid: "ROUTER_SSID".try_into().unwrap(),
            password: "ROUTER_PASSWORD".try_into().unwrap(),
            max_connections: 1,
            ssid_hidden: false,
            ..Default::default()
        },
        esp_csi_rs::WiFiMode::AccessPointStation,
        CSIConfig::default(),
        TrafficConfig::default(),
        false,
        NetworkArchitechture::RouterAccessPointStation,
        None,
        false,
    );

    // Initalize CSI collector
    csi_collector
        .init(controller, interfaces, seed, &spawner)
        .unwrap();

    // Collect CSI for 10 seconds
    csi_collector.start(Some(10));

    loop {}
}
