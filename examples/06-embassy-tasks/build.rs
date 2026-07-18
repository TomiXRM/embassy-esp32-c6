fn main() {
    // linkall.x は最後のリンカスクリプトにすること
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
