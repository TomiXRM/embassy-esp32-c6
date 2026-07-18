---
title: "2. enumで選択肢を表す"
description: データ付きenumで「どれかひとつ」の状態を表す方法と、C++のenumとの違いを学びます。
part: 3
lesson: 2
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part03/01-struct
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 (edition 2024)"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
  - https://doc.rust-jp.rs/book-ja/ch06-01-defining-an-enum.html
---

## このページでできるようになること

- 「どれかひとつ」の状態をenum（列挙型）で定義できる
- バリアントにデータを持たせた**データ付きenum**を書ける
- C++のenumとRustのenumの違いを説明できる

## 先に結論

enumは「取りうる選択肢を全部並べた型」です。ボタンなら「押された・離された・押しっぱなし」のどれかであり、同時に2つにはなりません。Rustのenumが強力なのは、**選択肢ごとに違うデータを持たせられる**ことです。「押しっぱなし（1500ミリ秒）」のように、状態と一緒にその状態にだけ意味のある値を運べます。C++のenumは実質ただの整数ですが、Rustのenumは「形の違うstructを選択肢として束ねたもの」に近い道具です。

## 身近なたとえ

enumは、天気予報の「晴れ・くもり・雨」のようなものです。今日の天気はこの中のどれかひとつで、「晴れと雨が同時」はありません。さらにRustのenumでは、「雨（降水量 10mm）」のように選択肢に情報を添えられます。晴れに降水量は要りませんが、雨には必要です。

たとえと違う点として、enumの選択肢（**バリアント**と呼びます）はコンパイル時に固定です。プログラムの実行中に「新しい天気」を追加することはできず、定義に書いたバリアント以外の値は存在できません。この「それ以外がない」保証が、次のページのmatchで効いてきます。

## 仕組み

ボタンの状態をenumで書いてみます。

```rust
enum ButtonEvent {
    Pressed,                   // 押された
    Released,                  // 離された
    Held { duration_ms: u32 }, // 押しっぱなし（継続時間つき）
}
```

- `Pressed`と`Released`はデータなしのバリアント
- `Held { duration_ms: u32 }`は**データ付きバリアント**。structのようにフィールドを持ちます
- `Held(u32)`のようにタプル型のデータを持たせる書き方もあります

値を作るときは`型名::バリアント名`と書きます。

```rust
let e1 = ButtonEvent::Pressed;
let e2 = ButtonEvent::Held { duration_ms: 1500 };
```

`e1`も`e2`も同じ`ButtonEvent`型です。だから同じ配列に入れたり、同じ関数に渡したりできます。「形が違うのに同じ型として扱える」のがデータ付きenumの便利なところです。

### C++のenumとの違い

Arduino（C++）のenumは、名前の付いた整数にすぎません。

```cpp
enum ButtonEvent { PRESSED, RELEASED, HELD };
// HELDの「継続時間」はenumに入れられない。
// 別のグローバル変数などで持つしかない
unsigned long heldDurationMs;
```

「継続時間」を別の変数で持つと、「今はHELDではないのにheldDurationMsに古い値が残っている」という食い違いが起きえます。Rustのデータ付きenumでは継続時間は`Held`の中にしか存在しないので、**状態とデータが食い違うことが型の仕組み上ありえません**。これがC++のenum（またはenum class）との一番大きな違いです。

## Rustではどう書くか

ボタンイベントの列を処理する例です。Rust Playgroundでそのまま動きます。

```rust
enum ButtonEvent {
    Pressed,                   // 押された
    Released,                  // 離された
    Held { duration_ms: u32 }, // 押しっぱなし（継続時間つき）
}

fn describe(event: &ButtonEvent) {
    match event {
        ButtonEvent::Pressed => println!("押されました"),
        ButtonEvent::Released => println!("離されました"),
        ButtonEvent::Held { duration_ms } => {
            println!("{} ミリ秒 押しっぱなしです", duration_ms);
        }
    }
}

fn main() {
    let events = [
        ButtonEvent::Pressed,
        ButtonEvent::Held { duration_ms: 1500 },
        ButtonEvent::Released,
    ];

    for event in &events {
        describe(event);
    }
}
```

## コードを一行ずつ読む

- `let events = [...]` — バリアントが違っても全部`ButtonEvent`型なので、ひとつの配列に入ります
- `match event { ... }` — enumの中身を場合分けして取り出すのが`match`です。詳しくは次のページで学ぶので、ここでは「バリアントごとに処理を書ける」と読んでください
- `ButtonEvent::Held { duration_ms } => ...` — データ付きバリアントは、matchのときに中のデータを変数として取り出せます。`Held`のときにしか`duration_ms`は存在しないので、取り違えが起きません

組み込みでは、こうしたenumが「taskからtaskへ送るイベント」の形として大活躍します。たとえば通信イベントなら次のように書けます。

```rust
enum UartEvent {
    ByteReceived(u8), // 1バイト受信した（その値つき）
    FrameError,       // 通信の形式エラー
}
```

これは第9部のtask間通信（Channel）でそのまま使う設計です。

## 実行方法

[Rust Playground](https://play.rust-lang.org/)にコードを貼り付けて「Run」を押します。

```text
押されました
1500 ミリ秒 押しっぱなしです
離されました
```

## よくある失敗

### 型名を付け忘れる（E0425）

```rust
let e = Pressed; // ButtonEvent:: を忘れた
```

```text
error[E0425]: cannot find value `Pressed` in this scope
  |
7 |     let e = Pressed;
  |             ^^^^^^^ not found in this scope
  |
help: consider importing this unit variant
  |
1 + use crate::ButtonEvent::Pressed;
```

バリアント名は単独では名前として見つかりません。`ButtonEvent::Pressed`と書くのが基本です。コンパイラは`use`で短縮する方法も提案していますが、この教材ではどのenumのバリアントか分かるように`型名::`を付けて書きます。

### データ付きバリアントのデータを忘れる（E0533）

```rust
let e = ButtonEvent::Held; // { duration_ms: ... } を忘れた
```

```text
error[E0533]: expected value, found struct variant `ButtonEvent::Held`
  |
7 |     let e = ButtonEvent::Held;
  |             ^^^^^^^^^^^^^^^^^ not a value
  |
help: you might have meant to create a new value of the struct
  |
7 |     let e = ButtonEvent::Held { duration_ms: /* value */ };
```

`Held`はデータ付きなので、データなしでは値になりません。定義した形の通り、`ButtonEvent::Held { duration_ms: 1500 }`とデータを添えて書きます。

## やってみよう

`ButtonEvent`に`DoubleClick { interval_ms: u32 }`（2回押しの間隔つき）を追加してみましょう。追加すると`describe`のmatchがエラーになります。これは次のページで学ぶ**網羅性チェック**が「新しいバリアントの処理を書き忘れているよ」と教えてくれている状態です。matchに1行足して動かしてください。

## 確認問題

1. structとenumの役割の違いを一言で説明してください。
2. C++のenumにできなくて、Rustのenumにできることは何ですか。
3. `ButtonEvent::Held { duration_ms: 1500 }`の`duration_ms`は、`Pressed`のときどこにありますか。

<details>
<summary>答え</summary>

1. structは「AもBも持つ」（全部同時に持つ）、enumは「AかBのどれか」（同時にはひとつ）を表す。
2. バリアントごとに違うデータを持たせられること（データ付きenum）。C++のenumは名前付きの整数でしかない。
3. どこにもない。`duration_ms`は`Held`バリアントの中にだけ存在し、他のバリアントのときは値そのものが存在しない。だから状態とデータの食い違いが起きない。

</details>

## まとめ

- enumは「取りうる選択肢を全部並べた型」。値は必ずどれかひとつのバリアント
- バリアントにはデータを持たせられる。状態とデータがセットになり、食い違いが型レベルでなくなる
- ボタン状態・通信イベントなど、組み込みの「状態」表現の主役になる

## 次のページ

enumの中身を取り出して場合分けする道具がmatchです。「選択肢を全部処理したか」をコンパイラが確認してくれる、網羅性チェックの心強さを体験します。

- 前のページ: [1. structでまとめる](/embassy-esp32-c6/part03/01-struct/)
- 次のページ: [3. matchで場合分けする](/embassy-esp32-c6/part03/03-match/)
