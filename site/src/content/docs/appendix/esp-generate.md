---
title: "4. esp-generateでプロジェクトを作る"
description: マイコン用Rustプロジェクトを作るesp-generateの使い方。TUIの操作、各オプションの意味、この教材の構成にする選び方、ヘッドレス生成までを解説します。
status: complete
sources:
  - https://github.com/esp-rs/esp-generate
  - https://docs.espressif.com/projects/rust/book/
last_verified: "2026-07-19"
verified_with: "esp-generate 1.3.0"
---

マイコン用のRustプロジェクトは `cargo new` では作りません。**esp-generate** という専用ツールを使います。このページは、その使い方をひととおり説明する参考ページです。第1部の[8. Rustプロジェクトの作成](/embassy-esp32-c6/part01/08-new-project/)の詳しい版だと思ってください。

## なぜ cargo new ではないのか

`cargo new` が作るのは、パソコンで動く普通のプログラムのひな形です。マイコン開発では、それに加えて次のような設定が要ります。

- どのチップ向けにビルドするか（ESP32-C6など）
- どのターゲット（`riscv32imac-unknown-none-elf`）でビルドするか
- どうやって書き込み、ログを見るか（probe-rs や espflash）
- ブートローダやリンカスクリプトの指定（build.rs）

esp-generate は、これらを**質問に答えるだけで全部そろえてくれる**ツールです。答え方には2つのやり方があります。画面で選ぶ**TUI（対話モード）**と、コマンド一発の**ヘッドレスモード**です。

## インストール

```bash
cargo install esp-generate --locked
```

`--locked` は「ツール側が固定したバージョンでビルドする」という意味で、途中で依存が変わってビルドが壊れるのを防ぎます。入ったか確認します。

```bash
esp-generate --help
```

冒頭に、生成されるプロジェクトが使うクレートのバージョン（esp-hal `~1.1.0` など）が表示されます。この教材の[バージョン固定表](/embassy-esp32-c6/project/versions/)と同じ世代です。

## 使い方その1：TUI（対話モード）

プロジェクト名だけ付けて実行すると、画面（TUI = Text User Interface、文字で作られた操作画面）が開きます。

```bash
esp-generate --chip esp32c6 my-blinky
```

TUIでは次の順で進みます。

1. **オプションの選択画面**が出ます。オプションが縦に並び、いくつかは「グループ」に分かれています。
2. **矢印キー（↑↓）**で項目を移動します。
3. **スペースキー**で、そのオプションのオン・オフを切り替えます（選ぶと印が付きます）。
4. 選び終わったら**エンターキー**で確定します。生成が始まります。

:::note[グループの中は1つだけ]
「ログの出し方」や「BLEのライブラリ」など、**同じグループの中では1つしか選べません**。たとえばログは `defmt` と `log` のどちらか片方です。両方は選べません。
:::

## オプションの一覧

`esp-generate list-options` で、選べるオプションが全部見られます。1つの意味を詳しく知りたいときは `esp-generate explain <名前>`（例: `esp-generate explain probe-rs`）です。主なものを表にします。

| オプション | 意味 | 依存 |
| --- | --- | --- |
| `unstable-hal` | esp-halのunstable機能を有効化（ADC/PWM/TWAI/sleep等に必要） | - |
| `embassy` | Embassy（非同期フレームワーク）を入れる | unstable-hal |
| `alloc` | esp-allocによるヒープ確保を有効化（無線で必要） | - |
| `wifi` | esp-radioでWi-Fiを使う | alloc, unstable-hal |
| `ble-trouble` | esp-radio + TrouBLEでBLEを使う | alloc, unstable-hal, embassy |
| `ble-bleps` | 別のBLEライブラリ（bleps）を使う | alloc, unstable-hal |
| `probe-rs` | 書き込み・ログをprobe-rsで行う（espflashの代わり） | - |
| `defmt` | ログを defmt で出す | - |
| `log` | ログを log クレートで出す | probe-rsを選ばないこと |
| `panic-rtt-target` | パニック時の処理を panic-rtt-target にする | probe-rs |
| `esp-backtrace` | パニック時の処理を esp-backtrace にする | probe-rsを選ばないこと |
| `ci` | GitHub Actions（自動チェック）を付ける | - |
| `vscode` / `helix` / `neovim` / `zed` | エディタ用の設定を付ける | - |
| `embedded-test` | オンチップテスト（実機テスト）を有効化 | probe-rs |
| `wokwi` | Wokwi（オンラインシミュレータ）用の設定 | 一部チップ不可 |

`ble-trouble` と `ble-bleps` は同じ「BLEライブラリ」グループなので、選ぶならどちらか一方です。この教材は **`ble-trouble`（TrouBLE）** を使います。

:::caution[log と probe-rs は同時に選べない]
`log` は「probe-rsを選ばないこと」が条件です。probe-rsは実機のメモリをRTT経由で読むため、logクレートが流すシリアル出力を読めないからです。**probe-rsを使うならログは `defmt`** になります。この関係は[書き込みとログ表示](/embassy-esp32-c6/part01/09-flash-monitor/)の考え方と同じです。
:::

## この教材の構成にするには

この教材は **probe-rs + defmt + Embassy** を基本にしています。TUIでは次のオプションにチェックを入れてください。

- **必ず**: `unstable-hal`、`embassy`、`probe-rs`、`defmt`
- **無線を使うなら**: `alloc`、`wifi`、`ble-trouble`
- **あると便利**: `ci`、使っているエディタ（`vscode` など）

espflashで書き込みたい場合は、`probe-rs` を選ばずに、ログを `defmt`（espflashは `--log-format defmt` で読めます）または `log`、パニック処理を `esp-backtrace` にします。詳しくは[書き込みとログ表示](/embassy-esp32-c6/part01/09-flash-monitor/)を見てください。

## 使い方その2：ヘッドレス（コマンド一発）

TUIを使わず、オプションを `-o`（`--option`）で並べて一発生成できます。`--headless` を付けます。何度も同じ構成を作るときや、手順を記録に残したいときに便利です。次のコマンドは実際に動作を確認したものです。

**基本（Lチカ相当。Embassy + probe-rs + defmt）:**

```bash
esp-generate -c esp32c6 -o embassy -o unstable-hal -o probe-rs -o defmt my-blinky --headless
```

**無線あり（Wi-Fi + BLE も入れる）:**

```bash
esp-generate -c esp32c6 \
  -o embassy -o unstable-hal -o alloc \
  -o wifi -o ble-trouble \
  -o probe-rs -o defmt \
  my-wireless --headless
```

生成後は、そのフォルダで `cargo run` するだけで、ビルド→書き込み→ログ表示まで進みます（probe-rsを選んだ場合。実機とダウンロードモードについては[書き込みとログ表示](/embassy-esp32-c6/part01/09-flash-monitor/)を参照）。

## 生成されるファイルと、この教材のexamplesの関係

上の基本コマンドで作られるのは次のファイルです。第1部の[8. Rustプロジェクトの作成](/embassy-esp32-c6/part01/08-new-project/)で一つずつ説明しているものと同じです。

```text
my-blinky/
├── Cargo.toml            部品リスト（依存クレート）
├── build.rs              ビルド時の指示書（リンカスクリプト等）
├── rust-toolchain.toml   使うRustツールチェーンとターゲットの固定
├── .cargo/config.toml    ビルド先ターゲットと runner（probe-rs run …）
└── src/
    └── bin/main.rs       プログラム本体
```

実際に生成物を確認すると、この教材の `examples/` とほぼ同じ内容になります。たとえば `.cargo/config.toml` の runner は `probe-rs run --chip=esp32c6 --preverify --always-print-stacktrace --no-location`、`Cargo.toml` の esp-hal は `features = ["defmt", "esp32c6", "unstable"]` で、どちらも教材のexamplesと一致します。**この教材のexamplesは「esp-generateでこれらのオプションを選んだ結果」とほぼ同じ**、と考えて構いません。

違いは2つだけあります。

1. **パニック処理**: esp-generateはprobe-rsを選ぶと `panic-rtt-target` を使います。この教材のexamplesは `esp-backtrace`（defmt機能つき）を使っています。どちらもRTT経由でパニック内容を出せるので、動作に問題はありません。
2. **ツールの切り替え**: esp-generateは「作るときに」probe-rsかespflashを1つ決めます。この教材のexamplesは、`cargo feature` で後から切り替えられるようにしてあります（`--features espflash`）。

## よくある失敗

- **`esp-generate` が見つからない**: `cargo install esp-generate --locked` を実行し、`~/.cargo/bin` がPATHに入っているか確認します。
- **wifiを選んだのにビルドが通らない**: `wifi` は `alloc` と `unstable-hal` が前提です。TUIなら依存は自動で促されますが、ヘッドレスでは自分で `-o alloc -o unstable-hal` も付けます。
- **logとprobe-rsを両方選んでしまう**: 同時には選べません。probe-rsなら `defmt` にします。

## やってみよう

`esp-generate list-options` を実行して、オプションの全体像を眺めてみましょう。気になったオプションを1つ選び、`esp-generate explain <名前>` でその説明を読んでみてください。5分でesp-generateの地図が頭に入ります。

## 確認問題

1. マイコンのプロジェクトを `cargo new` ではなく esp-generate で作るのはなぜですか。
2. probe-rs を使うとき、ログのオプションに `log` を選べないのはなぜですか。
3. Wi-Fiを使うプロジェクトをヘッドレスで作るとき、`-o wifi` のほかに最低限どのオプションが要りますか。

<details>
<summary>答え</summary>

1. マイコン開発にはチップ指定・ターゲット・書き込み方法・ブートローダ設定などの設定ファイルが必要で、esp-generateがそれらをまとめて用意してくれるから。
2. probe-rsは実機メモリをRTT経由で読む仕組みで、logクレートが流すシリアル出力を読めないから（probe-rsならdefmtを使う）。
3. `-o alloc` と `-o unstable-hal`（wifiの前提）。この教材の構成なら加えて `-o embassy -o probe-rs -o defmt`。
</details>

## まとめ

- マイコン用プロジェクトは esp-generate で作る。TUI（対話）とヘッドレス（コマンド一発）の2通り
- この教材の構成は `unstable-hal` `embassy` `probe-rs` `defmt`（無線なら `alloc` `wifi` `ble-trouble` を追加）
- 生成物は教材の `examples/` とほぼ同じ。`list-options` と `explain` で全オプションを調べられる

## 次のページ

プロジェクトの中身を一行ずつ知りたくなったら、第5部[2. main以前に起きること](/embassy-esp32-c6/part05/02-before-main/)で、電源が入ってから `main` が呼ばれるまでの流れを追ってみてください。
