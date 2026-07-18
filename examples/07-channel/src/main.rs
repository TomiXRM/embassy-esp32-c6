//! 07-channel: チャネルでタスク間通信
//!
//! ボタン担当タスクとLED担当タスクを分け、その間を
//! `embassy_sync::channel::Channel`（メッセージキュー）でつなぎます。
//! - ボタンタスク: BOOTボタン（GPIO9）の押下を検出し、イベントを送信
//! - LEDタスク:    イベントを受信してGPIO10の外付けLEDをトグル。
//!                 3秒間イベントが来なければ「イベントなし」とログ表示
//! 共有変数を直接触らず「メッセージを送る」ことで、タスク同士を
//! 安全に、かつ疎結合に連携させるのがポイントです。
//!
//! 配線:
//! - LED: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//! - ボタン: 配線不要（ボード上のBOOTボタン = GPIO9 をそのまま使用）

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer, with_timeout};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use log::info;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// ボタンから送るイベントの種類。
/// 今回は1種類だけですが、enumにしておけば
/// 「長押し」「ダブルクリック」などを後から増やせます。
#[derive(Debug, Clone, Copy)]
enum ButtonEvent {
    /// ボタンが1回押された
    Pressed,
}

/// タスク間をつなぐチャネル（容量4のメッセージキュー）。
/// staticに置くことで、どのタスクからも参照できます。
/// `CriticalSectionRawMutex`は割り込みを含むどの実行文脈からも
/// 安全に使えるロック方式です（Channel::new()はconstなので初期化子に書ける）。
static CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 4> = Channel::new();

/// ボタンタスク: BOOTボタンの押下を検出してイベントを送信する
#[embassy_executor::task]
async fn button_task(mut button: Input<'static>) {
    loop {
        // High→Lowの変化（＝押された瞬間）をawaitで待つ
        button.wait_for_falling_edge().await;

        // チャタリング対策: 30ms待ってから本当に押されているか確認
        Timer::after(Duration::from_millis(30)).await;
        if button.is_low() {
            // キューが満杯のときは空きが出るまでここで待つ（バックプレッシャ）
            CHANNEL.send(ButtonEvent::Pressed).await;

            // 離されるまで待ってから次の押下を受け付ける
            button.wait_for_rising_edge().await;
            Timer::after(Duration::from_millis(30)).await;
        }
    }
}

/// LEDタスク: イベントを受信してLEDをトグルする
#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    let mut count: u32 = 0;
    loop {
        // receive()に3秒のタイムアウトを付ける。
        // 時間内に受信できればOk(イベント)、できなければErr(TimeoutError)
        match with_timeout(Duration::from_secs(3), CHANNEL.receive()).await {
            Ok(ButtonEvent::Pressed) => {
                count += 1;
                led.toggle();
                info!("[LEDタスク] イベント受信: {}回目 → LEDをトグル", count);
            }
            Err(_) => {
                info!("[LEDタスク] イベントなし（3秒間ボタンが押されていません）");
            }
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // LED用のGPIO10を出力に設定。最初は消灯（Low）
    let led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    // BOOTボタン（GPIO9）を内部プルアップ付きの入力に設定
    // （押すとLowになる。詳しくは02-buttonを参照）
    let config = InputConfig::default().with_pull(Pull::Up);
    let button = Input::new(peripherals.GPIO9, config);

    info!("ボタンタスクとLEDタスクを起動します");

    // button_task(button)などは「生成トークン」のResultを返し、タスクの
    // 空きがない場合はErrになる（各タスク1個ずつなのでunwrapで問題ない）
    spawner.spawn(button_task(button).unwrap());
    spawner.spawn(led_task(led).unwrap());

    // mainはもう仕事がないので待機するだけ
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
