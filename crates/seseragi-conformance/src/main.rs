mod checks;
mod discovery;
mod execution;
mod execution_case;
mod generated_module;
mod pipeline;
mod project_compile;
mod project_execution;
mod provider_compatibility;
mod provider_conformance_profile;
mod provider_contract;
mod provider_design_validation;
mod provider_lifecycle;
mod provider_manifest;
mod provider_stream;
mod provider_typescript_abi;
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
    let mut root = None;
    let mut list = false;
    let mut json = false;
    let mut only = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--list" => list = true,
            "--json" => json = true,
            flag if flag.starts_with("--only=") => {
                let kinds = flag["--only=".len()..]
                    .split(',')
                    .filter(|kind| !kind.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                if kinds.is_empty() || only.replace(kinds).is_some() {
                    eprintln!("--only takes one comma-separated list of check kinds");
                    std::process::exit(2);
                }
            }
            flag if flag.starts_with('-') => {
                eprintln!("unknown option: {flag}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            _ => {
                if root.replace(PathBuf::from(&arg)).is_some() {
                    eprintln!(
                        "only one repository ROOT is accepted; fixture paths are not selectors"
                    );
                    std::process::exit(2);
                }
            }
        }
    }
    runner::run(root.unwrap_or_else(|| PathBuf::from(".")), list, json, only);
}

fn print_usage() {
    println!(
        "usage: seseragi-conformance [ROOT] [--list] [--json] [--only=KIND[,KIND...]]\n\n\
         ROOT defaults to the current directory.\n\
         --list  list discovered fixture cases instead of running them\n\
         --json  emit a machine-readable JSON report\n\
         --only  run only these check kinds, named as failures report them\n\
                 (for example typescriptIr,coreIr)"
    );
}
