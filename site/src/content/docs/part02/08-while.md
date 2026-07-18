---
title: "8. while"
description: 条件が成り立つ間だけ繰り返すwhileループを書けるようになります。
part: 2
lesson: 8
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part02/07-loop
hardware:
  - ESP32-C6-DevKitC-1（Rust Playgroundで試す場合は不要）
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch03-05-control-flow.html
---

## このページでできるようになること

- `while 条件 { ... }` で条件付きの繰り返しを書ける
- `loop` と `while` をどう使い分けるか説明できる
- 「条件がいつfalseになるか」を意識してループを設計できる

## 先に結論

`while` は「条件が `true` の間だけ回るループ」です。毎周の先頭で条件を調べ、`false` になった瞬間にループを抜けます。`loop` + `if` + `break` で書ける内容を1行にまとめた形と考えれば、新しく覚えることはほとんどありません。大事なのは、**ループの中で条件を `false` に近づける変化を起こすこと**。それを忘れると意図しない無限ループになります。

## 身近なたとえ

`while` は「お湯が沸くまでコンロにかけ続ける」ような繰り返しです。「まだ沸いていない間は加熱を続ける」──条件（沸いていない）を毎回確かめて、成り立たなくなったら（沸いたら）やめます。

ただし実際の `while` は条件を**周回の最初にしか**調べません。周回の途中で条件が変わっても、すぐには止まらず、次の周の先頭まで走りきってから判定されます。

## 仕組み

```rust
while 条件 {
    // 条件がtrueの間、繰り返される
}
```

実行の流れはこうです。

1. 条件を調べる。`false` ならループの次の行へ抜ける
2. `true` なら `{ }` の中を実行する
3. 1に戻る

前のページの `loop` で書くとこうなります。

```rust
loop {
    if !条件 {
        break;
    }
    // 処理
}
```

つまり `while` は「先頭で条件チェックして抜けるloop」の省略形です。使い分けの目安は次のとおりです。

- **`loop`**: わざと終わらせない（組み込みの `main`）、または抜ける場所がループの途中・複数ある
- **`while`**: 「〜の間だけ」と条件が最初から1つに決まっている

なお、C++の `while(1)` のような無限ループをRustで `while true` と書くと、コンパイラが「`loop` を使って」と警告します。「終わらない」という意図は `loop` で表すのがRust流です。

## Arduinoではどう書くか

書き方はほぼ同じで、条件のカッコの有無だけが違います。

```cpp
int countdown = 5;
while (countdown > 0) {   // C++: カッコが必要
  countdown--;
}
```

```rust
let mut countdown = 5;
while countdown > 0 {     // Rust: カッコ不要、波カッコ必須
    countdown -= 1;
}
```

Rustには `countdown--`（デクリメント演算子）が**ありません**。`countdown -= 1;` と書きます。`++` もないので `count += 1;` です。

## RustとEmbassyではどう書くか

これは抜粋です。貼りつけ先の完全なコードは examples/01-blinky を見てください。

```rust
let mut countdown = 5;
while countdown > 0 {
    log::info!("あと {} 秒", countdown);
    countdown -= 1;
}
log::info!("発射!");

let mut voltage = 100;
while voltage > 10 {
    voltage = voltage / 2;
}
log::info!("最終電圧: {}", voltage);
```

## コードを一行ずつ読む

- `while countdown > 0 {` — 条件は `bool` を返す式。`if` と同じ規則です
- `countdown -= 1;` — **条件をfalseに近づける一歩**。この行がループ設計の心臓部です
- `log::info!("発射!");` — 条件が `false`（`countdown` が0）になった瞬間、ここへ抜けてきます
- `voltage = voltage / 2;` — 100 → 50 → 25 → 12 → 6 と半分にしていき、10以下になったら止まります。整数の割り算なので小数点以下は切り捨てです

## 実行方法

動かし方は2通りです（詳しくは[1. 変数とlet](/embassy-esp32-c6/part02/01-variables/)）。

```text
INFO - あと 5 秒
INFO - あと 4 秒
INFO - あと 3 秒
INFO - あと 2 秒
INFO - あと 1 秒
INFO - 発射!
INFO - 最終電圧: 6
```

## よくある失敗

**失敗1: 条件に `=` を書いた（E0308）**

```rust
let mut countdown = 5;
while countdown = 0 { // ==のつもりが=
    countdown -= 1;
}
```

```text
error[E0308]: mismatched types
  |
3 |     while countdown = 0 {
  |           ^^^^^^^^^^^^^ expected `bool`, found `()`
  |
help: you might have meant to compare for equality
  |
3 |     while countdown == 0 {
  |                      +
```

代入式の値は `()` なので、`bool` を待っている `while` とかみ合いません。`help` は「等しいか比較したかったのでは?」と `==` への修正案まで出しています。C++では黙って動いてしまう（そして永遠にバグを探すことになる）間違いが、Rustではコンパイルの時点で捕まります。

**失敗2: 条件がfalseに近づかない**

```rust
let mut countdown = 5;
while countdown > 0 {
    log::info!("あと {} 秒", countdown);
    // countdown -= 1; を書き忘れた
}
```

コンパイルは通りますが、`countdown` はずっと5のままなので「あと 5 秒」が無限に流れ続けます。`while` を書いたら「この条件は、ループの中のどの行のおかげで、いつか `false` になるのか?」と自分に問いかける習慣をつけましょう。答えられなければ無限ループです。

## やってみよう

`let mut value = 1;` から始めて、`while value < 1000 { value *= 2; }` で2倍にし続け、最後に表示してみましょう。1000を超えた最初の2のべき乗（1024）になれば成功です。`*= 2` を `*= 3` に変えると結果はどうなるでしょうか。

## 確認問題

1. `while` と `loop` の使い分けの目安を説明してください。
2. `while countdown = 0` のエラーで `found ()` と表示されるのはなぜでしょうか?
3. `while x > 0 { log::info!("{}", x); }` が無限ループになるのはなぜでしょうか?

<details>
<summary>答え</summary>

1. 条件が「〜の間だけ」と1つに決まっているなら `while`。意図的に終わらせない、または途中・複数箇所で抜けるなら `loop`。
2. `countdown = 0` は代入式で、代入式の値は `()` だから。`while` は `bool` を期待するので型が合いません。
3. ループの中に `x` を減らす処理がなく、条件が永遠に `true` のままだからです。

</details>

## まとめ

- `while 条件 { }` は「先頭で条件チェックするloop」の省略形
- ループの中に条件を `false` に近づける一歩を必ず入れる
- Rustに `++` / `--` はない。`+= 1` / `-= 1` を使う

## 次のページ

「5回繰り返す」「配列の全要素を順に処理する」なら、回数管理を自動でやってくれる `for` がいちばん安全です。次のページで学びます。

[9. forとrange →](/embassy-esp32-c6/part02/09-for/)

---

- 前のページ: [7. loopと無限ループ](/embassy-esp32-c6/part02/07-loop/)
- 次のページ: [9. forとrange](/embassy-esp32-c6/part02/09-for/)
