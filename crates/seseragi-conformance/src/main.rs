mod checks;
mod discovery;
mod execution;
mod execution_case;
mod generated_module;
mod pipeline;
mod runner;
mod runtime_abi;
mod runtime_package;

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
