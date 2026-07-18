# ESP32-C6「キモい機能」のRust対応状況（esp-hal 1.1.1 / esp-radio 0.18.0で検証）

調査日: 2026-07-18。ローカルソース+公式docs（docs.espressif.com/projects/rust、ESP-IDF esp32c6、TRM v1.2、Datasheet v1.5）で全項目検証済み。本教材はesp-halのunstable featureを有効化済みのため「unstableで試せる」=そのまま書ける。

## 総括表（verdict: 今すぐ試せる / unstableで試せる / 概念のみ(ESP-IDF)）

| 機能 | C6ハード事実 | esp-hal 1.1.1 | verdict |
|---|---|---|---|
| GPIO Matrix | GPIO0-30。24-30フラッシュ用/12-13 USB/ストラップ4,5,8,9,15 | stable。各ドライバのwith_xxx()が実体。interconnect::{InputSignal,OutputSignal}/split()はunstable | 今すぐ試せる |
| ETM | 50チャネル(TRM) | etm(unstable)。Etm::new→channel0..49、channel.setup(&event,&task)。配線済み: GPIO(event+task)/SYSTIMER(event)/TIMG(event+task)。**LEDC/PCNT/RMTのETM未実装** | unstableで試せる |
| RMT | TX×2+RX×2、RAM 48×32bit | rmt(unstable)。Rmt::new(RMT, Rate::from_mhz(80))→configure_tx/rx→with_pin、PulseCode、**async対応**。**esp-hal-smartledは~1.0固定で1.1.1非互換→生PulseCode一択** | unstableで試せる |
| PCNT | 4ユニット×2ch、フィルタ10bit(≤1023 APBサイクル) | pcnt(unstable)。unit.set_filter/channel.set_edge_signal(input.peripheral_input())/EdgeMode/value()->i16/上下限+割り込み。asyncなし。モジュールdocに4逓倍エンコーダ例 | unstableで試せる |
| MCPWM | タイマ×3/オペレータ×3(6出力)/デッドタイム/キャプチャ3ch | mcpwm(unstable)。DeadTimeCfg+LinkedPins(相補+デッドタイム)、SWシンク。**キャプチャ/HWシンク/ETM連携未実装**。qa-test/mcpwm.rsのみ | unstableで試せる |
| DMA | GDMA 6ch。SPI2/UHCI/I2S/AES/SHA/ADC/PARLIO | dma(unstable)。DMA_CH0..2、DmaTxBuf/RxBuf、dma_buffers!。接続済み: SPI2/I2S0/PARL_IO/AES/SHA/Mem2Mem。**ADCのDMA未実装** | unstableで試せる |
| ADC連続+DMA | ハード対応、IDFにadc_continuousあり | **oneshotのみ** | 概念のみ(ESP-IDF) |
| LEDCハードフェード | 6ch、HWフェード | ledc(unstable)。**channel.start_duty_fade(start%, end%, ms) + is_duty_fade_running() あり** | unstableで試せる |
| SDM(Σ-Δ) | 4ch、2次(TRM §7.5.4.1) | モジュールなし | 概念のみ(ESP-IDF) |
| PARLIO | 幅1/2/4/8/16bit(16bitは半二重のみ)、最大40MHz | parl_io(unstable)。ParlIo::new(PARL_IO, dma_ch)（DMA必須）、TxSixteenBits等、async対応 | unstableで試せる |
| Dedicated GPIO | CSR経由8ch、csrrsi/csrrci高速パスは下位4ch(TRM §1.14) | gpio::dedicated(unstable)。**フルドライバあり**（DedicatedGpioInput/Output/Flex、バンドル、write_ll/read_all_ll） | unstableで試せる |
| GPIOグリッチフィルタ | ピン固定+フレキシブル×8 | **未実装**（あるのはPCNT内フィルタのみ） | 概念のみ(ESP-IDF) |
| LPコア | RV32IMAC 20MHz、LP SRAM 16KB | lp_core(unstable): LpCore::new/run(LpCoreWakeupSource::HpCpu)/load_lp_code!。LP側は**esp-lp-hal 0.3.0**（esp32c6対応、examples: blinky/i2c/uart）。2バイナリ構成で難度高 | unstableで試せる |
| eFuse | - | efuse(stable)。chip_revision/interface_mac_address等。**書き込みAPIなし=読み取り専用** | 今すぐ試せる(読み取り) |
| USB Serial/JTAG | CDC-ACM+JTAG固定機能（OTG不可） | usb_serial_jtag(unstable)。into_async()、embedded-io(-async)実装 | unstableで試せる |
| Wi-Fiスニファ/CSI | プロミスキャス/raw 802.11 TX/CSI対応 | esp-radio 0.18: feature **sniffer**(+unstable)→Interfaces.sniffer（set_promiscuous_mode/set_receive_cb/send_raw_frame）、feature **csi**→WifiController::set_csi(CsiConfig, cb) | unstableで試せる |
| TWAI×2 | コントローラ×2 | twai(unstable)。TWAI0/TWAI1両方impl済み、ピン自由配線、async | unstableで試せる |

## 公式example（tag esp-radio-v0.18.0）

- ETM: examples/peripheral/etm_timer（唯一）
- RMT: examples/async/embassy_rmt_tx / embassy_rmt_rx（WS2812例は無い）
- LPコア: examples/peripheral/lp_core/lp_blinky, lp_i2c + esp-lp-hal/examples
- TWAI: examples/peripheral/twai、MCPWM: qa-test/src/bin/mcpwm.rs
- PCNT/PARL_IO/dedicated GPIO: exampleなし（PCNTはモジュールdocに例）

## 教材example実現性（検証済み）

- **(a) RMT×オンボードWS2812(GPIO8)**: 可能。Rmt::new(80MHz)→into_async()→channel0.configure_tx(clk_divider)→with_pin(GPIO8)→PulseCode::new(Level, len, Level, len)配列+end_marker→transmit().await。GRB 24bit/T0H≈0.4µs等を自前計算（smartled不可のため）
- **(b) PCNT+フィルタ**: 可能。set_filter(≤1023)、EdgeMode::Increment/Hold、value()。ポーリングか割り込み
- **(c) ETM**: 可能だが範囲限定。①GPIOイベント→GPIOタスク直結（gpio::etm::Channels::new(GPIO_SD)、falling_edge/toggle — etm.rs docに完全例）②GPIO→TIMG cnt_start/stop ③SYSTIMERアラーム→GPIOトグル。**GPIO→LEDCは不可**（LEDC ETM未実装）。configured channelはdropで無効化→束縛保持必須

## 執筆注意

- gpio::etmはイベント8ch+タスク8ch（ETM本体50chとは別）。イベントrising/falling/any_edge、タスクset/clear/toggleのみ
- 「C6ハードは対応・IDFは対応・Rustは未対応」（ADC連続/SDM/グリッチフィルタ）は"ハード対応とライブラリ対応を混同しない"の生きた実例として使う
- versions.mdのunstable一覧にetm/pcnt/rmt/parl_io/gpio::dedicated/lp_core/i2sを追記のこと
