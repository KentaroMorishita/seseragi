#[path = "support/differential/generator.rs"]
mod generator;
#[path = "support/generic_impl_process.rs"]
mod process;

use generator::{batch, generate, Case};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::Instant,
};

#[derive(Debug, PartialEq, Eq)]
struct Observation {
    stdout: Vec<u8>,
    exit: Option<i32>,
    defect: Option<String>,
}
fn defect_kind(message: &str) -> String {
    let message = message.strip_prefix("Seseragi ").unwrap_or(message);
    match message {
        "Int overflow" => "int-overflow".into(),
        "Int division by zero" => "division-by-zero".into(),
        "Int remainder by zero" => "remainder-by-zero".into(),
        _ => format!("other:{message}"),
    }
}
fn observe(output: &Output) -> Observation {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let defect = if output.status.code() == Some(70) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(stderr.trim()) {
            if json["kind"] == "Defect" && json["message"].is_string() {
                Some(defect_kind(json["message"].as_str().unwrap()))
            } else {
                Some(format!("invalid diagnostic:{stderr}"))
            }
        } else if let Some(message) = stderr.trim().strip_prefix("Seseragi Defect: ") {
            Some(defect_kind(message))
        } else {
            Some(format!("invalid diagnostic:{stderr}"))
        }
    } else if stderr.is_empty() {
        None
    } else {
        Some(format!("unexpected stderr:{stderr}"))
    };
    Observation {
        stdout: output.stdout.clone(),
        exit: output.status.code(),
        defect,
    }
}
fn valid(observation: &Observation) -> bool {
    match observation.exit {
        Some(0) => observation.defect.is_none(),
        Some(70) => observation
            .defect
            .as_ref()
            .is_some_and(|kind| !kind.starts_with("invalid")),
        _ => false,
    }
}
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

struct Executables {
    native: PathBuf,
    ts: PathBuf,
}
fn build(source: &str, directory: &Path) -> Executables {
    let input = directory.join("main.ssrg");
    fs::write(&input, source).unwrap();
    let compiled = seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        "main.ssrg",
        "differential/main",
        source,
    ))
    .unwrap_or_else(|diagnostics| {
        panic!(
            "generated program did not type-check:\n{}\n{source}",
            seseragi_driver::render_terminal_diagnostics(&diagnostics, source)
        )
    });
    let ts = directory.join("ts");
    seseragi_runtime::build_main_with_options(
        &compiled,
        &ts,
        seseragi_runtime::BuildTarget::Process,
        seseragi_runtime::ProcessRunOptions {
            diagnostic_format: seseragi_runtime::DiagnosticFormat::Json,
            hash_seed: seseragi_runtime::RandomSeed::Fixed(1170),
            random_seed: seseragi_runtime::RandomSeed::Fixed(1170),
            ..Default::default()
        },
    )
    .unwrap_or_else(|error| panic!("TS build failed: {error}\n{source}"));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(ts.join("artifact-manifest.json")).unwrap()).unwrap();
    let ts = ts.join(manifest["entry"].as_str().unwrap());
    let native = directory.join("native");
    let output = process::bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(input)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        directory,
    );
    assert!(
        output.status.success(),
        "native build failed (no generated case may be filtered out):\n{}\n{source}",
        String::from_utf8_lossy(&output.stderr)
    );
    Executables {
        native: native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        ts,
    }
}
fn execute(command: &mut Command, directory: &Path, input: Option<&Path>) -> (Observation, Output) {
    command
        .current_dir(directory)
        .env_remove("SESERAGI_NATIVE_METRICS");
    command.stdin(match input {
        Some(path) => Stdio::from(fs::File::open(path).unwrap()),
        None => Stdio::null(),
    });
    let output = process::bounded(command, directory);
    (observe(&output), output)
}
fn matches_expected(case: &Case, observation: &Observation) -> bool {
    match &case.expected {
        None => true,
        Some(Ok(stdout)) => {
            observation
                == &Observation {
                    stdout: stdout.as_bytes().to_vec(),
                    exit: Some(0),
                    defect: None,
                }
        }
        Some(Err(kind)) => {
            observation
                == &Observation {
                    stdout: Vec::new(),
                    exit: Some(70),
                    defect: Some((*kind).into()),
                }
        }
    }
}
fn compare(
    case: &Case,
    executables: &Executables,
    directory: &Path,
    input: Option<&Path>,
    engines: &[PathBuf],
) -> bool {
    let mut commands = vec![Command::new(&executables.native)];
    commands.extend(engines.iter().map(|engine| {
        let mut command = Command::new(engine);
        command.arg(&executables.ts);
        command
    }));
    let observations = std::thread::scope(|scope| {
        let children: Vec<_> = commands
            .into_iter()
            .enumerate()
            .map(|(index, mut command)| {
                let output_directory = directory.join(format!("engine-{index}"));
                fs::create_dir_all(&output_directory).unwrap();
                scope.spawn(move || execute(&mut command, &output_directory, input))
            })
            .collect();
        children
            .into_iter()
            .map(|child| child.join().unwrap())
            .collect::<Vec<_>>()
    });
    let native = &observations[0].0;
    let matched = observations.iter().all(|(observation, _)| {
        valid(observation) && observation == native && matches_expected(case, observation)
    });
    let mut details = String::new();
    for (index, (observation, output)) in observations.iter().enumerate() {
        let name = if index == 0 {
            "native".into()
        } else {
            engines[index - 1].display().to_string()
        };
        details.push_str(&format!(
            "{name}: exit={:?} defect={:?} stdout={:?}\nstderr: {}\n",
            observation.exit,
            observation.defect,
            String::from_utf8_lossy(&observation.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !matched {
        eprintln!(
            "MISMATCH seed={} area={} expected={:?}\n{details}PROGRAM:\n{}",
            case.seed,
            case.area,
            case.expected,
            if case.area == "replay" {
                case.body.clone()
            } else {
                case.standalone()
            }
        );
    }
    matched
}
fn setting(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .map(|value| {
            value
                .parse()
                .unwrap_or_else(|_| panic!("{name} must be a nonnegative integer"))
        })
        .unwrap_or(default)
}

#[test]
fn generated_targets_match() {
    let start = Instant::now();
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-differential-")
        .tempdir()
        .unwrap();
    let engines = [
        process::runtime("bun", scratch.path()),
        process::runtime("node", scratch.path()),
    ];
    if let Some(path) = std::env::var_os("SESERAGI_DIFF_REPLAY") {
        let source = fs::read_to_string(path).unwrap();
        let executables = build(&source, scratch.path());
        let case = Case {
            seed: 0,
            area: "replay",
            body: source.clone(),
            expected: None,
        };
        assert!(
            compare(&case, &executables, scratch.path(), None, &engines),
            "replay mismatch; original program:\n{source}"
        );
        return;
    }
    let count = setting(
        "SESERAGI_DIFF_COUNT",
        if std::env::var_os("SESERAGI_DIFF_LONG").is_some() {
            600
        } else {
            30
        },
    );
    let first = setting("SESERAGI_DIFF_SEED", 1170);
    assert!(
        count > 0 && count <= 1_000_000,
        "count must be in 1..=1000000"
    );
    first.checked_add(count - 1).expect("seed range overflow");
    let mut mismatches = 0;
    let mut completed = 0;
    while completed < count {
        let size = (count - completed).min(30);
        let cases: Vec<_> = (0..size)
            .map(|offset| generate(first + completed + offset))
            .collect();
        let directory = scratch.path().join(format!("batch-{completed}"));
        fs::create_dir(&directory).unwrap();
        eprintln!(
            "differential: building seeds {}..={}",
            cases.first().unwrap().seed,
            cases.last().unwrap().seed
        );
        let executables = build(&batch(&cases), &directory);
        for (index, case) in cases.iter().enumerate() {
            let input = directory.join("selector");
            fs::write(&input, format!("{index}\n")).unwrap();
            if !compare(case, &executables, &directory, Some(&input), &engines) {
                mismatches += 1;
            }
        }
        completed += size;
        fs::remove_dir_all(&directory).unwrap();
        eprintln!(
            "differential: {completed}/{count} programs, {mismatches} mismatches, {:.2}s",
            start.elapsed().as_secs_f64()
        );
    }
    assert_eq!(
        mismatches, 0,
        "{mismatches}/{count} differential programs mismatched; see seeds and programs above"
    );
}

#[test]
fn generator_is_repeatable_and_int_oracle_covers_defects_and_rounding() {
    for seed in 1170..1370 {
        assert_eq!(generate(seed).standalone(), generate(seed).standalone());
    }
    let cases: Vec<_> = (1170..1210)
        .map(generate)
        .filter(|case| case.area == "int")
        .collect();
    assert!(cases
        .iter()
        .any(|case| matches!(case.expected, Some(Err("int-overflow")))));
    assert!(cases
        .iter()
        .any(|case| matches!(case.expected, Some(Ok(_)))));
    for operator in ["/", "%", "+", "-", "*"] {
        assert!(cases.iter().any(|case| case.body.contains(operator)));
    }
}

#[test]
fn observations_preserve_stdout_exit_and_defect_kind() {
    fn output(exit: i32, stderr: &str) -> Output {
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(exit << 8)
        };
        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(exit as u32)
        };
        Output {
            status,
            stdout: b"\0\xff\n".to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }
    let native = observe(&output(70, "Seseragi Defect: Seseragi Int overflow\n"));
    let ts = observe(&output(
        70,
        r#"{"kind":"Defect","message":"Seseragi Int overflow","groups":[]}"#,
    ));
    assert_eq!(native, ts);
    assert!(valid(&native));
    assert_eq!(native.stdout, b"\0\xff\n");
    assert_ne!(
        native,
        observe(&output(70, "Seseragi Defect: Int division by zero\n"))
    );
    assert_ne!(
        defect_kind("Int division by zero"),
        defect_kind("Int remainder by zero")
    );
    assert_ne!(
        defect_kind("unrecognized Int overflow message"),
        defect_kind("Int overflow")
    );
    assert!(!valid(&observe(&output(0, "unexpected stderr"))));
    assert!(!valid(&observe(&output(70, "seseragi: runtime defect"))));
    assert!(!valid(&observe(&output(
        70,
        r#"{"kind":"TypedFailure","message":"Int overflow"}"#
    ))));
    assert!(!valid(&observe(&output(70, r#"{"kind":"Defect"}"#))));
    let case = Case {
        seed: 0,
        area: "int",
        body: String::new(),
        expected: Some(Err("int-overflow")),
    };
    assert!(!matches_expected(&case, &native));
    assert!(matches_expected(
        &case,
        &Observation {
            stdout: Vec::new(),
            ..ts
        }
    ));
    assert!(!matches_expected(
        &case,
        &Observation {
            stdout: Vec::new(),
            exit: Some(0),
            defect: Some("int-overflow".into())
        }
    ));
}
