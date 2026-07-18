//! button: ボタン監視task（送信側）
//!
//! 責務: BOOTボタン（GPIO9）をデバウンス付きで監視し、
//!       - 押された瞬間 → ボタンイベントをチャネルへ送出（radio taskが送信する）
//!       - 押している間 → 共有フラグ（AtomicBool）をtrueに保つ
//!         （ハートビートに載せる「現在のボタン状態」の源になる）
//! 依存方向: config のみ。チャネルやフラグの実体は app が持ち、
//!           このtaskは引数で「送信の口」を受け取るだけ（疎結合）。

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Input;
use log::info;

use crate::config;

/// ボタンtaskからradio taskへ送るイベント
#[derive(Debug, Clone, Copy)]
pub enum ButtonEvent {
    /// ボタンが1回押された
    Pressed,
}

/// ボタン監視task。
/// 07-channelと同じ「エッジ待ち + デバウンス」のパターンです。
#[embassy_executor::task]
pub async fn button_task(
    mut button: Input<'static>,
    events: Sender<'static, CriticalSectionRawMutex, ButtonEvent, 4>,
    pressed: &'static AtomicBool,
) {
    loop {
        // High→Lowの変化（＝押された瞬間）をawaitで待つ
        button.wait_for_falling_edge().await;

        // チャタリング対策: 少し待ってから本当に押されているか確認
        Timer::after(Duration::from_millis(config::DEBOUNCE_MS)).await;
        if button.is_low() {
            // ハートビート用の「現在押されている」フラグを立てる
            pressed.store(true, Ordering::Relaxed);
            info!("[ボタン] 押下を検出 → イベントを送信キューへ");

            // radio taskへ即時送信を依頼（キュー満杯時は空くまで待つ）
            events.send(ButtonEvent::Pressed).await;

            // 離されるまで待ってから次の押下を受け付ける
            button.wait_for_rising_edge().await;
            Timer::after(Duration::from_millis(config::DEBOUNCE_MS)).await;
            pressed.store(false, Ordering::Relaxed);
        }
    }
}
