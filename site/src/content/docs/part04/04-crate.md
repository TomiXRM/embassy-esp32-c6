---
title: "4. crateと依存関係"
description: crateの意味とCargo.tomlへの依存追加を学びます。バージョン指定とfeatureの読み方も扱います。
part: 4
lesson: 4
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part04/03-pub
  - part01/08-new-project
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 / embedded-hal 1.0.0（ホストPCでcargo check/run済み）"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html
  - https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
  - https://doc.rust-lang.org/cargo/reference/features.html
---

## このページでできるようになること

- crate・パッケージ・moduleの関係を説明できる
- `Cargo.toml` に依存crateを追加できる
- バージョン指定（`1.0`、`~1.1.0`）とfeatureの意味が読める

## 先に結論

**crate（クレート）** はRustのコンパイル単位で、配布の単位でもあります。あなたのプロジェクトも1つのcrateで、その中を前ページまでのmoduleで整理してきました。他人のcrateは [crates.io](https://crates.io/) で公開されていて、`Cargo.toml` の `[dependencies]` に1行書くだけで使えます。バージョンは semver（セマンティックバージョニング）の規則で指定し、**feature** で機能の有効・無効を切り替えます。この教材で使うcrateとバージョンは[バージョン固定表](/embassy-esp32-c6/project/versions/)に固定されています。

## 身近なたとえ

料理にたとえると、module が「自分の台所の引き出しの仕切り」だとすれば、crate は「市販の調味料のボトル」です。しょうゆを自分で醸造する人はいません。ラベル（バージョンとfeature）を確認して買ってきて、自分の料理に使います。

実際の技術との違いを一言添えると、crateは買い切りの品物ではなく**ソースコードごと取り込んで一緒にコンパイルする**ものです。だからバージョンが変わると、自分のコードのコンパイルが通らなくなることもあります。ラベルの確認（バージョン固定）が調味料以上に重要です。

## 仕組み

言葉の関係を整理します。

- **パッケージ** — `cargo new` が作る単位。`Cargo.toml` を1つ持つ
- **crate** — コンパイルの単位。実行ファイルになる binary crate と、部品として使われる library crate がある
- **module** — crateの中の整理棚（1〜3ページで学んだもの）

`Cargo.toml` の `[dependencies]` に書いた crate は、cargoが自動でダウンロードして一緒にビルドしてくれます。実際にビルドされた正確なバージョンの記録は `Cargo.lock` に残り、チーム全員が同じ組み合わせでビルドできます。

バージョン指定の読み方（semver: `メジャー.マイナー.パッチ`）:

| 書き方 | 意味 |
|---|---|
| `"1.0.0"` | 1.0.0以上、2.0.0未満で互換とみなす（`^1.0.0` と同じ） |
| `"~1.1.0"` | 1.1.x のみ許す。マイナー更新を止めたいときに使う |
| `"=1.0.0"` | 1.0.0ちょうどに固定 |

この教材の examples では esp-hal を `~1.1.0` で指定しています。理由は[バージョン固定表](/embassy-esp32-c6/project/versions/)にある通り、unstable feature配下のAPIはマイナー更新で変わりうるためです。

## RustとEmbassyではどう書くか

第5部以降で頻出する **embedded-hal**（組み込み向け共通インターフェース集）を追加してみます。ホストPCの練習用プロジェクトで試せます。

```bash
cargo add embedded-hal@1.0.0
```

`Cargo.toml` にはこう入ります。

```toml
[dependencies]
embedded-hal = "1.0.0"
```

これだけで `use` できるようになります。

```rust
// Cargo.tomlに embedded-hal = "1.0.0" を追加すると使えるようになる
use embedded_hal::digital::PinState;

fn main() {
    let state = PinState::High;
    match state {
        PinState::High => println!("ピンはHigh"),
        PinState::Low => println!("ピンはLow"),
    }
}
```

次に、この教材の examples が実際に使っている指定を見てみます（これは抜粋です。完全な記述は examples/Cargo.toml（ワークスペース共通の依存表）を見てください）。

```toml
esp-hal = { version = "~1.1.0", features = ["esp32c6", "unstable", "log-04"] }
```

`features = [...]` が **feature（フィーチャー）** です。crateの持つ機能のうち、どれを有効にするかのスイッチです。esp-halは1つのcrateで多数のチップに対応しているため、`esp32c6` featureで「ESP32-C6用のコードを有効にする」と指示しています。

## コードを一行ずつ読む

- `use embedded_hal::digital::PinState;` — crate名 `embedded-hal` は、コード内では `embedded_hal` とアンダースコアになります。ハイフンはRustの識別子に使えないためです。
- `features = ["esp32c6", ...]` — featureを忘れると、そのチップ向けのAPIが存在せず大量のエラーになります。組み込みcrateでは「チップ名feature」が必須のことが多いです。

## 実行方法

```bash
cargo new dep-practice
cd dep-practice
cargo add embedded-hal@1.0.0
# src/main.rs を上のコードに置き換えてから
cargo run
```

```text
ピンはHigh
```

初回はダウンロードとビルドで少し待ちます。

## よくある失敗

**1. crate名のハイフンとアンダースコアの混同**

`use embedded-hal::...` と書くと文法エラーです。Cargo.tomlでは `embedded-hal`（ハイフン）、コード内では `embedded_hal`（アンダースコア）。この読み替えは最初は全員が間違えます。

**2. 世代の違う情報をコピーしてくる**

ネット記事の `esp-wifi 0.15` や `esp-hal-embassy` のコードをそのまま貼ると、この教材の環境ではコンパイルできません。crateは世代でAPIが大きく変わります。**依存を足すときは、記事の日付とバージョン表記を必ず確認**してください。この教材内では[バージョン固定表](/embassy-esp32-c6/project/versions/)が唯一の正解です。

**3. featureの付け忘れ**

esp-halを追加したのに `features = ["esp32c6"]` がないと、`Output` などの型が見つからずエラーの山になります。エラーの内容ではなくCargo.tomlを疑うのが近道です。

## やってみよう

練習用プロジェクトで `cargo add log@0.4` を実行し、`Cargo.toml` に何が増えたか、`Cargo.lock` のどこに `log` が現れたかを見てみましょう。追加した行を消せば依存も消えます。

## 確認問題

1. crateとmoduleの違いは何ですか？
2. `esp-hal = "~1.1.0"` の `~` は何を意味しますか？
3. `features = ["esp32c6"]` は何のためにありますか？

<details>
<summary>答え</summary>

1. crateはコンパイル・配布の単位、moduleはcrateの内部を整理する仕組みです。
2. 1.1.x のパッチ更新だけを許し、1.2.0などのマイナー更新は取り込まない指定です。
3. 1つのcrateが持つ機能のうち、どれを有効にするかを選ぶスイッチです。esp-halでは対象チップの選択に使います。
</details>

## まとめ

- crateはコンパイルと配布の単位。`[dependencies]` に1行で取り込める
- バージョンはsemverで指定し、`Cargo.lock` が実際の組み合わせを記録する
- featureは機能スイッチ。組み込みcrateではチップ名featureが必須のことが多い

## 次のページ

crateを跨いでコードが協調できるのは、「共通の能力の約束」を定義する **trait** のおかげです。次のページは第4部の山場、traitです。

[5. trait — 共通の能力を定義する](/embassy-esp32-c6/part04/05-trait/)

---

前のページ: [3. pubと公開範囲](/embassy-esp32-c6/part04/03-pub/)
