//! heartbeat: 500ms周期送信task（送信側）
//!
//! 責務: 一定周期（config::HEARTBEAT_PERIOD_MS）で「ハートビートを送って」と
//!       radio taskへ依頼する。実際の電波送信・seq採番はradio taskの仕事。
//! 依存方向: config / power のみ。チャネルの実体は app が持ち、
//!           このtaskは引数で「送信の口」を受け取るだけ。

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Ticker};

use crate::{config, power};

/// ハートビート送信の依頼（中身のない合図。ボタン状態はradio taskが読む）
#[derive(Debug, Clone, Copy)]
pub struct HeartbeatTick;

/// 500msごとにハートビート送信を依頼するtask
#[embassy_executor::task]
pub async fn heartbeat_task(ticks: Sender<'static, CriticalSectionRawMutex, HeartbeatTick, 2>) {
    let mut ticker = Ticker::every(Duration::from_millis(config::HEARTBEAT_PERIOD_MS));
    loop {
        ticker.next().await;

        // 省電力フック: 次の周期までの待ち時間はスリープ候補（今は何もしない）
        power::before_idle();

        // radio taskがACK待ちなどで忙しいときは無理に積まない（try_send）。
        // ハートビートは「最新の生存確認」が届けばよく、
        // 古い依頼を溜め込む意味がないため、あふれた分は捨てる。
        let _ = ticks.try_send(HeartbeatTick);
    }
}
