---
title: "9. テストと保守"
description: マイコン上では動かないcargo testを、純粋ロジックの分離とホストターゲット指定で活用する方法を学びます。
part: 12
lesson: 9
difficulty: advanced
estimated_minutes: 20
prerequisites:
  - part12/08-project-structure
  - part05/01-no-std
hardware:
  - なし（このページはホストPCだけで完結します）
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1（テストはホストターゲットで実行）"
last_verified: "2026-07-18"
sources:
  - https://doc.rust-lang.org/book/ch11-00-testing.html
  - https://doc.rust-lang.org/cargo/commands/cargo-test.html
---

## このページでできるようになること

- no_stdターゲットで`cargo test`が動かない理由を説明できる
- ハードウェア非依存のロジックを分離してホストPCでテストできる
- 「テスト可能な設計 = 依存の少ない設計」という関係を説明できる

## 先に結論

ESP32-C6の上では`cargo test`は動きません。no_stdターゲットにはテストを数えて実行し結果を表示するテストランナー（std前提の仕組み）がないからです。しかし、あきらめる必要はありません。**ハードウェアに依存しないロジックはホストPC向けにビルドすればテストできます**。最終プロジェクトのprotocol・error・configはそのために分離されており、ホストターゲットを指定した`cargo test`で**10件のテストが実際に通ります**（本リポジトリで確認済み）。パケットの変換や重複判定のような「純粋な計算」をハードウェアから切り離すことは、テストのテクニックである以前に、前ページで学んだ依存設計そのものです。

## 身近なたとえ

料理で言えば、「調味料の分量計算」と「火加減」を分けておくことです。分量計算は紙と電卓があれば家でも検算できますが、火加減はキッチンに立たないと確かめられません。レシピ全体を「キッチンでしか確認できないもの」にしてしまうと、検算のたびに調理が必要になります。計算部分を切り出してあれば、間違いの大半は台所に立つ前に潰せます。

たとえと違うのは、ソフトウェアではこの分離を**cfg属性とCargo.tomlで機械的に強制できる**ことです。「うっかり火加減の話が計算に混ざる」と、ホスト向けビルドが通らなくなって気づけます。

## 仕組み

### なぜマイコン上でcargo testが動かないのか

`cargo test`は、テスト関数を集めて実行し「何件通った」を表示するプログラム（テストランナー)を作って走らせます。この仕組みはstd（OSのある環境）を前提としています。riscv32imac-unknown-none-elfのようなno_stdターゲットにはOSも標準出力もないため、この形のテストは走りません。

そこで発想を変えます。**テストしたいのはハードウェアではなくロジック**です。「8バイトのパケットを正しく作れるか」「壊れたパケットを弾けるか」「重複を見分けられるか」——これらはCPUがRISC-VでもApple SiliconでもX86でも答えが同じ、純粋な計算です。ならばホストPC向けにビルドして走らせればいいのです。

### 分離を支える3点セット

final-wireless-buttonでは次の3つがこの分離を実現しています（[前のページ](/embassy-esp32-c6/part12/08-project-structure/)の構造の続きです）。

1つ目、src/lib.rsの条件付きコンパイルです。

```rust
// テストビルド（ホストPC）のときだけstdを使い、それ以外はno_std。
#![cfg_attr(not(test), no_std)]

// ハードウェア非依存の純粋モジュール（ホストでもビルド・テスト可能）
pub mod config;
pub mod error;
pub mod protocol;

// ハードウェア依存モジュール（組み込みターゲットのときだけビルドする）
#[cfg(target_os = "none")]
pub mod app;
```

2つ目、Cargo.tomlでハードウェア依存クレートを組み込みターゲット限定にします。

```toml
# ハードウェア依存クレートは「組み込みターゲット(target_os = "none")のときだけ」有効にする
[target.'cfg(target_os = "none")'.dependencies]
esp-hal.workspace = true
esp-radio = { workspace = true, features = ["wifi", "esp-now", "unstable"] }
# ...
```

3つ目、純粋モジュール自身が規律を守ること。protocol.rsが使うのは`core`と自分のerror型だけで、esp-halもEmbassyも一切useしていません。

### テストは仕様書になる

src/protocol.rsのテストから2本抜粋します（完全なコードはexamples/final-wireless-button/src/protocol.rsを見てください）。

```rust
    #[test]
    fn rejects_corrupted_payload() {
        let mut bytes = Packet::Event { seq: 1000 }.to_bytes();
        bytes[3] ^= 0xFF; // 電波ノイズによる1バイト破損を模擬
        assert_eq!(Packet::from_bytes(&bytes), Err(DecodeError::BadChecksum));
    }

    #[test]
    fn dedup_detects_resent_seq() {
        let mut table: DedupTable<4> = DedupTable::new();
        let mac = [1, 2, 3, 4, 5, 6];
        assert!(table.check_and_update(&mac, 1)); // 新規
        assert!(!table.check_and_update(&mac, 1)); // 再送 → 重複
        assert!(table.check_and_update(&mac, 2)); // 次のseq → 新規
    }
```

「ノイズで1バイト壊れたパケットはチェックサムで弾かれる」「同じseqの再送は重複と判定される」——テストはそのまま、日本語の仕様の言い換えになっています。電波ノイズを実機で狙って起こすのはほぼ不可能ですが、ホストテストなら`bytes[3] ^= 0xFF`の1行で毎回確実に再現できます。

## 実行方法

ホストPCのターゲットを明示して実行します。Apple Siliconの場合は次の通りです。

```bash
cargo test -p final-wireless-button --lib --target aarch64-apple-darwin
```

Intel Macなら`x86_64-apple-darwin`、Linuxなら`x86_64-unknown-linux-gnu`を指定します（自分のホストターゲット名は`rustc -vV`の`host:`行で確認できます）。期待される結果:

```text
running 10 tests
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

パケットの往復変換・破損検出・重複判定など10件のテストが数秒で終わります。ボードへの書き込みは一切不要です。

## よくある失敗

1. **`--target`を付け忘れる** — examples/.cargo/config.tomlの既定ターゲットがriscv32imac-unknown-none-elfなので、そのまま`cargo test`すると組み込みターゲット向けにテストをビルドしようとして失敗します。ホストのターゲットを明示するのが確実です
2. **純粋モジュールにハードウェア依存を混ぜる** — protocol.rsに`use esp_hal::...`を1行足しただけで、ホスト向けビルドが通らなくなります。逆に言えば、テストが通り続けている限り分離は守られています（コンパイラが番人になる）
3. **「実機がないとテストできない」と思い込む** — 実機でしか確かめられないのは配線・電気・タイミングです。判断ロジックの間違い（チェックサム計算のバグ、境界値の扱い）は、ホストテストの方が速く確実に見つかります

## やってみよう

src/protocol.rsのtestsモジュールに、自分のテストを1本足してみましょう。例えば「`seq: 0`のHeartbeatが往復変換できる」テストです。書けたら上のコマンドで11件になることを確認してください。5分でテストを増やせる体験そのものが、この構造の価値です。

## 確認問題

1. ESP32-C6上で`cargo test`が動かないのはなぜですか。
2. protocolモジュールをホストでテストできるようにするために、Cargo.tomlで行っている工夫は何ですか。
3. 「テスト可能な設計 = 依存の少ない設計」と言えるのはなぜですか。

<details>
<summary>答え</summary>

1. `cargo test`のテストランナーはstd（OSのある環境）を前提としており、no_stdターゲットにはそれを動かす仕組み（OS・標準出力など）がないからです。
2. esp-halなどハードウェア依存クレートを`[target.'cfg(target_os = "none")'.dependencies]`に置き、組み込みターゲットのビルドでだけ有効にしていることです（lib.rs側の`#[cfg(target_os = "none")]`によるモジュール切り替えとセット）。
3. あるコードをホストで動かすには、そのコードが依存するものすべてがホストでも成立する必要があるからです。esp-halに依存した瞬間ホストでは動かせません。依存が少ない（純粋な）コードほど、そのまま切り出してテストできます。

</details>

## まとめ

- no_stdターゲットにテストランナーはない。だから「ロジックをホストでテストする」に発想を切り替える
- 分離の3点セット: `cfg_attr(not(test), no_std)`＋モジュールのcfg分け＋ターゲット条件付き依存。final-wireless-buttonでは10件のテストがホストで通る
- テストできる形に切り出せるかどうかは、依存設計の健全さをそのまま映す鏡

## 次のページ

部品はすべて揃いました。仕様から設計、実装、そして2台での実演まで——最終プロジェクト「無線ボタン端末」を完成させます。

[10. 最終プロジェクト — 無線ボタン端末 →](/embassy-esp32-c6/part12/10-final-project/)

---

前: [8. プロジェクトの分割](/embassy-esp32-c6/part12/08-project-structure/) | 次: [10. 最終プロジェクト — 無線ボタン端末](/embassy-esp32-c6/part12/10-final-project/)
