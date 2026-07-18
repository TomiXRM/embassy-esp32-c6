---
title: "6. generics"
description: 型引数を使って「型だけが違う同じ処理」を1回で書きます。dyn Traitとの使い分けも整理します。
part: 4
lesson: 6
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part04/05-trait
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1（ホストPCでcargo check/run済み）"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch10-01-syntax.html
  - https://doc.rust-lang.org/book/ch18-02-trait-objects.html
---

## このページでできるようになること

- 型引数 `<T>` 付きの関数とstructを書ける
- トレイト境界（`T: PartialOrd` など）の役割を説明できる
- ジェネリクスと `dyn Trait` の違いと使い分けが分かる

## 先に結論

**generics（ジェネリクス）** は「型だけが違う同じ処理」を1回で書く仕組みです。`fn max_of<T: PartialOrd>(a: T, b: T) -> T` のように、型の代わりに**型引数** `T` を置きます。`T` に何でも入るわけではなく、**トレイト境界**で「比較できる型なら何でも」と条件を付けます。コンパイラは使われた型ごとに専用版を自動生成する（単相化）ため実行時コストはゼロですが、そのぶんプログラムサイズは増えます。実行時に型を切り替えたいときだけ `dyn Trait` を使います。

## 身近なたとえ

クッキーの「抜き型」を考えてください。星形の抜き型は、生地がプレーンでもココアでも抹茶でも同じ形のクッキーを作れます。生地（型）ごとに抜き型を作り直す必要はありません。

実際の技術との違いを一言添えると、Rustのジェネリクスはコンパイル時に「生地ごとの専用抜き型」を**実際に複製して作ります**。使う側からは1つに見えますが、機械語のレベルではu16版・f32版が別々に存在します。これが速さの理由であり、サイズが増える理由でもあります。

## 仕組み

前ページの `fn alert<L: StatusLight>` で、実はもうジェネリクスを使っていました。一般形はこうです。

```text
fn 関数名<T: 条件>(引数: T) -> T
        ^^^^^^^^^ 型引数の宣言とトレイト境界
```

- `<T>` — 「Tという名前の型の穴」を宣言する
- `T: PartialOrd` — 穴に入れてよい型の条件（比較できること）
- 条件がないと、コンパイラは `T` に対して**何もできません**。足し算も比較も表示も、すべてtraitの能力だからです

## RustとEmbassyではどう書くか

Playgroundで動く完全なコードです。ジェネリック関数、ジェネリックstruct、そして `dyn Trait` を並べます。

```rust
fn max_of<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

struct Reading<T> {
    value: T,
    tick: u32,
}

impl<T: core::fmt::Debug> Reading<T> {
    fn show(&self) {
        println!("tick {}: {:?}", self.tick, self.value);
    }
}

// dyn Trait の比較用
trait Speaker {
    fn speak(&self);
}

struct Dog;
struct Cat;

impl Speaker for Dog {
    fn speak(&self) {
        println!("wan");
    }
}
impl Speaker for Cat {
    fn speak(&self) {
        println!("nya");
    }
}

fn main() {
    println!("{}", max_of(3, 7));
    println!("{}", max_of(1.5, 0.5));

    let r = Reading { value: 3300u16, tick: 42 };
    r.show();
    let f = Reading { value: 3.3f32, tick: 43 };
    f.show();

    // dyn Trait: 実行時に切り替える
    let animals: [&dyn Speaker; 2] = [&Dog, &Cat];
    for a in animals {
        a.speak();
    }
}
```

## コードを一行ずつ読む

- `max_of(3, 7)` と `max_of(1.5, 0.5)` — 同じ関数名ですが、コンパイラは整数版と小数版の**2つの専用関数**を生成しています。これを**単相化（monomorphization）**と呼びます。呼び出しは普通の関数と同じ速さです。
- `struct Reading<T>` — structにも型引数を付けられます。ADCの生値（u16）にも変換後の電圧（f32）にも、同じ「計測値+時刻」の形を使い回せます。
- `impl<T: core::fmt::Debug> Reading<T>` — 「Tが表示できる型のときだけshowメソッドが生える」という書き方です。境界はimpl単位でも付けられます。
- `[&dyn Speaker; 2]` — DogとCatは**別の型**なので、普通は同じ配列に入れられません。`&dyn Speaker` は「Speakerの約束を果たす何かへの参照」という1つの型なので、混ぜられます。その代わり、どのspeakを呼ぶかは実行時に表引き（vtable）で決まり、わずかなコストが掛かります。

## ジェネリクスとdyn Traitの使い分け

| | ジェネリクス `<T: Trait>` | `dyn Trait` |
|---|---|---|
| 型が決まる時 | コンパイル時 | 実行時 |
| 速度 | 直接呼び出し（速い） | 表引き1回分のコスト |
| コードサイズ | 型の数だけ複製され増える | 1つだけ |
| 違う型を混ぜる | できない | できる |

組み込みでは基本は**ジェネリクス**です。esp-halもembedded-halもこの方式です。フラッシュ容量が厳しいときや、異なる型のドライバを1つのリストで管理したいときに `dyn` を検討します。

## 実行方法

Rust Playground に貼り付けて Run します。

```text
7
1.5
tick 42: 3300
tick 43: 3.3
wan
nya
```

## よくある失敗

**1. 境界なしで演算しようとする**

```rust
fn max_of<T>(a: T, b: T) -> T {
    if a > b { a } else { b } // エラー: binary operation `>` cannot be applied to type `T`
}
```

「Tは何でもよい」と宣言した以上、コンパイラは比較できる保証がないと判断します。`T: PartialOrd` という条件を付けて初めて `>` が使えます。制約は嫌がらせではなく、「この関数はどんなTでも絶対に壊れない」ことの証明に必要な条項です。

**2. 1回の呼び出しに違う型を混ぜる**

`max_of(3, 1.5)` はエラーです。`T` は1回の呼び出しの中では1つの型に決まるため、整数と小数を混ぜられません。`max_of(3.0, 1.5)` のように型をそろえます。

## やってみよう

`fn min_of<T: PartialOrd>(a: T, b: T) -> T` を自分で書いて、`max_of` と並べて呼んでみましょう。u16でもf32でも動くことを確認してください。

## 確認問題

1. トレイト境界のない `<T>` に対してできる操作はどれくらいありますか？
2. 単相化とは何ですか？ 利点と欠点を1つずつ挙げてください。
3. 型の違うドライバを1つの配列に入れたいとき、ジェネリクスとdynのどちらを使いますか？

<details>
<summary>答え</summary>

1. ほぼ何もできません。move、参照を取る、などだけです。演算・比較・表示はすべてtraitの能力です。
2. 使われた型ごとに専用のコードをコンパイル時に生成することです。利点は実行時コストゼロ、欠点はコードサイズの増加です。
3. `dyn Trait` です。ジェネリクスは1つの変数・配列に対して1つの型に固定されます。
</details>

## まとめ

- ジェネリクスは「型だけが違う同じ処理」を1回で書く仕組み。トレイト境界で能力を保証する
- 単相化により実行時コストはゼロ。ただしコードサイズは型の数だけ増える
- 実行時に型を切り替えたいときだけ `dyn Trait`。組み込みの基本はジェネリクス

## 次のページ

embedded-halの `Result<(), Self::Error>` に出てきた `Self::Error` の正体、**関連型**を学びます。これが読めるとembedded-halのドキュメントが急に読みやすくなります。

[7. associated typeの入門](/embassy-esp32-c6/part04/07-associated-type/)

---

前のページ: [5. trait — 共通の能力を定義する](/embassy-esp32-c6/part04/05-trait/)
