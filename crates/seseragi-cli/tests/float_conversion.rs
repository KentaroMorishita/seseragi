use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-float-conversion-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct OwnedChild {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
}
impl OwnedChild {
    fn start(command: &mut Command, directory: &Path) -> Self {
        let stdout = directory.join("stdout");
        let stderr = directory.join("stderr");
        command
            .stdout(Stdio::from(File::create(&stdout).unwrap()))
            .stderr(Stdio::from(File::create(&stderr).unwrap()));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Self {
            child: command.spawn().unwrap(),
            stdout,
            stderr,
        }
    }
    fn wait(&mut self, timeout: Duration) -> Output {
        let deadline = Instant::now() + timeout;
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "deadline for owned PID {}\n{}",
                self.child.id(),
                fs::read_to_string(&self.stderr).unwrap()
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        self.child.wait().unwrap();
        Output {
            status,
            stdout: fs::read(&self.stdout).unwrap(),
            stderr: fs::read(&self.stderr).unwrap(),
        }
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            #[cfg(unix)]
            unsafe {
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                // Only this still-live child's dedicated process group.
                kill(-(self.child.id() as i32), 15);
                // Do not reap the group leader during the grace interval. Its
                // reserved PID prevents group-ID reuse before the final kill;
                // compiler descendants may outlive a terminated parent.
                std::thread::sleep(Duration::from_secs(2));
                kill(-(self.child.id() as i32), 9);
            }
            #[cfg(windows)]
            {
                if let Ok(mut terminator) = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &self.child.id().to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    let until = Instant::now() + Duration::from_secs(5);
                    while Instant::now() < until {
                        if terminator.try_wait().ok().flatten().is_some() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let _ = terminator.kill();
                    let _ = terminator.wait();
                }
            }
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn bounded(command: &mut Command, directory: &Path) -> Output {
    OwnedChild::start(command, directory).wait(Duration::from_secs(120))
}
fn runtime(name: &str, directory: &Path) -> PathBuf {
    let output = successful(bounded(
        Command::new(if cfg!(windows) { "where" } else { "which" }).arg(name),
        directory,
    ));
    PathBuf::from(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
}

fn root() -> PathBuf {
    // Node24 cannot open script entries through Windows verbatim paths from
    // fs::canonicalize. Cargo's manifest path is absolute; use lexical parents.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root above the crate")
        .to_path_buf()
}

fn check_raw_strict(
    text: &str,
    profile: &str,
    value: seseragi_driver::BuildProfile,
    output: &Path,
    scratch: &Path,
) {
    let raw = seseragi_driver::compile_module(
        seseragi_driver::CompileInput::new("float-conversion.ssrg", "proof/int-remainder", text)
            .with_profile(value),
    )
    .unwrap();
    fs::write(output.join("raw.ts"), raw.generated.typescript).unwrap();
    let mut files = vec![output.join("raw.ts")];
    if profile == "development" {
        files.push(output.join("entry.ts"));
    }
    let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2023","DOM","DOM.Iterable"],"allowImportingTsExtensions":true},"files":files});
    fs::write(
        output.join("tsconfig.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    successful(bounded(
        Command::new(runtime("node", scratch))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(output.join("tsconfig.json")),
        scratch,
    ));
}

fn build_programs(
    source: &Path,
    project: bool,
    scratch: &Path,
) -> Vec<(String, PathBuf, Option<PathBuf>, bool)> {
    let native = scratch.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch,
    ));
    let mut programs = vec![(
        "native".to_owned(),
        native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        None,
        true,
    )];
    for (profile, value) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            &scratch,
        ));
        if project {
            let loaded = seseragi_project::load_local_project(&source).unwrap();
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(value)).unwrap();
            assert_eq!(compiled.compiled.modules.len(), 2);
            let raw = output.join("raw");
            fs::create_dir(&raw).unwrap();
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
            if profile == "development" {
                assert!(output.join("entry.ts").is_file());
                files.push(output.join("entry.ts"));
            }
            let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"allowImportingTsExtensions":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2023","DOM","DOM.Iterable"]},"files":files});
            fs::write(
                output.join("project-raw.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            successful(bounded(
                Command::new(runtime("node", &scratch))
                    .arg(root().join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(output.join("project-raw.json")),
                &scratch,
            ));
        } else {
            check_raw_strict(
                &fs::read_to_string(&source).unwrap(),
                profile,
                value,
                &output,
                &scratch,
            );
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = output.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            programs.push((
                format!("{profile}-{engine}"),
                runtime(engine, &scratch),
                Some(entry.clone()),
                false,
            ));
        }
    }
    assert_eq!(programs.len(), 5);
    programs
}

fn check_file_entry(source: PathBuf, project: bool) {
    let scratch = Scratch::new();
    let programs = build_programs(&source, project, &scratch.0);
    for (label, executable, argument, native) in programs {
        for (scenario, input, expected, code, diagnostic) in [
            ("empty", Some(b"".as_slice()), Some(b"".as_slice()), 0, ""),
            (
                "half-even",
                Some(b"2.5\n3.5\n-2.5\n-3.5\n".as_slice()),
                Some(b"2\n4\n-2\n-4\n".as_slice()),
                0,
                "",
            ),
            (
                "signed-crlf",
                Some(b"+2.5\r\n-.5\n".as_slice()),
                Some(b"2\n0\n".as_slice()),
                0,
                "",
            ),
            (
                "safe-range",
                Some(b"9007199254740991\n-9007199254740991\n".as_slice()),
                Some(b"9007199254740991\n-9007199254740991\n".as_slice()),
                0,
                "",
            ),
            (
                "empty-line",
                Some(b"1\n\n2\n".as_slice()),
                None,
                1,
                "DataFailure EmptyFloat",
            ),
            (
                "unicode",
                Some(b"2.5\xf0\x9f\x99\x82\n".as_slice()),
                None,
                1,
                "InvalidFloat offset=3",
            ),
            (
                "parse-overflow",
                Some(b"1e309\n".as_slice()),
                None,
                1,
                "FloatParseOverflow",
            ),
            (
                "outside-int",
                Some(b"9007199254740992\n".as_slice()),
                None,
                1,
                "FloatOutsideIntRange",
            ),
            ("nan", Some(b"NaN\n".as_slice()), None, 1, "FloatNotFinite"),
            (
                "infinity",
                Some(b"-Infinity\n".as_slice()),
                None,
                1,
                "FloatNotFinite",
            ),
            (
                "invalid-utf8",
                Some(b"\xc3\xa9\xff".as_slice()),
                None,
                1,
                "InvalidUtf8 { offset: 2 }",
            ),
            ("missing", None, None, 1, "FileNotFound"),
            (
                "write-directory",
                Some(b"2.5\n".as_slice()),
                None,
                1,
                "IsADirectory",
            ),
        ] {
            let cwd = scratch.0.join(format!("{label}-{scenario}"));
            fs::create_dir(&cwd).unwrap();
            if let Some(input) = input {
                fs::write(cwd.join("input.txt"), input).unwrap();
            }
            let old = b"old output must survive failed read, parse and overflow";
            if scenario == "write-directory" {
                fs::create_dir(cwd.join("output.txt")).unwrap();
            } else {
                fs::write(cwd.join("output.txt"), old).unwrap();
            }
            let mut command = Command::new(&executable);
            command.current_dir(&cwd).env("PATH", "");
            if let Some(argument) = &argument {
                command.arg(argument);
            }
            if native {
                command.env("SESERAGI_NATIVE_METRICS", "1");
            }
            let result = bounded(&mut command, &cwd);
            assert_eq!(
                result.status.code(),
                Some(code),
                "{label}/{scenario}: {result:?}"
            );
            assert!(result.stdout.is_empty(), "{label}/{scenario}: {result:?}");
            let stderr = String::from_utf8_lossy(&result.stderr);
            if native {
                assert!(
                    stderr.contains("managed-live=0"),
                    "{label}/{scenario}: {stderr}"
                );
            }
            if code == 0 {
                assert_eq!(fs::read(cwd.join("output.txt")).unwrap(), expected.unwrap());
                if !native {
                    assert!(stderr.is_empty(), "{stderr}");
                }
            } else {
                if !diagnostic.is_empty() {
                    assert!(stderr.contains(diagnostic), "{label}/{scenario}: {stderr}");
                }
                if scenario != "write-directory" {
                    assert_eq!(fs::read(cwd.join("output.txt")).unwrap(), old);
                }
            }
            if let Some(input) = input {
                assert_eq!(fs::read(cwd.join("input.txt")).unwrap(), input);
            }
        }
    }
}

fn check_semantic_program(
    source: &Path,
    expected: &str,
    code: i32,
    reason: &str,
    trace: Option<&str>,
) {
    let scratch = Scratch::new();
    for (label, executable, argument, native) in build_programs(source, false, &scratch.0) {
        let cwd = scratch.0.join(format!("run-{label}"));
        fs::create_dir(&cwd).unwrap();
        let mut command = Command::new(executable);
        command.current_dir(&cwd).env("PATH", "");
        if let Some(argument) = argument {
            command.arg(argument);
        }
        if native {
            command.env("SESERAGI_NATIVE_METRICS", "1");
        }
        let result = bounded(&mut command, &cwd);
        assert_eq!(result.status.code(), Some(code), "{label}: {result:?}");
        assert_eq!(result.stdout, expected.as_bytes(), "{label}");
        let stderr = String::from_utf8_lossy(&result.stderr);
        if native {
            assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
            if code == 70 {
                assert!(stderr.contains(reason), "{label}: {stderr}");
            }
        } else if code == 70 {
            assert_eq!(stderr, "seseragi: runtime defect\n");
        } else {
            assert!(stderr.is_empty(), "{label}: {stderr}");
        }
        if let Some(trace) = trace {
            assert_eq!(
                fs::read(cwd.join("trace.txt")).unwrap(),
                trace.as_bytes(),
                "{label}"
            );
        }
    }
}

#[test]
fn float_conversion_values_saved_and_partial_functions_match_original_strict_ts_and_native() {
    check_semantic_program(
        &root().join("examples/spec/fixtures/rust-backend/float-conversion-values.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/float-conversion-values.stdout"),
        0,
        "",
        None,
    );
}

#[test]
fn float_conversion_file_preserves_inputs_and_failed_outputs() {
    check_file_entry(
        root().join("examples/spec/fixtures/rust-backend/filesystem-float-round.ssrg"),
        false,
    );
}

#[test]
fn float_conversion_path_package_keeps_frozen_imports_and_error_owners() {
    let path = root().join("examples/spec/fixtures/rust-backend-projects/float-round-file");
    let paths = [
        "seseragi.toml",
        "seseragi.lock",
        "src/main.ssrg",
        "vendor/rounder/seseragi.toml",
        "vendor/rounder/src/main.ssrg",
    ];
    let before: Vec<_> = paths
        .iter()
        .map(|p| std::fs::read(path.join(p)).unwrap())
        .collect();
    check_file_entry(path.clone(), true);
    for (p, original) in paths.iter().zip(before) {
        assert_eq!(std::fs::read(path.join(p)).unwrap(), original);
    }
}

#[test]
fn float_conversion_cold_replay_failure_and_defect_release_once() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/float-conversion-effects.ssrg");
    assert_eq!(original.matches("body \"2.5\"").count(), 1);
    for (mode, code, expected, reason) in [
        ("2.5", 0, "acquired\ncold\nreleased\n4\n", ""),
        (
            "nonfinite",
            0,
            "acquired\ncold\nreleased\nFailure: FloatNotFinite\n",
            "",
        ),
        (
            "range",
            0,
            "acquired\ncold\nreleased\nFailure: FloatOutsideIntRange\n",
            "",
        ),
        (
            "defect",
            70,
            "acquired\ncold\nreleased\n",
            "aggregate defect",
        ),
    ] {
        let path = scratch.0.join(format!("{mode}.ssrg"));
        std::fs::write(
            &path,
            original.replace("body \"2.5\"", &format!("body {mode:?}")),
        )
        .unwrap();
        check_semantic_program(&path, expected, code, reason, None);
    }
}

#[test]
fn conversion_evaluates_mode_and_value_once_across_saved_and_returned_stages() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/float-conversion-trace.ssrg");
    let events = [
        "mode", "value", "mode", "value", "value", "select", "mode", "value", "mode", "value",
    ];
    let core = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
        "float-conversion-trace.ssrg",
        "proof/float-conversion",
        source,
    ))
    .unwrap()
    .core_ir;
    let id = |name: &str| {
        format!(
            "v_{}",
            format!("proof/float-conversion::{name}")
                .bytes()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    };
    let mut rust = seseragi_backend_rust::emit_rust_module(&core, "probe")
        .unwrap()
        .source;
    rust.truncate(rust.rfind("fn main()").unwrap());
    // Observe only the three source helper entries. Rounding and conversion
    // selection, application, and sharing retain the compiler's original code.
    for (name, event) in [
        ("modeValue", "mode"),
        ("inputValue", "value"),
        ("selected", "select"),
    ] {
        let start = rust.find(&format!("fn {}_direct(", id(name))).unwrap();
        let body = start + rust[start..].find("-> V {").unwrap() + "-> V {".len();
        rust.insert_str(
            body,
            &format!(" TRACE.with(|t| t.borrow_mut().push({event:?})); "),
        );
    }
    rust.push_str(&format!("\nthread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}\nfn main() {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(native_text(&result), \"2:22:2:2.0\"); TRACE.with(|t| assert_eq!(&*t.borrow(), &{events:?})); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); eprintln!(\"managed-live=0\"); }}", id("probe")));
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
    let expected = serde_json::json!({"value":"2:22:2:2.0","trace":events});
    for (label, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(label);
        fs::create_dir(&output).unwrap();
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "float-conversion-trace.ssrg",
                "proof/float-conversion",
                source,
            )
            .with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        fs::write(output.join("original.ts"), &raw).unwrap();
        let mut traced = raw;
        for (name, event) in [
            ("modeValue", "mode"),
            ("inputValue", "value"),
            ("selected", "select"),
        ] {
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
