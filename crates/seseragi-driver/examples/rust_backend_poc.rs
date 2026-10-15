//! Opt-in feasibility experiment; never used by the production driver.
use seseragi_lowering::{lower_typed_module, CoreModule};
use seseragi_project::{link_module, standard_module_target};
use seseragi_semantics::analyze_linked_module;
use seseragi_syntax::{parse_diagnostics, parse_unlinked_module_interface};
use std::{collections::BTreeMap, env, fs};

#[path = "rust_backend_poc/backend.rs"]
mod backend;
#[cfg(test)]
#[path = "rust_backend_poc/tests.rs"]
mod tests;

fn frontend(path: &str, source: &str) -> Result<CoreModule, String> {
    let interface = parse_unlinked_module_interface(path, "rust-poc/main", source);
    let targets = interface
        .imports
        .iter()
        .filter_map(|import| {
            standard_module_target(&import.specifier)
                .map(|target| (import.specifier.clone(), target))
        })
        .collect::<BTreeMap<_, _>>();
    let linked = link_module(interface, &targets).map_err(|error| format!("{error:#?}"))?;
    let analyzed = analyze_linked_module(parse_diagnostics(path, source), linked, source)
        .map_err(|error| format!("{error:#?}"))?;
    Ok(lower_typed_module(analyzed.typed_hir))
}

fn main() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let valid = matches!(args.first().map(String::as_str), Some("dump")) && args.len() == 2
        || matches!(args.first().map(String::as_str), Some("emit")) && matches!(args.len(), 3 | 4);
    if !valid {
        return Err("Usage: cargo run -p seseragi-driver --example rust_backend_poc -- dump source.ssrg | emit source.ssrg output.rs [iterations]".into());
    }
    let source = fs::read_to_string(&args[1]).map_err(|error| error.to_string())?;
    let core = frontend(&args[1], &source)?;
    if args[0] == "dump" {
        println!("{}", serde_json::to_string_pretty(&core).unwrap());
    } else {
        let output = args.get(2).ok_or("emit requires an output path")?;
        let iterations = args
            .get(3)
            .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
            .transpose()?
            .unwrap_or(1);
        fs::write(output, backend::emit(&core, "probe", iterations)?).map_err(|e| e.to_string())?;
    }
    Ok(())
}
