use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
struct Program {
    name: String,
    executable: PathBuf,
    entry: Option<PathBuf>,
}
impl Program {
    fn command(&self, directory: &Path) -> Command {
        let mut command = Command::new(&self.executable);
        if let Some(entry) = &self.entry {
            command.arg(entry);
        }
        command
            .current_dir(directory)
            .env("SESERAGI_NATIVE_METRICS", "1");
        command
    }
    fn check(&self, output: std::process::Output, expected: &str) {
        assert!(output.status.success(), "{}: {output:?}", self.name);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            expected,
            "{}",
            self.name
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        if self.entry.is_none() {
            assert!(stderr.contains("managed-live=0"), "{}: {stderr}", self.name);
        } else {
            assert!(stderr.is_empty(), "{}: {stderr}", self.name);
        }
    }
}
fn native_program(source: &Path, directory: &Path) -> Program {
    let native = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        directory,
    ));
    Program {
        name: "native".into(),
        executable: native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        entry: None,
    }
}
fn programs(fixture: &str, directory: &Path) -> Vec<Program> {
    let source = process::copy_input(
        &root()
            .join("examples/spec/fixtures/rust-backend")
            .join(fixture),
        directory,
    );
    let mut programs = vec![native_program(&source, directory)];
    let package = directory.join("typescript-package");
    fs::create_dir_all(package.join("src")).unwrap();
    fs::copy(&source, package.join("src/main.ssrg")).unwrap();
    fs::write(package.join("seseragi.toml"), "[package]\nname = \"fixture/native-child-process\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        directory,
    ));
    if fixture == "child-process.ssrg" {
        let output = directory.join("native-package");
        fs::create_dir(&output).unwrap();
        let mut program = native_program(&package, &output);
        program.name = "native-package".into();
        programs.push(program);
    }
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let dist = directory.join(label);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&package)
                .args(["--target", "process", "--profile", label, "--out-dir"])
                .arg(&dist),
            directory,
        ));
        let original = fs::read_to_string(&source).unwrap();
        let generated = compile_module(
            CompileInput::new("child.ssrg", "single-file/main", &original).with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        let raw = dist.join("original.ts");
        fs::write(&raw, generated).unwrap();
        let config = dist.join("tsconfig.json");
        fs::write(&config, serde_json::to_vec(&serde_json::json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": false,
                "allowImportingTsExtensions": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "types": ["bun", "node"], "typeRoots": [root().join("node_modules/@types")],
                "baseUrl": root(), "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                "lib": ["ES2023", "DOM", "DOM.Iterable"]
            }, "files": [raw]
        })).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for engine in ["bun", "node"] {
            programs.push(Program {
                name: format!("{label}-{engine}"),
                executable: runtime(engine, directory),
                entry: Some(dist.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    programs
}

#[test]
fn child_configuration_matches_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let expected = CONFIG_EXPECTED;
    for program in programs("child-process-config.ssrg", scratch.path()) {
        program.check(
            bounded(&mut program.command(scratch.path()), scratch.path()),
            expected,
        );
    }
}

const CONFIG_EXPECTED: &str = concat!(
        "EmptyExecutableName\nExecutableNameContainsSeparator \"a/b\"\nExecutableNameContainsSeparator \"a\\\\b\"\nok\n",
        "ArgumentContainsNul { index: 1, offset: 3 }\nArgumentContainsNul { index: 2, offset: 2 }\nok\n",
        "EnvironmentNameContainsNul \"name\\u0000\"\nEnvironmentValueContainsNul \"value\\u0000\"\nEnvironmentNameContainsNul \"name\\u0000\"\n",
        "ok\nok\nInvalidCaptureLimit 0\nInvalidCaptureLimit -1\nok\nok\nok\n"
    );

#[test]
fn child_saved_configuration_stages_match_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    for program in programs(
        "../rust-backend-regressions/child-process-partials.ssrg",
        scratch.path(),
    ) {
        program.check(
            bounded(&mut program.command(scratch.path()), scratch.path()),
            CONFIG_EXPECTED,
        );
    }
}

const CHILD: &str = r#"
import fs from 'node:fs';
import { basename } from 'node:path';
const mode = process.argv[2];
const ready = () => fs.writeFileSync('child.pid', String(process.pid));
const alive = () => { process.on('SIGTERM', () => fs.writeFileSync('terminated', 'yes')); setInterval(() => {}, 1000); ready(); };
if (mode === 'reject-input') { alive(); fs.closeSync(0); }
else if (['cancel-captured', 'cancel-inherited', 'cancel-stream', 'stream-input-fail', 'after-close', 'signal', 'kill'].includes(mode)) {
  if (mode === 'signal' || mode === 'kill') setInterval(() => {}, 1000);
  else { alive(); if (mode === 'cancel-stream') process.stdout.write('ready'); }
  if (!mode.startsWith('cancel-') || mode === 'cancel-stream') process.stdin.resume();
} else if (mode === 'inherited') {
  process.stdout.write('inherited out:' + fs.readFileSync(0).toString() + '\n'); process.stderr.write('inherited err\n'); process.exitCode = 9;
} else {
  if (mode === 'duplex' || mode === 'stream-duplex') { process.stdout.write(Buffer.alloc(200000, 97)); process.stderr.write(Buffer.alloc(200000, 98)); }
  if (mode === 'repeat') fs.writeFileSync('runs', String(Number(fs.existsSync('runs') ? fs.readFileSync('runs') : 0) + 1));
  const input = [];
  process.stdin.on('data', chunk => input.push(chunk));
  process.stdin.on('end', () => {
    if (mode === 'exact') { process.stdout.write('1234'); process.stderr.write('5678'); }
    else if (mode === 'overflow-stdout') { alive(); process.stdout.write('12345'); }
    else if (mode === 'overflow-stderr') { alive(); process.stderr.write('12345'); }
    else if (mode === 'default-limit') { alive(); process.stdout.write(Buffer.alloc(8 * 1024 * 1024 + 1)); }
    else if (mode === 'arguments') process.stdout.write(JSON.stringify(process.argv.slice(3)));
    else if ((mode === 'directory' || mode === 'default-directory')) process.stdout.write(basename(process.cwd()));
    // libuv drops a leading '=' entry (the empty name) from process.env on POSIX,
    // so Node cannot observe it there; the spawn itself is still checked via A=B.
    else if (mode === 'environment') process.stdout.write((process.env[''] ?? (process.platform === 'win32' ? undefined : 'empty')) + '|' + process.env.A);
    else if (mode === 'context') process.stdout.write([process.env.SESERAGI_CHILD_VALUE, process.env.SESERAGI_CHILD_EMPTY === '', process.env.SESERAGI_CHILD_PARENT === undefined].join('|'));
    else if (mode === 'duplex' || mode === 'stream-duplex') { process.stdout.write(String(Buffer.concat(input).length)); }
    else if (mode !== 'empty') { process.stdout.write(Buffer.concat(input)); process.stderr.write('warn'); process.exitCode = 7; }
  });
}
"#;

#[cfg(unix)]
fn assert_reaped(directory: &Path) {
    if let Ok(pid) = fs::read_to_string(directory.join("child.pid")) {
        let status = Command::new("/bin/kill")
            .args(["-0", pid.trim()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(!status.success(), "child {pid} still exists");
        fs::remove_file(directory.join("child.pid")).unwrap();
    }
}
#[cfg(not(unix))]
fn assert_reaped(_: &Path) {}

#[test]
fn child_execution_matches_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path()).unwrap();
    let node = runtime("node", &directory);
    let script = directory.join("child.mjs");
    fs::write(&script, CHILD).unwrap();
    fs::create_dir(directory.join("working directory雪🙂")).unwrap();
    let mut cases = vec![
        ("echo", "ChildExited 7:68f09f998200:7761726e\n"),
        ("repeat", "ChildExited 7:68f09f998200:7761726e\n"),
        ("duplex", "ChildExited 0:200007:200000\n"),
        ("stream-duplex", "ChildExited 0:400014:400000\n"),
        ("reject-input", "error: input failed\n"),
        ("missing", "error: spawn failed\n"),
        ("exact", "ChildExited 0:31323334:35363738\n"),
        ("empty", "ChildExited 0::\n"),
        ("arguments", "ChildExited 0:5b22222c2274776f20776f726473222c22e99baaf09f9982222c223b6563686f20756e6578706563746564225d:\n"),
        ("environment", "ChildExited 0:656d7074797c423d6571:\n"),
        ("default-directory", "ChildExited 0:776f726b696e67206469726563746f7279e99baaf09f9982:\n"),
        ("directory", "ChildExited 0:776f726b696e67206469726563746f7279e99baaf09f9982:\n"),
        ("context", "ChildExited 0:e99baaf09f99827c747275657c74727565:\n"),
        ("stream", "ChildExited 7:68f09f998200:7761726e\n"),
        ("stream-input-fail", "error: input: broken input\n"),
        ("after-close", "error: child: ChildInputAfterClose\n"),
        ("overflow-stdout", "error: ChildOutputLimitExceeded { channel: ChildStdout, limitBytes: 4 }\n"),
        ("overflow-stderr", "error: ChildOutputLimitExceeded { channel: ChildStderr, limitBytes: 4 }\n"),
        ("default-limit", "error: ChildOutputLimitExceeded { channel: ChildStdout, limitBytes: 8388608 }\n"),
        ("bad-directory", "error: spawn failed\n"),
        ("nul-name", "error: spawn failed\n"),
        ("no-path", "error: ChildSpawnFailed { executable: SearchPath \"node\", detail: \"SearchPath executable requires PATH in the child environment\" }\n"),
        ("cancel-stream", "pending:7265616479:\n"),
    ];
    #[cfg(unix)]
    {
        cases.extend([
            ("signal", "ChildSignaled Terminate::\n"),
            ("kill", "ChildHostTerminated \"SIGKILL\"::\n"),
            ("relative", "ChildExited 7:68f09f998200:7761726e\n"),
            ("permission", "error: spawn failed\n"),
        ]);
        fs::write(
            directory.join("not-executable"),
            "no executable permissions",
        )
        .unwrap();
        std::os::unix::fs::symlink(&node, directory.join("child-node")).unwrap();
        fs::create_dir_all(directory.join("other/target")).unwrap();
        std::os::unix::fs::symlink("other/target", directory.join("link")).unwrap();
        cases.push(("relative-parent", "ChildExited 7:68f09f998200:7761726e\n"));
    }
    let mut failures = Vec::new();
    for program in programs("child-process.ssrg", &directory) {
        for (mode, expected) in &cases {
            let output = bounded(
                program
                    .command(&directory)
                    .arg(&node)
                    .arg(&script)
                    .arg(mode)
                    .env("PATH", "")
                    .env("SESERAGI_CHILD_PARENT", "parent")
                    .env(
                        "SESERAGI_APPLICATION_ROOT",
                        directory.join("working directory雪🙂"),
                    ),
                &directory,
            );
            if std::panic::catch_unwind(|| program.check(output, expected)).is_err() {
                failures.push(format!("{} / {mode}", program.name));
            }
            assert_reaped(&directory);
            if *mode == "repeat" {
                assert_eq!(fs::read_to_string(directory.join("runs")).unwrap(), "2");
                fs::remove_file(directory.join("runs")).unwrap();
            }
            #[cfg(unix)]
            if *mode == "cancel-stream" {
                assert_eq!(
                    fs::read_to_string(directory.join("terminated")).unwrap(),
                    "yes"
                );
            }
            let _ = fs::remove_file(directory.join("terminated"));
        }
        let output = bounded(
            program
                .command(&directory)
                .arg(&node)
                .arg(&script)
                .arg("search")
                .env("PATH", node.parent().unwrap()),
            &directory,
        );
        program.check(output, "ChildExited 7:68f09f998200:7761726e\n");
        let input = directory.join("inherited-input");
        fs::write(&input, "forwarded input").unwrap();
        let output = successful(bounded(
            program
                .command(&directory)
                .arg(&node)
                .arg(&script)
                .arg("inherited")
                .stdin(fs::File::open(input).unwrap()),
            &directory,
        ));
        assert_eq!(
            output.stdout,
            b"inherited out:forwarded input\nChildExited 9\n"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.starts_with("inherited err\n"),
            "{}: {stderr}",
            program.name
        );
        if program.entry.is_none() {
            assert!(stderr.contains("managed-live=0"));
        } else {
            assert_eq!(stderr, "inherited err\n");
        }
    }
    assert!(
        failures.is_empty(),
        "TS/native comparisons failed: {failures:?}"
    );
}

#[test]
fn child_cancellation_matches_ts_and_native_with_blocked_input_and_inherited_stdio() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path()).unwrap();
    let node = runtime("node", &directory);
    let script = directory.join("child.mjs");
    fs::write(&script, CHILD).unwrap();
    for program in programs("child-process-cancellation.ssrg", &directory) {
        for mode in ["cancel-captured", "cancel-inherited"] {
            let output = bounded(
                program
                    .command(&directory)
                    .arg(&node)
                    .arg(&script)
                    .arg(mode),
                &directory,
            );
            program.check(output, "cancelled\n");
            assert_reaped(&directory);
            #[cfg(unix)]
            assert_eq!(
                fs::read_to_string(directory.join("terminated")).unwrap(),
                "yes"
            );
            let _ = fs::remove_file(directory.join("terminated"));
        }
    }
}

#[test]
fn child_callable_references_and_saved_stages_match_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path()).unwrap();
    let node = runtime("node", &directory);
    let script = directory.join("child.mjs");
    fs::write(&script, CHILD).unwrap();
    let input = directory.join("empty-stdin");
    fs::write(&input, "").unwrap();
    let expected = concat!(
        "ChildExited 7:7361766564:7761726e\n",
        "ChildExited 7:7361766564:7761726e\n",
        "ChildExited 7:7361766564:7761726e\n",
        "ChildExited 0\n",
    );
    for program in programs("child-process-callables.ssrg", &directory) {
        let output = bounded(
            program
                .command(&directory)
                .arg(&node)
                .arg(&script)
                .arg("saved")
                .stdin(fs::File::open(&input).unwrap()),
            &directory,
        );
        program.check(output, expected);
    }
}

#[test]
fn native_child_provider_pins_are_rejected_before_building() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path();
    fs::create_dir(directory.join("src")).unwrap();
    fs::write(
        directory.join("src/main.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/child-process.ssrg"),
    )
    .unwrap();
    fs::write(
        directory.join("seseragi.toml"),
        "[package]\nname = \"fixture/native-child-provider\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n[providers]\n\"std/child-process::ChildProcesses\" = \"seseragi/runtime-bun#child-process\"\n",
    )
    .unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(directory),
        directory,
    ));
    let output = directory.join("native");
    let refused = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(directory)
            .args(["--entry", "main", "--out-dir"])
            .arg(&output),
        directory,
    );
    assert!(!refused.status.success());
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains(
            "unsupported native ChildProcesses provider selection/artifact configuration"
        ),
        "{refused:?}"
    );
    assert!(!output.exists());
}
