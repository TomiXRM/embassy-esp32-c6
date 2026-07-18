# esp-rs エコシステム調査資料（ESP32-C6 / no_std）

調査日: 2026-07-18（crates.io API / esp-rs GitHub / docs.espressif.com/projects/rust で検証）

## 全体像：2025〜2026年の再編

1. **esp-wifi → esp-radio**: esp-wifiは0.15.1（2025-10-14）で凍結。後継は同一リポジトリ内の**esp-radio**（バージョン連番継続）。最新安定版0.18.0（2026-04-16）、1.0.0-beta.0あり（2026-06-03）。esp-generateは0.18.0を採用
2. **esp-hal-embassy → esp-rtos**: esp-hal-embassyは0.9.1で凍結（embassy-executor 0.7系対応まで）。Embassy統合と無線用スケジューラは**esp-rtos**（features: "embassy", "esp-radio"）へ移行。esp-rtos 0.3.0はembassy-executor ^0.10 / embassy-time-driver ^0.2 に依存

## 採用構成（リリース済みクレートのみ）

| 役割 | クレート | バージョン | 備考 |
|---|---|---|---|
| HAL | esp-hal | ~1.1.0 (1.1.1) | features: esp32c6, unstable, log-04。MSRV 1.88 |
| Embassy統合+スケジューラ | esp-rtos | 0.3.0 | features: esp32c6, embassy, (esp-radio), (esp-alloc), log-04 |
| 無線 | esp-radio | 0.18.0 | wifiはstable feature。ble/esp-now/ieee802154はunstable feature必要 |
| ブートローダ連携 | esp-bootloader-esp-idf | 0.5.0 | `esp_app_desc!()`必須 |
| 実行器 | embassy-executor | 0.10.0 | |
| 時間 | embassy-time | 0.5.x | |
| ネットワーク | embassy-net | 0.9.1 | tcp, udp, dhcpv4, medium-ethernet, dns |
| BLEホスト | trouble-host | 0.6.0 | **0.7はbt-hci 0.9要求のため不可**。bt-hci 0.8.0と組み合わせ |
| 同期プリミティブ | embassy-sync | 0.7系 | 公式BLE例が0.7使用（troubleの互換都合） |
| ログ | esp-println 0.17.0 + log 0.4 | | esp-backtrace 0.19.0 (panic-handler, println) |
| ヒープ | esp-alloc | 0.10.0 | 無線使用時に必要 |

## esp-hal 1.1.1 の stable / unstable（esp32c6公式docsで確認）

- **stable**: clock, gpio, i2c, interrupt, peripherals, rng, spi, system, time, uart, efuse
- **unstable featureが必要**: analog(ADC), delay, dma, ledc, mcpwm, twai, rtc_cntl(sleep), timer, usb_serial_jtag, rmt, i2s ほか（semver保証なし）

## エントリポイント（esp-generate 1.3.0テンプレート準拠）

```rust
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    // 無線を使う場合はこの後で esp_radio::wifi::new / BleConnector::new
    ...
}
```

- build.rs: `println!("cargo:rustc-link-arg=-Tlinkall.x");`
- .cargo/config.toml: target riscv32imac-unknown-none-elf、rustflags `-C force-frame-pointers`、runner `espflash flash --monitor --chip esp32c6`
- edition 2024

## ツールチェーン

- **stable Rust**（Xtensaと違いespupは不要）。target: `riscv32imac-unknown-none-elf`
- 書き込み: espflash 4.5.0（2026-07-09）。probe-rsもUSB-JTAG経由で対応（esp-generateにオプションあり）

## 個別機能の対応状況（公式docs/examplesで確認）

- **Sleep**: rtc_cntl（unstable）。`Rtc::sleep_deep(&[&dyn WakeSource]) -> !` / `sleep_light`。`TimerWakeupSource`, `GpioWakeupSource`あり。C6は support_status = "partial"
- **TWAI**: unstable。`Twai<'d, Dm>`、async版 `transmit_async/receive_async`、`TwaiRx/TwaiTx`分割あり
- **ADC**: analog::adc（unstable）。ADC1のみ。`read_oneshot`、`into_async()`でasync版あり。校正型AdcCalBasic等
- **PWM**: LEDCとMCPWMの両方がC6に存在しesp-halが対応（両方unstable）
- **BLE**: esp-radio README「TrouBLEを推奨」。公式例: trouble-host 0.6.0 + bt-hci 0.8 + `BleConnector` → `ExternalController::<_, 1>::new`
- **ESP-NOW**: esp-radioのesp-now feature（unstable）
- **IEEE 802.15.4**: esp-radioに低レベルドライバあり（unstable）。Thread/Zigbeeスタックは対象外

## 情報源

- https://crates.io/api/v1/crates/esp-hal ほかcrates.io API
- https://github.com/esp-rs/esp-hal（examples/wifi/embassy_dhcp、examples/ble/bas_peripheral）
- https://docs.espressif.com/projects/rust/esp-hal/1.1.1/esp32c6/
- https://github.com/esp-rs/esp-generate/tree/main/template
- https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html
