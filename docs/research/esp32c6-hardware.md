# ESP32-C6 / ESP32-C6-DevKitC-1 ハードウェア調査資料（公式資料のみ）

調査日: 2026-07-18

## 一次資料

- DS: ESP32-C6 Series Datasheet v1.5 — https://documentation.espressif.com/esp32-c6_datasheet_en.pdf
- UG: ESP32-C6-DevKitC-1 User Guide — https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html
- SCH: ESP32-C6-DevKitC-1 回路図 v1.2 (2023-01-10) — https://dl.espressif.com/dl/schematics/esp32-c6-devkitc-1-schematics_v1.2.pdf
- IDF-TWAI: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/peripherals/twai.html
- IDF-Sleep: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/api-reference/system/sleep_modes.html

## 1. コア仕様 [DS]

- HP CPU: 32bit RISC-V RV32IMAC、最大160MHz
- LP CPU: 32bit RISC-V RV32IMAC、最大20MHz（LPコアあり）
- メモリ: HP SRAM 512KB、LP SRAM 16KB、ROM 320KB
- フラッシュ: DevKitC-1搭載のESP32-C6-WROOM-1モジュールは8MB SPIフラッシュ [UG]
- 無線: Wi-Fi 6 (802.11ax, 2.4GHzのみ, 最大150Mbps, TWT対応)、Bluetooth LE (Bluetooth 5.3認証, 最大+20dBm)、IEEE 802.15.4-2015 (250Kbps, Thread 1.3 / Zigbee 3.0)

## 2. DevKitC-1ボード [UG + SCH]

- RGB LED: WS2812B（アドレサブル）が **GPIO8** に接続。単色のユーザーLEDは無い（赤LEDは電源表示のみでGPIO制御不可）
- BOOTボタン: **GPIO9**（SW1がGPIO9をGNDへ引く）。RSTボタンはCHIP_PU
- USB: USB-C×2 — ①CP2102N経由のUART（UART0 = GPIO16 TX / GPIO17 RX）②ネイティブUSB Serial/JTAG（GPIO12 D− / GPIO13 D+）
- ヘッダ露出GPIO: 0–13, 15, 16–23（QFN40ダイにGPIO14は存在しない）
- ストラッピングピン注意 [UG]: GPIO4 (MTMS), GPIO5 (MTDI), GPIO8, GPIO9, GPIO15

## 3. ペリフェラル [DS]

| ペリフェラル | 数・詳細 |
|---|---|
| UART | HP UART×2 + LP UART×1 |
| I2C | HP I2C×1 + LP I2C×1 |
| SPI | SPI0/1はフラッシュ用。汎用は**SPI2**（GDMA対応） |
| TWAI (CAN) | **2コントローラ**、ISO 11898-1 (CAN 2.0) |
| ADC | **SAR ADC1のみ、12bit、7ch: ADC1_CH0–CH6 = GPIO0–GPIO6**。ADC2なし。最大100kSPS |
| LEDC PWM | 6チャンネル |
| MCPWM | あり（タイマー×3、オペレータ×3、PWM出力×6）[DS §4.2.1.10] |
| DAC | **なし** |
| USB Serial/JTAG | あり（USB 2.0 FS、GPIO12/13固定） |
| SDIO | スレーブのみ（GPIO18–23） |
| 温度センサ | あり（−40…125°C） |
| その他 | I2S×1、PCNT、RMT 4ch、PARLIO、52bit systimer、54bit GPタイマー×2、WDT×3 |

## 4. 電力モード [DS Tables 5-7〜5-11]

| モード | 代表電流（データシート典型値） | 備考 |
|---|---|---|
| Active Wi-Fi TX | 252〜354mA（変調・出力による） | Table 5-7 |
| Active Wi-Fi RX | 78〜82mA | Table 5-7 |
| Active BLE TX | 130mA @0dBm 〜 315mA @+20dBm、RX 71mA | Table 5-8 |
| Modem-sleep | 27mA（160MHz、周辺クロックoff） | Table 5-10 |
| Light-sleep | 180µA（周辺電源on）/ 35µA（周辺電源off） | Table 5-11 |
| Deep-sleep | 7µA（RTCタイマー+LPメモリon） | Table 5-11 |
| 電源断 (CHIP_PU=L) | 1µA | Table 5-11 |

- Deep-sleepでは**HP SRAMは保持されない**。LP SRAM (16KB) は保持される
- 復帰要因 [IDF-Sleep]: Light-sleep — RTCタイマー、任意GPIOレベル、EXT1、UART、LPコア。Deep-sleep — RTCタイマー、EXT1（LP GPIO = GPIO0–7）、LPコア/LP UART

## 5. GPIO一覧 [DS Table 2-10]

- QFN40（WROOM-1搭載ダイ）: GPIO0–13, 15–30 の30本
- フラッシュ用（使用禁止）: GPIO24, 25, 26, 28, 29, 30。GPIO27はVDD_SPI
- LP GPIO（Deep-sleep中も有効）: GPIO0–7
- ADC対応: GPIO0–6（GPIO0/1は32kHz水晶と兼用）
- 注意ピン: GPIO4/5/8/9/15（ストラッピング）、GPIO12/13（USB S/J）、GPIO4–7（JTAG）、GPIO16/17（UART0コンソール）

## 6. ストラッピングとブート [DS §3]

- SPI boot: GPIO9 = 1（GPIO8は任意）
- Joint download boot: GPIO8 = 1 かつ GPIO9 = 0
- GPIO9はデフォルト弱プルアップ。BOOTボタンを押しながらリセットで書き込みモード
- GPIO8はDevKitC-1でWS2812のデータ線と兼用 — リセット時にLowへ引かれると書き込みモードに入れない

## 7. TWAIトランシーバ [IDF-TWAI]

- 「ESP32-C6は内蔵TWAIトランシーバを持たないため、外付けトランシーバが必須」（公式明記）
- 公式ページの例: TJA105x系（ISO 11898-2）。SN65HVD230は業界でよく使われる3.3V品だがC6ページには明記なし（UNVERIFIED as official例）

## 8. 電気的制限 [DS Tables 5-1/5-2/5-4]

- 推奨動作電圧: 3.0–3.6V（典型3.3V）
- 絶対最大定格: 入力 −0.3〜3.6V、IO出力累積 1000mA
- ピン駆動能力（典型, PAD_DRIVER=3）: ソース40mA、シンク28mA。内部プル抵抗 約45kΩ
- V_IH = 0.75×VDD、V_IL = 0.25×VDD
