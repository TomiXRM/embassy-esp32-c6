//! receiver（受信側）: 無線ボタン端末の受信エントリポイント
//!
//! 送信側からのESP-NOWパケットを受信して:
//! - (送信元MAC, seq) で重複を判定し、重複でもACKは返す（再送を止めるため）
//! - ボタンイベント/ハートビートをログに表示
//! - ボタン状態をLED（GPIO10）にミラーリング
//! - 2秒以上ハートビートが来なければ「送信側ロスト」を警告
//!
//! 書き込み: `cargo run --release --bin receiver`
//!
//! このファイルはハードウェアの初期化だけを行い、
//! 受信処理の本体はライブラリ側の app モジュールに任せます。
//!
//! 配線:
//! - LED: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::efuse::{InterfaceMacAddress, interface_mac_address};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use final_wireless_button::{app, config};
use log::info;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let hal_config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(hal_config);

    esp_println::logger::init_logger_from_env();

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

    // 送信側と同じWi-Fiチャネルに合わせる
    esp_now.set_channel(config::WIFI_CHANNEL).unwrap();

    let mac = interface_mac_address(InterfaceMacAddress::Station);
    info!("無線ボタン端末（受信側）起動 MAC={}", mac);

    // ボタン状態をミラーリングするLED
    let led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    // 受信ループへ（戻らない。詳細は src/app.rs → src/radio.rs）
    app::run_receiver(esp_now, led).await
}
