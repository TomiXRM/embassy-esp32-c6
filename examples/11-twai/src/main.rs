//! 11-twai: TWAI（CAN）でフレームを送受信する（セルフテストモード）
//!
//! TWAI0コントローラを500kbpsで動かし、ID 0x123 の標準フレーム（データ4バイト）を
//! 1秒ごとに送信して、自分自身で受信します。
//!
//! この例は「セルフテストモード」で動きます。セルフテストモードでは
//! 送信フレームにACK（他ノードからの受信確認）が不要なため、
//! 外付けのCANトランシーバや相手ノードがなくても1台だけで動作確認できます。
//!
//! 【重要】実際のCANバスに接続する場合（TwaiMode::Normal）:
//! - TJA1051/TJA1050などの外付けCANトランシーバが必ず必要です。
//!   ESP32-C6はトランシーバを内蔵していません。
//! - ESP32-C6のピンをCANバスのCAN_H/CAN_Lへ直接つないでは絶対にいけません。
//!   バスの差動電圧でチップが壊れる恐れがあります（TX/RXはトランシーバにつなぐ）。
//!
//! 配線（この例・セルフテスト用）:
//! - GPIO2 (TX) と GPIO3 (RX) をジャンパワイヤで直接つなぐ
//!
//! 注意: TWAIはesp-halの unstable API です（将来のバージョンで変わる可能性があります）。

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::twai::{BaudRate, EspTwaiFrame, StandardId, TwaiConfiguration, TwaiMode};
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

    // TWAI0を500kbps・セルフテストモードで設定する。
    // new_no_transceiver()はトランシーバなしでピン同士を直結する構成用
    // （TXをオープンドレイン+プルアップに設定してくれる）。
    // 引数の順番は「RXピン, TXピン」なので注意！
    let twai_config = TwaiConfiguration::new_no_transceiver(
        peripherals.TWAI0,
        peripherals.GPIO3, // RX
        peripherals.GPIO2, // TX
        BaudRate::B500K,
        TwaiMode::SelfTest,
    )
    .into_async(); // 非同期(async)版に変換

    // start()で設定を確定し、実際に動くTwaiドライバを得る
    let mut twai = twai_config.start();

    // 送信するフレーム: 標準ID 0x123、データ4バイト。
    // セルフテストモードで自分の送信を自分で受信するには、
    // 「自己受信フレーム」(new_self_reception)として送る必要がある
    let id = StandardId::new(0x123).unwrap();
    let frame = EspTwaiFrame::new_self_reception(id, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();

    info!("TWAIセルフテストを開始します（500kbps, ID=0x123）");

    loop {
        // --- 送信 ---
        match twai.transmit_async(&frame).await {
            Ok(()) => info!("送信OK: {frame:?}"),
            Err(e) => error!("送信エラー: {e:?}"),
        }

        // --- 受信 ---
        // セルフテストモードなので、いま送ったフレームが自分に届く
        match twai.receive_async().await {
            Ok(received) => info!("受信OK: {received:?}"),
            Err(e) => error!("受信エラー: {e:?}"),
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
