mod format;
mod run;

fn main() {
    let exit = match run(std::env::args().skip(1)) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("seseragi: {error}");
            2
        }
    };
    std::process::exit(exit);
}

fn run(arguments: impl IntoIterator<Item = String>) -> Result<i32, String> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, path] if command == "run" => run::run_file(path.as_ref()),
        [command, path] if command == "format" => {
            format::format_file(path.as_ref(), format::FormatMode::Write)
        }
        [command, flag, path] if command == "format" && flag == "--check" => {
            format::format_file(path.as_ref(), format::FormatMode::Check)
        }
        [flag] if flag == "--help" || flag == "-h" => {
            print_usage();
            Ok(0)
        }
        _ => Err("invalid arguments; run `seseragi --help` for usage".to_owned()),
    }
}

fn print_usage() {
    println!(
        "Usage:\n  seseragi run path/to/app.ssrg\n  seseragi format [--check] path/to/app.ssrg"
    );
}
