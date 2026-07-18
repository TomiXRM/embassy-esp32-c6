---
title: "7. associated typeの入門"
description: traitの中で型を約束する「関連型」を学び、embedded-halのError関連型が読めるようになります。
part: 4
lesson: 7
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part04/06-generics
status: complete
code_status: cargo-check-passed
verified_with: "Rust 1.97.1 / embedded-hal 1.0.0（ホストPCでcargo check/run済み）"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch20-02-advanced-traits.html
  - https://docs.rs/embedded-hal/1.0.0/embedded_hal/digital/trait.ErrorType.html
  - https://doc.rust-lang.org/core/convert/enum.Infallible.html
---

## このページでできるようになること

- 関連型（associated type）を「traitが約束する型の欄」として説明できる
- `Self::Error` という表記が読める
- embedded-halの `ErrorType` と `Infallible` の意味が分かる

## 先に結論

trait はメソッドだけでなく**型そのものを約束の項目にできます**。これが **関連型（associated type）** で、traitの中に `type Output;` のように書きます。実装する側が「私の場合、Outputはf32です」と1つ決めます。embedded-halでは失敗の型を `type Error` として実装ごとに決めさせており、メソッドの戻り値に現れる `Result<(), Self::Error>` は「成否を返す。失敗の詳しい型は実装ごとに違う」と読みます。絶対に失敗しない実装は `Error = Infallible`（不可能という意味の型）を選びます。

## 身近なたとえ

願書の記入欄を考えてください。願書（trait）には「連絡先: ＿＿＿」という**空欄**があります。空欄の埋め方は人それぞれで、電話番号の人もメールアドレスの人もいます。ただし1人につき1つ、必ず埋めます。

実際の技術との違いを一言添えると、関連型の空欄に入るのは値ではなく**型**です。そして空欄を埋めるのは実行時ではなく**コンパイル時**で、埋め忘れや矛盾はエラーとして検出されます。

## 仕組み

関連型付きtraitの形はこうです。

```rust
trait Sensor {
    type Output; // 関連型: 実装ごとに1つ決める

    fn read(&mut self) -> Self::Output;
}
```

- `type Output;` — 「Outputという名前の型の欄がある」という宣言
- `Self::Output` — 「自分（実装した型）が欄に書き込んだ型」という意味

前ページのジェネリクス `<T>` と似ていますが、役割が違います。ジェネリクスは**使う側**が型を選びます（`max_of` は呼ぶたびにu16でもf32でもよい）。関連型は**実装する側**が1回だけ決めます（温度センサの出力は常にf32、と実装時に確定する）。「1つの型につき答えは1つ」の関係なら関連型が適しています。

## RustとEmbassyではどう書くか

Playgroundは使えませんが、embedded-halを依存に加えたプロジェクト（4ページ参照）で動く完全なコードです。

```rust
trait Sensor {
    type Output; // 関連型: 実装ごとに1つ決める

    fn read(&mut self) -> Self::Output;
}

struct TempSensor;
struct TiltSensor;

impl Sensor for TempSensor {
    type Output = f32; // 温度は小数
    fn read(&mut self) -> f32 {
        25.5
    }
}

impl Sensor for TiltSensor {
    type Output = bool; // 傾きはオン/オフ
    fn read(&mut self) -> bool {
        false
    }
}

fn log_reading<S: Sensor>(sensor: &mut S) -> S::Output {
    sensor.read()
}

fn main() {
    let mut t = TempSensor;
    let mut s = TiltSensor;
    println!("{}", log_reading(&mut t));
    println!("{}", log_reading(&mut s));
}
```

## コードを一行ずつ読む

- `type Output = f32;` — TempSensorが欄を埋めた瞬間です。以後、TempSensorの `Self::Output` はf32を意味します。
- `fn read(&mut self) -> f32` — traitでは `Self::Output` だった戻り値を、実装では確定した型で書けます。
- `fn log_reading<S: Sensor>(...) -> S::Output` — 「Sが欄に書いた型をそのまま返す」関数です。TempSensorに使えばf32が、TiltSensorに使えばboolが返ります。

## 本物の例 — embedded-halのError関連型

embedded-hal 1.0のピン系traitは、こう定義されています（これは抜粋です。完全な定義は[公式ドキュメント](https://docs.rs/embedded-hal/1.0.0/embedded_hal/digital/trait.ErrorType.html)を見てください）。

```rust
pub trait ErrorType {
    type Error: Error; // 失敗の型の欄。Errorという能力を持つ型に限る
}

pub trait OutputPin: ErrorType {
    fn set_low(&mut self) -> Result<(), Self::Error>;
    fn set_high(&mut self) -> Result<(), Self::Error>;
}
```

読み方を分解します。

- `type Error: Error;` — 欄に入れられる型に**条件付き**の宣言です。「Errorという名前の欄には、（embedded-halの）Error traitを実装した型しか書けない」という意味です。
- `OutputPin: ErrorType` — 「OutputPinを実装するなら、先にErrorTypeも実装していること」という前提条件です。
- `Result<(), Self::Error>` — 「成功なら何も返さない。失敗ならあなたが欄に書いた型で理由を返す」です。

なぜこうなっているのでしょうか。GPIOピンの失敗理由はチップやピンの種類で異なります。I2C経由のIOエキスパンダのピンなら通信エラーが起こりえますが、マイコン直結のGPIOはまず失敗しません。失敗の型を1つに固定せず「欄」にしておくことで、両方を同じtraitで扱えます。

失敗しない実装のために、Rustには `core::convert::Infallible`（インファリブル、「失敗は不可能」を表す型）があります。esp-halのGPIOはこれを使っています。試しに自分でも「絶対に失敗しない偽物ピン」を書いてみます。

```rust
use core::convert::Infallible;
use embedded_hal::digital::{ErrorType, OutputPin};

struct FakePin;

impl ErrorType for FakePin {
    type Error = Infallible;
}

impl OutputPin for FakePin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
```

`Error = Infallible` は「この欄に書く型はあるが、その型の値は存在しえない」という宣言です。つまり `Err(...)` を作ること自体が不可能で、戻り値は事実上 `Ok(())` だけになります。

## 実行方法

```bash
cargo add embedded-hal@1.0.0
cargo run
```

```text
25.5
false
```

## よくある失敗

**1. 関連型の指定を忘れる**

`impl Sensor for TempSensor` の中に `type Output = f32;` を書かないと、`not all trait items implemented, missing: 'Output'` というエラーになります。メソッドと同じで、型の欄も埋め忘れは許されません。

**2. ジェネリクスと関連型を混同する**

「なぜ `trait Sensor<T>` にしないの？」という疑問はもっともです。`trait Sensor<T>` だと、同じ型に `Sensor<f32>` と `Sensor<bool>` の**両方を実装できてしまいます**。温度センサの出力は1種類のはずなので、「実装1つにつき型1つ」を強制できる関連型が適切です。逆に `max_of` のように呼ぶたびに型を変えたい場合はジェネリクスです。

## やってみよう

`TiltSensor` の `Output` を `bool` から `u8`（傾きの角度）に変えてみましょう。`read` の戻り値も直す必要があることをコンパイラが教えてくれます。欄とメソッドの型が常に連動していることを体感してください。

## 確認問題

1. 関連型とジェネリクス型引数の一番の違いは何ですか？
2. `Result<(), Self::Error>` を日本語で読み下してください。
3. `type Error = Infallible;` は何を宣言していますか？

<details>
<summary>答え</summary>

1. 関連型は実装する側が実装ごとに1つ決める。ジェネリクスは使う側が使うたびに選べる。
2. 「成功なら値なし、失敗ならこの実装が決めたError型で理由が返る」です。
3. 「この実装は絶対に失敗しない」という宣言です。Infallible型の値は作れないため、Errは決して返せません。
</details>

## まとめ

- 関連型はtraitが持つ「型の欄」。実装ごとに1つ埋める
- `Self::Error` は「実装が欄に書いた失敗の型」。embedded-halはこの方式で多様なハードウェアを1つのtraitで扱う
- 失敗しえない実装は `Infallible` を使う

## 次のページ

失敗の型を自分で設計する番です。自作エラーenum、Fromによる変換、`?` の連鎖を学びます。

[8. エラー設計](/embassy-esp32-c6/part04/08-error-design/)

---

前のページ: [6. generics](/embassy-esp32-c6/part04/06-generics/)
