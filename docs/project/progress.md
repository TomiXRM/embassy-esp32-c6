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

drafted 22ページの内訳と次の一手はFINAL_REPORT.mdの優先順位を参照。

## examplesの状態

| example | 状態 |
|---|---|
| 01-blinky | cargo-check-passed |
| 02-button | cargo-check-passed |
| 03-uart | cargo-check-passed |
| 04-i2c | cargo-check-passed |
| 05-spi | cargo-check-passed |
| 06-embassy-tasks | cargo-check-passed |
| 07-channel | cargo-check-passed |
| 08-wifi | cargo-check-passed |
| 09-ble | cargo-check-passed |
| 10-esp-now | cargo-check-passed |
| 11-twai | cargo-check-passed |
| 12-sleep | cargo-check-passed |
| 13-adc-pwm | cargo-check-passed |
| final-wireless-button | cargo-check-passed（protocolはホストテスト10/10成功） |

実機確認（hardware-tested）は今回の制作範囲では未実施。
