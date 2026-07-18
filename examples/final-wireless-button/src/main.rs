//! final-wireless-button（送信側）: 無線ボタン端末のエントリポイント
//!
//! BOOTボタン（GPIO9）の押下をESP-NOWで受信端末へ送る「無線ボタン端末」です。
//! - 押した瞬間: イベントパケットを即時送信（ACKが来なければ再送）
//! - 500msごと: 現在のボタン状態を載せたハートビートを送信
//! - 5秒以上ACKが得られない: エラー状態としてLED（GPIO10）を高速点滅
//!
//! このファイルはハードウェアの初期化とtaskの起動だけを行い、
//! アプリ本体の配線はライブラリ側の app モジュールに任せます。
//!
//! 受信側は `cargo run --bin receiver` で別のボードに書き込みます。
//!
//! 配線:
//! - LED: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//! - ボタン: 配線不要（ボード上のBOOTボタン = GPIO9 をそのまま使用）

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::efuse::{InterfaceMacAddress, interface_mac_address};
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use final_wireless_button::{app, config};
// defmt の global_logger を提供するクレートをリンクする（feature で切替）。
// probe-rs: rtt-target(RTT)、espflash: esp-println(USBシリアル)。
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // defmt(RTT) の初期化。probe-rs モードのときだけ RTT を張る。
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let hal_config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(hal_config);

    // 無線スタックはヒープを使うため、アロケータを用意する（10-esp-nowと同じ）
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // Wi-Fiドライバを初期化してESP-NOWインターフェースを得る。
    // コントローラはdropすると無線が止まるので変数として保持しておく
    let (_controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default()).unwrap();
    let esp_now = interfaces.esp_now;

    // 送受信するボード同士は同じWi-Fiチャネルに合わせる必要がある
    esp_now.set_channel(config::WIFI_CHANNEL).unwrap();

    let mac = interface_mac_address(InterfaceMacAddress::Station);
    info!(
        "無線ボタン端末（送信側）起動 MAC={=[u8]:02x}",
        mac.as_bytes()
    );

    // LED（エラー表示用）とBOOTボタン
    let led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
    let button_config = InputConfig::default().with_pull(Pull::Up);
    let button = Input::new(peripherals.GPIO9, button_config);

    // アプリ本体のtask群を配線して起動（詳細は src/app.rs）
    app::spawn_sender_tasks(&spawner, button, led, esp_now);

    // mainはもう仕事がないので待機するだけ
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
