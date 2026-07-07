mod checks;
mod discovery;
mod execution;
mod pipeline;
mod runner;

use std::path::PathBuf;

fn main() {
    let mut root = PathBuf::from(".");
    let mut list = false;
    for arg in std::env::args().skip(1) {
        if arg == "--list" {
            list = true;
        } else {
            root = PathBuf::from(arg);
        }
    }
    runner::run(root, list);
}
