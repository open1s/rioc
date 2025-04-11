fn main() {
    let out_dir = std::env::var("RIOC_OUT_DIR").unwrap_or_else(|_| {
        std::env::var("OUT_DIR").unwrap_or_else(|_| String::from("./target/.rioc"))
    });
    // Expose the out directory to macros.
    println!("cargo:rustc-env=RIOC_OUT_DIR={}", out_dir)
}