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
| 書き込み・実行ツール（既定） | **probe-rs 0.29系**（`probe-rs run --chip esp32c6`、ログはdefmt/RTT） |
| 書き込みツール（代替） | espflash 4.5.0（`--features espflash` でdefmt-over-serial、`--log-format defmt`） |
| ログ | defmt 1.0.1 + rtt-target 0.6.2（probe-rs）/ esp-println 0.17 defmt-espflash（espflash） |
| サイト生成 | Astro 7.1.1 + Starlight 0.41.3 + astro-mermaid 2.1.0 |
| Node.js | v24.18.0 |
| 動作確認OS | macOS (Darwin 25.2.0, Apple Silicon) |
| 対象ボード | ESP32-C6-DevKitC-1（ESP32-C6-WROOM-1、8MBフラッシュ） |

## Rustクレート（examples/のCargo.tomlと一致させる）

| クレート | バージョン | features（esp32c6向け） | 安定性 |
|---|---|---|---|
| esp-hal | ~1.1.0 | esp32c6, unstable, defmt | 1.x安定（unstable API使用箇所は下記） |
| esp-rtos | 0.3.0 | esp32c6, embassy, esp-radio※, esp-alloc※, defmt | 0.x |
| esp-radio | 0.18.0 | esp32c6, wifi, ble※, esp-now※, unstable※, defmt | wifiはstable feature、他はunstable |
| esp-bootloader-esp-idf | 0.5.0 | esp32c6, defmt | - |
| defmt | 1.0.1 | - | ログのフォーマット層（両モード共通） |
| rtt-target | 0.6.2 | defmt | probe-rsモードのRTTシンク（optional dep、default） |
| esp-println | 0.17.0 | esp32c6, defmt-espflash | espflashモードのシリアルシンク（optional dep、`espflash` feature） |
| esp-backtrace | 0.19.0 | esp32c6, panic-handler, defmt | - |
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
- etm, pcnt, rmt, parl_io, gpio::dedicated, lp_core, i2s（応用編4で使用。詳細は docs/research/weird-features-rust.md）

教材でADC・PWM・TWAI・sleep・Embassy時刻ドライバを扱うために必要。esp-halを更新する場合はこれらの章のコードとexamplesを必ず再検証すること。esp-radioのble/esp-now/802.15.4も同様にunstable。

## ログ・書き込みツールの切替（probe-rs 既定 / espflash 代替）

各exampleは cargo feature でログの出口を切り替える。コードは常に `defmt::info!` 等。

- **既定（probe-rs）**: `default = ["probe-rs"]` → `rtt-target`(defmt) が global logger。`main` 冒頭で `rtt_target::rtt_init_defmt!()`。`.cargo/config.toml` の runner は `probe-rs run --chip=esp32c6`。
- **espflash**: `cargo run -p <crate> --no-default-features --features espflash` → `esp-println`(defmt-espflash) が global logger。runner を `espflash flash --monitor --chip esp32c6 --log-format defmt` に差し替える。
- `build.rs` は両モードで `-Tdefmt.x` を `-Tlinkall.x` の前に付与。
- defmtは精度・幅指定（`{:.2}`,`{:>5}`）非対応。floatは全精度 `{=f32}` 表示、byte列は `{=[u8]:02x}`、Format非実装型は `defmt::Display2Format`/`Debug2Format` でラップ、という書式差がある。
- **注意**: probe-rsで実機ログを見るには物理的なUSB-JTAG接続とダウンロードモードでの初回書き込みが必要。本移行はcargo checkによるコンパイル検証のみで、**実機での書き込み・ログ表示は未検証**（作業時に実機がダウンロードモードに入れられず未確認）。

## 世代混在の禁止

- 旧世代（esp-wifi 0.15以前、esp-hal-embassy、esp-hal 0.2x系）のAPI・記事・コードを教材へ持ち込まない
- embassy-executorのAPIは0.10系（`Spawner::spawn`はSpawnTokenを受け取り不可失敗はtoken生成時に返る）で統一
