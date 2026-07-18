//! 17-https: Wi-Fi経由のHTTPS (TLS) GET
//!
//! Wi-Fiアクセスポイントにステーション（子機）として接続し、
//! DHCPでIPアドレスを取得したあと、HTTPSで https://www.example.com/ へ
//! GETリクエストを送り、ステータスコードと本文の先頭500バイトを表示します。
//! 以降は60秒ごとに同じリクエストを繰り返します。
//!
//! HTTPクライアントには reqwless、TLSには embedded-tls を使います。
//! 08-wifi（平文HTTP）との違いは、通信路がTLSで暗号化される点です。
//!
//! 重要な注意点（教材として正直に書きます）:
//! - embedded-tls は **TLS 1.3のみ** 対応です。TLS 1.2までしか
//!   話せないサーバには接続できません。
//! - この例は `TlsVerify::None` を使っています。**通信は暗号化されますが、
//!   サーバ証明書を検証していない**ため、接続相手が本物のサーバである
//!   保証はありません（中間者攻撃を検出できない）。実運用ではルートCA証明書を
//!   組み込んで `TlsVerify::Certificate` で検証すべきです。
//! - TLSには大きな受信/送信レコードバッファ（TLSの最大レコード長16KiB+α、
//!   ここでは16640バイト×2）が必要です。スタックに置くと溢れるので
//!   staticに確保しています。またWi-FiドライバとTLS処理のために
//!   esp-allocのヒープも必要です。
//! - TLSの乱数はesp-halのハードウェア乱数生成器（Rng）から作った
//!   64ビットシードで初期化します。
//!
//! 参考にした実装:
//! - esp32c3-embassy (Claudio Mattera, MIT OR Apache-2.0)
//!   <https://github.com/claudiomattera/esp32c3-embassy>
//! - esp-hal公式example examples/wifi/embassy_dhcp（tag: esp-radio-v0.18.0）
//!   <https://github.com/esp-rs/esp-hal>
//!
//! 注意: ESP32-C6のWi-Fiは**2.4GHz帯のみ**対応です。
//! 5GHz専用のアクセスポイントには接続できません。
//!
//! ビルド前に環境変数でSSIDとパスワードを渡してください:
//!   SSID=あなたのSSID PASSWORD=あなたのパスワード cargo run --release -p https
//! 未設定でもビルドは通りますが、プレースホルダのままなので接続には失敗します。

#![no_std]
#![no_main]

use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use embedded_io_async::Read;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ram;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{Config as WifiConfig, ControllerConfig, Interface, WifiController};
// defmtログの出口を選ぶ: probe-rsではrtt-target、espflashではesp-printlnをリンクする
#[cfg(feature = "espflash")]
use esp_println as _;
use reqwless::client::{HttpClient, TlsConfig, TlsVerify};
use reqwless::request::Method;
#[cfg(feature = "probe-rs")]
use rtt_target as _;
use static_cell::StaticCell;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

// 環境変数SSID/PASSWORDをコンパイル時に埋め込む。
// 未設定の場合はプレースホルダになる（ビルドは通るが接続はできない）。
const SSID: &str = match option_env!("SSID") {
    Some(v) => v,
    None => "your-ssid",
};
const PASSWORD: &str = match option_env!("PASSWORD") {
    Some(v) => v,
    None => "your-password",
};

/// GET先のURL。httpsスキームを指定するとreqwlessがTLSで接続する
const URL: &str = "https://www.example.com/";

// embassy-netのスタックが内部で使うリソース（ソケット数ぶんの領域）
static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

// reqwlessのTcpClientが使うTCP接続の状態（接続1本、送受信バッファ各4KiB）
static TCP_CLIENT_STATE: StaticCell<TcpClientState<1, 4096, 4096>> = StaticCell::new();

// TLSのレコードバッファ。TLSレコードは最大16KiB+ヘッダなので16640バイト確保する。
// 合計約32KiBと大きいため、スタックではなくstatic領域に置く
static TLS_READ_BUFFER: StaticCell<[u8; 16640]> = StaticCell::new();
static TLS_WRITE_BUFFER: StaticCell<[u8; 16640]> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // probe-rsモードではRTTを初期化し、defmtのグローバルロガーを起動する
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Wi-FiドライバとTLS処理はヒープを使うため、esp-allocでヒープを確保する
    // （公式exampleと同じ構成: 回収済みRAM 64KiB + 通常RAM 36KiB）
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // ステーション（子機）モードの設定。SSIDとパスワードを渡す
    let station_config = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(SSID)
            .with_password(PASSWORD.into()),
    );

    info!("Wi-Fiを初期化します");
    let (controller, interfaces) = match esp_radio::wifi::new(
        peripherals.WIFI,
        ControllerConfig::default().with_initial_config(station_config),
    ) {
        Ok(v) => v,
        Err(e) => panic!("Wi-Fiの初期化に失敗しました: {e:?}"),
    };

    // ステーション用のネットワークインタフェースを取り出す
    let wifi_interface = interfaces.station;

    // DHCPでIPアドレスをもらう設定
    let net_config = embassy_net::Config::dhcpv4(Default::default());

    // ハードウェア乱数生成器。TCPシーケンス番号とTLSのシードの両方に使う
    let rng = Rng::new();
    let seed = ((rng.random() as u64) << 32) | rng.random() as u64;

    // ネットワークスタックを生成。stackは操作用ハンドル、runnerは駆動役
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        STACK_RESOURCES.init(StackResources::new()),
        seed,
    );

    // Wi-Fi接続を維持するタスクと、ネットワークスタックを回すタスクを起動
    spawner.spawn(connection_task(controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());

    // DHCPでIPアドレスが取れるまで待つ
    info!("IPアドレスの取得を待っています...");
    stack.wait_config_up().await;
    if let Some(config) = stack.config_v4() {
        // embassy-net(smoltcp)のIP型はdefmt::Formatを実装しないためDisplayを橋渡しする
        info!(
            "IPアドレスを取得しました: {}",
            defmt::Display2Format(&config.address)
        );
    }

    // ---- ここからHTTPSクライアントの準備 ----

    // TCP接続を作る部品と、URLのホスト名を解決するDNSの部品
    let tcp_client = TcpClient::new(stack, TCP_CLIENT_STATE.init(TcpClientState::new()));
    let dns_socket = DnsSocket::new(stack);

    // TLSの内部乱数（鍵交換などに使用）のシードをハードウェア乱数から作る。
    // reqwlessはこの64ビット値でChaCha8乱数生成器を初期化する
    let tls_seed = ((rng.random() as u64) << 32) | rng.random() as u64;

    // TLS設定。TlsVerify::None = サーバ証明書を検証しない（上の注意を参照）
    let tls_config = TlsConfig::new(
        tls_seed,
        TLS_READ_BUFFER.init([0; 16640]),
        TLS_WRITE_BUFFER.init([0; 16640]),
        TlsVerify::None,
    );

    // TLS対応のHTTPクライアント。URLがhttpsなら自動でTLSハンドシェイクを行う
    let mut client = HttpClient::new_with_tls(&tcp_client, &dns_socket, tls_config);

    loop {
        info!("{} へHTTPS GETリクエストを送ります", URL);

        // ヘッダ受信用のバッファ（本文の一部もここに入る）
        let mut rx_buffer = [0u8; 4096];

        // リクエストを組み立てる。この時点でDNS解決→TCP接続→TLSハンドシェイクまで行われる
        let mut request = match client.request(Method::GET, URL).await {
            Ok(request) => request,
            Err(e) => {
                // reqwless::Errorはdefmt::Formatを実装しないためDebugを橋渡しする
                error!(
                    "接続またはTLSハンドシェイクに失敗しました: {:?}",
                    defmt::Debug2Format(&e)
                );
                Timer::after(Duration::from_secs(60)).await;
                continue;
            }
        };

        // リクエストを送信して応答ヘッダを受信する
        let response = match request.send(&mut rx_buffer).await {
            Ok(response) => response,
            Err(e) => {
                error!(
                    "リクエストの送受信に失敗しました: {:?}",
                    defmt::Debug2Format(&e)
                );
                Timer::after(Duration::from_secs(60)).await;
                continue;
            }
        };

        // reqwlessのStatusCodeもdefmt::Formatを実装しないためDebugを橋渡しする
        info!(
            "HTTPステータス: {:?}",
            defmt::Debug2Format(&response.status)
        );

        // 本文を先頭500バイトまで読み取る
        let mut reader = response.body().reader();
        let mut body = [0u8; 500];
        let mut total = 0;
        while total < body.len() {
            match reader.read(&mut body[total..]).await {
                Ok(0) => break, // 本文の終わり
                Ok(n) => total += n,
                Err(e) => {
                    warn!(
                        "本文の受信中にエラーが発生しました: {:?}",
                        defmt::Debug2Format(&e)
                    );
                    break;
                }
            }
        }

        info!("---- 本文の先頭{}バイト ----", total);
        match core::str::from_utf8(&body[..total]) {
            Ok(text) => info!("{}", text),
            Err(_) => info!("(UTF-8として表示できないデータでした)"),
        }
        info!("---- ここまで ----");

        info!("60秒後にもう一度リクエストします");
        Timer::after(Duration::from_secs(60)).await;
    }
}

/// Wi-Fi接続を維持するタスク。切断されたら5秒待って再接続する
#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    info!("Wi-Fi接続管理タスクを開始します");
    loop {
        info!("SSID「{}」へ接続します...", SSID);
        match controller.connect_async().await {
            Ok(connected) => {
                info!("Wi-Fiに接続しました: {:?}", connected);
                // 切断されるまでここで待つ
                let disconnected = controller.wait_for_disconnect_async().await.ok();
                warn!("Wi-Fiが切断されました: {:?}", disconnected);
            }
            Err(e) => {
                error!("Wi-Fi接続に失敗しました: {:?}", e);
            }
        }
        // 少し待ってから再接続する
        Timer::after(Duration::from_secs(5)).await;
    }
}

/// ネットワークスタック本体を動かし続けるタスク
#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}
