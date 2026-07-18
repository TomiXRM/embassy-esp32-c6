---
title: "10. 最初のLチカ"
description: ESP32-C6のGPIO10に外付けLEDをつなぎ、RustとEmbassyで1秒間隔の点滅（Lチカ）をさせます。コードを一行ずつ読み解きます。
part: 1
lesson: 10
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part01/09-flash-monitor
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル（データ通信対応）
  - ブレッドボード
  - LED（赤など、砲弾型）
  - 抵抗 330Ω
  - ジャンパ線 2本
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
---

## このページでできるようになること

- 外付けLEDをESP32-C6-DevKitC-1に正しく配線できる
- RustとEmbassyでLEDを点滅させられる
- Lチカのコードを一行ずつ、自分の言葉で説明できる
- ボード上のRGB LEDが単純なON/OFFで光らない理由を説明できる

## 先に結論

LチカはGPIO10を出力に設定し、「点ける→待つ→消す→待つ」を無限に繰り返すだけのプログラムです。待つ処理には`Timer::after(...).await`を使います。配線は「GPIO10 → 330Ωの抵抗 → LEDの足の長いほう → LEDの足の短いほう → GND」です。ボードに載っているRGB LED（GPIO8）はWS2812Bというアドレサブル（信号制御式）LEDなので、単純なON/OFFでは光りません。だから今回は外付けLEDを使います。

## 身近なたとえ

Lチカは「部屋の電気のスイッチを、決まったリズムでカチカチし続けるロボット」です。スイッチを入れる（`set_high`）、1秒待つ、切る（`set_low`）、1秒待つ。これの繰り返しです。

ただし実際のマイコンでは、スイッチが動かすのは照明ではなく「GPIOピンの電圧」です。ピンの電圧を3.3V（High）と0V（Low）に切り替えることで、つながっているLEDが点いたり消えたりします。

## なぜボードのLEDを使わないのか

ESP32-C6-DevKitC-1にはGPIO8にRGB LEDが載っています。しかしこれは**WS2812B**というアドレサブルLEDで、内部に小さな制御チップが入っています。色と明るさのデータを決められたタイミングの信号で送らないと光りません。ピンをHighにするだけでは点灯しないのです。

もうひとつの赤いLEDは電源表示用で、プログラムからは制御できません。つまり**このボードには、単純なON/OFFで光るユーザーLEDがありません**。そこで、外付けのLEDをGPIO10につないで使います。

## 配線

| つなぐ順番 | 部品 | 注意 |
|---|---|---|
| 1 | ボードの**GPIO10**ピン | ピンヘッダの刻印を確認 |
| 2 | **抵抗330Ω** | 向きはどちらでもよい |
| 3 | **LEDのアノード（足が長いほう、＋）** | 向きを間違えると光らない |
| 4 | **LEDのカソード（足が短いほう、−）** | |
| 5 | ボードの**GND**ピン | |

```text
GPIO10 ──[330Ω]──▶|── GND
                 LED
        （▶| はLED。長い足が抵抗側）
```

注意事項は3つです。

- **抵抗を必ず入れる**こと。抵抗なしで直結すると、LEDとGPIOピンに大きすぎる電流が流れます（理由は[6. 電圧と電流の最低限](/embassy-esp32-c6/part01/06-volt-current/)で説明した通りです）
- **LEDには向きがある**こと。逆向きだと壊れはしませんが光りません
- 配線は**USBケーブルを抜いた状態**で行うこと

## RustとEmbassyではどう書くか

これが完全なコードです（`examples/01-blinky/src/main.rs`）。

```rust
//! 01-blinky: 最初のLチカ

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use log::info;

// esp-idf形式ブートローダが要求するアプリ記述子
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // GPIO10を出力に設定。最初は消灯（Low）
    let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    info!("Lチカを開始します");

    loop {
        led.set_high(); // 点灯
        Timer::after(Duration::from_millis(500)).await;
        led.set_low(); // 消灯
        Timer::after(Duration::from_millis(500)).await;
    }
}
```

## コードを一行ずつ読む

重要な行だけを取り上げます。今は「そういうおまじないだ」で構わない行もあります。第5部と第9部でぜんぶ種明かしします。

```rust
#![no_std]
#![no_main]
```

OSがないマイコンなので、OS前提の標準ライブラリ（std）と、ふつうの`main`の起動方法を使わない、という宣言です。詳しくは[第5部](/embassy-esp32-c6/part05/01-no-std/)で扱います。

```rust
esp_bootloader_esp_idf::esp_app_desc!();
```

ブートローダ（電源投入直後に動くプログラム）へ「これは正しいアプリです」と伝える情報を埋め込みます。1回書くだけのお約束です。

```rust
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
```

Embassyで動く`main`の入口です。`async`は「待っている間に他の仕事へ切り替えられる関数」の印。戻り値の`-> !`は「この関数は永遠に戻らない」という意味です。マイコンのプログラムには「終了」がないためです。

```rust
let peripherals = esp_hal::init(config);
```

チップの初期化です。`peripherals`は、GPIOやタイマーなど**ペリフェラル（周辺機器）ぜんぶの所有権**をまとめた変数です。ここから`GPIO10`などを取り出して使います。

```rust
esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
```

Embassyが時間を測るためのタイマーと、taskを切り替える仕組みを起動します。`Timer::after`はこれがないと動きません。

```rust
let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
```

GPIO10を**出力ピン**として構え、初期状態をLow（0V＝消灯）にします。あとで`set_high`/`set_low`で状態を変えるので`mut`（変更可能）を付けます。

```rust
loop {
    led.set_high(); // 点灯
    Timer::after(Duration::from_millis(500)).await;
    led.set_low(); // 消灯
    Timer::after(Duration::from_millis(500)).await;
}
```

本体です。ピンをHigh（3.3V）にして500ミリ秒待ち、Low（0V）にして500ミリ秒待つ。`loop`は無限ループなので、これが電源を切るまで続きます。`.await`は「待っている間、CPUを無駄に回さない待ち方」です。第9部の主役なので、今は`delay(500)`の仲間だと思ってください。

## 実行方法

配線を確認してからUSBケーブルをつなぎ、プロジェクトのフォルダで実行します。

```bash
cargo run --release
```

ビルド→書き込み→モニタ表示が自動で行われます（前のページで設定した通りです）。期待される結果は次の2つです。

- モニタに `Lチカを開始します` と表示される
- LEDが0.5秒点灯・0.5秒消灯を繰り返す

## よくある失敗

- **LEDが光らない（ログは出ている）**: ほとんどが配線ミスです。①LEDの向き（長い足が抵抗側・GPIO10側）、②GNDにつないでいるか、③GPIO10の隣のピンに挿していないか、の順に確認してください。プログラムは動いているので、ハード側の問題です
- **書き込みに失敗する**: ポートを選び間違えたか、他のモニタが開きっぱなしです。前ページの[よくある失敗](/embassy-esp32-c6/part01/09-flash-monitor/)を見直してください
- **ボードのRGB LEDを光らせようとGPIO8に変えたが光らない**: WS2812Bは専用の信号が必要なため、このコードでは光りません。仕様どおりの動きです
- **`Timer::after`でビルドエラーが出る**: `esp_rtos::start(...)`を消したり、呼ぶ前に`Timer`を使ったりすると起きます。Embassyの時計はこの行が起動しているためです

## やってみよう

`from_millis(500)`の数字を2か所とも`100`に変えて、もう一度`cargo run --release`してみてください。点滅が速くなります。次に、点灯500ミリ秒・消灯1500ミリ秒のように**非対称な点滅**にしてみましょう。

## 確認問題

1. ボードに載っているRGB LED（GPIO8）が`set_high()`で光らないのはなぜですか。
2. `let mut led = ...`の`mut`を消すとコンパイルエラーになります。なぜだと思いますか。
3. LEDと直列に入れた330Ωの抵抗は何のためですか。

<details>
<summary>答え</summary>

1. WS2812Bというアドレサブル（信号制御式）LEDだからです。色と明るさのデータを専用のタイミングの信号で受け取らないと光りません。単純な電圧のHigh/Lowでは点灯しません。
2. `set_high()`/`set_low()`はピンの状態を**変更**するメソッドだからです。Rustでは変数は最初、変更不可（不変）で、変更するには`mut`が必要です（詳しくは第2部で学びます）。
3. LEDに流れる電流を安全な大きさに制限するためです。抵抗がないとLEDとGPIOピンに過大な電流が流れ、壊れる恐れがあります。

</details>

## まとめ

- Lチカ＝GPIOを出力にして、High/Lowの切り替えと待ちを無限ループで繰り返す
- 待ちは`Timer::after(Duration::from_millis(500)).await`。Embassyの時計は`esp_rtos::start`が起動する
- ボードのRGB LED（GPIO8のWS2812B）は単純なON/OFFでは光らないので、外付けLED（GPIO10）を使う

## 次のページ

おめでとうございます、環境構築とLチカが完了しました。第2部からはRustという言語そのものを、変数から一歩ずつ学びます。Lチカのコードに出てきた`let`や`mut`の意味がはっきり分かるようになります。

- 前: [9. 書き込みとシリアル表示](/embassy-esp32-c6/part01/09-flash-monitor/)
- 次: [第2部 1. 変数とlet](/embassy-esp32-c6/part02/01-variables/)
