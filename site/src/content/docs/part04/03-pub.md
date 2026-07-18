---
title: "3. pubと公開範囲"
description: 何を公開し何を隠すかを設計します。pub、pub(crate)、非公開フィールドの使い分けを学びます。
part: 4
lesson: 3
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part04/02-file-split
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1（ホストPCでcargo check/run済み）"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html
  - https://doc.rust-lang.org/reference/visibility-and-privacy.html
---

## このページでできるようになること

- 「既定で非公開」の理由を説明できる
- `pub` と `pub(crate)` を使い分けられる
- フィールドを隠して、メソッド経由でだけ触らせる設計ができる

## 先に結論

Rustでは module の中身は**既定ですべて非公開**です。外から使わせたいものにだけ `pub` を付けます。公開範囲を絞る中間の指定として `pub(crate)`（同じcrate内だけ公開）もあります。structでは「struct自体は `pub`、フィールドは非公開」にして、値の変更をメソッド経由に限定するのが定石です。**公開したものは後から変えにくくなる**ので、迷ったら非公開にしておきます。

## 身近なたとえ

自動販売機を思い浮かべてください。ボタンとコイン投入口（公開されている操作）だけが外に出ていて、お金を数える仕組みや在庫の棚（内部の仕組み）は扉の中に隠れています。誰でも棚を直接触れたら、お金を入れずに商品を取れてしまいます。

実際の技術との違いを一言添えると、`pub` はセキュリティ機能ではありません。悪意ある人を止める鍵ではなく、**チームメイトや未来の自分がうっかり内部に依存するのを防ぐ**、コンパイル時の約束です。

## 仕組み

公開範囲の指定は主に3段階です。

| 書き方 | 誰から見えるか |
|---|---|
| （何も付けない） | 同じ module の中だけ |
| `pub(crate)` | 同じ crate（プロジェクト）の中だけ |
| `pub` | どこからでも |

自分のプログラムの中で使う分には `pub` と `pub(crate)` の差は出ませんが、第4ページで学ぶ「ライブラリとして公開されるcrate」では大きな差になります。「crateの外にまで見せたいのか？」と自問して選びます。

## RustとEmbassyではどう書くか

「ボタンが押された回数」を数えるカウンタを、**外から直接書き換えられない**形で作ります。Playgroundで動く完全なコードです。

```rust
mod counter {
    pub struct PressCounter {
        count: u32, // 非公開フィールド
    }

    impl PressCounter {
        pub fn new() -> Self {
            PressCounter { count: 0 }
        }

        pub fn press(&mut self) {
            self.count = self.count.saturating_add(1);
        }

        pub fn count(&self) -> u32 {
            self.count
        }
    }
}

use counter::PressCounter;

fn main() {
    let mut c = PressCounter::new();
    c.press();
    c.press();
    println!("{}回押されました", c.count());
    // c.count = 9999; // エラー: countは非公開
}
```

## コードを一行ずつ読む

- `pub struct PressCounter` — 型そのものは公開します。外で変数の型として使えるようにするためです。
- `count: u32` — フィールドに `pub` を付けていません。**module の外からは読むことも書くこともできません。**
- `pub fn press(&mut self)` — 変更はこのメソッド経由だけです。`saturating_add(1)` は「上限に達したらそれ以上増やさない足し算」で、u32があふれて0に戻る事故を防ぎます。
- `pub fn count(&self) -> u32` — 読み取り専用の窓口です。これで「増えることはあっても、勝手に書き換えられることはない」と保証できます。

最後のコメント行を外すと、コンパイラは `field 'count' of struct 'PressCounter' is private` と教えてくれます。この設計なら「countがおかしな値になった」というバグの容疑者は `press()` だけになり、調べる範囲が一気に狭まります。

## 実行方法

Rust Playground に貼り付けて Run します。

```text
2回押されました
```

## よくある失敗

**1. structは公開したのにフィールドで詰まる**

`pub struct` にしただけでは、外から `PressCounter { count: 0 }` と直接作ることはできません（E0451: フィールドが非公開のため）。だからこそ `new()` のような**関連関数を公開して入口にする**のが定石です（第3部7ページの復習です)。

**2. とりあえず全部 `pub` にしてしまう**

エラーは消えますが、あらゆるコードが内部フィールドに依存できてしまいます。半年後に `count` の型を変えたくなったとき、直す場所がプロジェクト中に散らばります。「公開は借金」と考えて、最小限から始めるのが安全です。

## やってみよう

`PressCounter` に `pub fn reset(&mut self)`（countを0に戻す）を追加してみましょう。`main` で `c.reset();` を呼び、`0回押されました` と表示されれば成功です。

## 確認問題

1. `pub` を何も付けなかった関数は、どこから呼べますか？
2. structを `pub` にして、フィールドを非公開にすると何がうれしいですか？
3. `pub(crate)` はどんなときに `pub` と差が出ますか？

<details>
<summary>答え</summary>

1. 同じ module の中からだけ呼べます。
2. 値の変更が公開メソッド経由に限定され、不正な値になる経路を絞れます。内部の実装を後から自由に変えられます。
3. crateをライブラリとして他のプロジェクトから使うときです。`pub(crate)` の項目は外部からは見えません。
</details>

## まとめ

- 既定は非公開。外に見せたいものにだけ `pub` を付ける
- フィールドは隠し、`new()` とメソッドを公開の入口にするのが定石
- 公開範囲は後から広げるのは簡単、狭めるのは大変。迷ったら非公開

## 次のページ

module は自分のコードの整理でした。次は**他人の書いたコードのかたまり = crate** を自分のプロジェクトに取り込む方法、つまり Cargo.toml の依存関係を学びます。

[4. crateと依存関係](/embassy-esp32-c6/part04/04-crate/)

---

前のページ: [2. ファイル分割](/embassy-esp32-c6/part04/02-file-split/)
