//! 20-etm: CPUを介さない配線 — Event Task Matrix (ETM)
//!
//! ETMは「あるペリフェラルのイベント」を「別のペリフェラルのタスク」に
//! ハードウェア内部で直結する仕組みです。通常、ボタンでLEDを切り替えるには
//! 割り込み→CPUがLEDを操作、という経路が必要ですが、ETMなら
//! イベント（例: GPIOの立ち下がりエッジ）がタスク（例: GPIOのトグル）を
//! 直接起動します。割り込みすら使わず、CPUは一切関与しません。
//!
//! この例では2本のETMチャネルを張ります:
//! - チャネル0: BOOTボタン(GPIO9)の立ち下がりエッジ → GPIO10のLEDをトグル
//! - チャネル1: SYSTIMERのアラーム(500ms周期) → GPIO11のLEDをトグル
//!
//! セットアップ後、CPU(メインタスク)はただ眠っているだけですが、
//! ボタンでLED1が切り替わり、LED2は勝手に点滅し続けます。
//!
//! 配線:
//! - GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//! - GPIO11 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//! - GPIO9はボード上のBOOTボタン（押すとGNDに落ちる）なので配線不要
//!
//! 注意: ETMはesp-halの unstable API です（将来のバージョンで変わる可能性があります）。
//! なお、Embassyの時刻ドライバにはTIMG0を渡しているため、SYSTIMERはこの例で
//! 自由に使えます。

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::etm::Etm;
use esp_hal::gpio::etm::{Channels, InputConfig, OutputConfig};
use esp_hal::gpio::{Level, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::time::Duration as HalDuration;
use esp_hal::timer::PeriodicTimer;
use esp_hal::timer::systimer::{SystemTimer, etm::Event as SystimerEvent};
use esp_hal::timer::timg::TimerGroup;
// defmtログの出口を選ぶ: probe-rsではrtt-target、espflashではesp-printlnをリンクする
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    // probe-rsモードではRTTを初期化し、defmtのグローバルロガーを起動する
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // --- GPIO側のETMチャネル（イベント8本+タスク8本）を取り出す ---
    let gpio_ext = Channels::new(peripherals.GPIO_SD);

    // --- チャネルA: BOOTボタン(GPIO9)の立ち下がり → GPIO10トグル ---
    // イベント: GPIO9が High→Low になった瞬間（ボタンを押した瞬間）
    // BOOTボタンは押すとGNDにつながるので、離しているときのために内部プルアップを有効化
    let button_event = gpio_ext
        .channel0_event
        .falling_edge(peripherals.GPIO9, InputConfig { pull: Pull::Up });
    // タスク: GPIO10の出力レベルを反転する
    let led_task = gpio_ext.channel0_task.toggle(
        peripherals.GPIO10,
        OutputConfig {
            open_drain: false,
            pull: Pull::None,
            initial_state: Level::Low,
        },
    );

    // --- チャネルB: SYSTIMERのアラーム0(500ms周期) → GPIO11トグル ---
    // イベント: SYSTIMERアラーム0の発火パルス
    let syst = SystemTimer::new(peripherals.SYSTIMER);
    let alarm0 = syst.alarm0;
    let timer_event = SystimerEvent::new(&alarm0);
    // タスク: GPIO11の出力レベルを反転する
    let led2_task = gpio_ext.channel1_task.toggle(
        peripherals.GPIO11,
        OutputConfig {
            open_drain: false,
            pull: Pull::None,
            initial_state: Level::Low,
        },
    );

    // --- ETM本体でイベントとタスクを結線する ---
    let etm = Etm::new(peripherals.ETM);
    // 重要: setup()が返す「結線済みチャネル」は、dropされるとチャネルが
    // 無効化されてしまいます。ループの前で変数に束縛して持ち続けること！
    let _channel_a = etm.channel0.setup(&button_event, &led_task);
    let _channel_b = etm.channel1.setup(&timer_event, &led2_task);

    // アラーム0を500ms周期で発火させる（発火のたびにETM経由でGPIO11がトグル
    // = LED2は1秒周期で点滅する）。このタイマーもdropすると止まるので保持する
    let mut periodic = PeriodicTimer::new(alarm0);
    periodic.start(HalDuration::from_millis(500)).unwrap();

    info!("ETMの結線が完了しました。ここから先、LEDの制御にCPUは関与しません");

    loop {
        // CPUがやることはもう何もない。眠っていてもETMは動き続ける
        info!("CPUは寝ています。それでもボタンでLEDが切り替わります");
        Timer::after(Duration::from_secs(5)).await;
    }
}
