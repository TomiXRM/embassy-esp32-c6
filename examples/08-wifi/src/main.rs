//! 08-wifi: Wi-Fi接続とHTTP GET
//!
//! Wi-Fiアクセスポイントにステーション（子機）として接続し、
//! DHCPでIPアドレスを取得したあと、DNSで example.com のIPアドレスを引き、
//! TCPソケットでHTTP GETリクエストを送って応答の先頭500バイトを表示します。
//! 以降は30秒ごとに同じリクエストを繰り返します。
//!
//! 注意: ESP32-C6のWi-Fiは**2.4GHz帯のみ**対応です。
//! 5GHz専用のアクセスポイントには接続できません。
//!
//! ビルド前に環境変数でSSIDとパスワードを渡してください:
//!   SSID=あなたのSSID PASSWORD=あなたのパスワード cargo run --release -p wifi
//! 未設定でもビルドは通りますが、プレースホルダのままなので接続には失敗します。

#![no_std]
#![no_main]

use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_net::dns::DnsQueryType;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
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

// embassy-netのスタックが内部で使うリソース（ソケット数ぶんの領域）
static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // probe-rsモードではRTTを初期化し、defmtのグローバルロガーを起動する
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Wi-Fiドライバはヒープを使うため、esp-allocでヒープを確保する
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

    // TCPシーケンス番号の予測を防ぐため、乱数でシードを作る
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

    // TCPソケット用の送受信バッファ
    let mut rx_buffer = [0u8; 4096];
    let mut tx_buffer = [0u8; 1024];

    loop {
        // DNSで example.com のIPv4アドレスを解決する
        let address = match stack.dns_query("example.com", DnsQueryType::A).await {
            Ok(addresses) => match addresses.first() {
                Some(addr) => *addr,
                None => {
                    error!("DNS応答にアドレスが含まれていません");
                    Timer::after(Duration::from_secs(30)).await;
                    continue;
                }
            },
            Err(e) => {
                error!("DNS解決に失敗しました: {:?}", defmt::Debug2Format(&e));
                Timer::after(Duration::from_secs(30)).await;
                continue;
            }
        };
        info!(
            "example.com のIPアドレス: {}",
            defmt::Display2Format(&address)
        );

        // TCPソケットを作ってポート80（HTTP）へ接続
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        info!("{}:80 へ接続します", defmt::Display2Format(&address));
        match socket.connect((address, 80)).await {
            Ok(()) => info!("接続しました"),
            Err(e) => {
                error!("接続に失敗しました: {:?}", defmt::Debug2Format(&e));
                Timer::after(Duration::from_secs(30)).await;
                continue;
            }
        }

        // 最小限のHTTP/1.1 GETリクエストを送る
        let request = b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
        match socket.write_all(request).await {
            Ok(()) => info!("HTTPリクエストを送信しました"),
            Err(e) => {
                error!("送信に失敗しました: {:?}", defmt::Debug2Format(&e));
                Timer::after(Duration::from_secs(30)).await;
                continue;
            }
        }

        // 応答を先頭500バイトまで読み取る
        let mut response = [0u8; 500];
        let mut total = 0;
        while total < response.len() {
            match socket.read(&mut response[total..]).await {
                Ok(0) => break, // サーバが接続を閉じた
                Ok(n) => total += n,
                Err(e) => {
                    warn!(
                        "受信中にエラーが発生しました: {:?}",
                        defmt::Debug2Format(&e)
                    );
                    break;
                }
            }
        }

        info!("---- 応答の先頭{}バイト ----", total);
        match core::str::from_utf8(&response[..total]) {
            Ok(text) => info!("{}", text),
            Err(_) => info!("(UTF-8として表示できないデータでした)"),
        }
        info!("---- ここまで ----");

        socket.close();

        info!("30秒後にもう一度リクエストします");
        Timer::after(Duration::from_secs(30)).await;
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
