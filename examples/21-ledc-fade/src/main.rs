//! 21-ledc-fade: ハードウェアフェード — LEDC任せでLEDをじわっと明滅
//!
//! LEDC(PWM)ペリフェラルには「デューティ比を指定時間かけて自動で変化させる」
//! ハードウェアフェード機能があります。start_duty_fade()で開始すると、
//! あとはLEDCが勝手にデューティ比を少しずつ動かしてくれるので、
//! CPUはフェード中も完全に自由です。
//!
//! この例ではGPIO10のLEDを2秒かけて 0%→100%、また2秒かけて 100%→0% と
//! 繰り返しフェードさせます。フェード中、メインタスクは250msごとに
//! カウンタをログ出力して「CPUが別の仕事をできている」ことを示します。
//!
//! 配線: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//!
//! 注意: LEDCはesp-halの unstable API です（将来のバージョンで変わる可能性があります）。

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::DriveMode;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::{error, info};

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

    // --- LEDC(PWM)の設定（13-adc-pwmと同じ: 5kHz・12bit分解能） ---
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty12Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(5),
        })
        .unwrap();

    // チャンネル0にGPIO10を割り当てる（最初はデューティ0% = 消灯）
    let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO10);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    info!("ハードウェアフェードを開始します（2秒で明→暗→明…）");

    // フェードの向き: (開始デューティ%, 終了デューティ%)
    let mut fade = (0u8, 100u8);
    // CPUが自由であることを示すためのカウンタ
    let mut counter: u32 = 0;

    loop {
        let (start, end) = fade;

        // 2000msかけて start% → end% へハードウェアが自動でフェードする。
        // パラメータが不正（範囲外・時間が長すぎ等）だとErrが返るのでmatchで処理
        match channel0.start_duty_fade(start, end, 2000) {
            Ok(()) => info!("フェード開始: {start}% → {end}% (2000ms)"),
            Err(e) => {
                // ここに来るのはパラメータ設定ミスのとき（教材の値では起きないはず）
                error!("フェードを開始できませんでした: {e:?}");
                Timer::after(Duration::from_secs(1)).await;
                continue;
            }
        }

        // フェードが終わるまで待つ。この間、デューティ比の更新はすべて
        // LEDCハードウェアの仕事。CPUは250msごとに別の仕事（ログ出力）ができる
        while channel0.is_duty_fade_running() {
            counter += 1;
            info!("フェード中もCPUは別の仕事をしています: {counter}");
            Timer::after(Duration::from_millis(250)).await;
        }

        // 向きを反転して繰り返す（0→100 の次は 100→0）
        fade = (end, start);
    }
}
