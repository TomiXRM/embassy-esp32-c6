//! 12-sleep: ディープスリープと復帰要因（タイマー + GPIO）
//!
//! 起動するとリセット要因・復帰要因をログに出し、GPIO10のLEDを3回点滅させ、
//! 5秒待ってから10秒間のディープスリープに入ります。
//! ディープスリープからはRTCタイマー（10秒経過）または
//! GPIO7がLowになったとき（EXT1ウェイクアップ）に復帰します。
//!
//! 【重要】ディープスリープ中はHP SRAM（メインメモリ）の内容が保持されません。
//! 復帰するとプログラムは電源投入時と同じように「最初から」実行し直されます。
//! 変数の値は消えるので、残したいデータはLP SRAMやフラッシュに置く必要があります
//! （esp-halでは #[ram(persistent)] 属性などの手段があります）。
//!
//! ESP32-C6のディープスリープでGPIO復帰に使えるのは
//! LP（低消費電力）ドメインにあるGPIO0〜GPIO7だけです。
//! esp-halではEXT1ウェイクアップ（Ext1WakeupSource）でピンとレベルを指定します。
//! ※ GpioWakeupSourceという似た名前のAPIもありますが、そちらは
//!   ライトスリープ専用で、ディープスリープからの復帰には使えません。
//!
//! 配線:
//! - GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//! - GPIO7 → 抵抗10kΩ → 3V3（プルアップ。スリープ中の誤動作防止に必須）
//! - GPIO7 → タクトスイッチ → GND（ボタンを押すとLowになり復帰する）
//!
//! 注意: rtc_cntl（スリープ関連）はesp-halの unstable API です。

#![no_std]
#![no_main]

use core::time::Duration as CoreDuration;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig, RtcPinWithResistors};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel};
use esp_hal::rtc_cntl::{Rtc, reset_reason, wakeup_cause};
use esp_hal::system::Cpu;
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

    // なぜ起動（リセット）したのかを表示する。
    // 電源投入なら PowerOn 系、ディープスリープ復帰なら CoreDeepSleep になる
    info!("リセット要因: {:?}", reset_reason(Cpu::ProCpu));
    // ディープスリープから復帰した場合、その原因（Timer / Ext1 など）が分かる
    info!("復帰要因: {:?}", wakeup_cause());

    // LEDを3回点滅させて「起動した」ことを目でも確認できるようにする
    let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
    for _ in 0..3 {
        led.set_high();
        Timer::after(Duration::from_millis(200)).await;
        led.set_low();
        Timer::after(Duration::from_millis(200)).await;
    }

    info!("5秒後にディープスリープに入ります…");
    Timer::after(Duration::from_secs(5)).await;

    // --- ウェイクアップ要因の準備 ---
    // 1. RTCタイマー: 10秒経ったら復帰する
    let timer_wakeup = TimerWakeupSource::new(CoreDuration::from_secs(10));

    // 2. EXT1（LP GPIO）: GPIO7がLowレベルになったら復帰する。
    //    ESP32-C6ではピンごとに復帰レベル(High/Low)を指定できる
    let mut wake_pin = peripherals.GPIO7;
    let mut wakeup_pins: [(&mut dyn RtcPinWithResistors, WakeupLevel); 1] =
        [(&mut wake_pin, WakeupLevel::Low)];
    let ext1_wakeup = Ext1WakeupSource::new(&mut wakeup_pins);

    info!("おやすみなさい（10秒タイマー / GPIO7=Lowで復帰）");

    // ディープスリープへ。この関数からは戻らず（戻り値 `!`）、
    // 復帰するとプログラムは最初から実行される
    let mut rtc = Rtc::new(peripherals.LPWR);
    rtc.sleep_deep(&[&timer_wakeup, &ext1_wakeup]);
}
