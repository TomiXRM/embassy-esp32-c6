---
title: "8. Rustプロジェクトの作成"
description: esp-generateでESP32-C6用のRustプロジェクトを作り、Cargo.toml・build.rs・.cargo/config.tomlなど各ファイルの役割を説明します。
part: 1
lesson: 8
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part01/07-setup
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1"
last_verified: "2026-07-18"
sources:
  - https://github.com/esp-rs/esp-generate
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
---

## このページでできるようになること

- esp-generateでESP32-C6用プロジェクトを作れる
- プロジェクトの各ファイル（Cargo.toml、build.rs、.cargo/config.toml、src/main.rs）の役割を説明できる
- 「クレート」と「依存関係」という言葉の意味が分かる

## 先に結論

マイコン用のRustプロジェクトは、`cargo new`ではなく**esp-generate**で作ります。マイコン開発には「どのチップ向けにビルドするか」「どうやって書き込むか」などの設定ファイルが必要で、esp-generateがそれを全部そろえてくれるからです。生成されるファイルのうち大事なのは4つ。部品リストの**Cargo.toml**、ビルド時の指示書**build.rs**、ビルド先と書き込み方法の設定**.cargo/config.toml**、そしてプログラム本体の**src/main.rs**です。

## 身近なたとえ

esp-generateは「プラモデルの箱」のようなものです。箱を開けると、部品（ライブラリ）と説明書（設定ファイル）が最初からそろっていて、すぐ組み立てを始められます。ゼロから部品を集める必要はありません。

たとえと違うのは、中身がすべてテキストファイルで、あとから自由に書き換えられることです。今日は箱の中身を確認するのが目的です。

## プロジェクトを作る

作業用フォルダで次を実行します（`my-blinky`は好きな名前で構いません）。

```bash
esp-generate --chip esp32c6 my-blinky
```

画面に選択肢が出ます。矢印キーとスペースで選び、この教材では**Embassyを有効にするオプションを必ずチェック**してください（この教材の全コードはEmbassy前提です）。ログ出力（log）のオプションもあれば有効にします。終わったら生成されたフォルダへ移動します。

```bash
cd my-blinky
```

## 仕組み — 生成されたファイルの地図

```text
my-blinky/
├── Cargo.toml          ← 使うライブラリ（クレート）の一覧
├── build.rs            ← ビルド時に実行される指示書
├── .cargo/
│   └── config.toml     ← ターゲットと書き込みコマンドの設定
└── src/
    └── main.rs         ← プログラム本体
```

### Cargo.toml — 部品リスト

Rustではライブラリのことを**クレート（crate）**と呼び、使いたいクレートをCargo.tomlに書きます。これを**依存関係**といいます。この教材のLチカで使うのは次のクレートです（教材の`examples/`ではワークスペースという仕組みで一括管理していますが、意味は同じです）。

```toml
[dependencies]
esp-hal = { version = "~1.1.0", features = ["esp32c6", "unstable", "log-04"] }
esp-rtos = { version = "0.3.0", features = ["esp32c6", "embassy", "log-04"] }
esp-bootloader-esp-idf = { version = "0.5.0", features = ["esp32c6", "log-04"] }
esp-println = { version = "0.17.0", features = ["esp32c6", "log-04"] }
esp-backtrace = { version = "0.19.0", features = ["esp32c6", "panic-handler", "println"] }
log = "0.4"
embassy-executor = "0.10.0"
embassy-time = "0.5"
```

それぞれの役割は、

- **esp-hal**: ESP32-C6のGPIOやタイマーをRustから安全に触るための層（第5部で詳しく）
- **esp-rtos**: Embassy（非同期の仕組み）とチップをつなぐ土台
- **esp-println / log / esp-backtrace**: ログの表示と、エラー時の情報表示
- **embassy-executor / embassy-time**: taskの実行と時間待ち

`features = ["esp32c6", ...]`は「このクレートのESP32-C6向け機能を有効にする」という指定です。同じクレートが複数のチップに対応しているため、必ずチップ名を指定します。

バージョンは教材全体で固定しています。生成されたものと教材が食い違うときは、[バージョン固定表](/embassy-esp32-c6/project/versions/)に合わせてください。

### build.rs — ビルド時の指示書

```rust
fn main() {
    // linkall.x は最後のリンカスクリプトにすること
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
```

プログラムの各部分をメモリのどこへ置くかを決める「リンカスクリプト」を指定しています。OSのないマイコンでは、この配置指定が必須です（メモリの話は第5部で扱います）。

### .cargo/config.toml — ビルド先と書き込み方法

```toml
[target.riscv32imac-unknown-none-elf]
runner = "espflash flash --monitor --chip esp32c6"

[build]
rustflags = ["-C", "force-frame-pointers"]
target = "riscv32imac-unknown-none-elf"

[env]
ESP_LOG = "info"
```

- `target = ...`: 「このプロジェクトは常にC6向けにビルドする」という指定。毎回コマンドで指定しなくてよくなります
- `runner = "espflash flash --monitor ..."`: `cargo run`と打ったとき、ビルド後に**自動でespflashが書き込みとモニタ表示までやる**設定です
- `ESP_LOG = "info"`: ログの表示レベル（infoレベル以上を表示）

### src/main.rs — プログラム本体

生成直後のmain.rsにも、初期化のお決まりコードが入っています。中身は次の2ページで実物を動かしながら読むので、ここでは「`#![no_std]`で始まり、`main`関数があり、無限ループで終わる」ことだけ眺めておけば十分です。

## 実行方法 — ビルドできるか確認

ボードをつなぐ前に、コンパイルが通るかだけ確かめられます。

```bash
cargo build --release
```

初回は依存クレートを全部コンパイルするので数分かかります。`Finished`と表示されれば成功です。`--release`は最適化ありでビルドする指定で、マイコンでは基本こちらを使います。

## よくある失敗

- **`error: linker ... not found`や`linkall.x`関連のエラー**: build.rsや.cargo/config.tomlを消したり書き換えたりすると起きます。生成されたままの内容へ戻してください
- **`can't find crate for std`というエラー**: ターゲット追加（`rustup target add riscv32imac-unknown-none-elf`）を忘れているか、config.tomlの`target`行が消えています。[前のページ](/embassy-esp32-c6/part01/07-setup/)の手順2を確認してください
- **初回ビルドが遅くて failed に見える**: 数分かかるのは正常です。エラー表示が出ていない限り待ってください

## やってみよう

Cargo.tomlを開いて、`[dependencies]`に並ぶクレートを1行ずつ眺め、「これは何の係か」を上の説明と照らし合わせてみてください。全部言えたらこのページは合格です。

## 確認問題

1. マイコンのプロジェクトを`cargo new`ではなくesp-generateで作るのはなぜですか。
2. `.cargo/config.toml`の`runner`に設定されているコマンドは、`cargo run`したとき何をしてくれますか。
3. 「クレート」とは何ですか。

<details>
<summary>答え</summary>

1. ターゲット指定・リンカスクリプト・書き込み設定など、マイコン開発に必須の設定ファイル一式を自動でそろえてくれるからです。`cargo new`はパソコン用の最小構成しか作りません。
2. ビルドされたプログラムをespflashでボードへ書き込み、続けてシリアルモニタを開いてログを表示します。
3. Rustにおけるライブラリ（またはプログラム）のまとまりの単位です。Cargo.tomlに書いて取り込みます。

</details>

## まとめ

- プロジェクトは`esp-generate --chip esp32c6 名前`で作る。Embassyのオプションを有効にする
- Cargo.toml＝部品リスト、build.rs＝メモリ配置の指示、.cargo/config.toml＝ターゲットと書き込み設定
- `cargo run`一発でビルド→書き込み→モニタまで動くように設定されている

## 次のページ

プロジェクトができたので、いよいよボードへ書き込みます。espflashの動きと、シリアルモニタでログを見る方法を学びます。

- 前: [7. 開発環境の構築](/embassy-esp32-c6/part01/07-setup/)
- 次: [9. 書き込みとシリアル表示](/embassy-esp32-c6/part01/09-flash-monitor/)
