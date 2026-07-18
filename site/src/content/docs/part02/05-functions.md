---
title: "5. 関数"
description: 引数と戻り値のある関数をfnで定義し、式と文の違いを理解します。
part: 2
lesson: 5
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part02/04-bool
hardware:
  - ESP32-C6-DevKitC-1（Rust Playgroundで試す場合は不要）
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch03-03-how-functions-work.html
---

## このページでできるようになること

- `fn` で引数と戻り値のある関数を定義して呼び出せる
- 引数と戻り値に型注釈が**必須**である理由を説明できる
- 「最後の式が戻り値になる」というRustの規則を使える

## 先に結論

関数は `fn 名前(引数: 型, ...) -> 戻り値の型 { 本体 }` で定義します。変数と違って、**引数と戻り値の型は省略できません**。関数は他人（未来の自分を含む）が使う「窓口」なので、型を明記して約束をはっきりさせる設計です。本体の**最後の式にセミコロンを付けない**と、それが戻り値になります。`return` と書いてもよいですが、Rustでは最後の式で返すのが普通です。

## 身近なたとえ

関数は「注文票つきの自動販売機」です。決まった形の入力（お金 = 引数）を入れると、決まった種類の出力（飲み物 = 戻り値)が出てきます。中の仕組みを知らなくても、入力と出力の約束さえ分かれば使えます。

ただし実際の関数は自動販売機と違い、出力のない（何も返さない）関数も作れますし、同じ入力に対して必ず同じ出力とは限りません（センサを読む関数など)。

## 仕組み

```rust
fn area(width: u32, height: u32) -> u32 {
    width * height
}
```

- `fn` — 関数定義の合図。blinkyの `async fn main` にも入っていましたね
- `area` — 関数名。変数と同じく小文字とアンダースコアで
- `(width: u32, height: u32)` — 引数の並び。**それぞれに型注釈が必須**です
- `-> u32` — 戻り値の型。返さない関数なら `->` ごと省略します
- `width * height` — 最後の式。**セミコロンがない**ことに注目してください。この式の値が戻り値になります

なぜ引数の型は省略できないのでしょうか。関数の中身を読まなくても使い方が分かるように、そしてコンパイラが呼び出し側の間違い（型の合わない値を渡した等）をすぐ指摘できるようにするためです。型注釈は関数の「取扱説明書」の役割を果たします。

ここでRust特有の大事な区別をひとつ。**式（expression）**は値を生む書き方（`1 + 2`、`a > b`）、**文（statement）**は値を生まない書き方（`let x = 5;`）です。セミコロンを付けると式は文になり、値が捨てられます。「最後の式が戻り値」という規則は、この区別の上に成り立っています。

## Arduinoではどう書くか

Arduino（C++）の関数と考え方はほぼ同じです。

```cpp
unsigned long area(unsigned long width, unsigned long height) {
  return width * height;   // C++は必ずreturnを書く
}
```

Rustとの違いは、①戻り値の型を後ろに `-> u32` と書く、②`return` を書かず最後の式で返せる、③何も返さない関数の型（C++の `void`）は書かなくてよい、の3点です。

## RustとEmbassyではどう書くか

関数の定義は `main` の**外**に貼ります。blinkyなら `#[esp_rtos::main]` の行より上、Playgroundなら `fn main()` の外です。呼び出しはこれまでどおり `main` の中に貼ります（これは抜粋です。貼りつけ先の完全なコードは examples/01-blinky を見てください）。

```rust
// ここから3つはmainの外に貼る
fn area(width: u32, height: u32) -> u32 {
    width * height
}

fn blink_message(count: u32) {
    log::info!("{} 回点滅しました", count);
}

fn half_period(period_ms: u64) -> u64 {
    period_ms / 2
}
```

```rust
// ここはmainの中に貼る
let a = area(40, 25);
log::info!("面積は {} です", a);
blink_message(3);
let h = half_period(1000);
log::info!("半分の時間は {} ミリ秒です", h);
```

## コードを一行ずつ読む

- `fn blink_message(count: u32) {` — `->` がないので何も返さない関数です（正確には `()` という「空っぽ」を表す値を返しています。この `()` はエラーメッセージによく登場します）
- `fn half_period(period_ms: u64) -> u64` — blinkyの点滅間隔は `u64` のミリ秒でした。実際の型に合わせた関数にしておくと、あとで `Duration::from_millis(half_period(1000))` のように本物の待ち時間に使えます
- `let a = area(40, 25);` — 呼び出しは `名前(値, 値)`。戻り値を変数で受け取ります

## 実行方法

動かし方は2通りです（詳しくは[1. 変数とlet](/embassy-esp32-c6/part02/01-variables/)）。

```text
INFO - 面積は 1000 です
INFO - 3 回点滅しました
INFO - 半分の時間は 500 ミリ秒です
```

## よくある失敗

**失敗1: 最後の式にセミコロンを付けた（E0308）**

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b; // セミコロンのせいで値が捨てられる
}
```

```text
error[E0308]: mismatched types
  |
1 | fn add(a: i32, b: i32) -> i32 {
  |    ---                    ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     a + b;
  |          - help: remove this semicolon to return this value
```

「`i32` を返すはずが `()`（空っぽ）が返っている」というエラーです。セミコロンを付けた瞬間、`a + b` は「値を捨てる文」になり、関数は何も返さなくなります。`help` が「このセミコロンを消せば値が返る」とピンポイントで教えてくれています。Rust初心者が最初に必ず踏む失敗なので、`found ()` を見たら「セミコロンかも」と疑ってください。

**失敗2: 引数の型を書き忘れた**

```rust
fn add(a, b) { // 型注釈なし
    a + b
}
```

```text
error: expected one of `:`, `@`, or `|`, found `,`
  |
1 | fn add(a, b) {
  |         ^ expected one of `:`, `@`, or `|`
  |
help: if this is a parameter name, give it a type
  |
1 | fn add(a: TypeName, b) {
```

変数の型推論に慣れると引数でも省略したくなりますが、関数の引数は必ず型を書きます。`help` の `give it a type`（型を付けて）が直し方そのものです。

## やってみよう

摂氏温度を華氏に変換する関数 `fn to_fahrenheit(celsius: f32) -> f32` を書いてみましょう。式は `celsius * 1.8 + 32.0` です。`to_fahrenheit(25.0)` が 77 になれば成功です。

## 確認問題

1. 変数の型は省略できるのに、関数の引数の型が省略できないのはなぜでしょうか?
2. `fn double(x: i32) -> i32 { x * 2; }` はコンパイルできるでしょうか? できないならどこを直しますか?
3. `-> ` がない関数は何を返しているでしょうか?

<details>
<summary>答え</summary>

1. 関数は「使う側との約束（窓口）」だから。型を明記することで、中身を読まなくても使い方が分かり、呼び出し側の間違いをコンパイラが即座に指摘できます。
2. できません（E0308）。`x * 2` のセミコロンを削除して、最後の式として値を返します。
3. `()`（ユニットと呼ばれる空っぽの値）。エラーメッセージの `found ()` はこれです。

</details>

## まとめ

- `fn 名前(引数: 型) -> 戻り値型 { ... }`。引数と戻り値の型は必須
- 最後の式（セミコロンなし）が戻り値になる。`found ()` を見たらセミコロンを疑う
- 関数の型注釈は「取扱説明書」。呼び出し側の間違いをコンパイル時に見つけられる

## 次のページ

条件によって処理を変える `if` を学びます。Rustの `if` は値を返す「式」でもある、という便利な性質も紹介します。

[6. ifで分岐する →](/embassy-esp32-c6/part02/06-if/)

---

- 前のページ: [4. boolと比較](/embassy-esp32-c6/part02/04-bool/)
- 次のページ: [6. ifで分岐する](/embassy-esp32-c6/part02/06-if/)
