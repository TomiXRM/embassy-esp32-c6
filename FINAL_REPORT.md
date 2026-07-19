# 最終報告

作成日: 2026-07-18（本編）/ 同日追記: 応用編1〜4 / 2026-07-19追記: ツールチェーンをprobe-rs+defmtへ移行

## 追記(2026-07-19): 書き込み・ログ基盤を probe-rs + defmt へ移行

- 動機: probe-rsは他チップにも使えるデバッグホストで、書き込み＋ブレークポイント＋ログを一元化できるため既定を espflash から切替
- 実施: 全22 examplesを esp-println+log → **defmt(RTT)** に移行。`.cargo/config.toml` の runner を `probe-rs run --chip esp32c6` に。espflashは `--features espflash`（defmt-over-serial）で残す二本立て
- 検証: `cargo check --workspace`（probe-rs既定）と全22クレートの `--features espflash` の**両モードでゼロエラー**、`cargo fmt --check` クリーン、final-wireless-buttonのホストテスト10/10維持。CIに両モード＋ホストテストを追加
- defmtの書式差（精度指定不可→float全精度表示、byte列は `{=[u8]:02x}`、Format非実装型は Display2Format/Debug2Format）を各クレートで解消
## 追記(2026-07-19): 実機検証（XIAO ESP32-C6 + probe-rs）

前回 `!matched` で詰まっていた実機が復帰し、probe-rs（JTAG経由、RISC-V IDCODE=Espressif確認）で書き込み・実機動作を検証した。

- **書き込み・defmtログ表示: 成功**（`cargo run` → probe-rs run → フラッシュ＋RTTでdefmtログ表示。日本語も正常）。probe-rs + defmt 移行が実機でエンドツーエンドに動作
- **20/22 exampleを書き込み・起動確認**（panic無し）。うち **14例は挙動までdefmtログで確認（hardware-tested）**:
  - blinky, embassy-tasks（マルチタスク）, channel（timeout）, ledc-fade（HWフェード中もCPU別処理）, etm（CPU非介入結線）, adc-pwm（ADC実測変動）, sleep（Deep Sleep突入をUSB電断で確認）, i2c（バススキャン）, ble（C6-BUTTONアドバタイズ）, ble-hid（C6-KEYBOARDアドバタイズ）, esp-now（ブロードキャスト送信Ok）, wifi（スタック起動＋AP探索）, sensor-node（劣化運転＋RTC RAM起動回数）
- **機能未確認（起動のみ）**: uart/spi（ループバック用ジャンパ無し）, twai（GPIO2↔GPIO3ジャンパ必須）, pcnt（GPIO10↔GPIO18ジャンパ必須）, button/keymatrix（ボタン未配線・未操作）, rmt-ws2812（XIAOはGPIO8にWS2812非搭載）
- **未フラッシュ**: final-wireless-button（2台目必要）, https（認証情報必要）。いずれも構成要素は実機確認済み
- ボード差の注意: 基準はDevKitC-1、実機はXIAO ESP32-C6。オンボードLEDやWS2812の有無が異なる（progress.md参照）
- 復帰の教訓: 前回の `!matched` はダウンロードモード（BOOT+RESET）または再接続で解消。probe-rsはCDCシリアルではなくJTAGインターフェースを使うためシリアルポート(/dev/cu.usbmodem)が出なくても書き込める



## 追記: 応用編（本編120ページ完成後の拡張）

本編に加えて、実在プロジェクトを読み解く応用編4本（計42ページ、全て完全原稿）とexamples 8本を追加した。

| 応用編 | ページ | 題材（ライセンス） | 追加example |
|---|---|---|---|
| 1 キーボードを作る視点 | 10 | Keyball Embassy/RP2040ファーム（ライセンス無し→転載ゼロ・独自再構成） | 14-keymatrix, 15-ble-hid |
| 2 センサ端末を作る視点 | 10 | esp32c3-embassy（MIT/Apache-2.0、同世代スタック） | 16-sensor-node, 17-https |
| 3 ロボットファームを読む | 12 | luhsoccer_firmware（LuhBots, MIT、RoboCup SSL実戦機） | - |
| 4 深淵・キモい機能図鑑 | 10 | ESP32-C6の周辺機能（テーマ: CPUに全部やらせない） | 18-rmt-ws2812, 19-pcnt, 20-etm, 21-ledc-fade |

- 総計: サイト175ページ（本編120+付録3+応用編42+その他）、examples 22クレート全cargo check通過
- 応用編4は全機能に「Rustからの現在地」（stable/unstable/ESP-IDFのみ）を付記。ETM実例（CPU・割り込み不関与のボタン→LED直結）とRMTによるオンボードWS2812点灯を含む
- チーム実績等の事実は公式一次資料で裏取り（LuhBotsのSSL成績、SSLルール、TDP）
- 特記: 応用編3の執筆中にluhsoccer_firmwareのCRC実装の不具合疑い（digest()使い捨てによるCRC定数化）を発見し、実コード検証の上で教訓として掲載

---

公開サイト: https://tomixrm.github.io/embassy-esp32-c6/
リポジトリ: https://github.com/TomiXRM/embassy-esp32-c6

## 制作結果

- 総ページ数： **教材120ページ**（12部×10）+ はじめに1 + 付録3（用語集・Arduino対応表・トラブルシューティング）+ プロジェクト情報7 = サイト生成133ページ
- 完全原稿（complete）： **98 / 120**
- 構成・下書き（drafted）： **22 / 120**（全て本文・学習目標・確認問題あり。骨格のみ＝outlinedのページは0）
- 未完成（本文なし）： **0**
- サンプル数： **14プロジェクト**（examples/01〜13 + final-wireless-button）
- cargo check成功： **14 / 14**（riscv32imac-unknown-none-elf、エラー0・警告0、`cargo fmt --check`クリーン。final-wireless-buttonのprotocolはホスト（aarch64-apple-darwin）で単体テスト10/10成功）
- 実機確認済み： **0**（実機なしのため。hardware-tested表記は教材内に一切なし）
- サイトビルド： **成功**（Astro 7.1.1 + Starlight 0.41.3、133ページ、Pagefind全文検索・Mermaid込み）
- リンクチェック： **内部リンク切れ0件**（生成後の全133 HTMLを機械走査）。他に重複タイトル0、空ページ0、TODOマーカー0、コードフェンス閉じ忘れ0を確認

## 採用した技術構成

- 対象: ESP32-C6-DevKitC-1（ESP32-C6-WROOM-1, 8MBフラッシュ）、no_std
- HAL: **esp-hal 1.1.1**（`esp32c6` + `unstable` + `log-04`）
- Embassy統合・スケジューラ: **esp-rtos 0.3.0**（`embassy` + `esp-radio`。旧esp-hal-embassyは凍結済みのため不採用）
- 無線: **esp-radio 0.18.0**（wifi / ble / esp-now）
- 非同期: embassy-executor 0.10.0 / embassy-time 0.5 / embassy-sync 0.7 / embassy-net 0.9.1
- BLE: trouble-host 0.6.0 + bt-hci 0.8（esp-radio 0.18がbt-hci ^0.8実装のため0.7系は不採用）
- ブート: esp-bootloader-esp-idf 0.5.0（`esp_app_desc!`）、ログ: esp-println 0.17 + log、panic: esp-backtrace 0.19
- ツールチェーン: Rust stable 1.97.1、target riscv32imac-unknown-none-elf、書き込みespflash 4.5.0
- サイト: Astro 7.1.1 + Starlight 0.41.3 + astro-mermaid 2.1.0、GitHub Actions→GitHub Pages
- 最終プロジェクト通信方式: **ESP-NOW**（BLE/Wi-Fiとの11観点比較の上で採用。比較表はpart12/10に掲載）

## 固定したバージョン

`docs/project/versions.md` に全クレート・ツールを固定（examples/Cargo.tomlのworkspace.dependenciesと一致）。unstable API（ADC/LEDC/TWAI/rtc_cntl/timer等）を使う理由と更新時に壊れうる箇所も同文書に記録。

## 実行した検証コマンド

- `cargo check --workspace`（examples、14クレート全て成功）
- `cargo fmt --all -- --check`（クリーン）
- `cargo test -p final-wireless-button --lib --target aarch64-apple-darwin`（10/10成功）
- `npm run build`（site、133ページ生成成功）
- 内部リンク機械チェック（dist全HTML走査、切れ0）
- frontmatter集計（status/code_status/prerequisites/学習目標/演習/確認問題の全ページ存在確認）
- 教材スニペットの追加検証: 第2〜4部の全スニペットと第9部のexample外API（select/join/Signal/Mutex等）は、検証用クレートで実際にcargo check/実行して確認済み。掲載エラー文面（E0382等）はrustc 1.97.1の実出力

## 実際に確認できた機能（cargo checkレベル）

GPIO出力/入力（async wait含む）、Embassy task/Spawner/Timer/Ticker/with_timeout/Channel/Signal/select、UART1 async、I2C master（スキャン+SHT30手順）、SPI2 async、TWAI（SelfTestモード、async送受信）、ADC1 oneshot（校正付きasync）、LEDC PWM、Wi-Fi STA + DHCP + DNS + TCP/HTTP GET、BLE GATTペリフェラル（advertise+notify）、ESP-NOW（ブロードキャスト送受信）、Deep Sleep（タイマー+EXT1起床）、最終プロジェクト（多task構成・自作プロトコル・再送/重複排除）

## ビルドのみ確認した機能

上記すべて「ビルドのみ確認」です。**実機動作は未確認**（実機なし）。特にExt1起床の極性、ADC校正値、Wi-Fi/BLEの実挙動、ESP-NOW到達距離は実機での確認が必要です。

## 調査のみで実装できなかった機能

- Wi-Fi Access Pointモード（概念説明のみ）
- UDP/DNS以外のembassy-netソケット応用、MQTT、小型HTTPサーバー（概念説明のみ）
- BLE Central（概念説明のみ）
- IEEE 802.15.4生フレーム、Thread/Zigbee（Rustスタック不在のため概念説明のみ）
- Flash書き込み・不揮発ストレージ・OTA（概念説明のみ）
- Watchdog実コード、Light Sleep実コード（概念説明のみ。sleepはDeep Sleepのみ実装）
- 消費電力の実測（データシート典型値のみ、出典明記）

## 技術的に不安定な機能

- esp-halの`unstable` feature配下API（ADC/LEDC/MCPWM/TWAI/rtc_cntl/timer/DMA等）— semver保証なし
- esp-radioのble/esp-now/ieee802154 feature — unstable指定
- esp-radio 1.0.0-beta系が既に存在し、次期リリースでAPI変更の可能性（教材は0.18.0に固定）

## 教材上の重要な判断

1. **esp-rtos採用**: esp-hal-embassy（凍結）ではなく現行のesp-rtos 0.3.0を採用。esp-generate公式テンプレートと同一構成
2. **trouble-host 0.6固定**: 最新0.7はbt-hci 0.9要求でesp-radio 0.18と組めないため（公式exampleと同じ0.6.0）
3. **embassy-sync 0.7**: trouble互換のため最新0.8ではなく0.7（公式example準拠）
4. **build-std不使用**: esp-generateテンプレートの`[unstable] build-std`はstableでは無効のため省略し、プリビルドcoreを使用
5. **Lチカは外付けLED（GPIO10）**: DevKitC-1のオンボードLEDはWS2812B（GPIO8）で単純ON/OFF不可のため
6. **最終プロジェクトはESP-NOW**: 接続レス即時性・ルーター不要・信頼性設計の学習価値から（比較表で理由明示）
7. **statusの正直申告**: 未検証コードを含むページはcompleteにせずdrafted、コードはconcept-only等で明示
8. **TWAIはSelfTestモードで学習開始**: トランシーバなしで安全に試せる構成にし、実バスへの発展手順を別途記載

## 既知の問題

- drafted 22ページはスニペット単体検証または実装素材が不足（内訳はdocs/project/progress.md）
- 実行ログ例（センサ値等）は実機未検証の想定出力（該当ページに注記あり）
- 第9部検証用スニペットクレートはscratchpadで検証済みだがリポジトリ未収録（examplesへの取り込みが望ましい）
- ESP-NOWのユニキャストACKは受信側でのPeerInfo登録が必要（final-wireless-buttonで実装済みだが実機検証待ち）

## 次に完成させるべきページ

1. part12/01-light-sleep（Light Sleep実コードの検証とcomplete化 — 最終プロジェクトの省電力化に直結）
2. part05/05-heap, 06-static, 07-panic（heapless/StaticCellスニペットの検証 — 基礎部の完成度向上）
3. part07/03-sensor-reading（移動平均コードの検証）
4. part07/06-servo（LEDCサーボの実装検証 — duty分解能の課題解決含む）
5. part10/07-udp（UdpSocketコードの検証）
6. part09/03-future（概念ページとしてreviewed昇格の判断）
7. part05/04-stack, 08-pac、part06/10-watchdog、part08/08-bus-sharing（概念ページの推敲）
8. part10/03-access-point, 10-mqtt-or-server、part11/05-central, 09, 10（対応クレート成熟待ちの再調査）

## 継続作業の優先順位

1. **実機検証**: 全14 examplesをESP32-C6-DevKitC-1で動作確認し、hardware-testedへ昇格（最優先。特にExt1起床・ADC・無線系）
2. 上記「次に完成させるべきページ」のdrafted→complete化
3. 第9部スニペットクレートのexamples取り込みとCI組み込み
4. esp-radio 1.0正式リリース時のバージョン追従（versions.mdの手順に従い全examples再検証）
5. 実機写真・実測ログ・電力実測値の追加
