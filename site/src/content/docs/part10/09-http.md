---
title: "9. HTTP"
description: WebのプロトコルHTTPを生のテキストのまま学びます。GETリクエストの1行ずつの意味と、応答の構造をESP32-C6で実際に確かめます。
part: 10
lesson: 9
difficulty: intermediate
estimated_minutes: 20
prerequisites:
  - part10/06-tcp
  - part10/08-dns
hardware:
  - ESP32-C6-DevKitC-1
  - USBケーブル
  - 2.4GHz帯に対応したWi-Fiアクセスポイント（インターネット接続あり）
status: complete
code_status: cargo-check-passed
verified_with: "esp-hal 1.1.1 / esp-radio 0.18.0 / embassy-net 0.9.1"
last_verified: "2026-07-18"
sources:
  - https://www.rfc-editor.org/rfc/rfc9112
  - https://docs.rs/embassy-net/0.9.1
---

## このページでできるようになること

- HTTPのGETリクエストを構成する各行の意味を説明できる
- HTTP応答（ステータス行・ヘッダ・本文）の構造を読める
- 「HTTPはTCPの上を流れるただのテキスト」であることを、自分のコードで確かめられる

## 先に結論

HTTP（HyperText Transfer Protocol）は、Webで使われる**会話の書式**です。中身は人間が読めるテキストで、「GET / HTTP/1.1」のようなお願い（リクエスト）を送ると、「HTTP/1.1 200 OK」で始まる返事（レスポンス）が返ります。運ぶのは前々ページのTCPで、HTTP自身は運びません。ここが第10部の登頂地点です。Wi-Fi（電波）→IP（住所）→TCP（確実な流れ）と積み上げてきた土台の上で、最後にテキストの会話をするだけ——ライブラリなしの手書き文字列でWebサーバと話せることを、実機で確かめます。

## 身近なたとえ

HTTPは「注文票の書式」です。ファストフード店の注文票に「商品名・サイズ・持ち帰りかどうか」の決まった欄があるように、HTTPのリクエストにも「何がほしいか（パス）・どの店宛か（Host）・注文後どうするか（Connection）」の決まった行があります。書式さえ守れば、誰が書いても（ブラウザでもマイコンでも）同じように通じます。

ただし注文票と違い、HTTPの書式は**改行コードまで厳密に決まっている**点に注意してください。各行の終わりは`\r\n`（キャリッジリターン＋ラインフィード）で、リクエストの終わりは**空行**（`\r\n`だけの行）で示します。1文字でも欠けると通じません。

## 仕組み

### リクエストの中身

`examples/08-wifi`が送っている生のリクエストがこれです。

```text
GET / HTTP/1.1
Host: example.com
Connection: close
（空行）
```

| 行 | 意味 |
|---|---|
| `GET / HTTP/1.1` | リクエスト行。「`/`（トップページ）を**取得（GET）**したい。話す書式はHTTP/1.1」 |
| `Host: example.com` | 宛先のサイト名。1台のサーバが複数のサイトを担当していることが多いため、**どのサイト宛かを必ず書く**（HTTP/1.1では必須） |
| `Connection: close` | 「返事を送り終えたら接続を閉じてください」。閉じてもらえると、受信側は`read`の`Ok(0)`で終わりを知れる |
| 空行 | 「お願いはここまで」の合図。これを送るまでサーバは待ち続ける |

`GET`は「取得」を表す**メソッド**の1つです。ほかにデータを送る`POST`などがありますが、まずGETが読めれば十分です。

### レスポンスの中身

返事も同じくテキストで、3つの部分からなります。

```text
HTTP/1.1 200 OK          ← ステータス行（結果の要約）
Content-Type: text/html  ← ヘッダ（返事についての情報）が数行続く
Content-Length: 1256
（空行）                  ← ヘッダの終わり
<!doctype html>          ← 本文（ここではHTML）
...
```

ステータス行の数字（ステータスコード）は結果の分類です。`200`は成功、`404`は「そのページはない」、`500`はサーバ側のエラー。数字の百の位だけでも（2=成功、4=こちらのミス、5=あちらの問題）覚えておくと切り分けが速くなります。

## RustとEmbassyではどう書くか

`examples/08-wifi/src/main.rs`からの抜粋です。TCPソケットの`connect`まで済んでいる状態から始まります。

```rust
// 最小限のHTTP/1.1 GETリクエストを送る
let request = b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
match socket.write_all(request).await {
    Ok(()) => info!("HTTPリクエストを送信しました"),
    Err(e) => {
        error!("送信に失敗しました: {e:?}");
        Timer::after(Duration::from_secs(30)).await;
        continue;
    }
}
```

```rust
// 応答を先頭500バイトまで読み取る
let mut response = [0u8; 500];
let mut total = 0;
while total < response.len() {
    match socket.read(&mut response[total..]).await {
        Ok(0) => break, // サーバが接続を閉じた
        Ok(n) => total += n,
        Err(e) => {
            warn!("受信中にエラーが発生しました: {e:?}");
            break;
        }
    }
}

info!("---- 応答の先頭{total}バイト ----");
match core::str::from_utf8(&response[..total]) {
    Ok(text) => info!("{text}"),
    Err(_) => info!("(UTF-8として表示できないデータでした)"),
}
```

## コードを一行ずつ読む

```rust
let request = b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
```

先頭の`b`はバイト列リテラルの印です（ソケットが送るのは文字ではなくバイトなので）。`\r\n`が3つの行それぞれの終わりに、そして最後に**もう1組**あることを確認してください。この最後の`\r\n`が「空行＝リクエストの終わり」です。HTTPライブラリを使わなくても、正しい書式の文字列さえ送ればWebサーバは応えてくれます。プロトコルとは書式の約束事にすぎない、ということが実感できる1行です。

```rust
match core::str::from_utf8(&response[..total]) {
```

受信したのはただのバイト列なので、表示する前にUTF-8の文字列として正しいか確認しています。500バイトで打ち切っているため、多バイト文字の途中で切れて`Err`になることもあります。`no_std`でも`core::str`が使える点にも注目です。

## 実行方法

これで`examples/08-wifi`の全行程がそろいました。

```bash
SSID=あなたのSSID PASSWORD=あなたのパスワード cargo run --release -p wifi
```

```text
INFO - HTTPリクエストを送信しました
INFO - ---- 応答の先頭500バイト ----
INFO - HTTP/1.1 200 OK
Content-Type: text/html
...
INFO - ---- ここまで ----
INFO - 30秒後にもう一度リクエストします
```

ブラウザが裏でやっていることを、あなたは手書きの60バイトで再現しました。

## よくある失敗

- **最後の空行（`\r\n\r\n`）を忘れて、応答が永遠に来ない**: サーバは「まだお願いの続きがある」と思って待ち続け、こちらは`set_timeout`のタイムアウトで終わります。書式の1文字が通信全体を止める、HTTP手書き派の代表的な失敗です
- **`Host:`行を省いて`400 Bad Request`が返る**: HTTP/1.1ではHostヘッダは必須です。1台のサーバが複数サイトを担当している場合、どのサイト宛か分からないためです
- **`https://`のサイトに同じ方法でつなごうとする**: HTTPS（ポート443）はHTTPを**TLSという暗号の層**で包んだものです。平文のリクエストを送っても通じません。TLSはメモリも計算も重く、本教材の範囲外とします。現代のWebサイトの大半はHTTPS専用なので、平文HTTPの実験にはexample.comのような教材向けサイトを使ってください
- **応答全体を500バイトで受け取れると思う**: 打ち切りはexampleの意図的な仕様です。全文がほしければ`Ok(0)`（接続クローズ）までバッファを繰り返し読み進める設計にします

## やってみよう

リクエスト行を`GET /nonexistent HTTP/1.1`に変えて（Hostヘッダはそのまま）、ステータスコードがどう変わるか確かめてみましょう。`404 Not Found`が観察できるはずです。エラー応答も正しいHTTPの会話であることが分かります。

## 確認問題

1. `Host: example.com`の行は何のためにありますか。
2. リクエストの最後に空行（`\r\n`だけ）を送るのはなぜですか。
3. 「Wi-FiがあるからHTTPが使える」という説明はなぜ不正確ですか。このページまでの内容で答えてください。

<details>
<summary>答え</summary>

1. 1台のサーバ（1つのIPアドレス）が複数のWebサイトを担当していることがあるため、どのサイト宛のリクエストかを伝えるためです。HTTP/1.1では必須です。
2. 「リクエストはここまで」という終わりの合図だからです。これがないとサーバは続きを待ち続けます。
3. Wi-Fiは電波でリンクをつなぐ最下層にすぎないからです。その上にIP（DHCPで住所を得る）、TCP（確実なバイトの流れ）、必要ならDNS（名前解決）が積み重なって、初めてHTTPの会話ができます。

</details>

## まとめ

- HTTPはTCPの上を流れるテキストの書式。リクエストは「リクエスト行＋ヘッダ＋空行」、応答は「ステータス行＋ヘッダ＋空行＋本文」
- 書式は`\r\n`まで厳密。最後の空行がリクエストの終わりの合図
- HTTPSはHTTP＋TLS（暗号層）で別物。本教材では平文HTTPまでを扱う

## 次のページ

第10部の最終ページです。IoTで定番のプロトコルMQTTの考え方と、C6を小さなHTTPサーバーにする発想を紹介し、層の積み重ねの総まとめをします。

- 前: [8. DNS](/embassy-esp32-c6/part10/08-dns/)
- 次: [10. MQTTと小型サーバー](/embassy-esp32-c6/part10/10-mqtt-or-server/)
