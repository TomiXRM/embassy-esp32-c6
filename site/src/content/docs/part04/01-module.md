---
title: "1. moduleで整理する"
description: modを使ってプログラムを名前空間に分け、1つのloopに全部書くスタイルから卒業する第一歩です。
part: 4
lesson: 1
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part03/06-methods
  - part03/10-lifetime
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1（ホストPCでcargo check/run済み）"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
  - https://doc.rust-lang.org/reference/items/modules.html
---

## このページでできるようになること

- なぜプログラムを分割するのかを、自分の言葉で説明できる
- `mod` でコードをグループ（module）に分けられる
- `モジュール名::関数名` の形で呼び出せる
- `use` で呼び出しを短く書ける

## 先に結論

プログラムが大きくなると、1つのファイルに全部書くスタイルは必ず破綻します。Rustには **module（モジュール）** という「コードを名前付きのグループに分ける仕組み」があり、`mod` キーワードで作ります。module の中の関数は `led::on()` のように「グループ名::関数名」で呼びます。毎回長く書きたくないときは `use` で短縮できます。この第4部では、module から始めて trait やエラー設計まで、「大きなプログラムを壊さずに育てる技術」を学びます。

## 身近なたとえ

工具箱を思い浮かべてください。仕切りのない箱にドライバーもネジも電池も全部放り込むと、最初のうちは困りませんが、道具が増えるほど目当ての物を探す時間が増えます。仕切りを付けて「ドライバーの区画」「ネジの区画」と分ければ、探す場所が一目で分かります。

module はこの「仕切り」です。ただし実際の技術との違いを一言添えると、module は**実行時には何もしません**。プログラムの動きを変える仕組みではなく、コンパイル時に「名前の重複を防ぎ、コードの置き場所を整理する」ための仕組みです。仕切りを付けても工具（コード）の性能は変わらない、という点だけは工具箱と同じです。

## 仕組み — なぜ分けるのか

Arduinoの `loop` に全部書くスタイルを思い出してください（第1部2ページ）。

```cpp
void loop() {
  // ボタンを読んで、チャタリングを取って、LEDを制御して、
  // シリアルに出力して、タイミングも計算して……全部ここに書く
}
```

機能が3つくらいまでなら耐えられます。しかし「ボタン + LED + 通信 + 省電力」と増えていくと、次の問題が起きます。

- 変数名がぶつかる（`count` はボタンの数？ 送信の数？）
- どの行がどの機能のためのコードか、読んでも分からなくなる
- 1か所直すと別の機能が壊れる

module は最初の対策です。**関係するコードに名前を付けてまとめ、境界線を引きます。**

## RustとEmbassyではどう書くか

まずは1ファイルの中で module に分けてみます。次のコードは完全なプログラムで、[Rust Playground](https://play.rust-lang.org/) にそのまま貼り付けて動かせます。

```rust
mod led {
    pub fn on() {
        println!("LED ON");
    }
    pub fn off() {
        println!("LED OFF");
    }
}

mod button {
    pub fn is_pressed() -> bool {
        // 本当はGPIOを読む。ここでは仮の値
        true
    }
}

use led::on;

fn main() {
    if button::is_pressed() {
        on(); // useしたので短く呼べる
    } else {
        led::off(); // フルパスでも呼べる
    }
}
```

まだLEDは `println!` の偽物ですが、「LED担当」と「ボタン担当」の境界線がはっきりしました。本物のGPIO制御に置き換えるのは第6部で行います。構造は今のうちに身に付けておきます。

## コードを一行ずつ読む

- `mod led { ... }` — 「ここからここまでは led という名前のグループです」という宣言です。`{}` の中がグループの中身です。
- `pub fn on()` — `pub` は「グループの外から使ってよい」という印です。`pub` を付けない関数は module の外から呼べません（詳しくは3ページ目）。
- `button::is_pressed()` — `::` は「〜の中の」と読みます。「button の中の is_pressed」です。
- `use led::on;` — 「この先 `on` と書いたら `led::on` のことだ」という短縮の宣言です。よく使う名前だけ `use` するのがコツです。

## 実行方法

Rust Playground にコードを貼り付けて「Run」を押します。

```text
LED ON
```

`button::is_pressed()` が `true` を返すので、LED ON が表示されます。仮の値を `false` に変えると LED OFF に変わります。

## よくある失敗

**1. `pub` を付け忘れる**

```text
error[E0603]: function `on` is private
```

module の中身は**最初はすべて非公開**です。外から呼びたい関数に `pub` が付いていないと、このエラーになります。「勝手に外から触られたくないものを守る」ための既定値なので、必要なものにだけ `pub` を付けます。

**2. パスを省略しすぎる**

`use led::on;` を書いていないのに `on()` と呼ぶと、`cannot find function 'on'`（そんな関数は見つからない）と言われます。コンパイラは module の境界を越えて勝手に探しません。`led::on()` とフルパスで書くか、`use` を書きます。

## やってみよう

上のコードに `buzzer`（ブザー）module を追加して、`pub fn beep()` で `println!("BEEP")` を出してみましょう。`main` の `on()` の直後に `buzzer::beep();` を足して、表示が2行になれば成功です。5分でできます。

## 確認問題

1. module の中の関数を外から呼べるようにするキーワードは何ですか？
2. `use button::is_pressed;` と書いた後、この関数はどう呼べますか？
3. module はプログラムの実行速度を変えますか？

<details>
<summary>答え</summary>

1. `pub` です。付けないと非公開のままです。
2. `is_pressed()` と短く呼べます（`button::is_pressed()` のままでも呼べます）。
3. 変えません。module はコンパイル時の整理の仕組みで、実行時のコストはありません。

</details>

## まとめ

- 1ファイル・1関数に全部書くスタイルは、機能が増えると必ず破綻する
- `mod 名前 { ... }` でコードをグループ化し、`名前::関数名` で呼ぶ
- 外に見せるものにだけ `pub` を付け、よく使う名前は `use` で短縮する

## 次のページ

module に分けても、1つのファイルが長くなる問題は残っています。次は module を**別ファイル**に切り出して、`mod` 宣言とファイルがどう対応するかを学びます。

[2. ファイル分割](/embassy-esp32-c6/part04/02-file-split/)

---

前のページ: [10. ライフタイムの直感](/embassy-esp32-c6/part03/10-lifetime/)
