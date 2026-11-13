mod benchmark;
mod build;
mod dev;
mod diagnostics;
mod doc;
mod dts;
mod experimental;
mod format;
mod help;
mod lint;
mod local_project;
mod lock;
mod new;
mod run;
mod target;
mod test;

/// The error of a command that already wrote its diagnostics to stderr.
/// `main` exits with status 2 without adding a `seseragi:` line.
pub(crate) const REPORTED: &str = "";

fn main() {
    let exit = match run(std::env::args().skip(1)) {
        Ok(code) => code,
        Err(error) if error == REPORTED => 2,
        Err(error) => {
            eprintln!("seseragi: {error}");
            2
        }
    };
    std::process::exit(exit);
}

fn run(arguments: impl IntoIterator<Item = String>) -> Result<i32, String> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    if let [command, rest @ ..] = arguments.as_slice() {
        if help::is_command(command) && command != "help" && wants_help(rest) {
            print!(
                "{}",
                help::command(std::slice::from_ref(command)).unwrap_or_default()
            );
            return Ok(0);
        }
    }
    match arguments.as_slice() {
        [] => {
            eprint!("{}", help::overview());
            Ok(2)
        }
        [command] if command == "help" => {
            print!("{}", help::overview());
            Ok(0)
        }
        [command, topic, ..] if command == "help" => {
            match help::command(std::slice::from_ref(topic)) {
                Some(page) => {
                    print!("{page}");
                    Ok(0)
                }
                None => Err(help::unknown_command(topic)),
            }
        }
        [command, arguments @ ..] if command == "experimental" => experimental::run(arguments),
        [command, arguments @ ..] if command == "benchmark" => benchmark::benchmark(arguments),
        [command, arguments @ ..] if command == "run" => run::run(arguments),
        [command, arguments @ ..] if command == "test" => test::test(arguments),
        [command, arguments @ ..] if command == "doc" => doc::doc(arguments),
        [command, arguments @ ..] if command == "dts" => dts::dts(arguments),
        [command, arguments @ ..] if command == "build" => build::build(arguments),
        [command, arguments @ ..] if command == "dev" => dev::dev(arguments),
        [command, arguments @ ..] if command == "new" => new::new(arguments),
        [command, arguments @ ..] if command == "lock" => lock::lock(arguments),
        [command, arguments @ ..] if command == "lint" => lint::lint(arguments),
        [command, arguments @ ..] if command == "format" => format::format(arguments),
        [flag] if flag == "--version" || flag == "-V" => {
            println!("{}", seseragi_release::format_human_version("seseragi"));
            Ok(0)
        }
        [flag] if flag == "--version-json" => {
            println!(
                "{}",
                serde_json::to_string(&seseragi_release::build_metadata("seseragi"))
                    .map_err(|error| format!("failed to encode version metadata: {error}"))?
            );
            Ok(0)
        }
        [flag] if flag == "--help" || flag == "-h" => {
            print!("{}", help::overview());
            Ok(0)
        }
        [flag, ..] if flag.starts_with('-') => Err(format!(
            "unknown option `{flag}`; run `seseragi --help` for usage"
        )),
        [command, ..] if !help::is_command(command) => Err(help::unknown_command(command)),
        _ => Err("invalid arguments; run `seseragi --help` for usage".to_owned()),
    }
}

/// `--help` or `-h` among a command's arguments asks for its help page.
fn wants_help(arguments: &[String]) -> bool {
    // Arguments after `--` belong to the program that `run` starts.
    arguments
        .iter()
        .take_while(|argument| *argument != "--")
        .any(|argument| argument == "--help" || argument == "-h")
}
