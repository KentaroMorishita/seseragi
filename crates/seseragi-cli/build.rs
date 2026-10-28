fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // The MSVC executable default reserves only 1 MiB. Checking ordinary
        // nested expressions can exceed that in an unoptimized compiler build.
        // Reserve 8 MiB for the CLI's main thread, without increasing the
        // initially committed pages or changing generated program stacks.
        println!("cargo:rustc-link-arg-bin=seseragi=/STACK:8388608");
    }
}
