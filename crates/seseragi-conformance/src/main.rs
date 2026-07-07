use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let cases = [
        root.join("examples/spec/artifacts/schema-1/basic"),
        root.join("examples/spec/artifacts/schema-1/recovery"),
        root.join("examples/spec/artifacts/token-schema-1/lexical-operators"),
        root.join("examples/spec/artifacts/token-schema-1/literals-and-nested-types"),
    ];
    let total = cases.len();

    let mut failed = 0;
    for case in &cases {
        if let Err(error) = check_tokens(case) {
            failed += 1;
            eprintln!("{}: {error}", case.display());
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }
    println!("TokenStream fixtures: {total} passed");
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
