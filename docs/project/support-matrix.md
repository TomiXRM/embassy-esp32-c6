---
title: 技術対応状況表
description: ESP32-C6のハードウェア対応とRust（esp-hal 1.1系）ライブラリ対応の対応状況一覧。
---

# 技術対応状況表（support-matrix）

確認日: 2026-07-18。使用構成は[バージョン固定表](./versions.md)参照（esp-hal 1.1.1 / esp-rtos 0.3.0 / esp-radio 0.18.0）。

状態の定義:
- **公式に安定対応** — esp-halのstable API（semver保証あり）
- **unstable API** — esp-halの`unstable` feature配下（動くがsemver保証なし）
- **実験的** — 公式にexperimental扱い、またはドライバが低レベルのみ
- **ビルド確認** — 本教材のexamplesでcargo checkが通ることを確認
- **実機確認済み** — 本教材の作業で実機動作を確認（今回の制作では実機なし）
- **概念説明のみ** — コードは扱わず仕組みの説明のみ
- **教材対象外** — 本教材では扱わない

「C6がハードウェアとして対応」と「現在のRustライブラリで実用的に扱える」は別物として列を分けている。

| 分野 | ESP32-C6ハード対応 | Rust HAL対応 | 非同期対応 | ビルド確認 | 実機確認 | 教材での扱い | 注意点 |
|---|---|---|---|---|---|---|---|
| GPIO入力 | ○ | 公式に安定対応 (gpio::Input) | ○ wait_for系 | 済 (02) | 未 | 第6部で実装 | 内蔵プル約45kΩ |
| GPIO出力 | ○ | 公式に安定対応 (gpio::Output) | -（同期で十分） | 済 (01) | 未 | 第6部で実装 | 駆動能力 典型40mA/28mA |
| GPIO割り込み | ○ | 公式に安定対応（async wait推奨） | ○ | 済 (02) | 未 | 第6部で実装 | 教材はasync waitを主とする |
| Timer | ○ (systimer, TIMG×2) | unstable API (timer) | ○ embassy-time経由 | 済 (06) | 未 | 第6部・第9部で実装 | esp-rtosが時刻ドライバ提供 |
| Watchdog | ○ (WDT×3) | unstable API (rtc_cntl等) | - | 未 | 未 | 第6部で概念+最小コード | |
| PWM (LEDC) | ○ 6ch | unstable API (ledc) | - | 済 (13) | 未 | 第7部で実装 | C6はMCPWMも搭載(unstable)。教材はLEDC主 |
| ADC | ○ ADC1のみ 12bit 7ch | unstable API (analog::adc) | ○ into_async | 済 (13) | 未 | 第7部で実装 | GPIO0–6のみ。ADC2なし。校正API別途 |
| UART | ○ HP×2+LP×1 | 公式に安定対応 (uart) | ○ | 済 (03) | 未 | 第8部で実装 | UART0はコンソール兼用(GPIO16/17) |
| I2C | ○ HP×1+LP×1 | 公式に安定対応 (i2c) | ○ | 済 (04) | 未 | 第8部で実装 | HP I2Cは1つのみ |
| SPI | ○ SPI2(汎用) | 公式に安定対応 (spi) | ○ | 済 (05) | 未 | 第8部で実装 | 汎用はSPI2の1系統 |
| DMA | ○ GDMA | unstable API (dma) | - | 未 | 未 | 概念説明のみ | |
| TWAI | ○ 2コントローラ | unstable API (twai) | ○ transmit/receive_async | 済 (11) | 未 | 第8部で実装 | **外付けトランシーバ必須** |
| USB Serial/JTAG | ○ | unstable API (usb_serial_jtag) | ○ | 未 | 未 | 第1部で書き込み手段として説明 | GPIO12/13固定 |
| Wi-Fi Station | ○ Wi-Fi 6 (2.4GHz) | esp-radio 0.18 (wifiはstable feature) | ○ embassy-net | 済 (08) | 未 | 第10部で実装 | 2.4GHzのみ |
| Wi-Fi Access Point | ○ | esp-radio対応 | ○ | 未 | 未 | 概念説明のみ | 教材はSTA主 |
| TCP | -（ソフトウェア） | embassy-net 0.9 | ○ | 済 (08) | 未 | 第10部で実装 | |
| UDP | -（ソフトウェア） | embassy-net 0.9 | ○ | 未 | 未 | 第10部で解説+コード断片 | |
| HTTP | -（ソフトウェア） | TcpSocket上に手書き/reqwless | ○ | 済 (08) | 未 | 第10部で実装（GET） | 「Wi-Fiがあれば自動で使える」ものではない |
| MQTT | -（ソフトウェア） | 成熟したno_stdクレートは限定的 | - | 未 | 未 | 概念説明のみ | ブローカー必要 |
| BLE Advertising | ○ BLE 5.3 | esp-radio ble (unstable) + trouble-host 0.6 | ○ | 済 (09) | 未 | 第11部で実装 | Bluetooth Classicは非対応 |
| BLE Peripheral | ○ | trouble-host (GATT) | ○ | 済 (09) | 未 | 第11部で実装 | |
| BLE Central | ○ | trouble-host (scan/central) | ○ | 未 | 未 | 概念説明のみ | 教材はPeripheral主 |
| ESP-NOW | ○ | esp-radio esp-now (unstable) | ○ | 済 (10) | 未 | 第11部・最終プロジェクトで実装 | |
| IEEE 802.15.4 | ○ 802.15.4-2015 | esp-radio低レベルドライバ (実験的) | - | 未 | 未 | 概念説明のみ | フレーム送受信レベル |
| Thread | ○ Thread 1.3 | Rustスタックなし（openthreadバインディングは教材対象外） | - | 未 | 未 | 概念説明のみ | |
| Zigbee | ○ Zigbee 3.0 | Rustスタックなし | - | 未 | 未 | 概念説明のみ | |
| Light Sleep | ○ 典型180µA/35µA | unstable API (rtc_cntl, support_status=partial) | - | 済 (12) | 未 | 第12部で実装 | 実測なし。データシート値のみ提示 |
| Deep Sleep | ○ 典型7µA | unstable API (Rtc::sleep_deep) | - | 済 (12) | 未 | 第12部で実装 | HP SRAM非保持=再起動 |
| Wake-up source | ○ timer/GPIO/LP | TimerWakeupSource, GpioWakeupSource | - | 済 (12) | 未 | 第12部で実装 | Deep sleepのGPIO起床はLP GPIO(0–7) |
| Flash | ○ 8MB (WROOM-1) | esp-storage等は教材対象外 | - | 未 | 未 | 概念説明のみ | |
| 不揮発ストレージ | ○ | 成熟した定番なし（sequential-storage等は発展） | - | 未 | 未 | 概念説明のみ | |
| OTA | ○（ブートローダ対応） | 教材対象外 | - | 未 | 未 | 教材対象外 | |
| 乱数 | ○ TRNG | 公式に安定対応 (rng) | - | 済 (08) | 未 | Wi-Fi章でseedに使用 | |
| 暗号関連機能 | ○ SHA/AES/RSA/ECC等 | unstable API (sha, aes) | - | 未 | 未 | 教材対象外 | |

括弧内の数字は対応するexample番号（examples/参照）。「ビルド確認 済」は本リポジトリでのcargo check成功を意味し、実機動作の保証ではない。
