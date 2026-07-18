---
title: "9. 借用 — 貸し借りの規則"
description: 所有権を移さずに値を使う借用と、&と&mutの規則、E0499・E0502の読み方を学びます。
part: 3
lesson: 9
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part03/08-ownership
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 (edition 2024)"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
  - https://doc.rust-jp.rs/book-ja/ch04-02-references-and-borrowing.html
---

## このページでできるようになること

- 所有権を移さずに値を使わせる**借用**（`&`と`&mut`）を使い分けられる
- 「読む借用は何個でも、書く借用は1個だけ」という規則と、その理由を説明できる
- E0499（二重可変借用）とE0502（読み書きの衝突）のエラーを読んで直せる

## 先に結論

前のページの3つ目の問い「誰が**一時的に**使うのか」に答えるのが借用です。`&値`と書くと、所有権を移さずに「読むための参照」を渡せます（**共有借用**）。`&mut 値`なら「書き換えるための参照」です（**可変借用**）。貸し借りには規則があります。**同時に存在できるのは、共有借用なら何個でも、可変借用なら1個だけ。両方の混在は不可。** つまり「みんなで読むか、1人で書くか」のどちらかです。この規則が、「読んでいる最中に誰かが書き換えて値が化ける」というバグを、コンパイル時に締め出します。

## 身近なたとえ

図書室の本と、その「書き込み権」で考えます。閲覧だけなら、同じ本を（コピーして）何人が同時に読んでも問題は起きません。しかし誰かが内容を書き換えている最中に他の人が読むと、書きかけの中途半端な内容を読んでしまいます。だから「編集したい人は1人で借り切り、その間は誰も読めない」が安全な運用です。

たとえと違うのは、この貸出管理を実行中に誰かが見張るのではなく、**コンパイラがコンパイル時に全部チェックする**ことです。この検査係をborrow checker（借用チェッカ）と呼びます。規則違反のプログラムは、そもそも実行ファイルになりません。

## 仕組み

| 書き方 | 名前 | できること | 同時に存在できる数 |
|---|---|---|---|
| `&値` | 共有借用 | 読む | 何個でも |
| `&mut 値` | 可変借用 | 読む・書く | 1個だけ（共有借用との同居も不可） |

大事な点を3つ補足します。

- 借用は「一時的」です。参照が最後に使われた場所で貸し出しは終わり、その後は次の借用ができます
- `&mut`で借りるには、そもそも持ち主が`let mut`である必要があります（変更不可の値の書き換え権は誰にも貸せません）
- 前のページのメソッドの`&self` / `&mut self`は、この共有借用と可変借用そのものです

なぜ「書く借用は1個だけ」なのでしょうか。組み込みに近い例で言えば、あるtaskがセンサ値の配列を平均計算のために読んでいる最中に、別の処理が同じ配列を書き換えたら、平均は「半分古く半分新しい」壊れた値になります。Rustはこの状況を型の規則として禁止しています（この規則は、第9部で複数taskを扱うときの安全性の土台にもなります）。

## Rustではどう書くか

ADC（第7部で学ぶ電圧測定）のサンプル配列を、読む関数と書き換える関数に貸す例です。Rust Playgroundでそのまま動きます。

```rust
// &[u16]: 読むだけ借りる（共有借用）
fn average(data: &[u16]) -> u16 {
    let mut sum: u32 = 0;
    for &v in data {
        sum += v as u32;
    }
    (sum / data.len() as u32) as u16
}

// &mut [u16]: 変更する前提で借りる（可変借用）
fn remove_offset(data: &mut [u16], offset: u16) {
    for v in data.iter_mut() {
        *v = v.saturating_sub(offset);
    }
}

fn main() {
    let mut samples: [u16; 4] = [512, 520, 508, 516];

    // 読むだけの借用は同時に何個あってもよい
    let avg1 = average(&samples);
    let avg2 = average(&samples);
    println!("平均: {} / {}", avg1, avg2);

    // 変更のための借用は、その間ほかの借用と同時に持てない
    remove_offset(&mut samples, 500);
    println!("補正後: {:?}", samples);
    println!("補正後の平均: {}", average(&samples));
}
```

## コードを一行ずつ読む

- `fn average(data: &[u16]) -> u16` — 引数は「u16スライスの共有借用」です。読むだけだと宣言しているので、呼ぶ側は安心して貸せます。所有権は移らないので、呼んだ後も`samples`は使えます（前のページのE0382はもう起きません）
- `average(&samples)` — 貸す側は`&`を付けます。「所有権ごと渡す」のか「読ませるだけ」なのかが、呼び出し行を見るだけで分かります
- `fn remove_offset(data: &mut [u16], ...)` — 「書き換えます」という宣言です。呼ぶ側も`&mut samples`と明示するので、値が変わる可能性のある行はコードから一目で分かります
- `*v = v.saturating_sub(offset);` — `*`は参照の指す先の値を意味します（参照そのものではなく中身を書き換える）。`saturating_sub`は0を下回らない引き算です
- `remove_offset(&mut samples, 500);` — この行の中だけ可変借用が存在します。行が終われば貸し出しも終わるので、次の行でまた`&samples`と読む借用ができます

## 実行方法

[Rust Playground](https://play.rust-lang.org/)にコードを貼り付けて「Run」を押します。

```text
平均: 514 / 514
補正後: [12, 20, 8, 16]
補正後の平均: 14
```

## よくある失敗

### 可変借用を2個作る（E0499）

```rust
let mut samples: [u16; 4] = [512, 520, 508, 516];

let first = &mut samples;
let second = &mut samples; // 2個目の可変借用
first[0] = 0;
second[1] = 0;
```

```text
error[E0499]: cannot borrow `samples` as mutable more than once at a time
  |
4 |     let first = &mut samples;
  |                 ------------ first mutable borrow occurs here
5 |     let second = &mut samples;
  |                  ^^^^^^^^^^^^ second mutable borrow occurs here
6 |
7 |     first[0] = 0;
  |     -------- first borrow later used here
```

3か所が指されています。「1個目の可変借用」「2個目の可変借用」、そして「1個目が**後で使われている**」場所です。ポイントは3つ目で、もし`first`を`second`より後で使っていなければ、1個目の貸し出しは終わった扱いになりエラーは出ません。「同時に」2個が生きていることが問題なのです。直すには、1個目の用事を済ませてから2個目を借ります。

### 読んでいる最中に書き換える（E0502）

```rust
let view = &samples;              // 読むための借用
remove_offset(&mut samples, 500); // 変更のための借用
println!("{:?}", view);           // 読む借用をまだ使っている
```

```text
error[E0502]: cannot borrow `samples` as mutable because it is also borrowed as immutable
   |
10 |     let view = &samples;
   |                -------- immutable borrow occurs here
11 |     remove_offset(&mut samples, 500);
   |                   ^^^^^^^^^^^^ mutable borrow occurs here
12 |     println!("{:?}", view);
   |                      ---- immutable borrow later used here
```

「immutable borrow（読む借用）が生きている間に、mutable borrow（書く借用）を作ろうとした」と読みます。もしこれが通ると、`view`の表示内容は「書き換え前とも後ともつかない値」になりかねません。直し方は2つあります。(1) `println!`を`remove_offset`の前に移動して、読む用事を先に終わらせる。(2) 書き換えた後で改めて`&samples`を借り直す。

## やってみよう

E0499の例で、`first[0] = 0;`を`let second = ...`の**前**に移動してみましょう。エラーが消えるはずです。「借用は最後に使った場所で終わる」ことを、コンパイラの挙動で確かめてください。

## 確認問題

1. 借用の規則を「読む」「書く」という言葉で説明してください。
2. 「書く借用は同時に1個だけ」という規則がないと、どんなバグが起きますか。
3. E0499のエラーで`first borrow later used here`が表示されるのはなぜ重要ですか。

<details>
<summary>答え</summary>

1. 読むだけの借用（`&`）は同時に何個でもよい。書くための借用（`&mut`）は同時に1個だけで、読む借用とも同居できない。「みんなで読むか、1人で書くか」。
2. 読んでいる最中に値が書き換わり、半分古く半分新しい壊れたデータを読んでしまうバグ（データ競合）。実行するまで気づきにくく、症状も不安定。
3. 借用は「最後に使われた場所」までしか生きていないから。1個目の借用がまだ使われる予定だからこそ「同時に2個」となりエラーになる、という理屈がその行で示されている。

</details>

## まとめ

- 借用は「所有権を移さず一時的に使わせる」仕組み。`&`は読む借用、`&mut`は書く借用
- 規則は「みんなで読むか、1人で書くか」。違反はE0499・E0502としてコンパイル時に見つかる
- エラーは「借りた場所」「衝突した場所」「後で使った場所」の3点を指す。3点を読めば直し方が見える

## 次のページ

借用には「貸し出し期間」がありました。この期間、つまり「参照はいつまで有効か」を扱う考え方がライフタイムです。第3部の仕上げとして、その直感をつかみます。

- 前のページ: [8. 所有権 — 誰がデータを持つのか](/embassy-esp32-c6/part03/08-ownership/)
- 次のページ: [10. ライフタイムの直感](/embassy-esp32-c6/part03/10-lifetime/)
