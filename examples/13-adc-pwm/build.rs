fn main() {
    // defmt.x は linkall.x より前に配置すること
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // linkall.x は最後のリンカスクリプトにすること
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
