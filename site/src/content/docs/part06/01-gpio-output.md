---
title: "1. GPIO出力"
description: esp-halのOutput型でGPIOピンをHigh/Lowに駆動し、LEDを制御します。set_high/set_low/toggleと電気的な限界も学びます。
part: 6
lesson: 1
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part01/10-blinky
  - part05/10-embedded-hal
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
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
  - https://documentation.espressif.com/esp32-c6_datasheet_en.pdf
  - https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html
---

## このページでできるようになること

- GPIOという言葉の意味と、出力モードで何が起きるかを説明できる
- `Output`型でピンを構え、`set_high`/`set_low`/`toggle`で駆動できる
- 1本のピンに流してよい電流の目安と、やってはいけない接続が分かる
- 「ピンの所有権」がバグをどう防ぐかを説明できる

## 先に結論

GPIO（General Purpose Input/Output、汎用入出力）は、プログラムから自由に使える多目的ピンです。出力モードにしたピンは、`set_high()`で3.3V、`set_low()`で0Vになります。esp-halでは`Output::new(ピン, 初期状態, 設定)`で出力ピンを作り、この`Output`型の値がピンの**所有権**を持ちます。同じピンを二重に使うコードはコンパイルの時点で弾かれます。ピン1本が流せる電流は数十mA程度なので、LEDには抵抗を入れ、モーターのような大電流の部品は直結しません。

## 身近なたとえ

出力モードのGPIOピンは「プログラムで操作できる小さなスイッチ」です。スイッチを入れる（High）とピンから3.3Vが出て、切る（Low）と0Vになります。第1部のLチカは、このスイッチをカチカチし続けるプログラムでした。

ただし本物のスイッチと違い、GPIOが切り替えているのは「電圧」であって、大きな電流を流す力はありません。照明やモーターを直接つなぐ壁のスイッチとは体力がまったく違う、という点が実際との違いです。

## 仕組み

GPIOは「General Purpose（汎用）」の名前どおり、入力にも出力にも設定できます。どちらで使うかはプログラムが決めます。

```mermaid
graph LR
  A[プログラム<br>set_high / set_low] --> B[GPIO出力レジスタ]
  B --> C[出力ドライバ回路]
  C --> D[物理ピン<br>3.3V or 0V]
  D --> E[LED・他の回路]
```

`set_high()`を呼ぶと、チップ内の**出力レジスタ**（第5部で学んだメモリ上の特別な番地）に1が書かれ、出力ドライバ回路がピンを3.3Vへ引き上げます。`set_low()`なら0Vへ引き下げます。プログラムから見れば1行ですが、裏ではレジスタへの書き込みが起きています。esp-halがこのレジスタ操作を安全な形に包んでくれているのは、[第5部 9. HAL](/embassy-esp32-c6/part05/09-hal/)で学んだとおりです。

電気的な限界も知っておきましょう（データシート典型値）。

| 項目 | 値 | 意味 |
|---|---|---|
| High時の電圧 | 約3.3V | 電源電圧と同じ |
| ソース電流（流し出し） | 典型40mA | ピンから外へ流せる電流の目安 |
| シンク電流（引き込み） | 典型28mA | 外からピンへ引き込める電流の目安 |
| 全ピン合計 | 最大1000mA | 累積の絶対最大定格 |

LED1個（抵抗込みで数mA〜10mA程度）なら問題ありませんが、モーターやたくさんのLEDを1本のピンで直接駆動してはいけません。その場合はトランジスタなどのスイッチ部品を間に入れます。

## RustとEmbassyではどう書くか

出力ピンを作って駆動する部分だけを見ます。これは抜粋です。完全なコードは `examples/01-blinky` を見てください。

```rust
use esp_hal::gpio::{Level, Output, OutputConfig};

// GPIO10を出力に設定。最初は消灯（Low）
let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

loop {
    led.set_high(); // 点灯
    Timer::after(Duration::from_millis(500)).await;
    led.set_low(); // 消灯
    Timer::after(Duration::from_millis(500)).await;
}
```

High/Lowを反転させたいだけなら`toggle()`も使えます（`examples/06-embassy-tasks`より抜粋）。

```rust
led.toggle(); // HighならLowへ、LowならHighへ
```

## コードを一行ずつ読む

```rust
let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
```

- `peripherals.GPIO10` — 初期化で受け取った`peripherals`から、GPIO10の所有権を**ムーブ**して渡します。もう一度`peripherals.GPIO10`を使うコードを書くと、所有権の規則（[第3部 8. 所有権](/embassy-esp32-c6/part03/08-ownership/)）によりコンパイルエラーになります。「同じピンを2箇所から操作してしまう」バグが、実行前に消えるのです
- `Level::Low` — 出力ピンになった瞬間の初期状態です。LEDなら「最初は消灯」を明示できます
- `OutputConfig::default()` — 駆動の細かい設定です。今は既定値のままで構いません
- `mut` — `set_high()`などはピンの状態を変更するメソッドなので、変数を変更可能にしておく必要があります

```rust
led.set_high();
led.set_low();
led.toggle();
```

この3つが出力の基本操作です。全部、裏ではレジスタ1回の書き込みで、非常に高速です。

## 配線

第1部のLチカと同じ配線です。

```text
GPIO10 ──[330Ω]──▶|── GND
                 LED
        （▶| はLED。長い足が抵抗側）
```

- 抵抗330Ωを必ず入れます。電流を安全な大きさに制限するためです
- LEDのアノード（長い足、＋）が抵抗側、カソード（短い足、−）がGND側です
- 配線はUSBケーブルを抜いた状態で行います

## 実行方法

`examples/01-blinky`のプロジェクトで実行します。

```bash
cargo run --release
```

LEDが0.5秒ごとに点滅すれば成功です。

## よくある失敗

- **`cannot borrow ... as mutable`エラー**: `let led = ...`と書いて`mut`を忘れています。`set_high()`は状態を変更するので`let mut led = ...`が必要です
- **`use of moved value: peripherals.GPIO10`エラー**: 同じピンで`Output::new`を2回呼んでいます。所有権が最初の`Output`へムーブ済みだからです。ピンは1つの変数に持たせ、必要な場所へ渡して使い回します
- **モーターや多数のLEDをピンに直結して動かない・ボードが不安定になる**: ピン1本の電流は典型40mAまでです。大電流の部品はトランジスタ等を介して駆動します
- **GPIO8に変えたら点滅しない**: GPIO8はWS2812B（信号制御式LED）専用です。単純なHigh/Lowでは光りません

## やってみよう

`set_high()`と`set_low()`の2行を`led.toggle();`1行に書き換えて、`Timer::after`も1つに減らしてみましょう。同じ点滅がより短いコードで書けます。動いたら、点滅間隔を100msにして違いを見てください。

## 確認問題

1. GPIOの「General Purpose（汎用）」とは、何が汎用なのですか。
2. `Output::new`に同じピンを2回渡すとコンパイルエラーになります。この仕組みは何というRustの規則によるもので、どんなバグを防ぎますか。
3. GPIOピンにモーターを直結してはいけないのはなぜですか。

<details>
<summary>答え</summary>

1. 用途が固定されておらず、入力にも出力にも、プログラムの目的に合わせて自由に設定して使えるという意味です。
2. 所有権の規則です。ピンの所有権は最初の`Output`にムーブされるため、二重利用がコンパイル時に検出されます。複数の場所から同じピンを操作して状態が食い違うバグを防ぎます。
3. GPIOピンが流せる電流は典型40mA程度で、モーターが必要とする電流よりずっと小さいからです。無理に流すとチップを傷める恐れがあり、トランジスタなどを間に入れて駆動します。

</details>

## まとめ

- GPIO出力は`Output::new(ピン, 初期Level, OutputConfig)`で作り、`set_high`/`set_low`/`toggle`で駆動する
- `Output`がピンの所有権を持つので、二重利用はコンパイルエラーになる
- ピン1本の電流は数十mAが限度。LEDには抵抗、大電流部品にはスイッチ部品を使う

## 次のページ

出力の次は入力です。ピンの電圧を「読む」側に回ると、ボタンやセンサの状態をプログラムに取り込めるようになります。

- 前: [第5部 10. embedded-hal](/embassy-esp32-c6/part05/10-embedded-hal/)
- 次: [2. GPIO入力](/embassy-esp32-c6/part06/02-gpio-input/)
