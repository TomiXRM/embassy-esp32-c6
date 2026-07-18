//! 15-ble-hid: BLE（Bluetooth Low Energy）HIDキーボード（HID over GATT / HOGP）
//!
//! ESP32-C6を「BLE（Bluetooth Low Energy）キーボード」として動かす例です。
//! HID over GATT Profile（HOGP）に沿って、HIDサービス（0x1812）を持つ
//! ペリフェラルを「C6-KEYBOARD」という名前でアドバタイズし、
//! BOOTボタン（GPIO9）を押すたびに「a」キーの押下→解放レポートを
//! 通知（Notify）で送ります。
//!
//! GATT構成（HOGPで求められる主な要素）:
//! - HIDサービス（0x1812）
//!   - Protocol Mode（0x2A4E）: Reportプロトコル固定（値=1）
//!   - Report（0x2A4D、Notify）+ Report Reference記述子（0x2908、Input型）
//!   - Report Map（0x2A4B）: キーボードのレポート仕様（HIDディスクリプタ）
//!   - HID Information（0x2A4A） / HID Control Point（0x2A4C）
//! - バッテリーサービス（0x180F）: Battery Level（0x2A19）
//! - デバイス情報サービス（0x180A）: PnP ID（0x2A50）、製造者名（0x2A29）
//!
//! ★正直な注意書き（本サンプルの限界）★
//! 本サンプルはHOGPのGATT構造（Report Reference記述子を含む）を一通り
//! 実装しており、コンパイルは通りますが、**実機での動作・OSとのペアリングは
//! 未検証**です。さらに、実際のOS（iOS/Android/Windows/macOS）がBLE
//! （Bluetooth Low Energy）HIDキーボードを受け入れるには、HOGP仕様上
//! **ペアリング（ボンディング）と通信の暗号化（Security Mode 1 Level 2以上）が
//! 必須**です。教材のバージョン固定（trouble-host 0.6.0、features =
//! ["gatt", "derive"]）では SMP（Security Manager Protocol、`security`
//! feature）を有効にしていないため、暗号化なしの接続となり、
//! 多くのOSはHID入力を拒否します。nRF Connect等のBLEスキャナアプリで
//! GATT構造とレポート通知を観察する用途を想定した学習用サンプルです。
//!
//! 構成: esp-radio（BLEコントローラ）+ trouble-host（BLEホストスタック）。
//! 09-ble と同じ骨組みに、HOGPのGATTテーブルを組み合わせています。
//!
//! 配線: 不要（ボード上のBOOTボタン = GPIO9 をそのまま使用）

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::ble::controller::BleConnector;
use log::{info, warn};
use trouble_host::prelude::*;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// 同時接続数の上限（この例ではセントラル1台のみ）
const CONNECTIONS_MAX: usize = 1;
/// L2CAPチャネル数の上限（シグナリング + ATTの2本）
const L2CAP_CHANNELS_MAX: usize = 2;

/// Report Map（HIDレポートマップ、0x2A4B）の中身。
/// 「このデバイスが送るレポートは何バイトで、各ビットが何を意味するか」を
/// USB HIDと同じ書式（HIDレポートディスクリプタ）で宣言する。
/// ここでは最小構成のキーボード（ブートキーボード互換の8バイトレポート）を定義。
const REPORT_MAP: [u8; 45] = [
    0x05, 0x01, // Usage Page (Generic Desktop) : デスクトップ機器の分類から…
    0x09, 0x06, // Usage (Keyboard)             : 「キーボード」を選ぶ
    0xA1, 0x01, // Collection (Application)     : ここからキーボードの定義
    // --- 1バイト目: 修飾キー（Ctrl/Shift/Alt/GUI）を1ビットずつ、計8ビット ---
    0x05, 0x07, //   Usage Page (Keyboard/Keypad)
    0x19, 0xE0, //   Usage Minimum (0xE0 = 左Ctrl)
    0x29, 0xE7, //   Usage Maximum (0xE7 = 右GUI)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1ビット)
    0x95, 0x08, //   Report Count (8個)
    0x81, 0x02, //   Input (Data, Variable, Absolute) → 修飾キーのビットマップ
    // --- 2バイト目: 予約領域（常に0を送る決まり） ---
    0x95, 0x01, //   Report Count (1個)
    0x75, 0x08, //   Report Size (8ビット)
    0x81, 0x01, //   Input (Constant) → 予約バイト
    // --- 3〜8バイト目: 同時に押されているキーのUsage IDを最大6個 ---
    0x95, 0x06, //   Report Count (6個)
    0x75, 0x08, //   Report Size (8ビット)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x65, //   Logical Maximum (0x65)
    0x05, 0x07, //   Usage Page (Keyboard/Keypad)
    0x19, 0x00, //   Usage Minimum (0)
    0x29, 0x65, //   Usage Maximum (0x65)
    0x81, 0x00, //   Input (Data, Array) → キーコード6個ぶんの配列
    0xC0, // End Collection
];

// 入力レポートは8バイト（USBのブートプロトコルキーボードと同じ形）:
//   [0] 修飾キーのビットマップ（bit0=左Ctrl, bit1=左Shift, bit2=左Alt,
//       bit3=左GUI, bit4〜7=右側の同キー）
//   [1] 予約（常に0）
//   [2..8] 押されているキーのUsage ID（最大6キー同時押し。空きは0）
/// 「a」キー（HID Usage ID 0x04）だけを押した状態のレポート
const KEY_A_DOWN: [u8; 8] = [0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00];
/// 全キーを離した状態のレポート（キーアップはこれを送って表す）
const KEY_ALL_UP: [u8; 8] = [0x00; 8];

// GATTサーバー定義。#[gatt_server]マクロがサービス一覧から
// 属性テーブル（BLE（Bluetooth Low Energy）のデータベース）を生成する
#[gatt_server]
struct Server {
    hid_service: HidService,
    battery_service: BatteryService,
    device_info_service: DeviceInfoService,
}

/// HIDサービス（Bluetooth SIG標準のサービスUUID 0x1812）。
/// HOGPでキーボードとして必要な特性（Characteristic）を並べる
#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
struct HidService {
    /// Protocol Mode（0x2A4E）: 0=ブートプロトコル, 1=レポートプロトコル。
    /// この例はレポートプロトコル固定（値1）。ホストが書き換えても無視する
    #[characteristic(
        uuid = characteristic::PROTOCOL_MODE,
        read,
        write_without_response,
        value = 1
    )]
    protocol_mode: u8,
    /// Input Report（0x2A4D）: キーボードの入力レポート本体（8バイト）。
    /// Report Reference記述子（0x2908）で「レポートID=0、種類=Input(1)」を宣言。
    /// ホスト（PCやスマートフォン）はこの記述子を見て、
    /// どのReport特性がどのレポートに対応するかを判別する
    #[descriptor(uuid = descriptors::REPORT_REFERENCE, read, value = [0x00, 0x01])]
    #[characteristic(uuid = characteristic::REPORT, read, notify)]
    input_report: [u8; 8],
    /// Report Map（0x2A4B）: 上で定義したHIDレポートディスクリプタ。読み取り専用
    #[characteristic(uuid = characteristic::REPORT_MAP, read, value = REPORT_MAP)]
    report_map: [u8; 45],
    /// HID Information（0x2A4A）: 4バイト固定。
    /// [HID仕様バージョン1.11(リトルエンディアン), 国コード0,
    ///  フラグ(bit1=NormallyConnectable)]
    #[characteristic(
        uuid = characteristic::HID_INFORMATION,
        read,
        value = [0x11, 0x01, 0x00, 0x02]
    )]
    hid_information: [u8; 4],
    /// HID Control Point（0x2A4C）: ホストがSuspend(0)/Exit Suspend(1)を
    /// 応答なし書き込みで通知してくる。この例では受け取ってログに出すだけ
    #[characteristic(uuid = characteristic::HID_CONTROL_POINT, write_without_response)]
    hid_control_point: u8,
}

/// バッテリーサービス（0x180F）。HOGPではHIDデバイスに必須とされている
#[gatt_service(uuid = service::BATTERY)]
struct BatteryService {
    /// バッテリー残量（0x2A19）。この例では実測せず100%固定のデモ値
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, value = 100)]
    level: u8,
}

/// デバイス情報サービス（0x180A）。HOGPではPnP ID特性が必須
#[gatt_service(uuid = service::DEVICE_INFORMATION)]
struct DeviceInfoService {
    /// PnP ID（0x2A50）: 7バイト固定。
    /// [ベンダーID種別(0x02=USB実装者フォーラム), ベンダーID(0x303A=Espressif,
    ///  リトルエンディアン), 製品ID(0x0001), 製品バージョン(0x0100)]
    #[characteristic(
        uuid = characteristic::PNP_ID,
        read,
        value = [0x02, 0x3A, 0x30, 0x01, 0x00, 0x00, 0x01]
    )]
    pnp_id: [u8; 7],
    /// 製造者名（0x2A29）。読み取り専用の文字列
    #[characteristic(
        uuid = characteristic::MANUFACTURER_NAME_STRING,
        read,
        value = "embassy-esp32-c6 textbook"
    )]
    manufacturer: &'static str,
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

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

    ble_hid_run(controller, button).await;
}

/// BLE（Bluetooth Low Energy）スタック全体を動かす
async fn ble_hid_run<C>(controller: C, mut button: Input<'_>)
where
    C: Controller,
{
    // デバイスアドレス。テストしやすいよう固定のランダムアドレスを使う
    // （本来はチップのMACアドレスなどから作る）
    let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    info!("デバイスアドレス: {:?}", address);

    // ホストスタックが使うメモリ（接続・チャネル管理領域）を確保
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    info!("HIDキーボードのGATTサーバーを起動し、アドバタイズを開始します");
    // GAPのAppearance（見た目の分類）を「キーボード（0x03C1）」にすると、
    // 接続したOSがキーボードのアイコンで表示してくれる
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "C6-KEYBOARD",
        appearance: &appearance::human_interface_device::KEYBOARD,
    }))
    .unwrap();

    // ble_taskはスタックの心臓部で、常に動かし続ける必要がある。
    // joinでアドバタイズ・接続処理と並行実行する
    let _ = join(ble_task(runner), async {
        loop {
            match advertise("C6-KEYBOARD", &mut peripheral, &server).await {
                Ok(conn) => {
                    // 接続が確立したら、GATTイベント処理とキー送信タスクを並行実行。
                    // どちらかが終わったら（=切断されたら）アドバタイズに戻る
                    let a = gatt_events_task(&server, &conn);
                    let b = hid_key_task(&server, &conn, &mut button);
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
    let report_map = server.hid_service.report_map;
    let control_point = server.hid_service.hid_control_point;
    let protocol_mode = server.hid_service.protocol_mode;
    let battery_level = server.battery_service.level;
    let pnp_id = server.device_info_service.pnp_id;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        // ホストは接続直後にReport Mapなどを読み取り、
                        // 「どんなHIDデバイスか」を学習する
                        if event.handle() == report_map.handle {
                            info!("[gatt] Report Mapが読み取られました");
                        } else if event.handle() == battery_level.handle {
                            info!("[gatt] バッテリー残量が読み取られました");
                        } else if event.handle() == pnp_id.handle {
                            info!("[gatt] PnP IDが読み取られました");
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == control_point.handle {
                            info!("[gatt] HID Control Point書き込み: {:?}", event.data());
                        } else if event.handle() == protocol_mode.handle {
                            info!("[gatt] Protocol Mode書き込み: {:?}", event.data());
                        } else {
                            info!("[gatt] 書き込み要求: {:?}", event.data());
                        }
                    }
                    _ => {}
                };
                // 応答を返す。dropでも送られるが、確実に送るため明示的に呼ぶ
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("[gatt] 応答送信エラー: {:?}", e),
                };
            }
            _ => {} // その他の接続イベントは無視
        }
    };
    info!("[gatt] 切断されました: {:?}", reason);
    Ok(())
}

/// アドバタイズ（接続可能な状態で存在を知らせる）を行い、
/// セントラル（PCやスマートフォン）からの接続を待つ
async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    // アドバタイズパケット（最大31バイト）にフラグ・サービスUUID・
    // Appearance・名前を詰める
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            // HIDサービス(0x1812)を持っていることを知らせる（リトルエンディアン）
            AdStructure::ServiceUuids16(&[[0x12, 0x18]]),
            // Appearance（AD種別0x19）= キーボード(0x03C1、リトルエンディアン)。
            // trouble-host 0.6のAdStructureに専用の列挙子がないためUnknownで
            // 生バイトを書く。OSのスキャン画面でキーボードとして扱われる材料になる
            AdStructure::Unknown {
                ty: 0x19,
                data: &[0xC1, 0x03],
            },
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

/// BOOTボタンが押されるたびに「a」キーの押下→解放レポートを通知するタスク。
/// 通知に失敗したら（=切断されたら）終了する
async fn hid_key_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    button: &mut Input<'_>,
) {
    let input_report = server.hid_service.input_report;
    loop {
        // ボタンが押される（プルアップなのでLowに落ちる）のを待つ
        button.wait_for_falling_edge().await;
        // チャタリング（接点の細かい振動）が落ち着くまで少し待つ
        Timer::after_millis(20).await;
        if button.is_high() {
            continue; // ノイズだったので無視
        }

        // キーダウン: 「a」(Usage ID 0x04) を押したレポートを通知
        info!("[hid] キーダウン: 'a'");
        if input_report.notify(conn, &KEY_A_DOWN).await.is_err() {
            info!("[hid] 通知に失敗しました（切断）");
            break;
        }
        // キーアップ: 全キー解放のレポートを送らないと押しっぱなし扱いになる
        Timer::after_millis(50).await;
        info!("[hid] キーアップ");
        if input_report.notify(conn, &KEY_ALL_UP).await.is_err() {
            info!("[hid] 通知に失敗しました（切断）");
            break;
        }

        // ボタンが離される（Highに戻る）のを待ってから次の押下を受け付ける
        button.wait_for_rising_edge().await;
        Timer::after_millis(20).await;
    }
}
