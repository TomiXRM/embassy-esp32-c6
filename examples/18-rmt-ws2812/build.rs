fn main() {
    // defmt.x は defmt のログ格納用リンカスクリプト。linkall.x より前に渡す。
    // linkall.x は最後のリンカスクリプトにすること
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
