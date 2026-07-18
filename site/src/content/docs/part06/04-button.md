---
title: "4. ボタンを読む"
description: ボード上のBOOTボタン（GPIO9）を読み、押すたびに外付けLEDをトグルします。asyncのエッジ待ちで「押された瞬間」を捕まえます。
part: 6
lesson: 4
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part06/03-pull-updown
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
  - https://documentation.espressif.com/esp32-c6_datasheet_en.pdf
---

## このページでできるようになること

- ボード上のBOOTボタン（GPIO9）をプログラムから読める
- 「レベル（今の状態）」と「エッジ（変化の瞬間）」の違いを説明できる
- `wait_for_falling_edge().await`で押された瞬間を捕まえられる
- BOOTボタンがストラッピングピンでもあることの注意点が分かる

## 先に結論

ESP32-C6-DevKitC-1にはBOOTボタンが載っていて、GPIO9につながっています。押すとGNDに落ちる配線なので、内部プルアップと組み合わせて「離すとHigh、押すとLow」で読めます。押された**瞬間**を捕まえるにはHigh→Lowの変化（フォーリングエッジ）を待つのが確実で、Embassyでは`button.wait_for_falling_edge().await`と書くだけです。待っている間CPUは他の仕事ができます。なおGPIO9はストラッピングピンでもあり、**BOOTボタンを押したままリセットすると書き込みモードで起動**します。これは故障ではなく仕様です。

## 身近なたとえ

「今の状態を読む」ことと「変化の瞬間を捕まえる」ことは別物です。玄関のチャイムで考えてみましょう。`is_low()`で状態を読み続けるのは、「今ボタンが押されてるかな？」と何度もドアを見に行く方法です。エッジ待ちは、チャイムが**鳴った瞬間に**気づく方法です。見に行き続ける必要はなく、鳴るまで別のことをしていられます。

実際のプログラムでは、「鳴った瞬間に気づく」を実現しているのはチップのGPIO割り込みという仕組みです（詳しくは[6. GPIO割り込みとasync wait](/embassy-esp32-c6/part06/06-gpio-interrupt/)で説明します）。このページでは、まず使い方をマスターします。

## 仕組み

信号の変化には名前が付いています。

```text
High ────┐          ┌────────
         │          │
Low      └──────────┘
         ↑          ↑
   フォーリングエッジ  ライジングエッジ
   （High→Low）      （Low→High）
   ＝押された瞬間     ＝離された瞬間
```

BOOTボタンはプルアップ方式（押す＝Low）なので、**押された瞬間＝フォーリングエッジ**、**離された瞬間＝ライジングエッジ**です。

esp-halの`Input`型には、これを待つasyncメソッドが最初から生えています。

- `wait_for_falling_edge().await` — High→Lowの変化まで待つ
- `wait_for_rising_edge().await` — Low→Highの変化まで待つ

`.await`しているあいだ、このtaskは眠っていて、CPUは他のtaskを動かせます。ポーリング（読み続けるループ）と違ってCPU時間を浪費しません。

## RustとEmbassyではどう書くか

これが完全なコードです（`examples/02-button/src/main.rs`）。押すたびにGPIO10のLEDをトグルし、回数をログに出します。

```rust
//! 02-button: BOOTボタンでLEDをトグル

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
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

    // LED用のGPIO10を出力に設定。最初は消灯（Low）
    let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    // BOOTボタン（GPIO9）を入力に設定。
    // ボタンはGPIO9とGNDの間に入っているので、内部プルアップを有効にして
    // 「離している間はHigh、押すとLow」になるようにします。
    let config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(peripherals.GPIO9, config);

    info!("BOOTボタンを押すとLEDが切り替わります");

    let mut count: u32 = 0;

    loop {
        // High→Lowの変化（＝ボタンが押された瞬間）をawaitで待ちます。
        // 待っている間、CPUは他の仕事ができます（ポーリング不要）。
        button.wait_for_falling_edge().await;

        // チャタリング対策: 機械式ボタンは押した瞬間に接点が細かくバタつくので、
        // 30ms待ってから本当に押されているかを確認します。
        Timer::after(Duration::from_millis(30)).await;
        if button.is_low() {
            count += 1;
            led.toggle();
            info!("ボタンが押されました（{}回目）", count);

            // ボタンが離される（Low→High）まで待ってから次の押下を受け付けます。
            button.wait_for_rising_edge().await;
            // 離すときにもチャタリングが起きるので、少し待って落ち着かせます。
            Timer::after(Duration::from_millis(30)).await;
        }
    }
}
```

## コードを一行ずつ読む

```rust
let config = InputConfig::default().with_pull(Pull::Up);
let mut button = Input::new(peripherals.GPIO9, config);
```

前ページで学んだとおり、内部プルアップを有効にしてGPIO9を入力にします。BOOTボタンはボードに配線済みなので、外付け部品は不要です。

```rust
button.wait_for_falling_edge().await;
```

このプログラムの心臓部です。「押された瞬間」までtaskを眠らせます。ボタンが押されない限り、この行から先へは進みません。

```rust
Timer::after(Duration::from_millis(30)).await;
if button.is_low() {
```

エッジを検出した後、30ms待ってから`is_low()`で「本当にまだ押されているか」を確認しています。機械式ボタンは押した瞬間に接点が細かくバタつく（チャタリング）ため、その対策です。仕組みは次のページで詳しく学びます。

```rust
button.wait_for_rising_edge().await;
```

離される（Low→High）まで待ちます。これがないと、押しっぱなしのあいだに次のフォーリングエッジ待ちへ戻ってしまい、1回の押下を複数回と数える恐れがあります。「押す→処理→離すのを待つ」の順で1押下＝1イベントを保証しています。

## 配線

LEDは第1部のLチカと同じく「GPIO10 → 330Ω → LEDアノード → LEDカソード → GND」です。ボタン側の配線は**不要**です。ボード上のBOOTボタンをそのまま使います。

## 実行方法

```bash
cargo run --release
```

期待される結果は次のとおりです。

- モニタに `BOOTボタンを押すとLEDが切り替わります` と表示される
- BOOTボタンを押すたびにLEDが点灯⇔消灯と切り替わる
- `ボタンが押されました（1回目）` のように回数がログに出る

## よくある失敗

- **書き込み後にプログラムが動かない／変なモードで起動した**: BOOTボタンを**押したままリセット（または電源投入）**していませんか。GPIO9はストラッピングピンで、起動の瞬間にLowだとチップは「書き込みモード」で立ち上がり、あなたのプログラムは動きません。故障ではありません。ボタンを離した状態でRSTボタンを押せば普通に起動します
- **押していないのに反応する／1回押したのに2回数える**: チャタリングや配線ノイズです。このコードには30msの対策が入っていますが、削って動かすと高い確率で再現します。次のページで正体を学びます
- **`is_high()`で押下判定を書いてしまい、動きが逆になる**: BOOTボタンはプルアップ方式なので**押す＝Low**です。判定は`is_low()`です
- **`wait_for_falling_edge`のあとに`.await`を忘れる**: asyncメソッドは`.await`しないと実行されません。コンパイラが「Futureが使われていない」と警告してくれるので、警告は必ず読みましょう

## やってみよう

`led.toggle()`の代わりに「押している間だけ点灯」に変えてみましょう。ヒント: フォーリングエッジ検出後に`led.set_high()`、`wait_for_rising_edge().await`の後に`led.set_low()`です。5分でできます。

## 確認問題

1. BOOTボタンを押した瞬間、GPIO9の信号はどちら向きのエッジになりますか。
2. `wait_for_rising_edge().await`を削除すると、押しっぱなしのときに何が起きる可能性がありますか。
3. BOOTボタンを押したままリセットするとプログラムが起動しないのはなぜですか。

<details>
<summary>答え</summary>

1. フォーリングエッジ（High→Low）です。プルアップにより普段はHighで、押すとGNDへつながってLowになるからです。
2. ループが先頭へ戻って再びフォーリングエッジを待ちますが、チャタリングやノイズによる小さな変化を拾って、1回の押下を複数回として数えてしまう恐れがあります。「離されるまで待つ」ことで1押下＝1イベントにしています。
3. GPIO9はストラッピングピンで、リセット直後の値でチップの起動モードが決まるからです。GPIO9がLow（押されたまま）だと書き込みモードで起動し、フラッシュ内のプログラムは実行されません。

</details>

## まとめ

- BOOTボタン＝GPIO9。プルアップ方式なので「離すとHigh、押すとLow」
- 押された瞬間は`wait_for_falling_edge().await`で捕まえる。待っている間、CPUは他の仕事ができる
- GPIO9はストラッピングピンでもある。押したままリセットすると書き込みモードになるのは仕様

## 次のページ

コードに入っていた「30ms待つ」の正体、チャタリングを掘り下げます。機械式スイッチを使う限り避けて通れない、実践的な必修テーマです。

- 前: [3. Pull-upとPull-down](/embassy-esp32-c6/part06/03-pull-updown/)
- 次: [5. チャタリング対策](/embassy-esp32-c6/part06/05-debounce/)
