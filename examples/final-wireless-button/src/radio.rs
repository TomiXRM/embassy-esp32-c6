//! radio: ESP-NOW送受信task（再送・ACK待ちを含む）
//!
//! 責務: 電波を扱う処理をここに集約する。
//!   - 送信側: イベント/ハートビートの実送信、seqの採番、ACK待ちと再送、
//!             リンク状態（Idle/Sending/Error）の判定
//!   - 受信側: 受信ループ、重複排除、ACK返信、LEDへの状態反映
//! 依存方向: protocol / config / error に依存。
//!           チャネルの「口」や状態通知先（app::LinkStateのSignal）は
//!           appから引数で受け取る（配線の決定権はappにある）。

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::{debug, info, warn};
use embassy_futures::select::{Either3, select3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, with_timeout};
use esp_hal::gpio::{Level, Output};
use esp_radio::esp_now::{BROADCAST_ADDRESS, EspNow, EspNowWifiInterface, PeerInfo};

use crate::app::LinkState;
use crate::button::ButtonEvent;
use crate::config;
use crate::error::TxError;
use crate::heartbeat::HeartbeatTick;
use crate::protocol::{DedupTable, Packet};

// ---------------------------------------------------------------------------
// 送信側
// ---------------------------------------------------------------------------

/// 送信側のESP-NOW担当task。
///
/// 3つの入力を並行して待つ:
/// 1. ボタンイベント（button task から） → seqを採番して送信し、ACKを待つ。
///    ACKが来なければ再送（最大 config::MAX_SEND_ATTEMPTS 回）
/// 2. ハートビート依頼（heartbeat task から） → 現在のボタン状態を載せて
///    送信するだけ（再送しない）
/// 3. ESP-NOWの受信 → ACKなら「最後にACKを受けた時刻」を更新
///
/// 最後のACKから config::LINK_DOWN_AFTER_MS 以上たつとErrorへ遷移し、
/// ACKが再び届いたらIdleへ復帰する（LED表示は app::led_task が担当）。
#[embassy_executor::task]
pub async fn sender_radio_task(
    mut esp_now: EspNow<'static>,
    button_events: Receiver<'static, CriticalSectionRawMutex, ButtonEvent, 4>,
    heartbeat_ticks: Receiver<'static, CriticalSectionRawMutex, HeartbeatTick, 2>,
    pressed: &'static AtomicBool,
    link_state: &'static Signal<CriticalSectionRawMutex, LinkState>,
) {
    // 全パケット共通の通し番号。受信側はこれで重複を見分ける
    let mut seq: u32 = 0;
    // 最後にACKを受け取った時刻（起動直後は「今」として5秒の猶予を持たせる）
    let mut last_ack = Instant::now();
    let mut state = LinkState::Idle;

    loop {
        match select3(
            button_events.receive(),
            heartbeat_ticks.receive(),
            esp_now.receive_async(),
        )
        .await
        {
            // --- 1. ボタンイベント → ACK必須で送信（再送あり） ---
            Either3::First(ButtonEvent::Pressed) => {
                seq = seq.wrapping_add(1);
                let packet = Packet::Event { seq };
                set_state(&mut state, LinkState::Sending, link_state);

                match send_event_with_retry(&mut esp_now, &packet).await {
                    Ok(attempts) => {
                        last_ack = Instant::now();
                        info!(
                            "[送信] イベント seq={} 送信成功（{}回目でACK）",
                            seq, attempts
                        );
                        set_state(&mut state, LinkState::Idle, link_state);
                    }
                    Err(e) => {
                        warn!(
                            "[送信] イベント seq={} 失敗: {:?}（{}回送ってもACKなし）",
                            seq,
                            e,
                            config::MAX_SEND_ATTEMPTS
                        );
                        // 失敗が続いている（最後のACKから一定時間経過）ならエラー状態へ
                        if ack_is_stale(last_ack) {
                            enter_error(&mut state, link_state);
                        } else {
                            set_state(&mut state, LinkState::Idle, link_state);
                        }
                    }
                }
            }

            // --- 2. ハートビート依頼 → 現在のボタン状態を載せて送るだけ ---
            Either3::Second(HeartbeatTick) => {
                seq = seq.wrapping_add(1);
                let packet = Packet::Heartbeat {
                    seq,
                    pressed: pressed.load(Ordering::Relaxed),
                };
                let bytes = packet.to_bytes();
                if let Err(e) = esp_now.send_async(&BROADCAST_ADDRESS, &bytes).await {
                    warn!("[送信] ハートビート seq={} 送信失敗: {:?}", seq, e);
                } else {
                    debug!("[送信] ハートビート seq={}", seq);
                }
                // ハートビートは再送しないが、ACKが長く途絶えていないかはここで確認
                // （500ms周期なので、およそ0.5秒ごとのチェックになる）
                if ack_is_stale(last_ack) {
                    enter_error(&mut state, link_state);
                }
            }

            // --- 3. 受信 → ACKなら「リンク生存」の証拠として記録 ---
            Either3::Third(received) => {
                if let Ok(Packet::Ack { seq: acked }) = Packet::from_bytes(received.data()) {
                    last_ack = Instant::now();
                    debug!("[送信] ACK受信 seq={}", acked);
                    if state == LinkState::Error {
                        info!("[送信] ACKが戻ったのでエラー状態から復帰します");
                    }
                    set_state(&mut state, LinkState::Idle, link_state);
                }
                // ACK以外（他の端末のブロードキャストなど）は無視する
            }
        }
    }
}

/// イベントパケットを送り、ACKが返るまで再送するヘルパー。
/// 成功したら「何回目の送信で成功したか」を返す。
async fn send_event_with_retry(
    esp_now: &mut EspNow<'static>,
    packet: &Packet,
) -> Result<u8, TxError> {
    let bytes = packet.to_bytes();
    let want_seq = packet.seq();
    let mut send_failures = 0u8;

    for attempt in 1..=config::MAX_SEND_ATTEMPTS {
        // 送信（電波に乗せる）。失敗したらACK待ちを飛ばして再試行
        if let Err(e) = esp_now.send_async(&BROADCAST_ADDRESS, &bytes).await {
            warn!("[送信] イベント送信エラー（{}回目）: {:?}", attempt, e);
            send_failures += 1;
            continue;
        }

        // ACK待ち（config::ACK_TIMEOUT_MS でタイムアウト → 再送へ）
        let timeout = Duration::from_millis(config::ACK_TIMEOUT_MS);
        match with_timeout(timeout, wait_for_ack(esp_now, want_seq)).await {
            Ok(()) => return Ok(attempt),
            Err(_) => {
                debug!(
                    "[送信] seq={} のACKが{}ms以内に来ない → 再送します（{}回目まで送信済み）",
                    want_seq,
                    config::ACK_TIMEOUT_MS,
                    attempt
                );
            }
        }
    }

    // 一度も電波に乗らなかったのか、ACKが来なかったのかを区別して返す
    if send_failures == config::MAX_SEND_ATTEMPTS {
        Err(TxError::SendFailed)
    } else {
        Err(TxError::AckTimeout)
    }
}

/// 指定したseqへのACKが届くまで受信し続ける（タイムアウトは呼び出し側で掛ける）
async fn wait_for_ack(esp_now: &mut EspNow<'static>, want_seq: u32) {
    loop {
        let received = esp_now.receive_async().await;
        if let Ok(Packet::Ack { seq }) = Packet::from_bytes(received.data()) {
            if seq == want_seq {
                return;
            }
        }
        // 目的以外のパケットは読み捨てて待ち続ける
    }
}

/// 最後のACKから config::LINK_DOWN_AFTER_MS 以上たっているか
fn ack_is_stale(last_ack: Instant) -> bool {
    last_ack.elapsed() >= Duration::from_millis(config::LINK_DOWN_AFTER_MS)
}

/// 状態が変わったときだけSignalで通知する（LED taskが受け取る）
fn set_state(
    current: &mut LinkState,
    new: LinkState,
    signal: &Signal<CriticalSectionRawMutex, LinkState>,
) {
    if *current != new {
        *current = new;
        signal.signal(new);
    }
}

/// エラー状態へ遷移する（ログ付き）
fn enter_error(current: &mut LinkState, signal: &Signal<CriticalSectionRawMutex, LinkState>) {
    if *current != LinkState::Error {
        warn!(
            "[送信] {}ms以上ACKなし → エラー状態（LED高速点滅）に入ります",
            config::LINK_DOWN_AFTER_MS
        );
    }
    set_state(current, LinkState::Error, signal);
}

// ---------------------------------------------------------------------------
// 受信側
// ---------------------------------------------------------------------------

/// 受信側のメインループ。
///
/// - パケットを受信 → 解読 → (送信元MAC, seq) で重複判定
/// - 重複でもACKは返す（送信側の再送を止めるため）。処理は新規のみ行う
/// - Event/Heartbeatをログに残し、ボタン状態をLED（GPIO10）へ反映
/// - config::HEARTBEAT_LOST_MS 以上何も届かなければ「送信側ロスト」を警告
pub async fn receiver_loop(mut esp_now: EspNow<'static>, mut led: Output<'static>) -> ! {
    let mut dedup: DedupTable<{ config::MAX_PEERS }> = DedupTable::new();
    // 直前まで受信できていたか（ロスト/復帰のログを1回ずつ出すため）
    let mut link_alive = false;

    info!(
        "[受信] 受信待ちを開始します（チャネル{}）",
        config::WIFI_CHANNEL
    );

    loop {
        let timeout = Duration::from_millis(config::HEARTBEAT_LOST_MS);
        match with_timeout(timeout, esp_now.receive_async()).await {
            // --- タイムアウト: ハートビートが途絶えた ---
            Err(_) => {
                warn!(
                    "[受信] {}ms以上ハートビートなし → 送信側をロストした可能性",
                    config::HEARTBEAT_LOST_MS
                );
                link_alive = false;
            }

            // --- 何か受信した ---
            Ok(received) => {
                let src = received.info.src_address;
                match Packet::from_bytes(received.data()) {
                    Err(e) => {
                        // 別プロトコルのパケットや壊れたパケットは捨てる
                        warn!(
                            "[受信] 解読できないパケット: {:?}（src={=[u8]:02x}）",
                            e, src
                        );
                    }
                    Ok(Packet::Ack { seq }) => {
                        // 受信側にACKが届くのは想定外（送信側が返すことはない）
                        debug!("[受信] 想定外のACKを無視 seq={} src={=[u8]:02x}", seq, src);
                    }
                    Ok(packet) => {
                        if !link_alive {
                            info!("[受信] 送信側 {=[u8]:02x} からの受信を開始/再開", src);
                            link_alive = true;
                        }

                        // ACKはユニキャスト（相手のMAC宛て）なので、
                        // 事前にピア登録が必要（ブロードキャストとの違いに注意）
                        ensure_peer(&esp_now, &src);
                        let ack = Packet::Ack { seq: packet.seq() }.to_bytes();
                        if let Err(e) = esp_now.send_async(&src, &ack).await {
                            warn!("[受信] ACK送信失敗 seq={}: {:?}", packet.seq(), e);
                        }

                        // 重複（ACKが失われて再送されたパケット）なら処理しない。
                        // ※ACKは上で返済み。これで送信側の再送は止まる
                        if !dedup.check_and_update(&src, packet.seq()) {
                            info!(
                                "[受信] 重複パケット seq={}（再送と判定）→ ACKのみ返して無視",
                                packet.seq()
                            );
                            continue;
                        }

                        match packet {
                            Packet::Event { seq } => {
                                info!("[受信] ボタンイベント! seq={} src={=[u8]:02x}", seq, src);
                                // イベントは「押された」の合図なのでLEDを点灯。
                                // 離された状態は次のハートビートで反映される
                                led.set_high();
                            }
                            Packet::Heartbeat { seq, pressed } => {
                                info!(
                                    "[受信] ハートビート seq={} ボタン={}",
                                    seq,
                                    if pressed { "押下中" } else { "離し中" }
                                );
                                // LEDにボタン状態をミラーリング
                                led.set_level(if pressed { Level::High } else { Level::Low });
                            }
                            Packet::Ack { .. } => unreachable!(), // 上の分岐で処理済み
                        }
                    }
                }
            }
        }
    }
}

/// 送信元がピア未登録なら登録する（ユニキャスト送信の前提条件）
fn ensure_peer(esp_now: &EspNow<'_>, mac: &[u8; 6]) {
    if esp_now.peer_exists(mac) {
        return;
    }
    let result = esp_now.add_peer(PeerInfo {
        interface: EspNowWifiInterface::Station,
        peer_address: *mac,
        lmk: None,     // 暗号化キーなし
        channel: None, // 現在のチャネルをそのまま使う
        encrypt: false,
    });
    match result {
        Ok(()) => info!("[受信] ピア登録: {=[u8]:02x}", *mac),
        Err(e) => warn!("[受信] ピア登録失敗 {=[u8]:02x}: {:?}", *mac, e),
    }
}
