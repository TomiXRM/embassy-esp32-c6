fn main() {
    // linkall.x は最後のリンカスクリプトにすること
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
