---
title: "6. SPI基礎"
description: SPIの4本の線（MOSI/MISO/SCK/CS）とモード（CPOL/CPHA）を学び、ESP32-C6のSPI2でループバック通信を動かします。
part: 8
lesson: 6
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part08/03-i2c-basics
  - part06/01-gpio-output
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル（データ通信対応）
  - ジャンパ線 1本
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
  - https://documentation.espressif.com/esp32-c6_datasheet_en.pdf
---

## このページでできるようになること

- SPIの4本の線（MOSI/MISO/SCK/CS）の役割を説明できる
- 「送信と受信が同時に起こる」全二重の意味を説明できる
- モード0〜3（CPOL/CPHA）が何を決めているか説明できる
- esp-halで`Spi`を初期化し、ループバックで転送を確かめられる

## 先に結論

SPI（Serial Peripheral Interface）は、MOSI・MISO・SCK・CSの4本の線を使う高速なシリアル通信です。I2Cのようなアドレスはなく、**CS（チップセレクト）線をLowにしたデバイスだけ**が相手になります。最大の特徴は**全二重**であること。クロック1発ごとにMOSIで1ビット送り出すと同時に、MISOから1ビット受け取ります。「送るだけ」「受けるだけ」の操作も、内部では常に双方向転送です。クロックの使い方には**モード0〜3**（CPOL/CPHAの組み合わせ)があり、デバイスのデータシートに合わせます。ESP32-C6で自由に使えるSPIはSPI2の1つです（SPI0/1はフラッシュ接続用）。

## 身近なたとえ

SPIの転送は「回転寿司のレーン」に似ています。レーンが1周する間に、自分の皿（送信データ）が相手へ届き、同時に相手の皿（受信データ）が自分へ届きます。皿を送らずに受け取ることはできず、何か送れば必ず何かが返ってきます。

ただし実際のSPIでは、レーンを回す速さ（クロック）を決めるのは常にマスタ側で、スレーブは自分のペースで送れない、という点がたとえとの違いです。

## 仕組み

4本の線の役割です。

| 線 | 正式名称 | 方向 | 役割 |
|---|---|---|---|
| SCK | Serial Clock | マスタ→スレーブ | ビットのタイミングを刻む |
| MOSI | Master Out Slave In | マスタ→スレーブ | マスタからの送信データ |
| MISO | Master In Slave Out | スレーブ→マスタ | スレーブからの返信データ |
| CS | Chip Select | マスタ→スレーブ | Lowにしたデバイスが通信相手（1台に1本） |

```mermaid
graph LR
  subgraph ESP32-C6（マスタ）
    SCK[SCK GPIO19]
    MOSI[MOSI GPIO18]
    MISO[MISO GPIO20]
    CS[CS GPIO21]
  end
  subgraph デバイス（スレーブ）
    dSCK[SCK]
    dSDI[SDI/MOSI]
    dSDO[SDO/MISO]
    dCS[CS]
  end
  SCK --> dSCK
  MOSI --> dSDI
  dSDO --> MISO
  CS --> dCS
```

- **アドレスの代わりにCS**: I2Cは「番号で呼ぶ」方式でしたが、SPIは「手を挙げさせる」方式です。複数デバイスをつなぐならCS線をデバイスの数だけ用意します
- **全二重**: クロック1発で、MOSIとMISOのビットが同時に1つずつ進みます。だからesp-halの基本操作は「送りながら受ける」`transfer`系です
- **速度**: クロック線があるので、I2C（100k〜400kHz程度）よりずっと速くできます。教材ではまず控えめな1MHzを使います

**モード（CPOL/CPHA）**はクロックの「休み方」と「読むタイミング」の約束です。

| モード | CPOL（アイドル時のSCK） | CPHA（データを取り込むエッジ） |
|---|---|---|
| モード0 | Low | 立ち上がり（最初のエッジ） |
| モード1 | Low | 立ち下がり（2番目のエッジ） |
| モード2 | High | 立ち下がり（最初のエッジ） |
| モード3 | High | 立ち上がり（2番目のエッジ） |

デバイスとマスタでモードが食い違うと、ビットが半クロックずれた「壊れたデータ」になります。使うデバイスのデータシートに従います（モード0対応のデバイスが最も多数派です）。

## RustとEmbassyではどう書くか

MOSIとMISOを直結し、送ったデータがそのまま返る**ループバック**で全二重を確かめます。これは抜粋です。完全なコードは `examples/05-spi` を見てください。

```rust
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;

// SPI2を初期化。周波数とモードは明示的に指定する
// - 周波数: 1MHz（多くのSPIデバイスが対応できる控えめな速度）
// - モード0: CPOL=0（クロックはアイドル時Low）、CPHA=0（立ち上がりエッジで取り込み）
let spi_config = SpiConfig::default()
    .with_frequency(Rate::from_mhz(1))
    .with_mode(Mode::_0);
let mut spi = Spi::new(peripherals.SPI2, spi_config)
    .expect("SPIの設定が不正です")
    .with_sck(peripherals.GPIO19)
    .with_mosi(peripherals.GPIO18)
    .with_miso(peripherals.GPIO20)
    .into_async();

// CS（チップセレクト）は自分でGPIOを操作する方式。
// 通常時はHigh（非選択）にしておき、通信の間だけLowにする
let mut cs = Output::new(peripherals.GPIO21, Level::High, OutputConfig::default());
```

転送はバッファ1つで「送りつつ受ける」書き方です。

```rust
const TX_DATA: [u8; 8] = [0xA5, 0x5A, 0x01, 0x02, 0x03, 0x04, 0x05, 0xFF];

let mut buf = TX_DATA;

cs.set_low(); // 通信開始（スレーブを選択）
let result = spi.transfer_in_place_async(&mut buf).await;
cs.set_high(); // 通信終了（選択を解除）
```

## コードを一行ずつ読む

```rust
.with_frequency(Rate::from_mhz(1))
.with_mode(Mode::_0);
```

- 周波数とモードは「なんとなくの既定値」に頼らず明示します。実デバイスをつなぐときは、この2つをデータシートの値に合わせるのが最初の仕事です

```rust
let mut cs = Output::new(peripherals.GPIO21, Level::High, OutputConfig::default());
```

- CSはSPIドライバの一部ではなく、**ただのGPIO出力**として自分で操作します。初期値は`High`（非選択）。ここを`Low`で作ると、電源投入直後からデバイスが選択されっぱなしになってしまいます

```rust
let mut buf = TX_DATA;
let result = spi.transfer_in_place_async(&mut buf).await;
```

- `transfer_in_place_async`は、`buf`の内容を送信しながら、**同じ`buf`を受信データで上書き**します。全二重が型に現れた形です。送信データを残したいので、`TX_DATA`のコピーを`buf`に作ってから渡しています

```rust
cs.set_low();
// ... 転送 ...
cs.set_high();
```

- 転送の前後をCSで挟みます。この「Lowの区間」がデバイスにとっての「自分宛ての通信」の範囲です

## 配線

ループバックはMOSIとMISOをつなぐだけです。

```text
GPIO18 (MOSI) ────ジャンパ線──── GPIO20 (MISO)
```

- SCK（GPIO19）とCS（GPIO21）はループバックでは配線不要です（信号は出ていますが、受け手がいません）
- 配線はUSBケーブルを抜いた状態で行います

## 実行方法

```bash
cd examples/05-spi
cargo run --release
```

```text
INFO - SPIループバックを開始します（GPIO18とGPIO20を直結してください）
INFO - OK: 送信 [A5, 5A, 01, 02, 03, 04, 05, FF] → 受信 [A5, 5A, 01, 02, 03, 04, 05, FF]
```

送信と受信が一致すれば成功です。ジャンパ線を抜くと受信データが変わり、`NG`の警告になります。

## よくある失敗

- **送受信が一致しない（NG警告）**: GPIO18とGPIO20がつながっていません。MISOがどこにもつながっていないと、受信データは不定になります
- **実デバイスでモード違い**: マスタがモード0、デバイスがモード3だと、読み取りエッジが半クロックずれてデータが壊れます。エラーにはならず「変な値が返る」ので気づきにくい失敗です
- **CSをHighのまま通信**: デバイスは自分宛てと認識せず、MISOを駆動しません。転送自体は成功（`Ok`）するのに応答が全部0xFFや0x00になる、という症状になります
- **SPI0/SPI1を使おうとする**: C6ではSPI0/1はフラッシュ接続専用です。汎用に使えるのはSPI2だけです

## やってみよう

`TX_DATA`の8バイトを好きな値に変えて、そのまま返ってくることを確かめましょう。次に`Rate::from_mhz(1)`を`Rate::from_mhz(10)`にしても、ジャンパ線1本のループバックなら問題なく動くことを確認してみてください（長い配線や実デバイスでは通らないことがある速度です）。

## 確認問題

1. SPIにはI2Cのようなアドレスがありません。通信相手はどうやって決めますか。
2. 「SPIは全二重」とはどういう意味ですか。`transfer_in_place_async`の動作で説明してください。
3. モード（CPOL/CPHA）が合っていないとどんな症状になりますか。

<details>
<summary>答え</summary>

1. CS（チップセレクト）線で決めます。マスタが通信したいデバイスのCSだけをLowにし、そのデバイスだけが応答します。
2. 送信と受信がクロックごとに同時に進むという意味です。`transfer_in_place_async`はバッファの内容を送信しながら、同じバッファを受信データで上書きします。
3. データを取り込むエッジがずれるため、エラーにはならずに壊れた値（ビットのずれたデータ）が返ります。デバイスのデータシートでモードを確認して合わせます。

</details>

## まとめ

- SPIはSCK/MOSI/MISO/CSの4本線。アドレスの代わりにCSで相手を選び、クロックは常にマスタが刻む
- 全二重なので基本操作は`transfer`系。送りながら同時に受ける
- 周波数とモード（CPOL/CPHA）はデバイスのデータシートに合わせて明示する。C6の汎用SPIはSPI2

## 次のページ

ループバックで転送の形は分かりました。次は実際のSPIデバイスに対して「コマンドを送り、応答を読む」手順を組み立てます。

- 前: [5. I2Cのエラー処理](/embassy-esp32-c6/part08/05-i2c-errors/)
- 次: [7. SPIデバイスを使う](/embassy-esp32-c6/part08/07-spi-device/)
