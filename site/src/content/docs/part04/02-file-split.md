---
title: "2. ファイル分割"
description: mod宣言とファイルの対応関係を理解し、moduleを別ファイルに切り出せるようになります。
part: 4
lesson: 2
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part04/01-module
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1（ホストPCでcargo check/run済み）"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html
  - https://doc.rust-lang.org/reference/items/modules.html
---

## このページでできるようになること

- `mod led;`（セミコロンで終わる形）が「ファイルを読み込む宣言」だと分かる
- `mod` 宣言とファイル名の対応規則を説明できる
- 3ファイル構成の小さなプロジェクトを作れる

## 先に結論

前のページの `mod led { ... }` は、中身を **`src/led.rs` という別ファイルに移す**ことができます。移した後は `main.rs` に `mod led;` と**セミコロンだけの宣言**を書きます。これが「led module の中身は led.rs にあります」という意味になります。ファイル名と module 名は必ず一致させます。宣言を書き忘れると、ファイルを置いただけではコンパイル対象になりません。

## 身近なたとえ

学校のクラス名簿を想像してください。職員室に「1組の名簿は1組のファイルにあります」という**目次**があり、名簿の本体は各クラスのファイルに入っています。目次に載っていないファイルは、たとえ棚にあっても「存在しない」扱いです。

実際の技術との違いを一言添えると、Rustの「目次」である `mod` 宣言は単なるメモではなく、**コンパイラへの命令**です。宣言がなければそのファイルは一切コンパイルされず、文法エラーがあっても気づけません。

## 仕組み

`mod` には2つの書き方があります。

| 書き方 | 意味 |
|---|---|
| `mod led { ... }` | 中身をその場に書く（前ページの形） |
| `mod led;` | 中身は `src/led.rs` にある、という宣言 |

コンパイラは `mod led;` を見ると、決まった場所からファイルを探します。

```text
src/
├── main.rs      ← mod led; mod button; と宣言する
├── led.rs       ← led module の中身
└── button.rs    ← button module の中身
```

module がさらに大きくなったら、`src/led.rs` の代わりに `src/led/` フォルダを作り、その中を `mod.rs` と複数ファイルに分ける形もあります。これは第12部のプロジェクト分割で使います。今は「1 module = 1ファイル」で十分です。

## RustとEmbassyではどう書くか

前ページのコードを3ファイルに分けます。この3ファイルで完全なプロジェクトです（`cargo new` で作ったプロジェクトの `src/` に置けば動きます）。

`src/main.rs`:

```rust
mod button; // 「src/button.rs を読み込む」という宣言
mod led; // 「src/led.rs を読み込む」という宣言

fn main() {
    if button::is_pressed() {
        led::on();
    } else {
        led::off();
    }
}
```

`src/led.rs`:

```rust
pub fn on() {
    println!("LED ON");
}

pub fn off() {
    println!("LED OFF");
}
```

`src/button.rs`:

```rust
pub fn is_pressed() -> bool {
    // 本当はGPIOを読む。ここでは仮の値
    true
}
```

## コードを一行ずつ読む

- `mod button;` — `led.rs` の中身が `{}` の中に書いてあるのと同じ意味になります。**ファイル側には `mod` を書きません**。`led.rs` の中身は最初から「led module の中」だからです。
- `pub fn on()` — ファイルに分けても公開規則は同じです。`main.rs` から呼ぶ関数には `pub` が必要です。
- 呼び出し側は `led::on()` のまま変わりません。**分割してもプログラムの意味は1文字も変わらない**のがポイントです。

## 実行方法

```bash
cargo new blink-split
cd blink-split
# 上の3ファイルを src/ に配置してから
cargo run
```

```text
LED ON
```

Rust Playground は1ファイルしか扱えないため、このページの練習は手元の `cargo` で行うのがおすすめです（第1部7ページで導入済みです）。

## よくある失敗

**1. ファイルは置いたのに `mod` 宣言を忘れる**

`src/led.rs` を作っただけでは何も起きません。`main.rs` に `mod led;` がないと、`led::on()` の行で `use of unresolved module or unlinked crate 'led'` というエラーになります。コンパイラは宣言されたファイルしか見ないからです。

**2. ファイル名と module 名が食い違う**

`mod led;` と宣言したのにファイル名が `Led.rs` や `leds.rs` だと、

```text
error[E0583]: file not found for module `led`
```

というエラーになります。module 名は小文字のスネークケース（例: `button_reader`）にし、ファイル名と完全に一致させます。

## やってみよう

前ページの「やってみよう」で作った `buzzer` module を、`src/buzzer.rs` に切り出してみましょう。`main.rs` に `mod buzzer;` を追加し、`cargo run` で BEEP が表示されれば成功です。

## 確認問題

1. `mod led { ... }` と `mod led;` の違いは何ですか？
2. `src/led.rs` の先頭に `mod led {` と書く必要はありますか？
3. `mod sensor;` と宣言したとき、コンパイラが探すファイルはどれですか？

<details>
<summary>答え</summary>

1. 前者は中身をその場に書く形、後者は中身を同名のファイルから読み込む宣言です。意味は同じです。
2. 不要です。ファイルの中身は自動的にその module の中身になります。書くと `sensor::sensor::` と二重の入れ子になってしまいます。
3. `src/sensor.rs`（または `src/sensor/mod.rs`）です。
</details>

## まとめ

- `mod 名前;` は「同名のファイルを module として読み込む」宣言
- ファイル名と module 名は完全一致。宣言がないファイルはコンパイルされない
- ファイルに分けても、呼び出し方（`led::on()`）と `pub` の規則は変わらない

## 次のページ

分割すると「どこまで外に見せるか」という設計の問題が生まれます。次は `pub` の種類と、公開範囲の決め方を学びます。

[3. pubと公開範囲](/embassy-esp32-c6/part04/03-pub/)

---

前のページ: [1. moduleで整理する](/embassy-esp32-c6/part04/01-module/)
