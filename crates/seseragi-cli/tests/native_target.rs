use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

fn executable(directory: &Path) -> std::path::PathBuf {
    directory.join(format!("program{}", std::env::consts::EXE_SUFFIX))
}

fn package(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("seseragi.toml"), "[package]\nname=\"fixture/native-entry\"\nversion=\"0.0.0\"\nlanguage=\"^0.1.0\"\n[run]\nentry=\"main\"\ntarget=\"process\"\n").unwrap();
}

#[test]
fn native_run_passes_stdin_stdout_stderr_and_exit_status() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let source = root.join("main.ssrg");
    fs::write(&source, "import * as effects from \"std/effect\"\npub effect fn main -> Unit with Stdin, Console fails String = do { line <- readLine () |> effects.mapError (\\_: StdinError -> \"input\"); println (show line) |> effects.mapError (\\_: ConsoleError -> \"output\"); effects.fail \"stopped\" }\n").unwrap();
    let input = root.join("input");
    fs::write(&input, "hello\n").unwrap();
    let run = |native| {
        let mut command = cli();
        command.arg("run").arg(&source);
        if native {
            command.args(["--target", "native"]);
        }
        bounded(
            command.stdin(Stdio::from(fs::File::open(&input).unwrap())),
            root,
        )
    };
    let ts = run(false);
    let native = run(true);
    assert_eq!(ts.status.code(), Some(1), "{ts:?}");
    assert_eq!(native.status.code(), ts.status.code(), "{native:?}");
    assert_eq!(native.stdout, b"Just hello\n");
    assert_eq!(native.stdout, ts.stdout);
    assert!(!native.stderr.is_empty());
    assert_eq!(native.stderr, ts.stderr);
    assert!(!root.join("dist").exists());
}

#[test]
fn native_build_uses_dist_or_out_dir_and_preserves_outputs_on_failure() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let source = root.join("main.ssrg");
    fs::write(&source, "pub effect fn main = println \"built\"\n").unwrap();
    successful(bounded(
        cli()
            .current_dir(root)
            .args(["build", "--target", "native", "main.ssrg"]),
        root,
    ));
    let dist = root.join("dist");
    assert_eq!(
        successful(bounded(&mut Command::new(executable(&dist)), root)).stdout,
        b"built\n"
    );
    let output = root.join("custom");
    successful(bounded(
        cli()
            .args(["build", "--target", "native", "--profile", "release"])
            .arg(&source)
            .arg("--out-dir")
            .arg(&output),
        root,
    ));
    let names = [
        "program.rs".to_owned(),
        "rust-backend.json".to_owned(),
        format!("program{}", std::env::consts::EXE_SUFFIX),
    ];
    let previous = names
        .iter()
        .map(|name| fs::read(output.join(name)).unwrap())
        .collect::<Vec<_>>();
    fs::write(&source, "pub effect fn main = missingName\n").unwrap();
    let failed = bounded(
        cli()
            .args(["build", "--target", "native"])
            .arg(&source)
            .arg("--out-dir")
            .arg(&output),
        root,
    );
    assert_eq!(failed.status.code(), Some(2), "{failed:?}");
    for (name, bytes) in names.iter().zip(previous) {
        assert_eq!(fs::read(output.join(name)).unwrap(), bytes);
    }
    assert_eq!(
        successful(bounded(&mut Command::new(executable(&output)), root)).stdout,
        b"built\n"
    );
}

#[test]
fn native_entry_ignores_same_name_method_and_facade_display_names() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    package(root);
    fs::write(root.join("src/model.ssrg"), "pub fn other -> Int = 99\n").unwrap();
    fs::write(
        root.join("src/facade.ssrg"),
        "pub import { other as main } from \"./model\"\n",
    )
    .unwrap();
    fs::write(root.join("src/main.ssrg"), "import { main as facadeMain } from \"./facade\"\nstruct Box { value: Int }\nimpl Box { fn main self: Box -> Int = self.value }\npub effect fn main = do { println (show (Box { value: 7 }).main); println (show (facadeMain ())) }\n").unwrap();
    successful(bounded(cli().args(["lock", "update"]).arg(root), root));
    let ts = successful(bounded(cli().arg("run").arg(root), root));
    let native = successful(bounded(
        cli()
            .args(["run", "--target", "native"])
            .arg(root.join("src/main.ssrg")),
        root,
    ));
    assert_eq!(native.stdout, b"7\n99\n");
    assert_eq!(native.stdout, ts.stdout);
    let standard = root.join("standard");
    successful(bounded(
        cli()
            .args(["build", "--target", "native", "--profile", "release"])
            .arg(root)
            .arg("--out-dir")
            .arg(&standard),
        root,
    ));
    assert_eq!(
        successful(bounded(&mut Command::new(executable(&standard)), root)).stdout,
        ts.stdout
    );
    let native = root.join("experimental");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(root)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        root,
    ));
    assert_eq!(
        successful(bounded(&mut Command::new(executable(&native)), root)).stdout,
        ts.stdout
    );
}

#[test]
fn missing_native_entry_has_source_location_and_does_not_select_a_method_or_reexport() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let source = root.join("main.ssrg");
    fs::write(
        &source,
        "struct Box { value: Int }\nimpl Box { fn main self: Box -> Int = self.value }\n",
    )
    .unwrap();
    let error = bounded(
        cli()
            .args(["build", "--target", "native"])
            .arg(&source)
            .arg("--out-dir")
            .arg(root.join("out")),
        root,
    );
    assert_missing(&error, &source);
    package(root);
    fs::write(
        root.join("src/model.ssrg"),
        "pub effect fn other = println \"wrong entry\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/main.ssrg"),
        "pub import { other as main } from \"./model\"\n",
    )
    .unwrap();
    successful(bounded(cli().args(["lock", "update"]).arg(root), root));
    let error = bounded(cli().args(["run", "--target", "native"]).arg(root), root);
    assert_missing(&error, &root.join("src/main.ssrg"));
}

fn assert_missing(error: &Output, source: &Path) {
    assert_eq!(error.status.code(), Some(2), "{error:?}");
    assert!(error.stdout.is_empty(), "{error:?}");
    let text = String::from_utf8_lossy(&error.stderr);
    assert!(
        text.contains(&format!("{}:0..0: SES-RUST-0001", source.display())),
        "{text}"
    );
    assert!(
        text.contains("entry function") && text.contains("not found"),
        "{text}"
    );
}

#[cfg(unix)]
mod watch {
    use super::*;
    use std::{
        process::Child,
        thread,
        time::{Duration, Instant},
    };

    struct Watcher(Child, std::path::PathBuf);
    impl Watcher {
        fn start(source: &Path, root: &Path) -> Self {
            use std::os::unix::process::CommandExt;
            let log = fs::File::create(root.join("watch.log")).unwrap();
            Self(
                cli()
                    .args(["dev", "--target", "native"])
                    .arg(source)
                    .process_group(0)
                    .stdout(log.try_clone().unwrap())
                    .stderr(log)
                    .spawn()
                    .unwrap(),
                root.join("watch.log"),
            )
        }
        fn stop(&mut self) {
            successful(
                Command::new("kill")
                    .args(["-INT", &self.0.id().to_string()])
                    .output()
                    .unwrap(),
            );
            wait(|| {
                self.0
                    .try_wait()
                    .unwrap()
                    .is_some_and(|status| status.success())
            });
            assert!(self.0.wait().unwrap().success());
        }
    }
    impl Drop for Watcher {
        fn drop(&mut self) {
            if thread::panicking() {
                eprintln!("{}", fs::read_to_string(&self.1).unwrap_or_default());
            }
            if self.0.try_wait().ok().flatten().is_none() {
                let _ = Command::new("kill")
                    .args(["-KILL", "--", &format!("-{}", self.0.id())])
                    .status();
                let _ = self.0.kill();
            }
            let _ = self.0.wait();
        }
    }
    fn wait(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(120);
        while !condition() {
            assert!(Instant::now() < deadline, "native watch deadline exceeded");
            thread::sleep(Duration::from_millis(100));
        }
    }
    fn log(root: &Path) -> String {
        fs::read_to_string(root.join("watch.log")).unwrap_or_default()
    }
    fn built(root: &Path) -> usize {
        log(root).matches("Built native app").count()
    }

    #[test]
    fn native_watch_rebuilds_entry_and_external_dependency_and_recovers_preserving_output() {
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path();
        let app = root.join("app");
        let dependency = root.join("dependency");
        package(&app);
        package(&dependency);
        fs::write(
            dependency.join("seseragi.toml"),
            "[package]\nname=\"fixture/dependency\"\nversion=\"0.0.0\"\nlanguage=\"^0.1.0\"\n[exports]\n\".\"=\"value\"\n",
        )
        .unwrap();
        fs::write(
            dependency.join("src/value.ssrg"),
            "pub fn message -> String = \"first\"\n",
        )
        .unwrap();
        let manifest = app.join("seseragi.toml");
        fs::write(&manifest, fs::read_to_string(&manifest).unwrap() + "\n[dependencies]\nvalue={ package=\"fixture/dependency\", path=\"../dependency\" }\n").unwrap();
        let entry = app.join("src/main.ssrg");
        fs::write(
            &entry,
            "import { message } from \"value\"\npub effect fn main = println (message ())\n",
        )
        .unwrap();
        successful(bounded(cli().args(["lock", "update"]).arg(&app), root));
        let mut watcher = Watcher::start(&app, root);
        wait(|| built(root) == 1);
        let output = app.join(".seseragi/dev");
        assert_eq!(
            successful(bounded(&mut Command::new(executable(&output)), root)).stdout,
            b"first\n"
        );
        let names = [
            "program.rs".to_owned(),
            "rust-backend.json".to_owned(),
            format!("program{}", std::env::consts::EXE_SUFFIX),
        ];
        let saved = names
            .iter()
            .map(|name| fs::read(output.join(name)).unwrap())
            .collect::<Vec<_>>();
        fs::write(
            dependency.join("src/value.ssrg"),
            "pub fn message -> String = missingName\n",
        )
        .unwrap();
        wait(|| log(root).contains("Build failed"));
        assert!(watcher.0.try_wait().unwrap().is_none());
        for (name, bytes) in names.iter().zip(saved) {
            assert_eq!(fs::read(output.join(name)).unwrap(), bytes);
        }
        fs::write(
            dependency.join("src/value.ssrg"),
            "pub fn message -> String = \"fixed\"\n",
        )
        .unwrap();
        wait(|| built(root) == 2);
        assert_eq!(
            successful(bounded(&mut Command::new(executable(&output)), root)).stdout,
            b"fixed\n"
        );
        fs::write(&entry, "import { message } from \"value\"\npub effect fn main = println (message () + \" entry\")\n").unwrap();
        wait(|| built(root) == 3);
        assert_eq!(
            successful(bounded(&mut Command::new(executable(&output)), root)).stdout,
            b"fixed entry\n"
        );
        watcher.stop();
    }

    #[test]
    fn native_file_watch_recovers_from_initial_compile_error() {
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path();
        let source = root.join("main.ssrg");
        fs::write(&source, "pub effect fn main = missingName\n").unwrap();
        let mut watcher = Watcher::start(&source, root);
        wait(|| log(root).contains("Build failed"));
        assert!(!executable(&root.join(".seseragi/dev")).exists());
        fs::write(&source, "pub effect fn main = println \"recovered\"\n").unwrap();
        wait(|| built(root) == 1);
        assert_eq!(
            successful(bounded(
                &mut Command::new(executable(&root.join(".seseragi/dev"))),
                root
            ))
            .stdout,
            b"recovered\n"
        );
        watcher.stop();
    }
}
