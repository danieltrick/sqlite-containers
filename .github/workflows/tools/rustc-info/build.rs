fn main() {
    println!("cargo:rustc-env=X_CARGO_PATH={}", env!["CARGO"]);
}
