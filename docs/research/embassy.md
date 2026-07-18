# Embassy フレームワーク調査資料

調査日: 2026-07-18（crates.io API / docs.rs / embassy-rs GitHub / Embassy Book で検証）

## 1. 最新バージョン（crates.io、2026-07-18時点）

| クレート | 最新安定版 |
|---|---|
| embassy-executor | 0.10.0 |
| embassy-time | 0.5.1 |
| embassy-sync | 0.8.0 |
| embassy-net | 0.9.1 |
| embassy-futures | 0.1.2 |
| embedded-hal | 1.0.0 |
| embedded-hal-async | 1.0.0 |
| embedded-io-async | 0.7.0 |
| heapless | 0.9.3 |
| static_cell | 2.1.1 |
| critical-section | 1.2.0 |
| esp-hal-embassy | 0.9.1 (2025-10-14) |

## 2. ★最重要の互換性情報★

- esp-hal-embassy 0.9.1 は embassy-executor ^0.7 / embassy-time ^0.4 / embassy-sync ^0.6.2 に固定されたまま**凍結**（レガシー）。
- 現行のEmbassy統合は **esp-rtos 0.3.0**（esp-rs調査参照）で、**embassy-executor ^0.10 / embassy-time 0.5系**を使う。
- ただしBLE（trouble-host 0.6）との互換のため **embassy-sync は0.7系** を使用（公式BLE例と同じ）。
- API解説はこのバージョン系列（executor 0.10 / time 0.5 / sync 0.7 / net 0.9 / futures 0.1）に合わせ、examplesのcargo check結果を正とする。

## 3. embassy-executor のAPI要点

- `#[embassy_executor::task]`: async fn限定、ジェネリクス不可、`pool_size`引数（既定1）
- `Spawner::spawn(token)`（0.10ではspawnは失敗しない設計に変更されたが、**0.7系では `spawn` は `Result` を返す**。教材コードは実際に使うバージョンのAPIで統一し、cargo checkで確認する）
- タスクはstatic領域に確保される（heap不要）
- RISC-Vではupstreamのexecutor-interruptは非対応。ESPチップの割り込みexecutorは esp-hal-embassy の `InterruptExecutor` が提供

## 4. embassy-time のAPI要点（0.4/0.5系で共通の形）

- `Timer::at/after/after_ticks/after_nanos/after_micros/after_millis/after_secs`
- `Ticker::every(Duration)` + `.next().await`、`reset()` 系
- `Instant`（起動からのtick）と `Duration`
- `with_timeout(Duration, future)` → タイムアウトで `Err(TimeoutError)`、内側のFutureはdropされる。`WithTimeout`トレイトで `.with_timeout()` も可
- `Delay` は embedded-hal / embedded-hal-async の遅延トレイトを実装

## 5. embassy-sync のAPI要点

- モジュール: channel, priority_channel, pubsub, signal, watch, mutex, rwlock, semaphore, pipe, once_lock, zerocopy_channel, blocking_mutex
- RawMutex型: `CriticalSectionRawMutex`（スレッド+割り込み共有）、`NoopRawMutex`（同一executor内）、`ThreadModeRawMutex`
- `Channel<M, T, const N>`: `send`/`receive`（async, MPMC, 有界, バックプレッシャあり）、`try_send`/`try_receive`、`sender()`/`receiver()`
- `Signal<M, T>`: `signal(val)`（前の値を上書き）、`wait()`、`try_take()`。最新値のみ保持
- `Mutex<M, T>`: `lock().await` → `MutexGuard`
- `PubSubChannel<M, T, CAP, SUBS, PUBS>`、`Watch<M, T, N>`（最新値のマルチ消費者、途中の値は失われうる）

## 6. embassy-futures 0.1.2

- `select::select(a, b)` → `Either::First/Second`。**負けた側のFutureはdropされる**（キャンセル）
- `select3/select4/select_array/select_slice`
- `join::join(a, b)` → 両方完了を待つ
- `block_on`、`yield_now`、`poll_once`

## 7. embassy-net（0.9系）

- `embassy_net::new(driver, config, resources, seed)` は自由関数で `(Stack<'d>, Runner<'d, D>)` を返す（Stack::newではない）
- `Stack`はCopy可能なハンドル。`Runner::run()` を専用taskで回す
- `Config::dhcpv4(Default::default())` / `Config::ipv4_static(...)`
- `tcp::TcpSocket::new(stack, &mut rx_buf, &mut tx_buf)` → `connect/accept`、embedded-io-asyncの`read`/`write`
- `udp::UdpSocket`、`dns::DnsSocket`
- ※esp-halとの組み合わせで実際に使うバージョンはexamplesのcargo checkで確定する

## 8. select vs join・キャンセルの意味論（Embassy Book / docs.rs）

- 並行性の2方式: 複数task vs 1 task内でjoin/select。join/selectは管理下の全Futureをチェックする（executorはtask単位でしか起こせない）ため、専用taskの方が起床が速い。Future同槽方式は借用共有が楽でRAM節約
- キャンセルモデル: **drop = キャンセル**。selectの負け側・with_timeoutのタイムアウト側はdropされ、それ以上pollされない
- 「タスクが終了またはキャンセルされたら再enqueueされない」(runtime.adoc)

## 情報源URL

- https://embassy.dev/book/
- https://docs.rs/embassy-executor / embassy-time / embassy-sync / embassy-net / embassy-futures
- https://github.com/embassy-rs/embassy
