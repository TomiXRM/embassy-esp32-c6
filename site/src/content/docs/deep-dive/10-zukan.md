---
title: "10. 図鑑の残りと、深淵の歩き方"
description: USB Serial/JTAG・eFuse・Wi-Fiスニファ/CSIなど残りの機能を総覧し、20機能×「Rustからの現在地」の総括表で応用編4を締めくくります。
lesson: 10
difficulty: advanced
estimated_minutes: 20
prerequisites:
  - deep-dive/09-bus-and-bits
status: complete
code_status: none
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-guides/usb-serial-jtag-console.html
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/security/security.html
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-guides/wifi.html
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-guides/coexist.html
  - https://docs.espressif.com/projects/rust/esp-radio/0.18.0/esp32c6/esp_radio/index.html
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/esp_hal/efuse/index.html
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/esp_hal/usb_serial_jtag/index.html
---

## このページでできるようになること

- 図鑑でまだ扱っていない機能（USB Serial/JTAG、eFuse、Wi-Fiスニファ/CSIなど）の概要を説明できる
- 20機能それぞれの「Rustからの現在地」を総括表で見渡せる
- 新しい機能に出会ったとき、何をどの順で調べればよいか（深淵の歩き方）が分かる

## 先に結論

応用編4の締めくくりに、残りの機能を1〜2段落ずつ総覧します。詳しく学んだものは既存章へのリンクで済ませ、重複はさせません。最後に全20機能×「Rustからの現在地」の総括表を置きます。持ち帰ってほしい結論は2つです。第一に、ESP32-C6は「Wi-Fi付きの高性能Arduino」として使うだけではもったいないチップだということ。第二に、機能を調べる順番は必ず「ハードは対応しているか（TRM/データシート）→ ESP-IDFにAPIはあるか → 今のesp-hal/esp-radioで書けるか」の三段で確認すること。この三段を区別できるようになったこと自体が、この図鑑の成果です。

## 身近なたとえ

博物館の常設展を全室ゆっくり見てきたので、最後は館内マップの前に戻ってきた——このページはそういう位置づけです。駆け足で通り過ぎた展示室を振り返り、どの部屋が「今すぐ触れる体験コーナー」で、どの部屋が「ガラス越しの展示」（＝ハードにはあるがRustからはまだ）なのかを地図に書き込みます。実際の開発では、この地図はバージョンが上がるたびに更新されていきます。

## USB Serial/JTAG — ケーブル1本で書き込みからブレークポイントまで

開発ボードのUSBポートの片方（GPIO12/13）は、CP2102Nのような外付け変換チップではなく、C6に**内蔵された**USB Serial/JTAGペリフェラルです。第1部9ページの書き込みとログ表示も、実はこの回路で全部できます。さらにJTAG（CPUを外から制御するデバッグ規格）を備えているので、probe-rsのようなツールでプログラムを任意の行で一時停止（ブレークポイント）し、変数の中身を覗き、一行ずつ実行できます。`info!`を差し込んで再書き込みするデバッグ（[第12部6ページ](/embassy-esp32-c6/part12/06-logging-debug/)）の、次の段階です。

**Rustからの現在地: unstableで試せる** — esp-halの`usb_serial_jtag`モジュール（unstable）でシリアル入出力を書けます（`into_async()`対応、embedded-io実装あり）。JTAGデバッグはプログラム側のコードではなくツール（probe-rs）の世界です。関連: [第1部9ページ](/embassy-esp32-c6/part01/09-flash-monitor/)。

## eFuse — 一度焼いたら戻せないヒューズ

eFuseは、チップ内部にある**一度書いたら二度と消せない**設定ビット群です。名前のとおり小さなヒューズを電気的に焼き切って1を記録します。MACアドレスやチップのリビジョンもここに入っています。この不可逆性こそが製品セキュリティの土台で、Secure Boot（署名のないプログラムを起動させない）、Flash Encryption（フラッシュの中身を暗号化する）、Digital Signature（秘密鍵をソフトから読めないままハードが署名する）といった機能の鍵や設定がeFuseに焼かれます。趣味の工作の先にある「製品として守る」世界の入口です。

**Rustからの現在地: 今すぐ試せる（読み取りのみ）** — esp-halの`efuse`はstable APIで、チップリビジョンやMACアドレスを読めます。書き込みAPIはなく、Secure Boot等の書き込み・運用はESP-IDF/espefuseツールの領分（概念のみ）です。焼く操作は取り返しがつかないので、初学者のうちは読み取り専用で正解です。

## Wi-Fiは測定器にもなる — スニファとCSI

第10部ではWi-Fiを「インターネットへの接続手段」として使いましたが、無線回路は見方を変えると**空間の測定器**です。スニファ（プロミスキャスモード）は、自分宛てでない802.11フレームも含めて周囲の電波を観測する機能で、生フレームの送信もできます。CSI（Channel State Information）はさらにキモい機能で、電波が部屋の壁や人体に反射・減衰した「チャネルの状態」を数値として取り出します。人の在室検知のような「Wi-Fiをセンサにする」研究はこのCSIが主役です。なお、他人の通信を観測する行為には法律とマナーの制約があります。自分の実験環境の電波だけを対象にしてください。

**Rustからの現在地: unstableで試せる** — esp-radio 0.18に`sniffer` feature（+`unstable`）が実在し、`Interfaces`の`sniffer`から`set_promiscuous_mode`・`set_receive_cb`・`send_raw_frame`が使えます。CSIも`csi` featureで`WifiController::set_csi(CsiConfig, コールバック)`が用意されています。

## 既存章で学んだ「キモい機能」たち

以下は本編・応用編ですでに詳しく扱ったので、リンクの掲示だけにします。

- **ESP-NOW** — SSIDもIPもサーバも要らないWi-Fi直接通信。[第11部7ページ](/embassy-esp32-c6/part11/07-espnow-basics/)と最終プロジェクトで体験済みです
- **Wi-Fi 6 + BLE + 802.15.4の同居** — 1チップに3つの無線世界。2.4GHz帯を時分割で分け合う共存制御ごと内蔵されています。[第11部9ページ](/embassy-esp32-c6/part11/09-ieee802154/)・[10ページ](/embassy-esp32-c6/part11/10-thread-zigbee/)
- **Deep Sleepの電源区画設計** — 「どの区画へ電気を残すか」を設計する話。[第12部2ページ](/embassy-esp32-c6/part12/02-deep-sleep/)と[8ページ（LPコア）](/embassy-esp32-c6/deep-dive/08-lp-core/)
- **TWAI×2** — CAN準拠のコントローラが2つ。ID調停やエラー処理まで踏み込むとUARTとの根本的な違いが見えます。[第8部9ページ](/embassy-esp32-c6/part08/09-twai-basics/)（外付けトランシーバ必須）

## 総括表 — 20機能×Rustからの現在地

凡例: **今すぐ** = stable APIで試せる / **unstable** = esp-halのunstable feature（本教材の構成では有効化済み）で試せる / **概念のみ** = ハードとESP-IDFにはあるがRust未実装。esp-hal 1.1.1 / esp-radio 0.18.0時点の状況で、バージョンが上がれば変わります。

| # | 機能 | ひとこと | Rustからの現在地 | 詳しくは |
|---|---|---|---|---|
| 1 | GPIO Matrix | チップ内の配線盤 | 今すぐ | [2ページ](/embassy-esp32-c6/deep-dive/02-gpio-matrix/) |
| 2 | RMT | 波形の演奏装置 | unstable | [3ページ](/embassy-esp32-c6/deep-dive/03-rmt/) |
| 3 | PCNT | ハードのパルス計数 | unstable | [4ページ](/embassy-esp32-c6/deep-dive/04-pcnt/) |
| 4 | ETM | 割り込みなしの直結 | unstable（対応周辺は限定） | [5ページ](/embassy-esp32-c6/deep-dive/05-etm/) |
| 5 | LEDCハードフェード | 明るさ変化を自動化 | unstable | [6ページ](/embassy-esp32-c6/deep-dive/06-ledc-dma/) |
| 6 | DMA (GDMA) | データの運搬係 | unstable | [6ページ](/embassy-esp32-c6/deep-dive/06-ledc-dma/) |
| 7 | ADC連続+DMA | 簡易オシロの入口 | 概念のみ | [6ページ](/embassy-esp32-c6/deep-dive/06-ledc-dma/) |
| 8 | MCPWM | モーター制御工場 | unstable（一部機能） | [7ページ](/embassy-esp32-c6/deep-dive/07-mcpwm/) |
| 9 | LPコア | 地下室のもう一人 | unstable（難度高） | [8ページ](/embassy-esp32-c6/deep-dive/08-lp-core/) |
| 10 | PARLIO | 自作パラレルバス | unstable | [9ページ](/embassy-esp32-c6/deep-dive/09-bus-and-bits/) |
| 11 | Dedicated GPIO | GPIO専用CPU命令 | unstable | [9ページ](/embassy-esp32-c6/deep-dive/09-bus-and-bits/) |
| 12 | SDM (Σ-Δ) | デジタルピンで疑似アナログ | 概念のみ | [9ページ](/embassy-esp32-c6/deep-dive/09-bus-and-bits/) |
| 13 | GPIOグリッチフィルタ | ノイズをCPUの手前で除去 | 概念のみ | [9ページ](/embassy-esp32-c6/deep-dive/09-bus-and-bits/) |
| 14 | USB Serial/JTAG | ケーブル1本でデバッグ | unstable | このページ |
| 15 | eFuse | 戻せない設定とセキュリティ | 今すぐ（読み取りのみ） | このページ |
| 16 | Wi-Fiスニファ/CSI | Wi-Fi=空間の測定器 | unstable | このページ |
| 17 | ESP-NOW | 設定いらずのWi-Fi直接通信 | unstable（第11部で体験済み） | [第11部7ページ](/embassy-esp32-c6/part11/07-espnow-basics/) |
| 18 | 無線3種の同居 | Wi-Fi6+BLE+802.15.4 | Wi-Fiは今すぐ/BLE・802.15.4はunstable/Threadスタックは概念のみ | [第11部10ページ](/embassy-esp32-c6/part11/10-thread-zigbee/) |
| 19 | Deep Sleep区画設計 | 電気を残す場所を選ぶ | unstable（第12部で体験済み） | [第12部2ページ](/embassy-esp32-c6/part12/02-deep-sleep/) |
| 20 | TWAI×2 | CAN準拠×2コントローラ | unstable（第8部で体験済み） | [第8部9ページ](/embassy-esp32-c6/part08/09-twai-basics/) |

## 深淵の歩き方

この表は2026年7月時点のスナップショットです。esp-halは開発が活発なので、「概念のみ」の行は年単位で減っていくでしょう。だからこそ、表の暗記より**調べる手順**を持ち帰ってください。

1. **ハードにあるか** — TRM（Technical Reference Manual）とデータシートを見る。チップの能力の上限はここで決まります
2. **ESP-IDFにあるか** — 公式C環境のドキュメントを見る。ハード機能の使い方と実例が最も充実しています
3. **Rustで書けるか** — docs.espressif.com/projects/rust のesp-hal/esp-radioのdocsを、**自分が固定したバージョン**で見る。unstableかどうかも確認します

この3段を混同しなければ、「ネットの記事に載っていたコードが動かない」「APIが見つからない」で迷子になることはありません。

## よくある失敗

- **総括表を「永遠の真実」として覚えてしまう** — ライブラリの対応状況は変わります。使う前に、固定したバージョンのdocsで再確認してください
- **「概念のみ」を「使えない」と読んでしまう** — ESP-IDFに切り替えれば今日から使えますし、esp-halへの実装を待つ・貢献するという道もあります。「Rustからは今は書けない」というだけです

## やってみよう

総括表から「今すぐ」または「unstable」の機能を1つ選び、docs.espressif.com/projects/rust でそのモジュールのページを開いてみてください。冒頭のコード例を、対応する本編の章と見比べる——それが次の自由研究の始まりです。

## 確認問題

1. 新しい周辺機能を調べるときの3段の手順を順番に言ってください。
2. eFuseが通常のフラッシュ設定と根本的に違う点は何ですか。
3. この応用編のテーマ「CPUに全部やらせるのをやめる」に当てはまる機能を、無線・電源・GPIOの分野から1つずつ挙げてください。

<details>
<summary>答え</summary>

1. ①TRM/データシートでハード対応を確認 → ②ESP-IDFのAPIを確認 → ③自分のバージョンのesp-hal/esp-radioで書けるかを確認。
2. 一度書いたら二度と消せない（不可逆）こと。この性質がSecure BootやFlash Encryptionの土台になる。
3. 例: 無線=ESP-NOWやスニファ（フレーム処理を無線ハードが担当）、電源=LPコア（監視を専用CPUに委譲）、GPIO=PCNT/RMT/PARLIO/グリッチフィルタなど（計数・波形・転送・ノイズ除去をハードが担当）。

</details>

## まとめ

- 図鑑の残り: USB Serial/JTAG（unstable+probe-rs）、eFuse（読み取りは今すぐ・焼くのは概念のみ）、Wi-Fiスニファ/CSI（esp-radio 0.18のfeatureで実在）。ESP-NOW・無線同居・Deep Sleep区画・TWAIは既存章で体験済み
- 20機能の総括表は「ハード対応・IDF対応・Rust対応」の三段を区別するための地図。地図は更新されるので、調べる手順のほうを持ち帰る
- ESP32-C6を高性能Arduinoとして消費しないでください。CPUに全部やらせるのをやめ、専用ハードウェアへ仕事を分担させること——それが組み込みの深淵へ入る最初の一歩です

## 次のページ

図鑑はここで閉じます。本教材に登場した用語を整理した用語集で、学んだ言葉を確かめてください。

- 前: [9. 自作バスと専用命令 — PARLIO・Dedicated GPIO・SDM](/embassy-esp32-c6/deep-dive/09-bus-and-bits/)
- 次: [用語集](/embassy-esp32-c6/appendix/glossary/)
