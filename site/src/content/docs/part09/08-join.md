---
title: "8. join — 全部待つ"
description: joinで複数の待ちを同時に進め、全部の完了を待ちます。selectとの使い分け、task分割との設計判断も学びます。
part: 9
lesson: 8
difficulty: intermediate
estimated_minutes: 15
prerequisites:
  - part09/07-select
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
status: complete
code_status: cargo-check-passed
verified_with: "embassy-futures 0.1 / embassy-time 0.5"
last_verified: "2026-07-18"
sources:
  - https://docs.rs/embassy-futures/0.1.2
  - https://embassy.dev/book/
---

## このページでできるようになること

- `join`で複数のFutureを同時に進め、全員の完了を待てる
- selectとjoinを目的で使い分けられる
- 「taskを分ける」か「1つのtask内でjoin/selectする」かの判断基準を説明できる

## 先に結論

`embassy_futures::join::join(a, b)`は、2つのFutureを**同時に進めて、両方が完成するまで**待ちます。結果は`(aの結果, bの結果)`のタプルで返ります。selectが「早い者勝ち（片方はキャンセル）」なのに対し、joinは「全員集合（誰もキャンセルされない）」です。さらに大きな設計判断として、並行処理は「taskを分ける」方法と「1つのtask内でjoin/selectを使う」方法があり、**起床の速さならtask分割、借用の共有しやすさとRAM節約ならjoin/select**という使い分けがあります。

## 身近なたとえ

カレー作りの「ご飯を炊く（40分）」と「カレーを煮込む（30分）」です。順番にやると70分ですが、同時に仕掛ければ40分で「両方そろって」完成します。joinは「両方そろったら次へ進む」の書き方です。

実際の技術との違い: 料理は2つのコンロが同時に加熱します（本当の並列）。joinは**1つのCPUが待ち時間を融通し合っている**だけで、どちらかがCPUを使う計算を始めれば、その間もう片方は進みません。「待ち」が主体の仕事でこそ効きます。

## 仕組み

selectとjoinを並べると役割がはっきりします。

| | `select(a, b)` | `join(a, b)` |
|---|---|---|
| 待つのは | 早い方1つ | 両方 |
| 戻り値 | `Either::First/Second` | `(aの結果, bの結果)` |
| 残った側 | dropされる（キャンセル） | キャンセルされない（全員完走） |
| 典型用途 | タイムアウト、複数イベントの一本化 | 複数の初期化・複数の待ちを全部そろえる |

もうひとつ、Embassyで並行処理を書く方法は2つあることを整理します。

1. **taskを分ける** — `spawner.spawn`でtaskを複数起動する（第4・5ページの方法）
2. **1つのtask内でjoin/selectを使う** — このページと前ページの方法

使い分けの目安は次のとおりです。

- **taskを分けると、起こしてほしいFutureだけが起こされます。** executorはtask単位でしか起こせないため、join/selectで束ねた場合は、どれか1つの完成でtaskが起きると**束ねた全Futureをまとめて確認**することになります。反応の速さ・無駄のなさではtask分割が有利です。
- **join/selectは、同じ変数への借用を共有しやすい**のが強みです。taskへは所有権をムーブするしかありませんが（第5ページ）、同じtask内のFuture同士なら1つの変数を貸し借りできます。taskごとのstatic領域も要らないのでRAMの節約にもなります。

迷ったら「長生きする独立した仕事はtask、その場かぎりの同時待ちはjoin/select」から始めるとよいでしょう。

## RustとEmbassyではどう書くか

2つの待ちを同時に仕掛けて、両方の完了を待つ最小例です。

```rust
use embassy_futures::join::join;

    let ((), ()) = join(
        Timer::after(Duration::from_millis(100)),
        Timer::after(Duration::from_millis(200)),
    )
    .await;
    info!("両方終わりました");
```

100msと200msのタイマーを同時に進めるので、合計は300msではなく**約200ms**（長い方）で終わります。

3つ以上を待ちたいときは`join3`/`join4`もあります。結果を使う場合は`let (res_a, res_b) = join(a, b).await;`のようにタプルで受け取ります。

## コードを一行ずつ読む

- `join(a, b)` — selectと同じく、**awaitしていないFuture**を2つ渡します。この時点ではまだ何も始まりません。
- `.await` — ここで2つのFutureが交互にpollされながら進みます。片方が先に完成しても捨てられず、もう片方の完成を待ちます。
- `let ((), ())` — `Timer::after`の結果は`()`（何もなし）なので、このタプルになります。結果のある Future（受信など）なら中身を取り出せます。

## 実行方法

joinそのものの専用exampleはありません。examples/06-embassy-tasks のmainのループに上のコードを貼り付けると動作を確認できます。

```bash
cd examples/06-embassy-tasks
cargo run --release
```

ハートビートの間隔が変わらないまま「両方終わりました」が出れば、joinが待ち時間を重ねて使えていることになります。

## よくある失敗

1. **「joinすれば合計時間が半分になる」と思う** — joinが縮めるのは**待ち時間の重なり**だけです。100ms待ち＋200ms待ちは200msになりますが、100msの計算＋200msの計算は300msのままです（CPUは1つ。計算は重ねられません）。
2. **順番にawaitして、joinのつもりになっている** — `a.await; b.await;`は「aが終わってからbを始める」逐次処理です。同時に進めたいなら必ず`join(a, b).await`の形にします。
3. **なんでもjoin/selectで1つのtaskに詰め込む** — 束ねたFutureのどれかが起きるたびに全員分の確認が走り、コードも読みにくくなります。独立して動き続ける仕事（LED係、ボタン係……）は素直にtaskへ分けましょう。

## やってみよう

上の例の2つのタイマーを`from_millis(100)`と`from_millis(200)`から好きな値に変え、`Instant::now()`と`elapsed()`（前々ページ参照）で実際の所要時間を測ってみましょう。「長い方の時間」にほぼ一致することが確かめられます。

## 確認問題

1. `join(a, b)`と`select(a, b)`の最大の違いは何ですか。「キャンセル」という言葉を使って答えてください。
2. 100ms待ちと200ms待ちをjoinすると約何msで完了しますか。100msの計算と200msの計算なら？
3. 「taskを分ける」方法が「join/selectで束ねる」方法より有利なのはどんな点ですか。逆はどうですか。

<details>
<summary>答え</summary>

1. selectは早い方だけを待ち、負けた側はdropされてキャンセルされます。joinは両方を待ち、誰もキャンセルされません。
2. 待ちなら約200ms（重ねられる）。計算なら約300ms（CPUは1つなので重ねられない）。
3. task分割は必要なtaskだけが起こされるので反応が速く無駄がない。join/selectは借用を共有しやすく、taskの置き場所が不要な分RAMを節約できる。

</details>

## まとめ

- joinは「全部待つ」。結果はタプル、誰もキャンセルされない
- 縮むのは待ち時間の重なりだけ。計算は重ならない
- 長生きの独立した仕事はtask分割、その場の同時待ちはjoin/select

## 次のページ

taskを分けたら、次はtask同士の連携です。データを安全に受け渡すChannel・Signal・Mutexという3つの道具を使い分けます。

[9. Channel・Signal・Mutex](/embassy-esp32-c6/part09/09-channel-signal-mutex/)

前のページ: [7. select — 早い者勝ち](/embassy-esp32-c6/part09/07-select/)
