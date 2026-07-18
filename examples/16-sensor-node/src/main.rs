//! 16-sensor-node: 電池駆動を想定したセンサノードのデューティサイクル
//!
//! BME280（温度・湿度・気圧センサ）を毎回の起動で1回だけ読み取り、
//! 測定値をRTC RAM上の履歴バッファに追記してから30秒のディープスリープに
//! 入る、という「起動→測定→就寝」のサイクルを繰り返します。
//! 電池で長期間動かすセンサノードの基本パターンです（この例では通信は省略）。
//!
//! 【重要】ディープスリープに入るとHP SRAM（メインメモリ）は消え、
//! 復帰後はプログラムが「最初から」実行し直されます。普通のstatic変数も
//! 毎回初期値に戻ります。唯一、RTC（LP）ドメインのRAMだけが通電され続ける
//! ため、`#[ram(unstable(rtc_fast))]` を付けたstatic変数はスリープをまたいで
//! 値が残ります（電源を抜くと消えます。USBを挿し直すと0から再開）。
//!
//! 配線（BME280モジュール）:
//! - VCC → 3.3V（5V専用モジュールでない限り3.3Vへ）
//! - GND → GND
//! - SDA → GPIO6
//! - SCL → GPIO7
//! （多くのモジュールはI2Cプルアップ抵抗を内蔵。無い場合はSDA/SCLを
//! 　それぞれ10kΩで3.3Vへプルアップする）
//!
//! I2Cアドレス: bme280-rsクレートの既定値は0x76（SDOピンがGNDのモジュール）。
//! SDOがVDDに接続されたモジュールは0x77になるので、その場合は
//! `AsyncBme280::new_with_address(i2c, 0x77, Delay)` を使う。
//!
//! このサンプルの構成は esp32c3-embassy プロジェクト
//! <https://gitlab.com/claudiomattera/esp32c3-embassy>（MIT OR Apache-2.0）
//! のデューティサイクル設計を参考に、ESP32-C6向けに簡略化したものです。
//!
//! 注意: rtc_cntl（スリープ関連）と #[ram] 属性はesp-halの unstable API です。

#![no_std]
#![no_main]

use core::time::Duration as CoreDuration;

use bme280_rs::{AsyncBme280, Configuration, Oversampling, Sample, SensorMode};
use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use esp_backtrace as _;
// defmt の global_logger をリンクする。probe-rs では rtt-target、
// espflash では esp-println がそれぞれ defmt ログの出口になる。
use esp_hal::Async;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, Error as I2cError, I2c};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ram;
use esp_hal::rtc_cntl::sleep::TimerWakeupSource;
use esp_hal::rtc_cntl::{Rtc, reset_reason, wakeup_cause};
use esp_hal::system::Cpu;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
#[cfg(feature = "espflash")]
use esp_println as _;
#[cfg(feature = "probe-rs")]
use rtt_target as _;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

/// 起動後、ディープスリープに入るまでの待ち時間
const AWAKE_TIME: Duration = Duration::from_secs(3);

/// ディープスリープの長さ
const SLEEP_TIME: CoreDuration = CoreDuration::from_secs(30);

/// 温度履歴のサイズ（直近8回分を保持するリングバッファ）
const HISTORY_SIZE: usize = 8;

// ---- RTC RAMに置く変数たち ----
// `#[ram(unstable(rtc_fast))]` を付けたstaticはRTC Fastメモリに配置される。
// この領域は電源投入時にブートローダが初期値を書き込むが、ディープスリープ
// からの復帰時には書き込みがスキップされるため、スリープ前の値がそのまま残る。
// （esp32c3-embassyプロジェクトで実績のあるパターン）

/// 何回目の起動かを数えるカウンタ（電源投入時のみ0に戻る）
#[ram(unstable(rtc_fast))]
static mut BOOT_COUNT: u32 = 0;

/// 過去の温度[℃]を保持するリングバッファ
#[ram(unstable(rtc_fast))]
static mut TEMP_HISTORY: [f32; HISTORY_SIZE] = [0.0; HISTORY_SIZE];

/// これまでに履歴へ書き込んだ総回数。
/// 書き込み位置は TEMP_COUNT % HISTORY_SIZE、有効な件数は min(TEMP_COUNT, 8)
#[ram(unstable(rtc_fast))]
static mut TEMP_COUNT: u32 = 0;

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    // probe-rs 経由の defmt(RTT) を初期化する（espflash 時は何もしない）
    #[cfg(feature = "probe-rs")]
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // なぜ起動したのかを表示する。
    // 電源投入なら PowerOn 系、ディープスリープ復帰なら CoreDeepSleep + Timer になる
    // reset_reason/wakeup_cause の戻り値型は defmt::Format 非対応（Debugのみ）
    // なので Debug2Format でラップして出力する
    info!(
        "リセット要因: {}",
        defmt::Debug2Format(&reset_reason(Cpu::ProCpu))
    );
    info!("復帰要因: {}", defmt::Debug2Format(&wakeup_cause()));

    // RTC RAM上のstatic変数への可変参照をここで1回だけ作る。
    //
    // SAFETY: ESP32-C6はシングルコアで、このプログラムではタスクも割り込み
    // ハンドラもこれらのstaticに触らない。可変参照を作るのがこの1箇所だけ
    // なので、Rustの「可変参照はただ1つ」のルールを破らず安全に使える。
    // （複数タスクから共有する場合はMutexなどの同期機構が必要になる）
    let boot_count: &mut u32 = unsafe { &mut *(&raw mut BOOT_COUNT) };
    let temp_history: &mut [f32; HISTORY_SIZE] = unsafe { &mut *(&raw mut TEMP_HISTORY) };
    let temp_count: &mut u32 = unsafe { &mut *(&raw mut TEMP_COUNT) };

    *boot_count += 1;
    info!("起動回数: {}", *boot_count);

    // --- BME280の初期化と測定 ---
    // I2C0を100kHzで初期化し、SDA=GPIO6 / SCL=GPIO7 を割り当てて非同期モードへ
    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("I2Cの設定が不正です")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    // アドレス0x76（既定値）のBME280ドライバを作る。0x77のモジュールなら
    // AsyncBme280::new_with_address(i2c, 0x77, Delay) にする
    let mut sensor = AsyncBme280::new(i2c, Delay);

    // センサから温度[℃]を1回読み取る。失敗しても止まらず、プレースホルダ値
    // （NaN）で続行する「劣化運転」にする。電池駆動のノードは、センサが
    // 一時的に不調でもスリープのサイクル自体は守り続けるのが望ましい
    let temperature_c: f32 = match measure(&mut sensor).await {
        Ok(sample) => {
            // read_sample()は測定項目ごとに Option<f32> を返す
            // （設定で無効化した項目は None になる）ため、matchで取り出す
            match (sample.temperature, sample.humidity, sample.pressure) {
                (Some(t), Some(h), Some(p)) => {
                    // 気圧はPa単位で返るので、なじみのあるhPaに直して表示。
                    // defmt は精度指定（{:.2}等）が使えないため、f32をそのまま出す
                    info!(
                        "温度: {=f32} C / 湿度: {=f32} %RH / 気圧: {=f32} hPa",
                        t,
                        h,
                        p / 100.0
                    );
                    t
                }
                (t, h, p) => {
                    // Option<f32> は defmt::Format を実装するので {} で整形できる
                    warn!("一部の測定値が取得できません: {} {} {}", t, h, p);
                    // 温度だけでも取れていればそれを使う
                    t.unwrap_or(f32::NAN)
                }
            }
        }
        Err(e) => {
            // センサ未接続・配線ミスなどでもサイクルは継続する（劣化運転）
            error!("BME280の測定に失敗: {}（プレースホルダ値で続行）", e);
            f32::NAN
        }
    };

    // --- 温度をリングバッファへ追記 ---
    let index = (*temp_count as usize) % HISTORY_SIZE;
    temp_history[index] = temperature_c;
    *temp_count += 1;

    // --- 履歴を古い順に表示 ---
    let valid = (*temp_count as usize).min(HISTORY_SIZE);
    info!("温度履歴（直近{}件・古い順）:", valid);
    for i in 0..valid {
        // バッファが一周した後は、次に書き込む位置(temp_count % 8)が
        // いちばん古いデータの位置になる
        let pos = if (*temp_count as usize) <= HISTORY_SIZE {
            i
        } else {
            ((*temp_count as usize) + i) % HISTORY_SIZE
        };
        // defmt は精度指定が使えないため f32 をそのまま出す
        info!("  [{}] {=f32} C", i, temp_history[pos]);
    }

    info!(
        "{}秒後に{}秒間のディープスリープへ入ります…",
        AWAKE_TIME.as_secs(),
        SLEEP_TIME.as_secs()
    );
    Timer::after(AWAKE_TIME).await;

    // ディープスリープへ。この関数からは戻らず、30秒後にRTCタイマーで
    // 復帰するとプログラムは最初から実行される（RTC RAMの値だけが残る）
    let timer_wakeup = TimerWakeupSource::new(SLEEP_TIME);
    let mut rtc = Rtc::new(peripherals.LPWR);
    rtc.sleep_deep(&[&timer_wakeup]);
}

/// BME280を初期化し、温度・湿度・気圧を1回測定する
async fn measure(sensor: &mut AsyncBme280<I2c<'static, Async>, Delay>) -> Result<Sample, I2cError> {
    // ソフトリセット＋キャリブレーション係数の読み出し
    sensor.init().await?;

    // 3項目ともオーバーサンプリング1倍・Normalモード（連続測定）に設定
    sensor
        .set_sampling_configuration(
            Configuration::default()
                .with_temperature_oversampling(Oversampling::Oversample1)
                .with_pressure_oversampling(Oversampling::Oversample1)
                .with_humidity_oversampling(Oversampling::Oversample1)
                .with_sensor_mode(SensorMode::Normal),
        )
        .await?;

    // 設定反映と最初の測定完了を待つ（ウォームアップ）
    Timer::after(Duration::from_millis(10)).await;

    sensor.read_sample().await
}
