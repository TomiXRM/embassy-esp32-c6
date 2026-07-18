---
title: "5. Result — 失敗を型で表す"
description: 失敗する可能性のある処理をResult型で表し、?演算子でエラーを呼び出し元へ伝える方法を学びます。
part: 3
lesson: 5
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part03/04-option
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 (edition 2024)"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
  - https://doc.rust-jp.rs/book-ja/ch09-02-recoverable-errors-with-result.html
---

## このページでできるようになること

- 「失敗するかもしれない処理」をResult型で表せる
- 自作のエラーenumを定義し、失敗の理由を型で区別できる
- ?演算子でエラーを呼び出し元へ渡す流れを書ける

## 先に結論

Optionは「値がない」ことしか伝えられません。しかし現実の処理は「センサがつながっていないから失敗」「値が範囲外だから失敗」のように、**理由の違う失敗**をします。`Result<T, E>`は「成功なら`Ok(値)`、失敗なら`Err(理由)`」を表すenumで、理由を運べるのがOptionとの違いです。組み込みでは通信も初期化も失敗しうるので、esp-halをはじめ実際のAPIの多くがResultを返します。失敗を呼び出し元へ渡す`?`演算子まで身につけると、エラー処理のコードが一気に読みやすくなります。

## 身近なたとえ

Resultは、宅配便の「配達結果の通知」のようなものです。無事に届けば「お届け完了（品物つき）」、だめなら「不在のため持ち帰り」「住所不明」のように**理由付きの不達通知**が返ります。理由が分かるから、再配達を頼むのか住所を直すのか、次の手を選べます。

たとえと違うのは、通知を無視できないことです。Resultは「使わずに捨てると警告される型」（must_use）に指定されていて、失敗の確認を忘れるとコンパイラが指摘します。

## 仕組み

`Result<T, E>`も標準ライブラリにある、ただのenumです。

```rust
enum Result<T, E> {
    Ok(T),  // 成功（結果の値つき）
    Err(E), // 失敗（理由つき）
}
```

- `T`は成功時の値の型、`E`は失敗の理由の型です
- 理由の型`E`には、自分で定義したenumを使うのが定番です

失敗の理由は、このように自作enumで表します。

```rust
#[derive(Debug)]
enum SensorError {
    NotConnected, // センサがつながっていない
    OutOfRange,   // 値が測定範囲の外
}
```

`#[derive(Debug)]`は「`{:?}`で表示できるようにする」という自動実装の指示です（詳しくは第4部のtraitで学びます）。エラーをログに出すために付けておくと便利です。

## Rustではどう書くか

温度センサの読み取りを2段階（生の値を読む→温度に変換する）で行い、どちらの失敗も呼び出し元へ伝える例です。センサの動作は作り物ですが、第8部で本物のI2Cセンサに置き換わっても構造は同じです。Rust Playgroundでそのまま動きます。

```rust
// 温度センサ読み取りの失敗を表す型
#[derive(Debug)]
enum SensorError {
    NotConnected, // センサがつながっていない
    OutOfRange,   // 値が測定範囲の外
}

// センサから生の値を読む（ここでは動作を作り物にしている）
fn read_raw(connected: bool) -> Result<u16, SensorError> {
    if connected {
        Ok(2350) // 生の値
    } else {
        Err(SensorError::NotConnected)
    }
}

// 生の値を温度（℃）に変換する
fn to_celsius(raw: u16) -> Result<f32, SensorError> {
    if raw > 4000 {
        return Err(SensorError::OutOfRange);
    }
    Ok(raw as f32 / 100.0)
}

// ?演算子: 失敗したらそのままErrを返して抜ける
fn read_temperature(connected: bool) -> Result<f32, SensorError> {
    let raw = read_raw(connected)?;
    let celsius = to_celsius(raw)?;
    Ok(celsius)
}

fn main() {
    match read_temperature(true) {
        Ok(t) => println!("温度: {} ℃", t),
        Err(e) => println!("読み取り失敗: {:?}", e),
    }

    match read_temperature(false) {
        Ok(t) => println!("温度: {} ℃", t),
        Err(e) => println!("読み取り失敗: {:?}", e),
    }
}
```

## コードを一行ずつ読む

- `fn read_raw(...) -> Result<u16, SensorError>` — 「成功ならu16、失敗ならSensorError」と型が宣言しています。呼ぶ側は失敗があり得ることをシグネチャだけで知れます
- `Ok(2350)` / `Err(SensorError::NotConnected)` — 成功と失敗はそれぞれのバリアントで包んで返します
- `let raw = read_raw(connected)?;` — ここが今日の主役、**?演算子**です。意味は「Okなら中身を取り出して続行、Errなら**この関数からそのErrを返して即座に抜ける**」。matchで書くと次と同じ意味です

```rust
let raw = match read_raw(connected) {
    Ok(v) => v,
    Err(e) => return Err(e),
};
```

  失敗処理を1文字に畳めるので、本来の処理の流れ（読む→変換する）がそのまま読めます
- `?`が使えるのは、**自分の関数もResult（またはOption）を返すとき**だけです。`read_temperature`の戻り値が`Result<f32, SensorError>`だから、`?`でErrをそのまま横流しできます
- `main`のmatch — どこかで最後には「失敗をどうするか」を決める場所が必要です。ここでは表示して終わりですが、実機では「リトライする」「LEDで知らせる」などが入ります（第12部で扱います）

## 実行方法

[Rust Playground](https://play.rust-lang.org/)にコードを貼り付けて「Run」を押します。

```text
温度: 23.5 ℃
読み取り失敗: NotConnected
```

1回目は`?`を2つ通過してOkが届き、2回目は最初の`?`でErrが横流しされて`to_celsius`は実行されていません。

## よくある失敗

### Okで包み忘れる（E0308）

```rust
fn to_celsius(raw: u16) -> Result<f32, SensorError> {
    if raw > 4000 {
        return Err(SensorError::OutOfRange);
    }
    raw as f32 / 100.0 // Okで包み忘れた
}
```

```text
error[E0308]: mismatched types
   |
 6 | fn to_celsius(raw: u16) -> Result<f32, SensorError> {
   |                            ------------------------ expected `Result<f32, SensorError>`
   |                                                     because of return type
10 |     raw as f32 / 100.0
   |     ^^^^^^^^^^^^^^^^^^ expected `Result<f32, SensorError>`, found `f32`
   |
help: try wrapping the expression in `Ok`
   |
10 |     Ok(raw as f32 / 100.0)
```

戻り値の型はResultなので、成功の値も`Ok(...)`で包む必要があります。`help`の提案通り`Ok(...)`を付ければ直ります。

### Resultを無視する（must_use警告）

```rust
read_raw(true); // 結果を受け取っていない
```

```text
warning: unused `Result` that must be used
   |
11 |     read_raw(true);
   |     ^^^^^^^^^^^^^^
   |
   = note: this `Result` may be an `Err` variant, which should be handled
```

エラーではなく警告ですが、「失敗したかもしれないのに確認していない」合図です。C言語では戻り値のエラーコードを無視するバグが定番でした。Rustではコンパイラが見張ってくれるので、警告が出たらmatchや`?`で処理を足してください。

### Resultを返さない関数で?を使う（E0277）

`main`のように何も返さない関数の中で`read_raw(true)?`と書くと、「`?`はResultを返す関数でしか使えない」という趣旨のエラー（E0277）になります。`?`は「失敗を呼び出し元へ渡す」道具なので、渡す先の型が合っている必要があります。matchで受けるか、関数の戻り値をResultにします。

## やってみよう

`SensorError`に`Timeout`（時間内に応答がなかった）を追加し、`read_raw`が`connected`とは別の条件で`Err(SensorError::Timeout)`を返すようにしてみましょう。`read_temperature`は**1文字も変えずに**新しいエラーも横流しできることを確認してください。これが`?`で流れを組んでおく利点です。

## 確認問題

1. OptionではなくResultを使うのはどんなときですか。
2. `?`演算子は何をしますか。matchで書くとどうなりますか。
3. エラー型を自作enumにする利点は何ですか。

<details>
<summary>答え</summary>

1. 「ない」だけでなく「なぜ失敗したか」の理由を伝えたいとき。
2. Okなら中身を取り出して続行し、Errならその値を自分の関数の戻り値としてreturnして抜ける。`match r { Ok(v) => v, Err(e) => return Err(e) }`と同じ。
3. 失敗の理由をバリアントで区別でき、呼び出し元がmatchで理由別の対処（リトライ、通知など）を書ける。網羅性チェックも効く。

</details>

## まとめ

- `Result<T, E>`は「成功Ok(T)か失敗Err(E)か」を表すenum。失敗の理由を運べる
- エラーの理由は自作enumで表すのが定番。`#[derive(Debug)]`を付けてログに出せるようにする
- `?`は「失敗したら呼び出し元へ渡す」演算子。エラー処理を畳んで、本来の流れを読みやすく保つ

## 次のページ

structとenumでデータを作り、Option/Resultで安全に受け渡せるようになりました。次は、データに「そのデータ専用の操作」を結びつけるメソッドを学びます。

- 前のページ: [4. Option — 「ないかもしれない」を型で表す](/embassy-esp32-c6/part03/04-option/)
- 次のページ: [6. メソッドを定義する](/embassy-esp32-c6/part03/06-methods/)
