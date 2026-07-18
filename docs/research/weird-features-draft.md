# ESP32-C6「キモい機能」ドラフト（プロジェクトオーナー提供、2026-07-18）

応用編4の設計図。テーマ: **「CPUに全部やらせるのをやめ、専用ハードウェアへ仕事を分担させよう」**
対象読者視点: ArduinoでSPIセンサを読んだことがある中学生から見た「びっくり機能」。

執筆時の扱い:
- 出典は主にESP-IDF公式ドキュメント（各項目にURLあり）。**教材化の際はesp-hal 1.1.1でのRust対応状況調査（別資料）と突き合わせ、各機能に「Rustからの現在地」を必ず付ける**
- 「CANピンをどこからでも出せる」はGPIO Matrixの効果であり、フラッシュ/USB/ストラッピングピン等の制約と外付けトランシーバ必須は正直に書く（ドラフト自身が明記）
- ESP-NOW/無線3種の詳細は第11部と重複させず、リンクで接続

## 優先上位10（この順で厚く扱う）

1. GPIO Matrix 2. 割り込みとハードウェア周辺回路 3. DMA 4. RMT 5. PCNT 6. ETM 7. MCPWM 8. LP CoreとDeep Sleep 9. ESP-NOW（既存章接続） 10. Wi-Fi/BLE/Threadの違い（既存章接続）

特にETM・RMT・PCNT・MCPWMが「Arduinoの上位互換」から抜け出す境目。

## 20機能の要点（ドラフト原文の要約。出典URL付き原文は本ファイル末尾のメモ参照）

1. **GPIO Matrix**: チップ内部のプログラム可能な配線盤。UART/SPI/TWAI等の信号をほぼ任意のGPIOへ。基板設計の失敗をソフトで救える。実験: 同じUARTを別ピンから出す
2. **ETM**: GPIOイベント→タイマー記録→周辺起動をCPU/割り込み無しで直結。「チップ内に小さなデジタル回路を構築する」感覚。実験: 超音波EchoのHigh区間をETM+タイマーで自動記録
3. **RMT**: 波形列（High xµs/Low yµs...）を記憶し正確に送受信。赤外線/WS2812/自作1線通信/「存在しない通信規格を波形から作れる」
4. **PCNT**: パルス計数+方向判定（A/B相）+上下限イベント+ノイズ除去をハードで。実験: ロータリーエンコーダ
5. **LP Core**: メインCPUがDeep Sleep中も動く小さなRISC-V。I2C照度センサを読み閾値で起こす公式例。「地下室でもう一人が働いていた」
6. **Dedicated GPIO**: GPIO操作専用のCPU命令（RISC-V CSR）。超高速ビット操作・独自通信
7. **MCPWM**: モーター制御工場。相補PWM+デッドタイム自動挿入（MOSFET上下短絡防止）、故障入力即停止、三相
8. **PARLIO**: 複数GPIOを束ねて自作データバス（4本なら1クロック4bit）。HUB75公式例
9. **DMA**: 「このメモリからこの周辺回路へ4096バイト運べ」→CPUは別の仕事へ。**単独で教えず「ADC→DMA→メモリ→処理task」のデータの流れとして教える**
10. **ADC連続+DMA**: 複数ch自動巡回・一定速度連続測定→DMA→一定量でCPUへ通知。簡易オシロの入口
11. **LEDCハードフェード**: 「2秒かけて目標へ」をハードが実行。CPU介入なしの連続フェード
12. **Sigma-Delta**: デジタルピンからPDM→RC平滑で疑似アナログ電圧
13. **GPIOグリッチフィルタ**: 短パルスをCPUに届く前に除去。「ノイズ対策=ソフトの待ち時間」というArduino的理解を壊す
14. **USB Serial/JTAG**: ケーブル1本で書き込み+コンソール+ブレークポイント/ステップ実行。「マイコンを途中停止して内部を見る」体験
15. **eFuse**: 一度焼いたら戻せない設定ビット。Secure Boot/Flash Encryption/Digital Signature（秘密鍵を読めないままRSA署名）。製品セキュリティの世界
16. **Wi-Fi=測定器**: Sniffer/生802.11/CSI。「Wi-Fiは空間を観測するセンサにもなる」
17. **ESP-NOW**: SSIDもIPもサーバも要らないWi-Fi（→第11部）
18. **Wi-Fi6+BLE+802.15.4同居**: 1チップで3世界の橋渡し、2.4GHz時分割共存（→第11部）
19. **Deep Sleepの区画設計**: 「どの区画へ電気を残すか」を設計する（→第12部）
20. **TWAI×2**: CANコントローラ2個。ID調停/ACK/エラーフレーム/Bus-Offまで踏み込むと「UARTと根本的に違う」が見える（→第8部）

## Arduino UNO対比の核（イントロ用）

- Arduino: 「入力を読む→判断する→出力する」をCPUが順番に実行
- ESP32: 「入力回路→計数回路→タイマー→PWM出力」という**ハードウェアの処理網を組み、CPUは設定だけして寝る**

## 出典（ドラフト原文に付されていたESP-IDF公式URL）

gpio/etm/gptimer/rmt/pcnt/ulp-lp-core/soc_caps/mcpwm/parlio/uart/adc_continuous/ledc/sdm/usb-serial-jtag-console/security/wifi-driver/esp_now/product-overview/coexist/sleep_modes/twai の各 docs.espressif.com esp32c6 ページ（執筆時に個別に再確認すること）
