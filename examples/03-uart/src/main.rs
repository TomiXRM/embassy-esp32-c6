//! 03-uart: UARTループバック通信
//!
//! UART1（TX=GPIO23、RX=GPIO22、115200bps）を非同期モードで使い、
//! 1秒ごとにメッセージを送信して、それを自分自身で受信します。
//! 送受信には embedded-io-async の Write / Read トレイトを使います。
//! （どのマイコンでも同じ書き方ができる、移植性の高いインタフェースです）
//!
//! 配線: GPIO23（TX）と GPIO22（RX）をジャンパ線で直結（ループバック）

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, with_timeout};
use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{Config as UartConfig, Uart};

use defmt::{error, info, warn};
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

// 送信するメッセージ（ループバックでそのまま戻ってくるはず）
const MESSAGE: &str = "Hello, UART! from ESP32-C6\r\n";

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // UART1を115200bpsで初期化し、TX=GPIO23 / RX=GPIO22 を割り当てて
    // into_async()で非同期モードに切り替える
    // （GPIO16/17はUART0=ログ出力用コンソールなので使わない）
    let uart_config = UartConfig::default().with_baudrate(115_200);
    let mut uart = Uart::new(peripherals.UART1, uart_config)
        .expect("UARTの設定が不正です")
        .with_tx(peripherals.GPIO23)
        .with_rx(peripherals.GPIO22)
        .into_async();

    info!("UARTループバックを開始します（GPIO23とGPIO22を直結してください）");

    // 受信バッファ。送信メッセージと同じ長さだけ受信する
    let mut buf = [0u8; MESSAGE.len()];

    loop {
        // 送信: write_all は embedded-io-async の Write トレイトのメソッド。
        // （writeは一部しか書き込まないことがあるため、全バイト書き込むwrite_allを使う）
        if let Err(e) = uart.write_all(MESSAGE.as_bytes()).await {
            error!("送信エラー: {:?}", e);
        }

        // 受信: read_exact はバッファがいっぱいになるまで待つ。
        // 配線を忘れると永遠に待ってしまうので、with_timeoutで500msの上限を付ける。
        // エラーはunwrapせず、matchで場合分けして扱う
        match with_timeout(Duration::from_millis(500), uart.read_exact(&mut buf)).await {
            // 受信成功 → UTF-8文字列に変換して表示
            Ok(Ok(())) => match core::str::from_utf8(&buf) {
                Ok(s) => info!("受信: {}", s.trim_end()),
                Err(_) => warn!("受信したがUTF-8として不正: {=[u8]:X}", &buf[..]),
            },
            // UARTの受信エラー（フレーミングエラーなど）
            Ok(Err(e)) => error!("受信エラー: {:?}", e),
            // タイムアウト（データが届かない）
            Err(_) => warn!("受信タイムアウト: GPIO23とGPIO22の配線を確認してください"),
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
