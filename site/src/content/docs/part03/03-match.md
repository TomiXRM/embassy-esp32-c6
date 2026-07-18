---
title: "3. matchで場合分けする"
description: matchによる場合分けと、書き忘れをコンパイラが見つけてくれる網羅性チェックを学びます。
part: 3
lesson: 3
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part03/02-enum
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 (edition 2024)"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch06-02-match.html
  - https://doc.rust-jp.rs/book-ja/ch06-02-match.html
---

## このページでできるようになること

- matchでenumや数値を場合分けできる
- 網羅性チェック（すべての場合を書いたかの確認）の利点を説明できる
- matchを「式」として使い、結果を変数や戻り値にできる

## 先に結論

matchは「値の形に応じて処理を選ぶ」道具です。ifとの最大の違いは**網羅性チェック**にあります。matchは取りうる場合を**すべて**書かないとコンパイルが通りません。enumにバリアントを追加すると、処理を書き忘れたmatchが全部コンパイルエラーになって見つかります。「書き忘れが実行前に必ず見つかる」ことは、動かしてみないとバグが分からない従来のやり方との大きな違いです。さらにmatchは式なので、場合分けの結果をそのまま変数や戻り値にできます。

## 身近なたとえ

matchは、郵便物の仕分け棚のようなものです。届いた郵便（値）を、宛先の形に合う棚（腕、armと呼びます）へ入れます。ポイントは「どの棚にも入らない郵便があってはいけない」というルールで、仕分け表に漏れがあると棚の管理者（コンパイラ）が受け取りを拒否します。

たとえと違うのは、実際のmatchは**上から順に試して、最初に合った腕だけ**が実行されることです。複数の棚に入りそうな郵便でも、先に書いた腕が勝ちます。

## 仕組み

前のページのLEDコマンドで見てみます。

```rust
enum LedCommand {
    Off,
    On,
    Blink { interval_ms: u32 },
}
```

このenumをmatchで場合分けするとき、`Blink`を書き忘れるとどうなるでしょうか。

```rust
match cmd {
    LedCommand::Off => println!("消灯"),
    LedCommand::On => println!("点灯"),
    // Blinkを書き忘れた
}
```

```text
error[E0004]: non-exhaustive patterns: `LedCommand::Blink { .. }` not covered
  |
9 |     match cmd {
  |           ^^^ pattern `LedCommand::Blink { .. }` not covered
  |
help: ensure that all possible cases are being handled by adding a match arm
with a wildcard pattern or an explicit pattern as shown
  |
  |         LedCommand::Blink { .. } => todo!(),
```

エラーメッセージを読んでみましょう。「non-exhaustive patterns（網羅していないパターン）」、つまり「`Blink`の場合が書かれていない」と場所つきで教えてくれています。これが**網羅性チェック**です。プログラムを動かす前に、場合分けの漏れが必ず見つかります。

C++のswitchでは、caseを書き忘れても警告止まりで、実行して初めて「何も起きない」バグに気づくことがよくあります。Rustではそのバグはコンパイルの段階で存在できません。

## Rustではどう書くか

matchを「式」として使う例です。Rust Playgroundでそのまま動きます。

```rust
enum LedCommand {
    Off,
    On,
    Blink { interval_ms: u32 },
}

// matchは「式」なので、結果を返せる
fn power_needed(cmd: &LedCommand) -> &str {
    match cmd {
        LedCommand::Off => "消費なし",
        LedCommand::On => "常に消費",
        LedCommand::Blink { interval_ms } if *interval_ms < 100 => "ほぼ常に消費",
        LedCommand::Blink { .. } => "半分くらい消費",
    }
}

fn main() {
    let commands = [
        LedCommand::Off,
        LedCommand::On,
        LedCommand::Blink { interval_ms: 50 },
        LedCommand::Blink { interval_ms: 500 },
    ];

    for cmd in &commands {
        println!("{}", power_needed(cmd));
    }

    // 数値のmatch: 範囲やその他をまとめられる
    let raw: u16 = 700;
    let level = match raw {
        0..=99 => "暗い",
        100..=899 => "ふつう",
        _ => "明るい",
    };
    println!("明るさ: {}", level);
}
```

## コードを一行ずつ読む

- `match cmd { パターン => 結果, ... }` — 各行が「腕」です。`=>`の左がパターン、右が処理
- 関数`power_needed`の本体はmatch式そのものです。合った腕の値（`"消費なし"`など）が、そのまま関数の戻り値になります。全部の腕が同じ型（ここでは`&str`）を返す必要があります
- `LedCommand::Blink { interval_ms } if *interval_ms < 100` — `if`付きの腕は**マッチガード**と呼びます。パターンに合い、かつ条件が真のときだけ選ばれます。`*`は参照から値を取り出す印で、[9. 借用](/embassy-esp32-c6/part03/09-borrow/)で詳しく学びます
- `LedCommand::Blink { .. }` — `..`は「中のデータは使わない」という省略記法です
- `0..=99` — 数値は範囲でもマッチできます。`0..=99`は0以上99以下です
- `_` — 「それ以外すべて」のパターンです。`u16`の全値（0〜65535）を範囲で書き切るのは大変なので、残りを`_`でまとめて網羅性を満たします

`_`は便利ですが、enumのmatchで安易に使うと網羅性チェックの恩恵が消えます。`_`があると、バリアントを追加してもエラーにならず、書き忘れに気づけなくなるからです。enumでは、できるだけバリアントを全部書くのがおすすめです。

## 実行方法

[Rust Playground](https://play.rust-lang.org/)にコードを貼り付けて「Run」を押します。

```text
消費なし
常に消費
ほぼ常に消費
半分くらい消費
明るさ: ふつう
```

`interval_ms: 50`のBlinkだけがマッチガードに引っかかり、「ほぼ常に消費」になっている点に注目してください。

## よくある失敗

### 場合の書き忘れ（E0004）

上の「仕組み」で見た通りです。エラーメッセージの`not covered`の後ろに、**足りないパターンがそのまま書いてあります**。コンパイラの提案（`LedCommand::Blink { .. } => todo!(),`）を貼り付けて、`todo!()`を自分の処理に置き換えるのが早い直し方です。

### 腕ごとに型が違う（E0308）

```rust
let level = match raw {
    0..=99 => "暗い",   // &str
    _ => 1,             // 整数を返してしまった
};
```

match式全体がひとつの値になるので、腕ごとに違う型は返せません。「expected `&str`, found integer」のようなE0308（型の不一致）になります。全部の腕を同じ型に揃えます。

### matchの後のセミコロン忘れ

matchを式として`let level = match ... };`のように使うときは、最後に`;`が必要です。文として使うとき（値を使わないとき）は不要です。エラーメッセージに`help: consider adding a semicolon`と出たら、この違いを思い出してください。

## やってみよう

`LedCommand`に`Breath { period_ms: u32 }`（ゆっくり明滅する呼吸パターン）を追加してみましょう。E0004が2か所以上で出るはずです。エラーメッセージの`not covered`を読みながら全部直してください。「変更の影響範囲をコンパイラが列挙してくれる」体験が、この教材で何度も出てくるRustらしい開発の流れです。

## 確認問題

1. 網羅性チェックとは何ですか。何がうれしいのですか。
2. enumのmatchで`_`を多用しない方がよい理由は何ですか。
3. 「matchは式である」とは、どういう意味ですか。

<details>
<summary>答え</summary>

1. matchが取りうる場合をすべて書いているか、コンパイラが確認すること。場合の書き忘れが実行前に必ず見つかる。
2. `_`が残りを全部受けてしまうので、バリアントを追加しても書き忘れがエラーにならず、網羅性チェックの恩恵が消えるから。
3. match全体がひとつの値になるということ。結果を変数に入れたり、関数の戻り値にしたりできる。そのため全腕の型を揃える必要がある。

</details>

## まとめ

- matchは値の形で場合分けする道具。上から順に試し、最初に合った腕が実行される
- 網羅性チェックにより、場合の書き忘れはコンパイルエラーになる。enumでは`_`に頼りすぎない
- matchは式。場合分けの結果をそのまま値として使える

## 次のページ

matchで扱う型の中で、一番よく使うのがOptionです。「値がないかもしれない」をnullではなく型で表す、Rustの安全性の要を学びます。

- 前のページ: [2. enumで選択肢を表す](/embassy-esp32-c6/part03/02-enum/)
- 次のページ: [4. Option — 「ないかもしれない」を型で表す](/embassy-esp32-c6/part03/04-option/)
