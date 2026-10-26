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
            "seseragi-float-parse-{}-{}",
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
        seseragi_driver::CompileInput::new("float-parse.ssrg", "proof/float-parse", text)
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

#[test]
fn float_file_sum_matches_strict_ts_and_preserves_failed_outputs() {
    check_file_entry(
        root().join("examples/spec/fixtures/rust-backend/filesystem-float-aggregate.ssrg"),
        false,
    );
}

#[test]
fn float_path_package_sum_preserves_error_owners_and_strict_ts() {
    check_file_entry(
        root().join("examples/spec/fixtures/rust-backend-projects/float-file-aggregate"),
        true,
    );
}

fn check_file_entry(source: PathBuf, project: bool) {
    let scratch = Scratch::new();
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
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
    for (profile, value) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            &scratch.0,
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
                Command::new(runtime("node", &scratch.0))
                    .arg(root().join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(output.join("project-raw.json")),
                &scratch.0,
            ));
        } else {
            check_raw_strict(
                &fs::read_to_string(&source).unwrap(),
                profile,
                value,
                &output,
                &scratch.0,
            );
        }
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
        for (scenario, input, expected, code, diagnostic) in [
            (
                "empty",
                Some(b"".as_slice()),
                Some(b"0.0\n".as_slice()),
                0,
                "",
            ),
            (
                "sum625",
                Some(b"2.5\n3.75\n".as_slice()),
                Some(b"6.25\n".as_slice()),
                0,
                "",
            ),
            (
                "signed",
                Some(b"+2.5\r\n-.5\n".as_slice()),
                Some(b"2.0\n".as_slice()),
                0,
                "",
            ),
            (
                "exponents",
                Some(b"1e2\n.25\n".as_slice()),
                Some(b"100.25\n".as_slice()),
                0,
                "",
            ),
            (
                "max-finite",
                Some(b"1.7976931348623157e308\n".as_slice()),
                Some(b"1.7976931348623157e308\n".as_slice()),
                0,
                "",
            ),
            (
                "min-subnormal",
                Some(b"5e-324\n".as_slice()),
                Some(b"5e-324\n".as_slice()),
                0,
                "",
            ),
            (
                "zeros",
                Some(b"+0\n-0\n0\n".as_slice()),
                Some(b"0.0\n".as_slice()),
                0,
                "",
            ),
            (
                "leading-zero",
                Some(b"01\n".as_slice()),
                Some(b"1.0\n".as_slice()),
                0,
                "",
            ),
            (
                "nan",
                Some(b"NaN\n".as_slice()),
                Some(b"NaN\n".as_slice()),
                0,
                "",
            ),
            (
                "infinity",
                Some(b"Infinity\n".as_slice()),
                Some(b"Infinity\n".as_slice()),
                0,
                "",
            ),
            (
                "opposite-infinities",
                Some(b"Infinity\n-Infinity\n".as_slice()),
                Some(b"NaN\n".as_slice()),
                0,
                "",
            ),
            (
                "sum-overflow",
                Some(b"1.7976931348623157e308\n1.7976931348623157e308\n".as_slice()),
                Some(b"Infinity\n".as_slice()),
                0,
                "",
            ),
            (
                "empty-line",
                Some(b"1\n\n2\n".as_slice()),
                None,
                1,
                "NumberFailure EmptyFloat",
            ),
            (
                "sign-only",
                Some(b"+\n".as_slice()),
                None,
                1,
                "NumberFailure InvalidFloat offset=1",
            ),
            (
                "unicode",
                Some(b"2.5\xf0\x9f\x99\x82\n".as_slice()),
                None,
                1,
                "NumberFailure InvalidFloat offset=3",
            ),
            (
                "syntax-before-overflow",
                Some(b"1e999x\n".as_slice()),
                None,
                1,
                "NumberFailure InvalidFloat offset=5",
            ),
            (
                "range",
                Some(b"1e309\n".as_slice()),
                None,
                1,
                "NumberFailure FloatParseOverflow",
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
                Some(b"2.5\n3.75\n".as_slice()),
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

#[test]
fn float_parse_saved_cold_plan_failure_and_defect_release_once() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/float-parse-effects.ssrg");
    assert_eq!(original.matches("body \"2.5\"").count(), 1);
    for (label, source, code, expected) in [
        (
            "success",
            original.to_owned(),
            0,
            "acquired\ncold\nreleased\n5.0\n",
        ),
        (
            "failure",
            original.replace("body \"2.5\"", "body \"2.5x\""),
            0,
            "acquired\ncold\nreleased\nFailure: InvalidFloat offset=3\n",
        ),
        (
            "defect",
            original.replace("body \"2.5\"", "body \"defect\""),
            70,
            "acquired\ncold\nreleased\n",
        ),
    ] {
        let source_path = scratch.0.join(format!("{label}.ssrg"));
        fs::write(&source_path, &source).unwrap();
        let native = scratch.0.join(format!("{label}-native"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source_path)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let result = bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(result.status.code(), Some(code), "{label}: {result:?}");
        assert_eq!(result.stdout, expected.as_bytes());
        assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
        if code == 70 {
            assert!(String::from_utf8_lossy(&result.stderr).contains("aggregate defect"));
        }
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let output = scratch.0.join(format!("{label}-{profile}"));
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&source_path)
                    .args(["--target", "process", "--profile", profile, "--out-dir"])
                    .arg(&output),
                &scratch.0,
            ));
            check_raw_strict(&source, profile, value, &output, &scratch.0);
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for engine in ["bun", "node"] {
                let result = bounded(
                    Command::new(runtime(engine, &scratch.0))
                        .arg(output.join(manifest["entry"].as_str().unwrap()))
                        .env("PATH", ""),
                    &scratch.0,
                );
                assert_eq!(
                    result.status.code(),
                    Some(code),
                    "{label}/{profile}/{engine}: {result:?}"
                );
                assert_eq!(result.stdout, expected.as_bytes());
                if code == 0 {
                    assert!(result.stderr.is_empty());
                } else {
                    assert_eq!(result.stderr, b"seseragi: runtime defect\n");
                }
            }
        }
    }
}

#[test]
fn float_parse_evaluates_direct_and_saved_source_once() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/float-parse-trace.ssrg");
    let module = "native/float-parse-trace";
    let core = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
        "float-parse-trace.ssrg",
        module,
        source,
    ))
    .unwrap()
    .core_ir;
    let id = |name: &str| {
        format!(
            "v_{}",
            format!("{module}::{name}")
                .bytes()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    };
    for entry in ["directProbe", "savedProbe"] {
        let mut rust = seseragi_backend_rust::emit_rust_module(&core, entry)
            .unwrap()
            .source;
        rust.truncate(rust.rfind("fn main()").unwrap());
        let start = rust
            .find(&format!("fn {}_direct(", id("inputValue")))
            .unwrap();
        let body = start + rust[start..].find("-> V {").unwrap() + "-> V {".len();
        // Instrument the source helper only; leave the parse expression intact.
        rust.insert_str(
            body,
            " TRACE.with(|t| t.borrow_mut().push(\"inputValue\")); ",
        );
        rust.push_str(&format!("\nthread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}\nfn main() {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), \"2.5\"); TRACE.with(|t| assert_eq!(&*t.borrow(), &[\"inputValue\"])); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}", id(entry)));
        let file = scratch.0.join(format!("{entry}.rs"));
        let binary = scratch
            .0
            .join(format!("{entry}{}", std::env::consts::EXE_SUFFIX));
        fs::write(&file, rust).unwrap();
        successful(bounded(
            Command::new("rustc")
                .args(["--edition=2021", "-O", "-o"])
                .arg(&binary)
                .arg(&file),
            &scratch.0,
        ));
        successful(bounded(Command::new(&binary).env("PATH", ""), &scratch.0));
    }
    for (label, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(label);
        fs::create_dir(&output).unwrap();
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("float-parse-trace.ssrg", module, source)
                .with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        fs::write(output.join("original.ts"), &raw).unwrap();
        let mut traced = raw;
        let start = traced.find("const inputValue =").unwrap();
        let body = start + traced[start..].find("=>").unwrap() + 2;
        let line_end = body + traced[body..].find('\n').unwrap();
        let end = if traced[..line_end].ends_with(';') {
            line_end - 1
        } else {
            line_end
        };
        assert!(!traced[body..end].contains('{'));
        traced.insert_str(end, ")");
        traced.insert_str(body, " ($trace.push(\"inputValue\"), ");
        traced.push_str("\nconst $trace: string[] = [];\nconsole.log(JSON.stringify([directProbe, savedProbe].map(fn => { $trace.length = 0; return [fn(undefined), [...$trace]]; })));\n");
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
            let observed: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(
                observed,
                serde_json::json!([["2.5", ["inputValue"]], ["2.5", ["inputValue"]]]),
                "{label}/{engine}"
            );
        }
    }
}
