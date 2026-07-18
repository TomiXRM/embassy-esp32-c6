---
title: "7. implと関連関数"
description: new()のような関連関数と関連定数を定義し、値の作り方を型に持たせる方法を学びます。
part: 3
lesson: 7
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part03/06-methods
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 (edition 2024)"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch05-03-method-syntax.html#associated-functions
  - https://doc.rust-jp.rs/book-ja/ch05-03-method-syntax.html
---

## このページでできるようになること

- selfを取らない**関連関数**（`new()`など）を定義できる
- `Self`と関連定数を使って、値の作り方を型にまとめられる
- `型名::関数()`と`値.メソッド()`の呼び分けができる

## 先に結論

implブロックには、`self`を取らない関数も書けます。これを**関連関数**と呼び、`BlinkConfig::new(200, 800)`のように`型名::関数名()`で呼びます。まだ値が存在しない段階で呼ぶ「値を作るための関数」が代表で、慣習として`new`という名前を使います。関連関数の中に「最低値の補正」のようなチェックを入れておけば、**その型の値は必ず正しい状態で生まれる**ことを保証できます。Rustには特別なコンストラクタ構文がなく、この`new`の慣習がその役割を担います。

## 身近なたとえ

メソッドが「機器に付いた専用ボタン」なら、関連関数は「その機器の工場」です。あたためボタンはレンジ本体がないと押せませんが、工場はレンジがまだ存在しない段階で動いて、レンジを作り出します。工場には検品もあり、規格外の製品は出荷前に直されます。

たとえと違う点として、関連関数は「値を作る関数」に限られません。selfを取らない関数なら何でも書けます。「この型に関係が深い道具を、型の名前の下にまとめて置く」のが関連関数の本質です。

## 仕組み

LED点滅の設定を例にします。

```rust
impl BlinkConfig {
    // 関連定数: この型に関係する決まった値
    const MIN_MS: u32 = 10;

    // 関連関数: selfを取らない。作るための関数
    fn new(on_ms: u32, off_ms: u32) -> Self {
        Self {
            on_ms: on_ms.max(Self::MIN_MS),
            off_ms: off_ms.max(Self::MIN_MS),
        }
    }
}
```

- `fn new(...) -> Self` — 引数に`self`が**ない**ので関連関数です。`Self`（大文字始まり）は「このimplの対象の型」、ここでは`BlinkConfig`の別名です
- `Self { on_ms: ..., off_ms: ... }` — structの値を作って返しています。`BlinkConfig { ... }`と書いても同じですが、`Self`なら型名を変えたときに直す場所が減ります
- `const MIN_MS: u32 = 10;` — **関連定数**です。型に関係する決まり値を、グローバル定数ではなく型の下に置けます。`Self::MIN_MS`または`BlinkConfig::MIN_MS`で参照します
- `.max(Self::MIN_MS)` — 2つの値の大きい方を返す数値のメソッドです。10未満の指定を10に引き上げる「検品」をしています

なぜフィールドを直接書いて作らないのでしょうか。`BlinkConfig { on_ms: 0, off_ms: 0 }`のような「動かない設定」を作れてしまうからです。`new`経由に揃えれば、補正やチェックを1か所に集められます。

## Rustではどう書くか

Rust Playgroundでそのまま動きます。

```rust
struct BlinkConfig {
    on_ms: u32,  // 点灯時間
    off_ms: u32, // 消灯時間
}

impl BlinkConfig {
    // 関連定数: この型に関係する決まった値
    const MIN_MS: u32 = 10;

    // 関連関数: selfを取らない。作るための関数
    fn new(on_ms: u32, off_ms: u32) -> Self {
        Self {
            on_ms: on_ms.max(Self::MIN_MS),
            off_ms: off_ms.max(Self::MIN_MS),
        }
    }

    // よく使う設定に名前を付けた関連関数
    fn slow() -> Self {
        Self::new(1000, 1000)
    }

    // こちらは普通のメソッド
    fn period_ms(&self) -> u32 {
        self.on_ms + self.off_ms
    }
}

fn main() {
    let custom = BlinkConfig::new(200, 800);
    println!("周期: {} ミリ秒", custom.period_ms());

    let slow = BlinkConfig::slow();
    println!("ゆっくり点滅の周期: {} ミリ秒", slow.period_ms());

    // 小さすぎる値は最低値に直される
    let fixed = BlinkConfig::new(1, 1);
    println!("補正後の周期: {} ミリ秒", fixed.period_ms());
}
```

## コードを一行ずつ読む

- `BlinkConfig::new(200, 800)` — 関連関数は`型名::関数名()`で呼びます。`::`は「型の中の名前」をたどる記号です
- `fn slow() -> Self { Self::new(1000, 1000) }` — よく使う設定に名前を付けた関連関数です。呼び出し側は`BlinkConfig::slow()`と書くだけで意図が伝わります。中で`new`を通しているので検品も効いています
- `custom.period_ms()` — こちらは`&self`を取るメソッドなので、作った値に対して`.`で呼びます。**「作るまでは`::`、作ってからは`.`」**と覚えてください
- `BlinkConfig::new(1, 1)` — 小さすぎる指定が`MIN_MS`へ補正され、周期は20ミリ秒になります。壊れた値が生まれない、というのが`new`に検品を置く効果です

この形は、この教材でこれから何度も出てきます。esp-halでも`Config::default()`のような関連関数で設定を作り、ペリフェラル（周辺機器）を初期化する流れが基本形です。

## 実行方法

[Rust Playground](https://play.rust-lang.org/)にコードを貼り付けて「Run」を押します。

```text
周期: 1000 ミリ秒
ゆっくり点滅の周期: 2000 ミリ秒
補正後の周期: 20 ミリ秒
```

## よくある失敗

### 関連関数を`.`で呼ぶ（E0599）

```rust
let config = BlinkConfig { on_ms: 200, off_ms: 800 };
let another = config.new(100, 100); // 値からnewを呼ぼうとした
```

```text
error[E0599]: no method named `new` found for struct `BlinkConfig` in the current scope
   |
14 |     let another = config.new(100, 100);
   |                          ^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods,
     functions must have a `self` parameter
help: use associated function syntax instead
   |
14 +     let another = BlinkConfig::new(100, 100);
```

エラーが仕組みをそのまま説明してくれています。「これは関連関数でありメソッドではない。メソッドとして使うには`self`引数が必要」。`new`はselfを取らないので、`BlinkConfig::new(...)`と型名から呼びます。

### Selfと戻り値の書き忘れ（E0308）

`fn new(...) { Self { ... } }`のように戻り値の型`-> Self`を書き忘れると、「expected `()`, found `BlinkConfig`」というE0308になります。戻り値を書かない関数は「何も返さない」（`()`を返す）扱いだからです。作る関数には必ず`-> Self`を付けます。

## やってみよう

`fast()`（点灯100ミリ秒・消灯100ミリ秒）という関連関数を追加して、`slow()`と同じように使ってみましょう。さらに`BlinkConfig::new(0, 5000)`を作って、`on_ms`だけが補正されることを確かめてください。

## 確認問題

1. メソッドと関連関数の違いは何ですか。呼び方はどう変わりますか。
2. `Self`は何を指しますか。
3. フィールドを直接書いて値を作らず、`new`を通す利点は何ですか。

<details>
<summary>答え</summary>

1. メソッドは`self`を取り、値に対して`値.メソッド()`で呼ぶ。関連関数は`self`を取らず、`型名::関数()`で呼ぶ。
2. そのimplブロックの対象の型（例では`BlinkConfig`）。
3. 補正や検査を1か所に集められるので、不正な状態の値が最初から作られないことを保証できる。

</details>

## まとめ

- 関連関数はselfを取らない、型に属する関数。`型名::関数()`で呼ぶ。値を作る`new`が代表
- `Self`はimpl対象の型の別名。関連定数で型に関係する決まり値も置ける
- `new`に検品を集めると「壊れた値が生まれない」ことを保証できる。esp-halの初期化もこの形

## 次のページ

型と操作を作る道具は揃いました。ここからこの部の核心、「そのデータは誰が持っていて、いつまで存在するのか」——所有権の話に入ります。

- 前のページ: [6. メソッドを定義する](/embassy-esp32-c6/part03/06-methods/)
- 次のページ: [8. 所有権 — 誰がデータを持つのか](/embassy-esp32-c6/part03/08-ownership/)
