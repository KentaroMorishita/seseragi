#[path = "../../test-support/native_cache.rs"]
mod native_cache;

#[path = "../../test-support/native_runtime.rs"]
mod native_runtime;

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
            "seseragi-text-processing-{}-{}",
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
        native_cache::configure(command);
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
        seseragi_driver::CompileInput::new("text-processing.ssrg", "proof/text-processing", text)
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

fn scalar_boundaries() -> (String, String) {
    let mut checks = Vec::new();
    for text in ["", "a", "\0", "é", "A👍🏽e\u{301}\0", "\u{feff}\u{10ffff}"] {
        let scalars = text.chars().collect::<Vec<_>>();
        let size = scalars.len() as i64;
        let mut indices = vec![-9_007_199_254_740_991, -1];
        indices.extend(0..=size + 1);
        indices.push(9_007_199_254_740_991);
        for &start in &indices {
            let expected = if start >= 0 && start < size {
                format!("Just {:?}", scalars[start as usize])
            } else {
                "missing".into()
            };
            checks.push(format!("text.scalarAt ({start}) {text:?} == ({expected})"));
            for &end in &indices {
                let expected = if start >= 0 && start <= end && end <= size {
                    let selected = scalars[start as usize..end as usize]
                        .iter()
                        .collect::<String>();
                    format!("Right {selected}")
                } else {
                    format!(
                        "Left InvalidScalarRange {{ start: {start}, end: {end}, length: {size} }}"
                    )
                };
                checks.push(format!(
                    "describe (text.sliceScalars ({start}) ({end}) {text:?}) == {expected:?}"
                ));
            }
        }
    }
    for whitespace in "\t\n\u{b}\u{c}\r \u{85}\u{a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}".chars() {
        let input = format!("{whitespace}{whitespace}雪,{whitespace}🙂!\u{feff}\0{whitespace}{whitespace}");
        checks.push(format!("text.words {input:?} == [\"雪,\", \"🙂!\\u{{feff}}\\0\"]"));
        for (operation, expected) in [
            ("trimStart", format!("雪,{whitespace}🙂!\u{feff}\0{whitespace}{whitespace}")),
            ("trimEnd", format!("{whitespace}{whitespace}雪,{whitespace}🙂!\u{feff}\0")),
        ] {
            checks.push(format!("text.{operation} {input:?} == {expected:?}"));
        }
    }
    let mut source =
        "import * as text from \"std/text\"\nfn describe result: Either<text.TextSliceError, String> -> String = match result { Left error -> \"Left \" + show error; Right value -> \"Right \" + value }\n"
            .to_owned();
    let mut expected = String::new();
    let mut groups = Vec::new();
    for (group, checks) in checks.chunks(24).enumerate() {
        source.push_str(&format!("fn group{group} -> String = {{\n let missing: Maybe<Char> = Nothing\n text.join \"\\n\" [\n"));
        for (offset, check) in checks.iter().enumerate() {
            let index = group * 24 + offset;
            source.push_str(&format!("  \"case-{index}:\" + show ({check}),\n"));
            expected.push_str(&format!("case-{index}:True\n"));
        }
        source.push_str("]\n}\n");
        groups.push(format!("group{group} ()"));
    }
    source.push_str(&format!("pub fn probe -> String = text.join \"\\n\" [{}]\npub effect fn main = println (probe ())\n", groups.join(", ")));
    (source, expected)
}

#[test]
fn text_operations_match_strict_ts_and_native_lifetimes() {
    let (boundaries, expected_boundaries) = scalar_boundaries();
    let fixture = fs::read_to_string(
        root().join("examples/spec/fixtures/rust-backend/text-processing-effects.ssrg"),
    )
    .unwrap();
    assert_eq!(fixture.matches("body \"ready\"").count(), 1);
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/text-line-filter.ssrg");
    compare_text_cases([
        ("original", original.to_owned(), 0, "雫\n🙂\n\n"),
        (
            "values",
            include_str!("../../../examples/spec/fixtures/rust-backend/text-processing.ssrg")
                .to_owned(),
            0,
            include_str!("../../../examples/spec/fixtures/rust-backend/text-processing.stdout"),
        ),
        (
            "scalars",
            include_str!("../../../examples/spec/fixtures/rust-backend/text-scalars.ssrg")
                .to_owned(),
            0,
            include_str!("../../../examples/spec/fixtures/rust-backend/text-scalars.stdout"),
        ),
        (
            "scalar-boundaries",
            boundaries,
            0,
            expected_boundaries.as_str(),
        ),
        (
            "success",
            fixture.clone(),
            0,
            "acquired\ncold\nreleased\n雫\n🙂\n雫\n🙂\n\n",
        ),
        (
            "failure",
            fixture.replace("body \"ready\"", "body \"fail\""),
            0,
            "acquired\ncold\nreleased\nFailure: body failure\n",
        ),
        (
            "defect",
            fixture.replace("body \"ready\"", "body \"defect\""),
            70,
            "acquired\ncold\nreleased\n",
        ),
    ]);
}

#[test]
fn text_scalar_effects_keep_cold_replay_typed_failure_defect_and_cleanup() {
    let fixture =
        include_str!("../../../examples/spec/fixtures/rust-backend/text-scalars-effects.ssrg");
    assert_eq!(fixture.matches("body \"ready\"").count(), 1);
    compare_text_cases([
        (
            "scalar-success",
            fixture.to_owned(),
            0,
            "acquired\ncold\nreleased\n👍🏽e\u{301}!\n👍🏽e\u{301}!\n",
        ),
        (
            "scalar-failure",
            fixture.replace("body \"ready\"", "body \"fail\""),
            0,
            "acquired\ncold\nreleased\nFailure: InvalidScalarRange { start: -1, end: 7, length: 6 }\n",
        ),
        (
            "scalar-defect",
            fixture.replace("body \"ready\"", "body \"defect\""),
            70,
            "acquired\ncold\nreleased\n",
        ),
    ]);
}

#[test]
fn text_case_matches_strict_ts_and_native_at_unicode_and_sigma_boundaries() {
    let mut checks = Vec::new();
    for (prefix, lower_prefix, preceded) in [
        ("", "", false),
        ("A", "a", true),
        ("a", "a", true),
        ("1", "1", false),
        ("中", "中", false),
        ("\u{345}", "\u{345}", false),
        ("\u{2b0}", "\u{2b0}", false),
        ("A\u{345}", "a\u{345}", true),
        ("A\u{200d}", "a\u{200d}", true),
        ("A'", "a'", true),
        ("A ", "a ", false),
        ("\u{1fbe}", "\u{1fbe}", true),
        ("Ⅰ", "ⅰ", true),
        ("\u{a7ce}", "\u{a7cf}", true),
    ] {
        for (suffix, lower_suffix, followed) in [
            ("", "", false),
            ("A", "a", true),
            ("1", "1", false),
            ("中", "中", false),
            ("'", "'", false),
            ("'A", "'a", true),
            (" A", " a", false),
            ("\u{301}", "\u{301}", false),
            ("\u{345}", "\u{345}", false),
            ("\u{345}A", "\u{345}a", true),
            ("\u{200d}A", "\u{200d}a", true),
            ("\u{1fbe}", "\u{1fbe}", true),
            ("Ⅰ", "ⅰ", true),
            ("\u{a7ce}", "\u{a7cf}", true),
        ] {
            let source = format!("{prefix}Σ{suffix}");
            let sigma = if preceded && !followed { 'ς' } else { 'σ' };
            let expected = format!("{lower_prefix}{sigma}{lower_suffix}");
            checks.push(format!("text.toLower {source:?} == {expected:?}"));
        }
    }
    let marks = "\u{301}".repeat(20_000);
    for (source, expected) in [
        (format!("A{marks}Σ{marks}!"), format!("a{marks}ς{marks}!")),
        (format!("A{marks}Σ{marks}A"), format!("a{marks}σ{marks}a")),
    ] {
        checks.push(format!("text.toLower {source:?} == {expected:?}"));
    }
    let mut source = "import * as text from \"std/text\"\n".to_owned();
    let mut expected = String::new();
    let mut groups = Vec::new();
    for (group, checks) in checks.chunks(24).enumerate() {
        source.push_str(&format!(
            "fn group{group} -> String = text.join \"\\n\" [\n"
        ));
        for (offset, check) in checks.iter().enumerate() {
            let index = group * 24 + offset;
            source.push_str(&format!(" \"case-{index}:\" + show ({check}),\n"));
            expected.push_str(&format!("case-{index}:True\n"));
        }
        source.push_str("]\n");
        groups.push(format!("group{group} ()"));
    }
    source.push_str(&format!(
        "pub effect fn main = println (text.join \"\\n\" [{}])\n",
        groups.join(", ")
    ));
    compare_text_cases([
        (
            "case-values",
            include_str!("../../../examples/spec/fixtures/rust-backend/text-case.ssrg").to_owned(),
            0,
            include_str!("../../../examples/spec/fixtures/rust-backend/text-case.stdout"),
        ),
        ("case-contexts", source, 0, expected.as_str()),
    ]);
}

#[test]
fn text_case_keeps_cold_replay_failure_argument_defects_and_cleanup() {
    let fixture =
        include_str!("../../../examples/spec/fixtures/rust-backend/text-case-effects.ssrg");
    assert_eq!(fixture.matches("body \"ready\"").count(), 1);
    compare_text_cases([
        (
            "case-success",
            fixture.to_owned(),
            0,
            "acquired\ncold\nreleased\naς|SS|strasse\naς|SS|strasse\n",
        ),
        (
            "case-failure",
            fixture.replace("body \"ready\"", "body \"fail\""),
            0,
            "acquired\ncold\nreleased\nFailure: strasse\n",
        ),
        (
            "case-lower-defect",
            fixture.replace("body \"ready\"", "body \"lower-defect\""),
            70,
            "acquired\ncold\nreleased\n",
        ),
        (
            "case-upper-defect",
            fixture.replace("body \"ready\"", "body \"upper-defect\""),
            70,
            "acquired\ncold\nreleased\n",
        ),
        (
            "case-fold-defect",
            fixture.replace("body \"ready\"", "body \"fold-defect\""),
            70,
            "acquired\ncold\nreleased\n",
        ),
    ]);
}

fn compare_text_cases<'a>(cases: impl IntoIterator<Item = (&'a str, String, i32, &'a str)>) {
    let scratch = Scratch::new();
    for (label, text, code, expected) in cases {
        let source = scratch.0.join(format!("{label}.ssrg"));
        fs::write(&source, &text).unwrap();
        let native = scratch.0.join(format!("{label}-native"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let output = bounded(
            Command::new(native.join(if cfg!(windows) {
                "program.exe"
            } else {
                "program"
            }))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(output.status.code(), Some(code), "{label}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "{label}: {output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
        if code == 70 {
            let message = if matches!(
                label,
                "scalar-defect" | "case-lower-defect" | "case-upper-defect" | "case-fold-defect"
            ) {
                "Seseragi Int division by zero"
            } else {
                "body defect"
            };
            assert!(stderr.contains(message), "{stderr}");
        }
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let output = scratch.0.join(format!("{label}-{profile}"));
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&source)
                    .args(["--profile", profile, "--out-dir"])
                    .arg(&output),
                &scratch.0,
            ));
            check_raw_strict(&text, profile, value, &output, &scratch.0);
            for engine in ["bun", "node"] {
                let result = bounded(
                    Command::new(runtime(engine, &scratch.0))
                        .arg(output.join("entry.js"))
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
                    "{label}/{profile}/{engine}: {result:?}"
                );
                let stderr = String::from_utf8_lossy(&result.stderr);
                if code == 0 {
                    assert!(stderr.is_empty(), "{stderr}");
                } else {
                    assert_eq!(stderr, "seseragi: runtime defect\n");
                }
            }
        }
    }
}

#[test]
fn line_filter_preserves_inputs_and_typed_failure_outputs() {
    let scratch = Scratch::new();
    let source =
        root().join("examples/spec/fixtures/rust-backend/filesystem-text-line-filter.ssrg");
    let source_text = fs::read_to_string(&source).unwrap();
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
        native.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }),
        None,
        true,
    )];
    for profile in ["development", "release"] {
        let output = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = output.join(manifest["entry"].as_str().unwrap());
        check_raw_strict(
            &source_text,
            profile,
            if profile == "development" {
                seseragi_driver::BuildProfile::Development
            } else {
                seseragi_driver::BuildProfile::Release
            },
            &output,
            &scratch.0,
        );
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
        for scenario in [
            "unicode-lines",
            "empty",
            "only-breaks",
            "kept-content",
            "missing",
            "invalid-utf8",
            "write-directory",
        ] {
            let cwd = scratch.0.join(format!("{label}-{scenario}"));
            fs::create_dir(&cwd).unwrap();
            let input = match scenario {
                "invalid-utf8" => vec![0xc3, 0xa9, 0xff],
                "empty" => vec![],
                "only-breaks" => "\r\n\r\n\u{85}\u{2028}\u{2029}".as_bytes().to_vec(),
                "kept-content" => "  kept \0🙂\né\n".as_bytes().to_vec(),
                _ => "雫\r\n\r\n🙂\u{85}é\u{2028}\u{2029}終\r"
                    .as_bytes()
                    .to_vec(),
            };
            if scenario != "missing" {
                fs::write(cwd.join("input.txt"), &input).unwrap();
            }
            let old = b"old longer output must be replaced completely or preserved on read failure";
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
            assert!(result.stdout.is_empty(), "{label}/{scenario}: {result:?}");
            let stderr = String::from_utf8_lossy(&result.stderr);
            if native {
                assert!(
                    stderr.contains("managed-live=0"),
                    "{label}/{scenario}: {stderr}"
                );
            }
            if matches!(
                scenario,
                "unicode-lines" | "empty" | "only-breaks" | "kept-content"
            ) {
                assert!(result.status.success(), "{label}/{scenario}: {stderr}");
                let expected: &[u8] = match scenario {
                    "empty" | "only-breaks" => b"",
                    "kept-content" => "  kept \0🙂\né\n".as_bytes(),
                    _ => "雫\n🙂\né\n終\n".as_bytes(),
                };
                assert_eq!(fs::read(cwd.join("output.txt")).unwrap(), expected);
            } else {
                assert_eq!(
                    result.status.code(),
                    Some(1),
                    "{label}/{scenario}: {stderr}"
                );
                let kind = match scenario {
                    "missing" => "FileNotFound",
                    "invalid-utf8" => "InvalidUtf8 { offset: 2 }",
                    _ => "IsADirectory",
                };
                assert!(stderr.contains(kind), "{label}/{scenario}: {stderr}");
                assert!(!stderr.contains("ProviderPackageDefect"));
                if scenario != "write-directory" {
                    assert_eq!(fs::read(cwd.join("output.txt")).unwrap(), old);
                }
            }
            if scenario != "missing" {
                assert_eq!(fs::read(cwd.join("input.txt")).unwrap(), input);
            }
        }
    }
}

#[test]
fn standard_web_program_text_line_filter_matches_native_and_strict_bridge() {
    let scratch = Scratch::new();
    let package = root()
        .join("examples/spec/fixtures/rust-backend-projects/program-wasm-text-line-filter-package");
    let selected = seseragi_project::load_local_project_module(
        &package,
        &seseragi_project::ModulePath::parse("processor").unwrap(),
    )
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&selected).unwrap();
    let modules = core
        .compiled
        .order
        .iter()
        .map(|id| &core.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let native =
        seseragi_backend_rust::emit_rust_project(&modules, &core.entry_module, "probe").unwrap();
    let rust = scratch.0.join("oracle.rs");
    let binary = scratch
        .0
        .join(format!("oracle{}", std::env::consts::EXE_SUFFIX));
    fs::write(&rust, native.source).unwrap();
    successful(bounded(
        native_runtime::rustc()
            .args(["--edition=2021", "-O", "-o"])
            .arg(&binary)
            .arg(&rust),
        &scratch.0,
    ));
    let result = successful(bounded(
        Command::new(&binary)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    ));
    assert_eq!(result.stdout, "prefix:雫\n🙂\n\n".as_bytes());
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    for profile in ["development", "release"] {
        let site = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&package)
                .args(["--target", "web", "--profile", profile, "--out-dir"])
                .arg(&site),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["programs"].as_array().unwrap().len(), 1);
        let program = &manifest["programs"][0];
        assert_eq!(program["hostImport"], "@seseragi/programs/processor");
        assert_eq!(program["wasm"], "programs/processor/program.wasm");
        let directory = site.join("programs/processor");
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":[],"lib":["ES2022","DOM"],"paths":{"@seseragi/programs/processor":[directory.join("index.d.mts")]}},"files":[package.join("host/bridge.ts")]});
        let config_path = scratch.0.join(format!("bridge-{profile}.json"));
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("-p")
                .arg(&config_path),
            &scratch.0,
        ));
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(root().join("scripts/probes/program-wasm-text-line-filter-probe.mjs"))
                    .arg(&directory),
                &scratch.0,
            ));
            let proof: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(proof["calls"], 18);
            assert_eq!(proof["managedLiveAtClose"], 0);
            assert_eq!(proof["callbacks"], 0);
        }
    }
}

#[test]
fn text_processing_evaluates_source_arguments_once_in_order() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/text-processing-trace.ssrg");
    let core = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
        "text-trace.ssrg",
        "native/text-trace",
        source,
    ))
    .unwrap()
    .core_ir;
    for (entry, value, names) in [
        ("linesProbe", "a|b", vec!["lineValue"]),
        ("joinProbe", "a|b", vec!["separatorValue", "itemsValue"]),
        (
            "partialProbe",
            "a|b",
            vec!["separatorValue", "marker", "itemsValue"],
        ),
    ] {
        let mut rust = seseragi_backend_rust::emit_rust_module(&core, entry)
            .unwrap()
            .source;
        rust.truncate(rust.rfind("fn main()").unwrap());
        let id = |name: &str| {
            format!(
                "v_{}",
                format!("native/text-trace::{name}")
                    .bytes()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
        };
        // Instrument source helper entries only. The generated text
        // calls and partial application stages remain untouched.
        for name in ["lineValue", "separatorValue", "itemsValue", "marker"] {
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
            native_runtime::rustc()
                .args(["--edition=2021", "-O", "-o"])
                .arg(&binary)
                .arg(&file),
            &scratch.0,
        ));
        successful(bounded(Command::new(&binary).env("PATH", ""), &scratch.0));
    }
    let expected = serde_json::json!([
        ["a|b", ["lineValue"]],
        ["a|b", ["separatorValue", "itemsValue"]],
        ["a|b", ["separatorValue", "marker", "itemsValue"]]
    ]);
    for (label, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(label);
        fs::create_dir(&output).unwrap();
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("text-trace.ssrg", "native/text-trace", source)
                .with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        fs::write(output.join("original.ts"), &raw).unwrap();
        let mut traced = raw;
        for name in ["lineValue", "separatorValue", "itemsValue", "marker"] {
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
        traced.push_str("\nconst $trace: string[] = [];\nconst observed = [linesProbe, joinProbe, partialProbe].map(fn => { $trace.length = 0; return [fn(undefined), [...$trace]]; });\nconsole.log(JSON.stringify(observed));\n");
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
