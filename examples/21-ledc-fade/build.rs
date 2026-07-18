fn main() {
    // defmt.x を linkall.x より前に置くこと（defmtのリンカセクションを先に定義する）
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // linkall.x は最後のリンカスクリプトにすること
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
