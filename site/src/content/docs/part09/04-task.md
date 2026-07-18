---
title: "4. task — 仕事を分割する"
description: "#[embassy_executor::task]でtaskを書きます。taskはOSスレッドではなく、協調して順番を譲り合う仕事の単位です。"
part: 9
lesson: 4
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part09/02-async-await
  - part03/08-ownership
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - LED
  - 抵抗330Ω
  - ブレッドボード・ジャンパワイヤ
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal ~1.1.0 / esp-rtos 0.3.0 / embassy-executor 0.10.0 / embassy-time 0.5"
last_verified: "2026-07-18"
sources:
  - https://embassy.dev/book/
  - https://docs.rs/embassy-executor/0.10.0
---

## このページでできるようになること

- `#[embassy_executor::task]`でtaskを定義し、動かせる
- taskがOSスレッドとどう違うか（協調的実行）を説明できる
- 長いCPU処理が他のtaskを止めてしまう理由を説明できる

## 先に結論

**task**（タスク）は、Embassyにおける「並行して進めたい仕事1つ」の単位です。`async fn`に`#[embassy_executor::task]`を付けて定義します。taskはOSスレッドではありません。OSが強制的に切り替えてくれる仕組みはなく、**各taskが`.await`で自発的に順番を譲る**ことで成り立っています（協調的実行）。だから、`.await`に到達しない長いCPU処理やbusyループを書くと、**他のtaskが全部止まります**。

## 身近なたとえ

taskは、教室の掃除の**係分担**に似ています。黒板係、床係、ゴミ捨て係。それぞれ自分の仕事だけに集中すればよく、他の係の手順を知る必要はありません。

ただし、この教室には先生（OS）がいません。**雑巾を洗いに行く間（待ち時間）に自分から場所を空ける**からこそ、1人ずつしか動けない狭い教室でも全員の仕事が進みます。誰かが床のど真ん中に居座って動かない（busyループ）と、全員の作業が止まります。

実際の技術との違い: 係の人数分の生徒がいるわけではありません。**動き手（CPUコア）は1つだけ**で、executorが「今どの係が動くか」を切り替えています。

## 仕組み

OSスレッドとEmbassyのtaskを比べます。

| | OSスレッド | Embassyのtask |
|---|---|---|
| 切り替え | OSが強制的に中断できる（プリエンプティブ） | `.await`で自発的に譲る（協調的） |
| スタック | スレッドごとに大きなスタックが必要 | 状態は必要な分だけstatic領域に確保（heap不要） |
| 数の上限 | メモリが許す限り動的に作れる | コンパイル時に決まる（`pool_size`、既定1） |
| 切り替えコスト | 大きい | 小さい |

`#[embassy_executor::task]`には決まりがあります。

- `async fn`にだけ付けられる（普通のfnは不可）
- ジェネリクス（型引数）は使えない
- 同じtaskを同時に動かせる数は`pool_size`で決める（既定は1）。置き場所がコンパイル時にstatic領域へ確保されるためです

## RustとEmbassyではどう書くか

examples/06-embassy-tasks では、「LED点滅」と「カウンタ表示」を別々のtaskに分割しています。

```rust
/// タスクA: LEDを500ms間隔で点滅させる
#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        led.toggle();
        ticker.next().await;
    }
}

/// タスクB: 1秒ごとにカウンタを増やしてログに表示する
#[embassy_executor::task]
async fn counter_task() {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut count: u32 = 0;
    loop {
        ticker.next().await;
        count += 1;
        info!("[タスクB] カウンタ = {}", count);
    }
}
```

mainからの起動は次の2行です（詳しくは次のページ）。

```rust
    spawner.spawn(blink_task(led).unwrap());
    spawner.spawn(counter_task().unwrap());
```

これは抜粋です。完全なコードは examples/06-embassy-tasks を見てください。

## コードを一行ずつ読む

- `#[embassy_executor::task]` — この`async fn`を「起動できるtask」に変換する属性です。状態の置き場所がstatic領域に確保されるので、heapのないマイコンでも安心して使えます。
- `async fn blink_task(mut led: Output<'static>)` — taskは引数を受け取れます。`Output<'static>`の`'static`は「プログラムが動いている間ずっと有効なピン」という意味です。taskは半永久的に動くので、途中で消えるかもしれない借用は渡せません。
- `loop { ... ticker.next().await; }` — taskの典型形です。無限ループの中に**必ず`.await`がある**ことに注目してください。これが「順番を譲る」保証になります。
- `led.toggle()` — LEDの所有権はこのtaskが持っています。他のtaskからこのLEDは触れません（所有権の移動は次のページで詳しく見ます）。

## 配線

| 部品 | 接続 |
|---|---|
| GPIO10 | 抵抗330Ωの片側へ |
| 抵抗330Ωの反対側 | LEDのアノード（足の長い方、＋）へ |
| LEDのカソード（足の短い方、−） | GNDへ |

抵抗を必ず入れてください。抵抗なしでLEDを直結すると、LEDとGPIOピンに過大な電流が流れます。

## 実行方法

```bash
cd examples/06-embassy-tasks
cargo run --release
```

```text
INFO - 2つのタスクを起動します
INFO - [タスクB] カウンタ = 1
INFO - [タスクB] カウンタ = 2
INFO - [タスクB] カウンタ = 3
INFO - [タスクB] カウンタ = 4
INFO - [main] 動作中です（ハートビート）
```

LEDが500ms間隔で点滅しながら、カウンタが1秒ごとに進み、5秒ごとにmainのハートビートが出ます。

## よくある失敗

1. **`.await`のないループを書いてしまう** — 例えば`loop { if button.is_low() { ... } }`のようなbusyループです。このtaskが順番を譲らないため、**LED点滅もログも全部止まります**。コンパイルエラーにはならないのがやっかいな点です。「taskのループには必ず`.await`」と覚えてください。
2. **長いCPU処理で他のtaskを待たせる** — 重い計算（大きな配列の処理や暗号計算など）は、`.await`が無い間ずっと他のtaskを止めます。協調的実行の宿命です。どうしても必要なら、処理を小分けにして合間に譲る、優先度の仕組みを使うなどの対策があります（[第10ページ](/embassy-esp32-c6/part09/10-cancel-backpressure/)で触れます）。
3. **`#[embassy_executor::task]`をasyncでない関数に付ける** — 「task関数はasyncでなければならない」という趣旨のコンパイルエラーになります。taskは中断・再開が前提なので、asyncであることが必須です。

## やってみよう

3つ目のtaskとして、2秒ごとに`info!("[タスクC] こんにちは")`と表示する`hello_task`を追加してみましょう。`counter_task`をコピーして名前と周期を変え、mainに`spawner.spawn(hello_task().unwrap());`を足すだけです。taskの追加がどれほど簡単か体感できます。

## 確認問題

1. Embassyのtaskの切り替えが起こるのは、コードのどんな場所ですか。
2. taskの中に`.await`を含まない無限ループを書くと何が起きますか。またコンパイラはそれを教えてくれますか。
3. taskがheapなしで動けるのはなぜですか。

<details>
<summary>答え</summary>

1. `.await`の場所です。taskが自発的に順番を譲ったときだけ切り替わります（協調的実行）。
2. そのtaskがCPUを独占し、他のすべてのtaskが止まります。コンパイルエラーにはならず、実行して初めて分かります。
3. taskの状態の置き場所が、コンパイル時にstatic領域へ確保されるからです（`pool_size`で数が決まっているのもこのため）。

</details>

## まとめ

- taskは「並行して進めたい仕事1つ」の単位。`#[embassy_executor::task]` + `async fn`で書く
- OSスレッドではなく協調的実行。`.await`で自発的に譲ることで成立している
- `.await`に到達しない長いCPU処理・busyループは他のtaskを全部止める

## 次のページ

taskを定義しただけでは動きません。起動役のSpawnerと、起動失敗（pool_size超過）の扱い、task間の所有権の受け渡しを学びます。

[5. Spawner](/embassy-esp32-c6/part09/05-spawner/)

前のページ: [3. Futureの直感的説明](/embassy-esp32-c6/part09/03-future/)
