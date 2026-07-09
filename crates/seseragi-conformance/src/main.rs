mod checks;
mod discovery;
mod execution;
mod execution_case;
mod generated_module;
mod pipeline;
mod report;
mod runner;
mod runtime_abi;
mod runtime_package;
mod suite;

use std::path::PathBuf;

fn main() {
    let mut root = PathBuf::from(".");
    let mut list = false;
    let mut json = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--list" => list = true,
            "--json" => json = true,
            _ => root = PathBuf::from(arg),
        }
    }
    runner::run(root, list, json);
}
