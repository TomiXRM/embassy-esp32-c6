# ESP32-C6 × Rust × Embassy 教科書

ArduinoでLチカ経験がある人が、ESP32-C6をRustとEmbassyで動かせるようになるまでを、中学生でも読める日本語で学べる教科書サイトです。

**公開サイト: https://tomixrm.github.io/embassy-esp32-c6/**

> Arduinoでは1つのloop関数に全部書いていた人が、RustとEmbassyを使って、複数の機能を安全に分割できるようになる。

## 構成

| ディレクトリ | 内容 |
|---|---|
| `site/` | Astro Starlight製の教科書サイト（全12部・120ページ + 付録） |
| `examples/` | 教材で使うRustサンプルコード（cargoワークスペース、全14プロジェクト） |
| `docs/project/` | カリキュラム・執筆ルール・バージョン固定表・技術対応状況表・進捗 |
| `docs/research/` | 公式資料に基づく技術調査資料（ESP32-C6ハード / esp-rs / Embassy） |

## 技術構成（2026-07-18固定）

- 対象: ESP32-C6-DevKitC-1 / no_std
- esp-hal 1.1.1 + esp-rtos 0.3.0 + esp-radio 0.18.0
- embassy-executor 0.10 / embassy-time 0.5 / embassy-net 0.9 / trouble-host 0.6
- Rust stable、target `riscv32imac-unknown-none-elf`、書き込みは espflash

詳細は [docs/project/versions.md](docs/project/versions.md) を参照。

## サイトをローカルで動かす

```bash
cd site
npm install
npm run dev
```

## サンプルコードを検証する

```bash
rustup target add riscv32imac-unknown-none-elf
cd examples
cargo check --workspace
```

実機へ書き込む場合（例: Lチカ）:

```bash
cargo install espflash
cd examples
cargo run -p blinky --release
```

## ライセンス・注記

- 教材本文は独自に執筆したものです。技術的事実は各ページのfrontmatter `sources` に示した公式資料に基づきます
- コードの検証状態（cargo-check-passed / hardware-tested等）は各ページとFINAL_REPORT.mdに明示しています
