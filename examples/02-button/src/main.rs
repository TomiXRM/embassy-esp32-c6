//! 02-button: BOOTボタンでLEDをトグル
//!
//! ボード上のBOOTボタン（GPIO9）を押すたびに、GPIO10に接続した
//! 外付けLEDを点灯⇔消灯と切り替え、押した回数をログに表示します。
//! ボタン入力はポーリングではなく、asyncの「エッジ待ち」で受け取ります。
//!
//! 配線:
//! - LED: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//! - ボタン: 配線不要（ボード上のBOOTボタン = GPIO9 をそのまま使用）

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;

use defmt::info;
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // LED用のGPIO10を出力に設定。最初は消灯（Low）
    let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    // BOOTボタン（GPIO9）を入力に設定。
    // ボタンはGPIO9とGNDの間に入っているので、内部プルアップを有効にして
    // 「離している間はHigh、押すとLow」になるようにします。
    let config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(peripherals.GPIO9, config);

    info!("BOOTボタンを押すとLEDが切り替わります");

    let mut count: u32 = 0;

    loop {
        // High→Lowの変化（＝ボタンが押された瞬間）をawaitで待ちます。
        // 待っている間、CPUは他の仕事ができます（ポーリング不要）。
        button.wait_for_falling_edge().await;

        // チャタリング対策: 機械式ボタンは押した瞬間に接点が細かくバタつくので、
        // 30ms待ってから本当に押されているかを確認します。
        Timer::after(Duration::from_millis(30)).await;
        if button.is_low() {
            count += 1;
            led.toggle();
            info!("ボタンが押されました（{}回目）", count);

            // ボタンが離される（Low→High）まで待ってから次の押下を受け付けます。
            button.wait_for_rising_edge().await;
            // 離すときにもチャタリングが起きるので、少し待って落ち着かせます。
            Timer::after(Duration::from_millis(30)).await;
        }
    }
}
