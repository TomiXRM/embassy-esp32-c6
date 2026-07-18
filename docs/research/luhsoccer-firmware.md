# luhsoccer_firmware（LuhBots）調査資料（応用編3の素材）

調査日: 2026-07-18。clone済み: scratchpad/luhsoccer_firmware

## チーム・リーグの検証済み事実（公式ソースのみ。教材の記述はこの範囲に限る）

- **RoboCup SSL**（公式ルールブック https://robocup-ssl.github.io/ssl-rules/sslrules.html ）: Div A 11台/Div B 6台、ロボットは直径0.18m×高さ0.15mの円筒に収まること、ボールはゴルフボール、**キック速度上限6.5m/s**、フィールドDiv A 12×9m。共有ビジョン（SSL-Vision、Ethernet配信）+ ssl-game-controller。**完全自律**（試合中の人間操作禁止）。Halt命令後2秒以内に停止。Stop中は1.5m/s未満。無線は周波数申告制・2キャリア切替可能であること・**Bluetooth禁止**（周波数固定不可のため）
- **チーム**: 「luhbots Soccer」— Leibniz Universität Hannover（ハノーファー）の学生チーム。クラブ設立2012年、SSL部門は2019年から
- **SSL成績（ssl.robocup.org公式結果）**: 2022 Div B **3位**（バンコク・SSL初出場）/ 2023 Div B **準優勝**（ボルドー。決勝でRobôCInに敗北。Ball Placementチャレンジ2位）/ 2024 Div A **5位タイ**（アイントホーフェン・Div A初参戦）/ 2025 不参加
- ⚠️ **SSLでの優勝歴はない**。「世界チャンピオン」の実績は同クラブの**RoboCup@Work部門（2015 Hefei・2016 Leipzig世界一）**（ハノーファー市・大学の公式プレスリリースで確認）。教材では混同せず正確に書くこと
- **Rust採用の一次情報（2023 TDP** https://ssl.robocup.org/wp-content/uploads/2023/02/2023_TDP_Luhbots.pdf **）**: 「マイコン変更に伴いファームウェアを全面書き直し。将来のマイコン更新に備えRustで記述し、embedded-halクレートでアーキテクチャ間のコード再利用を可能にする」（要旨訳）。2023年基板はRP2040×2構成、SX1280+SKY66112はTIGERs Mannheimに倣って採用、2022年の暫定構成（ESP32+NRF24L01）は無線に問題があり置換、と明記
- TDPに「Embassy」の語は**出てこない**。Embassy使用はリポジトリのCargo.tomlで確認（embassy-rp/executor 0.5/time/sync/usb）。「Embassy on RP2040」であり、C6で動くとは書かないこと
- TDP抽出テキスト: scratchpad/luhbots_2023_tdp.txt、luhbots_2024_etdp.txt、sslrules.txt

## 題材

- リポジトリ: https://github.com/luhbots/luhsoccer_firmware
- **ライセンス: MIT**（Copyright (c) 2022 soccer@luhbots.de）→ 出典明記の上で引用可
- 公開は2023/2024の2コミット（内部GitLabのスカッシュミラー）。READMEは空
- 規模: Rust約17.4k行（vendored HAL除く）。clippy pedantic+nursery、GitLab CI（clippy/カバレッジ/ベンチ）、libsに単体テストあり
- ツールチェーン: stable Rust、thumbv6m（RP2040）+ thumbv7em（ATSAM4E）

## 基板構成

| 基板 | MCU | フレームワーク | 役割 |
|---|---|---|---|
| Basestation | ATSAM4E8C (M4) | **RTIC 1.1** + atsam4-halフォーク | Ethernet(UDP/protobuf)⇄SX1280無線ブリッジ、最大16台、非常停止ボタン |
| Maincontroller | RP2040 | embassy-rp 0.1 / executor 0.5 / time 0.3 / sync 0.5 + defmt | 無線端点、電源/電池、ドリブラーPWM出力、ボールセンサ、UI(I2Cスレーブ0x42)、設定、WDT |
| Motorcontroller | RP2040 | 同Embassy + embassy-usb + panic-persist + nalgebra | 1kHz運動制御、TMC4671×4(共有SPI)、キッカーHV、USBテレメトリ |
| Dribblercontroller | RP2040 | 同Embassy（小規模） | サーボPWM入力→ESCON駆動、IRライトバリア |
| UI基板 | 不明（ui/は空） | - | メイン基板をI2Cマスタとしてポーリング |

Executor構成（RP2040 2枚）: **InterruptExecutor（高優先, SWI_IRQ_0）+ スレッドExecutor（低優先）+ motorcontrollerはcore1に専用executor**（spawn_core1）。

## トポロジ（Mermaid化の素）

intra-comms/src/lib.rsのコメントが公式: Server/Vision/GameController —protobuf→ Basestation —postcard→ Maincontroller ←postcard→ Motorcontroller

| 区間 | 物理 | プロトコル |
|---|---|---|
| AI-PC⇄Basestation | Ethernet UDP (smoltcp, DHCP) | protobuf (prost) To/FromBasestationWrapper |
| Basestation⇄ロボット0..15 | 2.4GHz SX1280 **FLRC 1.3Mb/s**、ロボット別32bit同期語、SKY66112 PA/LNA | postcardのBasestationToRobot/RobotToBasestation、TDMA的ポーリング（~5ms窓、RTT実測） |
| Main⇄Motor | UART0 **1Mbaud RTS/CTS** | **postcard+COBS+CRC16**（intra-comms::uart）。Main2Motor(Drive/Kick/Chip/ChargeHint...)/Motor2Main(MotorVelocity/CapVoltage) |
| Main→Dribbler | RCサーボ式PWM 500Hz 1-2ms | duty=速度。**20Hz未満→モータ停止フェイルセーフ** |
| Dribbler→Main | GPIO 1本 | ボール在否（IR、2kHz ADC、自動閾値校正） |
| Motor→TMC4671×4 | 共有SPI 2MHz mode3 + 個別CS (embassy_embedded_hal::shared_bus) | トルク目標/エンコーダ |
| Motor→キッカー | GPIO+PIO 10bit並列DAC（充電設定）+ サーボ（チップ切替）+ ADC（コンデンサ電圧） | - |

**CANは不使用**（UART/PWM/GPIO/I2C/SPIのみ）→ 教材では「彼らが手作りしたフレーミング+CRC+アドレスは、CAN/TWAIならハードがやってくれる」対比が使える。共有メッセージ定義クレートintra-commsは最終プロジェクトprotocol.rsの実戦版。

## Maincontrollerの11 task（IPCはlibs/syncの自作Observable<CriticalSectionRawMutex, T, 8> + Signal + Mutex、全て&'static引数で明示配線）

power_switch_task(高優先: 電源ボタン/シャットダウン) / watchdog_task(750ms開始,500ms給餌) / rf_task(SX1280送受、**RXタイムアウト50msで速度・キック・ドリブラーをゼロ+NO_RF_CONNECTION**) / motorcontroller_task+receive_task(UART橋、join3で3系統を≥1Hzキープアライブ再送、Mutex<NoopRawMutex>でTX共有) / dribbler_task(500Hz PWM+3秒スルーレート) / lightbarrier_task(1kHz+PT-3ローパス) / measure_task(100Hz電池ADC+ヒステリシス状態機械Usb/Critical/Low/Nominal/Full/Over、**Criticalで自己シャットダウン**) / ui_task / buzzer_task(PIO) / led_task(WS2812) / config_task(フラッシュ設定+SAVE_CONFIG_SIGNAL) / (+feature付きdribbler_test_task)

## Motorcontrollerの要点

- **motors_task（core1専有、1kHz）**: 起動時セルフテスト（開ループ1回転→エンコーダ数検証、失敗で当該モータfull_stop）→ 本体速度PID×3（I16F16固定小数点、ゲインは設定でライブ調整）→ 逆運動学Matrix3x4→ 4輪電流→ **合計電流制限は4輪一律スケーリング**（進行方向を保つ）→ TMC4671へ。**with_timeoutでticker.next()を包んで「lost wakeup」検出**。ログはChannel<_, Data, 128>にtry_send（満杯なら落とす）
- **kicker_task（InterruptExecutor）**: ~230Vコンデンサ充電（PIO-DAC）、**ボール在時のみ発射**、チップ/ストレート切替サーボ+2秒プランジャ整定ロックアウト、0Vコマンドで放電、4次多項式のキック時間校正
- **panic経路**: panic-persist→WDTリセット→次回起動はUSBでpanicメッセージ報告だけの縮退アプリ（panic.rs）。ホットパスにno-panic属性

## 「アプリ追加=task追加」の実証（ユーザの着目点）

1. main.rsのspawnリストが配線図: 機能追加=static Observable 1個+spawn 1行+新ファイル1個
2. `#[cfg(feature = "test_dribbler")] spawner.must_spawn(dribbler_test_task(&DRIBBLER_SPEED));` — テストアプリが**本物と同じObservable**へ注入（test_motors時はUART受信路を#[cfg(not)]でマスク）。keeper/lupferなど機体バリアントもfeature
3. taskがSpawnerを受け取り子taskをspawn（motorcontroller_task→receive_task）
4. join3/select_biased!でtask内並行（新規taskなしで3系統キープアライブ）
5. 優先度3層（InterruptExecutor/スレッド/core1）へtaskコード無変更で配置

## フェイルセーフの多層構造（第12部07の実戦形）

無線RXタイムアウト→ゼロ化 / GameState::Haltで全ゼロ / UARTキープアライブ≥1Hz / HW WDT（main 750ms、motor 2000ms）/ ドリブラーPWM<20Hzで停止 / 基地局の物理停止ボタン / 電池Criticalで自己シャットダウン。※WDT taskは無条件給餌（executor凍結の検出であってアプリ健全性ではない—正直に書く）

## C6教材への対応表

- そのまま持ち帰れる: Observable（純embassy-sync 131行、Watchとの比較素材）、Channel/Signal/Mutex/select/join、Ticker 1kHz+lost wakeup検出、UARTプロトコル（postcard+COBS+CRC16）、WDT給餌task、ADC+ヒステリシス、フラッシュ設定、defmt
- RP2040/ATSAM固有: PIO各種（→C6はRMT/LEDC）、dual core spawn_core1（C6は単一コア）、SWI_IRQ_0のInterruptExecutor（esp-rtosに相当機能あり）、boot2
- 無線SX1280+protobuf基地局 → 概念はC6のWi-Fi/802.15.4/ESP-NOWに写像可能

## 引用重要ファイル

libs/intra-comms/src/lib.rs（ASCIIトポロジ図）/ maincontroller/src/main.rs・motorcontroller/src/main.rs（spawnグラフ）/ libs/sync/src/observable.rs / libs/intra-comms/src/uart.rs / motorcontroller/src/kicker.rs / odometry.rs / maincontroller/src/rf.rs / power.rs

## 正直に書く注意点

- IMU（BMI270ドライバ）は未使用。オドメトリはエンコーダのみ。CameraVelocity/Positionは未実装（error!ログ）
- 空README、コメントアウト残骸、「基板設計者がオペアンプを正帰還にしてしまった」等のホットフィックスコメント → 実プロジェクトの現実として教材ではむしろ活かす
