# ESP32-C6 × Rust × Embassy 教科書サイト制作 
あなたは、次の4つの役割を兼任するリード担当者です。 * 組み込みRustエンジニア 
* ESP32-C6およびEmbassyの技術調査担当 * 中学生向け教材の編集者 * 
静的ドキュメントサイトの実装者 
必要に応じて複数のサブエージェントを並列に使ってください。 
このタスクは約5時間で区切ります。途中で私に細かい確認を求めず、合理的な判断を行い、その判断を記録してください。 
ただし、分からない仕様を想像で埋めることは禁止します。 --- # 1. 最終目的 
ESP32-C6を使って、Rustの基礎から組み込みRust、Embassy、周辺機器、無線通信、低消費電力設計まで学べる日本語の教科書サイトを作ってください。 
想定読者は次のような人です。 * ArduinoでLチカ程度は経験したことがある * 
C、C++、Rustはほとんど分からない * 中学生でも読める説明を必要としている * 
最終的にはArduinoより構造化されたプログラムを書きたい * 
非同期処理や型を使って、壊れにくい組み込みソフトウェアを作りたい 
この教材は単なるAPI一覧ではなく、 > 
Arduinoでは一つのloop関数に全部書いていた人が、RustとEmbassyを使って、複数の機能を安全に分割できるようになる 
ことをゴールにしてください。 --- # 2. 制作するもの 
以下を実際にリポジトリ内へ作成してください。 ## 必須成果物 1. 
ビルド可能な教科書サイト 2. 100〜200ページのカリキュラム 3. 
初期目標として120ページ分の教材ファイル 4. 教材で使用するRustサンプルコード 
5. サンプルコードをビルド検証する仕組み 6. 
使用ライブラリと対応状況の調査資料 7. 執筆ルール 8. 用語集 9. 最終作業報告書 
サイトは既存の構成がなければ、Astro 
Starlightなど、次を満たす静的ドキュメント基盤を採用してください。 * 
MarkdownまたはMDXで教材を書ける * サイドバーを章ごとに整理できる * 
コード表示が読みやすい * 全文検索できる * スマートフォンでも読める * 
Mermaidなどで図を表示できる * 静的サイトとして公開しやすい 
デザインに時間をかけすぎないでください。教材の内容、コードの正確性、ナビゲーションを優先します。 
--- # 3. 5時間の時間配分 おおむね次の順序で進めてください。 ## 
0〜30分：技術調査 * 現在のESP向けRust環境を調べる * 
ESP32-C6で利用できる機能を確認する * 
使用するRustツールチェーンとクレートを決める * 対応状況表を作る * 
公式サンプルを確認する * バージョンを固定する ## 30〜60分：教材設計 * 
全120ページのカリキュラムを設計する * 章の依存関係を整理する * 
ページテンプレートを作る * 完成基準を定義する * 
サンプルプロジェクトの構造を決める ## 60〜210分：サイトと教材の制作 * 
サイトを構築する * 全120ページのファイルを作る * 
サブエージェントを使って章単位で並列執筆する * 重要ページから完成させる * 
サンプルコードを実装する ## 210〜270分：検証 * サイトをビルドする * 
サンプルコードをビルドする * リンク切れを確認する * 重複説明を探す * 
誤ったESP32情報がないか確認する * 初心者には難しすぎる説明を修正する ## 
270〜300分：最終監査 * 完成ページ数を集計する * 
コンパイル済みコードを集計する * 未検証項目を明示する * 
次に執筆すべきページを優先順位順に整理する * FINAL_REPORT.mdを作る --- # 4. 
作業の優先順位 時間が不足した場合、次の順序を守ってください。 ## 
P0：必ず完成させる * ビルド可能なサイト * 120ページ分のカリキュラム * 
120ページ分の教材ファイル * 全ページの学習目標とアウトライン * 
技術対応状況表 * バージョン固定 * 執筆ルール * 用語集 * 最終報告書 ## 
P1：最低24ページを完全原稿にする 
少なくとも次の分野から、それぞれ完成度の高い代表ページを作ってください。 * 
Rustの基礎 * 所有権と借用 * struct、enum、match * traitとimpl * C++との違い 
* no_std * HAL * GPIO * UART * I2C * SPI * TWAI * Embassyのtask * Timer * 
Channel * MutexまたはSignal * Wi-Fi * BLE * ESP-NOW * sleep * エラー処理 * 
ファイル分割 * デバッグ * 最終プロジェクト ## 
P2：可能な限り全ページを完成させる 
残り時間で、完成原稿のページを増やしてください。 
ただし、薄い文章でページ数だけを稼がないでください。 
未完成ページを完成扱いしてはいけません。 --- # 5. 技術方針 
教材の中心は次の構成にしてください。 * 対象：ESP32-C6 * 
基準ボード：ESP32-C6-DevKitC-1 * 言語：Rust * 実行環境：no_std * 
HAL：現在公式に推奨されているesp-rs系HAL * 非同期実行：Embassy * 
ネットワーク：現在のesp-rs公式構成 * 
書き込み・モニタ：現在公式に推奨されるツール 
クレート名やAPI名を推測しないでください。 
esp-hal、esp-hal-embassy、esp-wifi、esp-radioなどは移行や再編が発生している可能性があります。現在の公式リポジトリ、リリース、サンプルコードを調べ、実際に組み合わせられる構成を選んでください。 
教材内で複数世代のAPIを混在させないでください。 ## ESP-IDFとの関係 
RustでESP32を扱う方法として、少なくとも次の2系統が存在することを説明してください。 
* ESP-IDFを利用するstd側の構成 * esp-halとEmbassyを利用するno_std側の構成 
ただし、本教材のコードは原則としてno_std＋HAL＋Embassyへ統一してください。 
ESP-IDF版コードとno_std版コードを同一ページで無秩序に混ぜないでください。 
--- # 6. 情報源 次の順序で情報を確認してください。 1. 
Espressif公式のESP32-C6データシート 2. ESP32-C6 Technical Reference Manual 
3. Espressif公式ハードウェア設計資料 4. Espressif公式ESP-IDFドキュメント 5. 
esp-rs公式ドキュメント 6. esp-rs公式リポジトリと公式examples 7. Embassy 
BookとEmbassy公式API資料 8. Rust公式The Rust Programming Language 9. 
Embedded Rust Book 10. embedded-hal公式資料 
個人ブログは、調査の入口としてのみ利用可能です。 
個人ブログの内容だけを根拠に、ピン番号、API、対応機能、電力特性、通信仕様を書かないでください。 
公式資料間で内容が異なる場合は、以下を記録してください。 * 
どの資料が異なるか * どちらを採用したか * 採用理由 * 確認日 * 使用バージョン 
公式資料の文章を大量にコピーまたは翻訳してはいけません。教材本文は独自に書き、参照元を示してください。 
--- # 7. 最初に作る技術対応状況表 
`docs/project/support-matrix.md`を作成してください。 
最低限、次の列を持たせます。 | 分野 | ESP32-C6ハード対応 | Rust HAL対応 | 
非同期対応 | ビルド確認 | 実機確認 | 教材での扱い | 注意点 | | -- | 
------------- | ---------- | ----- | ----- | ---- | ------ | --- | 
対象機能は最低限、次を含めてください。 * GPIO入力 * GPIO出力 * GPIO割り込み 
* Timer * Watchdog * PWM * ADC * UART * I2C * SPI * DMA * TWAI * USB 
Serial/JTAG * Wi-Fi Station * Wi-Fi Access Point * TCP * UDP * HTTP * MQTT * 
BLE Advertising * BLE Peripheral * BLE Central * ESP-NOW * IEEE 802.15.4 * 
Thread * Zigbee * Light Sleep * Deep Sleep * Wake-up source * Flash * 
不揮発ストレージ * OTA * 乱数 * 暗号関連機能 
次の状態を明確に区別してください。 * 公式に安定対応 * unstable API * 実験的 
* ビルドのみ確認 * 実機確認済み * 概念説明のみ * 現時点では教材対象外 
「ESP32-C6がハードウェアとして対応している」と「Rustの現在のライブラリで実用的に扱える」を混同しないでください。 
--- # 8. ESP32-C6固有の注意事項 以下を必ず確認し、教材へ反映してください。 
## Bluetooth ESP32-C6で扱うのはBluetooth Low Energyです。 Bluetooth 
Classicの教材やコードをESP32-C6向けとして掲載しないでください。 
タイトルや本文でも、単に「Bluetooth」と書かず、必要に応じて「Bluetooth Low 
Energy」または「BLE」と明記してください。 ## CAN 
ESP32-C6のCAN相当機能はTWAIとして説明してください。 
次を明確に分けて説明してください。 * TWAIコントローラ * CAN/TWAIトランシーバ 
* CANバス配線 * 終端抵抗 * ビットレート * フレーム * ACK * エラー状態 * 
バスオフ ESP32-C6単体のピンをCAN_H、CAN_Lへ直接接続する説明は禁止します。 
外付けトランシーバが必要であることを、配線図とともに説明してください。 ## 
無線 次を混同しないでください。 * Wi-Fi * TCP/IP * UDP * HTTP * MQTT * 
ESP-NOW * BLE * IEEE 802.15.4 * Thread * Zigbee 
「Wi-FiがあるからHTTPが使える」のように、物理通信とアプリケーションプロトコルを一段で説明しないでください。 
## 低消費電力 単にsleep関数を呼べば省電力になる、という説明は禁止します。 
最低限、次を扱ってください。 * Active * Modem Sleep * Light Sleep * Deep 
Sleep * CPUが停止する範囲 * RAM保持 * 無線通信との関係 * 復帰要因 * 
起動し直しとの違い * GPIO状態 * RTC領域 * 平均電流と瞬間電流 * 測定方法 * 
開発ボード上のUSB-UARTやLEDによる消費電力 
実測していない電流値を断定しないでください。 --- # 9. 
120ページのカリキュラム 
原則として12部×10ページの120ページ構成にしてください。 
調査結果に応じて多少変更して構いませんが、変更理由を記録してください。 ## 
第1部：ESP32-C6と開発環境 1. この教材で作れるもの 2. 
ArduinoからRustへ移る理由 3. ESP32-C6とは何か 4. 
マイコンと普通のパソコンの違い 5. 必要な部品 6. 電圧と電流の最低限 7. 
開発環境の構築 8. Rustプロジェクトの作成 9. 書き込みとシリアル表示 10. 
最初のLチカ ## 第2部：Rustの最初の一歩 * 変数 * mut * 数値型 * bool * 関数 * 
if * loop * while * for * 配列とタプル ## 第3部：Rustらしいデータの扱い * 
struct * enum * match * Option * Result * メソッド * impl * 所有権 * 借用 * 
ライフタイムの直感的説明 ## 第4部：大きなプログラムの作り方 * module * 
ファイル分割 * pub * crate * trait * generics * associated typeの入門 * 
エラー設計 * 状態機械 * C++との設計比較 ## 第5部：組み込みRustの基礎 * 
no_std * main以前に起きること * メモリ * stack * heap * static * panic * PAC 
* HAL * embedded-hal ## 第6部：GPIO、割り込み、時間 * GPIO出力 * GPIO入力 * 
Pull-up/Pull-down * ボタン * チャタリング * GPIO割り込み * Timer * Ticker * 
Timeout * Watchdog ## 第7部：アナログと波形制御 * ADC * 分圧 * センサ値 * 
PWM * LEDの明るさ * サーボ制御 * 周波数 * duty比 * ハードウェアタイマー * 
小さな制御プロジェクト ## 第8部：UART、I2C、SPI、TWAI * UART基礎 * 
UART非同期受信 * I2C基礎 * I2Cセンサ * I2Cエラー * SPI基礎 * SPIデバイス * 
バス共有 * TWAI基礎 * TWAI通信 ## 第9部：Embassyによる非同期処理 * 
同期処理と非同期処理 * asyncとawait * Futureの直感的説明 * task * Spawner * 
Timer * select * join * Channel、Signal、Mutex * 
キャンセル、詰まり、優先順位 ## 第10部：Wi-Fiとネットワーク * Wi-Fiの基礎 * 
Station * Access Point * IPアドレス * DHCP * TCP * UDP * DNS * HTTP * 
MQTTまたは小型Webサーバー ## 第11部：BLE、ESP-NOW、802.15.4 * BLEの基礎 * 
Advertising * ServiceとCharacteristic * Peripheral * Central * 
BLEでボタン状態を送る * ESP-NOWの基礎 * ESP-NOWの再送と重複排除 * IEEE 
802.15.4 * Thread/ZigbeeとRust対応状況 ## 第12部：実用設計と最終プロジェクト 
* Light Sleep * Deep Sleep * Wake-up * 消費電力測定 * Flashと設定保存 * 
ログとデバッグ * エラーからの復旧 * プロジェクトの分割 * テストと保守 * 
最終プロジェクト --- # 10. 最終プロジェクト 
最終プロジェクトでは、ESP32-C6を使った無線ボタン端末を作ってください。 ## 
要求仕様 送信側は次の動作を行います。 * ボタン状態を監視する * 
ボタンが押されたら即座にイベントを送信する * 
500ミリ秒ごとに生存確認と現在状態を送信する * パケットへ連番を付ける * 
受信側で重複を判定できる * 通信失敗を想定する * 必要に応じて再送する * 
一定時間応答がなければ異常状態にする * 複数taskへ責務を分割する * 
ChannelまたはSignalでtask間通信する * sleep可能な構造を検討する 
通信方式は、ESP-NOW、BLE、Wi-Fiの中から適切なものを比較し、教材の主実装を一つ選んでください。 
比較項目は次を含めます。 * 通信遅延 * 接続確立 * 消費電力 * 到達距離 * 再送 
* ACK * 複数台 * スマートフォンとの接続 * ルーターの必要性 * 実装の難しさ * 
Rustライブラリの成熟度 最終的に以下のような構成へ分割してください。 ```text 
src/ ├── main.rs ├── app.rs ├── button.rs ├── protocol.rs ├── 
radio.rs ├── heartbeat.rs ├── power.rs ├── error.rs └── 
config.rs ``` 
ただファイルを分けるのではなく、それぞれの責務と依存方向を説明してください。 
--- # 11. 各ページの形式 1ページは約15分で完了できる内容にしてください。 
目安は次の通りです。 * 読む：7〜9分 * コードを動かす：3〜5分 * 
演習と確認：2〜3分 
難しいテーマは一つの長いページへ詰め込まず、複数ページへ分割してください。 
各ページは原則として次のテンプレートを使います。 ```markdown --- title: 
description: part: lesson: difficulty: estimated_minutes: 15 prerequisites: 
hardware: status: verified_with: last_verified: sources: --- # タイトル ## 
このページでできるようになること - 具体的な学習目標 - 具体的な学習目標 ## 
先に結論 このページで最も重要なことを3〜5文で説明する。 ## 身近なたとえ 
中学生でも理解できる例を使う。 ## 仕組み 図や小さなコードを使って説明する。 
## Arduinoではどう書くか 必要な場合のみ掲載する。 ## 
RustとEmbassyではどう書くか 完全なコードまたは動く最小コードを掲載する。 ## 
コードを一行ずつ読む 重要な行だけを説明する。 ## 配線 
必要な場合はピン、電圧、抵抗、外付け部品を示す。 ## 実行方法 
コマンドと期待される結果を示す。 ## よくある失敗 最低2件を書く。 ## 
やってみよう 5分以内でできる変更課題を書く。 ## 確認問題 2〜3問を書く。 ## 
まとめ 3項目以内でまとめる。 ## 次のページ 次に学ぶ理由を説明する。 ``` --- 
# 12. 中学生向け文章の規則 以下を必ず守ってください。 * 一文を短くする * 
一つの段落で一つの話だけをする * 専門用語は最初に意味を説明する * 
略語は正式名称を一度書く * 知っている前提で話を進めない * 
「簡単です」で説明を省略しない * 比喩の直後に、実際の技術との違いも説明する 
* コードを突然大量に見せない * まず目的を説明してからコードを見せる * 
エラーが起きる理由も説明する * 電子回路の危険な接続を軽く扱わない * 
Rustのコンパイラエラーを敵として扱わない * 
「なぜこの制約があるのか」を説明する * 子どもっぽすぎる口調にはしない * 
です・ます調で統一する 
「箱」「貸し借り」「鍵」「順番待ち」などの比喩は利用できます。 
ただし、比喩だけで説明を終わらせず、最後に必ず正式な用語へ戻してください。 
--- # 13. Rustで必ず扱う内容 最低限、次を扱ってください。 ## 基本文法 * let 
* mut * const * static * 基本型 * 配列 * slice * tuple * 関数 * if * loop * 
while * for * match ## データ設計 * struct * enum * Option * Result * impl * 
method * associated function * trait * generic * associated type * newtype 
## 所有権 * move * copy * clone * borrow * mutable borrow * lifetime * 
static lifetime ライフタイムは記号の暗記から始めず、 * 誰がデータを持つのか 
* いつまで存在するのか * 誰が一時的に使うのか から説明してください。 ## 
組み込み特有 * no_std * heapを使わない設計 * staticな領域 * 固定長バッファ * 
heapless * 割り込み * volatile * unsafeの境界 * panic * ログ * feature flag 
unsafeを「危険だから使ってはいけない」とだけ説明せず、安全な抽象化の内側で必要になる理由を説明してください。 
--- # 14. C++との比較 
Rustを優れているように見せるためだけの比較は禁止します。 
次のような実務上の違いを、小さな対比コードで説明してください。 * 
classとstruct＋impl * interface、抽象クラスとtrait * constructorとnew関数 * 
destructorとDrop * pointerとreference * nullとOption * exceptionとResult * 
templateとgeneric * virtual dispatchとdyn Trait * enumとRustのデータ付きenum 
* RAII * move semantics * ownership * header/source分割とmodule * macro * 
compile-time error * undefined behavior * thread safety * static 
initialization 比較では必ず次を示してください。 1. C++ではどう考えるか 2. 
Rustではどう考えるか 3. Rust側の制約が必要な理由 4. Rustでも解決できない問題 
5. 組み込み開発でどちらが扱いやすい場面か --- # 15. Embassyで必ず扱う内容 
最低限、次を扱ってください。 * executor * task * Spawner * async fn * await 
* Future * Poll * Wakerの直感的説明 * Timer * Ticker * Timeout * select * 
join * Channel * Signal * Mutex * PubSub * task間の所有権 * static領域 * 
割り込みからの起床 * 協調的な実行 * taskを止める処理 * cancellation * 
backpressure * バッファ満杯 * 優先順位 * watchdogとの関係 
次の誤解を明確に解消してください。 * asyncは自動的に別コアで動くわけではない 
* taskはOSスレッドと同じではない * await中にCPUが必ず動き続けるわけではない 
* asyncにすればすべて高速になるわけではない * 
Mutexを使えば設計が自動的に安全になるわけではない * 
Channelを増やせば責務分割になるわけではない * 
長いCPU処理は他taskを止める可能性がある --- # 16. 
ファイル分割とアーキテクチャ 
単に`main.rs`を複数ファイルへ切るだけの説明にしないでください。 
次の順序で説明してください。 1. 一つのmain.rsで小さな動作を作る 2. 
ハードウェア操作を分ける 3. 通信データを分ける 4. 状態を分ける 5. 
taskを分ける 6. task間通信を設計する 7. エラー型を整理する 8. 
設定値を整理する 9. テスト可能な純粋ロジックを分ける 
次を悪い例として扱ってください。 * 
すべてのperipheralを一つの巨大structに入れる * 
どこからでも共有Mutexへアクセスする * taskが互いの内部状態を直接変更する * 
main.rsが初期化、通信、状態管理、エラー復旧を全部行う * 
Channelをグローバル変数として無計画に増やす * 
型を使わずu8やboolだけで状態を表現する * エラーをすべてunwrapする 
状態機械、メッセージ、責務、依存方向を図で説明してください。 --- # 17. 
ハードウェア教材 必要部品を最小構成と追加構成に分けてください。 ## 最小構成 
* ESP32-C6-DevKitC-1 * USBケーブル * ブレッドボード * LED * 抵抗 * 
タクトスイッチ * ジャンパ線 ## 追加構成の候補 * I2C温湿度センサ * 
SPIディスプレイまたはSPIセンサ * 可変抵抗 * サーボ * 2台目のESP32-C6 * 
TWAIトランシーバ * TWAI終端抵抗 * 電流測定器 
使用する部品は入手性と教材全体での再利用性を優先してください。 
センサを章ごとに無計画に増やさないでください。 
配線例には最低限、次を示してください。 * ボード上のピン名 * GPIO番号 * 
電源電圧 * GND * 抵抗 * 外付けトランシーバ * 注意事項 
ピン番号は対象ボードの公式回路図または公式ボード資料で確認してください。 --- 
# 18. コードの検証 教材コードには次の検証状態を付けてください。 * 
`concept-only` * `syntax-reviewed` * `cargo-check-passed` * 
`hardware-tested` 
実機を利用できない場合、実機確認済みとは書かないでください。 ## 必須検証 
可能な範囲で次を自動化してください。 * cargo fmt --check * cargo check * 
cargo clippy * サイトのビルド * Markdownリンク確認 * frontmatter確認 * 
重複タイトル確認 * 空ページ確認 * TODO確認 * コードブロックの対応確認 
サンプルコードは`examples/`配下へ整理してください。 例： ```text examples/ 
├── 01-blinky/ ├── 02-button/ ├── 03-uart/ ├── 04-i2c/ ├── 
05-spi/ ├── 06-embassy-tasks/ ├── 07-channel/ ├── 08-wifi/ ├── 
09-ble/ ├── 10-esp-now/ ├── 11-twai/ ├── 12-sleep/ └── 
final-wireless-button/ ``` 
教材内の断片コードと、実際にビルドする完全コードが食い違わないようにしてください。 
できれば教材側から完全コードの該当箇所を参照する構成にしてください。 --- # 
19. バージョン管理 `docs/project/versions.md`を作ってください。 
最低限、次を記録してください。 * Rust toolchain * target * esp-hal系クレート 
* Embassy系クレート * 無線系クレート * embedded-hal * 
espflashまたは使用する書き込みツール * サイト生成ツール * Node.js * 
動作確認OS * 対象ボード * 確認日 
Cargo.tomlでバージョン範囲を広くしすぎないでください。 unstable 
APIを利用する場合は、なぜ必要なのか、更新時に何が壊れ得るのかを書いてください。 
--- # 20. サイト内の補助コンテンツ 次を作成してください。 ## 用語集 
最低限、以下を収録します。 * MCU * SoC * GPIO * peripheral * register * 
interrupt * polling * driver * PAC * HAL * trait * ownership * borrow * 
lifetime * async * await * Future * task * executor * Channel * Mutex * UART 
* I2C * SPI * TWAI * Wi-Fi * BLE * ESP-NOW * TCP * UDP * sleep ## 
Arduinoからの対応表 例： | Arduino | Rust／Embassy | | --------------- | 
---------------------- | | setup | 初期化処理 | | loop | task内のloop | | 
delay | Timer::after系 | | digitalWrite | GPIO Output | | digitalRead | GPIO 
Input | | Serial | UARTまたはログ | | Wire | I2C | | SPI | SPI HAL | | 
attachInterrupt | GPIO async waitまたは割り込み | | millis | 
Instant、Duration | | global variable | 所有者を決めた状態またはstatic | | 
callback | async task、Channel、関数 | 
単純な名前の置き換えではなく、設計思想の違いも説明してください。 ## 
トラブルシューティング 最低限、次を扱ってください。 * ボードが認識されない * 
書き込めない * シリアル出力が見えない * targetが違う * feature flagが違う * 
バージョンが合わない * 所有権エラー * static lifetimeエラー * 
taskへperipheralを渡せない * Wi-Fiへ接続できない * BLEが見つからない * I2C 
ACKが返らない * SPIモードが違う * TWAIがACKされない * Deep 
Sleepから想定通り復帰しない --- # 21. 禁止事項 以下は禁止します。 * 
公式資料を確認せずAPIを創作する * 古いブログのコードをそのまま使う * 
ESP32、ESP32-C3、ESP32-C6の機能を混同する * Bluetooth 
ClassicをESP32-C6向けとして説明する * TWAIピンをCAN_H、CAN_Lへ直接接続する * 
全コードでunwrapを使用する * unsafeを説明せず多用する * 
実測していない電力値を断定する * 実機未確認なのに「動作確認済み」と書く * 
同じ説明を言い換えてページ数を増やす * 目次だけ作って完成と報告する * 
サイトの見た目だけを作り込む * 初心者向けという理由で技術的な嘘を書く * 
Rustならメモリバグがすべてなくなると説明する * 
asyncなら自動的に高速になると説明する * 
Arduinoを一方的に低品質な環境として扱う * 参考資料の文章を大量にコピーする * 
生成したコードを一度もビルドせず完成扱いする --- # 22. 完成条件 
最低限、次を満たしてください。 * サイトがローカルでビルドできる * 
120ページがナビゲーションに登録されている * 
全120ページに固有の学習目標がある * 全120ページに15分以内の演習がある * 
全120ページに前提ページと次ページが設定されている * 
少なくとも24ページが完全原稿になっている * 
少なくとも12個のサンプルプロジェクトがある * 
ビルド可能なサンプル数が報告されている * 技術対応状況表がある * 
バージョン表がある * 用語集がある * Arduino対応表がある * 
最終プロジェクトの設計とコードがある * 未検証部分が明示されている * 
サイト内に明らかな空ページがない * 壊れたリンクがない * 
完成ページと未完成ページを区別できる --- # 23. 最終報告 
最後に`FINAL_REPORT.md`を作成してください。 次を必ず含めます。 ```markdown # 
最終報告 ## 制作結果 - 総ページ数： - 完全原稿： - 構成・下書き： - 未完成： 
- サンプル数： - cargo check成功： - 実機確認済み： - サイトビルド： - 
リンクチェック： ## 採用した技術構成 ## 固定したバージョン ## 
実行した検証コマンド ## 実際に確認できた機能 ## ビルドのみ確認した機能 ## 
調査のみで実装できなかった機能 ## 技術的に不安定な機能 ## 教材上の重要な判断 
## 既知の問題 ## 次に完成させるべきページ ## 継続作業の優先順位 ``` 
数字をごまかさないでください。 
「120ページ作成」と書く場合は、完全原稿、下書き、骨格のみを分けて集計してください。 
--- # 24. 作業開始時の行動 
最初に長い説明を返すのではなく、リポジトリを確認して作業を開始してください。 
開始後、次のファイルを早い段階で作ってください。 ```text docs/project/ 
├── curriculum.md ├── support-matrix.md ├── versions.md ├── 
writing-guide.md ├── hardware-kit.md ├── source-policy.md └── 
progress.md ``` 
`progress.md`には、章ごとの状態を次のように記録してください。 ```text 
planned outlined drafted reviewed cargo-check-passed hardware-tested ``` 
サブエージェントへ章を割り振る場合も、全員が同じwriting-guide.mdとversions.mdを使用するようにしてください。 
章ごとに異なるクレートバージョンや用語が使われないよう、最後に統合レビューを行ってください。 
最終目標はページ数ではありません。 読者が、 1. Rustの基本を理解する 2. 
ESP32-C6で周辺機器を動かす 3. Embassyで複数処理を分割する 4. 
無線通信を実装する 5. エラーや通信断を考慮する 6. 
Arduinoの一枚岩のloopから卒業する 
ところまで、順番に学べる教材を作ることです。
