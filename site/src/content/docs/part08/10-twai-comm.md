---
title: "10. TWAIで通信する"
description: セルフテストモードでTWAIフレームを送受信し、トランシーバ2組と終端抵抗を使った実バス（Normalモード）への発展手順を学びます。
part: 8
lesson: 10
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part08/09-twai-basics
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル（データ通信対応）
  - ジャンパ線 1本
  - （発展）2台目のESP32-C6-DevKitC-1
  - （発展）TWAIトランシーバ×2（TJA1051等の3.3V対応品）
  - （発展）終端抵抗 120Ω×2
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/peripherals/twai.html
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
---

## このページでできるようになること

- セルフテストモードでTWAIフレームを送信・受信できる
- `StandardId`と`EspTwaiFrame`でフレームを組み立てられる
- 実バス（Normalモード＋トランシーバ2組＋終端抵抗）へ発展させる手順が分かる

## 先に結論

前ページのとおり、普通のCAN送信は「他ノードのACK」がないと失敗します。そこで学習にはACK不要の**セルフテストモード**（`TwaiMode::SelfTest`）を使います。トランシーバも相手ノードも不要で、GPIO2（TX）とGPIO3（RX）をジャンパ線1本で直結すれば、自分の送ったフレームを自分で受信できます。フレームは`StandardId::new(0x123)`と`EspTwaiFrame::new_self_reception(id, &データ)`で組み立て、`transmit_async`/`receive_async`を`await`します。初期化の`new_no_transceiver`は**引数がRX、TXの順**である点に注意してください。実バスに進むときは、モードをNormalへ、構成をトランシーバ経由へ変え、2ノード＋両端120Ωのバスを組みます。

## 身近なたとえ

セルフテストモードは「本番前のひとりリハーサル」です。観客（他ノード）の拍手（ACK)がなくても進行を止めない練習モードで、自分のセリフを録音して聞き返す（自己受信）ことで、台本（フレームの組み立て）と発声（送受信の手順）を確認できます。

ただし本物のリハーサルと違い、コントローラの動作は本番（Normalモード）とほぼ同じで、**変わるのはACKの要否だけ**です。ここで書いたコードの大部分は実バスでもそのまま使えます。

## 仕組み

セルフテスト構成と実バス構成の違いを図で押さえます。

```mermaid
graph TB
  subgraph セルフテスト構成（このページで動かす）
    A[TWAI0<br>TX=GPIO2 / RX=GPIO3] -->|ジャンパ線1本で直結| A
  end
```

```text
実バス構成（発展）:

  C6 その1                                      C6 その2
  TX GPIO2 ──▶ TXD┐                  ┌TXD ◀── TX GPIO2
  RX GPIO3 ◀── RXD│トランシーバ1     │トランシーバ2 RXD ──▶ RX GPIO3
                  │(TJA1051/3.3V)   │(TJA1051/3.3V)
                  ├CAN_H ═══════════┤CAN_H
                  ├CAN_L ═══════════┤CAN_L
                 [120Ω]            [120Ω]
                  ├3.3V/VIO, GND    ├3.3V/VIO, GND
```

このページの主な登場要素です。

| 要素 | 役割 |
|---|---|
| `TwaiMode::SelfTest` | ACK不要で送信が成立する学習・自己診断用モード |
| `new_no_transceiver` | トランシーバなしのピン直結構成用の初期化（TXをオープンドレイン＋プルアップに設定） |
| `StandardId` | 標準フォーマットの11ビットID（0x000〜0x7FF） |
| `EspTwaiFrame::new_self_reception` | 「自分でも受信する」印付きのフレームを作る。セルフテストで自己受信するために必要 |
| `transmit_async` / `receive_async` | フレーム単位の送受信。`await`で完了やフレーム到着を待つ |

UARTが「バイトの列」を送るのに対し、TWAIは**フレーム（ID＋最大8バイト）という意味のかたまり**を送る点が使い心地の違いです。受信側は`receive_async`でフレーム1個を丸ごと受け取ります。

## RustとEmbassyではどう書くか

これは抜粋です。完全なコードは `examples/11-twai` を見てください。

```rust
use esp_hal::twai::{BaudRate, EspTwaiFrame, StandardId, TwaiConfiguration, TwaiMode};

// TWAI0を500kbps・セルフテストモードで設定する。
// 引数の順番は「RXピン, TXピン」なので注意！
let twai_config = TwaiConfiguration::new_no_transceiver(
    peripherals.TWAI0,
    peripherals.GPIO3, // RX
    peripherals.GPIO2, // TX
    BaudRate::B500K,
    TwaiMode::SelfTest,
)
.into_async(); // 非同期(async)版に変換

// start()で設定を確定し、実際に動くTwaiドライバを得る
let mut twai = twai_config.start();

// 送信するフレーム: 標準ID 0x123、データ4バイト。
// セルフテストモードで自分の送信を自分で受信するには、
// 「自己受信フレーム」(new_self_reception)として送る必要がある
let id = StandardId::new(0x123).unwrap();
let frame = EspTwaiFrame::new_self_reception(id, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();

loop {
    // --- 送信 ---
    match twai.transmit_async(&frame).await {
        Ok(()) => info!("送信OK: {frame:?}"),
        Err(e) => error!("送信エラー: {e:?}"),
    }

    // --- 受信 ---
    // セルフテストモードなので、いま送ったフレームが自分に届く
    match twai.receive_async().await {
        Ok(received) => info!("受信OK: {received:?}"),
        Err(e) => error!("受信エラー: {e:?}"),
    }

    Timer::after(Duration::from_secs(1)).await;
}
```

## コードを一行ずつ読む

```rust
TwaiConfiguration::new_no_transceiver(
    peripherals.TWAI0,
    peripherals.GPIO3, // RX
    peripherals.GPIO2, // TX
    ...
)
```

- **引数はRX、TXの順**です。UARTの`with_tx`/`with_rx`のような名前付きメソッドではなく位置引数なので、逆にすると一切通信できません。コメントで順番を明示しておくのが自衛策です
- `new_no_transceiver`は、TXピンをオープンドレイン＋プルアップに設定し、ピン直結でもバスらしい電気的振る舞いになるようにしてくれます

```rust
let mut twai = twai_config.start();
```

- 設定段階（`TwaiConfiguration`）と稼働段階（`Twai`）が別の型になっています。`start()`を呼んで初めてバスに参加し、送受信メソッドが使えるようになります。「設定中の中途半端な状態で送信してしまう」誤りが型で防がれています

```rust
let id = StandardId::new(0x123).unwrap();
let frame = EspTwaiFrame::new_self_reception(id, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
```

- `StandardId::new`は11ビットに収まらない値（0x800以上）だと`None`を返します。0x123は定数で範囲内が自明なので、ここでは`unwrap`を使い、その理由をこの一文で明示しています。データも最大8バイトの制限があり、`new_self_reception`も同様に検査します
- 通常の送信フレームではなく**自己受信フレーム**にするのは、セルフテストで自分に届けるためです

```rust
twai.transmit_async(&frame).await
twai.receive_async().await
```

- どちらも`await`で待ちます。特に`receive_async`は「フレームが届くまで眠る」ので、受信専用taskを作って待たせておく設計（第9部で本格化します）と好相性です

## 配線

セルフテストモードの配線はこれだけです。

```text
GPIO2 (TX) ────ジャンパ線──── GPIO3 (RX)
```

- 配線はUSBケーブルを抜いた状態で行います
- GPIO2は第7部のADC実験と同じピンです。可変抵抗などが残っていたら外してください（排他利用）

## 実行方法

```bash
cd examples/11-twai
cargo run --release
```

```text
INFO - TWAIセルフテストを開始します（500kbps, ID=0x123）
INFO - 送信OK: EspTwaiFrame { id: Standard(StandardId(291)), data: [222, 173, 190, 239], .. }
INFO - 受信OK: EspTwaiFrame { id: Standard(StandardId(291)), data: [222, 173, 190, 239], .. }
```

送信と受信のIDとデータが一致すれば成功です（291は0x123、[222, 173, 190, 239]は0xDE 0xAD 0xBE 0xEFの10進表示です）。

## 実バスへの発展手順

トランシーバ2組と2台目のC6が用意できたら、次の順で本物のバスに進みます。

1. **配線を変える**: ジャンパ直結をやめ、各C6のGPIO2（TX）→トランシーバのTXD、GPIO3（RX）←RXDへ。トランシーバの電源は3.3V系の指定どおりに（VIOピンがある品種は3.3Vへ）。2つのトランシーバのCAN_H同士・CAN_L同士を撚り線でつなぎ、**両端に120Ωを1本ずつ**入れます
2. **コードを変える**: 構成をトランシーバ経由用に変えます。`new_no_transceiver`はピン直結専用なので使いません。モードは`TwaiMode::Normal`にします（前ページで見たとおり、Normalでは他ノードのACKが必要です。だから2台そろってから切り替えます）
3. **フレームを変える**: 自己受信（`new_self_reception`）は不要になり、通常の送信フレームで相手ノードに届けます。片方を「1秒ごとに送信」、もう片方を「`receive_async`で待ち受け」にすると動作が見やすいです
4. **動かして観察する**: 片方の電源を切るとACKが返らなくなり、送信側にエラーが積み上がる様子（やがてバスオフ）も観察できます

手順2〜3のNormalモード用コードは、この教材のexamplesではまだ検証していません。挑戦するときはesp-halの公式ドキュメント（sources参照）で該当APIを確認しながら進めてください。

## よくある失敗

- **RXとTXの引数を逆に渡す**: 位置引数なのでコンパイルは通りますが、送信も受信もできません。「RXが先」を必ず確認します
- **通常フレームで送って自己受信できない**: セルフテストモードで自分に届くのは`new_self_reception`で作ったフレームだけです。送信は成功するのに`receive_async`が永遠に待つ症状になります
- **Normalモードのままピン直結で動かす**: ACKを返す相手がいないため送信が失敗し続けます。1台構成では必ずSelfTestモードを使います
- **GPIO2に前の実験の部品が残っている**: ADC章と同じピンを使うため、可変抵抗がつながったままだと信号が乱れます

## やってみよう

`StandardId::new(0x123)`を別のID（0x000〜0x7FFの範囲内）に、データを自分の好きな最大8バイトに変えて、ログの表示が追従することを確かめましょう。さらに0x800を渡すと`unwrap`がパニックすることも（一度だけ）観察して、「IDは11ビット」という制約が型検査に現れていることを体感してください。

## 確認問題

1. セルフテストモードが1台だけで動くのはなぜですか。
2. `new_no_transceiver`の引数で特に注意すべき点は何ですか。
3. 実バスへ発展させるとき、コード面で変わる主な点を2つ挙げてください。

<details>
<summary>答え</summary>

1. 通常のCANでは他ノードのACKがないと送信が失敗しますが、セルフテストモードはACK不要で送信が成立するからです。
2. 引数の順番が「RXピン、TXピン」であることです。位置引数なので逆でもコンパイルが通ってしまい、動かない原因になります。
3. モードを`TwaiMode::SelfTest`から`TwaiMode::Normal`へ変えること、自己受信フレーム（`new_self_reception`）をやめて通常フレームで相手ノードへ送ること。加えて構成もトランシーバ経由用に変えます。

</details>

## まとめ

- セルフテストモード＋GPIO2/GPIO3直結なら、トランシーバなしの1台でTWAIの送受信を学べる
- フレームは`StandardId`（11ビット）＋最大8バイトで組み立て、`transmit_async`/`receive_async`を`await`する
- 実バスはNormalモード＋トランシーバ2組＋両端120Ω。ACKの有無がセルフテストとの本質的な違い

## 次のページ

第8部で「待つ」処理がたくさん出てきました。UARTの受信待ち、I2Cの測定待ち、TWAIのフレーム待ち。第9部では、この「待ち」を武器に変えるEmbassyの非同期処理を正面から学びます。

- 前: [9. TWAI基礎](/embassy-esp32-c6/part08/09-twai-basics/)
- 次: [第9部 1. 同期処理と非同期処理](/embassy-esp32-c6/part09/01-sync-vs-async/)
