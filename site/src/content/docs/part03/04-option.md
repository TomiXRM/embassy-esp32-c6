---
title: "4. Option — 「ないかもしれない」を型で表す"
description: 値が存在しない可能性をOption型で表し、unwrapに頼らずmatchやif letで安全に扱う方法を学びます。
part: 3
lesson: 4
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part03/03-match
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 (edition 2024)"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values
  - https://doc.rust-lang.org/core/option/enum.Option.html
---

## このページでできるようになること

- 「値がないかもしれない」状況をOption型で表せる
- unwrapに頼らず、match・if let・unwrap_orでOptionを扱える
- unwrapがなぜ危険か、いつなら許されるかを説明できる

## 先に結論

プログラムには「値があるとは限らない」場面がたくさんあります。探し物が見つからない、センサがまだ準備できていない、設定がされていない——。多くの言語はこれをnull（何もないことを表す特別な値）で表しますが、nullは「うっかり普通の値のつもりで使う」事故の温床です。Rustは`Option<T>`という**enum**でこれを表します。「ないかもしれない値」は普通の値と型が違うので、「ない場合の処理」を書かない限り中身を取り出せません。null事故がコンパイルエラーに変わる、というのがOptionの価値です。

## 身近なたとえ

Optionは、中身が入っているか分からない「お弁当箱」のようなものです。箱を開ける（matchする）までは、中身をかじることはできません。「開けたら空だった」場合にどうするかを、開ける人が必ず決めることになります。

たとえと違う点として、実際の`Option<T>`は運まかせではありません。**型として**「空かもしれない」と宣言されているので、空の場合の扱いを書き忘れるとコンパイルが通りません。確認を強制するのは箱ではなくコンパイラです。

## 仕組み

`Option<T>`は標準ライブラリにある、ただのenumです。自分で定義するならこう書けます。

```rust
enum Option<T> {
    Some(T), // 値がある（中身つき）
    None,    // 値がない
}
```

- `T`は「中身の型」を表す型引数です。`Option<usize>`なら「usizeがあるかもしれない」型
- 前のページまでに学んだ「データ付きenum + match」がそのまま使われています。Optionは特別な文法ではありません
- `Some`と`None`はよく使うので、`Option::`を付けずにそのまま書けます

大事なのは、**`Option<usize>`と`usize`は別の型**だということです。「あるかもしれない値」をうっかり足し算に使うことはできず、必ず一度「ある/ない」の場合分けを通ります。

## Rustではどう書くか

I2Cバス（第8部で学ぶ通信線）の上で見つかったデバイスのアドレス一覧から、目的のアドレスを探す例です。「見つからないかもしれない」検索は、Optionの一番典型的な使い場所です。Rust Playgroundでそのまま動きます。

```rust
// 見つかったデバイスのアドレス一覧から、targetを探す
// 見つかれば位置（何番目か）を返す
fn find_device(addresses: &[u8], target: u8) -> Option<usize> {
    for (i, &addr) in addresses.iter().enumerate() {
        if addr == target {
            return Some(i); // 見つかった
        }
    }
    None // 見つからなかった
}

fn main() {
    let found = [0x3C, 0x48, 0x76];

    // matchで両方の場合を書く
    match find_device(&found, 0x48) {
        Some(i) => println!("0x48は{}番目にあります", i),
        None => println!("0x48は見つかりません"),
    }

    // 「あるときだけ」ならif letが短い
    if let Some(i) = find_device(&found, 0x76) {
        println!("0x76は{}番目にあります", i);
    }

    // 「ないときの代わりの値」を決めるならunwrap_or
    let pos = find_device(&found, 0x99).unwrap_or(0);
    println!("0x99の位置（なければ0）: {}", pos);
}
```

## コードを一行ずつ読む

- `fn find_device(...) -> Option<usize>` — 戻り値の型が「見つからないかもしれない」ことを宣言しています。この関数を呼ぶ人は、シグネチャを見ただけで「Noneの処理が要る」と分かります
- `.iter().enumerate()` — 第2部のforで学んだ書き方で、「番号と値のペア」を順に取り出します
- `match find_device(...)` — Optionの基本の扱い方です。`Some(i)`の腕では中身が変数`i`として使えます。matchなので`None`を書き忘れるとE0004（網羅性エラー）になります
- `if let Some(i) = ...` — 「Someのときだけ処理し、Noneなら何もしない」の短縮形です。matchで`None => {}`と書くのと同じ意味です
- `.unwrap_or(0)` — 「Someなら中身、Noneなら0」という意味です。「ない場合の代わりの値」が決まっているときに便利です

### unwrapについて

`Option`には`.unwrap()`というメソッドもあります。「Someなら中身を返し、**Noneならその場でプログラムを止める（panic）**」という乱暴な取り出し方です。

```text
thread 'main' panicked at src/main.rs:12:41:
called `Option::unwrap()` on a `None` value
```

パソコンならプログラムが終了するだけですが、マイコンでは**機器全体が止まる**ことを意味します。unwrapは「設計上Noneが絶対にありえない」と説明できる場所に限定し、基本はmatch・if let・unwrap_orで「ない場合」を自分で決めてください。この方針は教材全体で徹底します（詳しくは第5部のpanicのページで扱います）。

## 実行方法

[Rust Playground](https://play.rust-lang.org/)にコードを貼り付けて「Run」を押します。

```text
0x48は1番目にあります
0x76は2番目にあります
0x99の位置（なければ0）: 0
```

番号が0始まりであることに注意してください（0x48は先頭から2つ目ですが「1番目」と表示されます）。

## よくある失敗

### Optionを中身の型として使う（E0308）

```rust
let pos: usize = find_device(&found, 0x48); // Option<usize>をusizeに入れようとした
```

```text
error[E0308]: mismatched types
   |
12 |     let pos: usize = find_device(&found, 0x48);
   |              -----   ^^^^^^^^^^^^^^^^^^^^^^^^^ expected `usize`, found `Option<usize>`
   |
help: consider using `Option::expect` to unwrap the `Option<usize>` value,
panicking if the value is an `Option::None`
```

`Option<usize>`と`usize`は別の型なので直接は代入できません。これはエラーというより**Optionの安全装置が働いている**状態です。matchなどで場合分けして取り出します。なお、コンパイラの`help`は`expect`（unwrapの仲間）を提案してきますが、上で述べた通り、まず「Noneのときどうしたいか」を考えるのが正しい直し方です。

### Noneをunwrapして実行時に止まる

```rust
let pos = find_device(&found, 0x99).unwrap(); // 0x99は存在しない
```

コンパイルは通りますが、実行すると上で見たpanicで止まります。「コンパイルが通った＝安全」ではなく、unwrapは「止まってもよい」と自分で宣言する行為だと覚えてください。

## やってみよう

`find_device(&found, 0x99)`の結果をmatchで処理し、Noneの腕で「見つかりません。配線を確認してください」と表示してみましょう。実機開発では、この「ない場合」のメッセージがそのままトラブルシューティングの入り口になります。

## 確認問題

1. nullのある言語と比べて、Optionの何が安全なのですか。
2. `unwrap()`と`unwrap_or(0)`の違いは何ですか。
3. 「Someのときだけ何かして、Noneなら無視したい」とき、matchより短く書ける構文は何ですか。

<details>
<summary>答え</summary>

1. 「ないかもしれない値」が普通の値と別の型になっているので、「ない場合」の処理を書かないとコンパイルが通らない。nullをうっかり使う事故がコンパイルエラーに変わる。
2. `unwrap()`はNoneのときpanicでプログラムが止まる。`unwrap_or(0)`はNoneのとき代わりの値0を返し、止まらない。
3. `if let Some(x) = ...`。

</details>

## まとめ

- `Option<T>`は「値がないかもしれない」を表すただのenum。`Some(値)`か`None`のどちらか
- `Option<T>`と`T`は別の型。場合分けを通らないと中身は使えない——これがnull事故を防ぐ
- 取り出しはmatch・if let・unwrap_orが基本。unwrapは「Noneなら止まる」ことを理解して限定的に使う

## 次のページ

「ないかもしれない」の次は「失敗するかもしれない」です。失敗の理由も一緒に運べるResult型と、エラー処理を簡潔にする?演算子を学びます。

- 前のページ: [3. matchで場合分けする](/embassy-esp32-c6/part03/03-match/)
- 次のページ: [5. Result — 失敗を型で表す](/embassy-esp32-c6/part03/05-result/)
