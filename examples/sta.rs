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
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::Subscriber};
use embassy_time::{Duration, Timer};
use esp_csi_rs::{
    config::{CSIConfig, TrafficConfig, TrafficType, WiFiConfig},
    CSICollector, NetworkArchitechture,
};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use esp_println::println;
use esp_wifi::{init, EspWifiController};
use heapless::Vec;

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
    // Traffic (UDP packets) is generated every 500 milli seconds
    // Network Architechture is AccessPointStation (no NTP time collection)
    let csi_collector = CSICollector::new(
        WiFiConfig {
            ssid: "Connected Motion ".try_into().unwrap(),
            password: "automotion@123".try_into().unwrap(),
            ..Default::default()
        },
        esp_csi_rs::WiFiMode::Station,
        CSIConfig::default(),
        TrafficConfig {
            traffic_type: TrafficType::ICMPPing,
            traffic_interval_ms: 500,
        },
        true,
        NetworkArchitechture::AccessPointStation,
        None,
        false,
    );

    // Initalize CSI collector
    csi_collector.init(wifi, init, seed, &spawner).unwrap();

    // Collect CSI for 5 seconds
    // let reciever = csi_collector.start(Some(5));
    // To run indefinely, use the following line instead
    let csi = csi_collector.start(None);

    // Optionally spawn a task to process incoming CSI data
    // collector returns a Subscriber type with a buffer of 612 bytes, Capacity of 1, 2 subscribers, and 2 publishers
    spawner.spawn(csi_task(csi)).ok();

    loop {
        Timer::after(Duration::from_secs(1)).await
    }
}

#[embassy_executor::task]
async fn csi_task(
    mut csi_buffer: Subscriber<'static, CriticalSectionRawMutex, Vec<i8, 612>, 1, 2, 1>,
) {
    loop {
        // Wait for CSI data to be received
        let csi_data = csi_buffer.next_message().await;
        // Print the CSI data
        println!("CSI Data printed from Task: {:?}", csi_data);
    }
}
