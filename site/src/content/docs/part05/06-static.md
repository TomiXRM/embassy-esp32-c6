---
title: "6. staticとstatic_cell"
description: プログラムの寿命と同じだけ生きるstatic変数、'staticライフタイムの直感、StaticCellによる実行時初期化を学びます。
part: 5
lesson: 6
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part03/10-lifetime
  - part05/05-heap
status: drafted
code_status: syntax-reviewed
verified_with: "esp-hal 1.1.1, static_cell 2.1"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/reference/items/static-items.html
  - https://docs.rs/static_cell/2.1
---

## このページでできるようになること

- static変数が「プログラム開始から終了までずっと存在する変数」であることを説明できる
- `'static` というライフタイムの意味を直感で説明できる
- 実行時にしか作れない値を `'static` にする道具、`StaticCell` の役割が分かる

## 先に結論

`static` で宣言した変数は、RAM上の決まった住所に置かれ、プログラムが動いている間ずっと存在します。この「ずっと存在する」性質をライフタイムで表したものが `'static` です。Embassyのtask同士で共有するデータ（チャネルなど）は、どのtaskよりも長生きする必要があるため、staticに置くのが定石です。ただし `static` の初期値はコンパイル時に計算できる式（const）に限られます。実行時にしか作れない値を `'static` にしたいときは `static_cell::StaticCell` を使います。

## 身近なたとえ

ローカル変数が教室の黒板だとすると、static変数は校舎の玄関にある掲示板です。黒板は授業（関数）が終わると消されますが、玄関の掲示板は学校が開いている間ずっとそこにあり、どの教室からも見に行けます。

ただし実際のstaticは「誰でも自由に書き込める掲示板」ではありません。複数のtaskから同時に書き換えられると危険なので、Rustは書き換え可能な共有に必ず安全装置（後述のChannelや排他制御）を要求します。

## 仕組み

### static変数とその置き場所

第5部3ページの地図で見たとおり、static変数はRAM上の固定された場所に置かれ、初期値は起動時にフラッシュからコピー（またはゼロ埋め）されます。スタックのように片付けられることはありません。

examples/07-channel には、実際にstaticを使っている行があります（抜粋。完全なコードは examples/07-channel を見てください）。

```rust
/// タスク間をつなぐチャネル（容量4のメッセージキュー）。
/// staticに置くことで、どのタスクからも参照できます。
static CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 4> = Channel::new();
```

- 名前が大文字（`CHANNEL`）なのはstaticの慣習です
- 初期値 `Channel::new()` は**const関数**（コンパイル時に計算できる関数）なので、staticの初期値に書けます
- 容量4はheaplessと同じ発想の固定長です。ヒープを使わずにtask間の郵便受けを作れます

### 'static — 「プログラムと同じ寿命」

第3部で、ライフタイムは「その参照がいつまで有効か」の名前だと学びました。`'static` はその最長のもので、「プログラムが動いている限りずっと有効」を意味します。

なぜこれが重要かというと、Embassyのtaskは一度動き出すと**いつまで動き続けるか分からない**からです。taskに渡すデータが途中で消えてしまうと危険なので、コンパイラは「taskに渡す参照は `'static` であること」を要求します。blinkyや07-channelでtaskの引数の型に `Input<'static>` のように `'static` が現れるのは、この要求の表れです。

### StaticCell — 実行時の値を'staticにする

staticの初期値はconstに限られます。ところが、たとえば `esp_hal::init` が返すペリフェラルから作った値は、実行時にしか手に入りません。「実行時に作った値を、'staticな置き場に一度だけ入れる」ための道具が `static_cell::StaticCell` です。

次のコードは書き方を示す例です。実際の利用は第10部のWi-Fiなど、大きな実行時リソースを共有する場面で登場します。

```rust
use static_cell::StaticCell;

// 置き場だけを先に用意する（中身はまだ空）
static RESOURCE: StaticCell<[u8; 1024]> = StaticCell::new();

// 実行時に一度だけ初期化し、&'static mutを受け取る
let buf: &'static mut [u8; 1024] = RESOURCE.init([0; 1024]);
```

- `init` は**一度しか呼べません**。2回呼ぶとpanicします。「'staticな場所の唯一の所有権を渡す」ためです
- 戻り値は `&'static mut`、つまり「プログラムの寿命の間ずっと有効で、書き換えもできる唯一の参照」です。これをtaskに渡せます

## よくある失敗

- **`static mut` を使おうとする** — 検索すると古い記事で見かけますが、`static mut` への読み書きはコンパイラが安全を確認できず、現在のRustでは実質使えない（unsafeが必須で、かつ誤りやすい）ものです。共有して書き換えたいなら `Channel`（第9部）や `StaticCell` を使ってください。
- **`StaticCell::init` を2回呼んでpanicする** — ループの中や、2回呼ばれる関数の中で `init` すると2回目でpanicします。`init` は `main` の初期化部分で1回だけ呼ぶ、と決めておくのが安全です。
- **ローカル変数への参照をtaskに渡そうとする** — 「`'static` が必要」というコンパイルエラーになります。エラーはいじわるではなく、「taskより先にその変数が消えるかもしれない」という本物の危険を教えています。データをstaticに移すのが正しい対処です。

## やってみよう

examples/07-channel の `main.rs` を開いて、`static` が使われている行と、taskの引数の型に `'static` が現れる場所をすべて探してください。「taskより長生きすべきものがstaticに置かれている」という目で読めるようになれば成功です。

## 確認問題

1. static変数とローカル変数の寿命の違いを一言で説明してください。
2. `Channel::new()` がstaticの初期値に書けるのはなぜですか。
3. `StaticCell` はどんなときに必要になりますか。

<details>
<summary>答え</summary>

1. ローカル変数は関数を抜けると消えますが、static変数はプログラムが動いている間ずっと存在します。
2. `Channel::new()` がconst関数で、初期値をコンパイル時に計算できるからです。
3. 初期値がコンパイル時に計算できない（実行時にしか作れない）値を、'staticな置き場に入れたいときです。ペリフェラル初期化後にしか作れないリソースをtaskと共有する場面が典型です。

</details>

## まとめ

- static変数はRAMの固定住所に置かれ、プログラムと同じ寿命を持つ。`'static` はその寿命の名前
- taskに渡すデータは `'static` が要求される。task間共有のChannelはstaticに置くのが定石
- 実行時にしか作れない値は `StaticCell` で一度だけ初期化して `&'static mut` を得る

## 次のページ

`StaticCell::init` を2回呼ぶとpanicする、と書きました。そもそもpanicとは何で、起きたらマイコンはどうなるのでしょうか。次のページでpanicとの付き合い方を学びます。

[← 前のページ: heapを使わない設計](/embassy-esp32-c6/part05/05-heap/) | [次のページ: panicとの付き合い方 →](/embassy-esp32-c6/part05/07-panic/)
