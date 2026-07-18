# esp32c3-embassy（claudiomattera）調査資料（応用編2の素材）

調査日: 2026-07-18

## 題材

- リポジトリ: https://github.com/claudiomattera/esp32c3-embassy （主リポジトリはGitLab: gitlab.com/claudiomattera/esp32c3-embassy）
- **ライセンス: MIT OR Apache-2.0** → コードの引用・移植が可能（出典明記の上で）
- 内容: BME280（I2C）+ WaveShare 1.54" 電子ペーパー（SPI）の非同期気象ステーション。READMEで「参照・例・出発点として意図的に作り込んである」と明言
- 活動: 現役メンテ中。v0.8.0（2026-02-13）。62星
- clone済み: scratchpad/esp32c3-embassy

## ★最重要: スタック世代が本教材とほぼ同一★

v0.8.0で esp-wifi→esp-radio / esp-hal-embassy→esp-rtos 移行済み（CHANGELOGに移行記録あり — それ自体が移行の生きた教材）:

| 彼ら（v0.8.0） | 本教材 |
|---|---|
| esp-hal 1.0.0 (esp32c3, unstable, log-04) | esp-hal 1.1.1 (esp32c6, unstable, log-04) |
| esp-rtos 0.2.0 | esp-rtos 0.3.0 |
| esp-radio 0.17.0 (wifi) | esp-radio 0.18.0 |
| embassy-executor 0.9.1 / time 0.5 / net 0.7 | executor 0.10 / time 0.5 / net 0.9 |
| `#[esp_rtos::main]` + esp_rtos::start(...) | 同一パターン |

→ **世代書き換え不要**。ピン・feature・target（riscv32imc→imac）・マイナーバージョン差の調整のみでC6へ移植可能。

## アーキテクチャ（デューティサイクル）

起動 → BOOT_COUNTインクリメント（RTC fast RAM）→ 時刻がRTC RAMに無ければWi-Fi接続してHTTPSで時刻取得（Adafruit IO）、あれば**Wi-Fiを一切使わない** → センサtask+表示taskをspawn → 300秒後に時刻をRTC RAMへ保存（予定スリープ時間を加算）→ deep sleep 300秒 → 最初へ

- データのアップロードはしない（HTTPは時刻同期のみ、出力は電子ペーパーとログ）
- task構成: connection（Wi-Fi、STOP_WIFI_SIGNALで停止）/ net_task / sample_task → Channel<NoopRawMutex, (OffsetDateTime, Sample), 3> → update_display_task
- **deep sleep前にWi-Fi停止必須**（`controller.stop_async()`。「止めないと無限ブロック」とコメントあり）

## 教材が未カバーの学びどころ

1. **公開ドライバクレートの活用**: bme280-rs 0.3（作者自身のクレート、AsyncBme280、embedded-hal-async 1.0準拠→esp-hal 1.1でそのまま使える）。設定はビルダー（Configuration + Oversampling + SensorMode::Normal）、**校正はドライバ内部**（アプリは物理量だけ受け取る）、Sampleは各フィールドOption。→ 第8部の自前SHT30レジスタ叩きとの対比が最高の教材
2. **RTC fast RAMでdeep sleepを生き残る状態**: `#[ram(unstable(rtc_fast))]` static — BOOT_COUNT、96件のHistoryBuf<Reading>、起動時刻。単一コア前提の自作SyncUnsafeCellでラップ
3. **壁時計時刻の維持**: boot_time = サーバepoch − Instant::now()。保存時に予定スリープ時間を加算。**丸め起床**（09:46:12+5分周期→09:50:00に起きる）アルゴリズム（clock.rs:107-138）
4. **no_stdでHTTPS**: reqwless 0.13 + embedded-tls（TLS 1.3のみ）。16640バイト×2のレコードバッファ、TlsVerify::None（暗号化はするが証明書検証なし—トレードオフを正直に扱う素材）、esp-hal RngのRand_coreラッパでシード
5. **taskはResultを返せない問題**: `task()` → `task_fallible() -> Result` のラッパパターン。モジュール別エラーenum+From連鎖。センサ故障時はダミー値で継続（劣化運転）
6. **型付きペリフェラル構造体**: DisplayPeripherals/SensorPeripherals（型付きGPIOnフィールド）で「誰がどのピンを持つか」を型で表現。SPI DMA + embedded-hal-bus ExclusiveDevice + SpiDeviceジェネリックドライバの層構造。uomでSI単位の型安全

## センサ詳細

- I2C 25kHz（低速で確実）。`AsyncBme280::new(i2c, Delay)` → `init()` → `set_sampling_configuration(...)` → 10ms待ち → `read_sample()`
- bme280-rsはuom feature可（型付き量）。教材exampleではf32のまま扱う方針

## C6移植の差分（要点）

- feature esp32c3→esp32c6、target riscv32imc→riscv32imac、linker等はそのまま
- ピン番号変更（I2CはC6教材標準のGPIO6/7へ）
- esp-radio 0.17→0.18、esp-rtos 0.2→0.3、executor 0.9→0.10の小改名対応
- Rtc::sleep_deep / TimerWakeupSource / #[ram(unstable(rtc_fast))] はC6でも同一API
