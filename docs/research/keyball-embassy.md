# Keyball Embassy/RP2040 ファームウェア調査資料（応用編の素材）

調査日: 2026-07-18

## 題材

- 記事: https://zenn.dev/nazo6/articles/keyball-embassy-rp2040 （2024-05-22公開、nazo6氏）
- リポジトリ: https://github.com/nazo6/rktk-keyball-rs （旧名 keyball-embassy-rp2040 → keyball-rs。記事時点のコードはコミット eed7ac8、legacyブランチに保存）
- **ライセンス: 記事時点のリポジトリにはLICENSEファイルなし** → 教材ではコードの引用は数行の最小限に留め、必ず出典（ファイルパス+URL）を明記。大部分は独自の言葉と独自図で解説する
- 現在は作者のフレームワーク rktk (MIT) ベースに書き換え済み（rp2040 + nrf52840対応）

## アーキテクチャ（記事時点）

- モジュール層: `device/`（チップ固有）/ `driver/common/`（チップ非依存ドライバ）/ `driver/rp2040/`（PIO等）/ `keyboard/`（キーコード・キーマップ）/ `state/`（純粋状態機械）/ `task/`（並行処理の配線）/ `usb/`
- **spawnするembassy taskは1つだけ**。並行処理は `task::start` 内の embassy_futures::join / select ツリーで構成（構造化並行性）。→ 教材第9部「複数task vs 1task内join/select」の実例
- 主なフューチャー: usb_task / led_task / core_task（USB列挙の早い者勝ちでmaster/slave判定、QMKのSPLIT_USB_DETECT相当）/ master側 main_loop・split_handler・report / slave側 main_loop・split_handler

### Channel/Signal/Mutex構成

| プリミティブ | 型 | 用途 |
|---|---|---|
| S2mChannel | Channel<ThreadModeRawMutex, SlaveToMaster, 64> | Pressed/Released/Mouse{x,y}/Message |
| M2sChannel | Channel<ThreadModeRawMutex, MasterToSlave, 64> | LED制御等 |
| kbレポート | Channel<ThreadModeRawMutex, KeyboardReport, 10> | usbd-hidのレポート |
| mouseレポート | Channel<ThreadModeRawMutex, MouseReport, 10> | 同上 |
| LedCtrl | Signal<ThreadModeRawMutex, LedControl> | LEDアニメ指示（最新値のみ） |
| RemoteWakeupSignal | Signal<CriticalSectionRawMutex, ()> | USBリモートウェイクアップ |
| DISPLAY | Mutex<ThreadModeRawMutex, Option<Oled>> | OLED共有。panicハンドラからはtry_lock |
| COMM_SEMAPHORE | FairSemaphore<ThreadModeRawMutex, 3> | 1-wireバス調停 |

- ワイヤ上のメッセージはpostcard+serdeでシリアライズ（rkyvからunsafe排除のため移行）
- 当時のembassy: executor 0.5 / time 0.3 / sync 0.5 / embassy-rp 0.1 / embassy-usb 0.2（git版）。現行rktkはexecutor 0.7系

## キースキャン（Duplex Matrix）

- 物理は5×4ピンだが、**col→row と row→col の2回スキャン**（Flexピンで方向切替）で論理5×7/片手を実現
- async settle: `col.wait_for_high().await` で立ち上がり待ち（busy-waitでなく）
- 左右判定: 特定マトリクス位置(2,6)のジャンパで判定
- **デバウンスは実質なし**: エッジ検出のみ。スキャンループを`MIN_KB_SCAN_INTERVAL=20ms`にペーシングすることで代用（`Instant::now()`+`elapsed`+残り時間`Timer::after_micros`）→ 教材のDebouncer（14-keymatrix）と対比できる

## キーマップ / レイヤ / Tap-Hold

```rust
enum KeyCode { Key(..), Mouse(..), Modifier(..), WithModifier(..), Layer(LayerOp), Special(..) }
enum KeyAction { Tap(KeyCode), TapHold(KeyCode, KeyCode) }
enum KeyDef { None, Inherit, Key(KeyAction) }
type Layer = [[KeyDef; COLS*2]; ROWS]; // 4レイヤ
```

- `AllPressed`が各キーの押下開始`Option<Instant>`を保持し、毎周期 Pressed/Pressing(Duration)/Released(Duration) イベントへ変換
- **Tap-Holdは押下時間のみで判定**（TAP_THRESHOLD=200ms超でHold）。QMKのPERMISSIVE_HOLD（他キー割り込みでHold確定）はこの版には無い
- レイヤ: `[bool; 4]`、上位から検索、Inheritは下へ透過、Noneは遮断。Move=モーメンタリ、Toggle=リリース時
- `state/`は**HAL非依存の純粋ロジック**（embassy-timeの型のみ）→ 教材part12/09（テスト可能な設計）と同じ思想

## 左右通信（PIO半二重1-wire）

- TRSケーブル3極（V+/GND/データ1線）。PIO0のSM0=RX、SM1=TXで1本のピンを共有。100kbps
- フレーミング: スタートビット+8データ+開始チェック+終了チェックの10ビット/バイト、パケット最大8バイト
- TX時は`set pindirs`で出力化→送信後に入力へ戻す。RX復帰時はFIFOドレイン+300µsガード
- 調停はFairSemaphore、衝突時は**ベストエフォート（ACK/再送/CRCなし）**。README自ら「衝突で不安定」と記載 → 最終プロジェクト（seq/ACK/再送/重複排除）との対比が最高の教材
- QMKのserial_vendor.cを参考に実装

## USB HID（embassy-usb）

- Builderで2つのHIDインターフェース: キーボード（KeyboardReport::desc()、ポーリング10ms）とマウス（MouseReport::desc()、4ms）
- reportタスクはチャンネルから受けて`write_serialize`。SUSPENDED中の入力でRemoteWakeupSignal→`device.remote_wakeup()`
- master/slave判定: `select(hid.keyboard.ready(), Timer::after_millis(200))`

## トラックボール（PMW3360）

- 自作async SPIドライバ（SPI0+DMA、7MHz）。SROMファームアップロード、burst_read 12バイト、データシート指定の待ち時間をTimer::after_microsで
- `Ball::init(..).await.ok()` → `Option<Ball>`で**ボール無しでも劣化運転**
- オートマウスレイヤ: マウス系イベントでlayer1有効化+Instant記録、500ms動きなしで解除。LED色も連動
- スクロールモード: 除数で割った**余りを持ち越す**固定小数点的な蓄積（教材向きの小ネタ）

## エコシステム

- **rktk**（同作者、MIT、crates.io v0.2.0）: この記事のファームを核/ドライバ分離でライブラリ化。rp2040+nrf52840。ESP32非対応
- **RMK**（HaoboGu/rmk、Apache-2.0、~1.7k星、活発）: Embassyベースで最成熟。STM32/nRF52/RP2040/**ESP32対応（esp32c6_bleの例あり）**。Vial対応、keyboard.toml設定、BLE分割・ドングルモード
- **rumcake**（Univa/rumcake、MIT）: keyberonベースのライブラリ。ESP32非対応、nightly必要、事実上休止 → 歴史的文脈として紹介、現役はRMK

## チップ非依存 vs RP2040固有（C6移植の視点）

| 部品 | 判定 |
|---|---|
| join/selectツリー、Channel/Signal/Mutex構成、ループペーシング、Tap-Hold/レイヤ/オートマウス状態機械 | **そのまま移植可**（embassy-sync/time/futuresのみ依存） |
| マトリクススキャンのロジック | 移植可（esp_hal::gpio::Flexが対応物） |
| PMW3360/SSD1306ドライバ | 移植可（async SPI/I2Cがあればよい） |
| USB HID | **C6には汎用USBデバイスコントローラがない**（Serial/JTAGのみ）→ BLE HID（RMKがC6でやる方式）か、USBが要るならS3等を選ぶ |
| PIO半二重1-wire | RP2040固有（C6にPIOなし）→ 半二重UARTか2線UART、または無線（ESP-NOW）で再設計。フレーミング/調停の設計は流用可 |
| WS2812駆動 | RP2040はPIO、C6はRMT（esp-hal-smartled系）で等価 |
| ダブルタップでBOOTSEL、bind_interrupts!等 | RP2040固有 |
| USB列挙によるmaster/slave判定 | USBが前提。C6では別方式（配線・設定・先着など）が必要 |

記事時点のコードは scratchpad に取得済み（keyball-article/）。教材への引用は最小限とし、出典パスを明記すること。
