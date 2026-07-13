use std::io;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    if let Err(error) = seseragi_lsp::run(stdin.lock(), stdout.lock()) {
        eprintln!("seseragi-lsp: {error}");
        std::process::exit(1);
    }
}
