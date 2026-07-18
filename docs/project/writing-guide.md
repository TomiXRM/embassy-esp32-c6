---
title: 執筆ルール
description: 全ページ共通の執筆規則。テンプレート、文体、技術的な禁止事項、frontmatter仕様。
---

# 執筆ルール（writing-guide）

すべての執筆者（サブエージェント含む）はこのルールと `versions.md` に従うこと。

## 1. ファイルとfrontmatter

- 置き場所: `site/src/content/docs/partNN/MM-slug.md`（curriculum.mdの表に従う）
- frontmatterは次の形式。**title先頭に「N. 」と章内番号を付ける**（サイドバーの並びと一致させるため番号はファイル名のMMと同じ数字）

```markdown
---
title: "10. 最初のLチカ"
description: ESP32-C6のWS2812 LEDをRustで点滅させます。
part: 1
lesson: 10
difficulty: basic          # basic | intermediate | advanced
estimated_minutes: 15
prerequisites:
  - part01/09-flash-monitor
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
status: complete           # planned | outlined | drafted | reviewed | complete
code_status: cargo-check-passed  # none | concept-only | syntax-reviewed | cargo-check-passed | hardware-tested
verified_with: "esp-hal <versions.mdの値>"
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/...
---
```

- `status` は正直に付ける。**未完成ページをcomplete扱いにしない**
- `code_status` は実際の検証結果のみ。**実機確認していないものにhardware-testedと書かない**

## 2. ページテンプレート（完全原稿）

```markdown
## このページでできるようになること
- 具体的な学習目標（2〜4個）

## 先に結論
最重要事項を3〜5文で。

## 身近なたとえ
中学生でも分かる例。**比喩の直後に、実際の技術との違いを必ず一言添える。**

## 仕組み
図（Mermaid可）や小さなコードで説明。

## Arduinoではどう書くか
必要な場合のみ。

## RustとEmbassyではどう書くか
動く最小コード。examples/の完全コードと食い違わないこと。

## コードを一行ずつ読む
重要な行だけ。

## 配線
必要な場合のみ。ピン名・GPIO番号・電圧・抵抗・注意事項。

## 実行方法
コマンドと期待される結果。

## よくある失敗
最低2件。エラーが起きる理由も書く。

## やってみよう
5分以内でできる変更課題。

## 確認問題
2〜3問。<details>で答えを畳んでよい。

## まとめ
3項目以内。

## 次のページ
次に学ぶ理由を1〜2文で。リンク付き。
```

- outlinedページは「このページでできるようになること」「先に結論」＋アウトライン（見出しのみ可）＋「次のページ」を必ず持つ
- 難しいテーマは1ページに詰め込まず分割する

## 3. 文体規則

- です・ます調で統一。子どもっぽすぎる口調にしない
- 一文を短く。一段落一話題
- 専門用語は最初に意味を説明する。略語は正式名称を一度書く（例: BLE（Bluetooth Low Energy））
- 「簡単です」で説明を省略しない
- コードは目的を説明してから見せる。突然大量に見せない
- コンパイラエラーを敵として扱わない。「なぜこの制約があるのか」を説明する
- 比喩（箱・貸し借り・鍵・順番待ち等）は使ってよいが、必ず正式な用語へ戻す

## 4. リンク規則

- サイト内リンクは**ベースパス込みの絶対パス**: `/embassy-esp32-c6/part06/04-button/`（末尾スラッシュ必須）
- 前ページ・次ページのリンクを本文末尾に置く
- 外部リンクは公式資料を優先

## 5. 技術規則（違反禁止）

- **バージョンとAPI**: `versions.md` に固定されたクレートバージョンのAPIだけを使う。APIを推測で書かない。世代の違うAPIを混在させない
- **コードの正**: `examples/` 配下のcargo check済みコードを正とし、ページ内コードはそこから抜粋する
- **Bluetooth**: ESP32-C6はBLE（Bluetooth Low Energy）のみ。Bluetooth Classicを載せない。単に「Bluetooth」と書かず「BLE」または「Bluetooth Low Energy」と書く
- **TWAI**: 外付けトランシーバ必須。C6のピンをCAN_H/CAN_Lへ直結する説明は禁止
- **無線の層**: Wi-Fi（物理・リンク）とTCP/IP、HTTP、MQTT（上位層）を一段で説明しない
- **省電力**: 「sleepを呼べば省電力」とは書かない。モード別に止まる範囲・復帰要因を書く。実測していない電流値を断定しない（データシート典型値は出典付きで可）
- **ピン番号**: docs/research/esp32c6-hardware.md（公式資料由来）に従う。主要な事実: ユーザーLED=GPIO8のWS2812B（単色LEDなし）、BOOTボタン=GPIO9、ADCはGPIO0〜6のみ、UART0=GPIO16/17、ストラッピング=GPIO4/5/8/9/15、GPIO14は存在しない
- **チップ混同禁止**: ESP32（Xtensa）/C3/C6の情報を混ぜない
- unwrapを全コードで多用しない（初期化直後など失敗が設計上ありえない箇所に限定し、理由を書く）
- Arduinoを一方的に低品質な環境として扱わない
- Rustでメモリバグが「すべて」なくなる、asyncで「自動的に」速くなる、といった過大な記述をしない
- 公式資料の文章を大量にコピー・翻訳しない。独自に書き、sourcesにURLを載せる

## 6. 用語統一

| 使う | 使わない |
|---|---|
| task（Embassyの文脈） | タスク（初出で「task（タスク）」は可） |
| BLE / Bluetooth Low Energy | Bluetooth（単独） |
| TWAI（CANとの関係は初出で説明） | CAN（C6の機能名として） |
| 所有権 / 借用 / ライフタイム | オーナーシップ等の英語カナ |
| ペリフェラル（周辺機器） | 周辺装置 |
| 書き込み（フラッシュへの） | 焼く |
| Wi-Fi | WiFi / wifi |
| ESP32-C6-DevKitC-1 | devkit等の略記（初出以降は「開発ボード」可） |

## 7. コード表示

- Rustコードブロックは ```rust。コマンドは ```bash。出力は ```text
- 断片を示すときは「これは抜粋です。完全なコードは examples/XX を見てください」と明記
- コード内コメントは日本語で最小限
