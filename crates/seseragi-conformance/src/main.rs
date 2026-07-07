use std::fs;
use std::path::{Path, PathBuf};

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
    let artifacts = root.join("examples/spec/artifacts");
    let frontend_cases = discover_cases(&artifacts.join("schema-1"));
    let mut token_cases = frontend_cases.clone();
    token_cases.extend(discover_cases(&artifacts.join("token-schema-1")));
    let token_total = token_cases.len();
    let cst_total = frontend_cases.len();

    if list {
        println!("TokenStream fixtures:");
        for case in &token_cases {
            println!("{}", case.display());
        }
        println!("LosslessCst fixtures:");
        for case in &frontend_cases {
            println!("{}", case.display());
        }
        return;
    }

    let mut failed = 0;
    for case in &token_cases {
        if let Err(error) = check_tokens(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }
    for case in &frontend_cases {
        if let Err(error) = check_cst(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }
    println!("TokenStream fixtures: {token_total} passed");
    println!("LosslessCst fixtures: {cst_total} passed");
}

fn discover_cases(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut cases = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    cases.sort();
    cases
}

fn check_tokens(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("tokens.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected tokens: {error}"))?;
    let stream = seseragi_syntax::lex("main.ssrg", &source);
    let actual_value = serde_json::to_value(&stream)
        .map_err(|error| format!("failed to encode tokens: {error}"))?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected tokens: {error}"))?;

    if actual_value != expected_value {
        return Err("token artifact mismatch".to_owned());
    }
    if stream.reconstructed_text() != source {
        return Err("token raw text does not reconstruct source".to_owned());
    }
    Ok(())
}

fn check_cst(case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("cst.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected CST: {error}"))?;
    let cst = seseragi_syntax::parse_cst("main.ssrg", &source);
    let actual_value =
        serde_json::to_value(&cst).map_err(|error| format!("failed to encode CST: {error}"))?;
    let expected_value: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected CST: {error}"))?;

    if actual_value != expected_value {
        return Err("CST artifact mismatch".to_owned());
    }
    Ok(())
}
