fn main() {
    // defmt.x は linkall.x より先に渡すこと（defmtのリンカ定義を先に解決させる）
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // linkall.x は最後のリンカスクリプトにすること
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
