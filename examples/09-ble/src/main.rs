//! 09-ble: BLE（Bluetooth Low Energy）GATTペリフェラル
//!
//! ESP32-C6をBLE（Bluetooth Low Energy）のペリフェラル（周辺機器）として動かし、
//! 「C6-BUTTON」という名前でアドバタイズ（存在通知）します。
//! スマートフォンのBLE（Bluetooth Low Energy）スキャナアプリ
//! （nRF ConnectやLightBlueなど）から接続すると、
//! 標準のバッテリーサービス（Battery Service）が見えます。
//!
//! 提供する特性（Characteristic）は2つ:
//! - バッテリー残量（標準UUID）: 2秒ごとに値を1ずつ減らして通知（Notify）します。
//!   実際の電池電圧ではなく、通知の仕組みを見せるためのデモ値です。
//! - BOOTボタン状態（独自UUID）: ボード上のBOOTボタン（GPIO9）を
//!   押す/離すたびに true/false を通知します。
//!
//! ※ ESP32-C6が対応する無線は BLE（Bluetooth Low Energy）のみで、
//!    Bluetooth Classic（イヤホン等で使う従来規格）には対応していません。
//!
//! 構成: esp-radio（BLEコントローラ）+ trouble-host（BLEホストスタック）。
//! esp-rs公式の bas_peripheral 例（esp-radio-v0.18.0 タグ）をベースにしています。
//!
//! 配線: 不要（ボード上のBOOTボタン = GPIO9 をそのまま使用）

#![no_std]
#![no_main]

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_time::Timer;
use esp_backtrace as _;
// defmt の global_logger をリンクする。probe-rs では rtt-target、
// espflash では esp-println がそれぞれ defmt ログの出口になる。
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
#[cfg(feature = "espflash")]
use esp_println as _;
use esp_radio::ble::controller::BleConnector;
#[cfg(feature = "probe-rs")]
use rtt_target as _;
use trouble_host::prelude::*;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// 同時接続数の上限（この例ではセントラル1台のみ）
const CONNECTIONS_MAX: usize = 1;
/// L2CAPチャネル数の上限（シグナリング + ATTの2本）
const L2CAP_CHANNELS_MAX: usize = 2;

// GATTサーバー定義。#[gatt_server]マクロがサービス一覧から
// 属性テーブル（BLE（Bluetooth Low Energy）のデータベース）を生成する
#[gatt_server]
struct Server {
    battery_service: BatteryService,
}

/// バッテリーサービス（Bluetooth SIG標準のサービスUUID 0x180F）
#[gatt_service(uuid = service::BATTERY)]
struct BatteryService {
    /// バッテリー残量（標準UUID 0x2A19）。読み取りと通知に対応。
    /// この例では実測値ではなく、2秒ごとに減らすデモ値を入れる
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify, value = 100)]
    level: u8,
    /// BOOTボタンの状態（独自の128ビットUUID）。押されていればtrue。
    /// 標準サービスに独自の特性を追加する例でもある
    #[characteristic(
        uuid = "408813df-5dd4-1f87-ec11-cdb001100000",
        read,
        notify,
        value = false
    )]
    button_pressed: bool,
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    // probe-rs 経由の defmt(RTT) を初期化する（espflash 時は何もしない）
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // 無線スタックはヒープを使うため、アロケータを用意する（公式例と同じ72KB）
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // BOOTボタン（GPIO9）。ボタンとGNDの間に入っているので内部プルアップを
    // 有効にし、押されるとLowになる
    let button_config = InputConfig::default().with_pull(Pull::Up);
    let button = Input::new(peripherals.GPIO9, button_config);

    // BLE（Bluetooth Low Energy）コントローラ（電波を扱う下位層）を初期化し、
    // HCIというインターフェース経由でホストスタック（trouble-host）につなぐ
    let connector = BleConnector::new(peripherals.BT, Default::default()).unwrap();
    let controller: ExternalController<_, 1> = ExternalController::new(connector);

    ble_peripheral_run(controller, button).await;
}

/// BLE（Bluetooth Low Energy）スタック全体を動かす
async fn ble_peripheral_run<C>(controller: C, mut button: Input<'_>)
where
    C: Controller,
{
    // デバイスアドレス。テストしやすいよう固定のランダムアドレスを使う
    // （本来はチップのMACアドレスなどから作る）
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    // Address は defmt::Format 非対応のため、生の6バイトを16進で出す
    info!("デバイスアドレス: {=[u8]:02x}", address.addr.raw());

    // ホストスタックが使うメモリ（接続・チャネル管理領域）を確保
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    info!("GATTサーバーを起動し、アドバタイズを開始します");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "C6-BUTTON",
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .unwrap();

    // ble_taskはスタックの心臓部で、常に動かし続ける必要がある。
    // joinでアドバタイズ・接続処理と並行実行する
    let _ = join(ble_task(runner), async {
        loop {
            match advertise("C6-BUTTON", &mut peripheral, &server).await {
                Ok(conn) => {
                    // 接続が確立したら、GATTイベント処理と通知タスクを並行実行。
                    // どちらかが終わったら（=切断されたら）アドバタイズに戻る
                    let a = gatt_events_task(&server, &conn);
                    let b = notify_task(&server, &conn, &mut button);
                    select(a, b).await;
                }
                Err(e) => {
                    panic!("[adv] エラー: {:?}", e);
                }
            }
        }
    })
    .await;
}

/// BLE（Bluetooth Low Energy）ホストスタックの内部処理を回し続けるタスク。
/// 他のBLE処理と並行して常時動かす必要がある
async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            panic!("[ble_task] エラー: {:?}", e);
        }
    }
}

/// 接続が切れるまでGATTイベント（読み取り・書き込み要求）を処理する
async fn gatt_events_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) -> Result<(), Error> {
    let level = server.battery_service.level;
    let button_pressed = server.battery_service.button_pressed;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == level.handle {
                            info!(
                                "[gatt] バッテリー残量が読み取られました: {}",
                                server.get(&level)
                            );
                        } else if event.handle() == button_pressed.handle {
                            info!(
                                "[gatt] ボタン状態が読み取られました: {}",
                                server.get(&button_pressed)
                            );
                        }
                    }
                    GattEvent::Write(event) => {
                        info!("[gatt] 書き込み要求: {=[u8]:02x}", event.data());
                    }
                    _ => {}
                };
                // 応答を返す。dropでも送られるが、確実に送るため明示的に呼ぶ
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("[gatt] 応答送信エラー: {}", e),
                };
            }
            _ => {} // その他の接続イベントは無視
        }
    };
    info!("[gatt] 切断されました: {}", reason);
    Ok(())
}

/// アドバタイズ（接続可能な状態で存在を知らせる）を行い、
/// セントラル（スマートフォンなど）からの接続を待つ
async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    // アドバタイズパケット（最大31バイト）にフラグ・サービスUUID・名前を詰める
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            // バッテリーサービス(0x180F)を持っていることを知らせる
            AdStructure::ServiceUuids16(&[[0x0f, 0x18]]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] アドバタイズ中（名前: {}）", name);
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] 接続されました");
    Ok(conn)
}

/// 接続中のセントラルへ通知（Notify）を送るタスク。
/// - BOOTボタンの変化を待ち、押す/離すたびにボタン状態を通知
/// - 2秒ごとにバッテリー残量（デモ値）を1減らして通知
/// 通知に失敗したら（=切断されたら）終了する
async fn notify_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    button: &mut Input<'_>,
) {
    let level = server.battery_service.level;
    let button_char = server.battery_service.button_pressed;
    let mut battery: u8 = 100;
    loop {
        // ボタンのエッジ（変化）待ちと2秒タイマーを並行して待つ
        match select(button.wait_for_any_edge(), Timer::after_secs(2)).await {
            Either::First(_) => {
                // チャタリング（接点の細かい振動）が落ち着くまで少し待つ
                Timer::after_millis(20).await;
                let pressed = button.is_low(); // プルアップなので押下=Low
                info!("[notify] ボタン状態を通知: {}", pressed);
                if button_char.notify(conn, &pressed).await.is_err() {
                    info!("[notify] 通知に失敗しました（切断）");
                    break;
                }
            }
            Either::Second(_) => {
                // デモ用: 実測値ではなく単に1ずつ減らす（0になったら100へ戻す）
                battery = if battery == 0 { 100 } else { battery - 1 };
                info!("[notify] バッテリー残量を通知: {}", battery);
                if level.notify(conn, &battery).await.is_err() {
                    info!("[notify] 通知に失敗しました（切断）");
                    break;
                }
            }
        }
    }
}
