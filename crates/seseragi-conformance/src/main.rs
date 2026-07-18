mod checks;
mod discovery;
mod execution;
mod execution_case;
mod generated_module;
mod pipeline;
mod project_compile;
mod project_execution;
mod report;
mod runner;
mod runtime_abi;
mod runtime_package;
mod runtime_stage;
mod stdlib_surface;
mod suite;
mod surface_ast;
mod typescript_ir;

use std::path::PathBuf;

fn main() {
    let mut root = PathBuf::from(".");
    let mut list = false;
    let mut json = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--list" => list = true,
            "--json" => json = true,
            flag if flag.starts_with('-') => {
                eprintln!("unknown option: {flag}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            _ => root = PathBuf::from(arg),
        }
    }
    runner::run(root, list, json);
}

fn print_usage() {
    println!(
        "usage: seseragi-conformance [ROOT] [--list] [--json]\n\n\
         ROOT defaults to the current directory.\n\
         --list  list discovered fixture cases instead of running them\n\
         --json  emit a machine-readable JSON report"
    );
}
