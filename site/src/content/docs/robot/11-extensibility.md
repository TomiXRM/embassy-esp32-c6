---
title: "11. アプリ追加=task追加 — asyncが効く理由"
description: 機能追加がspawn1行で済むのはなぜか。テストtaskの注入、taskの子spawn、優先度3層への無変更配置——luhsoccer_firmwareの実例4つで検証します。
lesson: 11
difficulty: advanced
estimated_minutes: 30
prerequisites:
  - robot/10-failsafe
  - part09/05-spawner
  - part09/08-join
status: complete
code_status: none
last_verified: "2026-07-18"
sources:
  - https://github.com/luhbots/luhsoccer_firmware
---

## このページでできるようになること

- 「機能追加=static 1個+spawn 1行+ファイル1個」がこのファームで成り立つ理由を、所有権と通信の面から説明できる
- featureフラグでテストtaskを本物と同じ配線に注入する手法を説明できる
- taskコードを1行も変えずに優先度の違う実行器へ配置できる構造を説明できる

## 先に結論

この応用編を貫く問いは「なぜEmbassyのasync設計だと、後からアプリ（機能）を足すのが楽なのか」でした。luhsoccer_firmwareには証拠が4つあります。**①main.rsのspawnリストがそのまま機能一覧**になっていて、LEDなどの機能追加はstatic変数1個+spawn 1行+新ファイル1個で済む。**②テスト用taskをfeatureフラグで本物と同じObservableに注入**でき、本番コードを書き換えずに単体で動作確認できる。**③taskがSpawnerを受け取って子taskを産める**ので、機能の内部構造は外から見えない。**④join3やselect_biased!でtask内の並行**もできるので、なんでもtaskにする必要すらない。これが成り立つ理由は一つに集約されます——**各taskはペリフェラルを所有権ごと抱え込んで自己完結し、外との接点は型の付いたObservable/Channelだけ**。結合が細いから、足しても既存が壊れないのです。

## 身近なたとえ

文化祭の出店に似ています。各クラス（task）は自分の機材と材料（ペリフェラル）を自分で持ち込み、他のクラスとのやりとりは校内放送（Observable）だけ。新しい出店を1つ増やすには、実行委員の出店リスト（main.rsのspawnリスト）に1行書き足せばよく、既存の店の内部を作り直す必要はありません。

たとえと違うのは、文化祭では機材の「貸して/返して」が口約束なのに対し、Rustでは所有権としてコンパイラが強制する点です。あるtaskが持つUARTを別のtaskがこっそり触るコードは、そもそもコンパイルが通りません。

## 実証① spawnリストが機能一覧そのもの

出典はすべて [luhbots/luhsoccer_firmware](https://github.com/luhbots/luhsoccer_firmware)（MITライセンス）です。メイン基板の`maincontroller/src/main.rs`の後半は、こういう並びです。

```rust
// 抜粋: maincontroller/src/main.rs
spawner.must_spawn(watchdog_task(p.WATCHDOG));
spawner.must_spawn(dribbler_task(p.PIN_20, p.PWM_CH2, &DRIBBLER_SPEED));
spawner.must_spawn(lightbarrier_task(p.PIN_15, &HAS_BALL));
spawner.must_spawn(rf_task(/* ピンとObservable群 */));
spawner.must_spawn(motorcontroller_task(/* ピンとObservable群 */));
spawner.must_spawn(ui_task(/* ... */));
spawner.must_spawn(buzzer_task(sm0, &VOLTAGE_STATE));
spawner.must_spawn(config_task(p.FLASH, &CONFIG, &SAVE_CONFIG_SIGNAL));
spawner.must_spawn(measure_task(/* ... */));
spawner.must_spawn(led_task(leds, &VOLTAGE_STATE, &NO_RF_CONNECTION));
```

このリストを上から読むだけで「Watchdog、ドリブラー、ボールセンサ、無線、モータ基板連携、UI、ブザー、設定保存、電池監視、LED」という機能一覧が分かります。**設計書とコードが一致している**——5ページで見た11 taskの分業の、これが配線盤です。

たとえばLED表示という「アプリ」の実体は、(1) `static NO_RF_CONNECTION: Observable<...>`のようなstatic変数、(2) `led.rs`という1ファイル、(3) 上のspawn 1行、の3点だけです。既存のrf_taskは`no_connection.set(true)`と書き込むだけで、**LEDが読んでいることを知りません**。だからLEDの次に「無線切断でブザーを鳴らす機能」を足したくなったら、同じObservableを購読するtaskをもう1つspawnするだけで、rf_taskもled_taskも無変更です。

## 実証② テストアプリを「本物の配線」に注入する

このファームで一番おもしろい仕掛けがこれです。

```rust
// 抜粋: maincontroller/src/main.rs
#[cfg(feature = "test_dribbler")]
spawner.must_spawn(dribbler_test_task(&DRIBBLER_SPEED));
```

`--features test_dribbler`付きでビルドしたときだけ、ドリブラーを一定パターンで回すテストtaskが追加でspawnされます。注目すべきは引数です——**本物のrf_taskが書き込むのと同じ`&DRIBBLER_SPEED`**を受け取っています。テスト専用の裏口APIではなく、本番とまったく同じObservableへ値を流し込むので、dribbler_task側はテストか本番かを区別できませんし、する必要もありません。

ではテスト中に本物の指令と衝突したら? モータ基板側の同種のテスト（test_motors / test_kicker）では、UARTから来る本物の指令を`#[cfg(not(...))]`でマスクしています。

```rust
// 抜粋: motorcontroller/src/maincontroller.rs
Main2Motor::Drive(velocity) => {
    // ...
    #[cfg(not(feature = "test_motors"))]
    movement_setpoint.set_if_different(movement);
    #[cfg(feature = "test_motors")]
    debug!("Test build. test value {} is not changed to {}", /* ... */);
}
```

テストビルドでは「受信はするが、Observableには書かずログに出すだけ」。つまり**注入と遮断がどちらもObservableの書き込み点1か所の制御で済む**わけです。データの通り道が型付きの一本道だからできる芸当で、グローバル変数を方々から書き換える設計ではこうはいきません。

さらにCargo.tomlには`keeper`（キーパー機体）や`lupfer`という機体バリアントのfeatureもあります。前ページで見た「keeperビルドでは常にチップキック」のような機体差も、同じ仕組みで1つのコードベースに同居しています。

## 実証③ taskが子taskを産む — 機能の内部は外から見えない

```rust
// 抜粋: maincontroller/src/motorcontroller.rs
pub async fn motorcontroller_task(
    /* ... */
    spawner: Spawner,
) {
    // ...UARTを初期化して送受に分割...
    let (rx, tx) = uart.split();

    spawner.must_spawn(receive_task(
        MotorControllerReceiver::new(rx),
        actual_velocity,
        kicker_voltage,
    ));
    send(/* txを使う送信ループ */).await;
}
```

motorcontroller_taskは引数で`Spawner`を受け取り、自分でUARTを送受に分割してから、受信担当の`receive_task`を**自分で**spawnします。main.rsから見えるのは「motorcontroller_taskを1つspawnした」ことだけで、内部が実は2 taskで動いていることは隠されています。機能の内部構造を変えても（taskを増やしても減らしても）、main.rsのspawnリストは変わらない——**拡張の影響範囲が機能の内側に閉じる**構造です。

## 実証④ なんでもtaskにしない — task内並行

前ページで見たUARTキープアライブは、速度・ボール有無・キックの3系統を並行に送りますが、taskは3つではありません。

```rust
// 抜粋: maincontroller/src/motorcontroller.rs
join3(has_ball_fut, velocity_fut, kick_speed_fut).await;
```

1つのtaskの中で3つのasyncブロックを`join3`で同時に走らせています。3系統は同じUART送信口（Mutexで保護）を共有するので、同じtaskにまとめる方が自然です。kicker_taskも5つのObservableの変化を`select_biased!`で1本のループにまとめていました（9ページ）。**「並行にしたい」と「taskを増やす」は別の判断**で、第9部で学んだjoin/selectがその中間の道具になります。taskにするのは独立した機能単位、task内並行にするのは1機能の中の複数の待ち、という使い分けです。

## 優先度3層へ、taskコード無変更で配置する

モータ基板の`motorcontroller/src/main.rs`には実行器が3つあります。

| 実行器 | 実体 | 載っているtask |
|---|---|---|
| EXECUTOR_HIGH | InterruptExecutor（ソフトウェア割り込みSWI_IRQ_0の優先度で動く） | kicker_task |
| EXECUTOR_CORE1 | core1（2つ目のCPUコア）専用のExecutor | motors_task（1kHz制御） |
| EXECUTOR_LOW | メインループのExecutor | watchdog、UART連携、設定、ログ |

重要なのは、**どのtaskの定義にも「自分がどの実行器で動くか」が書かれていない**ことです。kicker_taskは`#[task]`の付いたふつうのasync関数で、高優先に置かれているのはmain.rsのspawn呼び出しの場所がそう決めているだけです。「この処理は時間に厳しいから高優先へ」「この制御ループは専用コアへ」という配置換えが、**taskコード無変更でspawn行の移動だけ**でできる。優先度設計を後から調整できるのは、拡張性のもう一つの顔です。

## なぜasyncだと足しやすいのか — 原理に戻る

4つの実証に共通する理由を整理します。

1. **taskは所有権で自己完結する**: ペリフェラル（ピン、UART、SPIなど）は引数でtaskに渡され、以後そのtaskだけの持ち物になります。「他の機能が同じピンを触っていて壊れる」事故は、コンパイル時に締め出されます
2. **外との接点は型付きの通信だけ**: taskどうしをつなぐのは`Observable<..., LocalVelocity, 8>`のような型の付いたstaticだけで、書き手は読み手を知りません。接点が細く型で守られているから、taskを足しても既存の接点は変化しません
3. **taskは軽い**: Embassyのtaskはstaticに配置される状態機械で（第9部）、OSのスレッドのようなスタックを持ちません。「気軽に足せるコスト」が拡張の前提を支えています

これは教材第9部で学んだ設計原則と、最終プロジェクト（第12部10）で実践した構造の、そのままのスケールアップです。static+Channel/Signalで配線した無線ボタン端末と、Observable×10本で配線したこのロボットは、**同じ設計が3桁違う規模で通用する**ことの証明になっています。

ただし正直な注意も一つ。asyncにすれば**自動的に**拡張しやすくなるわけではありません。このファームが拡張しやすいのは、通信を型付きの一本道に限る・ペリフェラルを共有しない、という**規律**を守っているからです。asyncとtaskはその規律を安く実行できる道具、というのが正確な言い方です。

## C6への正直な翻訳

このモータ基板はRP2040(2コア)なので、そのままESP32-C6には移りません。

- **core1相当は不可**: C6はシングルコア（RISC-V 1コア）なので、EXECUTOR_CORE1のような「制御ループにコアを1つ専有させる」構成は再現できません
- **InterruptExecutor相当は可**: 高優先の実行器という考え方自体は、esp-rtos（この教材のスタック）に相当機能があります。「時間に厳しいtaskを高優先へ」という設計はC6でも実践できます
- スレッド実行器＋Observable/Channel/Signal＋featureフラグの部分は、embassy-syncとCargoの標準機能なので**そのまま**使えます

「2コア前提の設計を1コアへ持ち込むときは、専有コアの代わりに優先度で守る」——これが読み替えの要点です。

## よくある誤解

- **「taskを足すとその分ロボットが遅くなる」**: 待ち中心のtask（ボタン待ち、1Hz送信など）は、待っている間CPUを使いません（第9部1ページ）。増えて問題になるのはCPUを実際に使う計算で、だからこそ1kHzの制御ループだけはコア専有や高優先で守られています
- **「テストコードは本番と別の仕組み（モックの注入口など）が必要」**: このファームは本番の配線（Observable）をそのまま注入口として使い、featureフラグで書き込み元を差し替えるだけです。通信の通り道が最初から1本に絞られていれば、テスト専用の裏口は要りません
- **「並行処理したければtaskを増やすしかない」**: join/selectでtask内並行という選択肢があります。共有物（このコードではUART送信口）を一緒に使う処理どうしは、むしろ1つのtaskにまとめる方が設計として素直です

## 確認問題

1. 「無線が切れたらブザーを鳴らす」機能をこのファームに足すとします。変更が必要なファイル・行を挙げてください（rf_taskの変更は必要ですか?）。

<details>
<summary>答え</summary>

rf_taskの変更は不要です。rf_taskはすでに`NO_RF_CONNECTION`というObservableに切断状態を書き込んでいるので、(1) それを購読して鳴らすtaskを新ファイルに書き、(2) main.rsにspawn 1行を足すだけです（既存のbuzzer_taskを拡張するなら、その引数に`&NO_RF_CONNECTION`を足す形でも可）。書き手は読み手を知らないので、購読者が増えても送信側は無変更です。

</details>

2. `dribbler_test_task`が「本物と同じ`&DRIBBLER_SPEED`」を受け取ることには、どんな利点がありますか。

<details>
<summary>答え</summary>

テストが本番とまったく同じ経路（Observable→dribbler_task→PWM出力）を通るので、テストで動けば本番の配線も動くと確認できます。dribbler_task側にテスト用の分岐や裏口APIを作る必要がなく、本番コードがテストのために汚れません。

</details>

3. kicker_taskを高優先から低優先へ移したいとき、kicker.rsのコードはどこを変える必要がありますか。

<details>
<summary>答え</summary>

kicker.rs側は無変更です。taskの定義は自分がどの実行器で動くかを知らないので、main.rsでspawnする場所（`EXECUTOR_HIGH`のspawnerか、低優先executorのspawnerか）を変えるだけで配置が変わります。

</details>

## まとめ

- spawnリスト=機能一覧。機能追加はstatic 1個+spawn 1行+ファイル1個で、既存taskは無変更——結合が「型付きの通信」だけに絞られているため
- featureフラグでテストtaskを本物のObservableに注入し、本物の入力を`#[cfg(not)]`で遮断できる。子task・task内並行(join/select)・実行器の選択と、拡張の道具は段階的に選べる
- これは教材第9部と最終プロジェクトの設計原則がそのまま17,000行にスケールした姿。ただし効いているのはasyncそのものではなく、asyncが安くしてくれる「所有権で自己完結・通信は一本道」という規律

## 次のページ

4基板・17,000行の読解はここまでです。最後のページでは、このロボットから**C6の自分のプロジェクトへ持ち帰れるもの**を対応表と演習にまとめ、3つの応用編、そして教材全体を締めくくります。

[12. 持ち帰るもの — 17,000行から教材へ](/embassy-esp32-c6/robot/12-lessons/)

---

前のページ: [10. 多層フェイルセーフ — 止まれることが強さ](/embassy-esp32-c6/robot/10-failsafe/)
