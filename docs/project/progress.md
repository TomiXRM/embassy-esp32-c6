---
title: 進捗管理
description: 章ごとの執筆状態とサンプルコードの検証状態。
---

# 進捗管理（progress）

状態: planned → outlined → drafted → reviewed/complete。コードは cargo-check-passed → hardware-tested。

最終更新: 2026-07-18

## 全体集計

- 教材ページ: **120 / 120 作成済み**（completeが98、draftedが22、outlined/plannedは0）
- 付録: 3 / 3 complete（用語集・Arduino対応表・トラブルシューティング）
- サンプル: **14 / 14 が cargo check 通過**（実機確認は0 — 実機なしのため）
- サイトビルド: 133ページ生成成功、内部リンク切れ0

## 部ごとの状態

| 部 | complete | drafted | 備考 |
|---|---|---|---|
| 第1部 ESP32-C6と開発環境 | 10 | 0 | |
| 第2部 Rustの最初の一歩 | 10 | 0 | 全スニペット実コンパイル検証 |
| 第3部 Rustらしいデータの扱い | 10 | 0 | エラー実文面はrustc 1.97.1の実出力 |
| 第4部 大きなプログラムの作り方 | 10 | 0 | |
| 第5部 組み込みRustの基礎 | 4 | 6 | 04〜08,10はスニペット単体検証が残る |
| 第6部 GPIO・割り込み・時間 | 9 | 1 | 10-watchdogは概念のみ（WDT実コード非掲載） |
| 第7部 アナログと波形制御 | 7 | 3 | サーボ・移動平均コードが未検証 |
| 第8部 UART・I2C・SPI・TWAI | 9 | 1 | 08-bus-sharingは概念のみ |
| 第9部 Embassy | 9 | 1 | 03-futureはコード無し概念ページ |
| 第10部 Wi-Fi | 7 | 3 | AP/UDP/MQTTは概念のみ |
| 第11部 BLE・ESP-NOW・802.15.4 | 7 | 3 | Central/802.15.4/Thread-Zigbeeは概念のみ |
| 第12部 実用設計・最終プロジェクト | 6 | 4 | Light-sleep実コード・電力実測・Flash保存が未検証 |
| 付録 | 3 | 0 | |
| 応用編1 キーボード（10ページ） | 10 | 0 | examples 14/15対応 |
| 応用編2 センサ端末（10ページ） | 10 | 0 | examples 16/17対応 |
| 応用編3 ロボットファーム（12ページ） | 12 | 0 | luhsoccer_firmware読解（MIT） |
| 応用編4 深淵・キモい機能図鑑（10ページ） | 10 | 0 | examples 18〜21対応、全機能にRust対応状況バッジ |

drafted 22ページの内訳と次の一手はFINAL_REPORT.mdの優先順位を参照。

## examplesの状態

| example | 状態 | 実機（XIAO ESP32-C6, probe-rs, 2026-07-19） |
|---|---|---|
| 01-blinky | cargo-check-passed | **hardware-tested**: 書込・起動・defmtログOK |
| 02-button | cargo-check-passed | flashes+boots（ボタン押下は未操作） |
| 03-uart | cargo-check-passed | flashes+boots（ドライバ動作・ループバック用ジャンパ無し→タイムアウト） |
| 04-i2c | cargo-check-passed | **hardware-tested**: I2C初期化＋バススキャン動作（センサ無しで未検出） |
| 05-spi | cargo-check-passed | flashes+boots（転送動作・ジャンパ無し→NG検出） |
| 06-embassy-tasks | cargo-check-passed | **hardware-tested**: マルチタスク・カウンタ・ハートビート動作 |
| 07-channel | cargo-check-passed | **hardware-tested**: task間通信＋with_timeout動作 |
| 08-wifi | cargo-check-passed | **hardware-tested**: Wi-Fiスタック起動＋AP探索（認証情報無しで接続失敗は正常） |
| 09-ble | cargo-check-passed | **hardware-tested**: BLEスタック起動＋「C6-BUTTON」アドバタイズ |
| 10-esp-now | cargo-check-passed | **hardware-tested**: ブロードキャスト送信成功（Ok）、自MAC取得 |
| 11-twai | cargo-check-passed | flashes+boots（TWAI初期化・送受信はGPIO2↔GPIO3ジャンパ必須で未配線） |
| 12-sleep | cargo-check-passed | **hardware-tested**: リセット/復帰要因取得＋Deep Sleep突入（USB電断で確認） |
| 13-adc-pwm | cargo-check-passed | **hardware-tested**: ADC実測値変動＋PWMデューティ算出 |
| final-wireless-button | cargo-check-passed（protocolはホストテスト10/10成功） | 未フラッシュ（2台目必要。構成要素のesp-now/マルチタスクは実機確認済み） |
| 14-keymatrix | cargo-check-passed | flashes+boots（スキャン動作・ボタン未配線） |
| 15-ble-hid | cargo-check-passed | **hardware-tested**: HID GATT起動＋「C6-KEYBOARD」アドバタイズ（ペアリングはSMP未実装） |
| 16-sensor-node | cargo-check-passed | **hardware-tested**: RTC RAM起動回数復元＋センサ不在を劣化運転で継続 |
| 17-https | cargo-check-passed | 未フラッシュ（認証情報要。同スタックのwifiは実機確認済み） |
| 18-rmt-ws2812 | cargo-check-passed | flashes+boots（RMT動作・XIAOはGPIO8にWS2812非搭載で不可視） |
| 19-pcnt | cargo-check-passed | flashes+boots（PCNT初期化・計数はGPIO10↔GPIO18ジャンパ必須で0） |
| 20-etm | cargo-check-passed | **hardware-tested**: ETM結線動作（CPU非介入、panicなし） |
| 21-ledc-fade | cargo-check-passed | **hardware-tested**: HWフェード動作＋フェード中もCPU別処理 |

2026-07-19、実機（XIAO ESP32-C6、probe-rs + defmt/RTT）で20/22 exampleを書き込み・起動確認。うち14例は挙動までdefmtログで確認（hardware-tested）。残りは外付け部品/2台目/認証情報/ジャンパが必要なため単体では機能未確認（起動・panic無しは確認済み）。final-wireless-buttonとhttpsは未フラッシュだが構成要素（esp-now/wifi/マルチタスク/protocolホストテスト）は確認済み。

注: 基準ボードはESP32-C6-DevKitC-1だが実機はXIAO ESP32-C6。オンボードLED位置等が異なる（XIAOはGPIO8にWS2812非搭載）ため、LED可視系（18-rmt-ws2812等）は実機では不可視。
