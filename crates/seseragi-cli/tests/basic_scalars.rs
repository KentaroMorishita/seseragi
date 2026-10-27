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
            "seseragi-basic-scalars-{}-{}",
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
        seseragi_driver::CompileInput::new("string-equality.ssrg", "proof/string-equality", text)
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
fn bool_unit_and_not_match_raw_strict_ts_native_and_cold_scope_lifetimes() {
    let fixture =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-scalar-effects.ssrg");
    let prefix = "acquired\nunit:True:True\ncreated\ncold\ncold\n";
    assert_eq!(fixture.matches("body \"ready\"").count(), 1);
    assert_eq!(fixture.matches("() == token ()").count(), 1);
    assert_eq!(fixture.matches("let inverted = !first").count(), 1);
    for (label, source, code, expected) in [
        (
            "success",
            fixture.to_owned(),
            0,
            format!("{prefix}flag:True:True\nrelease:True\ndone\n"),
        ),
        (
            "failure",
            fixture.replace("body \"ready\"", "body \"fail\""),
            0,
            format!("{prefix}flag:True:True\nrelease:True\nFailure: body failure\n"),
        ),
        (
            "defect",
            fixture.replace("body \"ready\"", "body \"defect\""),
            70,
            format!("{prefix}flag:True:True\nrelease:True\n"),
        ),
        (
            "unit-argument-defect",
            fixture.replace("() == token ()", "() == brokenToken ()"),
            70,
            "acquired\nrelease:True\n".into(),
        ),
        (
            "not-argument-defect",
            fixture.replace("let inverted = !first", "let inverted = !(brokenBool ())"),
            70,
            format!("{prefix}release:True\n"),
        ),
    ] {
        let scratch = Scratch::new();
        let input = scratch.0.join(format!("{label}.ssrg"));
        fs::write(&input, &source).unwrap();
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let output = scratch.0.join(profile);
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&input)
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
                assert_eq!(
                    result.stdout,
                    expected.as_bytes(),
                    "{label}/{profile}/{engine}"
                );
                assert_eq!(
                    result.stderr,
                    if code == 0 {
                        &b""[..]
                    } else {
                        &b"seseragi: runtime defect\n"[..]
                    },
                    "{label}/{profile}/{engine}"
                );
            }
        }
        let native = scratch.0.join("native");
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&input)
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
        assert_eq!(result.stdout, expected.as_bytes(), "{label}: {result:?}");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
        if label == "defect" {
            assert!(stderr.contains("body defect"), "{stderr}");
        }
    }
}

#[test]
fn wrong_nominal_owner_and_unprovided_float_eq_reject_before_output() {
    for source in [
        "struct Bool {value: Int}\npub effect fn main = println (show (!(Bool {value: 1})))\n",
        "struct Unit {value: Int}\npub effect fn main = println (show (eq () (Unit {value: 1})))\n",
        "pub effect fn main = println (show (eq 1.0 1.0))\n",
    ] {
        let scratch = Scratch::new();
        let input = scratch.0.join("invalid.ssrg");
        fs::write(&input, source).unwrap();
        for profile in ["development", "release"] {
            let output = scratch.0.join(profile);
            let result = bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&input)
                    .args(["--profile", profile, "--out-dir"])
                    .arg(&output),
                &scratch.0,
            );
            assert_eq!(result.status.code(), Some(2), "{result:?}");
            assert!(result.stdout.is_empty());
            assert!(!output.exists());
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("SES-T"),
                "{result:?}"
            );
        }
        let output = scratch.0.join("native");
        let result = bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&input)
                .args(["--entry", "main", "--out-dir"])
                .arg(&output),
            &scratch.0,
        );
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        assert!(result.stdout.is_empty());
        assert!(!output.exists());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("SES-T"),
            "{result:?}"
        );
    }
}

#[test]
fn basic_scalar_operations_evaluate_each_source_argument_once_in_order() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-scalar-trace.ssrg");
    let core = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
        "scalar-trace.ssrg",
        "native/trace",
        source,
    ))
    .unwrap()
    .core_ir;
    for (entry, value, names) in [
        ("boolBinary", "False", vec!["leftBool", "rightBool"]),
        ("boolNegated", "True", vec!["leftBool", "rightBool"]),
        ("unitBinary", "True", vec!["leftUnit", "rightUnit"]),
        ("unitNegated", "False", vec!["leftUnit", "rightUnit"]),
        (
            "boolPartial",
            "False",
            vec!["leftBool", "marker", "rightBool"],
        ),
        (
            "unitPartial",
            "True",
            vec!["leftUnit", "marker", "rightUnit"],
        ),
        ("notProbe", "False", vec!["leftBool"]),
    ] {
        let mut rust = seseragi_backend_rust::emit_rust_module(&core, entry)
            .unwrap()
            .source;
        rust.truncate(rust.rfind("fn main()").unwrap());
        let id = |name: &str| {
            format!(
                "v_{}",
                format!("native/trace::{name}")
                    .bytes()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
        };
        // Instrument source helper entries only. The generated equality
        // expressions and selected dictionary stages remain untouched.
        for name in ["leftBool", "rightBool", "leftUnit", "rightUnit", "marker"] {
            let start = rust.find(&format!("fn {}_direct(", id(name))).unwrap();
            let body = start + rust[start..].find("-> V {").unwrap() + "-> V {".len();
            rust.insert_str(
                body,
                &format!(" TRACE.with(|t| t.borrow_mut().push({name:?})); "),
            );
        }
        rust.push_str(&format!("\nthread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}\nfn main() {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), {value:?}); TRACE.with(|t| assert_eq!(&*t.borrow(), &{names:?})); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}", id(entry)));
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
    let expected = serde_json::json!([
        [false, ["leftBool", "rightBool"]],
        [true, ["leftBool", "rightBool"]],
        [true, ["leftUnit", "rightUnit"]],
        [false, ["leftUnit", "rightUnit"]],
        [false, ["leftBool", "marker", "rightBool"]],
        [true, ["leftUnit", "marker", "rightUnit"]],
        [false, ["leftBool"]]
    ]);
    for (label, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(label);
        fs::create_dir(&output).unwrap();
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("scalar-trace.ssrg", "native/trace", source)
                .with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        fs::write(output.join("original.ts"), &raw).unwrap();
        let mut traced = raw;
        for name in ["leftBool", "rightBool", "leftUnit", "rightUnit", "marker"] {
            let start = traced.find(&format!("const {name} =")).unwrap();
            let body = start + traced[start..].find("=>").unwrap() + 2;
            let line_end = body + traced[body..].find('\n').unwrap();
            let end = if traced[..line_end].ends_with(';') {
                line_end - 1
            } else {
                line_end
            };
            assert!(!traced[body..end].contains('{'));
            traced.insert_str(end, ")");
            traced.insert_str(body, &format!(" ($trace.push({name:?}), "));
        }
        traced.push_str("\nconst $trace: string[] = [];\nconst observed = [boolBinary, boolNegated, unitBinary, unitNegated, boolPartial, unitPartial, notProbe].map(fn => { $trace.length = 0; return [fn(undefined), [...$trace]]; });\nconsole.log(JSON.stringify(observed));\n");
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
                .args(["--project"])
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
            assert_eq!(observed, expected, "{label}/{engine}");
        }
    }
}
