use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
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
    fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::new(&self.executable);
        if let Some(entry) = &self.entry {
            command.arg(entry);
        } else {
            command.env("SESERAGI_NATIVE_METRICS", "1");
        }
        command.current_dir(cwd).env("PATH", "");
        command
    }

    fn check(&self, output: Output, expected: &[u8]) {
        let output = successful(output);
        assert_eq!(output.stdout, expected, "{}", self.name);
        if self.entry.is_none() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
                "{}: {output:?}",
                self.name
            );
        } else {
            assert!(output.stderr.is_empty(), "{}: {output:?}", self.name);
        }
    }
}

fn programs(source: &Path, directory: &Path) -> Vec<Program> {
    let input = process::copy_input(source, directory);
    let source = input.as_path();
    let native = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        directory,
    ));
    let mut programs = vec![Program {
        name: "native".into(),
        executable: native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        entry: None,
    }];
    let original = fs::read_to_string(source).unwrap();
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let dist = directory.join(label);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", label, "--out-dir"])
                .arg(&dist),
            directory,
        ));
        let generated = compile_module(
            CompileInput::new("process.ssrg", "single-file/main", &original).with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        let raw = dist.join("original.ts");
        fs::write(&raw, generated).unwrap();
        let config = dist.join("tsconfig.json");
        fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({
                "compilerOptions": {
                    "strict": true, "noEmit": true, "skipLibCheck": false,
                    "allowImportingTsExtensions": true, "target": "ES2022",
                    "module": "ESNext", "moduleResolution": "Bundler",
                    "types": ["bun", "node"],
                    "typeRoots": [root().join("node_modules/@types")],
                    "baseUrl": root(),
                    "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                    "lib": ["ES2023", "DOM", "DOM.Iterable"]
                },
                "files": [raw]
            }))
            .unwrap(),
        )
        .unwrap();
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

fn context_expected(arguments: &[OsString], value: &OsString, cwd: &Path) -> Vec<u8> {
    let arguments = format!(
        "[{}]",
        arguments
            .iter()
            .map(|argument| argument.to_string_lossy())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let cwd = cwd.to_string_lossy().replace('\\', "/");
    format!(
        "{arguments}\n{arguments}\nJust {}\nJust \nNothing\nerror: InvalidEnvironmentName \nerror: InvalidEnvironmentName bad\0name\nNothing\nJust Unicode value 雪🙂\n{cwd}\n{cwd}\n{arguments}\n",
        value.to_string_lossy()
    )
    .into_bytes()
}

fn compare_context(
    programs: &[Program],
    directory: &Path,
    arguments: &[OsString],
    value: &OsString,
) {
    let expected = context_expected(arguments, value, directory);
    for program in programs {
        let output = bounded(
            program
                .command(directory)
                .args(arguments)
                .env("SESERAGI_PROCESS_VALUE", value)
                .env("SESERAGI_PROCESS_EMPTY", "")
                .env_remove("SESERAGI_PROCESS_MISSING")
                .env("SESERAGI_PROCESS_雪🙂", "Unicode value 雪🙂"),
            directory,
        );
        program.check(output, &expected);
    }
}

#[test]
fn process_context_matches_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path()).unwrap();
    let source = root().join("examples/spec/fixtures/rust-backend/process-context.ssrg");
    let mut programs = programs(&source, &directory);
    let saved = directory.join("saved.ssrg");
    fs::write(
        &saved,
        fs::read_to_string(&source)
            .unwrap()
            .replace(
                "(\\unit: Unit -> process.arguments unit)",
                "process.arguments",
            )
            .replace(
                "(\\name: String -> process.environment name)",
                "process.environment",
            )
            .replace(
                "(\\unit: Unit -> process.currentDirectory unit)",
                "process.currentDirectory",
            ),
    )
    .unwrap();
    let saved_native = directory.join("saved-native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&saved)
            .args(["--entry", "main", "--out-dir"])
            .arg(&saved_native),
        &directory,
    ));
    programs.push(Program {
        name: "native-saved-references".into(),
        executable: saved_native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        entry: None,
    });
    let cwd = directory.join("working directory 雪🙂");
    fs::create_dir(&cwd).unwrap();
    compare_context(&programs, &cwd, &[], &OsString::from("available 雪🙂"));
    compare_context(
        &programs,
        &cwd,
        &[
            "first",
            "",
            "--",
            "--flag=value",
            "two words",
            "雪🙂",
            "a\nb",
            &"x".repeat(2048),
        ]
        .map(Into::into),
        &OsString::from("line one\nline two=雪🙂"),
    );
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        compare_context(
            &programs,
            &cwd,
            &[OsString::from_vec(b"bad\xff\xed\xa0\x80".to_vec())],
            &OsString::from_vec(b"value\xff\xed\xa0\x80".to_vec()),
        );
        let backslash = directory.join("back\\slash");
        fs::create_dir(&backslash).unwrap();
        compare_context(&programs, &backslash, &[], &OsString::from("backslash"));
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::ffi::OsStringExt;
        let invalid = directory.join(OsString::from_vec(b"directory\xff".to_vec()));
        fs::create_dir(&invalid).unwrap();
        compare_context(
            &programs,
            &invalid,
            &[],
            &OsString::from("invalid cwd bytes"),
        );
    }
}

#[cfg(unix)]
#[test]
fn deleted_directory_is_observed_when_each_cold_effect_runs() {
    use std::{
        io::Write,
        process::{Child, Stdio},
        time::{Duration, Instant},
    };
    struct Owned(Child);
    impl Drop for Owned {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path()).unwrap();
    let source =
        root().join("examples/spec/fixtures/rust-backend/process-directory-unavailable.ssrg");
    let programs = programs(&source, &directory);
    for profile in ["development", "release"] {
        let dist = directory.join(profile);
        seseragi_runtime::stage_typescript_package(&dist).unwrap();
        fs::write(
            dist.join("directory-entry.ts"),
            r#"import process from "node:process";
import { main } from "./original.ts";
import { run } from "@seseragi/runtime/effect";
import { liveProcess } from "@seseragi/runtime/process";
import { liveConsole } from "@seseragi/runtime/console";
import { createProcessStdin } from "@seseragi/runtime/stdin";
const stdin = createProcessStdin();
// Reset Node's startup cwd cache after loading modules, before deletion.
process.chdir(process.env.SESERAGI_PROCESS_TEST_DIRECTORY!);
try {
  const result = await run(main(undefined), {
    process: liveProcess, console: liveConsole, stdin
  });
  if (result.kind !== "success") {
    console.error(result);
    process.exitCode = 1;
  }
} finally {
  stdin.close();
}
"#,
        )
        .unwrap();
        let config = dist.join("tsconfig.json");
        let mut configuration: serde_json::Value =
            serde_json::from_slice(&fs::read(&config).unwrap()).unwrap();
        configuration["files"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(dist.join("directory-entry.ts")));
        fs::write(&config, serde_json::to_vec(&configuration).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config),
            &directory,
        ));
        successful(bounded(
            Command::new(runtime("bun", &directory))
                .current_dir(&dist)
                .args([
                    "build",
                    "directory-entry.ts",
                    "--target=node",
                    "--outfile=directory-entry.mjs",
                ]),
            &directory,
        ));
    }
    for program in programs {
        let cwd = directory.join(format!("removed-{}", program.name));
        fs::create_dir(&cwd).unwrap();
        let stdout = directory.join("child.stdout");
        let stderr = directory.join("child.stderr");
        let mut command = program.command(&cwd);
        if let Some(entry) = &program.entry {
            command = Command::new(&program.executable);
            command.arg(entry.parent().unwrap().join("directory-entry.mjs"));
            command.current_dir(&cwd).env("PATH", "");
        }
        let mut child = Owned(
            command
                .env("SESERAGI_PROCESS_TEST_DIRECTORY", &cwd)
                .stdin(Stdio::piped())
                .stdout(fs::File::create(&stdout).unwrap())
                .stderr(fs::File::create(&stderr).unwrap())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(15);
        while fs::read(&stdout).unwrap() != b"ready\n" {
            assert!(child.0.try_wait().unwrap().is_none(), "{}", program.name);
            assert!(Instant::now() < deadline, "{}", program.name);
            std::thread::sleep(Duration::from_millis(10));
        }
        fs::remove_dir(&cwd).unwrap();
        child
            .0
            .stdin
            .take()
            .unwrap()
            .write_all(b"continue\n")
            .unwrap();
        let status = loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "{}", program.name);
            std::thread::sleep(Duration::from_millis(10));
        };
        child.0.wait().unwrap();
        program.check(
            Output {
                status,
                stdout: fs::read(stdout).unwrap(),
                stderr: fs::read(stderr).unwrap(),
            },
            b"ready\nCurrentDirectoryUnavailable\nCurrentDirectoryUnavailable\n",
        );
    }
}
