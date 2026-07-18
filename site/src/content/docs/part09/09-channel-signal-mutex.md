---
title: "9. Channel・Signal・Mutex"
description: task間の連携道具を使い分けます。Channelは順番待ちの列、Signalは最新値だけ、Mutexは共有データの鍵です。
part: 9
lesson: 9
difficulty: intermediate
estimated_minutes: 18
prerequisites:
  - part09/05-spawner
  - part03/02-enum
  - part05/06-static
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - LED
  - 抵抗330Ω
  - ブレッドボード・ジャンパワイヤ
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal ~1.1.0 / esp-rtos 0.3.0 / embassy-executor 0.10.0 / embassy-time 0.5 / embassy-sync 0.7"
last_verified: "2026-07-18"
sources:
  - https://docs.rs/embassy-sync/0.7.2
  - https://embassy.dev/book/
---

## このページでできるようになること

- Channel・Signal・Mutexを目的で使い分けて、task間の連携を設計できる
- `CriticalSectionRawMutex`などのRawMutex型パラメータの意味を説明できる
- 「Mutexを使えば安全な設計になる」わけではない理由を説明できる

## 先に結論

taskへ渡した所有権は返ってこないので、task同士の連携には専用の道具を使います。**Channel**は「イベントを1件も取りこぼしたくない」ときの有界メッセージキュー（満杯なら送信側が待つ＝バックプレッシャ）。**Signal**は「最新値だけ分かればよい」ときの上書き掲示板。**Mutex**は「共有データを読み書きしたい」ときの鍵で、`lock().await`で順番を待ちます。3つとも型パラメータに**RawMutex**（保護の方式）を取り、迷ったら`CriticalSectionRawMutex`を選べば割り込みを含むどの実行文脈からも安全です。

## 身近なたとえ

- **Channel** — 回転寿司の注文レーン。注文（メッセージ）は順番に流れ、全部が板前に届きます。レーンが満杯なら、空くまで注文できません。
- **Signal** — 駅の発車案内の電光掲示板。新しい情報が来たら前の表示は消えます。「3本前の発車時刻」は誰も必要としません。
- **Mutex** — 1つしか鍵のないトイレ。入るときに鍵を取り、出るときに返す。鍵を持っている間は誰も入れません。

実際の技術との違い: どのたとえでも「人が判断して」譲り合いますが、実際は**待ちはすべて`.await`**で表現され、順番が来るまでtaskは眠っています。また鍵（MutexGuard）は「返し忘れ」が起きません。スコープを抜けると自動で返されます。

## 仕組み

3つの道具の使い分けを先に一覧で示します。

| | Channel | Signal | Mutex |
|---|---|---|---|
| 形 | 有界キュー（先入れ先出し） | 最新値1つ（上書き） | 共有データ＋鍵 |
| 向く用途 | ボタンイベント、コマンド列 | 最新のセンサ値、状態通知 | 設定値や集計値の共有 |
| 取りこぼし | しない（満杯なら送信側が待つ） | 古い値は消える（仕様） | ―（メッセージではない） |
| 主なAPI | `send().await` / `receive().await` / `try_send` | `signal(値)` / `wait().await` | `lock().await` → ガード |

### RawMutexという型パラメータ

`Channel<CriticalSectionRawMutex, ButtonEvent, 4>`の1つ目の型パラメータが**RawMutex**で、「中身をどの方式で保護するか」の指定です。共有する相手がどんな実行文脈か（同じexecutorのtaskだけか、割り込みも触るか）で選びます。

| 型 | 保護の方式 | 使いどころ |
|---|---|---|
| `CriticalSectionRawMutex` | クリティカルセクション（短時間、割り込みを止める） | 割り込みハンドラを含むどこからでも共有できる。迷ったらこれ |
| `NoopRawMutex` | 何もしない | 同一executorのtask同士だけで共有するとき（協調的なので割り込まれない） |

この教材では一貫して`CriticalSectionRawMutex`を使います。まず安全側に倒し、最適化は必要になってから考えます。

## RustとEmbassyではどう書くか

### Channel — ボタン係とLED係をつなぐ

examples/07-channel は、ボタン検出とLED操作を別taskに分け、Channelでつないでいます。

```rust
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

/// タスク間をつなぐチャネル（容量4のメッセージキュー）。
/// staticに置くことで、どのタスクからも参照できます。
static CHANNEL: Channel<CriticalSectionRawMutex, ButtonEvent, 4> = Channel::new();
```

```rust
/// ボタンタスク: BOOTボタンの押下を検出してイベントを送信する
#[embassy_executor::task]
async fn button_task(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        Timer::after(Duration::from_millis(30)).await; // チャタリング対策
        if button.is_low() {
            // キューが満杯のときは空きが出るまでここで待つ（バックプレッシャ）
            CHANNEL.send(ButtonEvent::Pressed).await;
            button.wait_for_rising_edge().await;
            Timer::after(Duration::from_millis(30)).await;
        }
    }
}
```

受信側（`led_task`）は`CHANNEL.receive().await`でイベントを1件ずつ取り出します。これは抜粋です。完全なコードは examples/07-channel を見てください。

### Signal — 最新値だけを届ける

```rust
use embassy_sync::signal::Signal;

/// 最新の温度だけを保持する掲示板
static LATEST_TEMP: Signal<CriticalSectionRawMutex, i32> = Signal::new();

#[embassy_executor::task]
async fn display_task() {
    loop {
        // 値が来るまで眠って待つ。来たら最新値だけ受け取る
        let temp = LATEST_TEMP.wait().await;
        info!("最新の温度: {}", temp);
    }
}

    // 送る側（読み取りtaskなど）
    LATEST_TEMP.signal(25);
    LATEST_TEMP.signal(26); // 25は上書きされ、26だけが残る
```

### Mutex — 共有データに鍵をかける

```rust
use embassy_sync::mutex::Mutex;

/// 複数taskが読み書きする合計カウンタ
static TOTAL_COUNT: Mutex<CriticalSectionRawMutex, u32> = Mutex::new(0);

        // ロックが取れるまで待ち、取れたら中の値を書き換える
        let mut count = TOTAL_COUNT.lock().await;
        *count += 1;
        // countがスコープを抜けるとロックは自動的に返される
```

## コードを一行ずつ読む

- `static CHANNEL: Channel<...> = Channel::new();` — staticに置くのは、どのtaskからも参照できる`'static`な置き場所が必要だからです（[staticのページ](/embassy-esp32-c6/part05/06-static/)参照）。`Channel::new()`はconstなので初期化子に直接書けます。
- `Channel<CriticalSectionRawMutex, ButtonEvent, 4>` — 「保護方式・運ぶ型・容量4」。容量が**有限（有界）**であることが大事で、満杯時に`send`が待つことでメモリあふれを防ぎます。
- `CHANNEL.send(ButtonEvent::Pressed).await` — 満杯なら空きが出るまでこのtaskが眠ります。この「送る側が待たされる」性質をバックプレッシャと呼び、次のページの主役です。
- `LATEST_TEMP.signal(26)` — `send`と違って待ちません。前の値は容赦なく上書きされます。「全部届ける」のではなく「最新を知らせる」道具だからです。
- `TOTAL_COUNT.lock().await` — 鍵が空くまで譲って待ちます。返り値のガード（`count`）を通してだけ中身に触れられ、ガードが消えると自動で解錠されます。

## 配線

examples/07-channel の配線です。

| 部品 | 接続 |
|---|---|
| LED | GPIO10 → 抵抗330Ω → LEDアノード（＋） → LEDカソード（−） → GND |
| ボタン | 配線不要（ボード上のBOOTボタン＝GPIO9を使用） |

## 実行方法

```bash
cd examples/07-channel
cargo run --release
```

```text
INFO - ボタンタスクとLEDタスクを起動します
INFO - [LEDタスク] イベントなし（3秒間ボタンが押されていません）
INFO - [LEDタスク] イベント受信: 1回目 → LEDをトグル
INFO - [LEDタスク] イベント受信: 2回目 → LEDをトグル
```

BOOTボタンを押すたびにLEDがトグルします。ボタン係とLED係が、共有変数なしで連携できています。

## よくある失敗

1. **Mutexを使えば自動的に安全な設計になると考える** — Mutexが守るのは「同時アクセスでデータが壊れない」ことだけです。ロックを長く握ったまま`.await`すれば他のtaskを長時間待たせますし、「読む→計算→別のロックで書く」のように鍵の外に論理を置けば、順序の前後で意図しない結果になります。**鍵は短く握る・関連するデータは1つのMutexにまとめる**という設計はプログラマの仕事です。
2. **Channelを増やせば責務分割になると考える** — Channelはあくまで通信路です。「誰が何を決めるのか」が曖昧なままChannelを張り巡らせると、メッセージが行き交うだけの追いにくいコードになります。まずtaskの役割（ボタン係・LED係……）を決め、**役割の境界にだけ**Channelを置きます。
3. **最新値がほしいのにChannelを使う（またはその逆）** — センサの最新値をChannelで送ると、受信が遅れたとき古い値が列に溜まります。逆にボタンイベントをSignalで送ると、連打が上書きで消えます。「全部届ける＝Channel、最新だけ＝Signal」を最初に選び分けてください。

## やってみよう

`ButtonEvent`enumに`Released`（離された）を追加し、`button_task`の`wait_for_rising_edge().await`の後で送ってみましょう。`led_task`の`match`に腕を1本足すだけで、「押した・離した」の両方が届くようになります。enumのメッセージは、あとから種類を増やせるのが強みです。

## 確認問題

1. ボタンの押下イベントを届けるのにSignalが不向きな理由は何ですか。
2. `Channel<CriticalSectionRawMutex, u8, 4>`の3つのパラメータはそれぞれ何を表しますか。
3. 「Mutexを使っているのに設計として安全でない」例を1つ挙げてください。

<details>
<summary>答え</summary>

1. Signalは最新値だけを保持し、前の値を上書きするからです。受信が間に合わないと連打の一部が消えます。取りこぼしたくないイベントはChannelを使います。
2. 保護の方式（割り込みを含むどこからでも使えるロック）、運ぶデータの型、キューの容量（有界。満杯なら送信側が待つ）。
3. 例: ロックを握ったまま長い`.await`や重い処理をして他のtaskを待たせる。関連する2つの値を別々のMutexに入れ、読み書きの間に他のtaskが割り込んで矛盾した組み合わせになる、など。

</details>

## まとめ

- 全部届けるならChannel（有界・満杯なら送信側が待つ）、最新だけならSignal、共有データの読み書きならMutex（`lock().await`）
- RawMutexは保護方式の指定。迷ったら`CriticalSectionRawMutex`
- MutexもChannelも道具にすぎない。安全で追いやすい設計（鍵は短く、役割の境界に通信路）は自分で作る

## 次のページ

Channelが満杯になったら？ selectで負けたFutureの「途中まで」は？ 非同期設計の落とし穴——キャンセル安全性・詰まり・優先順位——を最後にまとめます。

[10. キャンセル・詰まり・優先順位](/embassy-esp32-c6/part09/10-cancel-backpressure/)

前のページ: [8. join — 全部待つ](/embassy-esp32-c6/part09/08-join/)
