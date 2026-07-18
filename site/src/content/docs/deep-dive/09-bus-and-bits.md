---
title: "9. 自作バスと専用命令 — PARLIO・Dedicated GPIO・SDM"
description: 複数のGPIOを束ねて自作データバスにするPARLIO、GPIO操作のためにCPU命令セットまで拡張したDedicated GPIO、デジタルピンから疑似アナログを作るSDMの3点セットを、Rust対応状況付きで学びます。
lesson: 9
difficulty: advanced
estimated_minutes: 20
prerequisites:
  - deep-dive/06-ledc-dma
  - deep-dive/02-gpio-matrix
status: complete
code_status: concept-only
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/peripherals/parlio.html
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/peripherals/dedic_gpio.html
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/peripherals/sdm.html
  - https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/peripherals/gpio.html
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/esp_hal/parl_io/index.html
  - https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/esp_hal/gpio/dedicated/index.html
  - https://documentation.espressif.com/esp32-c6_technical_reference_manual_en.pdf
---

> **Rustからの現在地**: PARLIOは**unstableで試せる**（esp-hal 1.1.1の`parl_io`、DMA必須・async対応）。Dedicated GPIOは**unstableで試せる**（`gpio::dedicated`にフルドライバ）。SDMとGPIOグリッチフィルタは**概念のみ（ESP-IDF）**です。

## このページでできるようになること

- PARLIOが「GPIOを束ねた自作パラレルバス」であること、何本束ねると何が起きるかを説明できる
- Dedicated GPIOのために**CPUの命令セットそのものが拡張されている**ことの意味を説明できる
- SDM（シグマデルタ変調）でデジタルピンから疑似アナログ電圧を作る原理を説明できる
- 3つの機能それぞれの「Rustからの現在地」を言える

## 先に結論

この図鑑の最後の技術ページは、GPIOの常識を壊す3点セットです。①PARLIO（Parallel IO）は複数のGPIOを束ねて、1クロックで複数bitを送るパラレルバスを作る回路。DMA（6ページ）と組んで、CPUに触らせずデータを流します。②Dedicated GPIOは、GPIOの読み書きのためにRISC-VのCSR（Control and Status Register、CPU内部の特殊レジスタ）命令まで用意された最速のビット操作。**ペリフェラルではなくCPU自体が拡張されている**のがキモです。③SDM（Sigma-Delta Modulation）は、DACを持たないC6で、デジタルピンの高速なON/OFF密度と外付けRCフィルタから疑似アナログ電圧を作ります。Rustからの現在地は、PARLIOとDedicated GPIOはunstableで試せる、SDMは概念のみ（ESP-IDF）です。

## 身近なたとえ

3つとも「道路」でたとえられます。PARLIOは1車線の道路を4車線に拡張する話（1回の信号で4台=4bitが通る）。Dedicated GPIOは、料金所の係員（通常のGPIOレジスタ経由）を通らず、運転席から直接ゲートを開ける専用リモコン（CPU命令）を持つ話。SDMは、車の通過密度で「交通量」というアナログな量を表す話です。

たとえの限界を言っておくと、PARLIOの「車線」は同期して動く1本のクロックに従うデータ線で、Dedicated GPIOの「リモコン」はcsrrsi/csrrciといった実在するRISC-V命令、SDMの「密度」は電圧の時間平均です。以下で正確に見ます。

## ① PARLIO — GPIOを束ねて自作データバス

UARTもSPIも、データ線は基本1本で、8bitを送るには8回に分けて送ります（シリアル通信、第8部）。PARLIOはこれを並列に戻します。GPIOを1/2/4/8/16本束ねて（16bit幅は半二重のみ）、クロックに同期して1クロックで束の本数ぶんのbitを出し入れします。クロックは最大40MHzなので、8bit幅なら理論上40Mバイト/秒がGPIOから流れ出る計算です。

この速度はCPUのループでは供給できません。だからPARLIOは設計からしてDMA前提です。「メモリのこのバッファを、この8本のピンから流せ」という、6ページの思想そのものです。

代表的な使い道が、パラレル入力の表示デバイスです。ESP-IDFにはHUB75方式のLEDマトリクスパネル（駅の電光掲示板のような多色パネル）をPARLIOで駆動する公式exampleがあります。ロジックアナライザ的にパラレル入力を取り込む方向（RX)もあります。

**Rustからの現在地: unstableで試せる** — esp-hal 1.1.1の`parl_io`モジュール（unstable）。`ParlIo::new(peripherals.PARL_IO, dma_channel)`とDMAチャンネル必須の設計で、`TxSixteenBits`などの幅指定型があり、async対応です。教材のexampleはありませんが、docsのコード例から始められます。

## ② Dedicated GPIO — CPU命令セットがGPIOのために拡張されている

普通のGPIO操作は、CPUがバスを経由してGPIO周辺回路のレジスタへ書き込む間接的な操作です。十分速いのですが、1回の操作に数クロックかかり、タイミングも周辺バスの都合に左右されます。

ESP32-C6はここで驚きの手を打ちました。**CPUコア自体にGPIO専用の経路を増設し、RISC-VのCSR命令でピンを直接叩けるようにした**のです（TRM §1.14）。最大8チャンネルを割り当てられ、うち下位4チャンネルは`csrrsi`/`csrrci`という即値1命令で操作できる高速パスです。命令の実行=ピンの変化なので、クロック単位でタイミングが読めます。ソフトウェアで独自の通信プロトコルを1bitずつ正確に作る（ビットバンギング）ときの最終兵器です。

「GPIOが速いマイコン」は珍しくありませんが、「GPIOのために命令セットを拡張したマイコン」となると話が違います。周辺回路への分業どころか、CPUそのものをGPIO向けに改造してあるわけです。

**Rustからの現在地: unstableで試せる** — esp-hal 1.1.1に`gpio::dedicated`モジュール（unstable）としてフルドライバがあります。`DedicatedGpioInput`/`DedicatedGpioOutput`、複数チャンネルの束（バンドル）、複数ピンを1命令で書く`write_ll`・全チャンネルを読む`read_all_ll`まで揃っています。

## ③ SDM — デジタルピンから疑似アナログ電圧

C6にはDAC（Digital to Analog Converter、数値→アナログ電圧の変換器）がありません（ハードウェア調査資料参照）。それでも「1.65Vくらいの中間電圧を出したい」ときの答えがSDM（シグマデルタ変調）です。

原理はPWMの親戚です。ピンを高速にON/OFFし、「1が出る密度」で目標値を表現します。2次のシグマデルタ変調器（C6は4チャンネル）がパルス列を作り、そのままでは0か3.3Vかの点滅なので、外付けの抵抗とコンデンサ（RCローパスフィルタ）で平均化すると、密度に比例したなめらかな電圧になります。LEDCのPWMより高い周波数成分にノイズを押しやれるのが変調器の賢いところで、音声のような信号にも応用されます（PDM: Pulse Density Modulationと同じ系統）。

**Rustからの現在地: 概念のみ（ESP-IDF）** — ハードにあり、ESP-IDFには`sdm`ドライバがありますが、esp-hal 1.1.1にSDMモジュールはありません。6ページのADC連続+DMAに続く「ハード対応とライブラリ対応は別物」の実例その2です。

## 小ネタ: GPIOグリッチフィルタ

3点セットのおまけに、地味ながら思想がよく出ている機能を1つ。C6のGPIOには、規定より短いパルス（グリッチ=ノイズ）を**CPUに届く前に**捨てるハードウェアフィルタがあります（ピン固定のフィルタ+自由に割り当てられるフレキシブルフィルタ8本)。Arduinoで「チャタリングはdelayで待って読み直す」と覚えた人には、「ノイズ対策=ソフトの待ち時間」という常識を壊す機能です。

これもRustからの現在地は**概念のみ（ESP-IDF）**。esp-hal 1.1.1に汎用GPIOグリッチフィルタのAPIはありません。ただし[4ページのPCNT](/embassy-esp32-c6/deep-dive/04-pcnt/)で使ったユニット内蔵フィルタは同じ思想のもので、こちらはRustで設定済みです。

## よくある失敗

- **PARLIOを「GPIOを速く叩くAPI」と誤解する** — PARLIOはクロック同期+DMAのバス回路です。1本のピンを不定期にトグルしたいだけならGPIOかDedicated GPIOが適切で、PARLIOはバッファに用意した連続データを一定速度で流す用途に使います
- **SDMの出力をそのままアナログ入力に配線する** — フィルタなしのSDM出力はただの高速デジタルパルスです。RCローパスフィルタ（例: 抵抗+コンデンサ）を通して初めて疑似アナログ電圧になります
- **「esp-halに無い=C6にできない」と結論する** — SDMもグリッチフィルタもハードは持っています。逆に「TRMに載っている=今Rustで書ける」でもありません。docs（docs.espressif.com/projects/rust）とESP-IDFドキュメントの両方を確認する癖をつけましょう

## やってみよう

紙とペンで確かめましょう。PARLIOで8bit幅・クロック10MHzのバスを組んだとします。(1) 1秒間に流せるのは何バイトですか。(2) 同じ量をCPUのループ（1バイトあたり10命令、160MHz と仮定）で送るとCPU時間の何%を食いますか。ざっくりで構いません——「DMA前提」の意味が数字で見えてきます。

## 確認問題

1. PARLIOの16bit幅には、他の幅にない制限があります。何ですか。
2. Dedicated GPIOが「ただの速いGPIO」と本質的に違う点はどこですか。
3. SDM・グリッチフィルタ・ADC連続+DMAの3つに共通する「Rustからの現在地」の状況は何ですか。

<details>
<summary>答え</summary>

1. 16bit幅は半二重（送信と受信を同時にできない）のみ。
2. GPIO周辺回路のレジスタ経由ではなく、CPUの命令セット（RISC-VのCSR命令）に専用経路が組み込まれていること。下位4チャンネルはcsrrsi/csrrciの1命令で操作できる。
3. いずれも「C6のハードウェアとESP-IDFは対応しているが、esp-hal 1.1.1では未実装＝概念のみ」。ハード対応とライブラリ対応は別物という本編の教訓の実例。

</details>

## まとめ

- PARLIO=GPIOを1/2/4/8/16本束ねる自作パラレルバス。DMA前提で、esp-halでもasync+DMAで書ける（unstable）
- Dedicated GPIOはCPU命令セット拡張によるクロック精度のビット操作。esp-halにフルドライバあり（unstable）
- SDMとGPIOグリッチフィルタはハードにあるがesp-halは未実装で概念のみ（ESP-IDF）。「ハード対応とライブラリ対応を混同しない」を合言葉に

## 次のページ

図鑑もいよいよ最終ページです。まだ紹介しきれていない機能を一気に総覧し、20機能×「Rustからの現在地」の総括表で深淵の全体地図を手に入れましょう。

- 前: [8. LPコア — 地下室でもう一人が働いている](/embassy-esp32-c6/deep-dive/08-lp-core/)
- 次: [10. 図鑑の残りと、深淵の歩き方](/embassy-esp32-c6/deep-dive/10-zukan/)
