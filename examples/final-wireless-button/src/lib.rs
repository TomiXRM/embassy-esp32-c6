//! final-wireless-button: 無線ボタン端末（最終プロジェクト）のライブラリ部分
//!
//! ESP-NOWで「ボタン端末（送信側）」と「受信端末（受信側）」をつなぐ
//! 最終プロジェクトです。送信側はBOOTボタンの押下イベントと500ms周期の
//! ハートビートを送り、受信側はACKを返しながらLEDへ状態を反映します。
//!
//! モジュール構成と依存方向（矢印の先に依存する）:
//!
//! ```text
//! main.rs / bin/receiver.rs ──> app（bin はクレート内では app のみに依存）
//! app ──> button / heartbeat / radio / protocol / config（全体の配線役）
//! radio ──> protocol / config / error（+ 状態通知の型 app::LinkState を引数で受領）
//! button / heartbeat ──> config / power（チャネルの「口」は app から引数で受領）
//! protocol ──> error（純粋関数のみ。ハードウェア非依存）
//! error / config / power ──> なし
//! ```
//!
//! protocol / error / config はハードウェアに依存しない純粋なモジュールで、
//! ホストPC上でも単体テストできます（Cargo.tomlの解説コメント参照）。

// テストビルド（ホストPC）のときだけstdを使い、それ以外はno_std。
#![cfg_attr(not(test), no_std)]

// ハードウェア非依存の純粋モジュール（ホストでもビルド・テスト可能）
pub mod config;
pub mod error;
pub mod protocol;

// ハードウェア依存モジュール（組み込みターゲットのときだけビルドする）
#[cfg(target_os = "none")]
pub mod app;
#[cfg(target_os = "none")]
pub mod button;
#[cfg(target_os = "none")]
pub mod heartbeat;
#[cfg(target_os = "none")]
pub mod power;
#[cfg(target_os = "none")]
pub mod radio;
