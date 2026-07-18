//! 10-esp-now: ESP-NOWでボード同士が直接通信
//!
//! ESP-NOWはEspressif独自の通信方式で、Wi-Fiの電波を使いながら
//! アクセスポイント（ルーター）なしでボード同士が直接パケットを
//! やり取りできます。接続手続きが不要で低遅延なのが特徴です。
//!
//! このプログラムは:
//! - 1秒ごとにブロードキャスト（全員宛て）でパケットを送信
//!   （内容: 通し番号カウンタ4バイト + 自分のMACアドレス6バイト）
//! - 同時に受信も待ち、届いたパケットの送信元MACアドレスと内容をログ表示
//!
//! 同じプログラムを2台のESP32-C6に書き込むと、お互いのパケットが
//! 受信ログに表示されます。1台だけでも送信ログは確認できます。
//!
//! 構成: esp-radio 0.18 の esp-now 機能（unstable feature）。
//! esp-rs公式の embassy_esp_now 例（esp-radio-v0.18.0 タグ）をベースにしています。
//!
//! 配線: 不要

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::efuse::{InterfaceMacAddress, interface_mac_address};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::esp_now::BROADCAST_ADDRESS;
// defmtログの出口を選ぶ: probe-rsではrtt-target、espflashではesp-printlnをリンクする
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// 送信するパケットの長さ（カウンタ4バイト + MACアドレス6バイト）
const PAYLOAD_LEN: usize = 10;

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    // probe-rsモードではRTTを初期化し、defmtのグローバルロガーを起動する
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // 無線スタックはヒープを使うため、アロケータを用意する（公式例と同じ72KB）
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // Wi-Fiドライバを初期化すると、ESP-NOWインターフェースも一緒に得られる。
    // コントローラ本体はESP-NOWだけなら操作不要（ただしdropすると
    // 無線が止まるので、変数名を _controller にして保持しておく）
    let (_controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default()).unwrap();
    let mut esp_now = interfaces.esp_now;

    // 送受信するボード同士は同じWi-Fiチャネルに合わせる必要がある
    esp_now.set_channel(11).unwrap();

    info!("ESP-NOWバージョン: {}", esp_now.version().unwrap());

    // 自分のMACアドレス（工場出荷時にeFuseへ書き込まれている固有番号）。
    // ESP-NOWはWi-FiのStation用MACアドレスで送信される
    let mac = interface_mac_address(InterfaceMacAddress::Station);
    // MACアドレスは6バイトを16進で表示する（defmtのバイト列ヒント）
    info!("自分のMACアドレス: {=[u8]:02x}", mac.as_bytes());

    let mut counter: u32 = 0;
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        // 「1秒タイマー」と「パケット受信」を並行して待ち、
        // 先に完了した方を処理する
        match select(ticker.next(), esp_now.receive_async()).await {
            // 1秒経過 → ブロードキャスト送信
            Either::First(_) => {
                counter = counter.wrapping_add(1);
                let mut payload = [0u8; PAYLOAD_LEN];
                payload[..4].copy_from_slice(&counter.to_le_bytes());
                payload[4..].copy_from_slice(mac.as_bytes());

                let status = esp_now.send_async(&BROADCAST_ADDRESS, &payload).await;
                info!("送信 counter={} 結果={:?}", counter, status);
            }
            // パケット受信 → 送信元MACアドレスと内容をログ表示
            Either::Second(received) => {
                let data = received.data();
                // MAC・データはバイト列なので16進表示する（defmtのバイト列ヒント）
                info!(
                    "受信 送信元MAC={=[u8]:02x} 宛先MAC={=[u8]:02x} データ={=[u8]:02x}",
                    received.info.src_address, received.info.dst_address, data
                );
                // このプログラム同士なら10バイト（カウンタ+MAC）のはず
                if data.len() == PAYLOAD_LEN {
                    let peer_counter = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    info!(
                        "  → 相手のカウンタ={} 相手のMAC={=[u8]:02x}",
                        peer_counter,
                        &data[4..]
                    );
                }
            }
        }
    }
}
