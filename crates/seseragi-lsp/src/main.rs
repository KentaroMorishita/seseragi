use serde_json::json;
use std::{env, io};

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [argument] if argument == "--version" || argument == "-V" => {
            println!(
                "{} {} (protocol {}, analysis schema {})",
                seseragi_lsp::SERVER_NAME,
                seseragi_lsp::SERVER_VERSION,
                seseragi_lsp::PROTOCOL_VERSION,
                seseragi_lsp::ANALYSIS_SCHEMA_VERSION
            );
            return;
        }
        [argument] if argument == "--version-json" => {
            println!(
                "{}",
                json!({
                    "name": seseragi_lsp::SERVER_NAME,
                    "version": seseragi_lsp::SERVER_VERSION,
                    "protocolVersion": seseragi_lsp::PROTOCOL_VERSION,
                    "analysisSchemaVersion": seseragi_lsp::ANALYSIS_SCHEMA_VERSION,
                })
            );
            return;
        }
        [] => {}
        [argument] if argument == "--stdio" => {}
        _ => {
            eprintln!("usage: seseragi-lsp [--stdio | --version | --version-json]");
            std::process::exit(2);
        }
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    if let Err(error) = seseragi_lsp::run(stdin.lock(), stdout.lock()) {
        eprintln!("seseragi-lsp: {error}");
        std::process::exit(1);
    }
}
