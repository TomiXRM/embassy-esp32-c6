//! 14-keymatrix: 2×2キーマトリクスのスキャン
//!
//! ブレッドボード上に押しボタン4個を「行×列」の格子状に配線し、
//! 少ないピン数（行2本+列2本=4ピン）で4個のキーを読み取ります。
//! - 行(row): GPIO18, GPIO19 … 出力。普段はHigh、スキャン時に1行だけLowにする
//! - 列(col): GPIO20, GPIO21 … 入力（内部プルアップ）。Lowなら「押されている」
//! - LED:     GPIO10 … どれかのキーが押されるたびにトグル
//!
//! スキャンの流れ（10ms周期のTicker）:
//! 1. 行0(GPIO18)だけLowにして1ms待つ（信号が落ち着くのを待つ）
//! 2. 列0/列1を読む。Lowなら「行0×その列」のキーが押されている
//! 3. 行0をHighに戻し、行1(GPIO19)で同じことを繰り返す
//! さらにチャタリング対策として、3回連続で同じ状態を観測したときだけ
//! 「押された/離された」と確定します（Debouncer構造体）。
//! 確定したイベントはチャネル経由でロガータスクへ送ります。
//!
//! 配線（ボタン4個。それぞれ行の線と列の線の交点に入れる）:
//! - ボタン(0,0): GPIO18 ↔ GPIO20
//! - ボタン(0,1): GPIO18 ↔ GPIO21
//! - ボタン(1,0): GPIO19 ↔ GPIO20
//! - ボタン(1,1): GPIO19 ↔ GPIO21
//! - LED: GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//!
//! ダイオードについて: キーを3個以上同時押しすると、押したキー同士が
//! 行・列の線を経由して電気的につながり、押していないキーまで押された
//! ように見える「ゴースト」が起きることがあります。各ボタンに直列で
//! ダイオード（1N4148など、行→列の向き）を入れると電流が一方通行になり
//! ゴーストを防げます。今回の2×2デモではダイオードなしでも動作します
//! （気になるのは3キー同時押しのときだけ）。

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Ticker, Timer};
use esp_backtrace as _;
// defmt の global_logger をリンクする。probe-rs では rtt-target、
// espflash では esp-println がそれぞれ defmt ログの出口になる。
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// 行・列の本数（2×2マトリクス）
const ROWS: usize = 2;
const COLS: usize = 2;

/// 「押された/離された」を確定するのに必要な連続観測回数。
/// スキャン周期10ms × 3回 = 約30msのチャタリングを吸収できます。
const DEBOUNCE_COUNT: u8 = 3;

/// キーの状態変化を表すイベント。スキャンタスク→ロガータスクへ送ります。
#[derive(Debug, Clone, Copy)]
struct KeyEvent {
    /// 行番号（0=GPIO18, 1=GPIO19）
    row: u8,
    /// 列番号（0=GPIO20, 1=GPIO21）
    col: u8,
    /// true=押された、false=離された
    pressed: bool,
}

/// タスク間をつなぐチャネル（容量8のメッセージキュー）。
/// staticに置くことで、どのタスクからも参照できます。
static CHANNEL: Channel<CriticalSectionRawMutex, KeyEvent, 8> = Channel::new();

/// 1キー分のチャタリング除去（デバウンス）を行う小さな状態機械。
///
/// 「確定済みの状態」と「連続で何回、逆の状態を観測したか」だけを持ちます。
/// 逆の状態がN回続いたら確定状態を反転し、そのとき1回だけ変化を報告します。
/// ハードウェアに依存しない純粋なロジックなので、PC上の単体テストでも
/// そのまま検証できる作りです。
#[derive(Clone, Copy)]
struct Debouncer<const N: u8> {
    /// 確定済みの状態（true=押されている）
    pressed: bool,
    /// 確定状態と逆の観測が何回連続したか
    count: u8,
}

impl<const N: u8> Debouncer<N> {
    const fn new() -> Self {
        Self {
            pressed: false,
            count: 0,
        }
    }

    /// 1回のスキャン結果（raw: 押されて見えたか）を入力する。
    /// 状態が「押された→離された」等に確定変化したときだけ
    /// `Some(新しい状態)` を返し、それ以外は `None` を返します。
    fn update(&mut self, raw: bool) -> Option<bool> {
        if raw == self.pressed {
            // 確定状態と同じ観測 → カウンタをリセットするだけ
            self.count = 0;
            None
        } else {
            // 確定状態と逆の観測 → 連続回数を数える
            self.count += 1;
            if self.count >= N {
                // N回連続したので状態変化を確定
                self.pressed = raw;
                self.count = 0;
                Some(raw)
            } else {
                None
            }
        }
    }
}

/// スキャンタスク: 10ms周期でマトリクス全体を走査し、
/// デバウンス済みのキーイベントをチャネルへ送る
#[embassy_executor::task]
async fn scan_task(mut rows: [Output<'static>; ROWS], cols: [Input<'static>; COLS]) {
    // 全キー分のデバウンサ（最初はすべて「離されている」状態）
    let mut debouncers = [[Debouncer::<DEBOUNCE_COUNT>::new(); COLS]; ROWS];

    // Tickerは「10msごと」を正確に刻むタイマ。処理にかかった時間の分だけ
    // 待ち時間を自動で短くしてくれるので、周期がずれません。
    let mut ticker = Ticker::every(Duration::from_millis(10));

    loop {
        ticker.next().await;

        // 1行ずつLowにして、その行のキーを読む
        for (r, row) in rows.iter_mut().enumerate() {
            row.set_low();
            // 配線の電圧が落ち着くまで少し待つ（セトリング時間）
            Timer::after(Duration::from_millis(1)).await;

            for (c, col) in cols.iter().enumerate() {
                // 列がLow = この行×列の交点のキーが押されている
                let raw_pressed = col.is_low();

                // デバウンサに通し、状態が確定変化したときだけイベント送信
                if let Some(pressed) = debouncers[r][c].update(raw_pressed) {
                    let event = KeyEvent {
                        row: r as u8,
                        col: c as u8,
                        pressed,
                    };
                    // キューが満杯のときは空きが出るまでここで待つ
                    CHANNEL.send(event).await;
                }
            }

            // 読み終わったら行をHigh（非選択）に戻す
            row.set_high();
        }
    }
}

/// ロガータスク: キーイベントを受信してログ表示し、押下でLEDをトグルする
#[embassy_executor::task]
async fn logger_task(mut led: Output<'static>) {
    loop {
        let event = CHANNEL.receive().await;
        if event.pressed {
            info!("キー ({},{}) 押された", event.row, event.col);
            // どのキーでも、押された瞬間にLEDを反転
            led.toggle();
        } else {
            info!("キー ({},{}) 離された", event.row, event.col);
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // probe-rs 経由の defmt(RTT) を初期化する（espflash 時は何もしない）
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // 行線: 出力。普段はHigh（非選択）にしておき、スキャン時だけLowにする
    let rows = [
        Output::new(peripherals.GPIO18, Level::High, OutputConfig::default()),
        Output::new(peripherals.GPIO19, Level::High, OutputConfig::default()),
    ];

    // 列線: 内部プルアップ付き入力。どのキーも押されていなければHigh、
    // 「Lowの行」とつながるキーが押されるとLowになる
    let input_config = InputConfig::default().with_pull(Pull::Up);
    let cols = [
        Input::new(peripherals.GPIO20, input_config),
        Input::new(peripherals.GPIO21, input_config),
    ];

    // LED用のGPIO10を出力に設定。最初は消灯（Low）
    let led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    info!("2×2キーマトリクスのスキャンを開始します");

    spawner.spawn(scan_task(rows, cols).unwrap());
    spawner.spawn(logger_task(led).unwrap());

    // mainはもう仕事がないので待機するだけ
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
