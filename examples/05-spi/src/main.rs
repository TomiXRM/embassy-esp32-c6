//! 05-spi: SPIループバック通信
//!
//! SPI2マスタ（SCK=GPIO19、MOSI=GPIO18、MISO=GPIO20、1MHz、モード0）を使い、
//! CS（チップセレクト、GPIO21）は通常のGPIO出力として手動で制御します。
//! MOSIとMISOを直結したループバックで、送ったデータが
//! そのまま返ってくることを1秒ごとに確認します。
//!
//! 配線: GPIO18（MOSI）と GPIO20（MISO）をジャンパ線で直結（ループバック）

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::{error, info, warn};

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

// 送信するテストデータ（ループバックでそのまま戻ってくるはず）
const TX_DATA: [u8; 8] = [0xA5, 0x5A, 0x01, 0x02, 0x03, 0x04, 0x05, 0xFF];

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // SPI2を初期化。周波数とモードは明示的に指定する
    // - 周波数: 1MHz（多くのSPIデバイスが対応できる控えめな速度）
    // - モード0: CPOL=0（クロックはアイドル時Low）、CPHA=0（立ち上がりエッジで取り込み）
    let spi_config = SpiConfig::default()
        .with_frequency(Rate::from_mhz(1))
        .with_mode(Mode::_0);
    let mut spi = Spi::new(peripherals.SPI2, spi_config)
        .expect("SPIの設定が不正です")
        .with_sck(peripherals.GPIO19)
        .with_mosi(peripherals.GPIO18)
        .with_miso(peripherals.GPIO20)
        .into_async();

    // CS（チップセレクト）は自分でGPIOを操作する方式。
    // 通常時はHigh（非選択）にしておき、通信の間だけLowにする
    let mut cs = Output::new(peripherals.GPIO21, Level::High, OutputConfig::default());

    info!("SPIループバックを開始します（GPIO18とGPIO20を直結してください）");

    loop {
        // 転送バッファ。SPIは全二重通信なので、送信と同時に受信が起こり、
        // transfer_in_place_asyncはバッファの内容を受信データで上書きする
        let mut buf = TX_DATA;

        cs.set_low(); // 通信開始（スレーブを選択）
        let result = spi.transfer_in_place_async(&mut buf).await;
        cs.set_high(); // 通信終了（選択を解除）

        // 結果はunwrapせず、matchで場合分けして扱う
        match result {
            Ok(()) if buf == TX_DATA => {
                info!("OK: 送信 {:02X?} → 受信 {:02X?}", TX_DATA, buf);
            }
            Ok(()) => {
                warn!(
                    "NG: 送信 {:02X?} と受信 {:02X?} が一致しません。GPIO18とGPIO20の配線を確認してください",
                    TX_DATA, buf
                );
            }
            Err(e) => error!("SPI転送エラー: {:?}", e),
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
