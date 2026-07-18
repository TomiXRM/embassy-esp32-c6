---
title: "7. 開発環境の構築"
description: rustupでRustを入れ、RISC-Vターゲットとespflashを追加して、ESP32-C6の開発環境を作ります。macOS/Windows/Linux共通の手順です。
part: 1
lesson: 7
difficulty: basic
estimated_minutes: 20
prerequisites:
  - part01/05-parts
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル（データ通信対応）
status: complete
code_status: none
last_verified: "2026-07-18"
sources:
  - https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html
  - https://rustup.rs/
  - https://github.com/esp-rs/espflash
  - https://github.com/esp-rs/esp-generate
---

## このページでできるようになること

- rustupでRust（stable）をインストールできる
- ESP32-C6用のビルドターゲットを追加できる
- 書き込みツールespflashとプロジェクト生成ツールesp-generateを導入できる
- インストールが成功したかを自分で確認できる

## 先に結論

やることは4つだけです。①rustupでRust本体を入れる、②`rustup target add riscv32imac-unknown-none-elf`でC6用のターゲットを足す、③`cargo install espflash`で書き込みツールを入れる、④`cargo install esp-generate`でプロジェクト生成ツールを入れる。ESP32-C6はRISC-VのCPUなので、**ふつうのstable Rustだけで開発できます**。特別なコンパイラは不要です。手順はmacOS・Windows・Linuxでほぼ共通です。

## 身近なたとえ

環境構築は「工作を始める前に、机に道具を並べる作業」です。Rustは工具セット、ターゲットはC6専用のドリルビット、espflashは完成品をボードへ運ぶ配達係です。

たとえと違うのは、道具がぜんぶ無料で、コマンド数行でそろうことです。一度そろえれば、この教材の最後まで同じ道具を使い続けます。

## 仕組み — 何を入れているのか

```mermaid
graph LR
  A[あなたのRustコード] -->|"rustc（Rust本体）"| B[riscv32imac用の実行ファイル]
  B -->|espflash| C[ESP32-C6のフラッシュ]
```

- **rustup**: Rust本体（コンパイラrustcとビルドツールcargo）を管理するインストーラです
- **ターゲット `riscv32imac-unknown-none-elf`**: 「RISC-V 32bit・OSなし」向けの機械語を出すための部品です。パソコン用のRustに、マイコン用の出力先を追加するイメージです
- **espflash**: できあがったプログラムをUSB経由でボードに書き込むツールです
- **esp-generate**: ESP32シリーズ用のプロジェクトのひな形を作るツールです（次のページで使います）

## 手順1: Rustを入れる（rustup）

すでにRustが入っている人は手順2へ進んでください（`rustc --version`で確認できます）。

**macOS / Linux**: ターミナルで次を実行します。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

質問には基本的にそのままEnter（標準インストール）で答えます。終わったら**ターミナルを開き直して**ください。

**Windows**: [rustup.rs](https://rustup.rs/) から `rustup-init.exe` をダウンロードして実行します。C++ビルドツールのインストールを求められたら、画面の指示に従って入れてください。

インストールされるのは**stable**版のRustです。この教材はstable版だけで進みます。

## 手順2: C6用ターゲットを追加する

```bash
rustup target add riscv32imac-unknown-none-elf
```

これはOS共通です。名前の意味は「riscv32imac＝RISC-V 32bitでIMAC拡張あり」「none＝OSなし」「elf＝実行ファイル形式」です。

## 手順3: espflashを入れる

```bash
cargo install espflash
```

`cargo install`はRust製ツールをソースからビルドして入れるので、数分かかることがあります。エラーが出ずに終われば成功です。

## 手順4: esp-generateを入れる

```bash
cargo install esp-generate
```

これも数分かかります。次のページでプロジェクトを作るときに使います。

## 実行方法 — インストール確認

4つのコマンドで確認します。バージョン番号が表示されれば成功です。

```bash
rustc --version
rustup target list --installed
espflash --version
esp-generate --version
```

```text
rustc 1.97.1 （例。これ以降のstableなら可）
riscv32imac-unknown-none-elf を含む一覧
espflash 4.5.0
esp-generate 1.3.0
```

## OSごとの注意

- **macOS**: 初回に`xcode-select --install`（コマンドラインツール）を求められることがあります。指示に従ってください
- **Windows**: ボードのUARTポート（CP2102Nチップ経由）が認識されない場合は、Silicon Labs公式のCP210xドライバを入れます。USB(SERIAL)側のポートを使う場合はドライバ不要なことが多いです
- **Linux**: シリアルポートを使う権限が必要です。多くのディストリビューションでは`dialout`グループ（Arch系は`uucp`）に自分のユーザーを追加し、再ログインします

```bash
sudo usermod -a -G dialout $USER
```

## よくある失敗

- **`cargo: command not found`（コマンドが見つからない）**: rustupの直後はPATH（コマンドの検索経路）がまだ反映されていません。ターミナルを開き直すと直ります
- **`cargo install espflash`が途中で失敗する**: リンカやCコンパイラが無い環境で起きます。macOSはコマンドラインツール、Windowsはビルドツール、Linuxは`build-essential`相当（gcc, pkg-config, libudev-dev等）を入れてから再実行してください
- **espflashは入ったのにボードが見えない**: 環境構築の問題ではなく、ケーブルが充電専用の可能性が高いです。[5. 必要な部品](/embassy-esp32-c6/part01/05-parts/)で確認した「データ通信対応ケーブル」を使ってください

## やってみよう

上の確認コマンド4つを実行して、表示されたバージョンをメモしておきましょう。教材のコードが動かないときに、まずここを見比べると原因を絞れます。

## 確認問題

1. ESP32-C6の開発に、特別版ではなく**stable Rust**が使えるのはなぜですか。
2. `riscv32imac-unknown-none-elf`の「none」は何が「ない」という意味ですか。

<details>
<summary>答え</summary>

1. ESP32-C6のCPUが標準的なRISC-Vアーキテクチャだからです。Rust本体（stable）がRISC-V向けのコード生成を最初からサポートしています（旧ESP32のXtensaコアでは専用ツールチェーンが必要でした）。
2. OSがない、という意味です。生成されるプログラムはOSの助けなしにマイコン上で直接動きます。

</details>

## まとめ

- rustup（stable）→ ターゲット追加 → `cargo install espflash` → `cargo install esp-generate` の4手順
- C6はRISC-VなのでstableのRustだけで開発できる
- 確認は`--version`系コマンド。Linuxはシリアルポートの権限に注意

## 次のページ

道具がそろいました。次はesp-generateを使って、実際にESP32-C6用のRustプロジェクトを作り、中身のファイルをひとつずつ見ていきます。

- 前: [6. 電圧と電流の最低限](/embassy-esp32-c6/part01/06-volt-current/)
- 次: [8. Rustプロジェクトの作成](/embassy-esp32-c6/part01/08-new-project/)
