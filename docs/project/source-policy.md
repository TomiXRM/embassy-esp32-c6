---
title: 情報源ポリシー
description: 教材執筆で参照する情報源の優先順位と引用規則。
---

# 情報源ポリシー（source-policy）

## 参照優先順位

1. Espressif公式 ESP32-C6データシート（v1.5）
2. ESP32-C6 Technical Reference Manual
3. Espressif公式ハードウェア設計資料（DevKitC-1ユーザーガイド・回路図）
4. Espressif公式ESP-IDFドキュメント（C6向けページ）
5. esp-rs公式ドキュメント（docs.espressif.com/projects/rust、docs.rs）
6. esp-rs公式リポジトリと公式examples（github.com/esp-rs/esp-hal）
7. Embassy BookとEmbassy公式API資料（embassy.dev、docs.rs）
8. The Rust Programming Language（公式）
9. The Embedded Rust Book
10. embedded-hal公式資料

## 規則

- 個人ブログは調査の入口としてのみ利用可。**ピン番号・API・対応機能・電力特性・通信仕様の根拠にしない**
- ピン番号は公式回路図またはボードユーザーガイドで確認（→ docs/research/esp32c6-hardware.md に集約済み）
- APIはdocs.rsの該当バージョン、または公式examplesのコードで確認。**推測で書かない**
- 公式資料の文章を大量にコピー・翻訳しない。本文は独自に書き、frontmatterの`sources`に参照元URLを載せる
- 電流値等の数値は「データシート典型値」と明記し、実測値と混同しない

## 資料間の相違の記録

| 相違 | 採用 | 理由 | 確認日 |
|---|---|---|---|
| esp-generateテンプレートは`[unstable] build-std`を含むが、stable Rustではこのテーブルは無視される | build-std行を省略し、rustupのプリビルドcore/allocを使用 | stableツールチェーンで確実にビルドできることを優先。riscv32imac-unknown-none-elfはTier 2でプリビルド提供あり | 2026-07-18 |
| trouble-host最新は0.7だがesp-radio 0.18はbt-hci ^0.8実装 | trouble-host 0.6.0を採用 | 公式examples（esp-radio-v1.0.0-beta.0タグのble/bas_peripheral）が0.6.0を使用。0.7はbt-hci 0.9要求で組めない | 2026-07-18 |
| embassy-sync最新は0.8だが公式BLE例は0.7 | 0.7系に統一 | trouble-host 0.6との互換。公式例のコメント「TODO: update once trouble supports 0.8」に従う | 2026-07-18 |
| esp-hal-embassy（旧統合）とesp-rtos（新統合） | esp-rtos 0.3.0 | esp-hal-embassyは0.9.1で凍結されmonorepoから削除済み。esp-generate現行テンプレートがesp-rtosを採用 | 2026-07-18 |
| SN65HVD230はESP-IDFのC6 TWAIページに明記なし（TJA105xのみ例示） | 配線例はTJA105x系を主、SN65HVD230は「よく使われる3.3V品」として言及に留める | 公式例示を優先 | 2026-07-18 |
