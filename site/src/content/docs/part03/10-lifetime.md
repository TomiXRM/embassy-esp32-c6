---
title: "10. ライフタイムの直感"
description: 参照はいつまで有効なのかというライフタイムの考え方を、E0597とE0106のエラーから直感的に学びます。
part: 3
lesson: 10
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part03/09-borrow
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 (edition 2024)"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
  - https://doc.rust-jp.rs/book-ja/ch10-03-lifetime-syntax.html
---

## このページでできるようになること

- ライフタイムを「いつまで存在するか」という言葉で説明できる
- E0597（参照が長生きしすぎ）のエラーを読んで直せる
- 関数シグネチャの`'a`が何を約束しているか読める

## 先に結論

所有権のページで立てた問いのうち、まだ残っているのが「**いつまで存在するのか**」です。値は持ち主がスコープを抜けたら片付けられます。では、その値への参照が持ち主より長生きしたら？——参照の先にはもう何もありません。この「値が生きている期間」と「参照が使われる期間」の関係を、Rustは**ライフタイム**と呼んで検査します。規則はひとつだけです。**参照は、指している値より長生きしてはならない。** `'a`のような記号は新しい概念ではなく、この期間に付けた名前にすぎません。普段はコンパイラが自動で判断してくれるため書く必要はなく、複数の参照が絡んで判断できないときにだけ書き足します。

## 身近なたとえ

文化祭の教室の場所を書いた「案内板」を想像してください。案内板（参照）そのものは軽くて便利ですが、文化祭が終わって教室が片付けられた（値が解放された）後も案内板が残っていたら、それを信じた人は空っぽの教室に案内されてしまいます。「案内板は、案内先が存在する期間内でだけ使ってよい」——これがライフタイムの規則です。

たとえと違うのは、Rustでは「古い案内板をうっかり信じる」ことが**起こりえない**点です。案内先より長生きする案内板は、コンパイルの段階で作らせてもらえません。C/C++ではこの「宙ぶらりんの参照（dangling pointer）」が実行時の重大バグの定番でした。

## 仕組み

規則違反を実際に起こしてみるのが一番の近道です。

```rust
fn main() {
    let outside;
    {
        let inside = 5;
        outside = &inside; // 内側の値への参照を、外側の変数に入れる
    }                      // ← ここでinsideは片付けられる
    println!("{}", outside); // 片付けられた値を指す参照を使おうとした
}
```

```text
error[E0597]: `inside` does not live long enough
  |
4 |         let inside = 5;
  |             ------ binding `inside` declared here
5 |         outside = &inside;
  |                   ^^^^^^^ borrowed value does not live long enough
6 |     }
  |     - `inside` dropped here while still borrowed
7 |     println!("{}", outside);
  |                    ------- borrow later used here
```

エラーの読み方です。「`inside`の寿命が足りない」という要約のあと、4つの場所が示されます。値が生まれた場所、借りた場所、**借りられたまま片付けられた場所**（`dropped here while still borrowed`）、そして片付けの後で使った場所。時系列を並べ直すと「借りる→片付く→使う」となっていて、これが規則違反です。直すには、`inside`を外側のスコープで宣言して寿命を延ばすか、参照ではなく値そのものをコピーします。

大事なのは、E0597が新しい規則ではないことです。所有権（値はスコープを抜けたら片付く）と借用（借りている間の値は使われ得る）を組み合わせると、自然にこの検査になります。

## Rustではどう書くか

では、関数が参照を**返す**とどうなるでしょう。2つの文字列のうち長い方を返す関数で試します。

```rust
fn longer(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
```

```text
error[E0106]: missing lifetime specifier
  |
1 | fn longer(x: &str, y: &str) -> &str {
  |              ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
    signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
  |
1 | fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
```

コンパイラの`help`が核心を言っています。「戻り値は借用だが、`x`から借りたのか`y`から借りたのか、シグネチャに書かれていない」。どちらを返すかは実行時の長さ比較で決まるので、コンパイラには判断できません。そこで提案されている通り`'a`を書き足します。Rust Playgroundでそのまま動きます。

```rust
// 'aは「引数も戻り値も、少なくとも同じ期間は生きている」という印
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// スライスの中で最大の値「への参照」を返す
fn max_ref<'a>(data: &'a [u16]) -> &'a u16 {
    let mut max = &data[0];
    for v in data {
        if v > max {
            max = v;
        }
    }
    max
}

fn main() {
    let name1 = "sensor-a";
    let name2 = "button";
    println!("長いほう: {}", longer(name1, name2));

    let samples: [u16; 4] = [512, 700, 508, 516];
    let biggest = max_ref(&samples);
    println!("最大値: {}", biggest);
}
```

## コードを一行ずつ読む

- `fn longer<'a>(x: &'a str, y: &'a str) -> &'a str` — `'a`（「ライフタイムa」と読みます）は期間に付けた名前です。この行は「戻り値は`x`か`y`のどちらかから借りたものなので、**両方が生きている期間内でだけ使えます**」という約束を書いています。新しい動作を追加するのではなく、すでにある事実をコンパイラと共有しているだけです
- 約束があるので、コンパイラは呼び出し側もチェックできます。たとえば`name2`が先に片付くスコープで戻り値を長く使おうとすれば、E0597系のエラーになります
- `fn max_ref<'a>(data: &'a [u16]) -> &'a u16` — 「配列を借りて、その中の1要素への参照を返す」関数です。戻り値は`data`の中身を指すので、配列が生きている間だけ有効——それを`'a`が表しています
- 実は`max_ref`のように**引数の参照が1つだけ**なら、`'a`を書かなくてもコンパイラが自動で補ってくれます（省略規則）。`longer`は候補が2つあるため省略できませんでした。「普段は書かない。曖昧なときだけ書く」が実際の使われ方です

もうひとつ、これから頻繁に目にする特別なライフタイムがあります。`'static`——「プログラムの最初から最後まで生きている」期間です。`"sensor-a"`のような文字列リテラルはプログラム本体に埋め込まれているので`'static`です。組み込みでは「電源が入っている間ずっと存在するデータ」が主役級に重要で、第5部のstaticとstatic_cellのページで再登場します。

## 実行方法

[Rust Playground](https://play.rust-lang.org/)にコードを貼り付けて「Run」を押します。

```text
長いほう: sensor-a
最大値: 700
```

## よくある失敗

### 参照が値より長生きする（E0597）

上の「仕組み」で見た通りです。`does not live long enough`を見たら、「借りる→片付く→使う」の3か所をエラー表示から探してください。値の宣言を外側へ移すか、参照をやめて値を持つのが基本の直し方です。

### 関数から局所変数の参照を返す（E0106 / E0515）

```rust
fn make_ref() -> &u16 {
    let value = 512;
    &value // 関数が終わるとvalueは片付けられる
}
```

まずシグネチャの段階でE0106（誰から借りたのか書かれていない）になり、仮にライフタイムを付けても「cannot return reference to local variable」（E0515）になります。関数の中で作った値は関数の終わりで片付けられるので、その参照を返す方法は**存在しません**。これはコンパイラの意地悪ではなく、「片付いた値を指す案内板」を渡さないための規則です。参照ではなく値そのもの（`u16`）を返せば解決します。

## やってみよう

`max_ref`の戻り値を、配列より長生きさせてみましょう。

```rust
let biggest;
{
    let samples: [u16; 4] = [512, 700, 508, 516];
    biggest = max_ref(&samples);
} // ← ここでsamplesは片付けられる
println!("{}", biggest);
```

E0597が出るはずです。エラー表示の中から「借りた場所」「片付いた場所」「使った場所」の3か所を探してください。関数を1回はさんでも、コンパイラが「参照は値より長生きできない」規則を追跡し続けていることが分かります。

## 確認問題

1. ライフタイムの規則を一文で説明してください。
2. `fn longer<'a>(x: &'a str, y: &'a str) -> &'a str`の`'a`は何を約束していますか。
3. 関数内で作った値への参照を返せないのはなぜですか。

<details>
<summary>答え</summary>

1. 参照は、指している値が生きている期間を超えて使ってはならない。
2. 戻り値の参照は`x`と`y`の両方が生きている期間内でだけ有効、という約束。コンパイラはこの約束を使って呼び出し側の使い方も検査する。
3. 関数内の値は関数の終わりで片付けられるので、返した参照は必ず「片付いた値を指す参照」になるから。値そのものを返すのが正しい。

</details>

## まとめ

- ライフタイムは「参照はいつまで有効か」の検査。規則は「参照は値より長生きしない」のひとつだけ
- `'a`は期間に付けた名前で、シグネチャに事実を書き込む道具。普段は省略でき、曖昧なときだけ書く
- `'static`はプログラム全期間の寿命。組み込みでは第5部のstatic_cellで主役になる

## 次のページ

これで第3部は完了です。struct・enum・Option/Result・所有権と借用・ライフタイム——Rustらしいデータの扱いが一通り手に入りました。次の部では、育ってきたプログラムをmoduleで整理する方法から始めます。

- 前のページ: [9. 借用 — 貸し借りの規則](/embassy-esp32-c6/part03/09-borrow/)
- 次のページ: [1. moduleで整理する](/embassy-esp32-c6/part04/01-module/)
