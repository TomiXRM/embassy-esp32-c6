---
title: "1. ADCで電圧を読む"
description: ESP32-C6のADC1で電圧を数値として読み取ります。12bit・4096段階の意味、使えるピン（GPIO0〜6）、減衰と校正を学びます。
part: 7
lesson: 1
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part06/01-gpio-output
  - part06/07-timer
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル（データ通信対応）
  - ブレッドボード
  - 可変抵抗 10kΩ
  - ジャンパ線 3本
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
  - https://documentation.espressif.com/esp32-c6_datasheet_en.pdf
---

## このページでできるようになること

- アナログとデジタルの違いを、電圧の言葉で説明できる
- ESP32-C6のADCの仕様（ADC1のみ・12bit・GPIO0〜6）を正しく述べられる
- 減衰（Attenuation）と校正がなぜ必要かを説明できる
- `read_oneshot`でADCの値を読み、シリアルに表示できる

## 先に結論

ADC（Analog to Digital Converter、アナログ-デジタル変換器）は、ピンの電圧を数値に変える回路です。ESP32-C6のADCは**12bit**なので、0V〜約3.3Vを**0〜4095の4096段階**の整数で表します。使えるのは**ADC1の7チャンネル（GPIO0〜GPIO6）だけ**で、ADC2はESP32-C6には存在しません。esp-halでは`AdcConfig`でピンを有効化し、`Adc::new(...).into_async()`で非同期版を作り、`read_oneshot(...).await`で1回ずつ読みます。

## 身近なたとえ

これまでのGPIO入力は「電気がある/ない」の2択しか読めない、ONとOFFだけのスイッチ確認でした。ADCは体温計のようなものです。「熱がある/ない」ではなく「36.8度」という細かい数値で読み取れます。

ただし体温計と違い、ADCが読むのは温度そのものではなく**電圧**です。温度や明るさを知りたいときは、まずセンサがそれを電圧に変え、その電圧をADCが数値に変える、という2段階になります。この点が実際との違いです。

## 仕組み

### アナログとデジタル

- **デジタル入力**: 電圧をHigh（1）かLow（0）の2値に丸めて読む
- **アナログ入力（ADC）**: 電圧の大きさをそのまま数値にする

ESP32-C6のADCは12bitです。12bitとは、結果を2進数12桁で表すという意味で、2の12乗 = **4096段階**（0〜4095）になります。測定範囲を約3.3Vとすると、1段階は約0.8mVです。

```text
0V ──────────────── 約3.3V
0                    4095
      12bit = 4096段階
```

### 使えるピンはGPIO0〜6だけ

ESP32-C6が持つのは**SAR ADC1のみ**です（SARは逐次比較型という変換方式の名前です）。チャンネルとピンの対応は固定です。

| チャンネル | ピン |
|---|---|
| ADC1_CH0〜CH6 | GPIO0〜GPIO6（番号がそのまま対応） |

無印ESP32にあったADC2は、**ESP32-C6には存在しません**。ネット記事でADC2を見かけても、それは別のチップの話です。また変換速度は最大100kSPS（1秒間に10万回）で、音声のような速い信号の録音には向きません。この教材では可変抵抗をGPIO2（ADC1_CH2）につなぎます。

### 減衰（Attenuation）

ADCの内部回路がそのまま測れる電圧範囲は狭く、3.3Vまで届きません。そこで入力を**減衰器**（信号を一定の割合で弱める回路）に通してから測ります。`Attenuation::_11dB`を指定すると約11dB（約1/3.5）に弱めてから測るため、**0V〜約3.3Vの全域**を扱えます。ただし範囲の上端付近は誤差が大きくなるので、精密さが必要な用途では範囲の真ん中あたりを使うのが安全です。

### 校正（キャリブレーション）

ADCには個体差があり、同じ電圧でもチップごとに読み値が少しずれます。esp-halの`AdcCalBasic`は、製造時にチップ内のeFuse（書き換えできない記録領域）へ保存された補正値を使って、このずれを直します。

## RustとEmbassyではどう書くか

ADCの設定と読み取りだけを見ます。これは抜粋です。完全なコードは `examples/13-adc-pwm` を見てください。

```rust
use esp_hal::analog::adc::{Adc, AdcCalBasic, AdcConfig, Attenuation};
use esp_hal::peripherals::ADC1;

type AdcCal = AdcCalBasic<ADC1<'static>>;

// GPIO2をADC1の入力として有効化。減衰11dBで0V〜約3.3Vを測れる
let mut adc1_config = AdcConfig::new();
let mut pot_pin =
    adc1_config.enable_pin_with_cal::<_, AdcCal>(peripherals.GPIO2, Attenuation::_11dB);
let mut adc1 = Adc::new(peripherals.ADC1, adc1_config).into_async();

loop {
    // ADCを1回読む（12bitなので0〜4095）
    let raw: u16 = adc1.read_oneshot(&mut pot_pin).await;
    info!("ADC生値 = {raw:4}");
    Timer::after(Duration::from_millis(500)).await;
}
```

## コードを一行ずつ読む

```rust
let mut pot_pin =
    adc1_config.enable_pin_with_cal::<_, AdcCal>(peripherals.GPIO2, Attenuation::_11dB);
```

- `enable_pin_with_cal` — GPIO2をADC入力として登録し、校正方式`AdcCal`（= `AdcCalBasic`）を適用します。ピンの所有権はここでムーブされ、返ってきた`pot_pin`が読み取りの窓口になります
- `Attenuation::_11dB` — 減衰量の指定です。11dBで約3.3Vまで測れます

```rust
let mut adc1 = Adc::new(peripherals.ADC1, adc1_config).into_async();
```

- `Adc::new` — ADC1本体と設定からドライバを作ります。`into_async()`で非同期版に変換すると、変換の完了待ちの間に他のtaskへ実行を譲れます

```rust
let raw: u16 = adc1.read_oneshot(&mut pot_pin).await;
```

- `read_oneshot` — 「ワンショット」つまり1回だけ変換して結果を返します。戻り値は`u16`で、範囲は0〜4095です

なおADCはesp-halの`unstable` feature配下のAPIです。esp-halのバージョンを上げるとAPIが変わる可能性があります（この教材はesp-hal 1.1系で固定しています）。

## 配線

可変抵抗（ポテンショメータ）を使います。仕組みは次ページで説明するので、ここでは配線だけ済ませましょう。

```text
可変抵抗（3端子）
  端 A ── 3V3
  中央（ワイパー）── GPIO2
  端 B ── GND
```

- 配線はUSBケーブルを抜いた状態で行います
- GPIOに入れてよい電圧は最大3.6Vです。可変抵抗の端を5Vピンにつなぐと、中央端子から3.3Vを超える電圧が出てチップを傷めます。**必ず3V3ピン**を使ってください

## 実行方法

`examples/13-adc-pwm`のプロジェクトで実行します。

```bash
cargo run --release
```

つまみを回すと、シリアルに出る`ADC生値`が0付近から4095付近まで変わります。

```text
INFO - ADC生値 = 2051, PWMデューティ =  50%
```

## よくある失敗

- **GPIO8やGPIO10を指定してコンパイルエラーになる**: ADCに使えるのはGPIO0〜6だけです。ピンとチャンネルの対応はチップ内部で固定されていて、他のピンには物理的にADC回路がつながっていません
- **値が0か4095に張り付く**: 可変抵抗の中央端子（ワイパー）ではなく端の端子をGPIO2につないでいることが多いです。3端子の真ん中を確認してください
- **何もつながないピンを読むと値がふらふら動く**: 入力がどこにもつながっていない「浮いた」状態では電圧が定まりません。GPIO入力のときと同じ現象です（[第6部 3. Pull-upとPull-down](/embassy-esp32-c6/part06/03-pull-updown/)）
- **値が理論どおりぴったりにならない**: ADCには誤差とノイズがあります。校正で大きなずれは補正されますが、数〜数十の揺れは正常です。整え方は[3. センサ値を整える](/embassy-esp32-c6/part07/03-sensor-reading/)で扱います

## やってみよう

読み取り間隔を`from_millis(500)`から`from_millis(100)`に縮めて、つまみを速く回したときの値の追従を見てみましょう。表示が速すぎて読みにくくなることも体感できます。

## 確認問題

1. 12bitのADCは電圧を何段階で表しますか。
2. ESP32-C6でADCに使えるピンはどれですか。またADC2はありますか。
3. `Attenuation::_11dB`を指定する目的は何ですか。

<details>
<summary>答え</summary>

1. 2の12乗 = 4096段階（0〜4095）です。
2. GPIO0〜GPIO6の7本（ADC1_CH0〜CH6）だけです。ADC2はESP32-C6には存在しません。
3. 入力を約11dB弱めてから測ることで、ADCがそのままでは測れない約3.3Vまでの電圧範囲を扱えるようにするためです。

</details>

## まとめ

- ADCは電圧を数値に変える。ESP32-C6は12bit（0〜4095）、ADC1のみ、GPIO0〜6のみ
- `AdcConfig` → `enable_pin_with_cal` → `Adc::new(...).into_async()` → `read_oneshot(...).await`の流れで読む
- 減衰11dBで約3.3Vまで測れる。校正はeFuseの補正値で個体差を直す

## 次のページ

つないだ可変抵抗は、なぜつまみの位置が電圧に変わるのでしょうか。その答えである「分圧」は、ADCを使う回路すべての基礎になります。

- 前: [第6部 10. Watchdog](/embassy-esp32-c6/part06/10-watchdog/)
- 次: [2. 分圧の考え方](/embassy-esp32-c6/part07/02-voltage-divider/)
