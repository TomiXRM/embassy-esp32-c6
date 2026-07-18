---
title: "2. mutと変更できる変数"
description: Rustの変数が既定で変更不可である理由と、mutを使った書き換えを学びます。
part: 2
lesson: 2
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part02/01-variables
hardware:
  - ESP32-C6-DevKitC-1（Rust Playgroundで試す場合は不要）
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html
---

## このページでできるようになること

- `let mut` で書き換え可能な変数を宣言できる
- Rustの変数が既定（デフォルト）で変更不可である理由を説明できる
- `+=` などの複合代入演算子を使える

## 先に結論

Rustの変数は、何もしなければ**変更不可（immutable、イミュータブル）**です。あとから値を書き換えたい変数だけ `let mut` と宣言します。「基本は変えられない、変えたいものだけ特別に許可する」という向きになっているのは、**変わらないはずの値がうっかり書き換わる事故**をコンパイル時に防ぐためです。書き換える必要のない変数が大半だと分かっていると、コードを読むときも安心できます。

## 身近なたとえ

学校に貼ってある時間割は、印刷された「変更不可」の掲示です。誰かが勝手にペンで書き換えたら大混乱になります。変更してよいのは、黒板の「今日の日直」のような、書き換える前提で用意された欄だけです。Rustは「この変数は印刷物か、黒板か」を宣言時に決めさせて、印刷物への書き込みをコンパイラが止めてくれます。

ただし本物の変数は紙や黒板と違って、`mut` を付けても書き換えられるのは**同じ型の値だけ**です。数値の箱に文字列を入れる、といった変更はできません。

## 仕組み

前のページで見たエラーを、あらためて正面から見ます。

```rust
let count = 0;
count = count + 1; // ここでコンパイルエラー
```

```text
error[E0384]: cannot assign twice to immutable variable `count`
  |
2 |     let count = 0;
  |         ----- first assignment to `count`
3 |     count = count + 1;
  |     ^^^^^^^^^^^^^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut count = 0;
  |         +++
```

`help` の指示どおり、`mut`（mutable = 変更可能、の略）を付ければ通ります。

```rust
let mut count = 0;
count = count + 1; // OK
```

大事なのは、これが「不便な制限」ではなく「宣言」だという点です。`mut` の付いていない変数を見たら、「この値はこの先ずっと変わらない」と読み手が信じてよい。プログラムが大きくなるほど、この保証が効いてきます。組み込みでは「設定値のつもりだった変数がいつの間にか書き換わっていた」というバグが実機の誤動作に直結するので、なおさら助かる仕組みです。

## Arduinoではどう書くか

Arduino（C++）では変数は最初から書き換え可能で、変更されたくないものに `const` を付けました。

```cpp
int count = 0;          // C++: 既定で変更できる
const int LED_PIN = 10; // 変更したくないものにconstを付ける
```

Rustは向きが逆です。

```rust
let led_pin = 10;    // Rust: 既定で変更できない（C++のconst側が標準）
let mut count = 0;   // 変更したいものにmutを付ける
```

つまりRustの `let` はC++の `const` に近く、`let mut` が普通の変数に近い関係です。「付け忘れたときに安全側に倒れる」のがRustの設計です。

## RustとEmbassyではどう書くか

カウントアップの例です（これは抜粋です。貼りつけ先の完全なコードは examples/01-blinky を見てください）。

```rust
let mut count = 0;
log::info!("最初の count = {}", count);
count = count + 1;
log::info!("1回目のあと count = {}", count);
count += 1;
log::info!("2回目のあと count = {}", count);
```

実は第1部のblinkyにも `mut` はすでに登場しています。

```rust
let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
```

`led.set_high()` はLEDピンの状態を**変更する**操作なので、`led` は `mut` で宣言する必要があったのです。

## コードを一行ずつ読む

- `let mut count = 0;` — 書き換え許可付きの宣言。型は推論で整数になります
- `count = count + 1;` — 「今の `count` に1を足した結果を、あらためて `count` に入れる」。数学の等式ではなく代入です
- `count += 1;` — 上の行の省略記法（**複合代入演算子**）。`-=`、`*=`、`/=` もあります

## 実行方法

動かし方は2通りです（詳しくは[1. 変数とlet](/embassy-esp32-c6/part02/01-variables/)）。blinkyに貼って `cargo run` するか、`log::info!` を `println!` に替えてPlaygroundで実行します。

```text
INFO - 最初の count = 0
INFO - 1回目のあと count = 1
INFO - 2回目のあと count = 2
```

## よくある失敗

**失敗1: mutを付け忘れた（E0384）**

このページ冒頭のエラーです。`cannot assign twice to immutable variable`（変更不可の変数には2回代入できない）と出たら、その変数を本当に書き換えたいのかをまず考え、書き換えたいなら `let mut` にします。「エラー番号 E0384 = mut忘れ」と覚えてしまってよいくらい頻出です。

**失敗2: 使っていないmutを付けた（警告）**

```rust
let mut count = 0;
log::info!("{}", count); // 一度も書き換えていない
```

```text
warning: variable does not need to be mutable
  |
2 |     let mut count = 0;
  |         ----^^^^^
  |         |
  |         help: remove this `mut`
```

今度はエラーではなく `warning`（警告）です。プログラムは動きますが、「この `mut` は不要ですよ」と教えてくれています。警告を放置すると本当に大事な警告が埋もれるので、指示どおり `mut` を外しておきましょう。コンパイラは「変更できるのに変更していない」ことまで見ているわけです。

## やってみよう

`let mut total = 0;` を宣言し、`total += 10;` を3回書いて、最後に `log::info!("合計 {}", total);` で表示してみましょう。30 になれば成功です。そのあと、わざと `mut` を消してE0384を自分の目で再現してみてください。

## 確認問題

1. Rustで変数が既定で変更不可になっているのは、どんな事故を防ぐためでしょうか?
2. C++の `const int x = 5;` に近いのは、Rustの `let x = 5;` と `let mut x = 5;` のどちらでしょうか?
3. `warning: variable does not need to be mutable` が出ました。プログラムは動くでしょうか? どう直すべきでしょうか?

<details>
<summary>答え</summary>

1. 変わらないはずの値がうっかり書き換えられてしまう事故。コンパイル時に発見できます。
2. `let x = 5;`。Rustは「変更不可」が標準で、変更したいときだけ `mut` を付けます。
3. 動きます（エラーではなく警告なので）。ただし不要な `mut` は外して、警告ゼロを保つのがよい習慣です。

</details>

## まとめ

- Rustの変数は既定で変更不可。書き換えたい変数だけ `let mut` にする
- E0384は「mut忘れ」のサイン。warningは動くが放置しない
- blinkyの `let mut led` のように、状態を変えるものには `mut` が必要

## 次のページ

`count` の型はずっと「整数」とだけ言ってきました。しかし整数にも `u8` や `i32` など種類があり、組み込みではどれを選ぶかが大切です。次のページで数値型を学びます。

[3. 数値型 →](/embassy-esp32-c6/part02/03-numbers/)

---

- 前のページ: [1. 変数とlet](/embassy-esp32-c6/part02/01-variables/)
- 次のページ: [3. 数値型](/embassy-esp32-c6/part02/03-numbers/)
