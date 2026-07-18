---
title: "1. 用語集"
description: この教材に登場する重要な用語を、カテゴリ別に中学生でも分かる言葉で説明します。
status: complete
---

この教材に出てくる重要な用語をカテゴリ別にまとめました。本文を読んでいて「この言葉なんだっけ」となったら、ここへ戻ってきてください。各用語には、くわしく学べる章へのリンクを付けています。

## ハードウェアの基礎

### MCU（Microcontroller Unit / マイコン）

CPU・メモリ・入出力機能を1つのチップに収めた小さなコンピュータです。パソコンと違ってOSを持たず、書き込んだ1つのプログラムだけを電源が入っている間ずっと実行し続けます。家電やおもちゃ、車の中など、身の回りのあらゆる場所で働いています。

→ 関連: [マイコンと普通のパソコンの違い](/embassy-esp32-c6/part01/04-mcu-vs-pc/)

### SoC（System on a Chip / システムオンチップ）

CPUだけでなく、無線機能やメモリなど「システム一式」を1枚のチップにまとめたものです。ESP32-C6はRISC-VのCPUに加えてWi-Fi 6、BLE（Bluetooth Low Energy）、IEEE 802.15.4の無線までも1チップに収めたSoCです。MCUよりも多機能なチップを指すときによく使われる言葉です。

→ 関連: [ESP32-C6とは何か](/embassy-esp32-c6/part01/03-what-is-c6/)

### GPIO（General Purpose Input/Output / 汎用入出力）

マイコンの「足（ピン）」を、プログラムから自由に入力・出力として使える機能です。出力にすればLEDを点けたり消したりでき、入力にすればボタンが押されたかを読み取れます。ESP32-C6-DevKitC-1ではGPIO8にWS2812B（RGB LED）、GPIO9にBOOTボタンがつながっています。

→ 関連: [GPIO出力](/embassy-esp32-c6/part06/01-gpio-output/)

### ペリフェラル（peripheral / 周辺機器）

CPUのまわりに置かれた「専門の係」のことです。UARTやI2C、タイマー、ADCなどがペリフェラルで、CPUの代わりに通信や計測の仕事を引き受けてくれます。RustではペリフェラルをHAL（後述）を通して安全に操作します。

→ 関連: [ESP32-C6とは何か](/embassy-esp32-c6/part01/03-what-is-c6/)、[HAL — esp-hal](/embassy-esp32-c6/part05/09-hal/)

### レジスタ（register）

ペリフェラルを操作するための「スイッチ盤」にあたる、特別なメモリ番地です。特定の番地に決まった値を書くとピンがHighになる、といったようにハードウェアと直結しています。ふだんはHALが代わりに読み書きしてくれるので、直接さわる機会は多くありません。

→ 関連: [PACとレジスタとunsafe](/embassy-esp32-c6/part05/08-pac/)

### 割り込み（interrupt）

「イベントが起きたらCPUに知らせてもらう」仕組みです。ボタンが押された瞬間にハードウェアがCPUへ合図を送り、CPUは今の仕事を一時中断してその処理へ飛びます。Embassyではこの仕組みが裏側で使われていて、私たちは`wait_for_falling_edge().await`のような読みやすい形で利用できます。

→ 関連: [GPIO割り込みとasync wait](/embassy-esp32-c6/part06/06-gpio-interrupt/)

### ポーリング（polling）

割り込みとは逆に、「変化がないかを自分から何度も見に行く」方式です。ボタンの状態をループでずっと読み続けるのがポーリングです。単純で分かりやすい一方、CPUが確認作業に付きっきりになるという弱点があります。

→ 関連: [ボタンを読む](/embassy-esp32-c6/part06/04-button/)、[同期処理と非同期処理](/embassy-esp32-c6/part09/01-sync-vs-async/)

### ドライバ（driver）

センサやディスプレイなど、特定の部品を動かすためのプログラム部品です。Rustの組み込み開発では、embedded-halという共通のtrait（後述）に合わせてドライバが書かれているため、同じドライバをさまざまなマイコンで使い回せます。

→ 関連: [embedded-hal](/embassy-esp32-c6/part05/10-embedded-hal/)

### PAC（Peripheral Access Crate / ペリフェラルアクセスクレート）

チップのレジスタをRustのコードから読み書きできるようにした、いちばん低いレベルのライブラリです。チップの設計データから自動生成されます。強力ですが間違った使い方も書けてしまうため、ふだんはこの上に作られたHALを使います。

→ 関連: [PACとレジスタとunsafe](/embassy-esp32-c6/part05/08-pac/)

### HAL（Hardware Abstraction Layer / ハードウェア抽象化層）

レジスタ操作の細かい手順を隠して、「ピンをHighにする」「1バイト送る」のような分かりやすい形でペリフェラルを使わせてくれる層です。この教材ではesp-halを使います。HALのおかげで、レジスタの番地を覚えなくても安全にハードウェアを動かせます。

→ 関連: [HAL — esp-hal](/embassy-esp32-c6/part05/09-hal/)

## Rustの言葉

### trait（トレイト）

「この能力を持っている」という共通の約束事を定義する仕組みです。たとえば「1バイト送れる」というtraitを決めておけば、UARTでもUSBでも同じ書き方で送信できます。他の言語のinterfaceに似ていますが、あとから既存の型へ実装を追加できる点が特徴です。

→ 関連: [trait — 共通の能力を定義する](/embassy-esp32-c6/part04/05-trait/)

### 所有権（ownership）

「データの持ち主は常に1人だけ」というRustの基本ルールです。変数を別の変数や関数へ渡すと持ち主が移り（ムーブ）、元の場所からは使えなくなります。このルールのおかげで、「同じピンを2か所から同時に操作してしまう」ようなバグをコンパイルの時点で防げます。

→ 関連: [所有権 — 誰がデータを持つのか](/embassy-esp32-c6/part03/08-ownership/)

### 借用（borrow）

所有権を渡さずに、データを一時的に「貸す」仕組みです。`&`で読み取り専用として貸し、`&mut`で書き込みもできる形で貸します。「読み取りの貸し出しは同時に何件でもよいが、書き込みの貸し出しは同時に1件だけ」という規則があります。

→ 関連: [借用 — 貸し借りの規則](/embassy-esp32-c6/part03/09-borrow/)

### ライフタイム（lifetime）

「そのデータがいつまで存在するか」という期間のことです。Rustは、借りたものを持ち主より長く使おうとするコードをコンパイル時に見つけて止めます。`'static`は「プログラムが終わるまでずっと存在する」という特別なライフタイムで、Embassyのtaskへ渡すデータによく登場します。

→ 関連: [ライフタイムの直感](/embassy-esp32-c6/part03/10-lifetime/)

## 非同期処理とEmbassy

### async（非同期）

「待ち時間のあいだ、他の仕事に手を貸せる」関数の書き方です。関数に`async`を付けると、その関数は途中で一時停止・再開できる特別な形にコンパイルされます。1つのCPUでも、待ち時間をうまく融通し合うことで複数の仕事を並行して進められます。

→ 関連: [asyncとawait](/embassy-esp32-c6/part09/02-async-await/)

### await

async関数の中で「ここで結果を待つ。待っている間は他のtaskへ順番をゆずる」と宣言する書き方です。`Timer::after_millis(500).await`と書くと、500ミリ秒待つ間CPUを独り占めせず、他のtaskが動けます。

→ 関連: [asyncとawait](/embassy-esp32-c6/part09/02-async-await/)

### Future（フューチャー）

「いつか完成する結果」を表すRustの型です。async関数を呼ぶと、すぐには実行されずFutureが返ります。executor（後述）がFutureを何度も進め（poll）、完成したら結果を取り出します。「注文票」のようなもので、注文しただけでは料理は出てこず、調理係が進めてはじめて完成します。

→ 関連: [Futureの直感的説明](/embassy-esp32-c6/part09/03-future/)

### task（タスク）

Embassyにおける「並行して動く仕事の単位」です。`#[embassy_executor::task]`を付けたasync関数がtaskになり、`Spawner`で起動します。LED点滅、ボタン監視、通信などを別々のtaskに分けると、お互いを待たせずに動かせます。

→ 関連: [task — 仕事を分割する](/embassy-esp32-c6/part09/04-task/)

### executor（エグゼキュータ / 実行器）

たくさんのtaskを順番に進める「調理場の司令役」です。動けるtaskを選んで実行し、awaitで止まったら次のtaskへ切り替えます。この教材ではesp-rtosが提供するexecutorの上でEmbassyのtaskが動きます。

→ 関連: [task — 仕事を分割する](/embassy-esp32-c6/part09/04-task/)、[Spawner](/embassy-esp32-c6/part09/05-spawner/)

### Channel（チャネル）

task同士でデータを安全に受け渡すための「ベルトコンベア」です。送る側が`send`、受け取る側が`receive`を使い、どちらもasyncで待てます。入れ物のサイズが決まっているので、受け取りが追いつかないときは送信側が自然に待たされます（バックプレッシャ）。

→ 関連: [Channel・Signal・Mutex](/embassy-esp32-c6/part09/09-channel-signal-mutex/)

### Mutex（ミューテックス / 排他ロック）

複数のtaskが同じデータを使うとき、「今は私が使用中」と札を掛けて順番を守らせる仕組みです。名前はmutual exclusion（相互排他）の略です。Embassyでは`lock().await`でロックを取り、使い終わると自動で返します。

→ 関連: [Channel・Signal・Mutex](/embassy-esp32-c6/part09/09-channel-signal-mutex/)

## 有線通信

### UART（Universal Asynchronous Receiver/Transmitter / 汎用非同期送受信回路）

2本の線（TXとRX）で文字や数値をやり取りする、いちばん基本的なシリアル通信です。時計の線がない代わりに、送る側と受け取る側が同じ速さ（ボーレート）を約束しておきます。パソコンに表示されるログもUART（またはUSB Serial/JTAG）経由で届いています。

→ 関連: [UART基礎](/embassy-esp32-c6/part08/01-uart-basics/)

### I2C（Inter-Integrated Circuit / アイ・スクエアド・シー）

SDA（データ）とSCL（クロック）の2本の線に、複数の部品をぶら下げられる通信方式です。各部品はアドレス（番号）で呼び分けられ、呼ばれた部品はACKという返事を返します。温度センサなど小型センサとの通信によく使われます。

→ 関連: [I2C基礎](/embassy-esp32-c6/part08/03-i2c-basics/)

### SPI（Serial Peripheral Interface / シリアルペリフェラルインタフェース）

MOSI・MISO・SCK・CSの4本の線で高速にデータをやり取りする通信方式です。I2Cより線は多いですが速度を上げやすく、ディスプレイやSDカードによく使われます。クロックの極性とタイミングの組み合わせで「モード0〜3」があり、部品と合わせる必要があります。

→ 関連: [SPI基礎](/embassy-esp32-c6/part08/06-spi-basics/)

### TWAI（Two-Wire Automotive Interface / トゥーワイ）

Espressifのチップに載っている、CAN（車の中で使われるネットワーク規格）互換の通信コントローラです。ノイズに強く、複数の機器が2本の線を共有して通信します。ESP32-C6のピンをそのままCANの線につなぐことはできず、外付けのトランシーバICが必ず必要です。

→ 関連: [TWAI基礎](/embassy-esp32-c6/part08/09-twai-basics/)

## 無線とネットワーク

### Wi-Fi（ワイファイ）

無線LANの規格です。ESP32-C6はWi-Fi 6（802.11ax）に対応していますが、使える電波は2.4GHz帯だけで、5GHz帯にはつながりません。Wi-Fiは電波の層を受け持ち、その上にIPアドレスやTCP/UDPといった層が積み重なって初めてインターネット通信ができます。

→ 関連: [Wi-Fiの基礎](/embassy-esp32-c6/part10/01-wifi-basics/)

### BLE（Bluetooth Low Energy / ブルートゥース・ローエナジー）

少ない電力で小さなデータをやり取りするための無線規格です。イヤホンで使われる従来のBluetooth Classicとは別物で、ESP32-C6はBLEのみに対応しています。ボタンの状態やセンサの値など、小さな情報を省電力で届ける用途に向いています。

→ 関連: [BLEの基礎](/embassy-esp32-c6/part11/01-ble-basics/)

### ESP-NOW（イーエスピー・ナウ）

Espressif独自の無線通信方式です。Wi-Fiの電波を使いますが、ルーターへの接続もIPアドレスも不要で、相手のMACアドレスを指定して直接データを送ります。接続の手間がないぶん、送った相手に届いたかは自分で確かめる設計が必要です。

→ 関連: [ESP-NOWの基礎](/embassy-esp32-c6/part11/07-espnow-basics/)

### TCP（Transmission Control Protocol / 伝送制御プロトコル）

インターネットで使われる「確実に届ける」通信方式です。相手と接続してから送り、届いたかを確認し、順番もそろえてくれます。Webページの取得（HTTP）などはTCPの上で動いています。確実なぶん、手間と時間がかかります。

→ 関連: [TCP](/embassy-esp32-c6/part10/06-tcp/)

### UDP（User Datagram Protocol / ユーザデータグラムプロトコル）

接続なしで「投げるだけ」の通信方式です。届いたかの確認や順番の保証はありませんが、そのぶん軽くて速いのが特長です。多少データが欠けても新しい値を送り続ければよいセンサ値の配信などに向いています。

→ 関連: [UDP](/embassy-esp32-c6/part10/07-udp/)

## 省電力

### sleep（スリープ）

電池を長持ちさせるために、チップの一部を止めて消費電力を下げる状態です。ESP32-C6には主にLight-sleep（CPUを止めるがメモリは保持）とDeep-sleep（大部分を止め、HP SRAMの内容は消える）があります。「sleepを呼べば省電力」と一言では言えず、どのモードで何が止まり、何をきっかけに目覚めるかをセットで設計します。

→ 関連: [Light Sleep](/embassy-esp32-c6/part12/01-light-sleep/)、[Deep Sleep](/embassy-esp32-c6/part12/02-deep-sleep/)、[Wake-upの設計](/embassy-esp32-c6/part12/03-wakeup/)

## 関連ページ

- [Arduinoからの対応表](/embassy-esp32-c6/appendix/arduino-map/)
- [トラブルシューティング](/embassy-esp32-c6/appendix/troubleshooting/)
