//! 19-pcnt: パルスカウンタ（PCNT）でエッジをハードウェアカウントする
//!
//! PCNTはCPUを介さずに入力信号のエッジ（立ち上がり/立ち下がり）を数える
//! ペリフェラルです。この例ではGPIO18（内部プルアップ入力）の立ち下がり
//! エッジをカウントし、毎秒カウンタ値をログに出します。
//!
//! 配線（どちらでも試せます）:
//! - おすすめ: ジャンパワイヤでGPIO10とGPIO18を接続する。
//!   この例はGPIO10を100msごとにトグルする「パルス発生タスク」を内蔵して
//!   いるので、立ち下がりエッジは毎秒ちょうど5回 → カウンタが毎秒+5ずつ
//!   増えるのが観察できます。
//! - あるいは: GPIO18とGNDの間をジャンパワイヤやボタンで手動で
//!   つなぎ外しする。接点は盛大にバウンス（チャタリング）しますが、
//!   PCNTのハードウェアグリッチフィルタが短いノイズを除去します。
//!   （わざとバウンスさせてフィルタの効果を見るデモでもあります）
//!
//! 実世界での代表的な用途はロータリーエンコーダの4逓倍デコードです
//! （後半のコメント参照）。

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::pcnt::Pcnt;
use esp_hal::pcnt::channel::{CtrlMode, EdgeMode};
use esp_hal::timer::timg::TimerGroup;
use log::info;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// パルス発生タスク: GPIO10を100msごとにトグルする。
/// High→Lowの立ち下がりは200ms周期で起きるので、GPIO10とGPIO18を
/// ジャンパでつなぐとカウンタは毎秒5ずつ増える。
#[embassy_executor::task]
async fn pulse_gen(mut pin: Output<'static>) {
    loop {
        pin.toggle();
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // パルス発生用の出力（GPIO10、最初はLow）
    let pulse_out = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
    spawner.spawn(pulse_gen(pulse_out).unwrap());

    // カウント対象の入力（GPIO18、内部プルアップ）。
    // 未接続時はHighに保たれ、GNDに触れるとLow=立ち下がりが発生する
    let input_config = InputConfig::default().with_pull(Pull::Up);
    let pulse_in = Input::new(peripherals.GPIO18, input_config);

    // PCNTを初期化してユニット0を使う（ESP32-C6はユニット4個×各2チャネル）
    let pcnt = Pcnt::new(peripherals.PCNT);
    let unit = pcnt.unit0;

    // ハードウェアグリッチフィルタ:
    // 指定したAPB_CLKサイクル数より短いパルスをノイズとして無視する。
    // ESP32-C6のAPB_CLKは40MHzなので、
    //   1000サイクル ÷ 40MHz = 25µs
    // より短いパルス（接点バウンスの大半）が除去される。
    // 設定値は10bitレジスタのため最大1023（≒25.6µs）まで。
    unit.set_filter(Some(1000)).unwrap();
    unit.clear(); // カウンタを0にリセット

    // チャネル0の設定
    let channel = &unit.channel0;
    // 制御信号は使わないので定数High（Level::HighはPeripheralInputを実装
    // していて「常にHighの仮想入力」として配線できる）に固定し、
    // High/Lowどちらでもカウント方向を変えない(Keep)ようにする
    channel.set_ctrl_signal(Level::High);
    channel.set_ctrl_mode(CtrlMode::Keep, CtrlMode::Keep);
    // エッジ信号にGPIO18を接続。
    // peripheral_input()はGPIOマトリクス(interconnect)経由でピンの信号を
    // ペリフェラルに配線するunstable APIで、esp-halの更新で変わる可能性がある
    channel.set_edge_signal(pulse_in.peripheral_input());
    // 立ち下がり(neg_edge)でインクリメント、立ち上がり(pos_edge)は何もしない(Hold)
    channel.set_input_mode(EdgeMode::Increment, EdgeMode::Hold);

    // ---- 実世界での使い方: ロータリーエンコーダの4逓倍デコード ----
    // A相をエッジ信号・B相を制御信号にすると、ハードウェアだけで回転方向
    // 込みのカウントができる（esp-hal pcntモジュールのdocの例より）:
    //   let ch0 = &unit.channel0;
    //   ch0.set_ctrl_signal(input_b.clone());  // B相で方向を判定
    //   ch0.set_edge_signal(input_a.clone());  // A相のエッジを数える
    //   ch0.set_ctrl_mode(CtrlMode::Reverse, CtrlMode::Keep);
    //   ch0.set_input_mode(EdgeMode::Increment, EdgeMode::Decrement);
    //   let ch1 = &unit.channel1;
    //   ch1.set_ctrl_signal(input_a);          // ch1はA/Bを入れ替える
    //   ch1.set_edge_signal(input_b);
    //   ch1.set_ctrl_mode(CtrlMode::Reverse, CtrlMode::Keep);
    //   ch1.set_input_mode(EdgeMode::Decrement, EdgeMode::Increment);
    // 2チャネルで全エッジを数えるため分解能が4倍になる（4逓倍）。

    // カウント開始（PCNTにasync APIはないため、この例はポーリングで読む）
    unit.resume();

    info!("PCNTを開始しました。GPIO10とGPIO18をジャンパでつなぐと毎秒+5ずつ増えます");

    loop {
        Timer::after(Duration::from_secs(1)).await;
        // value()は現在のカウンタ値（i16）を読むだけ。カウント自体は
        // ハードウェアが行うので、この間CPUは何もしなくてよい
        info!("カウンタ値: {}", unit.value());
    }
}
