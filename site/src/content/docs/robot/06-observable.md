---
title: "6. Observableを読む — 131行の自作同期プリミティブ"
description: luhsoccer_firmwareの自作同期プリミティブObservableを実コードで読解し、embassy-sync 0.7のWatchと比較して「標準にない部品は自作してよい」判断基準を学びます。
difficulty: advanced
estimated_minutes: 30
prerequisites:
  - robot/05-maincontroller
  - part09/03-future
  - part09/09-channel-signal-mutex
status: complete
code_status: concept-only
last_verified: "2026-07-18"
sources:
  - https://github.com/luhbots/luhsoccer_firmware/blob/main/libs/sync/src/observable.rs
  - https://github.com/luhbots/luhsoccer_firmware/blob/main/libs/sync/Cargo.toml
  - https://docs.rs/embassy-sync/0.7.2/embassy_sync/watch/index.html
---

## このページでできるようになること

- 「最新値の保持＋複数購読者＋変化時だけ通知」という要求を、embassy-syncの部品でどう組み立てるか説明できる
- 131行の実コードを読み、Waker登録と世代番号（ID）による通知の仕組みを追える
- embassy-sync 0.7の`Watch`と比較し、「標準にあるものを使うか、自作するか」の判断基準を持てる

## 先に結論

前ページで11個のtaskをつないでいた`Observable`は、luhbotsのチームが自作した同期プリミティブです。ソースは`libs/sync/src/observable.rs`の**わずか131行**。やることは3つだけです。(1) 常に最新値を1つ保持する、(2) 複数のtaskがそれぞれ「値が変わったら起こして」と購読できる、(3) `set_if_different`で「同じ値なら通知しない」を選べる。実装は`blocking_mutex::Mutex`＋`RefCell`＋`MultiWakerRegistration`＋世代番号という、すべてembassy-syncにある部品の組み合わせです。そして現在のembassy-sync 0.7には、ほぼ同じ役割の`Watch`が標準で入っています。彼らが使っていたembassy-sync 0.5には`Watch`がまだ存在しなかった——つまりこれは「**標準にない部品は、部品の部品から自作してよい**」ことの実証です。

## 身近なたとえ

学校の廊下の**電光掲示板**を想像してください。掲示板には常に最新のお知らせが1件だけ表示されています。生徒は前を通るたびに「前に見たときから変わったか」を掲示の**更新番号**で確かめます。番号が進んでいれば新しいお知らせを読み、同じなら素通りします。掲示板は「誰が読んだか」を気にせず、貼り替えたら**登録してある全員のポケベルを一斉に鳴らす**だけです。

たとえと違うのは、taskは「前を通るたびに確認」しない点です。async/awaitの世界では、taskは`next_value().await`で眠り、掲示板側が更新時にWaker（第9部3ページで学んだ「起こすためのボタン」）を押して起こします。ポーリングではなく通知駆動です。

## 要求 — Channel・Signalでは、なぜ足りないか

第9部9ページの部品と、ロボットの要求を突き合わせてみます。要求は「電池状態・ボール在否・速度指令のような**状態**を、複数のtaskに配りたい」でした。

| 部品 | 苦しい点 |
|---|---|
| `Channel` | キューであり、1つの値は**1人しか受け取れない**。全員に配るには人数分送る必要がある。また古い値が捨てられずに並ぶ |
| `Signal` | 最新値だけを持つ点は合格。しかし`wait()`が値を消費するため、**購読者が複数いると取り合い**になる |
| `PubSubChannel` | 複数購読はできるが履歴キュー方式で、購読者が遅いと取りこぼしの扱いが複雑になる。欲しいのは履歴ではなく「今の値」 |
| `Mutex<T>` | 最新値の保持はできるが、**変化を待つ**手段がない（ポーリングになる） |

つまり「**Signalの複数購読者版**」あるいは「**Mutexに通知機能を足したもの**」が欲しい。既製品にぴったりの棚がなければ、作るのが組み込みRustの流儀です。ではその作り方を、実コードで見ていきます。

## コードを読む(1) — 器の構造

以下はすべてMITライセンスのluhsoccer_firmware `libs/sync/src/observable.rs`からの抜粋です。まず器から。

```rust
// 抜粋: luhsoccer_firmware libs/sync/src/observable.rs（MIT）
pub struct Observable<M: RawMutex, T, const SUBS: usize> {
    inner: Mutex<M, RefCell<ObservableState<T, SUBS>>>,
}

struct ObservableState<T, const SUBS: usize> {
    value: T,                                  // 最新値そのもの
    wakers: MultiWakerRegistration<SUBS>,      // 眠っている購読者のWaker置き場
    id: u64,                                   // 更新のたびに増える世代番号
    subs: usize,                               // 現在の購読者数
}
```

ここの`Mutex`はasyncの`mutex::Mutex`ではなく、`embassy_sync::blocking_mutex::Mutex`です。`lock(|x| ...)`とクロージャで使い、中では`.await`できない代わりに、一瞬で終わるアクセスを割り込みからも安全に行えます。`RefCell`と組み合わせて「短い区間だけ中身を書き換える」のは、embassy-sync自身も内部で使う定石です。型引数`M: RawMutex`のおかげで、割り込みをまたぐなら`CriticalSectionRawMutex`、1つのexecutor内だけなら`NoopRawMutex`と、使う側が保護の強さを選べます。これも第9部で学んだ型引数の使い方そのままです。

`const SUBS: usize`は購読者の最大数です。ヒープのないno_std環境では、Wakerを置く配列の大きさをコンパイル時に決める必要があります。メイン基板ではすべて`Observable<CriticalSectionRawMutex, T, 8>`、つまり最大8購読者でした。

## コードを読む(2) — 書く側

```rust
// 抜粋: luhsoccer_firmware libs/sync/src/observable.rs（MIT）
pub fn set(&self, value: T) {
    self.inner.lock(|cell| {
        let mut inner = cell.borrow_mut();
        inner.value = value;
        inner.id += 1;          // 世代番号を進める
        inner.wakers.wake();    // 眠っている購読者を全員起こす
    })
}

pub fn set_if_different(&self, value: T)
where
    T: PartialEq,
{
    self.inner.lock(|cell| {
        let mut inner = cell.borrow_mut();
        if inner.value != value {
            inner.value = value;
            inner.id += 1;
            inner.wakers.wake();
        }
    })
}
```

`set`は3行の仕事です。値を置き換え、世代番号`id`を進め、登録済みのWakerを全部押す。`set_if_different`は`PartialEq`（`==`で比べられる型）に限定して「値が変わらないなら何もしない」を足したものです。たとえばモータ基板から1kHzで届く実測速度をそのまま`set`すると、購読者は毎秒1000回起こされます。実際には値が変わったときだけ起こせば十分——前ページのreceive_taskが`actual_velocity.set_if_different(velocity)`としていたのはこのためです。**通知の洪水を書く側で堰き止める**、消費電力にもCPU時間にも効く設計です。

## コードを読む(3) — 購読者になる

```rust
// 抜粋: luhsoccer_firmware libs/sync/src/observable.rs（MIT）
pub fn subscriber(&self) -> Result<Subscriber<'_, M, T, SUBS>, Error> {
    self.inner.lock(|cell| {
        let mut inner = cell.borrow_mut();
        if inner.subs >= SUBS {
            return Err(Error::SubscriberLimit);   // 定員オーバー
        }
        inner.subs += 1;
        Ok(())
    })?;
    Ok(Subscriber {
        sub_var: self,
        last_id: 0, // Always initialize the last id to 0 so the current value is received
                    // once.
    })
}
```

購読者は定員制で、超えると`Err`が返ります（配列が溢れて壊れるのではなく、型で断られる）。注目は原文コメントにもある`last_id: 0`です。器側の`id`は1から始まるので、新しい購読者は必ず「自分の知らない世代」から始まります。つまり**購読開始直後の最初の`next_value()`は、待たずに現在値を返します**。「購読した瞬間の状態をまず知りたい」はこの手の部品のよくある要求で、それを初期値1つで解決しています。

`Subscriber`には`Drop`実装もあり、購読者が消えると`subs`を1減らして席を返します。Rustの所有権が「席の返し忘れ」を防いでいる例です。

## コードを読む(4) — 心臓部のnext_value

```rust
// 抜粋: luhsoccer_firmware libs/sync/src/observable.rs（MIT）
pub async fn next_value(&mut self) -> T {
    poll_fn(|cx| {
        self.sub_var.inner.lock(|cell| {
            let mut inner = cell.borrow_mut();
            if self.last_id < inner.id {
                self.last_id = inner.id;          // ここまで読んだ、と記録
                Poll::Ready(inner.value.clone())  // 新しい値がある → 即返す
            } else {
                inner.wakers.register(cx.waker()); // なければWakerを置いて
                Poll::Pending                      // 眠る
            }
        })
    })
    .await
}
```

第9部3ページで学んだFutureの仕組みが、そのまま12行に収まっています。`poll_fn`は「pollされるたびにこのクロージャを実行するFuture」を作る関数です。ロジックは世代番号の比較だけです。

1. 自分が最後に見た`last_id`より器の`id`が新しければ、`last_id`を更新して値のクローンを返す（`Poll::Ready`）
2. 新しくなければ、自分のWakerを`MultiWakerRegistration`に登録して`Poll::Pending`——つまり眠る
3. 誰かが`set`すると`wakers.wake()`で起こされ、executorが再びこのクロージャをpollし、今度は1.の道を通る

大事な性質がもう1つ読み取れます。`set`が短時間に3回呼ばれても、購読者が受け取るのは**最後の値だけ**です（`id`が3進んでも、比較は「新しいか否か」だけなので）。Channelのような「全部届く」保証はありません。速度指令やセンサ値のような「最新だけが意味を持つ状態」にはこれが正解で、ボタン押下イベントのような「1個も落とせない出来事」には不正解です。部品選びの軸は常に**値の意味**です。

## embassy-sync 0.7のWatchと比べる

現在の教材環境（embassy-sync 0.7）には、これとほぼ同じ役割の`Watch`が標準で入っています。`Watch`がembassy-syncに追加されたのは0.6.1（2024年11月）で、luhsoccer_firmwareが依存するのは0.5。**Observableは、後に標準入りする機能の先行自作**だったことになります。並べて見ると:

| | Observable（自作、131行） | Watch（embassy-sync 0.7） |
|---|---|---|
| 最新値の保持 | あり（初期値必須の`new(value)`） | あり（`new()`は値なしで開始、`new_with(value)`も可） |
| 複数購読者 | `const SUBS`で定員、超過は`Err` | `const N`で定員、`receiver()`が`Option`を返す |
| 変化待ち | `next_value().await` | `changed().await` |
| 今の値を読むだけ | `get()` | `try_get()` / `get().await`（値が入るまで待つ版） |
| 条件付き更新 | `set_if_different`（`PartialEq`で比較） | `send_if_modified`（クロージャで自由に判定） |
| 追加機能 | なし | 条件付き待ち`changed_and`、定員を消費しない`anon_receiver`、`clear`など |
| 通知の仕組み | 世代番号＋`MultiWakerRegistration` | 同系統（メッセージIDベース） |

中身の発想はほとんど同じです。違いは、標準品が「値なし開始（中身は`Option`）」「クロージャによる汎用の条件付き送信」など**多くのプロジェクトの要求の和集合**を引き受けて大きくなっている点です（実装ファイルはドキュメントとテスト込みで1000行超）。自作のObservableは自分たちの要求（初期値は必ずある、比較は`==`で十分）に絞ったぶん、131行で読み切れます。

ここから持ち帰る判断基準はこうです。

1. **まず標準（embassy-sync）を探す**。ChannelでもSignalでもWatchでもない要求か、本当に確かめる
2. なければ、**標準の部品（blocking_mutex、waitqueue、poll_fn）を土台に、薄く自作する**。ゼロからunsafeで書くのではない
3. 自作した部品は**汎用ライブラリにせず、プロジェクト専用に小さく保つ**。彼らのsyncクレートも公開APIは実質この1個
4. 後に標準へ同等品が入ったら、乗り換えを検討できるようにしておく（次の節の通り、依存が薄いほど乗り換えも楽）

## C6でも動くのか

このファイルの依存を`libs/sync/Cargo.toml`で確かめると、embassy-syncとdefmt、heaplessだけです。`observable.rs`自体が使うのは`core`とembassy-sync、それにエラー型のログ表示（`Format`導出）用のdefmtのみで、**RP2040固有のコードは1行も含みません**。使っている`blocking_mutex::Mutex`・`waitqueue::MultiWakerRegistration`・`poll_fn`はいずれも教材のembassy-sync 0.7にも同名で存在します。つまり依存関係を見る限り、この131行はESP32-C6のプロジェクトへそのまま持ち込める形をしています（本教材ではC6向けのビルド検証まではしていないため、「移植の壁が原理的にない」という事実の指摘にとどめます）。

チップが変わっても同期プリミティブがそのまま使える——これは彼らが偶然そう書いたのではなく、embassy-syncが**HALから独立した層**として設計されているからです。第9部でChannelやSignalを学んだとき、そこにesp-halの型が一切出てこなかったのと同じ理由です。

## よくある誤解

- **「Mutexの中でawaitしていないのはなぜ？」**: ここで使われているのはblocking_mutexで、ロックはクロージャの間だけ保持され、`.await`をまたぎません。だから割り込みコンテキストからも安全に触れます。asyncの`mutex::Mutex`とは別物です。混同すると「awaitできない」というコンパイルエラーに出会います（これは制約ではなく、ロックを持ったまま眠る事故を型が防いでいるのです）。
- **「next_valueですべての更新を観測できる」**: できません。眠っている間に複数回`set`されたら、起きたとき受け取るのは最新値1つです。全部必要ならChannelを使います。
- **「idはu64だけど溢れない？」**: 1kHzで更新し続けても、u64が一周するには約5億年かかります。組み込みでも安心して「実質溢れない」とみなせる幅です。

## 確認問題

1. `Subscriber`の`last_id`が0で初期化され、器の`id`が1から始まるのはなぜですか？

<details>
<summary>答え</summary>

新しい購読者の最初の`next_value()`で必ず`last_id < id`が成立し、現在値が待ちなしで1回届くようにするためです。「購読開始時点の状態をまず知る」を追加コードなしで実現しています。

</details>

2. モータ基板からの実測速度を掲示するときに`set`ではなく`set_if_different`を使うと、何が節約できますか？

<details>
<summary>答え</summary>

値が変わっていないときのWaker起床（購読者taskの実行）が丸ごと省けます。1kHzで届く実測値が実際にはあまり変化しない場合、購読者を起こす回数が大幅に減り、CPU時間を節約できます。

</details>

3. あなたのプロジェクトで「最新のWi-Fi接続状態（接続中/切断）を3つのtaskに配りたい」とき、教材環境ではまず何を検討すべきですか？

<details>
<summary>答え</summary>

自作の前に、embassy-sync 0.7の標準品`Watch`を検討します。最新値保持＋複数購読者＋変化通知はWatchの守備範囲そのものです。標準に要求へ合う部品があるなら自作しないのが第一選択です。

</details>

## まとめ

- Observableは「最新値＋複数購読者＋変化時のみ通知」という要求を、blocking_mutex＋RefCell＋世代番号＋MultiWakerRegistrationという**標準部品の組み合わせ131行**で満たした自作同期プリミティブ
- 心臓部は`poll_fn`の中の世代番号比較。新しければReady、古ければWakerを登録してPending——第9部のFutureの知識で完全に読める
- 当時の標準になかった機能は自作してよい。ただし薄く・小さく・チップ非依存に。今なら同じ要求はembassy-sync 0.7の`Watch`が標準で満たす

## 次のページ

taskどうしの通信の次は、**基板どうし**の通信です。メイン基板とモータ基板を結ぶUARTの上で、postcard・COBS・CRC16を重ねた自前プロトコルがどう組み立てられているかを読みます。教材の最終プロジェクトで手書きした8バイトのパケットの、実戦版です。

[7. 基板間UARTプロトコル — postcard+COBS+CRC16](/embassy-esp32-c6/robot/07-uart-protocol/)

前のページ: [5. メイン基板を読む — 11個のtaskの分業](/embassy-esp32-c6/robot/05-maincontroller/)
