---
title: "3. Wake-upの設計"
description: タイマー起床とGPIO起床の使い分けを学びます。Deep-sleepのGPIO起床はLP GPIO（GPIO0〜7）限定という制限も実コードで確かめます。
part: 12
lesson: 3
difficulty: advanced
estimated_minutes: 20
prerequisites:
  - part12/02-deep-sleep
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - LED、抵抗330Ω
  - タクトスイッチ、抵抗10kΩ
  - ブレッドボード、ジャンパ線
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/system/sleep_modes.html
  - https://documentation.espressif.com/esp32-c6_datasheet_en.pdf
  - https://github.com/esp-rs/esp-hal
---

## このページでできるようになること

- タイマー起床とGPIO起床を用途で使い分けられる
- Deep-sleepのGPIO起床がLP GPIO（GPIO0〜7）に限られる理由と対処が分かる
- 復帰要因（`wakeup_cause`）で処理を分岐する設計を説明できる

## 先に結論

起床（wake-up）設計の基本は「定期的に起きる（タイマー起床）」と「事件が起きたら起きる（GPIO起床）」の組み合わせです。ESP32-C6のDeep-sleepでGPIO起床に使えるのは、**LP（低消費電力）電源ドメインにあるGPIO0〜GPIO7だけ**です。BOOTボタン（GPIO9）では起こせません。esp-halではDeep-sleepのGPIO起床を`Ext1WakeupSource`で指定します。**似た名前の`GpioWakeupSource`はLight-sleep専用**で、Deep-sleepからの復帰には使えません。複数の復帰要因は同時に登録でき、復帰後に`wakeup_cause()`でどれが原因だったかを区別できます。

## 身近なたとえ

目覚まし時計（タイマー）と玄関チャイム（GPIO）です。両方セットして寝れば、朝が来ても起きられるし、夜中に宅配が来ても起きられます。起きたあと「時計で起きたのか、チャイムで起きたのか」によって、最初にやること（着替えるのか、玄関へ走るのか）を変えるわけです。

実際のマイコンがたとえと違うのは、チャイムの線（GPIO）を家中どこにでも引けるわけではない点です。Deep-sleep中は建物の大部分が停電しており、「寝ている間も電気が来ている棟」（LPドメイン）につながるGPIO0〜7だけがチャイムとして機能します。

## 仕組み

### 2種類の起床の使い分け

| 起床方法 | 向いている用途 | 特徴 |
|---|---|---|
| タイマー起床 | 定期測定、定期送信（例: 1時間ごとに温度を送る） | 消費電力を周期から見積もりやすい。イベントには即応できない |
| GPIO起床 | ボタン、センサのアラート出力（例: 押されたら送信） | 事件が起きるまでほぼ眠りっぱなしにできる。定期処理はできない |

実用機器では「基本はGPIO起床で待ち、保険としてタイマー起床も登録して定期的に生存報告する」といった組み合わせがよく使われます。examples/12-sleepもタイマー+GPIOの2本立てです。

### ESP32-C6で使える復帰要因

公式資料（ESP-IDF Sleep Modesドキュメント、データシート）による整理です。

| 復帰要因 | Light-sleep | Deep-sleep |
|---|---|---|
| RTCタイマー | ○ | ○ |
| GPIOレベル（任意のGPIO） | ○（`GpioWakeupSource`） | ×（**使えない**） |
| EXT1（LP GPIO = GPIO0〜7） | ○ | ○（`Ext1WakeupSource`） |
| UART受信 | ○ | × |
| LPコア / LP UART | ○ | ○ |

### なぜGPIO0〜7だけなのか

Deep-sleep中はHP（高性能）ドメインの電源が切られています。ピンの状態を見張る回路が生き残っているのは、LPドメインに属するGPIO0〜7だけです（ESP32-C6データシート v1.5）。だから**GPIO9のBOOTボタンはDeep-sleepの起床には使えません**。examples/12-sleepが、ボード上のBOOTボタンではなくGPIO7に外付けボタンをつなぐのはこのためです。

### 名前の似た2つのAPIに注意

esp-halの`rtc_cntl::sleep`（unstable API）には、GPIO系の復帰要因が2つあります。

- `Ext1WakeupSource` — LP GPIO（0〜7）を使う。**Deep-sleepからの復帰に使えるのはこちら**
- `GpioWakeupSource` — **Light-sleep専用**。Deep-sleepに渡しても起こしてもらえない

名前だけ見ると逆に覚えそうになるので注意してください。この教材のexamplesで検証したのは`Ext1WakeupSource`によるDeep-sleep復帰です。

## RustとEmbassyではどう書くか

examples/12-sleepから、復帰要因を組み立てる部分を抜粋します（これは抜粋です。完全なコードはexamples/12-sleepを見てください）。

```rust
    // 1. RTCタイマー: 10秒経ったら復帰する
    let timer_wakeup = TimerWakeupSource::new(CoreDuration::from_secs(10));

    // 2. EXT1（LP GPIO）: GPIO7がLowレベルになったら復帰する。
    //    ESP32-C6ではピンごとに復帰レベル(High/Low)を指定できる
    let mut wake_pin = peripherals.GPIO7;
    let mut wakeup_pins: [(&mut dyn RtcPinWithResistors, WakeupLevel); 1] =
        [(&mut wake_pin, WakeupLevel::Low)];
    let ext1_wakeup = Ext1WakeupSource::new(&mut wakeup_pins);

    let mut rtc = Rtc::new(peripherals.LPWR);
    rtc.sleep_deep(&[&timer_wakeup, &ext1_wakeup]);
```

そして起動直後のこの2行が、起床設計の「答え合わせ」になります。

```rust
    info!("リセット要因: {:?}", reset_reason(Cpu::ProCpu));
    info!("復帰要因: {:?}", wakeup_cause());
```

## コードを一行ずつ読む

- `[(&mut dyn RtcPinWithResistors, WakeupLevel); 1]` — 「LPドメインのピンとして扱えるもの」だけを受け付ける配列です。`RtcPinWithResistors`はLP対応ピンが実装しているtraitで、GPIO0〜7以外のピンはここに入れられません。データシートの制限を型が守ってくれる、Rustらしい仕組みです
- `WakeupLevel::Low` — 「Lowになったら起こして」という指定です。ボタンでGNDへ落とす配線なので、待機中はプルアップでHighに保ちます。ピンごとにHigh/Lowを選べます
- `Ext1WakeupSource::new(&mut wakeup_pins)` — 配列を渡すので、複数ピンをまとめて登録することもできます
- `sleep_deep(&[&timer_wakeup, &ext1_wakeup])` — 復帰要因はスライスで複数登録でき、**先に条件が成立したもの勝ち**です
- `wakeup_cause()` — 12-sleepではログ表示に留めていますが、実用ではこの値で処理を分けます。たとえば「タイマー起床なら定期報告だけしてすぐ眠る、ボタン起床ならイベントを送信する」という分岐が、省電力機器の典型的な骨格です

## 配線

[前のページ](/embassy-esp32-c6/part12/02-deep-sleep/)と同じです。

- GPIO10 → 抵抗330Ω → LEDアノード(+) → LEDカソード(−) → GND
- GPIO7 → 抵抗10kΩ → 3V3（プルアップ。スリープ中の誤復帰防止に必須）
- GPIO7 → タクトスイッチ → GND

## 実行方法

```bash
cd examples
cargo run --release -p sleep
```

2つのシナリオを試して、ログの「復帰要因」を見比べてください。

1. **何もせず10秒待つ** → タイマー起床（Timerを示す値）
2. **スリープ中にGPIO7のボタンを押す** → EXT1起床（Ext1を示す値）

どちらもプログラムは先頭から再実行されますが、「なぜ起きたか」は区別できます。

## よくある失敗

1. **BOOTボタン（GPIO9）でDeep-sleepから起こそうとする** — GPIO9はLPドメイン外なのでEXT1に使えません。GPIO0〜7に外付けボタンをつなぎます（GPIO0/1はADCや32kHz水晶、GPIO4/5はストラッピングと兼用なので、空いていればGPIO6/7あたりが素直です）
2. **`GpioWakeupSource`をDeep-sleepに使ってしまう** — Light-sleep専用です。Deep-sleepでは起こしてもらえず、「タイマーでしか起きない」不思議な現象に見えます
3. **`WakeupLevel::Low`なのにプルアップしていない** — 浮いたピンはノイズで揺れるため、眠った直後に起きる・押していないのに起きる、といった誤動作になります

## やってみよう

配線をGPIO7からGPIO6に付け替え、コードの`peripherals.GPIO7`を`peripherals.GPIO6`に変えて動くことを確認しましょう。余裕があれば`peripherals.GPIO9`に変えてビルドし、LPドメイン外のピンを渡したときにコンパイラがどんなエラーで止めてくれるかも観察してみてください。

## 確認問題

1. Deep-sleepのGPIO起床に使えるピンはどれですか。また、それはなぜですか。
2. `Ext1WakeupSource`と`GpioWakeupSource`の違いを説明してください。
3. 「1時間ごとに温度を送りつつ、ボタンが押されたら即座に通知する」機器の復帰要因はどう設計しますか。

<details>
<summary>答え</summary>

1. GPIO0〜GPIO7です。Deep-sleep中はHPドメインの電源が切れており、ピンを見張る回路が生きているのはLPドメインのピンだけだからです。
2. `Ext1WakeupSource`はLP GPIO（0〜7）を使う復帰要因で、Deep-sleepからの復帰に使えます。`GpioWakeupSource`はLight-sleep専用で、Deep-sleepには使えません。
3. RTCタイマー（1時間）とEXT1（ボタンのピン、押下レベル）を両方登録して眠り、起床後に`wakeup_cause()`で分岐します。タイマー起床なら温度測定と送信、EXT1起床なら通知イベントの送信を行い、それぞれ終わったらまた両方登録して眠ります。

</details>

## まとめ

- 起床設計は「タイマー（定期）」と「GPIO（事件）」の組み合わせ。複数登録して早い者勝ち、原因は`wakeup_cause()`で区別
- Deep-sleepのGPIO起床はLP GPIO（GPIO0〜7）限定。BOOTボタン（GPIO9）は使えない
- Deep-sleepには`Ext1WakeupSource`。`GpioWakeupSource`はLight-sleep専用

## 次のページ

設計した省電力が本当に効いているかは、測らなければ分かりません。ただしµAからmAまで桁が5つも違う電流の測定には、それなりの流儀があります。

[4. 消費電力の測り方 →](/embassy-esp32-c6/part12/04-power-measurement/)

---

前: [2. Deep Sleep](/embassy-esp32-c6/part12/02-deep-sleep/) | 次: [4. 消費電力の測り方](/embassy-esp32-c6/part12/04-power-measurement/)
