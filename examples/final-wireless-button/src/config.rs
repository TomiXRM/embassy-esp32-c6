//! config: プロジェクト全体で使う定数
//!
//! 責務: 周期・リトライ回数・タイムアウトなどの「調整つまみ」を1箇所に集める。
//! 依存方向: どのモジュールにも依存しない（プリミティブ型の定数のみ）。
//! ここを書き換えるだけで動作パラメータを変更できます。

/// ESP-NOWで使うWi-Fiチャネル。送信側と受信側で同じ値にすること
pub const WIFI_CHANNEL: u8 = 11;

/// ハートビート送信の周期（ミリ秒）
pub const HEARTBEAT_PERIOD_MS: u64 = 500;

/// ボタンのチャタリング対策で待つ時間（ミリ秒）
pub const DEBOUNCE_MS: u64 = 30;

/// イベント送信後、ACKを待つ時間（ミリ秒）。これを過ぎたら再送する
pub const ACK_TIMEOUT_MS: u64 = 200;

/// イベント1件あたりの最大送信回数（初回1回 + 再送2回 = 3回）
pub const MAX_SEND_ATTEMPTS: u8 = 3;

/// 最後にACKを受け取ってからこの時間（ミリ秒）を超えたらエラー状態に入る
pub const LINK_DOWN_AFTER_MS: u64 = 5000;

/// 受信側: この時間（ミリ秒）を超えて何も受信できなければ「送信側ロスト」と警告
pub const HEARTBEAT_LOST_MS: u64 = 2000;

/// エラー状態のときのLED高速点滅の周期（ミリ秒）
pub const ERROR_BLINK_MS: u64 = 100;

/// 受信側が重複判定のために記憶する送信元の最大数
pub const MAX_PEERS: usize = 4;
