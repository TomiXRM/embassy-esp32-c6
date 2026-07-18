//! protocol: 無線ボタン端末の通信プロトコル（パケット定義と変換）
//!
//! 責務: パケット型（Event/Heartbeat/Ack）、シーケンス番号の扱い、
//!       バイト列との相互変換（シリアライズ/デシリアライズ）、重複判定表。
//! 依存方向: error（エラー型）のみに依存。ハードウェアには一切依存しない
//!           純粋な関数だけなので、ホストPCでも単体テストできます。
//!
//! パケット形式（固定長8バイト。serdeなどは使わず手書きで組み立てる）:
//!
//! ```text
//! +--------+--------+-------------------+--------+----------+
//! | byte 0 | byte 1 | bytes 2..6        | byte 6 | byte 7   |
//! | MAGIC  | 種別   | seq (u32, LE)     | フラグ | チェック |
//! | 0xB7   | 1/2/3  | 通し番号          | 0/1    | サム     |
//! +--------+--------+-------------------+--------+----------+
//! ```
//!
//! - 種別: 1=Event（ボタン押下）, 2=Heartbeat（生存確認）, 3=Ack（受信確認）
//! - seq: 送信側が全パケット共通で1ずつ増やす通し番号。受信側はこれで
//!        重複（再送でダブって届いたパケット）を見分け、Ackはこの番号を返す
//! - フラグ: Heartbeatでは「ボタンが今押されているか」(1=押下中)。他は0
//! - チェックサム: 先頭7バイトのXOR。壊れたパケットを捨てるための簡易検査

use crate::error::DecodeError;

/// このプロトコルのパケットである印（マジックナンバー）
pub const MAGIC: u8 = 0xB7;

/// パケットの固定長（バイト）
pub const PACKET_LEN: usize = 8;

const KIND_EVENT: u8 = 1;
const KIND_HEARTBEAT: u8 = 2;
const KIND_ACK: u8 = 3;

/// やり取りするパケットの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packet {
    /// ボタンが押された（送信側 → 受信側。ACK必須・再送あり）
    Event { seq: u32 },
    /// 生存確認 + 現在のボタン状態（送信側 → 受信側。500ms周期・再送なし）
    Heartbeat { seq: u32, pressed: bool },
    /// 受信確認（受信側 → 送信側。seqは受け取ったパケットの番号）
    Ack { seq: u32 },
}

impl Packet {
    /// このパケットのシーケンス番号を取り出す
    pub fn seq(&self) -> u32 {
        match self {
            Packet::Event { seq } => *seq,
            Packet::Heartbeat { seq, .. } => *seq,
            Packet::Ack { seq } => *seq,
        }
    }

    /// パケットを固定長バイト列へ変換する（シリアライズ）
    pub fn to_bytes(&self) -> [u8; PACKET_LEN] {
        let (kind, flag) = match self {
            Packet::Event { .. } => (KIND_EVENT, 0),
            Packet::Heartbeat { pressed, .. } => (KIND_HEARTBEAT, *pressed as u8),
            Packet::Ack { .. } => (KIND_ACK, 0),
        };
        let mut buf = [0u8; PACKET_LEN];
        buf[0] = MAGIC;
        buf[1] = kind;
        buf[2..6].copy_from_slice(&self.seq().to_le_bytes());
        buf[6] = flag;
        buf[7] = xor_checksum(&buf[..7]);
        buf
    }

    /// バイト列をパケットへ戻す（デシリアライズ）。
    /// 壊れたデータや別プロトコルのパケットはErrで弾く
    pub fn from_bytes(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() != PACKET_LEN {
            return Err(DecodeError::BadLength);
        }
        if data[0] != MAGIC {
            return Err(DecodeError::BadMagic);
        }
        if xor_checksum(&data[..7]) != data[7] {
            return Err(DecodeError::BadChecksum);
        }
        let seq = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
        match data[1] {
            KIND_EVENT => Ok(Packet::Event { seq }),
            KIND_HEARTBEAT => Ok(Packet::Heartbeat {
                seq,
                pressed: data[6] != 0,
            }),
            KIND_ACK => Ok(Packet::Ack { seq }),
            _ => Err(DecodeError::UnknownKind),
        }
    }
}

/// 先頭バイト列のXORチェックサム（簡易的な破損検出）
fn xor_checksum(bytes: &[u8]) -> u8 {
    let mut sum = 0u8;
    let mut i = 0;
    while i < bytes.len() {
        sum ^= bytes[i];
        i += 1;
    }
    sum
}

/// 受信側の重複判定表。
/// 「送信元MACアドレスごとに最後に受け取ったseq」を覚えておき、
/// 同じseqがもう一度届いたら「ACKが失われて再送されたもの」と判定する。
/// これも純粋なデータ構造なのでホストでテストできる。
pub struct DedupTable<const N: usize> {
    /// (送信元MACアドレス, 最後に受け取ったseq)
    entries: [Option<([u8; 6], u32)>; N],
}

impl<const N: usize> DedupTable<N> {
    /// 空の表を作る
    pub const fn new() -> Self {
        Self { entries: [None; N] }
    }

    /// 受信したパケットが「新規」ならtrueを返し、表を更新する。
    /// 記憶しているseqと同じなら重複（再送）なのでfalseを返す。
    /// 注意: 「同じseqだけ」を重複とみなす単純な方式なので、送信側が
    /// 再起動してseqが巻き戻っても新規として扱われる（教材向けの簡略化）。
    pub fn check_and_update(&mut self, mac: &[u8; 6], seq: u32) -> bool {
        // 既に知っている送信元か探す
        for slot in self.entries.iter_mut() {
            if let Some((known_mac, last_seq)) = slot {
                if known_mac == mac {
                    if *last_seq == seq {
                        return false; // 同じseq → 重複
                    }
                    *last_seq = seq;
                    return true;
                }
            }
        }
        // 初めての送信元 → 空きスロットに記録
        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some((*mac, seq));
                return true;
            }
        }
        // 表が満杯なら最も古い扱いとして先頭を上書き（教材向けの簡略化）
        self.entries[0] = Some((*mac, seq));
        true
    }
}

impl<const N: usize> Default for DedupTable<N> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 単体テスト
//
// このテストはESP32-C6上では実行できません（no_stdターゲットにはテスト
// ランナーがないため）。protocol/error/configはハードウェア非依存なので、
// ホストPC向けにビルドすれば実行できます。例（Apple Siliconの場合）:
//
//   cargo test -p final-wireless-button --lib --target aarch64-apple-darwin
//
// ※教材ではCIには組み込まず、手元での動作確認用にとどめます。
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DecodeError;

    #[test]
    fn event_roundtrip() {
        let packet = Packet::Event { seq: 42 };
        let bytes = packet.to_bytes();
        assert_eq!(Packet::from_bytes(&bytes), Ok(packet));
        assert_eq!(packet.seq(), 42);
    }

    #[test]
    fn heartbeat_roundtrip_keeps_pressed_flag() {
        for pressed in [false, true] {
            let packet = Packet::Heartbeat { seq: 7, pressed };
            let bytes = packet.to_bytes();
            assert_eq!(Packet::from_bytes(&bytes), Ok(packet));
        }
    }

    #[test]
    fn ack_roundtrip_with_max_seq() {
        let packet = Packet::Ack { seq: u32::MAX };
        let bytes = packet.to_bytes();
        assert_eq!(Packet::from_bytes(&bytes), Ok(packet));
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(Packet::from_bytes(&[]), Err(DecodeError::BadLength));
        assert_eq!(
            Packet::from_bytes(&[0u8; PACKET_LEN + 1]),
            Err(DecodeError::BadLength)
        );
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = Packet::Ack { seq: 1 }.to_bytes();
        bytes[0] = 0x00;
        assert_eq!(Packet::from_bytes(&bytes), Err(DecodeError::BadMagic));
    }

    #[test]
    fn rejects_corrupted_payload() {
        let mut bytes = Packet::Event { seq: 1000 }.to_bytes();
        bytes[3] ^= 0xFF; // 電波ノイズによる1バイト破損を模擬
        assert_eq!(Packet::from_bytes(&bytes), Err(DecodeError::BadChecksum));
    }

    #[test]
    fn rejects_unknown_kind() {
        let mut bytes = Packet::Event { seq: 5 }.to_bytes();
        bytes[1] = 99;
        bytes[7] = super::xor_checksum(&bytes[..7]); // チェックサムは正しく直す
        assert_eq!(Packet::from_bytes(&bytes), Err(DecodeError::UnknownKind));
    }

    #[test]
    fn dedup_detects_resent_seq() {
        let mut table: DedupTable<4> = DedupTable::new();
        let mac = [1, 2, 3, 4, 5, 6];
        assert!(table.check_and_update(&mac, 1)); // 新規
        assert!(!table.check_and_update(&mac, 1)); // 再送 → 重複
        assert!(table.check_and_update(&mac, 2)); // 次のseq → 新規
    }

    #[test]
    fn dedup_tracks_senders_independently() {
        let mut table: DedupTable<4> = DedupTable::new();
        let mac_a = [0xAA; 6];
        let mac_b = [0xBB; 6];
        assert!(table.check_and_update(&mac_a, 10));
        assert!(table.check_and_update(&mac_b, 10)); // 別の送信元なら同じseqでも新規
        assert!(!table.check_and_update(&mac_a, 10));
        assert!(!table.check_and_update(&mac_b, 10));
    }

    #[test]
    fn dedup_overwrites_when_full() {
        let mut table: DedupTable<2> = DedupTable::new();
        assert!(table.check_and_update(&[1; 6], 1));
        assert!(table.check_and_update(&[2; 6], 1));
        assert!(table.check_and_update(&[3; 6], 1)); // 満杯 → 先頭を上書き
        assert!(!table.check_and_update(&[3; 6], 1)); // 上書き後は記憶されている
    }
}
