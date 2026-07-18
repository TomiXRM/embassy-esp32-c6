---
title: "2. Arduinoからの対応表"
description: Arduinoの関数や書き方が、RustとEmbassyでは何に対応するのかを一覧と解説でまとめます。
status: complete
---

Arduinoでの書き方が、RustとEmbassyでは何に置き換わるのかをまとめました。ただし大事なのは「名前の置き換え」ではありません。多くの項目で**設計の考え方そのものが変わる**ので、表のあとに各行の解説を付けています。

Arduinoが劣っているという話ではありません。Arduinoは「すぐ動かせること」を最優先に設計された優れた環境です。RustとEmbassyは「大きくなっても壊れにくいこと」を優先しており、得意分野が違うのだと考えてください。

## 対応表

| Arduino | RustとEmbassy | くわしい章 |
|---|---|---|
| `setup()` | `main`の先頭で行う初期化（`esp_hal::init`など） | [最初のLチカ](/embassy-esp32-c6/part01/10-blinky/) |
| `loop()` | 各task内の`loop { ... }` | [loopと無限ループ](/embassy-esp32-c6/part02/07-loop/) |
| `delay(500)` | `Timer::after_millis(500).await` | [Timerで待つ](/embassy-esp32-c6/part06/07-timer/) |
| `digitalWrite(pin, HIGH)` | `Output`型の`set_high()` / `set_low()` / `toggle()` | [GPIO出力](/embassy-esp32-c6/part06/01-gpio-output/) |
| `digitalRead(pin)` | `Input`型の`is_high()` / `is_low()` | [GPIO入力](/embassy-esp32-c6/part06/02-gpio-input/) |
| `Serial.println(...)` | `log`の`info!(...)`（esp-println経由）、または`Uart`ドライバ | [書き込みとシリアル表示](/embassy-esp32-c6/part01/09-flash-monitor/)、[UART基礎](/embassy-esp32-c6/part08/01-uart-basics/) |
| `Wire`（I2C） | esp-halの`I2c`ドライバ | [I2C基礎](/embassy-esp32-c6/part08/03-i2c-basics/) |
| `SPI` | esp-halの`Spi`ドライバ | [SPI基礎](/embassy-esp32-c6/part08/06-spi-basics/) |
| `attachInterrupt(...)` | `Input`の`wait_for_falling_edge().await`などのasync待ち | [GPIO割り込みとasync wait](/embassy-esp32-c6/part06/06-gpio-interrupt/) |
| `millis()` | `Instant::now()`と`Duration` | [EmbassyのTimerとInstant](/embassy-esp32-c6/part09/06-embassy-time/) |
| グローバル変数 | 所有者を1つに決めた変数、または`static`（`StaticCell`等） | [所有権 — 誰がデータを持つのか](/embassy-esp32-c6/part03/08-ownership/)、[staticとstatic_cell](/embassy-esp32-c6/part05/06-static/) |
| コールバック関数 | async taskと`Channel`/`Signal`によるtask間通信 | [task — 仕事を分割する](/embassy-esp32-c6/part09/04-task/)、[Channel・Signal・Mutex](/embassy-esp32-c6/part09/09-channel-signal-mutex/) |

## 各行の解説 — 何が同じで、何が違うのか

### setup() → mainの先頭で初期化

Arduinoでは`setup()`という決まった関数に初期化を書きます。Rustでは`main`の先頭で`esp_hal::init(config)`を呼び、戻り値として**ペリフェラル一式の所有権**を受け取ります。単に場所が変わっただけではなく、「初期化するとピンや周辺機能が『値』として手に入り、その値を持っている人だけが使える」という所有権の考え方が加わります。同じピンを二重に使うミスは、この時点でコンパイルエラーになります。

### loop() → task内のloop

Arduinoの`loop()`は1本だけで、すべての仕事をこの中に詰め込みます。Embassyでは仕事ごとにtaskを作り、それぞれが自分の`loop`を持ちます。「LED点滅のloop」「ボタン監視のloop」を別々に書けるので、1本のループに処理を詰め込んで順番をやりくりする必要がなくなります。

### delay() → Timer::after

見た目はよく似ていますが、中身は正反対です。`delay(500)`はCPUを500ミリ秒**独り占めして**待ちます。その間は他の処理が一切できません。`Timer::after_millis(500).await`は「500ミリ秒後に起こして」と頼んで**CPUを他のtaskへゆずります**。この違いがあるからこそ、Embassyでは複数の仕事を1つのCPUで並行に進められます。周期をずらしたくない繰り返しには`Ticker::every`という専用の道具もあります。

### digitalWrite / digitalRead → Output型 / Input型

Arduinoではピンを「番号」で指定し、`pinMode`で向きを設定します。番号を打ち間違えても、入力のピンに出力しても、コンパイルは通ってしまいます。Rustではピンから`Output`や`Input`という**型**を作ります。出力用の型には`set_high()`があり、入力用の型には`is_high()`があります。「入力ピンに書き込む」という間違いは、そもそも書ける場所がないので起こりません。

### Serial.println → log(info!)またはUart

デバッグ表示には`log`クレートの`info!`マクロを使うのが基本です。esp-printlnが裏で表示先へ届けてくれます。Arduinoの`Serial.begin(115200)`のような速度設定は、ログ用途なら不要です。外部の機器とシリアル通信をしたい場合は、ログとは別にesp-halの`Uart`ドライバを使います。「人間向けのログ」と「機器向けの通信」を別の道具として区別するのがRust流です。

### Wire → I2c

役割はほぼ同じですが、エラーの扱いが違います。Arduinoの`Wire`は通信に失敗しても気づかず進んでしまう書き方になりがちです。esp-halの`I2c`は読み書きの結果を`Result`型で返すため、「センサが返事をしなかった」ことをコンパイラが無視させてくれません。失敗の処理を書くことが、言語の仕組みとして促されます。

### SPI → Spi

こちらも役割は同じで、モード（0〜3）やクロック速度を設定して使う点も共通です。違いは設定の渡し方で、esp-halでは設定を専用の型で組み立ててドライバに渡します。また、embedded-halのtraitに沿っているため、SPI用に書かれた汎用ドライバ（ディスプレイ用など）をそのままつなげます。

### attachInterrupt → async wait

Arduinoでは割り込みで呼ばれる関数（ISR）を登録し、その中では「短い処理だけ」「`volatile`変数で本体へ知らせる」といった注意点を自分で守る必要があります。Embassyでは`button.wait_for_falling_edge().await`と、**ふつうの手続きのように**「押されるまで待つ」と書けます。割り込みは裏側で使われていて、危険な部分はフレームワークが引き受けてくれます。

### millis() → InstantとDuration

`millis()`はただの数値（ミリ秒）を返すため、単位の取り違えや、引き算の順序ミスが起こりがちです。Embassyの`Instant::now()`は「時刻」を、時刻同士の差は`Duration`（時間の長さ）を返します。時刻と時間の長さが別の型なので、「時刻に時刻を足す」ような意味のない計算はコンパイルエラーになります。

### グローバル変数 → 所有者を決める、またはstatic

Arduinoでは`loop()`と割り込みの間で情報を共有するためにグローバル変数を多用します。どこからでも書けるため、大きくなると「誰がいつ書き換えたのか」が追えなくなります。Rustではまず「このデータの持ち主はどのtaskか」を決め、持ち主へムーブするのが基本です。どうしても共有が必要なら`static`と`StaticCell`、あるいは`Mutex`付きの`static`を使い、**共有することをコードに明示**します。

### コールバック → async taskとChannel

Arduinoやその他の環境では「イベントが起きたらこの関数を呼んで」というコールバック方式が定番です。処理が増えるとコールバックの中からコールバックを呼ぶ形になり、流れが追いにくくなります。Embassyでは、イベントを待つ側を1つのtaskとして書き、結果を`Channel`の`send`/`receive`や`Signal`で別のtaskへ渡します。処理の流れが上から下へ読める形のまま、並行動作を実現できるのが大きな違いです。

## 関連ページ

- [用語集](/embassy-esp32-c6/appendix/glossary/)
- [トラブルシューティング](/embassy-esp32-c6/appendix/troubleshooting/)
- [ArduinoからRustへ移る理由](/embassy-esp32-c6/part01/02-why-rust/)
