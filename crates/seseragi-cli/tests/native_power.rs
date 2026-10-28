use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-native-power-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn compare(source: &Path, expected: &[u8], exit: i32, defect: Option<&str>, trace: Option<&[u8]>) {
    compare_cases(
        source,
        &[Case {
            name: "default",
            input: None,
            output: None,
            expected,
            exit,
            defect,
            trace,
            failure: None,
        }],
    );
}

struct Case<'a> {
    name: &'a str,
    input: Option<&'a [u8]>,
    output: Option<&'a [u8]>,
    expected: &'a [u8],
    exit: i32,
    defect: Option<&'a str>,
    trace: Option<&'a [u8]>,
    failure: Option<&'a str>,
}

fn compare_cases(source: &Path, cases: &[Case<'_>]) {
    let scratch = Scratch::new();
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let mut programs = vec![(
        "native".to_owned(),
        native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        None,
        true,
    )];
    for (profile, build_profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            &scratch.0,
        ));
        let mut files = if source.is_dir() {
            let loaded = seseragi_project::load_local_project(source).unwrap();
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(build_profile))
                    .unwrap();
            assert_eq!(compiled.compiled.modules.len(), 2);
            let raw = output.join("raw");
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let relative = Path::new(&module.generated.metadata.outputs.typescript);
                assert!(
                    !relative.is_absolute()
                        && !relative
                            .components()
                            .any(|c| c == std::path::Component::ParentDir)
                );
                let file = raw.join(relative);
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
            }
            files
        } else {
            let compiled = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new(
                    "native-power.ssrg",
                    "proof/native-power",
                    &fs::read_to_string(source).unwrap(),
                )
                .with_profile(build_profile),
            )
            .unwrap();
            let raw = output.join("raw-original.ts");
            fs::write(&raw, compiled.generated.typescript).unwrap();
            vec![raw]
        };
        if profile == "development" {
            files.push(output.join("entry.ts"));
        }
        let config = serde_json::json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": false,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "allowImportingTsExtensions": true, "types": ["bun","node"],
                "typeRoots": [root().join("node_modules/@types")],
                "baseUrl": root(),
                "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                "lib": ["ES2023","DOM","DOM.Iterable"],
            },
            "files": files,
        });
        fs::write(
            output.join("raw-tsconfig.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(output.join("raw-tsconfig.json")),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = output.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            programs.push((
                format!("{profile}-{engine}"),
                runtime(engine, &scratch.0),
                Some(entry.clone()),
                false,
            ));
        }
    }
    assert_eq!(programs.len(), 5);
    for (label, executable, argument, native) in programs {
        for case in cases {
            let Case {
                input,
                output: expected_file,
                expected,
                exit,
                defect,
                trace,
                failure,
                ..
            } = case;
            let label = format!("{label}/{}", case.name);
            let directory = scratch.0.join(format!("run-{}", label.replace('/', "-")));
            fs::create_dir(&directory).unwrap();
            if let Some(input) = input {
                fs::write(directory.join("input.txt"), input).unwrap();
            }
            if input.is_some() {
                fs::write(directory.join("output.txt"), b"old output\n").unwrap();
            }
            let mut command = Command::new(&executable);
            command.current_dir(&directory).env("PATH", "");
            if let Some(argument) = &argument {
                command.arg(argument);
            }
            if native {
                command.env("SESERAGI_NATIVE_METRICS", "1");
            }
            let output = bounded(&mut command, &directory);
            assert_eq!(output.status.code(), Some(*exit), "{label}: {output:?}");
            assert_eq!(output.stdout, *expected, "{label}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            if native {
                assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
                if let Some(defect) = defect {
                    assert!(stderr.contains(*defect), "{label}: {stderr}");
                }
            } else if defect.is_some() {
                assert_eq!(stderr, "seseragi: runtime defect\n", "{label}");
            } else if failure.is_none() {
                assert!(stderr.is_empty(), "{label}: {stderr}");
            }
            if let Some(failure) = failure {
                assert!(stderr.contains(*failure), "{label}: {stderr}");
            }
            if let Some(trace) = trace {
                assert_eq!(
                    fs::read(directory.join("trace.txt")).unwrap(),
                    *trace,
                    "{label}"
                );
            }
            if let Some(input) = input {
                assert_eq!(
                    fs::read(directory.join("input.txt")).unwrap(),
                    *input,
                    "{label}"
                );
                assert_eq!(
                    fs::read(directory.join("output.txt")).unwrap(),
                    expected_file.unwrap_or(b"old output\n"),
                    "{label}"
                );
            }
        }
    }
}

#[test]
fn powers_preserve_values_saved_methods_and_raw_strict_ts_on_bun_node_and_native() {
    let fixture = root().join("examples/spec/fixtures/rust-backend/native-power.ssrg");
    compare(
        &fixture,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/native-power.stdout"),
        0,
        None,
        None,
    );
}

#[test]
fn powers_keep_cold_replay_saved_ref_and_typed_failure_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/native-power-effects.ssrg"),
        include_bytes!("../../../examples/spec/fixtures/rust-backend/native-power-effects.stdout"),
        0,
        None,
        None,
    );
}

#[test]
fn float_powers_preserve_nan_signed_zero_infinity_and_boundary_parity() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/float-power-specials.ssrg"),
        include_bytes!("../../../examples/spec/fixtures/rust-backend/float-power-specials.stdout"),
        0,
        None,
        None,
    );
}

#[test]
fn integer_power_defects_wait_for_scope_cleanup() {
    let scratch = Scratch::new();
    let original = fs::read_to_string(
        root().join("examples/spec/fixtures/rust-backend/int-remainder-defect.ssrg"),
    )
    .unwrap();
    assert_eq!(original.matches("(%)").count(), 1);
    assert_eq!(original.matches("selected 125").count(), 1);
    assert_eq!(original.matches("saved 0").count(), 1);
    for (name, exponent, reason) in [
        ("negative", "-1", "Int negative exponent"),
        ("overflow", "53", "Int overflow"),
    ] {
        let source = scratch.0.join(format!("{name}.ssrg"));
        fs::write(
            &source,
            original
                .replace("(%)", "(**)")
                .replace("selected 125", "selected 2")
                .replace("saved 0", &format!("saved ({exponent})")),
        )
        .unwrap();
        compare(
            &source,
            b"",
            70,
            Some(reason),
            Some(b"acquire\nsaved\nrelease\n"),
        );
    }
}

fn file_cases(source: &Path) {
    compare_cases(
        source,
        &[
            Case {
                name: "valid",
                input: Some(b"0\n1\n3\n52\n"),
                output: Some(b"1\n2\n8\n4503599627370496\n"),
                expected: b"",
                exit: 0,
                defect: None,
                trace: None,
                failure: None,
            },
            Case {
                name: "overflow",
                input: Some(b"3\n53\n"),
                output: None,
                expected: b"",
                exit: 70,
                defect: Some("Int overflow"),
                trace: None,
                failure: None,
            },
            Case {
                name: "negative",
                input: Some(b"-1\n"),
                output: None,
                expected: b"",
                exit: 70,
                defect: Some("Int negative exponent"),
                trace: None,
                failure: None,
            },
            Case {
                name: "parse",
                input: Some(b"3\n01\n"),
                output: None,
                expected: b"",
                exit: 1,
                defect: None,
                trace: None,
                failure: Some("NumberFailure InvalidIntDigit { offset: 1, radix: 10 }"),
            },
            Case {
                name: "unicode",
                input: Some("3\n1🙂\n".as_bytes()),
                output: None,
                expected: b"",
                exit: 1,
                defect: None,
                trace: None,
                failure: Some("NumberFailure InvalidIntDigit { offset: 1, radix: 10 }"),
            },
        ],
    );
}

#[test]
fn file_power_transform_preserves_inputs_and_failed_outputs() {
    file_cases(&root().join("examples/spec/fixtures/rust-backend/filesystem-int-powers.ssrg"));
}

fn copy_package(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_package(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn path_package_power_keeps_frozen_sources_and_target_shared_lock() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_package(
        &root().join("examples/spec/fixtures/rust-backend-projects/int-powers"),
        &package,
    );
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    let paths = [
        "seseragi.lock",
        "seseragi.toml",
        "src/main.ssrg",
        "vendor/numbers/seseragi.toml",
        "vendor/numbers/src/main.ssrg",
    ];
    let frozen: Vec<_> = paths
        .iter()
        .map(|p| (*p, fs::read(package.join(p)).unwrap()))
        .collect();
    file_cases(&package);
    for (path, bytes) in frozen {
        assert_eq!(fs::read(package.join(path)).unwrap(), bytes, "{path}");
    }
}

#[test]
fn powers_evaluate_source_operands_once_in_order_across_saved_stages() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/native-power-trace.ssrg");
    let events = [
        "left", "right", "select", "left", "right", "select", "left", "right", "right",
    ];
    let core = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
        "native-power-trace.ssrg",
        "proof/native-power",
        source,
    ))
    .unwrap()
    .core_ir;
    let id = |name: &str| {
        format!(
            "v_{}",
            format!("proof/native-power::{name}")
                .bytes()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    };
    let mut rust = seseragi_backend_rust::emit_rust_module(&core, "probe")
        .unwrap()
        .source;
    rust.truncate(rust.rfind("fn main()").unwrap());
    // Observe only the three source helper entries. Arithmetic and dictionary
    // selection, application, and sharing retain the compiler's original code.
    for (name, event) in [("left", "left"), ("right", "right"), ("selected", "select")] {
        let start = rust.find(&format!("fn {}_direct(", id(name))).unwrap();
        let body = start + rust[start..].find("-> V {").unwrap() + "-> V {".len();
        rust.insert_str(
            body,
            &format!(" TRACE.with(|t| t.borrow_mut().push({event:?})); "),
        );
    }
    rust.push_str(&format!("\nthread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}\nfn main() {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), \"32\"); TRACE.with(|t| assert_eq!(&*t.borrow(), &{events:?})); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); eprintln!(\"managed-live=0\"); }}", id("probe")));
    let file = scratch.0.join("trace.rs");
    let binary = scratch
        .0
        .join(format!("trace{}", std::env::consts::EXE_SUFFIX));
    fs::write(&file, rust).unwrap();
    successful(bounded(
        Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition=2021", "-O", "-o"])
            .arg(&binary)
            .arg(&file),
        &scratch.0,
    ));
    let result = successful(bounded(Command::new(&binary).env("PATH", ""), &scratch.0));
    assert!(result.stdout.is_empty());
    assert_eq!(result.stderr, b"managed-live=0\n");
    let expected = serde_json::json!({"value":32,"trace":events});
    for (label, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(label);
        fs::create_dir(&output).unwrap();
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "native-power-trace.ssrg",
                "proof/native-power",
                source,
            )
            .with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        fs::write(output.join("original.ts"), &raw).unwrap();
        let mut traced = raw;
        for (name, event) in [("left", "left"), ("right", "right"), ("selected", "select")] {
            let start = traced.find(&format!("const {name} =")).unwrap();
            let body = start + traced[start..].find("=>").unwrap() + 2;
            let line_end = body + traced[body..].find('\n').unwrap();
            let end = if traced[..line_end].ends_with(';') {
                line_end - 1
            } else {
                line_end
            };
            assert!(!traced[body..end].trim_start().starts_with('{'));
            traced.insert_str(end, ")");
            traced.insert_str(body, &format!(" ($trace.push({event:?}), "));
        }
        traced.push_str("\nconst $trace: string[] = [];\nconsole.log(JSON.stringify({value: probe(undefined), trace: $trace}));\n");
        fs::write(output.join("traced.ts"), traced).unwrap();
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"allowImportingTsExtensions":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2023","DOM","DOM.Iterable"]},"files":[output.join("original.ts"),output.join("traced.ts")]});
        fs::write(
            output.join("tsconfig.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(output.join("tsconfig.json")),
            &scratch.0,
        ));
        seseragi_runtime::stage_typescript_package(&output).unwrap();
        successful(bounded(
            Command::new(runtime("bun", &scratch.0))
                .current_dir(&output)
                .args([
                    "build",
                    "traced.ts",
                    "--target=node",
                    "--outfile=traced.mjs",
                ]),
            &scratch.0,
        ));
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0)).arg(output.join("traced.mjs")),
                &scratch.0,
            ));
            assert!(result.stderr.is_empty(), "{label}/{engine}: {result:?}");
            let observed: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(observed, expected, "{label}/{engine}");
        }
    }
}
