#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_time::{Duration, Timer};

use esp_backtrace as _;
use esp_hal::{rng::Rng, timer::timg::TimerGroup};
use esp_println::{self, println};
use defmt_rtt as _;

use foa::FoAResources;
use foa_sta::StaResources;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    println!("[PRINTLN] Starting sta_scan application...");
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    println!("[PRINTLN] Peripherals initialized");

    println!("[LOG] Starting sta_scan application...");
    
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    println!("[LOG] Embassy initialized");

    println!("[LOG] Creating FoA resources...");
    let stack_resources = mk_static!(FoAResources, FoAResources::new());
    println!("[LOG] FoA resources created, initializing WiFi...");
    
    println!("[LOG] Calling foa::init...");
    let ([mut sta_vif, ..], mut foa_runner) = foa::init(
        stack_resources,
        peripherals.WIFI,
    );
    println!("[LOG] foa::init completed");
    
    println!("[LOG] WiFi stack initialized, creating STA interface...");
    let sta_resources = mk_static!(StaResources, StaResources::default());
    let (mut sta_control, mut sta_runner, _net_device) =
        foa_sta::new_sta_interface(&mut sta_vif, sta_resources, Rng::new(peripherals.RNG));
    
    println!("[LOG] STA interface created, starting tasks...");
    
    // Heartbeat task to show the system is alive
    let heartbeat_task = async {
        let mut counter = 0;
        loop {
            Timer::after(Duration::from_secs(5)).await;
            counter += 1;
            println!("[HEARTBEAT] System alive - tick #{}", counter);
        }
    };
    
    join4(foa_runner.run(), sta_runner.run(), heartbeat_task, async {
        println!("[LOG] Starting main scan task...");
        let mut found_bss = heapless::FnvIndexMap::new();
        println!("[LOG] Starting WiFi scan for available networks...");
        let _ = sta_control.scan::<32>(None, &mut found_bss).await;
        println!("[LOG] Scan completed, found {} networks", found_bss.len());
        for (_, bss) in found_bss {
            println!(
                "[LOG] Found BSS, with SSID: \"{}\", BSSID: {}, channel: {}, last RSSI: {}.",
                bss.ssid, bss.bssid, bss.channel, bss.last_rssi
            );
        }
        println!("[LOG] Scan task completed");
    })
    .await;
}
