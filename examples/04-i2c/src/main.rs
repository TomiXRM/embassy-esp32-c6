//! 04-i2c: I2CバススキャンとSHT30温湿度センサ
//!
//! I2Cマスタ（SDA=GPIO6、SCL=GPIO7、100kHz）を非同期モードで使います。
//! 起動時にバス上のデバイスをスキャンしてアドレスを表示し、
//! SHT30温湿度センサ（アドレス0x44）が見つかったら
//! 2秒ごとに単発測定して温度と湿度を表示します。
//!
//! 配線: SHT30モジュールの SDA→GPIO6、SCL→GPIO7、VCC→3.3V、GND→GND
//! （多くのモジュールはプルアップ抵抗を内蔵。無い場合はSDA/SCLを
//! 　それぞれ10kΩで3.3Vへプルアップする）

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, Error as I2cError, I2c};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::{error, info, warn};

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// SHT30のI2Cアドレス（ADDRピンがGNDのとき0x44、VDDのとき0x45）
const SHT30_ADDR: u8 = 0x44;

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // I2C0を100kHz（標準モード）で初期化し、SDA=GPIO6 / SCL=GPIO7 を割り当てて
    // into_async()で非同期モードに切り替える
    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let mut i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("I2Cの設定が不正です")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    // --- バススキャン ---
    // 7bitアドレスの有効範囲 0x08〜0x77 を1つずつ試し、
    // ACK（応答）が返ってきたアドレスにデバイスがいると判断する。
    // 空の書き込みはドライバがエラーにするため、1バイト（0x00）を書き込む。
    // （多くのデバイスは0x00を「レジスタ番号の指定」と解釈するだけで無害だが、
    // 　書き込みに反応するデバイスがまれにあるので、実運用のスキャンでは注意）
    info!("I2Cバスをスキャンします (0x08..0x77)");
    let mut sht30_found = false;
    for addr in 0x08u8..0x78 {
        match i2c.write_async(addr, &[0x00]).await {
            Ok(()) => {
                info!("  デバイスを発見: 0x{:02X}", addr);
                if addr == SHT30_ADDR {
                    sht30_found = true;
                }
            }
            // ACKが返らない = そのアドレスにデバイスはいない（正常なこと）
            Err(I2cError::AcknowledgeCheckFailed(_)) => {}
            // それ以外はバス自体の異常（配線ミス・プルアップ不足など）
            Err(e) => warn!("  0x{:02X} でバスエラー: {:?}", addr, e),
        }
    }

    if !sht30_found {
        warn!(
            "SHT30 (0x{:02X}) が見つかりません。配線を確認してください",
            SHT30_ADDR
        );
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }

    info!("SHT30を検出しました。2秒ごとに温湿度を測定します");

    loop {
        Timer::after(Duration::from_secs(2)).await;

        // 単発測定コマンド 0x2C06（クロックストレッチ有効・高再現性）を送る。
        // I2Cではコマンドを上位バイト・下位バイトの順で送る
        if let Err(e) = i2c.write_async(SHT30_ADDR, &[0x2C, 0x06]).await {
            error!("測定コマンドの送信に失敗: {:?}", e);
            continue;
        }

        // 測定完了を待つ（高再現性測定の最大所要時間は15ms。余裕を見て20ms）
        Timer::after(Duration::from_millis(20)).await;

        // 測定結果6バイトを読み出す:
        // [温度上位, 温度下位, 温度CRC, 湿度上位, 湿度下位, 湿度CRC]
        let mut data = [0u8; 6];
        match i2c.read_async(SHT30_ADDR, &mut data).await {
            Ok(()) => {
                // data[2]とdata[5]はCRC-8チェックサム（多項式0x31、初期値0xFF）。
                // 通信の誤り検出に使えるが、この例では検証を省略する
                let raw_temp = u16::from_be_bytes([data[0], data[1]]);
                let raw_humi = u16::from_be_bytes([data[3], data[4]]);

                // データシートの変換式:
                //   温度[℃] = -45 + 175 × 生値 / 65535
                //   湿度[%RH] = 100 × 生値 / 65535
                let temp = -45.0 + 175.0 * (raw_temp as f32) / 65535.0;
                let humi = 100.0 * (raw_humi as f32) / 65535.0;

                info!("温度: {:.1} C / 湿度: {:.1} %RH", temp, humi);
            }
            Err(e) => error!("測定値の読み出しに失敗: {:?}", e),
        }
    }
}
