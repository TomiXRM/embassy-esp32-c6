---
title: "7. 基板間UARTプロトコル — postcard+COBS+CRC16"
description: メイン基板とモータ基板を結ぶintra-commsクレートを読解します。共有メッセージ定義、postcardとCOBSフレーミング、CRC16、1Hzキープアライブ、そしてCAN/TWAIとの対比。
difficulty: advanced
estimated_minutes: 30
prerequisites:
  - robot/06-observable
  - part08/02-uart-async
  - part08/09-twai-basics
  - part12/10-final-project
status: complete
code_status: concept-only
last_verified: "2026-07-18"
sources:
  - https://github.com/luhbots/luhsoccer_firmware/blob/main/libs/intra-comms/src/lib.rs
  - https://github.com/luhbots/luhsoccer_firmware/blob/main/libs/intra-comms/src/definitions.rs
  - https://github.com/luhbots/luhsoccer_firmware/blob/main/libs/intra-comms/src/uart.rs
  - https://github.com/luhbots/luhsoccer_firmware/blob/main/maincontroller/src/motorcontroller.rs
---

## このページでできるようになること

- メッセージ定義を共有クレートに置く設計の狙い（バージョン不整合の防止）を説明できる
- postcard→CRC16→COBS→UARTという層の重なりを、バイト列レベルで追える
- COBSフレーミングの原理（0x00区切り）を図で説明できる
- 「手作りしたフレーミング＋CRC＋アドレスは、CAN/TWAIならハードウェアがやってくれる」という対比と、それでもUARTを選ぶ合理性の両方を説明できる

## 先に結論

メイン基板とモータ基板は、UART（1Mbaud、RTS/CTSフロー制御つき）で結ばれています。その上を流れるメッセージは、`intra-comms`という**両方の基板が依存する共有クレート**に、Rustのenum（`Main2Motor` / `Motor2Main`）として定義されています。バイト列への変換は手書きではなく、serde＋postcardで自動化。その外側にCRC16を付け、COBSという方式で「パケットの区切り」を作ってからUARTへ流します。指令が変わらなくても**最低1Hzで再送するキープアライブ**があり、線が抜ければモータ基板側が気づけます。これは教材の最終プロジェクトで手書きした8バイトプロトコルの、そのまま実戦版です。そして一歩引いて見ると、フレーミング・CRC・再送といった彼らの手作り部分は、**CAN（C6ではTWAI）を選べばハードウェアが肩代わりしてくれる**ものでもあります。

## 身近なたとえ

2つの会社（基板）が手紙をやり取りする場面を考えます。まず両社は**同じ書式集**（共有クレート）を使って手紙を書きます。書式集が2冊に分かれていたら、片方だけ改訂されたとき事故が起きます。手紙は封筒（COBSフレーム）に入れ、封筒の切れ目がひと目で分かるようにし、改ざん・汚れ検出のための照合番号（CRC）を添えます。さらに「用件がなくても月1回は近況を送る」（キープアライブ）と決めておけば、便りが途絶えたこと自体が異常のサインになります。

たとえと違うのは、UARTはただのバイトの川で、封筒という概念を最初から持たないことです。区切りも検査も約束も、すべて送る側と受ける側がソフトウェアで作ります。この「作らなければならないもの一式」を今から読みます。

## 層の全体像

`Main2Motor::Drive(velocity)`という1つの値が線の上のバイト列になるまでの変換を、順に積むとこうなります。

```text
Main2Motor::Drive(LocalVelocity { .. })   … Rustのenum（型がある世界）
        │  (1) postcard（serde）で直列化
[タグ][forward][left][counterclockwise]    … コンパクトなバイト列
        │  (2) CRC16を計算して末尾に2バイト追加
[データ...][CRC上位][CRC下位]
        │  (3) COBSで0x00を消し、フレーム末尾に区切りの0x00を付ける
[COBS符号化されたデータ...][0x00]
        │  (4) UART 1Mbaud + RTS/CTS
────────────────────────────→ モータ基板へ
```

受信側は逆順です。0x00までを1フレームとして切り出し（COBS復号）、CRCを照合し、postcardでenumへ戻します。送信のコードは`libs/intra-comms/src/uart.rs`で、postcardの「フレーバ」（変換処理を積み重ねる仕組み）としてこの3層が1行に現れます。

```rust
// 抜粋: luhsoccer_firmware libs/intra-comms/src/uart.rs（MIT）
async fn send<const N: usize>(&mut self, message: &T) -> Result<(), SendError<Tx>> {
    let buf = postcard::serialize_with_flavor::<T, Crc16<Cobs<HVec<N>>>, Vec<u8, N>>(
        message,
        Crc16::new(Cobs::try_new(HVec::default())?),
    )?;
    self.tx.write_all(&buf[..]).await.map_err(SendError::Io)
}
```

`Crc16<Cobs<HVec<N>>>`という型がそのまま層の図です。内側から、固定長バッファ（HVec）に、COBS符号化を通して、CRC16を計算しながら書き込む。ヒープなしで、バッファサイズ`N`はメッセージごとにコンパイル時に決まります。

## メッセージ定義は共有クレートに置く

メッセージは`libs/intra-comms/src/definitions.rs`に定義されています。

```rust
// 抜粋: luhsoccer_firmware libs/intra-comms/src/definitions.rs（MIT）
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Main2Motor {
    Drive(LocalVelocity),
    /// mm/s
    Kick(u16),
    /// mm/s
    Chip(u16),
    /// us
    KickRaw(u16),
    BallInDribbler,
    BallNotInDribbler,
    CalibrateCapVoltage(u8),
    ChargeHint(KickerChargeHint),
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Motor2Main {
    MotorVelocity(LocalVelocity),
    // V
    CapVoltage(u8),
}
```

方向ごとにenumが1つ。「メイン→モータは指令、モータ→メインは実測の報告」という**会話の非対称性**が型に現れています。`#[derive(Serialize, Deserialize)]`を付けるだけで、バイト列への変換はpostcard（serdeのno_std向けコンパクト形式）が生成します。手書きの変換コードはゼロです。

重要なのは中身より**置き場所**です。このクレートは`maincontroller`と`motorcontroller`の両方が依存します。postcardはenumのバリアントを番号で送るため、もし定義が2か所にコピーされていて片方だけ`Kick`の前にバリアントを足したら、**受信側は別のメッセージとして誤解釈します**。エラーにならず「間違って成功する」のが直列化ズレの怖さです。定義が1か所なら、変更は両方のビルドに同時に届きます。さらにこのクレートは自分のバージョン番号を定数として埋め込み（`crate_version!`マクロ）、ロボットが基地局へ報告するメッセージにファームウェアバージョンとして載せています。相手と版がずれていないかを実行時にも確認できる二段構えです。

教材の最終プロジェクトも実は同じ構造でした。`final-wireless-button`では送信側と受信側の2つのバイナリが、同じ`protocol.rs`（1つのライブラリクレート）を共有しています。並べてみます。

| | 教材 protocol.rs | intra-comms |
|---|---|---|
| メッセージ定義 | enum `Packet`（3種） | enum `Main2Motor`（8種）/ `Motor2Main`（2種） |
| バイト列化 | 手書き（固定8バイト） | serde＋postcard（可変長、自動生成） |
| フレーム区切り | 固定長なので不要 | COBS（可変長なので必須） |
| 破損検出 | XORチェックサム1バイト | CRC16（2バイト） |
| 定義の共有 | 1クレートを2バイナリで共有 | 1クレートを2基板のファームで共有 |
| 版ずれ対策 | MAGICバイト | 共有クレート＋バージョン番号の実行時報告 |

手書きとpostcardの損得もここから読めます。固定8バイトの手書きは、仕様がバイト図で完全に説明でき、学ぶには最良です。しかしメッセージが10種類・可変長になった途端、手書きの変換とその保守はバグの温床になります。**種類が増えたらserdeに任せ、人間は型の設計に集中する**のが実戦の選択です。

## COBSフレーミング — 0x00区切りの原理

UARTはバイトの川なので、「どこからどこまでが1つのメッセージか」を自分で決める必要があります（第8部で学んだ通りです）。素朴には「区切りとして0x00を送る」と決めたいのですが、データの中身にも0x00は現れます。COBS（Consistent Overhead Byte Stuffing）は、**データ中の0x00を必ず除去できる**変換です。

原理は「0x00を『次の0x00までの距離』に書き換える」ことです。

```text
元のデータ:            11 22 00 33
                              └ データ中に0x00がある

COBS符号化:      [03] 11 22 [02] 33   [00]
                  │          │        └ フレーム終端（本物の区切り）
                  │          └ 「次の0x00は2バイト先」（元の00の置き換え）
                  └ 先頭に追加:「最初の0x00は3バイト先」
```

符号化後のフレーム本体には0x00が絶対に現れません。だから受信側は**0x00が来たら無条件にフレーム終端**と判断できます。途中で受信を始めても、ノイズでバイトが化けても、次の0x00から必ず復帰できる——これがCOBSの最大の美点です。オーバーヘッドは原則先頭の1バイトだけ（長いデータでも254バイトごとに1バイト）と小さく、組み込みのフレーミングの定番になっています。受信側（`uart.rs`の`Receiver`）は、UARTのバッファから読んだバイトを`CobsDecoder`に流し込み、フレームが完成した時点でpostcardの復号へ渡しています。

## CRC16 — そして実物ならではの発見

XORチェックサム（教材版）は1バイトの誤りは見つけられますが、複数バイトの誤りを見逃しやすい簡易検査です。CRC（巡回冗長検査）は連続したビット化け（バースト誤り）に強く、通信では標準的な選択です。intra-commsはcrcクレートの`CRC_16_ISO_IEC_14443_3_A`（非接触ICカードの規格で使われる16ビットCRC）を使い、postcardのフレーバとして「書き込むバイトを逐次CRC計算へ足し、最後に2バイト追記する」形で実装しています。

```rust
// 抜粋: luhsoccer_firmware libs/intra-comms/src/uart.rs（MIT）
impl<B> SerFlavor for Crc16<B>
where
    B: SerFlavor,
{
    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        self.crc.digest().update(&[data]);
        self.flav.try_push(data)
    }

    fn finalize(mut self) -> postcard::Result<Self::Output> {
        let crc = self.crc.digest().finalize();
        self.flav.try_extend(&crc.to_be_bytes())?;
        self.flav.finalize()
    }
    // ...
}
```

設計としては教科書通りです。ただ、このコードを注意深く読むと1つ気づくことがあります。crcクレート（3.0系）の`digest()`は、**呼ぶたびに初期状態の新しい計算器（Digest）を返す**APIです。上のコードは`self.crc.digest().update(...)`と、作った計算器をその場で使い捨てています。この読みが正しければ、1バイトごとの計算結果はどこにも蓄積されず、`finalize`で付くのは毎回「空データのCRC」という**定数**になります。受信側もまったく同じ実装で照合するので、通信は何事もなく動きます。しかしデータが途中で化けても、この2バイトは一致してしまう——CRCが実質「2バイトの固定フッタ」になっている可能性が高いのです（破損検出はCOBS復号やpostcardの形式チェックに頼ることになります）。

これを紹介するのは欠点探しのためではありません。持ち帰るべき教訓が2つあります。

1. **両端が同じ実装を共有すると、バグまで対称に共有される**。だから「通信が動いている」ことは「プロトコルが正しい」ことの証明にならない。検証には、既知の正解バイト列（テストベクタ）との比較や、別実装との相互接続テストが要る
2. 意図した設計（データ全体をCRCで守る）と実装は別物で、**実プロジェクトのコードでもこの距離は生じる**。読み手として原典を疑いながら読む姿勢は、MITライセンスのコードを学材にする最良の使い方でもある

## キープアライブ — 沈黙もプロトコルの一部

前ページで見た通り、メイン基板側の送信ループ（`maincontroller/src/motorcontroller.rs`）は、Observableの変化を待ちながら、**1秒間変化がなければ現在値をもう一度送ります**。

```rust
// 抜粋: luhsoccer_firmware maincontroller/src/motorcontroller.rs（MIT）
const MAX_TIME_BETWEEN_SENDS: Duration = Duration::from_hz(1);

let velocity_fut = async {
    loop {
        // 変化が来たらその値を、1秒待っても来なければ現在値を送る
        let value = (with_timeout(MAX_TIME_BETWEEN_SENDS, velocity_sub.next_value()).await)
            .unwrap_or_else(|_| command_velocity.get());
        if let Err(e) = sender.lock().await.drive(value).await {
            /* エラーログ */
        }
    }
};
// has_ball_fut / kick_speed_fut も同じ形で、最後に
// join3(has_ball_fut, velocity_fut, kick_speed_fut).await;
```

`with_timeout`で`next_value()`を包むだけで「変化駆動＋最低1Hzの再送」が1つのループになる、気持ちのよいイディオムです。速度・ボール在否・キック設定の3系統は、taskを3つに分けるのではなく**join3で1つのtask内に並行**させています（第9部8ページの判断基準どおり、3つは同じUART送信口を共有するのでtaskを分けるより借用が素直です)。その共有は`Mutex<NoopRawMutex, Sender>`で行います。3つのFutureは同じtaskの中にいて割り込み合わないので、いちばん軽いNoopRawMutexで足ります——第9部で学んだ「保護の強さは共有の範囲で選ぶ」の実例です。

受信が1秒以上途絶えたら異常、と受け手が判断できるのがキープアライブの価値です。無線が切れたときの安全網（50msでゼロ化）と合わせて、このロボットは「入力の沈黙」を必ずどこかの層が検知します。

## CAN/TWAIとの正面対比 — 手作りした物は何だったのか

第8部9〜10ページで学んだTWAI（CANと互換のC6のペリフェラル）を思い出してください。彼らがソフトウェアで手作りした物を並べると、きれいに対応します。

| intra-commsが手作りした物 | CAN/TWAIでは |
|---|---|
| COBSフレーミング（区切り） | フレーム構造をハードウェアが定義・検出 |
| CRC16の計算と照合 | CRC15の計算・照合・ACKをハードウェアが自動実行 |
| 宛先の区別（このUARTは1対1なので不要だが、無線側では同期語で実現） | 11/29ビットIDとハードウェアフィルタ |
| キープアライブと異常検知 | 送達確認（ACK）とエラーカウンタが標準装備 |
| 誤り時の再送（このプロトコルは再送せず次の周期送信に任せる） | ハードウェアが自動再送 |

「じゃあCANにすればよかったのに」と結論するのは早計です。UART選択には明確な合理性があります。

- **全二重**: UARTはTXとRXが独立しており、両方向が同時に流せます。CANは1本の共有バスを全ノードで取り合う半二重です
- **配線が軽い**: 基板間はTX/RX（＋RTS/CTS）の信号線だけ。TWAIは外付けトランシーバICが必須で（C6のピンをCAN_H/CAN_Lへ直結してはいけません）、部品と基板面積が増えます
- **速度**: 1Mbaudは古典CANの上限（1Mbps）と同等で、2枚の基板の点対点なら調停もいらず、帯域を独占できます
- **ソフトの自由度**: postcardのままメッセージを大きくでき、8バイト制限（古典CANのペイロード上限）を気にしなくてよい

つまり「1対1・短距離・両方向」ならUART＋自前プロトコルは軽くて速い。ノードが3つ以上に増え、配線を1本のバスにまとめたく、ノイズ環境で送達保証が欲しいなら、CAN/TWAIの出番です。**トポロジと信頼性要求が選択を決める**のであって、どちらかが常に上位互換なのではありません。

## よくある失敗

- **postcardのバイト列をそのままUARTへ流す（フレーミングなし）**: 受信側が途中から読み始めた瞬間、永久に境界がずれ続けます。可変長データには区切り（COBSや長さプレフィックス）が必須です。固定長だった教材のプロトコルが区切りなしで済んだのは、8バイトという長さ自体が区切りだったからです。
- **メッセージ定義を両側にコピペする**: バリアントの追加・並べ替えが片側だけに入ると、postcardは番号で照合するため**エラーにならずに別の意味へ化けます**。定義は必ず1つのクレートに置き、両方をそこへ依存させます。
- **チェックサムを「実装したから安全」と思い込む**: 本文の通り、実物ですらCRCが機能していない可能性があります。自作プロトコルには、正解バイト列を固定で持つ単体テスト（教材のprotocol.rsが行っている方式）を必ず付けましょう。

## 確認問題

1. COBS符号化後のフレーム本体に0x00が現れないことが、なぜ「途中受信からの復帰」を可能にするのですか？

<details>
<summary>答え</summary>

0x00はフレーム終端にしか現れないため、受信側はどんな状態からでも「次の0x00まで読み捨てれば、その次から新しいフレームの先頭」と確定できるからです。区切りバイトがデータ中に現れうる方式では、この保証がありません。

</details>

2. `Main2Motor`と`Motor2Main`が別のenumに分かれているのは、どんな設計判断ですか？

<details>
<summary>答え</summary>

通信の方向ごとに語彙を分ける判断です。モータ基板が`Drive`を送り返すような「ありえない会話」を型の段階で不可能にし、受信側の`match`も自分に来るメッセージだけを扱えば済みます。

</details>

3. あなたが基板を3枚以上のセンサノード網に拡張し、1本のバスで結びたくなったとします。UART＋intra-comms方式とTWAIのどちらを検討すべきですか？理由も答えてください。

<details>
<summary>答え</summary>

TWAIです。UARTは基本的に1対1で、多ノード化すると配線と調停を自作することになります。TWAIはバス型接続・ID調停・ACK・CRC・再送をハードウェアが提供します（C6では外付けトランシーバが必要な点に注意）。1対1に戻るならUARTの全二重・簡素さが再び有利になります。

</details>

## まとめ

- メッセージ定義は両基板が依存する共有クレートに置く。postcardは便利だが定義ズレに弱いので、「定義が1か所」こそが安全装置
- 変換はpostcard→CRC16→COBSの3層。COBSは「0x00を距離に書き換える」ことで、必ず復帰できるフレーム区切りを作る
- 手作りしたフレーミング・CRC・キープアライブは、CAN/TWAIならハードウェアの仕事。1対1・全二重・軽配線というUARTの利点との天秤で選ぶ

## 次のページ

プロトコルの先、UARTの向こう側にいるモータ基板の中身へ入ります。1kHzで回り続ける制御ループが、起動時の自己診断から固定小数点のPID、電流制限、そして「ループが遅れたことの検知」までをどう作っているかを読みます。

[8. 1kHzの制御ループ — モータ基板を読む](/embassy-esp32-c6/robot/08-motion/)

前のページ: [6. Observableを読む — 131行の自作同期プリミティブ](/embassy-esp32-c6/robot/06-observable/)
