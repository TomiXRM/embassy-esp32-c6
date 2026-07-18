---
title: "1. Light Sleep"
description: ESP32-C6の4つの電力モードを「CPU・RAM・無線・復帰要因」で整理し、Light-sleepで止まる範囲と残る範囲を学びます。
part: 12
lesson: 1
difficulty: advanced
estimated_minutes: 15
prerequisites:
  - part09/06-embassy-time
  - part11/07-espnow-basics
hardware:
  - ESP32-C6-DevKitC-1
status: drafted
code_status: concept-only
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://documentation.espressif.com/esp32-c6_datasheet_en.pdf
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/system/sleep_modes.html
---

## このページでできるようになること

- Active / Modem-sleep / Light-sleep / Deep-sleep を「CPUが止まる範囲・RAM保持・無線・復帰要因」で区別して説明できる
- Light-sleepで止まるもの・残るものを言える
- 「awaitで待つこと」と「電力モードを下げること」が別物だと説明できる

## 先に結論

省電力は「sleepという関数を呼べば達成できる」ものではありません。ESP32-C6には深さの違う電力モードがあり、モードごとに「止まる回路」「残る記憶」「使える復帰要因」が異なります。Light-sleepはCPUと大半のペリフェラル（周辺機器）のクロックを止め、データシート典型値で数十〜数百µAまで消費を下げつつ、HP SRAM（メインメモリ）は保持します。だから復帰後は**眠る直前の続きから**実行できます。ただしスリープ中は無線の受信もできません。「いつ眠り、何をきっかけに起きるか」を決める設計こそが省電力の本体です。

## 身近なたとえ

Light-sleepは、授業の合間に机に突っ伏してうたた寝するようなものです。チャイム（タイマー）や、肩をたたかれる（GPIOの変化）と目を覚まし、ノート（HP SRAM）はそのまま机の上に残っているので、授業の続きからすぐ再開できます。

ただし実際のLight-sleepは、たとえと違って**寝ている間は耳も閉じます**。無線パケットが届いても聞こえません。受信回路も一緒に止まっているからです。

## 仕組み

### 4つの電力モード

ESP32-C6の電力モードを一覧にします。電流値はすべて**ESP32-C6データシート v1.5（Table 5-7〜5-11）に記載の典型値**で、チップ単体の値です。

| モード | CPU | HP SRAM | 無線 | 代表電流（データシート典型値） | 主な復帰要因 |
|---|---|---|---|---|---|
| Active | 動作 | 保持 | 送受信可 | Wi-Fi送信時 252〜354mA、Wi-Fi受信時 78〜82mA | （動作中なので不要） |
| Modem-sleep | 動作 | 保持 | RF回路を間欠的にoff | 27mA（160MHz動作、周辺クロックoff） | （CPUは起きている） |
| Light-sleep | 停止（クロック停止） | **保持** | 停止（送受信とも不可） | 180µA（周辺電源on）/ 35µA（周辺電源off） | タイマー、GPIOレベル、UART受信、LPコア |
| Deep-sleep | 停止（電源off） | **消える**（LP SRAM 16KBのみ保持） | 停止 | 7µA（RTCタイマー+LPメモリon） | タイマー、EXT1（GPIO0〜7）、LPコア/LP UART |

重要な注意が2つあります。

- この値は**チップ単体**の典型値です。開発ボード（ESP32-C6-DevKitC-1）にはUSB-UARTブリッジ（CP2102N）や電源表示LED、レギュレータが載っていて常時電力を消費するため、ボード全体の電流は**この表の通りにはなりません**
- 私たちはこの値を実測していません。「データシートにこう書いてある」以上のことは、このページでは主張しません（測り方は[4. 消費電力の測り方](/embassy-esp32-c6/part12/04-power-measurement/)で扱います）

### Light-sleepで止まるもの・残るもの

- **止まる**: HP CPU、大半のペリフェラルのクロック、無線（Wi-Fi・BLE（Bluetooth Low Energy）とも送受信不可）
- **残る**: HP SRAMの内容とCPUの状態。だから復帰後はスリープに入った箇所の**続きから**実行される
- **復帰要因**（ESP-IDFのSleep Modesドキュメントより）: RTCタイマー、GPIOレベル、UART受信、LPコアなど。Deep-sleepより選択肢が多いのがLight-sleepの利点です

### awaitで待っても電力モードは変わらない

第9部で学んだ`Timer::after(...).await`は、**そのtaskを**待ち状態にして他のtaskへCPUを譲る仕組みです。チップの電力モードを切り替える操作ではないので、awaitで待っているだけではLight-sleepの電流にはなりません。「taskが待つこと」と「チップが眠ること」は別の階層の話です。

esp-halでは`Rtc`（`rtc_cntl`モジュール、unstable API）がスリープの入口です。Light-sleep用のGPIO復帰要因として`GpioWakeupSource`がありますが、**これはLight-sleep専用**で、Deep-sleepには使えません（詳しくは[3. Wake-upの設計](/embassy-esp32-c6/part12/03-wakeup/)）。なお本教材のexamplesでcargo check済みなのはDeep-sleepのコード（examples/12-sleep）で、Light-sleepの完全な動作例は用意していません。このページは概念の整理が目的です。

### 無線と眠りの両立が本当の設計問題

最終プロジェクト（final-wireless-button）は500msごとにハートビートを送るので、「送信の合間の約500msはLight-sleepで眠れるのでは？」と考えたくなります。しかし眠っている間は**受信もできない**ため、受信側からのACKや通知を取りこぼします。プロジェクトのpowerモジュールには、この論点がコメントとして残してあります（これは抜粋です。完全なコードはexamples/final-wireless-button/src/power.rsを見てください）。

```rust
/// スリープ実装を有効にしているか（現状は常にfalse）
pub const SLEEP_ENABLED: bool = false;

/// 「ここからしばらく待ちに入る」直前に呼ぶフック。
/// 現在は何もしない。実装するならここで
/// 「次のハートビートまでの時間を計算 → light sleepに入る」処理を置く。
pub fn before_idle() {
    // 何もしない（設計上のフックのみ）
}
```

「どこで眠れるか」をアプリの構造として先に用意しておき、眠る・眠らないは設計判断として分離する——これが実用的な省電力設計の入口です。

## よくある失敗

1. **awaitで待てば省電力になっていると思い込む** — awaitはtask切り替えの仕組みで、電力モードは変わりません。モードを下げるには明示的な操作が必要です
2. **Light-sleep中にESP-NOWやBLEの受信を期待する** — 受信回路も止まっています。「起きている時間帯」を送信側・受信側で設計して揃える必要があります
3. **開発ボードで35µAが出ると期待する** — USB-UARTブリッジや電源LEDが常時消費します。電池で長く動かすには、ボード側の回路まで含めた検討が必要です

## やってみよう

examples/final-wireless-button/src/heartbeat.rsを開き、`power::before_idle()`が呼ばれる場所を見つけてください。「もしここでLight-sleepに入ったら、どのパケットを取りこぼす可能性があるか」を、コードのコメントとして書き出してみましょう（ヒント: 送信側が受け取るパケットは1種類だけです）。

## 確認問題

1. Modem-sleepとLight-sleepの一番大きな違いは何ですか。
2. Light-sleepから復帰したプログラムは、どこから実行を再開しますか。それはなぜですか。
3. 「sleepを呼べば省電力」という説明がなぜ不十分なのか、2つ理由を挙げてください。

<details>
<summary>答え</summary>

1. CPUが動き続けるかどうかです。Modem-sleepはCPUが動いたまま無線RF回路を間欠的に止めるモード（典型27mA）、Light-sleepはCPUのクロックも止めるモード（典型180µA/35µA）です。
2. スリープに入った箇所の続きからです。HP SRAM（メインメモリ）とCPUの状態が保持されているからです。
3. （例）モードによって止まる範囲・残る記憶・使える復帰要因が全く違うので、どのモードにどう入るかを選ぶ必要があるから。また、眠っている間は受信もできないため、通信の設計（いつ起きるか）とセットで考えないと機能が壊れるからです。

</details>

## まとめ

- 電力モードは4段階。「CPU・RAM・無線・復帰要因」のセットで覚える（電流値はデータシート典型値であり、開発ボードではその通りにならない）
- Light-sleepはCPU停止・HP SRAM保持・無線停止。復帰後は続きから再開できる
- awaitは省電力モードではない。省電力は関数呼び出しではなく「いつ眠り、いつ起きるか」の設計

## 次のページ

さらに深く眠るDeep-sleepでは、メインメモリの内容ごと消えます。「再起動と何が違うのか」を、cargo check済みのコードで確かめます。

[2. Deep Sleep →](/embassy-esp32-c6/part12/02-deep-sleep/)

---

前: [第11部 10. Thread/ZigbeeとRust対応状況](/embassy-esp32-c6/part11/10-thread-zigbee/) | 次: [2. Deep Sleep](/embassy-esp32-c6/part12/02-deep-sleep/)
