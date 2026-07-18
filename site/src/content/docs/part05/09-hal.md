---
title: "9. HAL — esp-hal"
description: esp-halが何を抽象化するのか、所有権によるペリフェラル管理、stable/unstable featureの区別を学びます。
part: 5
lesson: 9
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part05/08-pac
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
  - https://github.com/esp-rs/esp-hal
---

## このページでできるようになること

- HAL（Hardware Abstraction Layer）が何を抽象化するかを説明できる
- esp-halの `Peripherals` が所有権でペリフェラルを管理する仕組みを説明できる
- esp-halのstable APIとunstable featureの違いを説明し、Cargo.tomlの `unstable` の意味が分かる

## 先に結論

esp-halはESP32シリーズ用のHAL（Hardware Abstraction Layer、ハードウェア抽象化層）です。前ページのPACが「レジスタの正確な辞書」だとすれば、esp-halは「正しい操作手順だけが書けるようにした道具箱」です。レジスタのビット操作を `Output::new` や `set_high` のような意味の分かる操作に変換し、所有権によって「同じペリフェラルの二重使用」をコンパイルエラーにします。esp-halはバージョン1.1系で、GPIO・UART・I2C・SPIなどの中核はstable（安定）ですが、ADC・PWM・タイマーなどはまだ `unstable` feature配下にあり、APIが変わる可能性があります。本教材は両方を使うため、Cargo.tomlで `unstable` を有効にしています。

## 身近なたとえ

esp-halは自動車の運転席のようなものです。エンジン内部（レジスタ）では何千という部品が動いていますが、運転者にはハンドル・アクセル・ブレーキという少数の分かりやすい操作だけが見えています。しかも「ブレーキとアクセルを同時に踏み込む」ような矛盾した操作は、そもそもできないよう設計されています。

ただし実際のesp-halが優れているのは、誤操作の防止を**実行時ではなくコンパイル時**にやる点です。危険な組み合わせは走る前、ビルドの段階で弾かれます。

## 仕組み

### esp-halが抽象化する3つのこと

**1. レジスタ操作 → 意味のある操作**

「GPIO10の出力レジスタのビット10に1を書く」が「`led.set_high()`」になります。データシートを暗記しなくても、やりたいことがそのままコードになります。

**2. チップごとの違い → 共通の書き方**

esp-halはESP32・C3・C6など複数チップに対応し、Cargo.tomlのfeature（本教材では `esp32c6`）でチップを選びます。GPIOの使い方はどのチップでもほぼ同じ書き方になるため、経験が他のチップでも通用します。

**3. 「触ってよいのは誰か」→ 所有権で管理**

`esp_hal::init` が返す `Peripherals` は、チップの全ペリフェラルの所有権を1回だけ渡す「鍵束」です。`peripherals.GPIO10` を `Output::new` に渡すと、GPIO10の鍵はmoveされます。第3部で学んだとおり、moveされた値は二度と使えません。つまり**「GPIO10を出力としても入力としても使う」ような矛盾したコードは、コンパイルの時点で書けない**のです。C言語ではベテランでも踏む「二重初期化」のバグが、型システムで消えています。

### stableとunstable — 2階建てのAPI

esp-halは1.x系に到達したクレートですが、全モジュールが同じ安定度ではありません。

| 区分 | モジュール | 意味 |
|---|---|---|
| **stable** | clock, gpio, i2c, interrupt, peripherals, rng, spi, system, time, uart, efuse | APIが安定。バージョン更新で壊れない（semver保証あり） |
| **unstable featureが必要** | analog(ADC), delay, dma, ledc, mcpwm, twai, rtc_cntl(sleep), timer, usb_serial_jtag, rmt, i2s ほか | 使えるが、マイナー更新でAPIが変わりうる（semver保証なし） |

本教材はADC（第7部）、PWM（第7部）、TWAI（第8部）、sleep（第12部）、そしてEmbassyの時刻ドライバ（timer）を扱うため、`unstable` featureを有効にしています。examples/のCargo.tomlにある次の指定がそれです。

```toml
esp-hal = { version = "~1.1.0", features = ["esp32c6", "unstable", "log-04"] }
```

`~1.1.0` は「1.1系の範囲内でのみ更新する」という指定です。unstableなAPIは1.2系で変わる可能性があるため、教材全体でバージョンを固定しています。ネット上の情報を読むときも、esp-halの**バージョン**（0.2x系の古い記事が多数あります）を必ず確認してください。古い世代のAPIは現在のコードと互換性がありません。

## RustとEmbassyではどう書くか

blinkyから、esp-halの典型的な使い方を抜粋します。完全なコードは examples/01-blinky を見てください。

```rust
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};

let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
let peripherals = esp_hal::init(config);

// GPIO10を出力に設定。最初は消灯（Low）
let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

led.set_high(); // 点灯
led.set_low();  // 消灯
```

## コードを一行ずつ読む

- `esp_hal::Config::default().with_cpu_clock(CpuClock::max())` — 設定はメソッドをつなげて組み立てます（ビルダーパターン）。既定値から必要な項目だけ変える書き方で、第4部で見た「正しい組み合わせしか作れない設計」の実例です。
- `let peripherals = esp_hal::init(config);` — ここで全ペリフェラルの鍵束を受け取ります。`init` は2回呼べません。ペリフェラルの所有権の出発点がプログラム中に1か所しかないことが、二重使用防止の土台です。
- `Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default())` — GPIO10の鍵をmoveで渡し、「出力ピン」という型 `Output` に変換します。初期状態（`Level::Low`）を必ず指定させる設計なので、「初期化直後のピンの状態が不定」という組み込みの定番トラブルが起きません。
- `led.set_high();` — 以後は `Output` 型のメソッドだけが使えます。入力用のメソッド（読み取りなど）はこの型に存在しないため、呼び間違いはコンパイルエラーになります。

## 実行方法

このコードはblinkyそのものです。動かして、`set_high`/`set_low` とLEDの点滅の対応を確かめられます。

```bash
cd examples/01-blinky
cargo run --release
```

期待される結果: GPIO10につないだLEDが0.5秒ごとに点滅し、シリアルに「Lチカを開始します」と表示されます。

## よくある失敗

- **同じピンを2回使ってコンパイルエラー** — `Output::new(peripherals.GPIO10, ...)` を2回書くと `use of moved value` エラーになります。これは仕様であり、安全装置が働いた証拠です。1つのピンを複数のtaskで使いたい場合の正しい方法は第9部で学びます。
- **`unstable` featureを外してビルドが通らない** — Cargo.tomlから `unstable` を消すと、`timer` モジュールなどが見つからずエラーになります。Embassyの時刻の仕組み自体がtimer（unstable）に依存しているため、本教材の構成では外せません。
- **古い記事のesp-hal 0.2x系のコードを混ぜる** — 型名や初期化の作法が違うため、ほぼ確実にコンパイルエラーになります。エラーメッセージに `esp_hal::` の見覚えのない型が出てきたら、参照元のバージョンを疑ってください。

## やってみよう

blinkyの `Level::Low` を `Level::High` に変えて書き込み、起動直後のLEDの状態がどう変わるかを観察してください。「初期状態を型で必ず指定させる」設計のありがたみを体感できます（5分以内。終わったら戻しておきましょう）。

## 確認問題

1. PACとHALの役割の違いを一言ずつで説明してください。
2. `Output::new` に渡した `peripherals.GPIO10` を、あとでもう一度使えないのはなぜですか。それは何を防いでいますか。
3. 本教材がesp-halの `unstable` featureを有効にしている理由をひとつ挙げてください。

<details>
<summary>答え</summary>

1. PACはレジスタに名前と型を与えた正確な辞書、HALはその上で「正しい手順だけが書けるようにした道具箱」です。
2. 所有権がmoveされたからです。同じペリフェラルを2か所から操作する矛盾（二重初期化・競合）をコンパイル時に防いでいます。
3. ADC・PWM・TWAI・sleep・タイマー（Embassyの時刻ドライバが必要とする）などがunstable feature配下にあるためです。

</details>

## まとめ

- esp-halはレジスタ操作を意味のある型と操作に変換し、チップ差を吸収するHAL
- `Peripherals` の所有権管理により、ペリフェラルの二重使用はコンパイルエラーになる
- GPIO/UART/I2C/SPIなどはstable、ADC/PWM/timer等はunstable feature。教材はバージョン固定（~1.1.0）で両方を使う

## 次のページ

esp-halはESP32ファミリー専用です。では、他社のチップでも使える温度センサのドライバはどう書かれているのでしょうか。その答えが、HALの共通言語embedded-halです。

[← 前のページ: PACとレジスタとunsafe](/embassy-esp32-c6/part05/08-pac/) | [次のページ: embedded-hal →](/embassy-esp32-c6/part05/10-embedded-hal/)
