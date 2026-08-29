mod build;
mod dev;
mod format;
mod local_project;
mod lock;
mod new;
mod run;
mod test;

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
        [command, path] if command == "run" => run::run_path(path.as_ref()),
        [command, arguments @ ..] if command == "test" => test::test(arguments),
        [command, arguments @ ..] if command == "build" => build::build(arguments),
        [command, arguments @ ..] if command == "dev" => dev::dev(arguments),
        [command, arguments @ ..] if command == "new" => new::new(arguments),
        [command, arguments @ ..] if command == "lock" => lock::lock(arguments),
        [command, path] if command == "format" => {
            format::format_file(path.as_ref(), format::FormatMode::Write)
        }
        [command, flag, path] if command == "format" && flag == "--check" => {
            format::format_file(path.as_ref(), format::FormatMode::Check)
        }
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
            print_usage();
            Ok(0)
        }
        _ => Err("invalid arguments; run `seseragi --help` for usage".to_owned()),
    }
}

fn print_usage() {
    println!(
        "Usage:\n  seseragi --version\n  seseragi --version-json\n  seseragi new web path/to/my-app\n  seseragi lock update [path/to/package]\n  seseragi run path/to/app.ssrg\n  seseragi run path/to/package\n  seseragi test [path/to/package] [--filter text | --exact module::suite::case] [--jobs n] [--timeout ms] [--seed int] [--target node]\n  seseragi build path/to/app.ssrg [--target process|web] [--out-dir path/to/dist]\n  seseragi build path/to/package [--target process|web] [--out-dir path/to/dist]\n  seseragi dev [path/to/package] [--host 127.0.0.1] [--port 3000] [--open]\n  seseragi format [--check] path/to/app.ssrg"
    );
}
