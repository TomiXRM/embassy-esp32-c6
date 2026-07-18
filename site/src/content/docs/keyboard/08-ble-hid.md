---
title: "8. C6の答え — BLE HIDキーボード"
description: HID over GATT（HOGP）のGATT構造を、cargo check済みのESP32-C6サンプル（examples/15-ble-hid）で完全解説します。ペアリング未実装という正直な限界も明記します。
lesson: 8
difficulty: advanced
estimated_minutes: 25
prerequisites:
  - keyboard/07-usb-hid
  - part11/03-service-characteristic
  - part11/04-peripheral
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - BLEスキャナアプリ（nRF Connect等）を入れたスマートフォン
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1 / esp-radio 0.18.0 / trouble-host 0.6.0"
last_verified: "2026-07-18"
sources:
  - https://www.bluetooth.com/specifications/specs/hid-over-gatt-profile-1-0/
  - https://www.bluetooth.com/specifications/specs/human-interface-device-service-1-0/
  - https://github.com/embassy-rs/trouble
  - https://zenn.dev/nazo6/articles/keyball-embassy-rp2040
---

## このページでできるようになること

- HID over GATT（HOGP）で必要になるGATT構造（サービス・特性・記述子）を列挙できる
- 前ページのレポートディスクリプタと8バイトレポートが、BLE（Bluetooth Low Energy）の上でどう運ばれるかを説明できる
- examples/15-ble-hidを読み、ボタン押下がnotifyになるまでの流れを追える
- このサンプルの限界（ペアリング未実装）と、実用にするための条件を正直に説明できる

## 先に結論

前ページで学んだHIDの階層は、BLE（Bluetooth Low Energy）の上でもほぼそのまま使えます。標準化された運び方が**HID over GATT Profile（HOGP）**で、レポートディスクリプタはGATTの「Report Map特性」として読み取られ、8バイトレポートは「Report特性」のnotify（通知）で飛びます。本教材のexamples/15-ble-hidは、HIDサービス（0x1812）・バッテリーサービス（0x180F）・デバイス情報サービス（0x180A）というHOGPの3点セットをtrouble-hostの`#[gatt_server]`で組み立てた、cargo check済みのC6実装です。ただし**正直な限界**があります。HOGP準拠のキーボードとしてOSに受け入れられるには**ペアリングと暗号化が必須**ですが、教材のバージョン構成ではSMP（Security Manager Protocol）を有効にしていません。そのため本サンプルは「OSで文字が打てるキーボード」ではなく、**nRF ConnectでGATT構造とレポート通知を観察するための学習用サンプル**です。

## 身近なたとえ

前ページの派遣スタッフのたとえを続けます。USB版は説明書と日報を「社内便（有線）」で届けていました。BLE版は同じ説明書と日報を「郵送（無線）」に切り替えただけです。書式は一切変わりません。ただし郵送には社内便になかった手続きがあります。誰でも読める普通郵便では人事情報は送れない——**本人確認と封緘（ペアリングと暗号化）を済ませた相手にしか、OSは日報の受け取りを認めない**のです。

たとえと違うのは、封緘なしでも「郵送の仕組み自体」は動くことです。だから観察ツール（nRF Connect）を使えば、封緘なしのサンプルでも説明書や日報の中身を全部見られます。学習にはむしろ好都合です。

## 仕組み

### HOGPのGATT構造 — キーボードに必要な棚一式

第11部3ページで、GATTは「サービスという棚に、特性という引き出しが並ぶ」構造だと学びました。HOGPは「キーボードならこの棚と引き出しを揃えよ」という品揃えの規格です。examples/15-ble-hidが実装している一式を示します。

| サービス | 特性 / 記述子 | UUID | 役割 |
|---|---|---|---|
| HIDサービス | （サービス本体） | 0x1812 | 「HID機器の棚」 |
| | Protocol Mode | 0x2A4E | ブート(0)/レポート(1)プロトコルの切替。この例は1固定 |
| | **Report** | 0x2A4D | **8バイトの入力レポート本体。notifyで飛ぶ主役** |
| | └ Report Reference記述子 | 0x2908 | このReportが「ID=0のInput型」だと申告する札 |
| | **Report Map** | 0x2A4B | **レポートディスクリプタ（45バイト）。前ページの説明書** |
| | HID Information | 0x2A4A | HID仕様バージョン等の自己紹介4バイト |
| | HID Control Point | 0x2A4C | ホストからのSuspend/Exit Suspend通知の受け口 |
| バッテリーサービス | Battery Level | 0x180F / 0x2A19 | 残量%。HOGPでは必須（この例は100固定） |
| デバイス情報サービス | PnP ID | 0x180A / 0x2A50 | ベンダーID等の身元7バイト。HOGPでは必須 |
| | Manufacturer Name | 0x2A29 | 製造者名の文字列 |

USB版との対応を一言でまとめると——**Report Map = レポートディスクリプタ、Reportのnotify = レポート送信、ポーリングの代わりに通知**。つまり前ページの知識の「運び方の差し替え」です。

Report Reference記述子（0x2908)は初見だと思います。1つのHIDデバイスは入力（キー押下）だけでなく出力（LEDの状態など）のレポートも持てるため、Report特性が複数並ぶことがあります。ホストはどのReportがどれなのかを、各Reportにぶら下がるこの札で見分けます。この例では「レポートID=0、種類=Input(1)」を意味する`[0x00, 0x01]`です。

### GATTテーブルの組み立て — #[gatt_server]

第11部4ページで見たtrouble-hostのマクロで、上の表をそのままRustの構造体にします。抜粋です（完全なコードは examples/15-ble-hid を見てください）。

```rust
#[gatt_server]
struct Server {
    hid_service: HidService,
    battery_service: BatteryService,
    device_info_service: DeviceInfoService,
}

#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)] // 0x1812
struct HidService {
    #[characteristic(
        uuid = characteristic::PROTOCOL_MODE,
        read,
        write_without_response,
        value = 1
    )]
    protocol_mode: u8,
    /// Report Reference記述子（0x2908）付きの入力レポート（8バイト）
    #[descriptor(uuid = descriptors::REPORT_REFERENCE, read, value = [0x00, 0x01])]
    #[characteristic(uuid = characteristic::REPORT, read, notify)]
    input_report: [u8; 8],
    /// Report Map（0x2A4B）: 45バイトのレポートディスクリプタ
    #[characteristic(uuid = characteristic::REPORT_MAP, read, value = REPORT_MAP)]
    report_map: [u8; 45],
    // …HID Information / HID Control Point が続く
}
```

`input_report`に`notify`が付いていることに注目してください。キーイベントは「ホストが読みに来る」のではなく「デバイスから押しつける」——第11部で学んだnotifyの典型的な使いどころです。`REPORT_MAP`の45バイトは前ページで冒頭を読んだ、修飾キー8ビット＋予約8ビット＋キー6個を宣言するあの説明書の全文です。

### アドバタイズ — 「私はキーボードです」と名乗る

OSのスキャン画面でキーボードらしく見せるため、アドバタイズには2つの材料を積みます。HIDサービスUUID（0x1812）と、**Appearance**（見た目の分類コード。キーボードは0x03C1）です。

```rust
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            // HIDサービス(0x1812)を持っていることを知らせる（リトルエンディアン）
            AdStructure::ServiceUuids16(&[[0x12, 0x18]]),
            // Appearance（AD種別0x19）= キーボード(0x03C1、リトルエンディアン)
            AdStructure::Unknown {
                ty: 0x19,
                data: &[0xC1, 0x03],
            },
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
```

ここに小さな現実が写っています。trouble-host 0.6の`AdStructure`にはAppearance専用の列挙子がないため、`Unknown`でAD種別0x19の生バイトを自分で書いています。ライブラリが規格のすべてを覆っているとは限らず、規格書を読んで生バイトで補う場面は実務でも普通にあります。バイト列がリトルエンディアン（下位バイトが先）なのは第2部以来おなじみの約束です。

### ボタンからnotifyまで

BOOTボタン（GPIO9）を押すと'a'のキーダウン→キーアップを通知します。第6部のボタン処理がそのまま生きています。

```rust
        // キーダウン: 「a」(Usage ID 0x04) を押したレポートを通知
        if input_report.notify(conn, &KEY_A_DOWN).await.is_err() {
            break; // 通知失敗 = 切断
        }
        // キーアップ: 全キー解放のレポートを送らないと押しっぱなし扱いになる
        Timer::after_millis(50).await;
        if input_report.notify(conn, &KEY_ALL_UP).await.is_err() {
            break;
        }
```

`KEY_A_DOWN`は`[0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00]`——前ページの「やってみよう」で書いたあの8バイトです。キーアップ（全ゼロ）を必ず対で送る理由も前ページで学んだとおり。全体の骨組み（`ble_task`との`join`、接続ごとの`select`、切断でアドバタイズへ戻るループ）は第11部4ページのPeripheralとまったく同じで、**新しいのはGATTテーブルの中身だけ**です。

### 正直な限界 — ペアリングなしではキーボードになれない

ここが本ページでいちばん大切な段落です。HOGPの仕様は、HIDのやり取りに**ペアリング（ボンディング）と通信の暗号化（Security Mode 1 Level 2以上）を要求**します。キー入力はパスワードそのものが流れる経路なので、平文での運用を規格が禁じているのです。

教材のバージョン固定（trouble-host 0.6.0、features = ["gatt", "derive"]）では、暗号化の手続きを担うSMP（Security Manager Protocol、`security` feature)を有効にしていません。そのため接続は暗号化されず、**iOS/Android/Windows/macOSはこのデバイスをキーボードとして受け入れません**。また、本サンプルはcargo checkとビルドの確認までで、**実機動作は未検証**です。

では何のためのサンプルか。**HOGPのGATT構造を、動くコードとして隅々まで観察するため**です。nRF Connectのような開発者向けアプリはペアリングなしでもGATTの読み取りとnotify購読ができるので、「OSがキーボードを受け入れる直前まで」の全部が見えます。ここから実用に進む道は最終ページで扱います（先に言えば、ペアリングまで実装済みのRMKというフレームワークがあります）。

## 実行方法

```bash
cd examples/15-ble-hid
cargo run --release
```

```text
INFO - デバイスアドレス: ...
INFO - HIDキーボードのGATTサーバーを起動し、アドバタイズを開始します
INFO - [adv] アドバタイズ中（名前: C6-KEYBOARD）
```

スマートフォンのnRF Connectで観察します（設計上の期待動作です）。

1. スキャン画面で「C6-KEYBOARD」を探す。Appearanceによりキーボードのアイコンが付く
2. CONNECTで接続すると、ログに「接続されました」と出る
3. サービス一覧にHID（0x1812）・Battery（0x180F）・Device Information（0x180A）が並ぶ
4. Report Map（0x2A4B）をREADすると45バイトのディスクリプタが読める（ログには「Report Mapが読み取られました」）
5. Report（0x2A4D）のnotifyを購読（3本矢印のアイコン）してからBOOTボタンを押すと、`00-00-04-00-00-00-00-00`（キーダウン）と`00-00-00-00-00-00-00-00`（キーアップ）の2通が届く

## よくある失敗

- **OSのBluetooth設定画面からペアリングして文字入力を試す** — 上で述べたとおりSMP未実装のため失敗します。これはバグではなく本サンプルの仕様上の限界です。観察はnRF Connect等で行ってください
- **notifyの購読を忘れて「何も届かない」と悩む** — notifyは受け手が購読（CCCDという記述子への書き込み。アプリのボタン一つでやってくれます）を済ませて初めて届きます。第11部のnotifyの復習です
- **Report Referenceを省いてホストを混乱させる** — Report特性だけ置いて記述子を忘れると、ホストはそのレポートの種類を判別できません。HOGPの品揃えは「全部そろって一式」です
- **アドバタイズにサービスUUIDもAppearanceも積まない** — 接続前のOSは中身を知らないため、スキャン画面でただの無名デバイスに見えます。名乗りはアドバタイズの仕事です

## やってみよう

`KEY_A_DOWN`を書き換えて、「Shift+A」（大文字のA）を送るレポートにしてみましょう。前ページの「やってみよう」で紙に書いた8バイトが、そのまま答えになります。ビルドしてnRF Connectで届くバイト列が変わることを確認してください（byte 0が0x02になっているはずです）。

## 確認問題

1. USB HIDとHID over GATTで「変わらないもの」を2つ、「変わるもの」を2つ挙げてください。
2. Report Map特性とReport特性の役割の違いを、前ページの「説明書と日報」のたとえで説明してください。
3. このサンプルをOSが受け入れるキーボードにするために足りないものは何ですか。また、規格がそれを必須にしている理由も答えてください。

<details>
<summary>答え</summary>

1. 変わらないもの: レポートディスクリプタ（Report Mapの中身）と8バイトのレポート形式。変わるもの: 運び方（有線ポーリング→無線notify）と、必須になるセキュリティ（ペアリング・暗号化）。他にバッテリーサービス等の必須サービス構成も挙げられます。
2. Report Mapが「説明書」で、接続時に一度READされるだけ。Reportが「日報」で、キー操作のたびにnotifyで繰り返し届きます。説明書を先に渡してあるから、日報は8バイトで済みます。
3. SMP（ペアリングと暗号化、Security Mode 1 Level 2以上）です。キーボードの通信にはパスワード等の機密がそのまま流れるため、盗聴できる平文のHID入力を規格が禁じています。

</details>

## まとめ

- HOGPは「HIDサービス0x1812＋バッテリー0x180F＋デバイス情報0x180A」の品揃え規格。Report Mapが説明書、Reportのnotifyがキーイベントの本体で、USB HIDの知識がほぼそのまま通用する
- examples/15-ble-hidは第11部のPeripheralの骨組みにHOGPのGATTテーブルを載せたもの。新しい概念はReport Reference記述子とAppearanceくらいしかない
- 本サンプルはSMP未実装のためOSには受け入れられない、GATT構造観察用の学習サンプル。実用への道（RMK等）は最終ページで扱う

## 次のページ

キーが打てて、無線で届く。しかし実物のキーボードにはトラックボールもOLEDもLEDもあり、しかもそれぞれ動く周期が違います。部品が1つ壊れても全体は動き続ける「劣化運転」を含む、統合の設計を読み解きます。

[9. 統合の設計 — 劣化運転と共有リソース →](/embassy-esp32-c6/keyboard/09-integration/)

---

前: [7. USB HIDの仕組みと、C6にUSBがない話](/embassy-esp32-c6/keyboard/07-usb-hid/) | 次: [9. 統合の設計 — 劣化運転と共有リソース](/embassy-esp32-c6/keyboard/09-integration/)
