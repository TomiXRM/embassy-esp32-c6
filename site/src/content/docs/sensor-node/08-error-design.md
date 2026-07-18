---
title: "8. taskはResultを返せない — 実プロジェクトのエラー設計"
description: Embassyのtaskは戻り値を返せません。esp32c3-embassyのtask_fallibleラッパ、モジュール別エラーenumとFrom連鎖、センサ故障時の劣化運転が、約1900行の実プロジェクトでどう機能しているかを読みます。
lesson: 8
difficulty: advanced
estimated_minutes: 25
prerequisites:
  - sensor-node/07-https
  - part04/08-error-design
  - part09/04-task
  - part12/07-error-recovery
status: complete
code_status: concept-only
last_verified: "2026-07-18"
sources:
  - https://github.com/claudiomattera/esp32c3-embassy
  - https://gitlab.com/claudiomattera/esp32c3-embassy
---

## このページでできるようになること

- Embassyのtaskが`Result`を返せない理由と、`task_fallible`ラッパパターンを説明できる
- モジュールごとのエラーenumと`From`連鎖が、規模の大きいプロジェクトで果たす役割を説明できる
- センサ故障時にダミー値で処理を続ける「劣化運転」の判断と危険性を説明できる

## 先に結論

第4部8ページで、失敗は`Result`で型にして返すと学びました。ところがEmbassyのtaskの戻り値は実質`()`——`?`でエラーを上へ投げようにも、**上には誰もいません**。spawnされたtaskの終了を待ってエラーを受け取る親がいないからです。参照元のesp32c3-embassyは、この制約を`task()`が`task_fallible() -> Result`を呼んでエラーをログに変換する**2段構え**で解決しています。エラー型はモジュールごとに小さなenumを定義し、`From`実装で連鎖させて`?`一発で伝播させます。そしてセンサが故障しても表示パイプラインを止めず、ダミー値で走り続ける**劣化運転**を実装しています。約1900行の実プロジェクトが、教材で学んだエラー設計の原則をどう「実戦形」にしているかを見ていきます。

## 身近なたとえ

学校の宿題なら、できなかった理由を先生に報告できます（`Result`を返す）。でも新聞配達のアルバイトは違います。配達中に一軒だけ留守で渡せなくても、報告する相手はその場にいません。配達員にできるのは、**記録ノートに書き残して（ログ）、残りの配達を続ける**ことだけです。taskは配達員です。出発したら、途中の失敗は自分で記録し、自分で対処するしかありません。

たとえと違うのは、taskの多くはループで永久に走り続けるので、「全部配り終えて帰ってくる」ことすら想定されていない点です。だからこそ、失敗をその場でログにする仕組みが必要になります。

## 仕組み

### なぜtaskはResultを返せないのか

`#[embassy_executor::task]`を付けた関数は、`Spawner::spawn`で実行器に登録されて走り出します。呼び出し元の`main`はspawnした後、taskの終了を`await`しません（そもそも多くのtaskは終わりません）。返した`Result`を受け取る相手が存在しない以上、言語仕様として返せても意味がなく、embassy-executorはtaskの戻り値を`()`に限定しています。

すると困るのが`?`演算子です。`?`は「エラーなら`return Err(...)`する」構文なので、`Result`を返せない関数の中では使えません。taskの中がすべて`match`と`if let`だらけになったら、第4部で身につけた快適なエラー伝播が台無しです。

### task_fallibleラッパ — 2段構えの解決

参照元の答えはシンプルです。Wi-Fi接続taskの実物を見てください（出典: esp32c3-embassy `src/wifi.rs` 120〜125行目、Claudio Mattera、MIT OR Apache-2.0。以下の引用も同じプロジェクト）。

```rust
// これは抜粋です（wifi.rs）
/// Task for WiFi connection
///
/// This will wrap [`connection_fallible()`] and trap any error.
#[embassy_executor::task]
async fn connection(controller: WifiController<'static>, ssid: String<32>, password: String<64>) {
    if let Err(error) = connection_fallible(controller, ssid, password).await {
        error!("Cannot connect to WiFi: {error:?}");
    }
}
```

taskの本体`connection`は5行だけ。実際の仕事は`connection_fallible() -> Result<(), Error>`が全部やります。fallible（失敗しうる）側は普通の`async fn`なので、中では`?`が使い放題です。エラーは最後にtask側の`if let Err`で受け止められ、ログになります。

このパターンは`main`にも適用されています。`main()`はRTC RAMの初期化だけして`main_fallible() -> Result`を呼び、返ってきたエラーを`error!`で表示する——役割の分離が徹底しています。**「失敗しうるロジック」と「失敗の最終処理」を関数の境界で分ける**。これがtaskの制約への実プロジェクトの答えです。

### モジュール別エラーenumとFrom連鎖

参照元は`wifi.rs`、`http.rs`、`clock.rs`、`sensor.rs`のそれぞれが自分専用の小さな`Error` enumを持ちます。たとえばHTTPモジュールはこうです。

```rust
// これは抜粋です（http.rs）
pub enum Error {
    ResponseTooLarge(CapacityError),
    Tcp(TcpError),
    TcpConnect(TcpConnectError),
    Dns(DnsError),
    Reqless(ReqlessError),
}
```

そして各バリアントに`impl From<TcpError> for Error`のような変換を実装します。`?`は失敗時に自動で`From`変換を挟むので、TCP・DNS・HTTPクライアントとレイヤの違うエラーが混ざる処理でも、`request.send(&mut buffer).await?`と書くだけで自分のモジュールの`Error`に揃います。

さらに上の階層では連鎖します。`clock.rs`の`Error`はHTTP側のエラーを`Synchronization(AdafruitIoError)`として包み、`main.rs`の`Error`は`Wifi(WifiError)`や`Clock(ClockError)`を包みます。つまり**エラー型の包含関係が、モジュールの依存関係をそのまま写している**のです。約1900行のプロジェクトでこれが効く理由は3つあります。

- **`?`だけで伝播が完結する** — エラー処理コードがロジックを覆い隠さない
- **どの層で失敗したかがログで一目で分かる** — `Clock(Synchronization(Dns(...)))`のように、失敗の経路が型の入れ子として残る
- **モジュールの独立性が保たれる** — `wifi.rs`は`clock.rs`のエラーを知らない。依存が一方向のまま増築できる（第12部8ページの原則）

### 劣化運転 — センサが壊れても表示は生かす

参照元の測定task（`sensor.rs`）には、印象的な一節があります。センサの読み取りに失敗したとき、taskを止めるのでも、その回をスキップするのでもなく——

```rust
// これは抜粋です（sensor.rs）
let sample = sample_result.unwrap_or_else(|error| {
    error!("Cannot read sample: {error:?}");
    warn!("Use a random sample");
    Sample::random(rng)
});
```

**乱数で作ったダミーの測定値**を下流へ流します。なぜこんなことをするのでしょうか。この気象ステーションでは、測定taskの下流にChannelを挟んで表示taskがいます。測定値が来なくなると、表示の更新も履歴グラフも止まり、電子ペーパーの前に立った人には「壊れているのか、単に更新が遅いのか」すら分かりません。ダミー値でも流れ続けていれば、表示・履歴・スリープのサイクルという**パイプライン全体は生きたまま**保てます。センサだけが壊れた端末と、全部が止まった端末では、現場での直しやすさがまるで違います。これが第12部7ページで学んだ「劣化運転」の実戦形です。

ただし、ダミー値には**本物と見分けがつかない**という危険があります。ログを見なければ、グラフに紛れ込んだ偽の気温に気づけません。教材の`examples/16-sensor-node`が同じ場面で`f32::NAN`（非数）を使ったのは、この危険への別の答えです。NaNは「欠測」として一目で分かり、平均などの計算にも紛れ込みません。参照元＝表示デモを止めないことを優先、教材16＝データの正直さを優先。**どちらを取るかは要求次第**であり、「劣化運転する」と決めた後にも設計判断が残るのです。

### 教材最終プロジェクトとの比較

第12部の最終プロジェクト（`examples/final-wireless-button`）もエラー設計を持っていますが、形が違います。あちらは`error.rs`という**どこにも依存しない1ファイル**に、`DecodeError`（受信データが壊れている理由）と`TxError`（送信が最終的に失敗した理由）という目的別のenumを集約しました。`From`連鎖はありません。規模が小さく、エラーの種類も「通信の受け」と「通信の送り」の2系統だけだからです。

| | final-wireless-button | esp32c3-embassy |
|---|---|---|
| 規模 | 数百行 | 約1900行 |
| エラー型の置き場所 | `error.rs`に集約 | 各モジュールに分散 |
| 伝播の仕組み | 呼び出し側で明示的に処理 | `From`連鎖＋`?` |
| 向いている場面 | エラーの種類が少なく安定 | モジュールが多く増築が続く |

小さいうちは集約が見通しやすく、モジュールが増えたら分散＋`From`連鎖へ。プロジェクトの成長に合わせてエラー設計も成長させる、と覚えてください。

なお、このページの内容はexamplesに対応する検証済みコードがありません（`code_status: concept-only`）。引用は参照元プロジェクトのものです。

## よくある失敗

- **taskの中で`?`を使おうとしてコンパイルエラーになる** — taskは`Result`を返せないため`?`が使えません。エラーは「`?`をやめる」ではなく「fallible関数に切り出す」で解決します。制約はコンパイラの意地悪ではなく、「このエラー、誰が受け取るの？」という設計上の問いです
- **手当たり次第に`unwrap()`でしのぐ** — taskが`Result`を返せないからと`unwrap()`を並べると、センサの一時的な不調ひとつでpanicし、端末全体が止まります。永久に走るtaskにとってpanicは最悪の結果です。ログ＋継続（または劣化運転）を選びます
- **ダミー値を本物と区別できる印なしで流す** — 劣化運転を採るなら、ログに残す・NaNや専用フラグで欠測と分かるようにする、のどちらかを必ず用意します。静かに偽データがグラフへ混ざるのが最悪のパターンです

## やってみよう

`examples/16-sensor-node`の`main`は、測定の失敗を`match`で受けてNaNで継続しています。これを参照元スタイルに書き換える計画を紙に書いてみましょう。`measure_fallible() -> Result<f32, SensorError>`という関数を切り出すとしたら、`SensorError`にはどんなバリアントが要りますか。I2Cのエラーと「一部の測定値がNone」の2系統を数え上げてみてください。

## 確認問題

1. Embassyのtaskが`Result`を返せないのは、技術的にはどんな状況が理由ですか。
2. `From`連鎖によるエラー設計で、`main.rs`の`Error`が`Wifi(WifiError)`を包むとき、モジュールの依存関係について何が言えますか。
3. 「劣化運転にダミー乱数値を使う」参照元と「NaNを使う」教材16、それぞれの利点を1つずつ挙げてください。

<details>
<summary>答え</summary>

1. spawnされたtaskの終了を`await`して戻り値を受け取る親がいないためです。返しても誰も読めない値になるので、embassy-executorは戻り値を`()`に限定しています。
2. 依存の向きが`main.rs → wifi.rs`の一方向であることです。エラー型の包含関係はモジュールの依存関係を写すので、逆向きの包含が現れたら設計の危険信号です。
3. 参照元（乱数）: 表示・履歴のパイプラインが本物らしいデータで動き続け、下流のデモや動作確認が止まらない。教材16（NaN）: 欠測が一目で分かり、偽データが本物に紛れ込まない。

</details>

## まとめ

- taskはエラーの受け取り手がいないため`Result`を返せない。`task()`＋`task_fallible() -> Result`の2段構えで、ロジック内では`?`を取り戻す
- モジュール別エラーenum＋`From`連鎖は、`?`一発の伝播と一方向依存を両立させる。エラー型の入れ子は失敗の経路の記録になる
- 劣化運転は「止めない」ための設計。ただしダミー値の正直な扱い（ログ・欠測の印）まで含めて設計する

## 次のページ

エラーの次は、ペリフェラルの「持ち主」の設計です。mainが握っている全ピンを、どうやって各taskへ安全に分配するのか——型がその答えになります。

[9. 型でピンの持ち主を決める →](/embassy-esp32-c6/sensor-node/09-peripherals-types/)

---

前: [7. no_stdでHTTPS](/embassy-esp32-c6/sensor-node/07-https/) | 次: [9. 型でピンの持ち主を決める](/embassy-esp32-c6/sensor-node/09-peripherals-types/)
