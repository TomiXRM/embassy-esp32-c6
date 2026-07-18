//! 01-blinky: 最初のLチカ
//!
//! GPIO10に接続した外付けLED（330Ω抵抗経由）を1秒間隔で点滅させます。
//! ESP32-C6-DevKitC-1のオンボードLEDはWS2812B（GPIO8）で、
//! 単純なON/OFFでは光らないため、この例では外付けLEDを使います。
//!
//! 配線: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use log::info;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // GPIO10を出力に設定。最初は消灯（Low）
    let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    info!("Lチカを開始します");

    loop {
        led.set_high(); // 点灯
        Timer::after(Duration::from_millis(500)).await;
        led.set_low(); // 消灯
        Timer::after(Duration::from_millis(500)).await;
    }
}
