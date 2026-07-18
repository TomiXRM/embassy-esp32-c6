---
title: "7. Timerで待つ"
description: embassy-timeのTimer::afterとDurationを使った「CPUを止めない待ち」を学びます。ブロッキングのdelayとの違いを理解します。
part: 6
lesson: 7
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part06/01-gpio-output
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル（データ通信対応）
  - ブレッドボード
  - LED（赤など、砲弾型）
  - 抵抗 330Ω
  - ジャンパ線 2本
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1, embassy-time 0.5"
last_verified: "2026-07-18"
sources:
  - https://docs.rs/embassy-time/
  - https://embassy.dev/book/
---

## このページでできるようになること

- `Timer::after(Duration)`で指定時間待てる
- `Duration`の作り方（秒・ミリ秒など）を使い分けられる
- ブロッキングの`delay`と`.await`する待ちの違いを説明できる

## 先に結論

Embassyで「待つ」の基本は`Timer::after(Duration::from_millis(500)).await`です。時間の長さは`Duration`型で表し、`from_secs`/`from_millis`などで作ります。この待ちはCPUを止めません。待っているtaskだけが眠り、CPUはその間ほかのtaskを実行します。Arduinoの`delay()`のようにCPU全体を占有する待ち（ブロッキング）とは仕組みが根本的に違います。なお、Embassyの時計は`esp_rtos::start(...)`が起動するので、この行より前に`Timer`は使えません。

## 身近なたとえ

ラーメン屋の券売機の前で10分待つのがブロッキングの`delay`です。あなた（CPU）はその場に立ちっぱなしで、他のことは何もできません。一方`Timer::after(...).await`は、番号札をもらって席で待つ方式です。「10分後に呼んでね」と頼んでおけば、待っている間に宿題（他のtask）を進められます。

実際のマイコンでは、「呼んでね」の相手はハードウェアタイマーです。タイマーが指定時刻になると割り込みが発生し、前ページで学んだのと同じ仕組み（Waker）で眠っていたtaskが起こされます。エッジ待ちも時間待ちも、裏側は同じ「割り込み→Waker→再開」なのです。

## 仕組み

`Timer::after`の待ちで起きることを整理します。

1. `Timer::after(d).await`を実行すると、「現在時刻 + d」の起床時刻が予約される
2. taskはWakerを残して眠る。executorは他の実行可能なtaskへCPUを回す
3. ハードウェアタイマーが起床時刻に達すると割り込みが発生し、Wakerがtaskを起こす
4. taskは`.await`の次の行から再開する

`Duration`は時間の長さを表す型で、主な作り方は次のとおりです。

| 書き方 | 意味 |
|---|---|
| `Duration::from_secs(1)` | 1秒 |
| `Duration::from_millis(500)` | 500ミリ秒 |
| `Duration::from_micros(100)` | 100マイクロ秒 |

`Timer::after_millis(500)`のような短縮形もありますが、この教材のコードは`Timer::after(Duration::from_millis(500))`の形で統一しています。

## Arduinoではどう書くか

```cpp
delay(500);  // 500ms、CPUはここで完全に停止する
```

`delay()`の間、CPUは空ループを回っているだけで、他の処理は一切進みません。LED点滅とボタン監視を同時にやろうとすると、`delay`のせいでボタンを取りこぼす。これがArduinoの`loop`一枚岩の典型的な壁でした（[第1部 2. ArduinoからRustへ移る理由](/embassy-esp32-c6/part01/02-why-rust/)）。

## RustとEmbassyではどう書くか

`examples/01-blinky`の点滅ループが最小の実例です。これは抜粋です。完全なコードは `examples/01-blinky` を見てください。

```rust
use embassy_time::{Duration, Timer};

loop {
    led.set_high(); // 点灯
    Timer::after(Duration::from_millis(500)).await;
    led.set_low(); // 消灯
    Timer::after(Duration::from_millis(500)).await;
}
```

複数のtaskがそれぞれ自分のペースで待つ例が`examples/06-embassy-tasks`です。mainは5秒ごと、別のtaskは1秒ごとに動きます。どちらの「待ち」もCPUを止めないので、両立できます。

```rust
// mainの生存確認ループ（抜粋）
loop {
    Timer::after(Duration::from_secs(5)).await;
    info!("[main] 動作中です（ハートビート）");
}
```

## コードを一行ずつ読む

```rust
Timer::after(Duration::from_millis(500)).await;
```

- `Duration::from_millis(500)` — 「500ミリ秒」という長さの値を作ります
- `Timer::after(...)` — 「今から500ミリ秒後に完了するFuture」を作ります。**この時点ではまだ待っていません**
- `.await` — ここで初めて待ちが始まります。taskは眠り、時間が来たら次の行へ進みます

```rust
esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
```

すべての例の冒頭にあるこの行が、Embassyの時計（タイマードライバ）とtask切り替えの仕組みを起動しています。`Timer`や`Ticker`はこの行が実行済みであることを前提に動きます。

## 実行方法

```bash
cargo run --release
```

`examples/06-embassy-tasks`を動かすと、LEDの点滅（500ms周期）・カウンタログ（1秒ごと）・ハートビート（5秒ごと）が同時に進むことを確認できます。

## よくある失敗

- **`.await`を忘れて一瞬で通過する**: `Timer::after(...)`だけではFutureを作っただけで、待ちは始まりません。点滅がおかしいときはまず`.await`の付け忘れを疑ってください
- **`esp_rtos::start`の前に`Timer`を使って動かない**: Embassyの時計がまだ起動していないためです。初期化の順序（`esp_hal::init`→`esp_rtos::start`→アプリの処理）を守ってください
- **ミリ秒のつもりで`from_secs`と書く**: `from_secs(500)`は500**秒**です。点滅が止まったように見えたら単位を確認しましょう
- **正確な周期処理を`Timer::after`の繰り返しで作ろうとする**: ループ内の処理時間のぶんだけ周期が少しずつ延びます。周期処理には次のページの`Ticker`を使います

## やってみよう

`examples/01-blinky`の2つの`Timer::after`を、点灯100ミリ秒・消灯900ミリ秒に変えてみましょう。「短く光って長く休む」信号灯のような点滅になります。`Duration::from_secs`と`from_millis`を混ぜて書いても構いません。

## 確認問題

1. `Timer::after(Duration::from_millis(500))`と書いた時点では、まだ待ちが始まっていません。待ちが始まるのはいつですか。
2. `delay(500)`（ブロッキング）と`Timer::after(...).await`の違いを、CPUの視点で説明してください。
3. `Timer::after`による待ちの裏側で、taskを起こしているのは何ですか。

<details>
<summary>答え</summary>

1. `.await`を実行した時点です。`Timer::after`はFuture（完了予定の予約票）を作るだけです。
2. `delay(500)`はCPUがその場で空回りして他の処理が一切進みません。`Timer::after(...).await`は待つtaskだけが眠り、CPUはその間に他のtaskを実行できます。
3. ハードウェアタイマーの割り込みです。予約した起床時刻になると割り込みが発生し、Wakerを通じて眠っていたtaskが実行可能に戻されます。

</details>

## まとめ

- 待ちの基本は`Timer::after(Duration::from_millis(500)).await`。時間の長さは`Duration`で作る
- `.await`する待ちはtaskだけを眠らせ、CPUは他のtaskに回る。ブロッキングの`delay`とは別物
- Embassyの時計は`esp_rtos::start`が起動する。使う前に必ず初期化する

## 次のページ

`Timer::after`の繰り返しでは、処理時間のぶんだけ周期が少しずつずれます。「ずれない周期処理」を作る専用の道具、`Ticker`を学びます。

- 前: [6. GPIO割り込みとasync wait](/embassy-esp32-c6/part06/06-gpio-interrupt/)
- 次: [8. Tickerで周期実行](/embassy-esp32-c6/part06/08-ticker/)
