---
title: "4. boolと比較"
description: 比較演算子と論理演算子を使って、条件を表すbool値を作れるようになります。
part: 2
lesson: 4
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part02/03-numbers
hardware:
  - ESP32-C6-DevKitC-1（Rust Playgroundで試す場合は不要）
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch03-02-data-types.html
  - https://doc.rust-lang.org/book/appendix-02-operators.html
---

## このページでできるようになること

- `bool` 型（`true` / `false`）の値を変数に持てる
- 比較演算子（`==` `!=` `<` `>` `<=` `>=`）で条件を書ける
- 論理演算子（`&&` `||` `!`）で条件を組み合わせられる

## 先に結論

`bool` は `true`（真）と `false`（偽）の2つしか値がない型です。`temp > 30` のような比較の結果はすべて `bool` になります。複数の条件は `&&`（かつ）、`||`（または）、`!`（〜でない）で組み合わせます。Rustでは数値の0や1を `bool` の代わりに使うことは**できません**。「ボタンが押されているか」「温度が範囲内か」など、組み込みの判断はすべて `bool` に行き着きます。

## 身近なたとえ

`bool` は「はい/いいえで答える質問の答え」です。「今28度は30度より暑い?」→いいえ（`false`）。「20度以上? かつ 30度以下?」→はい（`true`）。質問（比較式）を書くと、答え（`bool` 値）が返ってくるイメージです。

ただし実際の `bool` は日本語の曖昧な「はい/いいえ」と違い、必ず `true` か `false` のどちらかに決まります。「たぶん」や「未回答」はありません（「値がないかもしれない」を表す仕組みは第3部のOptionで学びます）。

## 仕組み

比較演算子は6種類です。結果はすべて `bool` になります。

| 演算子 | 意味 | 例（temp = 28） | 結果 |
|---|---|---|---|
| `==` | 等しい | `temp == 28` | `true` |
| `!=` | 等しくない | `temp != 28` | `false` |
| `<` | より小さい | `temp < 30` | `true` |
| `>` | より大きい | `temp > 30` | `false` |
| `<=` | 以下 | `temp <= 28` | `true` |
| `>=` | 以上 | `temp >= 30` | `false` |

論理演算子は3種類です。

| 演算子 | 意味 | 例 | 結果 |
|---|---|---|---|
| `&&` | かつ（AND） | `true && false` | `false` |
| `\|\|` | または（OR） | `true \|\| false` | `true` |
| `!` | 〜でない（NOT） | `!true` | `false` |

`=` が1個だと「代入」、2個で「等しいか比較」です。ここはArduinoのC++と同じですが、間違えたときの挙動が違います（「よくある失敗」で見ます）。

## Arduinoではどう書くか

C++にも `bool` があり、比較・論理演算子もほぼ同じ書き方です。大きな違いは、C++では**数値がそのまま条件に使えた**ことです。

```cpp
int flag = 1;
if (flag) { ... }   // C++: 0以外はtrue扱いで通る
```

Rustでは数値と `bool` は完全に別の型で、`1` を `true` の代わりにはできません。`digitalRead()` の `HIGH`/`LOW`（実体は整数）に慣れていると最初は戸惑いますが、「条件のつもりで書いた数値が実は間違い」という事故がなくなります。

## RustとEmbassyではどう書くか

これは抜粋です。貼りつけ先の完全なコードは examples/01-blinky を見てください。

```rust
let temp = 28;
let is_hot = temp > 30;
let is_comfortable = temp >= 20 && temp <= 30;
let needs_alert = temp < 0 || temp > 45;
let is_not_hot = !is_hot;
log::info!("暑い?          {}", is_hot);
log::info!("快適?          {}", is_comfortable);
log::info!("警報が必要?    {}", needs_alert);
log::info!("暑くない?      {}", is_not_hot);

let a = 5;
let b = 5;
log::info!("等しい?   {}", a == b);
log::info!("等しくない? {}", a != b);
```

## コードを一行ずつ読む

- `let is_hot = temp > 30;` — 比較式の結果（`bool`）をそのまま変数に入れています。`is_〜` という名前は「boolが入っている」ことが伝わるRustらしい命名です
- `temp >= 20 && temp <= 30` — 「20以上」**かつ**「30以下」。数学の `20 <= temp <= 30` のようにつなげて書くことはできず、比較を2つ書いて `&&` で結びます
- `temp < 0 || temp > 45` — どちらか一方でも `true` なら全体が `true` になります
- `!is_hot` — `bool` を反転します

## 実行方法

動かし方は2通りです（詳しくは[1. 変数とlet](/embassy-esp32-c6/part02/01-variables/)）。

```text
INFO - 暑い?          false
INFO - 快適?          true
INFO - 警報が必要?    false
INFO - 暑くない?      true
INFO - 等しい?   true
INFO - 等しくない? false
```

## よくある失敗

**失敗1: boolに数値を入れようとした（E0308）**

```rust
let is_on: bool = 1; // C++の癖で1をtrueのつもりで
```

```text
error[E0308]: mismatched types
  |
2 |     let is_on: bool = 1;
  |                ----   ^ expected `bool`, found integer
  |                |
  |                expected due to this
```

「`bool` を期待したのに整数が来た」。Rustでは `true` / `false` と書くしかありません。`expected 〜, found 〜` は型エラーの定番の言い回しで、「期待した型」と「実際に来た型」を教えてくれています。この形はこの先何度も見ることになります。

**失敗2: 型の違う値どうしを比較した（E0308）**

```rust
let brightness: u8 = 200;
let limit: u32 = 100;
log::info!("{}", brightness > limit); // u8とu32の比較
```

```text
error[E0308]: mismatched types
  |
4 |     println!("{}", brightness > limit);
  |                    ----------   ^^^^^ expected `u8`, found `u32`
  |
help: you can convert `brightness` from `u8` to `u32`, matching the type of `limit`
  |
4 |     println!("{}", u32::from(brightness) > limit);
```

計算と同じで、比較も同じ型どうしでないとできません。`help` は `u32::from(...)` という変換を提案しています（前のページの `as u32` でも解決できます）。

## やってみよう

`let voltage = 3;` を宣言して、「`voltage` が1以上 かつ 5以下なら `true`」になる `is_safe` を作り、表示してみましょう。値を 6 に変えると `false` になることも確かめてください。

## 確認問題

1. `temp == 30` と `temp = 30` の違いは何でしょうか?
2. 「10未満、または100より大きい」を表す式を `x` を使って書いてください。
3. C++では `if (1)` が通りますが、Rustで数値を条件に使えないのはなぜ安全につながるのでしょうか?

<details>
<summary>答え</summary>

1. `==` は「等しいかどうか」を比べて `bool` を返す比較。`=` は値を入れる代入で、条件にはなりません。
2. `x < 10 || x > 100`
3. 「数値を書いたが実は条件のつもりではなかった」「比較を書いたつもりが代入だった」という取り違えを、型の不一致としてコンパイル時に検出できるからです。

</details>

## まとめ

- `bool` は `true` / `false` だけの型。数値の0/1では代用できない
- 比較演算子の結果は `bool`。範囲は `a <= x && x <= b` のように2つの比較で書く
- `expected 〜, found 〜` は「期待した型と実際の型」を示す型エラーの定番表現

## 次のページ

同じ計算や判定を何度も書くのは大変です。処理に名前をつけて再利用する「関数」を次のページで学びます。

[5. 関数 →](/embassy-esp32-c6/part02/05-functions/)

---

- 前のページ: [3. 数値型](/embassy-esp32-c6/part02/03-numbers/)
- 次のページ: [5. 関数](/embassy-esp32-c6/part02/05-functions/)
