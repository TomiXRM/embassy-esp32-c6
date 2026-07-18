---
title: "6. EmbassyのTimerとInstant"
description: Timer::after系・Instant・Duration・Ticker・Delayの時間APIを、用途で使い分けられるようになります。
part: 9
lesson: 6
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part09/04-task
  - part06/07-timer
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - LED
  - 抵抗330Ω
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal ~1.1.0 / esp-rtos 0.3.0 / embassy-executor 0.10.0 / embassy-time 0.5"
last_verified: "2026-07-18"
sources:
  - https://docs.rs/embassy-time/0.5.1
  - https://embassy.dev/book/
---

## このページでできるようになること

- `Timer::after`系・`Instant`・`Duration`・`Ticker`を用途で使い分けられる
- `Timer::after`の繰り返しでは周期がずれ、`Ticker`ならずれにくい理由を説明できる
- `Delay`が何のためにあるか（embedded-halのドライバ互換）を説明できる

## 先に結論

embassy-timeの時間APIは役割で選びます。**「今から◯ms待つ」は`Timer::after`**、**「ずれない周期で繰り返す」は`Ticker`**、**「経過時間を測る」は`Instant`と`Duration`**、**「embedded-hal(-async)のドライバに渡す待ち係」は`Delay`**です。どの待ちも`.await`で順番を譲るので、他のtaskを止めません。

## 身近なたとえ

- `Timer::after` — キッチンタイマー。「今から3分」を計ります。
- `Ticker` — 学校のチャイム。作業に何分かかろうと、次のチャイムは**時間割どおり**に鳴ります。
- `Instant` — ストップウォッチのラップ表示。「スタートから今まで何秒か」を読みます。
- `Duration` — 「500ミリ秒」「3秒」といった**時間の長さ**そのもの。

実際の技術との違い: チャイムやタイマーは音で人を呼びますが、embassy-timeでは満了がFutureの完成（`Ready`）として通知され、taskは満了までWFIで眠っています。

## 仕組み

主なAPIを一覧にします。

| API | 役割 | 例 |
|---|---|---|
| `Timer::after(Duration)` | 今から指定時間待つ | `Timer::after(Duration::from_secs(5)).await` |
| `Timer::after_millis(n)`など | 上の省略形（`after_secs`/`after_micros`等） | `Timer::after_millis(500).await` |
| `Timer::at(Instant)` | 指定時刻まで待つ | `Timer::at(deadline).await` |
| `Ticker::every(Duration)` | ずれにくい周期タイマーを作る | `ticker.next().await`で次の周期を待つ |
| `Instant::now()` | 起動からの経過時刻を得る | `let start = Instant::now();` |
| `Duration` | 時間の長さ（`from_millis`等で作る） | `Duration::from_millis(500)` |
| `Delay` | embedded-hal(-async)の遅延トレイト実装 | 外部ドライバの引数に渡す |

**なぜ`Timer::after`の繰り返しはずれるのか。** `loop { 処理(3ms); Timer::after(500ms).await; }`と書くと、1周は「処理3ms＋待ち500ms＝503ms」になり、周回ごとにずれが積もります。`Ticker`は前回の起床時刻を基準に次の時刻を決める（処理にかかった時間を差し引く）ので、周期が保たれます。examples/06-embassy-tasks のLED点滅がTickerを使っているのはこのためです。

## RustとEmbassyではどう書くか

周期処理はTickerで書きます（examples/06-embassy-tasks より抜粋）。

```rust
#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    // Tickerは「一定周期の繰り返し」に向いています。
    // 処理にかかった時間を差し引いて次の起床時刻を決めるので、
    // Timer::afterの繰り返しよりも周期がずれにくいのが特長です。
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        led.toggle();
        ticker.next().await;
    }
}
```

一回きりの待ちと、経過時間の計測はこう書きます。

```rust
    // 一回きりの待ち
    Timer::after(Duration::from_secs(5)).await;

    // 経過時間の計測
    let start = Instant::now();
    Timer::after(Duration::from_millis(10)).await;
    let elapsed: Duration = start.elapsed();
    info!("経過: {} ms", elapsed.as_millis());
```

外部センサのドライバなど、embedded-hal-asyncの「待ち係」を要求する相手には`Delay`を渡します。

```rust
use embedded_hal_async::delay::DelayNs;

    let mut delay = embassy_time::Delay;
    delay.delay_ms(1).await;
```

これらは抜粋です。Tickerを使う完全なコードは examples/06-embassy-tasks を見てください。

## コードを一行ずつ読む

- `Ticker::every(Duration::from_millis(500))` — 「500msごと」の予定表を作ります。`mut`が必要なのは、Tickerが「次はいつ起きるか」を内部に覚えているからです。
- `ticker.next().await` — 次の予定時刻まで譲って眠ります。処理が長引いた分は自動で差し引かれます。
- `Instant::now()` — 起動からの経過を刻む時計の「今」を読みます。壁掛け時計（何時何分）ではなく、ストップウォッチであることに注意してください。
- `embassy_time::Delay` — それ自体は何もしない小さな型で、embedded-hal / embedded-hal-asyncの遅延トレイトを実装しています。「どんな待ち方をするか」をドライバに教える部品です（[embedded-halのページ](/embassy-esp32-c6/part05/10-embedded-hal/)参照）。

## 実行方法

```bash
cd examples/06-embassy-tasks
cargo run --release
```

LEDの点滅（Ticker、500ms）とカウンタ（Ticker、1秒）とハートビート（Timer::after、5秒）が、それぞれの周期で正しく進むことを確認してください。

## よくある失敗

1. **周期処理を`Timer::after`の繰り返しで書いて、周期がずれる** — 1周ごとに「処理時間」の分だけ遅れが積もります。1日動かすと分単位のずれになることもあります。周期が大事な処理は`Ticker`を使ってください。
2. **`Instant`を「時刻」だと思ってしまう** — `Instant`は起動からの経過であって、「何月何日何時」ではありません。電源を入れ直せば0からやり直しです。カレンダー上の時刻が必要なら、ネットワークからの時刻取得など別の仕組みが必要です。
3. **esp-halのブロッキングDelayと混同する** — `esp_hal::delay::Delay`は待つ間CPUを独占し、他のtaskを止めます。Embassyのtask内では`embassy_time`の`Timer`/`Ticker`/`Delay`（`.await`が付くもの）を使ってください。

## やってみよう

`counter_task`に`Instant`を仕込んで、起動からの経過秒数を一緒に表示してみましょう。task冒頭で`let start = Instant::now();`、表示行を`info!("カウンタ = {}（起動から{}秒）", count, start.elapsed().as_secs());`に変えるだけです。

## 確認問題

1. 「10msかかる処理を500ms周期で繰り返す」とき、`Timer::after(500ms)`をループに書くと実際の周期は約何msになりますか。
2. `Ticker`が周期を保てる仕組みを一言で説明してください。
3. `embassy_time::Delay`はどんな場面で使いますか。

<details>
<summary>答え</summary>

1. 約510ms（処理10ms＋待ち500ms）。周回ごとにずれが積もります。
2. 前回の起床時刻を基準に次の起床時刻を決める（処理にかかった時間を差し引く）からです。
3. embedded-hal / embedded-hal-asyncの遅延トレイトを要求する外部ドライバに、「待ち係」として渡す場面です。

</details>

## まとめ

- 一回待つなら`Timer::after`系、ずれない周期なら`Ticker`、計測なら`Instant`＋`Duration`
- `Timer::after`の繰り返しは処理時間の分だけ周期がずれる。周期物はTicker
- `Delay`はembedded-hal(-async)互換ドライバへ渡す待ち係。ブロッキングのesp-hal Delayとは別物

## 次のページ

「ボタンが押されるか、5秒経つか、早い方」——2つの待ちを競争させるselectを学びます。負けた側がどうなるかが重要ポイントです。

[7. select — 早い者勝ち](/embassy-esp32-c6/part09/07-select/)

前のページ: [5. Spawner](/embassy-esp32-c6/part09/05-spawner/)
