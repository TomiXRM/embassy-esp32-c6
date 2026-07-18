//! 06-embassy-tasks: 複数タスクを並行に動かす
//!
//! Embassyの「タスク」を2つ生成（spawn）し、mainと合わせて3つの処理を
//! 1つのCPUコア上で並行に動かします。
//! - タスクA: GPIO10の外付けLEDを500ms間隔で点滅（Tickerで周期実行）
//! - タスクB: 1秒ごとにカウンタを増やしてログに表示
//! - main:    5秒ごとにハートビート（生存確認）をログに表示
//! LEDの所有権はタスクAへ「ムーブ」され、以後mainからは触れません。
//!
//! 配線: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;

use defmt::info;
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// タスクA: LEDを500ms間隔で点滅させる
///
/// 引数`led`は所有権ごと受け取る（ムーブ）ので、このタスクだけが
/// LEDを操作できます。「同じピンを2箇所から触ってしまう」バグを
/// コンパイル時に防げるのがRustの強みです。
#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    // Tickerは「一定周期の繰り返し」に向いています。
    // 処理にかかった時間を差し引いて次の起床時刻を決めるので、
    // Timer::afterの繰り返しよりも周期がずれにくいのが特長です。
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        led.toggle();
        ticker.next().await;
    }
}

/// タスクB: 1秒ごとにカウンタを増やしてログに表示する
#[embassy_executor::task]
async fn counter_task() {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut count: u32 = 0;
    loop {
        ticker.next().await;
        count += 1;
        info!("[タスクB] カウンタ = {}", count);
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // GPIO10を出力に設定。最初は消灯（Low）
    let led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    info!("2つのタスクを起動します");

    // タスクを生成。ledはここでblink_taskにムーブされる。
    // blink_task(led)は「生成トークン」のResultを返し、タスクの空きが
    // ない場合はここでErrになる（各タスク1個ずつなのでunwrapで問題ない）
    spawner.spawn(blink_task(led).unwrap());
    spawner.spawn(counter_task().unwrap());

    // mainもひとつの並行処理として動き続ける
    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("[main] 動作中です（ハートビート）");
    }
}
