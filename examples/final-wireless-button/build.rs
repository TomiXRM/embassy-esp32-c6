fn main() {
    // 組み込みターゲット（target_os = "none"）のときだけリンカスクリプトを渡す。
    // ホストPC向けビルド（protocol等の単体テスト実行時）には不要かつ
    // ホストのリンカが解釈できないため、条件付きにしている。
    // linkall.x は最後のリンカスクリプトにすること
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("none") {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
    }
}
