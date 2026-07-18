---
title: "3. 基地局 — EmbassyではなくRTICという選択"
description: EthernetとSX1280無線を橋渡しする基地局はRTIC製です。同一プロジェクト内にRTICとEmbassyが同居する事実から、2つのフレームワークの設計思想を比較します。
difficulty: advanced
estimated_minutes: 20
prerequisites:
  - robot/02-system
  - part09/04-task
  - part09/09-channel-signal-mutex
status: complete
code_status: none
last_verified: "2026-07-18"
sources:
  - https://github.com/luhbots/luhsoccer_firmware
  - https://rtic.rs/1/book/en/
  - https://ssl.robocup.org/wp-content/uploads/2023/02/2023_TDP_Luhbots.pdf
---

## このページでできるようになること

- 基地局の役割（Ethernet⇄無線ブリッジ、最大16台のポーリング、非常停止）を説明できる
- RTICの基本要素（優先度付きtask、shared＋lock、idle）を読める
- RTICとEmbassyの考え方の違いを「優劣」ではなく「設計文化の違い」として説明できる
- no_std環境でprotobuf（prost）を動かすために何が必要かを説明できる

## 先に結論

基地局（Basestation）は、フィールド外のAIとロボット達をつなぐ翻訳機です。マイコンはATSAM4E8C（Cortex-M4）で、**このプロジェクトで唯一Embassyを使っていません**。代わりに**RTIC**（Real-Time Interrupt-driven Concurrency）というフレームワークで書かれています。RTICは「優先度付きの割り込みでtaskを駆動し、共有データはロックで守る」という、Embassyとはかなり違う文化を持ちます。同じチームの同じ製品の中に両方が同居している——これは2つのフレームワークを「実物で」比較できる絶好の教材です。結論を先に言うと、**どちらが優れているかという話ではなく、「何を中心に設計を組み立てるか」が違います**。RTICは優先度と共有資源、Embassyはtaskとデータの流れです。

## 身近なたとえ

RTICは**救急病院の呼び出し体制**に似ています。仕事は「呼び出し（割り込み）」で始まり、すべての仕事に優先度が付いていて、重症患者（高優先度）の処置は軽症の処置に割り込みます。共有の手術室（共有データ）を使うときは鍵をかけ、その間だけは誰にも邪魔されません。Embassyはどちらかというと**流れ作業の工房**で、職人（task)たちが自分の材料が届くのを待ち（await)、届いたら加工して次の職人へ渡します。

——ただし実際の技術では、RTICのロックは「優先度シーリング」という仕組みで実現され、待ち行列で並ぶわけではありません。ロック中は「そのデータを触り得る優先度の割り込みだけ」を一時的に禁止する、極めて短時間の操作です。

## 基地局の仕事

2ページ目の概念図から基地局の部分を拡大すると、仕事は3つです。

1. **Ethernet側**: smoltcp（no_stdのTCP/IPスタック。教材で使ったembassy-netの中身と同じもの）でDHCPからアドレスを取得し、UDPでAIサーバからのprotobufパケットを受ける
2. **無線側**: 受けた指令をロボット別に振り分け、SX1280無線で**最大16台を順番にポーリング**する。ロボットiに送信→短い受信窓で応答（テレメトリ）を待つ→次のロボットへ。応答が来れば往復時間（RTT）も実測する
3. **安全装置**: 基板上の**物理的な非常停止ボタン**。押されている間は全ロボットへ停止状態を送る

3つ目が地味に重要です。SSLは完全自律の競技ですが、ロボットが暴走したときに人間が確実に止める最後の手段は、AIソフトではなく**基地局の物理ボタン**に置かれています。ソフトウェアのどの層が死んでも効く停止手段を一番外側に置く——第12部の「多層防御」の思想がここにも見えます。

## RTICのコードを読む

RTICのプログラムは1つの`#[rtic::app]`モジュールにまとまります。実物の骨格を見てみましょう（抜粋。出典: luhsoccer_firmware (luhbots, MIT) basestation/src/main.rs）。

```rust
#[rtic::app(device = atsam4_hal::pac, dispatchers = [AES, USART0, USART1, EFC])]
mod app {
    #[shared]
    struct Shared {
        state,        // 全ロボットの指令・応答バッファ
        network,      // smoltcpのラッパ
        serial,
        status,
        stop_button,  // 非常停止ボタン
    }

    #[local]
    struct Local {
        ws,           // 状態表示LED
        rf: Transceiver,       // SX1280
        rf_amp: Option<Amp>,   // SKY66112アンプ
    }

    #[idle(shared = [state, network, status, stop_button])]
    fn idle(mut ctx: idle::Context) -> ! { /* 後述 */ }

    #[task(local = [rf, rf_amp], shared = [state, network], priority = 2)]
    fn transmit(mut ctx: transmit::Context, endpoint: IpEndpoint) { /* 無線送受信 */ }

    #[task(shared = [serial, status])]
    fn print_status(mut ctx: print_status::Context) { /* 1秒ごとに状態出力 */ }
}
```

Embassyとの違いがすでに3つ見えています。

- **リソース宣言が中央集権**: 共有データは`Shared`、特定taskだけが使うデータは`Local`として、**アプリの先頭で全部宣言**します。各taskが何を触るかは`#[task(shared = [...], local = [...])]`という属性で申告し、申告していないものには触れません。Embassyでは`static`なChannelやSignalを引数で渡して配線しましたが、RTICではフレームワークが配線表を管理します
- **taskに優先度が付く**: `priority = 2`のtaskは、優先度1のtaskを**実行途中でも中断して**走ります。第9部で学んだ「Embassyのtaskは自分からawaitで手放すまで走り続ける（協調的）」とは逆の、**横取りあり（プリエンプティブ）**の世界です。`dispatchers = [...]`は、この横取りを実現するために間借りする割り込み番号のリストです
- **taskは「短く走って終わる」関数**: Embassyのtaskは`loop`で永遠に生きるのが普通でしたが、RTICのtaskは呼ばれて、仕事をして、**リターンします**。周期実行したければ`print_status::spawn_after(1u64.secs())`のように「1秒後の自分」を予約してから終わります

## idleループと非常停止

RTICでは、どのtaskも走っていないときに`idle`が回ります。この基地局は、Ethernetの世話と非常停止ボタンの監視をidleに置いています（抜粋、簡略化。出典: 同main.rs）。

```rust
#[idle(shared = [state, network, status, stop_button])]
fn idle(mut ctx: idle::Context) -> ! {
    loop {
        // ネットワークを駆動し、AIからの指令が来たらstateへ反映
        ctx.shared.network.lock(|network| {
            let mut send_required = false;
            network.poll(/* 指令をstateへ書き込むクロージャ */);

            // 非常停止ボタン: 押されていたら全ロボットを停止状態に
            ctx.shared.stop_button.lock(|button| {
                if button.is_low().unwrap() {
                    ctx.shared.state.lock(|state| {
                        state.set_stop_state();
                        send_required = true;
                    });
                }
            });

            if send_required {
                transmit::spawn(network.get_latest_server_endpoint()).unwrap();
            }
        });
    }
}
```

読みどころは2つあります。

**`lock`だらけであること。** RTICでは共有データに触るとき、必ず`ctx.shared.xxx.lock(|xxx| { ... })`とクロージャで包みます。ロック中はそのデータを取り合う可能性のある割り込みが一時停止するので、データ競合は起きません。Embassyで`Mutex`の`lock().await`を学びましたが、RTICのlockはawaitしません——**ブロックせず、ごく短時間、割り込みを制限するだけ**です。コンパイル時にどのtaskがどのデータを触るか分かっているからできる芸当で、これがRTICの看板機能です。

**指令が来たときだけ`transmit::spawn`すること。** AIからパケットが届く、またはボタンが押されると、`transmit` task（優先度2）が起動され、無線の送受信を一巡します。データの到着がtaskを起こす——ここだけ見ると、Embassyの「Channelにsendすると受け側taskが起きる」と同じデータフローの発想です。フレームワークが違っても、設計の骨格は似てきます。

`transmit`の中身（ロボットへ送って短い受信窓で応答を待つ往復）は、無線の詳細と一緒に次のページで読みます。

## RTIC vs Embassy — 設計文化の比較

| | RTIC 1.1 | Embassy |
|---|---|---|
| taskの正体 | 優先度付き割り込みハンドラ | async関数（Future） |
| 並行の方式 | **横取りあり**。高優先度が低優先度を中断 | **協調的**。awaitで自発的に譲る |
| 待ち方 | 待たない（短く走って終わる）。周期実行はspawn_afterで予約 | `.await`で眠り、イベントで起きる |
| 共有データ | 中央宣言＋`lock`（優先度シーリング） | Channel/Signal/Mutexを引数で配線 |
| 得意分野 | 応答時間の保証・最悪実行時間の解析 | 「待ち」が多い処理の見通しの良さ |
| コードの形 | リソース表＋短い関数の集まり | 長生きするtaskとデータの流れ |

**RTICの強み**は、時間の保証がしやすいことです。優先度と横取りの規則が単純明快なので、「最悪でも何マイクロ秒で応答できるか」を紙の上で解析できます。ロックの実装も待ち行列なしの数命令で、実行コストがほぼ一定です。

**Embassyの強み**は、「待つ」コードの書きやすさです。基地局のような「受けて、変換して、送る」ブリッジは、awaitで素直に書けます。逆にRTICで長い待ち（たとえば無線応答待ち）をすると、このコードのように**ポーリングループで待つ**ことになりがちです。実際、`transmit` taskの中には送信完了フラグを`loop`で読み続ける箇所があります。Embassyなら`dio1.wait_for_high().await`と書いて、待っている間ほかのtaskへCPUを譲れたはずの場所です。

では、なぜ基地局はRTICなのでしょうか。リポジトリに理由の説明はないので断定はできませんが、状況証拠は読み取れます。ATSAM4EのHAL（atsam4-hal、チームはフォークを使用）は**async対応のHALではない**ため、Embassyのtaskで待つ書き方の恩恵を受けにくいこと。そして基地局の仕事は「短い処理の繰り返し＋確実な応答時間」で、RTICの得意分野に収まっていることです。**フレームワークはイデオロギーではなく、HALの事情と仕事の性質で選ばれる**——同居プロジェクトから読み取れる、いちばん実務的な教訓です。

## no_stdでprotobufを動かす

基地局にはもう1つ、教材で扱わなかった技術が入っています。AIサーバとの共通言語である**protobuf**（Protocol Buffers。Googleの構造化データ形式）を、no_stdのマイコン上で使っている点です。

```rust
// 出典: luhsoccer_firmware (luhbots, MIT) basestation/src/main.rs
#[global_allocator]
static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();
```

Rust用protobufライブラリの**prost**はメッセージの入れ物に`Vec`などヒープを使うため、ヒープなしのマイコンではそのままでは動きません。そこで`embedded_alloc`でヒープ（アロケータ）を用意しています。教材でesp-radioのために`esp-alloc`を入れたのと同じ構図です。「no_std＝ヒープ絶対禁止」ではなく、「**ヒープが必要なライブラリを使うなら、自分でアロケータを持ち込む**」が正確な理解でしたね。

ちなみにロボット内部の通信は、protobufではなくpostcard（no_std前提、ヒープ不要）です。**PC側との境界ではPC文化の形式（protobuf）を、マイコン同士では組み込み文化の形式（postcard)を**という使い分けも、境界の設計として読み応えがあります。

## よくある誤解

- **「RTICは古い、Embassyが新しい」** — どちらも現役で開発が続くフレームワークで、思想が違うだけです。応答時間の解析可能性を最優先する現場では、今もRTICが第一候補になります
- **「1つのプロジェクトではフレームワークを統一すべき」** — この製品は基板ごとに事情（HALのasync対応、仕事の性質）が違い、それぞれに合う道具を選んでいます。共有すべきは実行環境ではなく**メッセージ定義（intra-comms）**だった、という切り分けに注目してください
- **「RTICのlockはEmbassyのMutexと同じ」** — 見た目は似ていますが別物です。RTICのlockはawaitせず、割り込み優先度の操作で短時間だけ排他します。EmbassyのMutexのlockはawaitし、待っている間ほかのtaskが走ります

## 設計を考える

1. 非常停止ボタンの監視は、割り込み（ボタンのエッジでtask起動）ではなくidleループでのポーリングで書かれています。この設計の弱点と、それでも実用上成立している理由を考えてください。

<details>
<summary>考え方の例</summary>

弱点は、idleが何かで長時間止まるとボタン検出も止まることです。割り込みなら確実に検出できます。一方このidleはネットワーク処理とボタン監視だけの短いループを高速で回り続けるので、検出遅れは実用上ごくわずかです。また停止指令は毎回の無線ポーリングに乗るため、多少遅れても次の送信周期で届きます。「理論上最強の書き方」より「単純で十分速い書き方」を選ぶのも実戦の判断です。ただし自分で設計するなら、安全装置は割り込み駆動にする判断も十分あり得ます。

</details>

2. この基地局をESP32-C6＋Embassyで作り直すとしたら、「AIからのUDP受信」「16台のポーリング送受信」「非常停止ボタン」をどんなtask構成にしますか。

<details>
<summary>考え方の例</summary>

一例: (1) embassy-netでUDPを受け、指令をChannelへ流すtask、(2) Channelから取り出してロボット別バッファを更新し、無線を一巡させるポーリングtask、(3) ボタンをwait_for_low().awaitで待ち、Watch（またはSignal）で「停止中」を全taskへ知らせるtask。RTICで「優先度2のtask＋sharedのlock」だった構造が、Embassyでは「taskとChannel/Signalの配線」に写像されます。どちらでも表現できること、そして表現の道具が違うことが分かれば、このページの目的は達成です。

</details>

## まとめ

- 基地局はEthernet（smoltcp＋DHCP＋UDP＋protobuf）とSX1280無線を橋渡しし、最大16台を順番にポーリングして応答のRTTまで実測する。物理的な非常停止ボタンも持つ
- RTICは「優先度付き割り込み＋中央宣言のリソース＋lock」、Embassyは「協調的task＋awaitとデータフロー」。優劣ではなく、HALの事情と仕事の性質で選ばれる
- prost（protobuf）はヒープを要するため、embedded_allocでアロケータを持ち込んで動かしている。PCとの境界はprotobuf、マイコン同士はpostcardという使い分けにも設計がある

## 次のページ

基地局とロボットの間の2.4GHz無線リンクへ進みます。SX1280というチップの素性、SSLルールが無線に課す制約、そして「50ミリ秒受信できなければすべてゼロにする」——この応用編で最初のフェイルセーフの実物を読みます。

[4. 無線リンク — 50ミリ秒で止まる設計](/embassy-esp32-c6/robot/04-radio/)

---

前: [2. システム概念図 — 4枚の基板と1本のボール](/embassy-esp32-c6/robot/02-system/) | 次: [4. 無線リンク — 50ミリ秒で止まる設計](/embassy-esp32-c6/robot/04-radio/)
