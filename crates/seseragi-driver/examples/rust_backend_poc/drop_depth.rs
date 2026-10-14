//! Runtime model probe, not a Seseragi source conformance test.
#![allow(dead_code)]
include!("runtime.rs");

fn main() {
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(|| {
            let mut value = V::Unit;
            for _ in 0..20_000 {
                value = V::adt("Link", value);
            }
            eprintln!("constructed 20000 nodes; dropping on a 256 KiB stack");
            drop(value);
            eprintln!("dropped; live={}", LIVE.with(Cell::get));
        })
        .unwrap()
        .join()
        .unwrap();
}
