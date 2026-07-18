---
title: "8. Tickerで周期実行"
description: Timer::afterの繰り返しで周期がずれる理由と、embassy-timeのTickerによる「ずれない周期処理」を学びます。
part: 6
lesson: 8
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part06/07-timer
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

- `Timer::after`の繰り返しで周期がずれていく理由を説明できる
- `Ticker::every`と`.next().await`でずれない周期処理を書ける
- TimerとTickerの使い分けを判断できる

## 先に結論

「1秒ごとに測定する」のような周期処理を`Timer::after`の繰り返しで作ると、**処理にかかった時間のぶんだけ周期が延びて**、少しずつずれていきます。`Ticker::every(Duration)`は起床時刻を「開始時刻 + 周期 × 回数」で決めるため、処理時間に関係なく周期を保てます。使い方は、ループの外で`Ticker::every`で作り、ループ内で`.next().await`を呼ぶだけです。「一度だけ待つ」ならTimer、「一定周期で繰り返す」ならTicker、と使い分けます。

## 身近なたとえ

薬を「8時間ごと」に飲むとします。`Timer::after`方式は「飲み**終わってから**8時間タイマーをかける」やり方です。飲むのに手間取って10分かかれば、次は8時間10分後になり、毎回少しずつ後ろへずれます。`Ticker`方式は「朝8時・16時・24時」とあらかじめ時刻表を決めておくやり方です。飲むのに何分かかろうと、次の時刻は動きません。

実際のTickerが持っているのは時刻表そのものではなく「開始時刻と周期」だけで、次の起床時刻を計算で出しています。結果として時刻表と同じ効果になります。

## 仕組み

処理に50msかかるループを、周期500msのつもりで書いた場合を比べます。

```text
Timer::after方式（ずれる）
処理50ms→待ち500ms→処理50ms→待ち500ms→…
実際の周期 = 550ms。10回繰り返すと500msの遅れ

Ticker方式（ずれない）
起床時刻: 0ms, 500ms, 1000ms, 1500ms, …（最初に確定）
処理50ms→次の起床時刻まで450ms待つ→…
実際の周期 = 500ms
```

`Timer::after`は「今から500ms」なので、処理時間が毎回足し込まれます。`Ticker`は「次の予定時刻まで」なので、処理時間は待ち時間の中に吸収されます。

ただしTickerにも限界はあります。処理が周期より長くかかると、その回の予定時刻はもう過ぎているため、周期どおりの実行はできません。「周期内に終わる処理」で使うのが前提です。

## RustとEmbassyではどう書くか

`examples/06-embassy-tasks`のLED点滅taskがそのまま実例です。これは抜粋です。完全なコードは `examples/06-embassy-tasks` を見てください。

```rust
use embassy_time::{Duration, Ticker};

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

同じ例の`counter_task`は1秒周期のTickerでカウンタを刻んでいます。task自体の書き方（`#[embassy_executor::task]`や`spawner.spawn`）は第9部でじっくり扱うので、ここではTickerの使い方に注目してください。

## コードを一行ずつ読む

```rust
let mut ticker = Ticker::every(Duration::from_millis(500));
```

「500ms周期のtickerを、今を起点に開始する」という意味です。**ループの外で1回だけ**作るのがポイントです。ループの中で毎回作り直すと起点がリセットされ、`Timer::after`と同じずれ方をしてしまいます。`next()`が内部の状態（次の起床時刻）を進めるので`mut`が必要です。

```rust
loop {
    led.toggle();
    ticker.next().await;
}
```

`next().await`が「次の予定時刻まで眠る」です。`led.toggle()`にかかった時間は、次の待ち時間から自動的に差し引かれます。

## 実行方法

```bash
cargo run --release
```

`examples/06-embassy-tasks`では、LEDが500ms周期で点滅し続け、1秒ごとのカウンタログと5秒ごとのハートビートが同時に出ます。カウンタのログの間隔が長時間動かしても1秒を保つことを確認してください。

## よくある失敗

- **ループの中で`Ticker::every`を呼んでしまう**: 毎回「今から500ms」で作り直すことになり、Tickerの意味がなくなります。Tickerはループの外で1回だけ作ります
- **処理が周期より長くて周期を守れない**: 500ms周期のループ内に1秒かかる処理を書くと、どう待っても間に合いません。周期を延ばすか、処理を軽くする（または別taskへ分ける）必要があります
- **`mut`を忘れる**: `ticker.next()`はTicker内部の「次の起床時刻」を更新するため、`let mut ticker`が必要です
- **1回だけの待ちにTickerを使う**: 動きはしますが大げさです。単発の待ちは`Timer::after`、繰り返しはTickerが適材適所です

## やってみよう

`blink_task`のTickerを`Timer::after(Duration::from_millis(500)).await`に置き換え、`toggle()`の直後に`Timer::after(Duration::from_millis(100)).await`（処理時間が長いことの代わり）を入れて動かしてみましょう。点滅周期が600msに延びるのが分かります。Tickerに戻して同じ100msの待ちを入れても、周期は500msのまま保たれます。

## 確認問題

1. `Timer::after`の繰り返しによる周期処理がずれていくのはなぜですか。
2. Tickerが周期を保てるのは、次の起床時刻をどのように決めているからですか。
3. 周期500msのTickerを使うループの中に、700msかかる処理を書いたらどうなりますか。

<details>
<summary>答え</summary>

1. `Timer::after`は「処理が終わった時点から」指定時間を数えるため、毎回の処理時間が周期に足し込まれていくからです。
2. 「開始時刻 + 周期 × 回数」という予定時刻を基準に待つからです。処理にかかった時間は次の待ち時間から差し引かれます。
3. 予定時刻を過ぎてしまうため、500ms周期は守れません。Tickerは魔法ではなく、周期内に処理が終わることが前提です。周期を見直すか処理を分割します。

</details>

## まとめ

- 周期処理は`Ticker::every(Duration)` + `.next().await`。処理時間が周期に食い込まない
- Tickerはループの外で1回だけ作る。単発の待ちは`Timer::after`と使い分ける
- 処理が周期より長い場合はTickerでも守れない。設計の見直しが必要

## 次のページ

「待つ」の最後のピースは「待ちすぎを防ぐ」です。いつ来るか分からないイベントに制限時間を付ける`with_timeout`を学びます。

- 前: [7. Timerで待つ](/embassy-esp32-c6/part06/07-timer/)
- 次: [9. Timeout](/embassy-esp32-c6/part06/09-timeout/)
