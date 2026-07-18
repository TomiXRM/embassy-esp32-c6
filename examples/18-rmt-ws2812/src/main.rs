//! 18-rmt-ws2812: RMTペリフェラルでオンボードWS2812B RGB LEDを光らせる
//!
//! ついにオンボードLEDの出番です。これまでの章（01-blinkyなど）では
//! GPIO10の外付けLEDを使ってきましたが、ESP32-C6-DevKitC-1に最初から
//! 載っているLEDはWS2812B（アドレサブルRGB LED）で、**GPIO8**に接続
//! されています。単純なON/OFFでは光らず、1本の信号線にµs以下の精密な
//! パルス列を送る必要があるため、RMT（Remote Control Transceiver）
//! ペリフェラルで波形をハードウェア生成します。
//!
//! 配線: 不要（オンボードLED、GPIO8に直結済み）
//!
//! 注意: GPIO8はストラッピングピンでもあります。リセット時にLowだと
//! 書き込みモード関連の判定に影響するため、外部回路でGPIO8をLowに
//! 引っ張っているとブートに失敗することがあります（起動後にRMTの出力
//! として使う分には問題ありません）。
//!
//! WS2812Bのプロトコル:
//! - 1ピクセル = 24bit を「緑→赤→青」（GRB順・各バイトMSBファースト）で送る
//! - 0ビット: High 0.4µs → Low 0.85µs
//! - 1ビット: High 0.8µs → Low 0.45µs（許容誤差 ±0.15µs）
//! - 50µs以上Lowを保つと「リセット」となり、送った色がLEDに反映される
//!
//! esp-hal-smartledクレートはesp-hal ~1.0固定でesp-hal 1.1.1とは併用
//! できないため、この例ではPulseCodeを自前で組み立てます。

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Level;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::rmt::{PulseCode, Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::info;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

// ---- RMTのティック計算 ----
// RMTのソースクロックは80MHz。チャネルの分周器(clk_divider)を8にすると
//   80MHz ÷ 8 = 10MHz → 1ティック = 0.1µs (100ns)
// になり、WS2812のタイミングをキリのよいティック数で表せる。
const T0H: u16 = 4; // 0ビットのHigh: 4ティック = 0.4µs
const T0L: u16 = 8; // 0ビットのLow : 8ティック = 0.8µs（規格値0.85µs、誤差±0.15µs内）
const T1H: u16 = 8; // 1ビットのHigh: 8ティック = 0.8µs
const T1L: u16 = 4; // 1ビットのLow : 4ティック = 0.4µs（規格値0.45µs、誤差±0.15µs内）

/// 色相(0〜255)を虹色のRGBに変換する（HSVの簡易版、いわゆるカラーホイール）
/// 0→赤、85→緑、170→青、255→赤に戻る
fn wheel(pos: u8) -> (u8, u8, u8) {
    if pos < 85 {
        // 赤→緑
        (255 - pos * 3, pos * 3, 0)
    } else if pos < 170 {
        // 緑→青
        let p = pos - 85;
        (0, 255 - p * 3, p * 3)
    } else {
        // 青→赤
        let p = pos - 170;
        (p * 3, 0, 255 - p * 3)
    }
}

/// RGB値をWS2812用のパルス列に変換する。
/// 24bit（GRB順・MSBファースト）+ 終端マーカで25要素。
fn ws2812_pulses(r: u8, g: u8, b: u8) -> [PulseCode; 25] {
    let one = PulseCode::new(Level::High, T1H, Level::Low, T1L);
    let zero = PulseCode::new(Level::High, T0H, Level::Low, T0L);

    let mut data = [PulseCode::end_marker(); 25];
    // WS2812は緑→赤→青（GRB）の順で受け取る点に注意
    let grb: u32 = ((g as u32) << 16) | ((r as u32) << 8) | (b as u32);
    for (i, slot) in data.iter_mut().take(24).enumerate() {
        // 上位ビット(bit23)から順に送る
        let bit_is_one = grb & (1 << (23 - i)) != 0;
        *slot = if bit_is_one { one } else { zero };
    }
    // data[24]は終端マーカ（長さ0のエントリ）。ここで送信が止まる。
    data
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // RMTを80MHzで初期化し、async版に変換する
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80))
        .unwrap()
        .into_async();

    // チャネル0を送信用に設定してGPIO8（オンボードWS2812B）へ接続。
    // アイドル時はLow固定にしておく（フレーム間の>50µsのLowがリセットになる）
    let mut channel = rmt
        .channel0
        .configure_tx(
            &TxChannelConfig::default()
                .with_clk_divider(8) // 80MHz÷8=10MHz → 1ティック=0.1µs
                .with_idle_output(true)
                .with_idle_output_level(Level::Low),
        )
        .unwrap()
        .with_pin(peripherals.GPIO8);

    info!("RMTでオンボードWS2812Bを虹色に光らせます");

    let mut hue: u8 = 0;
    loop {
        let (r, g, b) = wheel(hue);
        // そのままだと眩しすぎるので1/8の明るさに落とす
        let data = ws2812_pulses(r / 8, g / 8, b / 8);

        // 24bit分のパルス列をRMTがハードウェアで送出する。
        // await中CPUは他の処理に使える（送信完了割り込みで再開）
        channel.transmit(&data).await.unwrap();

        // 約20msごとに色相を進める（フレーム間隔はリセット時間50µsより
        // 十分長いので、毎回確実に色が反映される）。1周は256×20ms≒5秒
        hue = hue.wrapping_add(1);
        Timer::after(Duration::from_millis(20)).await;
    }
}
