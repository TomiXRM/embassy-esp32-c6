//! error: このプロジェクトのエラー型
//!
//! 責務: 「何がどう失敗したか」を型で表す。ログや分岐で使う。
//! 依存方向: どのモジュールにも依存しない（純粋なenumのみ。ホストでもビルド可能）。

/// 受信したバイト列をパケットに解読（デシリアライズ）できなかった理由
//
// defmt::Format はハードウェアビルド（target_os = "none"）のときだけ derive する。
// このenumはradio.rsで `{:?}` としてログ出力されるため。ホスト向けテストビルドでは
// defmt は依存に入らないため derive しない（cargo test をそのまま動かすため）。
#[cfg_attr(target_os = "none", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// パケット長が仕様（PACKET_LEN）と違う
    BadLength,
    /// 先頭のマジックナンバーが違う（このプロトコルのパケットではない）
    BadMagic,
    /// チェックサム不一致（電波ノイズなどでデータが壊れている）
    BadChecksum,
    /// 未知のパケット種別
    UnknownKind,
}

/// イベント送信（ACK待ち + 再送）が最終的に失敗した理由
#[cfg_attr(target_os = "none", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxError {
    /// 送信自体（電波に乗せる処理）が毎回失敗した
    SendFailed,
    /// 送信はできたが、規定回数の再送でもACKが返らなかった
    AckTimeout,
}
