//! 13-adc-pwm: ポテンショメータでLEDの明るさを変える（ADC + LEDC PWM）
//!
//! GPIO2（ADC1チャンネル2）でポテンショメータ（可変抵抗）の電圧を読み取り、
//! その値をLEDC PWMのデューティ比に変換してGPIO10のLEDの明るさを変えます。
//! つまみを回すとLEDが明るくなったり暗くなったりします。
//!
//! - ADC: ADC1をワンショット（1回ずつ読む）モード + 非同期(async)で使用
//! - PWM: LEDCタイマー0を5kHz・12bit分解能に設定し、チャンネル0でGPIO10を駆動
//!
//! 配線:
//! - ポテンショメータの両端 → 3V3 と GND
//! - ポテンショメータの中央端子（ワイパー） → GPIO2
//! - GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(-) → GND
//!
//! 注意: ADC・LEDCはesp-halの unstable API です（将来のバージョンで変わる可能性があります）。

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcCalBasic, AdcConfig, Attenuation};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::DriveMode;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer};
use esp_hal::peripherals::ADC1;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use log::info;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

// ADCの校正方式。AdcCalBasicは「0Vのときに読み値が0になる」ようにバイアスを補正する
// いちばん基本的な校正です（補正値はチップ内のeFuseから読み出されます）
type AdcCal = AdcCalBasic<ADC1<'static>>;

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // --- ADCの設定 ---
    // GPIO2をADC1の入力として有効化。減衰11dBで0V〜約3.3Vの範囲を測定できる
    let mut adc1_config = AdcConfig::new();
    let mut pot_pin =
        adc1_config.enable_pin_with_cal::<_, AdcCal>(peripherals.GPIO2, Attenuation::_11dB);
    // into_async()で非同期版に変換。読み取り中はawaitで他のタスクに実行を譲れる
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config).into_async();

    // --- LEDC(PWM)の設定 ---
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    // タイマー0を「5kHz・12bit分解能」に設定。
    // 12bit = デューティを0〜4095の4096段階で表せる、という意味
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty12Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(5),
        })
        .unwrap();

    // チャンネル0にGPIO10を割り当て、タイマー0とひも付ける（最初はデューティ0% = 消灯）
    let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO10);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    info!("ポテンショメータを回してLEDの明るさを変えてみましょう");

    loop {
        // ADCを1回読む（12bitなので0〜4095）
        let raw: u16 = adc1.read_oneshot(&mut pot_pin).await;

        // ADCの読み値(0〜4095)をデューティ比(0〜100%)に変換する
        // 校正の補正でわずかに4095を超えることがあるためmin(100)で上限を保証
        let duty_pct = ((raw as u32 * 100) / 4095).min(100) as u8;
        channel0.set_duty(duty_pct).unwrap();

        info!("ADC生値 = {raw:4}, PWMデューティ = {duty_pct:3}%");

        Timer::after(Duration::from_millis(500)).await;
    }
}
