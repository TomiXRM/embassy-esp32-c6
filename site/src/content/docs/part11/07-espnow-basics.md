---
title: "7. ESP-NOWの基礎"
description: ESP-NOWの接続レス通信を学びます。ペアリング不要・同一チャンネル必須という特徴を、ブロードキャスト送受信のコードで確かめます。
part: 11
lesson: 7
difficulty: intermediate
estimated_minutes: 20
prerequisites:
  - part10/01-wifi-basics
  - part09/07-select
  - part06/08-ticker
hardware:
  - ESP32-C6-DevKitC-1（2台あると受信も確認できる。1台でも送信は確認可）
  - USBケーブル
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1 / esp-radio 0.18.0"
last_verified: "2026-07-18"
sources:
  - https://www.espressif.com/en/solutions/low-power-solutions/esp-now
  - https://github.com/esp-rs/esp-hal
---

## このページでできるようになること

- ESP-NOWが「接続レス」であることの意味と利点・制約を説明できる
- MACアドレス宛て送信とブロードキャストの違いが分かる
- 2台のESP32-C6でブロードキャスト送受信を動かせる

## 先に結論

ESP-NOWはEspressif独自の通信方式です。Wi-Fiと同じ2.4GHz帯の電波とフレームを使いますが、**ルーターへの接続もIPアドレスもペアリングも不要**で、ボード同士が直接パケットを送り合えます。宛先はMACアドレス（またはブロードキャスト=全員宛て）で指定します。接続手続きがないぶん起動直後から送れて低遅延ですが、**送る側と受ける側が同じWi-Fiチャンネルに合っていること**が必須です。TCPのような到達保証はありません（次ページの主題です）。

## 身近なたとえ

Wi-Fi + TCP（第10部）が「電話」だとすれば、ESP-NOWは「トランシーバー」です。電話は回線をつなぐ手続きをしてから話しますが、トランシーバーはボタンを押せば即座に話せます。そのかわり、お互いが**同じチャンネル**に合わせていなければ聞こえませんし、相手に届いた保証もありません。

ただし実際のESP-NOWはトランシーバーと違い、音声ではなく最大250バイトのデータパケットを送るもので、宛先MACアドレスを指定すれば「特定の相手宛て」にもできます。

## 仕組み

### Wi-Fiの電波を使うが、Wi-Fiのネットワークには入らない

第10部で学んだ積み重ね（Wi-Fi→IP→TCP→HTTP）と比べると、ESP-NOWの位置づけがよく分かります。

```mermaid
graph TD
  subgraph "第10部のWi-Fi通信"
    H["HTTP / MQTT"] --> T["TCP / UDP"] --> I["IP（アドレス・経路）"] --> W1["Wi-Fi（電波・フレーム）"]
  end
  subgraph "ESP-NOW"
    A["アプリのデータ（最大250バイト）"] --> W2["Wi-Fi（電波・フレーム）を直接利用"]
  end
```

- ルーター（アクセスポイント）不要。SSIDもパスワードもDHCPもなし
- 上位層（IP/TCP）を持たないので、そのぶんの手続きと待ち時間がない
- そのかわり、経路制御も到達保証も再送も、上位層が担っていた仕事は何もしてくれない

### 宛先はMACアドレス

各ボードには工場出荷時に固有のMACアドレス（6バイト）がeFuseに書き込まれています。ESP-NOWの宛先指定は2通りです。

| 宛先 | 意味 |
|---|---|
| 特定のMACアドレス | その1台宛て（ユニキャスト） |
| `BROADCAST_ADDRESS`（FF:FF:FF:FF:FF:FF） | 電波が届く全員宛て（ブロードキャスト） |

このページの例は、相手のアドレスを知らなくても動くブロードキャストを使います。

### 同一チャンネル必須

2.4GHz帯のWi-Fiチャンネル（1〜13）のうちどれを使うかを、送る側と受ける側で**一致**させる必要があります。ここがずれると、電波は出ているのに1バイトも届きません。ESP-NOWで一番多いつまずきです。

## Arduinoではどう書くか

ArduinoでもESP-NOWは人気で、`esp_now_init()`や`esp_now_send()`といったESP-IDFのC APIを直接呼び、受信はコールバック関数の登録で行います。Rust + Embassyでは、コールバックの代わりに`receive_async().await`で「受信を待つ」と書けるため、送信と受信をひとつの`select`ループに素直に並べられます。

## RustとEmbassyではどう書くか

examples/10-esp-nowから抜粋します。まず初期化です。

```rust
    // Wi-Fiドライバを初期化すると、ESP-NOWインターフェースも一緒に得られる。
    // コントローラ本体はESP-NOWだけなら操作不要（ただしdropすると
    // 無線が止まるので、変数名を _controller にして保持しておく）
    let (_controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default()).unwrap();
    let mut esp_now = interfaces.esp_now;

    // 送受信するボード同士は同じWi-Fiチャネルに合わせる必要がある
    esp_now.set_channel(11).unwrap();
```

続いて送受信ループです。1秒ごとの送信と受信待ちを`select`で並行させます。

```rust
    let mut counter: u32 = 0;
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        // 「1秒タイマー」と「パケット受信」を並行して待ち、
        // 先に完了した方を処理する
        match select(ticker.next(), esp_now.receive_async()).await {
            // 1秒経過 → ブロードキャスト送信
            Either::First(_) => {
                counter = counter.wrapping_add(1);
                let mut payload = [0u8; PAYLOAD_LEN];
                payload[..4].copy_from_slice(&counter.to_le_bytes());
                payload[4..].copy_from_slice(mac.as_bytes());

                let status = esp_now.send_async(&BROADCAST_ADDRESS, &payload).await;
                info!("送信 counter={} 結果={:?}", counter, status);
            }
            // パケット受信 → 送信元MACアドレスと内容をログ表示
            Either::Second(received) => {
                let data = received.data();
                info!(
                    "受信 送信元MAC={:02x?} 宛先MAC={:02x?} データ={:02x?}",
                    received.info.src_address, received.info.dst_address, data
                );
            }
        }
    }
```

これは抜粋です。完全なコードは examples/10-esp-now を見てください。

## コードを一行ずつ読む

- `esp_radio::wifi::new(...)` — ESP-NOWはWi-Fiの電波を使うので、入り口はWi-Fiドライバです。戻り値の`interfaces`から`esp_now`インターフェースを取り出します。接続（Station設定やDHCP）は一切していないことに注目してください
- `_controller` — 使わないのに変数で受けるのは、dropされると無線ごと止まるためです。アンダースコア付きの名前で「保持だけする」意図を示します（第3部の所有権の応用です）
- `esp_now.set_channel(11).unwrap()` — チャンネルを11に固定します。**通信する全ボードでこの値を一致させます**
- `let mac = interface_mac_address(InterfaceMacAddress::Station);` — （抜粋外）自分のMACアドレスをeFuseから読みます。ESP-NOWはWi-FiのStation用MACで送信されます
- `payload[..4].copy_from_slice(&counter.to_le_bytes());` — ペイロードは自分で決めた形式のただのバイト列です。ここでは通し番号4バイト+自分のMAC6バイトの計10バイト。形式の設計は完全に自分の責任です（次ページの伏線です）
- `send_async(&BROADCAST_ADDRESS, &payload).await` — 全員宛てに送信し、送信処理の完了を待ちます。戻り値`status`は「電波を送り終えた」ことを示すだけで、**誰かが受け取った保証ではありません**
- `receive_async().await` — パケットが届くまで待ちます。`received.info`に送信元・宛先MAC、`received.data()`に中身が入っています

## 配線

不要です。

## 実行方法

同じプログラムを2台のESP32-C6に書き込みます（1台ずつUSBポートにつないで`cargo run --release`）。

```bash
cd examples/10-esp-now
cargo run --release
```

2台目を書き込んで両方が動くと、お互いのログにこう表示されます。

```text
INFO - ESP-NOWバージョン: 1
INFO - 自分のMACアドレス: ...
INFO - 送信 counter=1 結果=Ok(())
INFO - 受信 送信元MAC=[...] 宛先MAC=[ff, ff, ff, ff, ff, ff] データ=[...]
INFO -   → 相手のカウンタ=7 相手のMAC=[...]
```

1台しかない場合も「送信 counter=...」のログで送信側の動作は確認できます。

## よくある失敗

- **2台とも動いているのに受信ログが出ない** — チャンネル不一致が第一容疑者です。両方のコードが`set_channel(11)`で同じ値か確認してください。片方だけ書き換えて実験した後に戻し忘れるのが典型です
- **宛先MACに自分のMACを書いてしまう** — 自分宛てのパケットは自分では受信できません。2台実験でユニキャストを試すときは「相手の」MACアドレス（起動ログに表示されます）を書きます
- **`esp-now`featureを付け忘れてビルドエラー** — `esp_now`モジュールが見つからないと言われたら、Cargo.tomlのesp-radioに`esp-now`と`unstable`のfeatureが付いているか確認してください（examples/10-esp-now/Cargo.tomlが正解の見本です）
- **送信結果がOkだから届いたと思い込む** — `send_async`のOkは送信完了であって受信確認ではありません。特にブロードキャストは誰も聞いていなくてもOkが返ります

## やってみよう

送信間隔`Ticker::every(Duration::from_secs(1))`を200msに変えて2台で動かし、受信ログの流れる速さが変わることを確認しましょう。Tickerなので処理時間が挟まっても周期はずれません（第6部8ページの復習です）。

## 確認問題

1. ESP-NOWがWi-Fi + TCP通信より速く送り始められるのはなぜですか。
2. 通信する2台のボードで必ず一致させなければならない設定は何ですか。
3. `send_async`が`Ok`を返したとき、確実に言えることは何ですか。言えないことは何ですか。

<details>
<summary>答え</summary>

1. ルーターへの接続・IPアドレスの取得・TCPの接続確立といった手続きが一切なく、起動してすぐMACアドレス宛てに電波を送れるから。
2. Wi-Fiチャンネル（この例では11）。ずれていると一切受信できません。
3. 言えるのは「送信処理が完了した（電波を出し終えた）」こと。言えないのは「相手が受け取った」こと。特にブロードキャストでは受信確認の仕組みがありません。

</details>

## まとめ

- ESP-NOWは接続手続きなし・ルーター不要でボード同士が直接通信する方式
- 宛先はMACアドレスかブロードキャスト。全ボードで同一チャンネル必須
- 送信Okは到達保証ではない。保証が欲しければ自分で作る（次ページ）

## 次のページ

「届いた保証がない」ならどうするか。連番・ACK・再送・重複排除という、信頼性を自分で組み立てる設計を学びます。最終プロジェクトの通信部分の土台になります。

[8. ESP-NOWの再送と重複排除 →](/embassy-esp32-c6/part11/08-espnow-reliability/)

---

前: [6. BLEでボタン状態を送る](/embassy-esp32-c6/part11/06-ble-button/) | 次: [8. ESP-NOWの再送と重複排除](/embassy-esp32-c6/part11/08-espnow-reliability/)
