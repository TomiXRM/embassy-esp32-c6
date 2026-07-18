---
title: "7. loopと無限ループ"
description: loop/break/continueの使い方と、組み込みで無限ループが主役である理由を学びます。
part: 2
lesson: 7
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part02/06-if
hardware:
  - ESP32-C6-DevKitC-1（Rust Playgroundで試す場合は不要）
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch03-05-control-flow.html
---

## このページでできるようになること

- `loop` で繰り返しを書き、`break` で抜け、`continue` で次の周回へ飛ばせる
- `break 値` でループから値を持ち帰れる
- 組み込みプログラムの `main` が無限ループである理由を説明できる

## 先に結論

`loop { ... }` は「わざと終わらない」繰り返しです。抜けたいときは `break`、その周だけ残りを飛ばして次へ進みたいときは `continue` を使います。組み込みプログラムに「終了」はありません。電源が入っている限り、LEDを点滅させ、ボタンを見張り続けます。だからblinkyの `main` の中心は `loop` でした。パソコンのプログラムでは無限ループはたいていバグですが、組み込みでは主役です。

## 身近なたとえ

`loop` は「駅の環状線」です。終点がなく、同じ駅を何度でも回り続けます。`break` は「電車を降りる」、`continue` は「この駅には停まらず次へ進む」に当たります。

ただし実際の `loop` は環状線と違って、1周ごとに変数の値が変わっていけます。「同じ道を回りながら、状態は毎周更新される」のがプログラムのループです。

## 仕組み

```rust
loop {
    // ここが何度でも実行される
    if 抜ける条件 {
        break;      // ループを終了して次の行へ
    }
    if 飛ばす条件 {
        continue;   // 残りを飛ばしてループの先頭へ
    }
    // 通常の処理
}
```

もうひとつ、Rustらしい機能が `break 値` です。`loop` も式なので、`break` に値を添えると、それがループ全体の値になります。

```rust
let answer = loop {
    // 何かを探して…
    break 見つけた値; // これがanswerに入る
};
```

「見つかるまで繰り返し、見つけたものを持ち帰る」がそのまま書けます。

blinkyを思い出してください。

```rust
loop {
    led.set_high(); // 点灯
    Timer::after(Duration::from_millis(500)).await;
    led.set_low(); // 消灯
    Timer::after(Duration::from_millis(500)).await;
}
```

`break` がないので永遠に回り続けます。これが意図した動作です。マイコンにはプログラムを終了したあとに戻る場所（OSのデスクトップ画面のようなもの）がないため、`main` は決して終わってはいけません。blinkyの `main` の型が `-> !`（「決して戻らない」を表す特別な型）だったのはこのためです。

## Arduinoではどう書くか

Arduinoでは無限ループはフレームワークに隠されていました。

```cpp
void loop() {
  // この関数が自動で何度も呼ばれる
}
```

`loop()` 関数を書くと、裏側でArduinoのランタイムが `while(1) { loop(); }` のように呼び続けてくれていたのです。Rust + Embassyでは隠されず、自分で `loop { }` を書きます。仕組みがむき出しになっただけで、やっていることは同じです。

## RustとEmbassyではどう書くか

`break` / `continue` / `break 値` の練習です。この例は必ず終わるループなので、blinkyの `loop {` の手前に貼れば、実行後にいつものLチカが始まります（これは抜粋です。完全なコードは examples/01-blinky を見てください）。

```rust
let mut count = 0;
loop {
    count += 1;
    if count % 2 == 0 {
        continue; // 偶数はとばす
    }
    log::info!("奇数: {}", count);
    if count >= 9 {
        break; // 9まで表示したら終わり
    }
}

let mut n = 0;
let answer = loop {
    n += 1;
    if n * n > 50 {
        break n; // ループの結果として n を返す
    }
};
log::info!("2乗が50を超える最小の数は {}", answer);
```

## コードを一行ずつ読む

- `if count % 2 == 0 { continue; }` — 偶数の周は `log::info!` まで進まず、すぐ次の周へ。`%` は割り算の余りです
- `if count >= 9 { break; }` — 抜ける条件を書き忘れると本当に無限ループになります（下の「よくある失敗」参照）
- `break n;` — このループの「答え」として `n` を持ち帰り、`answer` に入ります。`loop` が式だからできる書き方です

## 実行方法

動かし方は2通りです（詳しくは[1. 変数とlet](/embassy-esp32-c6/part02/01-variables/)）。

```text
INFO - 奇数: 1
INFO - 奇数: 3
INFO - 奇数: 5
INFO - 奇数: 7
INFO - 奇数: 9
INFO - 2乗が50を超える最小の数は 8
```

## よくある失敗

**失敗1: breakをループの外に書いた（E0268）**

```rust
let count = 10;
if count >= 9 {
    break; // ループの中ではない
}
```

```text
error[E0268]: `break` outside of a loop or labeled block
  |
4 |         break;
  |         ^^^^^ cannot `break` outside of a loop or labeled block
```

`break` は「今いるループから抜ける」命令なので、ループの中でしか意味を持ちません。エラーメッセージもそのまま「ループの外では `break` できない」と言っています。

**失敗2: 抜ける条件を書き忘れて意図しない無限ループにした**

これはコンパイルエラーに**ならない**失敗です。`break` のない `loop` は文法的に正しいので、コンパイラは止めてくれません。症状で気づきます。

- 開発ボード: 同じログが延々と流れ続け、貼った場所より下（Lチカなど）が始まらない
- Playground: 実行が終わらず、しばらくすると強制終了される

「ループの中で、抜ける条件に近づく変化（`count += 1` など）が毎周起きているか」を確認するのが直し方です。逆に言えば、blinkyの `loop` のように**意図して**終わらせないループでは、これが正しい姿です。

## やってみよう

`let mut total = 0;` と `let mut i = 0;` を用意し、`loop` の中で `i += 1; total += i;` を繰り返して、`total` が100を超えたら `break i` で抜けてみましょう。「何番まで足すと100を超えるか」が `answer` に入ります（答えは14です）。

## 確認問題

1. パソコンのプログラムと違い、組み込みの `main` が無限ループでなければならないのはなぜでしょうか?
2. `break` と `continue` の違いは何でしょうか?
3. `let x = loop { break 42; };` の `x` には何が入るでしょうか?

<details>
<summary>答え</summary>

1. マイコンには終了後に戻る場所（OS）がなく、電源が入っている限り仕事を続ける必要があるからです。
2. `break` はループ全体を終了する。`continue` はその周の残りを飛ばして次の周へ進む（ループ自体は続く）。
3. `42`。`loop` は式であり、`break` に添えた値がループ全体の値になります。

</details>

## まとめ

- `loop` は意図的な無限ループ。`break` で抜け、`continue` で次の周へ
- `break 値` でループから値を持ち帰れる（`loop` は式）
- 組み込みの `main` は終わってはいけない。blinkyの `loop` と `-> !` はそのための形

## 次のページ

「条件が成り立っている間だけ」繰り返す `while` を学びます。`loop` + `break` をひとまとめにした便利な形です。

[8. while →](/embassy-esp32-c6/part02/08-while/)

---

- 前のページ: [6. ifで分岐する](/embassy-esp32-c6/part02/06-if/)
- 次のページ: [8. while](/embassy-esp32-c6/part02/08-while/)
