---
title: バージョン固定表
description: 教材全体で使用するツールチェーンとクレートのバージョン。全コード・全ページはこの表に従う。
---

# バージョン固定表（versions）

確認日: **2026-07-18**。全執筆者・全サンプルコードはこの表のバージョンのAPIのみを使うこと。

## ツールチェーン・環境

| 項目 | 値 |
|---|---|
| Rust toolchain | stable 1.97.1（esp-hal 1.1.1のMSRVは1.88.0、テンプレート要求は1.95） |
| target | riscv32imac-unknown-none-elf |
| edition | 2024 |
| 書き込みツール | espflash 4.5.0 |
| サイト生成 | Astro 7.1.1 + Starlight 0.41.3 + astro-mermaid 2.1.0 |
| Node.js | v24.18.0 |
| 動作確認OS | macOS (Darwin 25.2.0, Apple Silicon) |
| 対象ボード | ESP32-C6-DevKitC-1（ESP32-C6-WROOM-1、8MBフラッシュ） |

## Rustクレート（examples/のCargo.tomlと一致させる）

| クレート | バージョン | features（esp32c6向け） | 安定性 |
|---|---|---|---|
| esp-hal | ~1.1.0 | esp32c6, unstable, log-04 | 1.x安定（unstable API使用箇所は下記） |
| esp-rtos | 0.3.0 | esp32c6, embassy, esp-radio※, esp-alloc※, log-04 | 0.x |
| esp-radio | 0.18.0 | esp32c6, wifi, ble※, esp-now※, unstable※, log-04 | wifiはstable feature、他はunstable |
| esp-bootloader-esp-idf | 0.5.0 | esp32c6, log-04 | - |
| esp-println | 0.17.0 | esp32c6, log-04 | - |
| esp-backtrace | 0.19.0 | esp32c6, panic-handler, println | - |
| esp-alloc | 0.10.0 | - | 無線使用時のみ |
| embassy-executor | 0.10.0 | - | - |
| embassy-time | 0.5 | - | - |
| embassy-sync | 0.7 | - | trouble-host 0.6との互換のため0.8ではなく0.7 |
| embassy-futures | 0.1 | - | - |
| embassy-net | 0.9.1 | tcp, udp, dhcpv4, medium-ethernet, dns | - |
| trouble-host | 0.6.0 | gatt, derive | 0.7はbt-hci 0.9要求のため**使用禁止** |
| bt-hci | 0.8.0 | - | - |
| embedded-hal | 1.0.0 | - | - |
| embedded-hal-async | 1.0.0 | - | - |
| embedded-io | 0.7.1 | - | - |
| embedded-io-async | 0.7.0 | - | - |
| heapless | 0.9 | - | - |
| static_cell | 2.1 | - | - |
| log | 0.4 | - | - |

※印は該当機能を使うexampleのみ有効化。

## unstable APIを使う理由と壊れうる箇所

esp-hal 1.1系で以下のモジュールは`unstable` feature配下にあり、**semver保証がない**（マイナー更新でAPIが変わりうる）:

- analog(ADC), ledc, mcpwm, twai, rtc_cntl(sleep), timer, dma, usb_serial_jtag, delay

教材でADC・PWM・TWAI・sleep・Embassy時刻ドライバを扱うために必要。esp-halを更新する場合はこれらの章のコードとexamplesを必ず再検証すること。esp-radioのble/esp-now/802.15.4も同様にunstable。

## 世代混在の禁止

- 旧世代（esp-wifi 0.15以前、esp-hal-embassy、esp-hal 0.2x系）のAPI・記事・コードを教材へ持ち込まない
- embassy-executorのAPIは0.10系（`Spawner::spawn`はSpawnTokenを受け取り不可失敗はtoken生成時に返る）で統一
