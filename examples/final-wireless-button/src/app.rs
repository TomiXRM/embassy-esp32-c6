//! app: task間の配線と状態機械の定義
//!
//! 責務: アプリ全体の「配線図」。チャネル・Signal・共有フラグの実体を持ち、
//!       各taskへ「口」を渡して起動する。リンク状態（LinkState）の定義と、
//!       それをLEDに表示するtaskもここにある。
//! 依存方向: button / heartbeat / radio / config に依存する（配線役なので
//!           全モジュールを知っている）。bin（main.rs / receiver.rs）は
//!           クレート内ではこのappだけに依存すればよい。
//!
//! 送信側のtask構成と配線:
//!
//! ```text
//! button_task ──BUTTON_EVENTS(Channel)──┐
//!                                        ├─> sender_radio_task ──> 電波(ESP-NOW)
//! heartbeat_task ──HEARTBEAT_TICKS──────┘        │
//!        │                                        │ LINK_STATE(Signal)
//!        └── BUTTON_PRESSED(AtomicBool) ──────────┤
//!                                                 v
//!                                             led_task ──> LED(GPIO10)
//! ```

use core::sync::atomic::AtomicBool;

use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, with_timeout};
use esp_hal::gpio::{Input, Output};
use esp_radio::esp_now::EspNow;

use crate::button::{self, ButtonEvent};
use crate::config;
use crate::heartbeat::{self, HeartbeatTick};
use crate::radio;

/// 送信側のリンク状態（状態機械）。
///
/// ```text
///  Idle ──ボタン押下──> Sending ──ACK受信──> Idle
///                          │
///                          └─(最後のACKから5秒以上失敗)─> Error ──ACK受信──> Idle
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// 待機中（正常）
    Idle,
    /// イベント送信中（ACK待ち・再送中）
    Sending,
    /// ACKが長時間得られない（LEDを高速点滅して知らせる）
    Error,
}

/// ボタンtask → radio task へのイベント（容量4のキュー）
static BUTTON_EVENTS: Channel<CriticalSectionRawMutex, ButtonEvent, 4> = Channel::new();

/// ハートビートtask → radio task への周期送信依頼（容量2で十分）
static HEARTBEAT_TICKS: Channel<CriticalSectionRawMutex, HeartbeatTick, 2> = Channel::new();

/// 「今ボタンが押されているか」の共有フラグ
/// （button taskが書き、radio taskがハートビートに載せるため読む）
static BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);

/// リンク状態の通知（radio taskが書き、led taskが読む。最新値のみ保持）
static LINK_STATE: Signal<CriticalSectionRawMutex, LinkState> = Signal::new();

/// 送信側の全taskを配線して起動する。binのmainからはこれを呼ぶだけ
pub fn spawn_sender_tasks(
    spawner: &Spawner,
    button: Input<'static>,
    led: Output<'static>,
    esp_now: EspNow<'static>,
) {
    info!("[app] 送信側タスクを起動します");
    // 各task関数はSpawnTokenの生成結果を返す
    // （タスク定義は各1個ずつなのでunwrapで問題ない）
    spawner.spawn(button::button_task(button, BUTTON_EVENTS.sender(), &BUTTON_PRESSED).unwrap());
    spawner.spawn(heartbeat::heartbeat_task(HEARTBEAT_TICKS.sender()).unwrap());
    spawner.spawn(
        radio::sender_radio_task(
            esp_now,
            BUTTON_EVENTS.receiver(),
            HEARTBEAT_TICKS.receiver(),
            &BUTTON_PRESSED,
            &LINK_STATE,
        )
        .unwrap(),
    );
    spawner.spawn(led_task(led).unwrap());
}

/// 受信側のメイン処理。binのmainからはこれを呼ぶだけ
/// （実体はradioモジュールの受信ループ）
pub async fn run_receiver(esp_now: EspNow<'static>, led: Output<'static>) -> ! {
    radio::receiver_loop(esp_now, led).await
}

/// 送信側のLED表示task。
/// エラー状態のときだけLED（GPIO10）を高速点滅させ、
/// 正常（Idle/Sending）に戻ったら消灯する。
#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    let mut state = LinkState::Idle;
    loop {
        match state {
            LinkState::Error => {
                // 点滅しながら、状態変化の通知も待つ（先に来た方を処理）
                let blink = Duration::from_millis(config::ERROR_BLINK_MS);
                match with_timeout(blink, LINK_STATE.wait()).await {
                    Ok(new_state) => {
                        state = new_state;
                        if state != LinkState::Error {
                            led.set_low(); // 復帰したら消灯に戻す
                        }
                    }
                    Err(_) => led.toggle(), // タイムアウト＝点滅を続ける
                }
            }
            _ => {
                // 正常時は消灯し、次の状態変化をひたすら待つ
                led.set_low();
                state = LINK_STATE.wait().await;
            }
        }
    }
}
