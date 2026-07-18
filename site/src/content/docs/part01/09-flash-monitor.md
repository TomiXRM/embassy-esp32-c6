---
title: "9. 書き込みとログ表示"
description: probe-rsでESP32-C6にプログラムを書き込み、defmtのログを見る方法を学びます。代替のespflashや、書き込みに失敗したときの対処も説明します。
part: 1
lesson: 9
difficulty: basic
estimated_minutes: 15
prerequisites:
  - part01/08-new-project
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル（データ通信対応）
status: complete
code_status: none
last_verified: "2026-07-18"
sources:
  - https://probe.rs/
  - https://github.com/esp-rs/espflash
  - https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html
---

## このページでできるようになること

- probe-rsでプログラムをボードへ書き込める
- defmtのログをprobe-rs越しに読める
- 代替のespflashに切り替えて書き込める
- ボードの2つのUSBポートの違いが分かる
- 書き込みに失敗したときにBOOTボタンで復旧できる

## 先に結論

書き込みは`probe-rs run`で行いますが、前のページの設定のおかげで、実際に打つのは`cargo run -p blinky`だけです（自作の1プロジェクトなら`cargo run`だけ）。ビルド→書き込み→ログ表示まで一気に進みます。ログはこれまでのプレーンなシリアル出力ではなく、**defmtという軽量な仕組みでRTT経由で送られ、probe-rsが読み取って表示します**。espflashを使いたい場合は`--no-default-features --features espflash`を付けて切り替えます。書き込みに失敗するときは、BOOTボタン（GPIO9）を押しながらリセットすると「ダウンロードモード（書き込みモード）」に入れます。

## 身近なたとえ

書き込み（フラッシュへの書き込み）は「マイコンの本棚に、新しい手順書を差し替える作業」です。マイコンは電源が入るたびに、本棚（フラッシュメモリ）の手順書を最初から読んで実行します。ログ表示は、マイコンが作業しながら書く「日報」をパソコン側でのぞく窓です。

たとえと違うのは、差し替え中は古い手順書が消され、途中で電源を抜くと中途半端な状態になることです。書き込み中はケーブルを抜かないでください。

## 仕組み — 書き込みとログの流れ

```mermaid
graph LR
  A["cargo build<br>（コンパイル）"] --> B["probe-rs run<br>（USB経由で書き込み）"]
  B --> C["リセット後、<br>新プログラムが起動"]
  C --> D["defmtログを表示<br>（RTT経由でprobeが読む）"]
```

`cargo run`は、この一連の流れを自動で行います。`.cargo/config.toml`の`runner = "probe-rs run --chip=esp32c6 ..."`という設定が効いているためです。

### defmtとRTT — ログの新しい出口

これまでのようにログを人が読める文字列としてシリアル線へ流すのではなく、この教材では**defmt**という仕組みを使います。defmtはログを小さな番号に圧縮してRTT（Real-Time Transfer、デバッグ用の高速な通り道）へ流し、パソコン側のprobe-rsが番号を元の文章に戻して表示します。おかげでマイコン側の負担が軽く、ログが速いのが利点です。コードでは`defmt::info!`のように書くだけで、難しい設定は要りません（内部の詳しい話は今は不要です）。

### 2つのUSBポート

ESP32-C6-DevKitC-1にはUSB-Cポートが2つあります。

| ポートの刻印 | 中身 | 特徴 |
|---|---|---|
| UART | CP2102Nチップ経由のシリアル変換（UART0 = GPIO16/17） | 昔ながらの確実な経路。Windowsはドライバが必要なことがある |
| USB | チップ内蔵のUSB Serial/JTAG（GPIO12/13） | 変換チップを通らない直結。ドライバ不要なことが多い |

**probe-rsは内蔵のUSB Serial/JTAG（「USB」刻印側）を使います**。probe-rsで書き込む場合はこちらに挿してください。espflashはどちらのポートでも書き込めます。迷ったら**USB側**につなぐのが確実です。

## 実行方法

プロジェクトのフォルダ（教材のexamplesワークスペースなら任意の場所）で実行します。

```bash
cargo run -p blinky
```

自分で作った1プロジェクトの中なら、パッケージ指定は不要で`cargo run`だけで動きます。書き込みの進行表示のあと、defmtのログが流れます。前のページで作ったプロジェクト（または教材の`examples/01-blinky`）なら、次のようなdefmt形式の行が出ます。

```text
INFO  Lチカを開始します
```

表示を終了するにはCtrl+Cを押します。マイコン側のプログラムは動き続けます（ログ表示は「日報をのぞく窓」なので、閉じても作業は止まりません）。

ログの量は環境変数`DEFMT_LOG`で変わります。`.cargo/config.toml`に`DEFMT_LOG = "info"`と設定済みなので、`info!`以上のログが表示されます。

### 代替: espflashで書き込む

probe-rsの代わりにespflashを使う場合は、feature（クレートの機能スイッチ）を切り替えて実行します。

```bash
cargo run -p blinky --no-default-features --features espflash
```

`--no-default-features`で既定のprobe-rs用の出口を外し、`--features espflash`でespflash用の出口（esp-println）に差し替えます。あわせて`.cargo/config.toml`のrunnerをespflash用の行に変えます。

```toml
runner = "espflash flash --monitor --chip esp32c6 --log-format defmt"
```

espflashの場合もログはdefmt形式です（`--log-format defmt`でespflashが復号します）。ログはシリアル経由で流れ、`INFO  Lチカを開始します`のように表示されます。

## 書き込みモードとBOOTボタン

ふだんは自動でボードが「ダウンロードモード（書き込みモード）」に切り替わります。まれに失敗するときや、ボードが認識されないときは手動で入れます。

1. **BOOTボタン**（GPIO9につながったボタン）を押したままにする
2. そのまま**RSTボタン**を短く押して離す
3. BOOTボタンを離し、もう一度`cargo run -p blinky`

これはGPIO9のレベルで起動モードが決まるというC6の仕組みを使った操作です。BOOTを押しながらリセットすると、プログラムを実行する代わりに書き込み待ち状態（ダウンロードモード）で起動します。ボードがパソコンに認識されないときも、この操作でダウンロードモードに入れると書き込めることがあります。

## よくある失敗

- **ボードが認識されない・`No probe found`などでprobeが見つからない**: ①「USB」刻印側（内蔵USB Serial/JTAG）に挿しているか、②ケーブルがデータ通信対応か、③Linuxならポートの権限（[7. 開発環境の構築](/embassy-esp32-c6/part01/07-setup/)参照）、の順で確認します。それでも見えなければ、上のBOOTボタンの手順で**ダウンロードモード（BOOT押しながらRESET）**に入れてから再実行してください
- **書き込みが途中で`Connection failed`などになる**: 上のBOOTボタンの手順で書き込みモードに入れてから再実行してください。USBハブを介している場合は直挿しも試します
- **ログが何も出ない**: 使っているツールとログの出口が食い違っている可能性があります。probe-rs（既定）はdefmt/RTTのログを表示し、esp-printlnのプレーンなシリアル出力は表示しません。逆にespflashに切り替えたのにrunnerがprobe-rsのままだと表示されません。ツールとfeatureの組み合わせを見直してください
- **2回目の実行で「ポートが開けない」エラー**: 前のログ表示が開いたままです。前のターミナルでCtrl+Cを押して閉じてください

## やってみよう

書き込みが終わってログが表示されている状態で、ボードのRSTボタンを押してみてください。起動時のログが最初から流れ直すはずです。「リセット＝本棚の手順書を最初から読み直す」を体感できます。

## 確認問題

1. `cargo run -p blinky`一発で書き込みとログ表示まで進むのは、どのファイルのどの設定のおかげですか。
2. BOOTボタンを押しながらリセットすると何が起きますか。
3. ログ表示を閉じると、マイコンのプログラムは止まりますか。

<details>
<summary>答え</summary>

1. `.cargo/config.toml`の`runner = "probe-rs run --chip=esp32c6 ..."`という設定です。`cargo run`がビルド後にこのコマンドを実行します。
2. GPIO9がLowの状態で起動するため、プログラムを実行せず「ダウンロードモード（書き込み待ちモード）」で立ち上がります。書き込みが失敗するときやボードが認識されないときの復旧手段です。
3. 止まりません。ログ表示は文字を表示しているだけで、プログラムはマイコン上で独立して動き続けます。

</details>

## まとめ

- 書き込み＋ログ表示は`cargo run -p blinky`一発（実体は`probe-rs run`）。ログはdefmtをRTT経由でprobe-rsが読む
- espflashで書き込むときは`--no-default-features --features espflash`＋runnerをespflash用に差し替え
- probe-rsは内蔵USB Serial/JTAG（「USB」刻印側）を使う。espflashはどちらのポートでも可
- 書き込めない・認識されないときは、BOOT（GPIO9）を押しながらRST→BOOTを離してダウンロードモードで再実行

## 次のページ

書き込みとログの確認ができるようになりました。次はいよいよ、LEDを配線してLチカを動かし、コードを一行ずつ読み解きます。

- 前: [8. Rustプロジェクトの作成](/embassy-esp32-c6/part01/08-new-project/)
- 次: [10. 最初のLチカ](/embassy-esp32-c6/part01/10-blinky/)
