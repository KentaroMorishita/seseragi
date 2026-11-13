//! Help text for `seseragi --help`, `seseragi help [command]` and
//! `seseragi <command> --help`. Every command is described once in
//! [`COMMANDS`]; the overview and the per-command pages both read it.

struct CommandHelp {
    /// The words that select the command, e.g. `lock update`.
    name: &'static str,
    summary: &'static str,
    usage: &'static [&'static str],
    arguments: &'static [(&'static str, &'static str)],
    options: &'static [OptionHelp],
    examples: &'static [&'static str],
}

struct OptionHelp {
    flag: &'static str,
    description: &'static str,
    default: Option<&'static str>,
}

const fn option(flag: &'static str, description: &'static str) -> OptionHelp {
    OptionHelp {
        flag,
        description,
        default: None,
    }
}

const fn option_default(
    flag: &'static str,
    description: &'static str,
    default: &'static str,
) -> OptionHelp {
    OptionHelp {
        flag,
        description,
        default: Some(default),
    }
}

const PROFILE: OptionHelp = option_default(
    "--profile development|release",
    "Build profile. release removes development checks.",
    "the package manifest's profile, else development",
);
const PATH_OR_PACKAGE: (&str, &str) = (
    "path",
    "A .ssrg source file, or a package directory (one with seseragi.toml).",
);
const PACKAGE: (&str, &str) = (
    "path/to/package",
    "A package directory. Defaults to the current directory.",
);

const COMMANDS: &[CommandHelp] = &[
    CommandHelp {
        name: "new",
        summary: "Create a project from a template",
        usage: &[
            "seseragi new cli path/to/my-tool",
            "seseragi new web path/to/my-app",
        ],
        arguments: &[
            ("cli", "A command-line tool for the process and native targets."),
            ("web", "A browser app built with Signal."),
            (
                "path",
                "A directory that does not exist yet. Its last segment becomes the package name.",
            ),
        ],
        options: &[],
        examples: &["seseragi new cli hello", "seseragi new web hello-web"],
    },
    CommandHelp {
        name: "run",
        summary: "Compile and run a program",
        usage: &[
            "seseragi run path/to/app.ssrg [--target process|web|native] [--profile development|release] [--diagnostic-format text|json] [--signal-mode cancel|forward] [--shutdown-grace-ms ms] [--hash-seed entropy|int] [--random-seed entropy|int]",
            "seseragi run path/to/package [--target process|web|native] [--profile development|release] [--diagnostic-format text|json] [--signal-mode cancel|forward] [--shutdown-grace-ms ms] [--hash-seed entropy|int] [--random-seed entropy|int]",
        ],
        arguments: &[PATH_OR_PACKAGE],
        options: &[
            option_default(
                "--target process|web|native",
                "Where the program runs. native builds and runs an executable.",
                "the manifest's [run].target, else process",
            ),
            PROFILE,
            option_default(
                "--diagnostic-format text|json",
                "How compile errors are printed.",
                "text",
            ),
            option_default(
                "--signal-mode cancel|forward",
                "cancel turns a termination signal into cancellation; forward hands it to std/process.signals.",
                "cancel",
            ),
            option_default(
                "--shutdown-grace-ms ms",
                "How long cancellation may take after a signal (cancel mode only).",
                "10000",
            ),
            option_default(
                "--hash-seed entropy|int",
                "Seed for hash ordering.",
                "entropy",
            ),
            option_default(
                "--random-seed entropy|int",
                "Seed for std/random.",
                "the manifest's [run].random_seed, else entropy",
            ),
        ],
        examples: &[
            "seseragi run hello.ssrg",
            "seseragi run . --target native",
        ],
    },
    CommandHelp {
        name: "build",
        summary: "Compile a program into an output directory",
        usage: &[
            "seseragi build path/to/app.ssrg [--target process|web|native] [--profile development|release] [--source-map emit|omit] [--out-dir path/to/dist]",
            "seseragi build path/to/package [--target process|web|native] [--profile development|release] [--source-map emit|omit] [--out-dir path/to/dist]",
        ],
        arguments: &[PATH_OR_PACKAGE],
        options: &[
            option_default(
                "--target process|web|native",
                "What to build. native writes an executable named program.",
                "the manifest's [run].target, else the one target main can use, else process",
            ),
            PROFILE,
            option_default(
                "--source-map emit|omit",
                "Whether to write source maps (not for native).",
                "omit for release, emit otherwise",
            ),
            option_default("--out-dir path", "The output directory.", "dist"),
        ],
        examples: &["seseragi build .", "seseragi build . --target native"],
    },
    CommandHelp {
        name: "dev",
        summary: "Rebuild on change and serve a web app",
        usage: &[
            "seseragi dev [path/to/package] [--target web] [--host 127.0.0.1] [--port 3000] [--open]",
            "seseragi dev [path/to/app.ssrg|path/to/package] --target native",
        ],
        arguments: &[PACKAGE],
        options: &[
            option_default(
                "--target web|native",
                "web serves the app; native rebuilds an executable into .seseragi/dev.",
                "web",
            ),
            option_default("--host address", "The address to listen on (web only).", "127.0.0.1"),
            option_default("--port number", "The port to listen on (web only).", "3000"),
            option("--open", "Open the app in a browser (web only)."),
        ],
        examples: &["seseragi dev --open"],
    },
    CommandHelp {
        name: "test",
        summary: "Run the package's tests",
        usage: &["seseragi test [path/to/package] [--filter text | --exact module::suite::case] [--jobs n] [--timeout ms] [--seed int] [--target node|native]"],
        arguments: &[PACKAGE],
        options: &[
            option("--filter text", "Run only cases whose id contains text."),
            option("--exact module::suite::case", "Run only the case with this id."),
            option_default("--jobs n", "How many cases run at once.", "the manifest's [test].jobs, else 1"),
            option_default(
                "--timeout ms",
                "Time limit for each case.",
                "the manifest's [test].timeout_ms, else 30000",
            ),
            option_default(
                "--cleanup-grace-ms ms",
                "Time allowed for cleanup after a case ends.",
                "the manifest's [test].cleanup_grace_ms, else 5000",
            ),
            option_default("--seed int", "Seed for randomized tests.", "the manifest's [test].seed, else 0"),
            option_default(
                "--target node|native",
                "Where the tests run.",
                "the manifest's [test].target, else [run].target",
            ),
        ],
        examples: &["seseragi test", "seseragi test --filter greeting --target native"],
    },
    CommandHelp {
        name: "benchmark",
        summary: "Run the package's benchmarks",
        usage: &["seseragi benchmark [path/to/package] [--target node|native] [--filter text | --exact module::suite::case] [--warmup n] [--samples n] [--minimum-sample-ms ms] [--timeout-ms ms] [--cleanup-grace-ms ms] [--seed int] [--json] [--baseline path] [--save-baseline path] [--regression-threshold-percent n]"],
        arguments: &[PACKAGE],
        options: &[
            option_default(
                "--target node|native",
                "Where the benchmarks run.",
                "the manifest's [benchmark].target, else [run].target",
            ),
            option("--filter text", "Run only cases whose id contains text."),
            option("--exact module::suite::case", "Run only the case with this id."),
            option_default("--warmup n", "Unreported runs before measuring.", "10"),
            option_default("--samples n", "Measured samples per case (at least 3).", "50"),
            option_default("--minimum-sample-ms ms", "Shortest duration of one sample.", "100"),
            option_default("--timeout-ms ms", "Time limit for each body and measuring phase.", "30000"),
            option_default("--cleanup-grace-ms ms", "Time allowed for cleanup.", "5000"),
            option_default("--seed int", "Seed for randomized benchmarks.", "0"),
            option("--json", "Print a machine-readable report."),
            option("--baseline path", "Compare with a saved report."),
            option("--save-baseline path", "Save this run's report."),
            option_default(
                "--regression-threshold-percent n",
                "How much slower than the baseline counts as a regression.",
                "5.0",
            ),
        ],
        examples: &["seseragi benchmark --save-baseline bench.json"],
    },
    CommandHelp {
        name: "doc",
        summary: "Generate API documentation or run documentation examples",
        usage: &[
            "seseragi doc [path/to/package] [--include-private]",
            "seseragi doc [path/to/package] --test [--target node] [--include-private]",
        ],
        arguments: &[PACKAGE],
        options: &[
            option("--include-private", "Also document declarations that are not pub."),
            option("--test", "Run the code examples in documentation comments instead."),
            option_default("--target node", "Where examples run with --test.", "node"),
        ],
        examples: &["seseragi doc", "seseragi doc --test"],
    },
    CommandHelp {
        name: "lint",
        summary: "Report warnings without running the program",
        usage: &["seseragi lint path/to/app.ssrg|path/to/package [--diagnostic-format text|json] [--deny-warnings]"],
        arguments: &[PATH_OR_PACKAGE],
        options: &[
            option_default("--diagnostic-format text|json", "How diagnostics are printed.", "text"),
            option("--deny-warnings", "Exit with status 1 when there are warnings."),
        ],
        examples: &["seseragi lint . --deny-warnings"],
    },
    CommandHelp {
        name: "format",
        summary: "Format source files",
        usage: &[
            "seseragi format [--check] [path ...]",
            "seseragi format [--check] -",
        ],
        arguments: &[
            (
                "path",
                "A .ssrg file, a directory or a package. Directories skip dist, node_modules, .seseragi, .git and .gitignore entries. Defaults to the package containing the current directory.",
            ),
            ("-", "Read source from standard input and write the result to standard output."),
        ],
        options: &[option(
            "--check",
            "Change nothing; list files that need formatting and exit with status 1 if any do.",
        )],
        examples: &["seseragi format", "seseragi format --check src tests"],
    },
    CommandHelp {
        name: "lock update",
        summary: "Write the package's seseragi.lock",
        usage: &["seseragi lock update [path/to/package]"],
        arguments: &[PACKAGE],
        options: &[],
        examples: &["seseragi lock update"],
    },
    CommandHelp {
        name: "dts convert",
        summary: "Convert TypeScript declarations into Seseragi bindings",
        usage: &["seseragi dts convert [path/to/package] [--entry id]"],
        arguments: &[PACKAGE],
        options: &[option("--entry id", "Convert only this entry of the manifest.")],
        examples: &["seseragi dts convert"],
    },
    CommandHelp {
        name: "experimental",
        summary: "Unstable native and WASM build commands",
        usage: &[
            "seseragi experimental emit-rust source.ssrg|package-root --entry function --output program.rs [--provider std/clock::Clock=seseragi/runtime-native#clock]",
            "seseragi experimental build-native source.ssrg|package-root --entry function --out-dir path [--provider std/clock::Clock=seseragi/runtime-native#clock]",
            "seseragi experimental build-wasm source.ssrg|package-root --export function|function<Type>|name=function<Type> [--export other] [--profile development|release] --out-dir path",
        ],
        arguments: &[],
        options: &[
            option("--entry function", "The function to run as main."),
            option("--output path", "Where emit-rust writes the Rust source."),
            option("--out-dir path", "The output directory."),
            option("--provider service=provider", "Select a provider for a service. Repeatable."),
            option("--export function", "A function the WASM module exports. Repeatable."),
            PROFILE,
        ],
        examples: &[],
    },
];

/// The command names, for suggestions on a mistyped command.
fn command_words() -> impl Iterator<Item = &'static str> {
    COMMANDS
        .iter()
        .map(|command| command.name.split(' ').next().unwrap_or(command.name))
        .chain(["help"])
}

pub(crate) fn overview() -> String {
    let mut text = String::from(
        "Seseragi compiler and tools\n\nUsage: seseragi <command> [arguments]\n\nCommands:\n",
    );
    for command in COMMANDS {
        text.push_str(&format!("  {:<14}{}\n", command.name, command.summary));
    }
    text.push_str(&format!("  {:<14}{}\n", "help", "Show help for a command"));
    text.push_str(
        "\nOptions:\n  -h, --help        Show this help\n  -V, --version     Show the version\n      --version-json  Show the version as JSON\n\nRun `seseragi help <command>` or `seseragi <command> --help` for details.\n",
    );
    text
}

/// The help page for `words`, the command words before `--help`. Words after
/// the command name (paths, flags) are ignored.
pub(crate) fn command(words: &[String]) -> Option<String> {
    let help = COMMANDS.iter().find(|command| {
        let mut parts = command.name.split(' ');
        let first = parts.next();
        first == words.first().map(String::as_str)
    })?;
    let mut text = format!("{}\n\nUsage:\n", help.summary);
    for usage in help.usage {
        text.push_str(&format!("  {usage}\n"));
    }
    if !help.arguments.is_empty() {
        text.push_str("\nArguments:\n");
        for (name, description) in help.arguments {
            text.push_str(&format!("  {name}\n      {description}\n"));
        }
    }
    text.push_str("\nOptions:\n");
    for option in help.options {
        text.push_str(&format!(
            "  {}\n      {}\n",
            option.flag, option.description
        ));
        if let Some(default) = option.default {
            text.push_str(&format!("      Default: {default}\n"));
        }
    }
    text.push_str("  -h, --help\n      Show this help\n");
    if !help.examples.is_empty() {
        text.push_str("\nExamples:\n");
        for example in help.examples {
            text.push_str(&format!("  {example}\n"));
        }
    }
    Some(text)
}

/// Whether `word` names a command, so `seseragi help word` has a page.
pub(crate) fn is_command(word: &str) -> bool {
    command_words().any(|name| name == word)
}

/// The error for an unknown command, naming the closest command if any.
pub(crate) fn unknown_command(word: &str) -> String {
    let closest = command_words()
        .map(|name| (edit_distance(word, name), name))
        .filter(|(distance, name)| *distance <= 2.max(name.len() / 3))
        .min();
    match closest {
        Some((_, name)) => format!(
            "unknown command `{word}`; did you mean `{name}`? Run `seseragi --help` for all commands"
        ),
        None => format!("unknown command `{word}`; run `seseragi --help` for all commands"),
    }
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (i, left_char) in left.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, right_char) in right.iter().enumerate() {
            let substitution = previous[j] + usize::from(left_char != *right_char);
            current.push(substitution.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_the_closest_command() {
        assert!(unknown_command("fromat").contains("did you mean `format`?"));
        assert!(unknown_command("tset").contains("did you mean `test`?"));
        assert!(!unknown_command("frobnicate").contains("did you mean"));
    }

    #[test]
    fn every_command_has_a_page() {
        for command in COMMANDS {
            let first = command.name.split(' ').next().unwrap();
            let page = super::command(&[first.to_owned()]).unwrap();
            assert!(page.contains("Usage:"), "{}", command.name);
            assert!(overview().contains(command.name));
        }
    }
}
