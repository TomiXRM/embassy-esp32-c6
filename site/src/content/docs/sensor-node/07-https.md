---
title: "7. no_stdでHTTPS — examples/17の解説"
description: reqwless 0.14とembedded-tlsでESP32-C6からHTTPS GETを行う検証済みコードを読みます。TLS 1.3の制約、16640バイトのレコードバッファ、TlsVerify::Noneの正直な意味、derピン事件まで。
lesson: 7
difficulty: advanced
estimated_minutes: 30
prerequisites:
  - sensor-node/06-clock
  - part10/02-station
  - part10/09-http
  - part04/04-crate
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - 2.4GHz帯のWi-Fiアクセスポイント
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal ~1.1.0 / esp-rtos 0.3.0 / esp-radio 0.18.0 / embassy-net 0.9.1 / reqwless 0.14 / der =0.8.0-rc.10"
last_verified: "2026-07-18"
sources:
  - https://github.com/claudiomattera/esp32c3-embassy
  - https://docs.rs/reqwless/0.14.0
  - https://github.com/drogue-iot/embedded-tls
  - https://datatracker.ietf.org/doc/html/rfc8446
---

## このページでできるようになること

- 平文HTTPとHTTPSの違いを「どの層が何を守るか」で説明できる
- no_stdのTLSスタック（reqwless + embedded-tls）の構成と、大きなバッファが必要な理由を説明できる
- `TlsVerify::None`が何を守り、何を守らないかを正直に説明できる
- rc版依存のクレートがある日突然ビルドできなくなる事故と、バージョン固定による防ぎ方を説明できる

## 先に結論

第10部の`08-wifi`は平文のHTTPでした。平文は経路上のどこでも中身が読めます。examples/17は同じ構成にTLS（Transport Layer Security）を足し、`https://`のURLへGETします。使うのはHTTPクライアントの**reqwless 0.14**と、そのTLSバックエンドの**embedded-tls**（TLS 1.3専用）です。TLSはひとかたまり最大16KiBの「レコード」でデータをやり取りするため、受信用・送信用に**16640バイトずつ**のバッファが要り、スタックに置くと溢れるので`StaticCell`でstatic領域に確保します。乱数の種はesp-halのハードウェア乱数生成器から作ります。ただしこの例は`TlsVerify::None`——**通信は暗号化されるが、相手が本物のサーバかは検証していません**。そしてCargo.tomlには`der = "=0.8.0-rc.10"`という奇妙なピン留めがあります。これは「rc版に依存したクレートが、安定版のリリースで壊れた」実話の跡です。

## 身近なたとえ

平文HTTPは、はがきで手紙を送るようなものです。配達に関わる全員が文面を読めます。HTTPSは封筒に入れて封をした手紙です。途中の人には中身が読めません。ただし`TlsVerify::None`は、**封筒には入れたけれど、届け先の相手が本人かどうか身分証を確認していない**状態です。封は完璧でも、最初から偽物の相手に渡していたら意味がありません。

たとえと違うのは、TLSの「封」は数学的な暗号で、盗み見だけでなく改ざんも検出できることです。また「身分証の確認」にあたる証明書検証は、確認する側がルート証明書という照合台帳をあらかじめ持っている必要があります。

## 仕組み

### なぜ暗号化が要るのか — 層で考える

「Wi-FiはWPA2/WPA3で暗号化されているからHTTPでも安全では？」と思うかもしれません。ここで第10部で学んだ層の区別が効きます。WPAの暗号化は**あなたの端末とアクセスポイントの間**だけを守るリンク層の仕組みです。アクセスポイントから先——プロバイダ、途中の中継網、サーバまでの経路——では、HTTPの中身は平文のまま流れます。TLSはTCPの上に載り、**端末とサーバの間を端から端まで**暗号化します。守る区間がまったく違うのです。

### no_stdのTLSスタック

パソコンのプログラムなら、HTTPSはOSと出来合いのライブラリに任せられます。no_stdでは自分で部品を積みます。

- **reqwless 0.14** — HTTPクライアント。URLが`https://`ならTLSハンドシェイクを自動で行う
- **embedded-tls** — TLS実装。**TLS 1.3のみ**対応。TLS 1.2までしか話せない古いサーバには接続できない
- 下回りは第10部と同じ: embassy-netのTCP/DNS、esp-radioのWi-Fi

### コードを一行ずつ読む

以下はすべて抜粋です。完全なコードは `examples/17-https` を見てください（cargo check済み）。全体の骨組み——Wi-Fi初期化、DHCP、`connection_task`/`net_task`——は`08-wifi`と同一で、差分はここからです。

まず、TLSのレコードバッファです。

```rust
// TLSのレコードバッファ。TLSレコードは最大16KiB+ヘッダなので16640バイト確保する。
// 合計約32KiBと大きいため、スタックではなくstatic領域に置く
static TLS_READ_BUFFER: StaticCell<[u8; 16640]> = StaticCell::new();
static TLS_WRITE_BUFFER: StaticCell<[u8; 16640]> = StaticCell::new();
```

TLSはデータを「レコード」という単位で暗号化して運びます。レコードの中身は最大16KiB（16384バイト）と決まっていて、相手がいっぱいまで詰めてくる可能性がある以上、受け側は最大サイズ＋暗号化の付加情報ぶんを常に受け止められなければなりません。それが16640バイトです。送受で2本、合計約32KiB——C6のRAMは512KBなので置けますが、taskのスタックに置くと簡単に溢れます。だから`StaticCell`でstaticに確保します（この設計は参照元esp32c3-embassyの`http.rs`と同じで、参照元は構造体のフィールドとして持っています）。

次に、乱数の種です。

```rust
// TLSの内部乱数（鍵交換などに使用）のシードをハードウェア乱数から作る。
// reqwlessはこの64ビット値でChaCha8乱数生成器を初期化する
let tls_seed = ((rng.random() as u64) << 32) | rng.random() as u64;
```

TLSは接続のたびに使い捨ての鍵を乱数から作ります。乱数が予測可能だと暗号全体が崩れるので、種はesp-halのハードウェア乱数生成器（`Rng`）から取ります。`random()`は32ビットを返すため、2回呼んで64ビットに組み立てています。参照元は同じことを`rand_core`のトレイトを実装した`RngWrapper`型として整理しています（`random.rs`）。

そしてTLS設定とクライアントの生成、リクエストです。

```rust
let tls_config = TlsConfig::new(
    tls_seed,
    TLS_READ_BUFFER.init([0; 16640]),
    TLS_WRITE_BUFFER.init([0; 16640]),
    TlsVerify::None,
);
let mut client = HttpClient::new_with_tls(&tcp_client, &dns_socket, tls_config);
// この1行でDNS解決→TCP接続→TLSハンドシェイクまで行われる
let mut request = client.request(Method::GET, URL).await?;
```

### TlsVerify::Noneの正直な意味

`TlsVerify::None`で得られるもの・得られないものを、ごまかさずに書きます。

| | TlsVerify::None | TlsVerify::Certificate |
|---|---|---|
| 通信の暗号化（盗聴防止） | される | される |
| 改ざんの検出 | される | される |
| **相手が本物のサーバである保証** | **ない** | ある（証明書を検証） |
| 中間者攻撃（なりすまし）の検出 | **できない** | できる |

TLSのハンドシェイクでサーバは証明書を提示しますが、`None`はそれを**確認せずに信用**します。あなたとサーバの間に割り込んだ攻撃者が偽サーバを立てても、TLS接続自体は成立してしまい、暗号化された通信路の先にいるのが攻撃者、という事態を検出できません。教材のexampleが`None`なのは、ルートCA証明書の組み込みと期限管理まで含めると1ページに収まらないための割り切りです。**製品ではルートCA証明書を組み込んで`TlsVerify::Certificate`にすべき**です。参照実装であるesp32c3-embassyも、同じ割り切りで`None`を使っています。「何を守れていないかを知った上で使う」——これが今回いちばん持ち帰ってほしい態度です。

### der =0.8.0-rc.10ピン事件 — バージョン固定の実話

examples/17のCargo.tomlには、直接は使わないクレートが1行あります。

```toml
# 直接は使わないが、embedded-tls 0.18と互換のあるrc版にderを固定するために明示する
der = "=0.8.0-rc.10"
```

経緯はこうです。embedded-tls 0.18は、証明書の解析に使う`der`クレートの**リリース候補版（rc版）**のAPIに依存して開発されました。その後、`der` 0.8.0の**安定版**が公開されましたが、rc版から安定版までの間にAPIが変わっていました。semver（セマンティックバージョニング）の規則上、`0.8.0-rc.10`を要求する依存は`0.8.0`安定版も受け入れてしまうため、何もしていないのに`cargo update`した日からビルドが壊れる——という事故が起きます。対策が`=`付きの完全固定です。`=0.8.0-rc.10`と書けば、cargoは一切の自動更新をしません。

これは第4部4ページで学んだ「依存関係は自分のコードの一部」の実戦形であり、この教材が`versions.md`で全クレートのバージョンを固定している理由そのものです。rc版・プレリリース版に依存するクレートを使うときは、壊れる前に自分でピンを打つ。壊れてから原因を探すと、変更していないコードがビルドできないという最も混乱する形で現れます。

## 実行方法

SSIDとパスワードを環境変数で渡してビルド・書き込みします（コンパイル時に埋め込まれます）。

```bash
SSID=あなたのSSID PASSWORD=あなたのパスワード cargo run --release -p https
```

期待されるログの流れです。

```text
INFO - Wi-Fiを初期化します
INFO - IPアドレスの取得を待っています...
INFO - IPアドレスを取得しました: 192.168.x.x/24
INFO - https://www.example.com/ へHTTPS GETリクエストを送ります
INFO - HTTPステータス: Ok
INFO - ---- 本文の先頭500バイト ----
（HTMLの先頭が表示される）
INFO - 60秒後にもう一度リクエストします
```

## よくある失敗

- **接続先がTLS 1.2までのサーバで、ハンドシェイクに失敗する** — embedded-tlsはTLS 1.3専用です。古い機器やローカルの簡易サーバにはTLS 1.3を話せないものがあり、その場合`client.request()`がエラーになります。接続先の対応バージョンを確認してください
- **TLSバッファをローカル変数に置いてスタックを溢れさせる** — `[u8; 16640]`を2本、関数内に書くとそれだけで32KiB強です。no_stdのスタックは小さく、静かに壊れることもあります。大きなバッファは`StaticCell`か`static`へ
- **`cargo update`でderが0.8.0安定版に上がりビルドが壊れる** — 上記の事件そのものです。`der = "=0.8.0-rc.10"`のピンを消さないでください。エラーメッセージはembedded-tls内部の型不一致として現れるため、原因がderだと気づきにくいのが厄介な点です
- **SSID/PASSWORD未設定のままで接続に失敗し続ける** — 環境変数はビルド時に埋め込まれます。設定し忘れるとプレースホルダのままビルドは通り、実行時に接続失敗を繰り返します

## やってみよう

`URL`定数を`https://`から`http://`に変えて（`http://example.com/`）、何が起きるか観察してみましょう。reqwlessは平文HTTPも話せるので通信は成功します。つまり**暗号化するかどうかを決めているのはURLのスキーム1文字**です。確認したら必ず`https://`に戻してください。

## 確認問題

1. WPA2で暗号化されたWi-Fiを使っていても、HTTPSが必要なのはなぜですか。
2. `TlsVerify::None`の通信を盗聴できますか。なりすませますか。それぞれ理由も答えてください。
3. `der = "=0.8.0-rc.10"`の`=`を外すと、将来どんな事故が起きえますか。

<details>
<summary>答え</summary>

1. WPAが守るのは端末とアクセスポイント間のリンク層だけだからです。アクセスポイントから先の経路では平文HTTPは誰でも読めます。TLSは端末とサーバの間を端から端まで守ります。
2. 盗聴はできません（通信路は正しく暗号化されます）。しかし、なりすましは可能です。証明書を検証しないため、中間者が偽サーバとしてTLS接続を成立させても検出できません。
3. `cargo update`時にsemver互換とみなされる`der` 0.8.0安定版へ自動更新され、rc版のAPIに依存しているembedded-tls 0.18のビルドが、コードを1行も変えていないのに壊れます。

</details>

## まとめ

- HTTPSはTCPの上のTLSで端末とサーバ間を暗号化する。Wi-Fiの暗号化とは守る区間が違う
- no_stdのTLSは部品を自分で積む。TLS 1.3専用のembedded-tlsと、16640バイト×2のレコードバッファをstaticに置く構成
- `TlsVerify::None`は暗号化のみで真正性の保証なし。製品は`TlsVerify::Certificate`。依存のrc版はピンで固定して事故を防ぐ

## 次のページ

Wi-Fi・TLS・センサ——部品が増えるほど「失敗の種類」も増えます。しかしEmbassyのtaskは`Result`を返せません。実プロジェクトはこの制約とどう付き合っているのか、参照元のエラー設計を読みます。

[8. taskはResultを返せない — 実プロジェクトのエラー設計 →](/embassy-esp32-c6/sensor-node/08-error-design/)

---

前: [6. 時計のないマイコンが時刻を知る方法](/embassy-esp32-c6/sensor-node/06-clock/) | 次: [8. taskはResultを返せない](/embassy-esp32-c6/sensor-node/08-error-design/)
