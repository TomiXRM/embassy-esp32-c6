---
title: "5. Spawner"
description: Spawnerでtaskを起動します。起動失敗がResultで返る仕組みと、taskへの所有権の受け渡し（ムーブ）を学びます。
part: 9
lesson: 5
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part09/04-task
  - part03/05-result
  - part03/08-ownership
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - LED
  - 抵抗330Ω
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal ~1.1.0 / esp-rtos 0.3.0 / embassy-executor 0.10.0 / embassy-time 0.5"
last_verified: "2026-07-18"
sources:
  - https://embassy.dev/book/
  - https://docs.rs/embassy-executor/0.10.0
---

## このページでできるようになること

- Spawnerでtaskを起動できる
- 起動失敗（pool_size超過）がどこでResultとして返るかを説明し、扱える
- taskへ引数を渡すと所有権がムーブすることを説明できる

## 先に結論

taskの起動役が**Spawner**です。mainが`async fn main(spawner: Spawner)`の形でSpawnerを受け取り、`spawner.spawn(...)`でtaskを走らせます。embassy-executor 0.10では、**失敗はtask関数を呼んで「生成トークン」を作る時点でResultとして返ります**（`blink_task(led)`がResult）。典型的な失敗は`pool_size`の超過です。また、taskへ渡した引数は**所有権ごとムーブ**され、以後mainからは触れません。これが「同じピンを2箇所から操作する」バグをコンパイル時に防ぎます。

## 身近なたとえ

Spawnerは、リレー競技の**スターター（出走係）**です。走者（task）はスタートラインに登録されて初めて走り出します。定員の決まったレーン（`pool_size`）が埋まっているのに登録しようとすると、その場で「もう走れません」と断られます。

実際の技術との違い: リレーと違って走者は交代せず、**全員が同時に走り続けます**（正確には、順番を譲り合いながら）。そしてスターターに渡したバトン（引数）は返ってきません——これが所有権のムーブです。

## 仕組み

起動の流れは2段階です。

```mermaid
flowchart LR
    A["blink_task(led)を呼ぶ"] -->|"Ok(トークン)"| B["spawner.spawn(トークン)"]
    A -->|"Err（poolに空きなし）"| C[起動できない<br>ここで対処する]
    B --> D[taskが実行キューに入り<br>動き始める]
```

1. **トークン生成**: `blink_task(led)`のようにtask関数を呼ぶと、taskはまだ動かず、「起動チケット」にあたる生成トークンが**Resultで**返ります。poolに空きがなければここで`Err`です。
2. **起動**: `spawner.spawn(トークン)`に渡すと、taskが実行キューへ入ります。この段階は失敗しません。

失敗を確かめるかどうかは状況次第です。各taskを1回ずつspawnするだけなら失敗は設計上ありえないので、`unwrap()`で十分です（理由をコメントに書きましょう）。同じtaskを何個も起動する場合は`match`で`Err`を扱います。

## RustとEmbassyではどう書くか

examples/06-embassy-tasks のmainです。

```rust
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // ...（初期化は省略）...

    // GPIO10を出力に設定。最初は消灯（Low）
    let led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    // タスクを生成。ledはここでblink_taskにムーブされる。
    // blink_task(led)は「生成トークン」のResultを返し、タスクの空きが
    // ない場合はここでErrになる（各タスク1個ずつなのでunwrapで問題ない）
    spawner.spawn(blink_task(led).unwrap());
    spawner.spawn(counter_task().unwrap());
```

これは抜粋です。完全なコードは examples/06-embassy-tasks を見てください。

同じtaskを複数起動したいときは、`pool_size`を指定し、`Err`もきちんと扱えます。

```rust
/// pool_size = 2: 同じtaskを同時に2つまで起動できる
#[embassy_executor::task(pool_size = 2)]
async fn worker_task(id: u8) {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("[worker {}] 動作中", id);
    }
}

    // 3つ目のトークン生成はErrになる
    spawner.spawn(worker_task(1).unwrap());
    spawner.spawn(worker_task(2).unwrap());
    match worker_task(3) {
        Ok(token) => spawner.spawn(token),
        Err(_) => info!("taskの空きがありません（pool_size超過）"),
    }
```

## コードを一行ずつ読む

- `async fn main(spawner: Spawner)` — Spawnerはexecutorから手渡しされます。自分でnewする必要はありません。
- `spawner.spawn(blink_task(led).unwrap())` — 内側の`blink_task(led)`が「トークン生成（失敗しうる）」、外側の`spawn`が「起動（失敗しない）」です。失敗のタイミングが内側にあるのが0.10系の特徴です。
- `blink_task(led)`の`led` — ここで所有権が`main`からtaskへ**ムーブ**します。この行のあとに`main`で`led.toggle()`と書くと、「moveされた値を使っている」というコンパイルエラーになります。エラーはいじわるではなく、「LEDの担当者は1人だけ」という設計をコンパイラが守ってくれている証拠です。
- `#[embassy_executor::task(pool_size = 2)]` — このtaskの「席」を2つ確保します。席の数はコンパイル時に決まるため、使うRAMも事前に確定します。

## 実行方法

```bash
cd examples/06-embassy-tasks
cargo run --release
```

前のページと同じ出力になれば成功です。今回はmainの2行（spawn）に注目して読み直してみてください。

## よくある失敗

1. **同じtaskを2回spawnして panic する** — `pool_size`は既定1です。2回目の`blink_task(...)`のトークン生成が`Err`になり、`unwrap()`でpanicします。複数動かしたいなら`pool_size`を増やすか、`match`で失敗を扱ってください。
2. **ムーブ済みの変数をmainで使ってしまう** — `spawner.spawn(blink_task(led).unwrap());`のあとに`led`を触ると`borrow of moved value: led`エラーです。LEDを操作したいなら、その操作もtaskの中に書くか、[Channel](/embassy-esp32-c6/part09/09-channel-signal-mutex/)で「操作の依頼」を送る設計にします。
3. **借用（&mut）でtaskに渡そうとする** — taskは半永久に生きるので、mainのローカル変数への参照は渡せず、ライフタイムのエラーになります。原則は「所有権ごとムーブ」です。

## やってみよう

`spawner.spawn(counter_task().unwrap());`をコピーしてもう1行増やし、実行してみましょう。`unwrap`がpanicし、その旨のエラーがシリアルに出ます。次に`counter_task`の属性を`#[embassy_executor::task(pool_size = 2)]`に変えると、今度は2つ動きます（カウンタ表示が2倍のペースで混ざります）。

## 確認問題

1. embassy-executor 0.10で、task起動の失敗はどの時点でどんな形で返りますか。
2. `spawner.spawn(blink_task(led).unwrap())`のあと、mainから`led`を使えないのはなぜですか。
3. この例の`unwrap()`が許容できる理由を説明してください。

<details>
<summary>答え</summary>

1. task関数を呼んで生成トークンを作る時点で、Resultとして返ります（`spawn`自体は失敗しません）。典型的な原因は`pool_size`超過です。
2. `led`の所有権がtaskへムーブしたからです。同じピンを2箇所から操作する事故をコンパイル時に防いでいます。
3. 各taskを1回ずつしかspawnしておらず、`pool_size`（既定1）を超えることが設計上ありえないからです。こうした「ありえない」理由はコメントで残すのが作法です。

</details>

## まとめ

- Spawnerがtaskの起動役。失敗はトークン生成時（task関数の呼び出し時）にResultで返る
- 典型的な失敗はpool_size超過。1回ずつのspawnなら理由を書いた上でunwrapでよい
- taskへの引数は所有権ごとムーブ。担当の一本化をコンパイラが保証する

## 次のページ

taskの中で必ず使う「時間」のAPIを整理します。Timer・Instant・Duration・Ticker——どれをいつ使うかを判断できるようになります。

[6. EmbassyのTimerとInstant](/embassy-esp32-c6/part09/06-embassy-time/)

前のページ: [4. task — 仕事を分割する](/embassy-esp32-c6/part09/04-task/)
