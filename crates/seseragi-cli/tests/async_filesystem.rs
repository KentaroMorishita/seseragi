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
            "seseragi-async-files-{}-{}",
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

struct Program {
    name: String,
    executable: PathBuf,
    argument: Option<PathBuf>,
}
impl Program {
    fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::new(&self.executable);
        if let Some(argument) = &self.argument {
            command.arg(argument);
        }
        command.current_dir(cwd).env("PATH", "");
        if self.argument.is_none() {
            command.env("SESERAGI_NATIVE_METRICS", "1");
        }
        command
    }
    fn check(&self, directory: &Path, output: Output, case: &str, trace: &str) {
        let exit = match case {
            "success" | "cancel" => 0,
            "typed-failure" => 1,
            "worker-defect" | "parent-defect" => 70,
            _ => unreachable!(),
        };
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{}: {output:?}",
            self.name
        );
        assert!(output.stdout.is_empty(), "{}: {output:?}", self.name);
        assert_eq!(read_trace(directory), trace, "{}: {output:?}", self.name);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if self.argument.is_none() {
            assert!(
                stderr.contains("managed-live=0"),
                "{}: {output:?}",
                self.name
            );
        }
        if case == "typed-failure" {
            assert!(stderr.contains("InvalidUtf8"), "{}: {output:?}", self.name);
        }
        if exit == 70 {
            assert!(
                stderr.contains(if self.argument.is_none() {
                    "Int overflow"
                } else {
                    "runtime defect"
                }),
                "{}: {output:?}",
                self.name
            );
        }
        let expected = if case == "success" {
            "processed:\n初期値\n42\n".as_bytes()
        } else {
            OLD
        };
        assert_eq!(
            fs::read(directory.join("output.txt")).unwrap(),
            expected,
            "{}",
            self.name
        );
    }
}
const OLD: &[u8] = b"old output must survive failure/cancellation";
fn read_trace(directory: &Path) -> String {
    fs::read_to_string(directory.join("trace.txt")).unwrap_or_default()
}
fn payload(case: &str) -> &[u8] {
    match case {
        "success" => "初期値\n42\n".as_bytes(),
        "typed-failure" => b"\xff",
        "worker-defect" => b"defect",
        "parent-defect" => b"unused",
        _ => unreachable!(),
    }
}

#[test]
fn asynchronous_files_preserve_results_failures_and_cleanup_on_native_bun_and_node() {
    let scratch = Scratch::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("examples/spec/fixtures/rust-backend/filesystem-async.ssrg");
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let mut programs = vec![Program {
        name: "native".into(),
        executable: native.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }),
        argument: None,
    }];
    for profile in ["development", "release"] {
        let dist = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&dist),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for name in ["bun", "node"] {
            programs.push(Program {
                name: format!("{profile}-{name}"),
                executable: runtime(name, &scratch.0),
                argument: Some(dist.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    for program in programs {
        for case in ["success", "typed-failure", "worker-defect"] {
            let directory = scratch.0.join(format!("{}-regular-{case}", program.name));
            fs::create_dir(&directory).unwrap();
            fs::write(directory.join("input.txt"), payload(case)).unwrap();
            fs::write(directory.join("control.txt"), "continue").unwrap();
            fs::write(directory.join("finish.txt"), "").unwrap();
            fs::write(directory.join("output.txt"), OLD).unwrap();
            let output = bounded(&mut program.command(&directory), &directory);
            let trace = format!(
                "scope-open\nscope-close\nreleased\n{}",
                if case == "success" { "joined\n" } else { "" }
            );
            program.check(&directory, output, case, &trace);
        }
        #[cfg(unix)]
        for case in [
            "success",
            "cancel",
            "typed-failure",
            "worker-defect",
            "parent-defect",
        ] {
            fifo::compare(&program, &scratch.0, case);
        }
    }
}

// Real OS I/O, held by an external producer. Neither reader receives data until
// both are open. The masked finalizer holds the process alive while we prove
// the data reader is closed. Windows instead exercises real exclusive files
// at the native host seam, plus the regular-file programs above.
#[cfg(unix)]
mod fifo {
    use super::*;
    use std::{
        ffi::CString,
        fs::OpenOptions,
        io::Write,
        os::unix::{ffi::OsStrExt, fs::OpenOptionsExt},
    };

    fn make(path: &Path) {
        unsafe extern "C" {
            fn mkfifo(path: *const std::ffi::c_char, mode: u32) -> i32;
        }
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { mkfifo(path.as_ptr(), 0o600) }, 0);
    }
    fn writer(path: &Path) -> Option<File> {
        let nonblock = if cfg!(target_os = "macos") {
            0x4
        } else {
            0x800
        };
        match OpenOptions::new()
            .write(true)
            .custom_flags(nonblock)
            .open(path)
        {
            Ok(file) => Some(file),
            Err(error) if error.raw_os_error() == Some(6) => None, // ENXIO: no reader
            Err(error) => panic!("opening FIFO writer: {error}"),
        }
    }
    fn until<T>(child: &mut OwnedChild, label: &str, mut observe: impl FnMut() -> Option<T>) -> T {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(value) = observe() {
                return value;
            }
            if child.child.try_wait().unwrap().is_some() {
                if let Some(value) = observe() {
                    return value;
                }
                panic!(
                    "{label}: child exited\n{}",
                    fs::read_to_string(&child.stderr).unwrap()
                );
            }
            assert!(
                Instant::now() < deadline,
                "{label}: owned PID {} in {}\ntrace: {:?}\nstderr: {}",
                child.child.id(),
                child.stderr.parent().unwrap().display(),
                read_trace(child.stderr.parent().unwrap()),
                fs::read_to_string(&child.stderr).unwrap_or_default(),
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    pub(super) fn compare(program: &Program, scratch: &Path, case: &str) {
        let directory = scratch.join(format!("{}-fifo-{case}", program.name));
        fs::create_dir(&directory).unwrap();
        for name in ["input.txt", "control.txt", "finish.txt"] {
            make(&directory.join(name));
        }
        fs::write(directory.join("output.txt"), OLD).unwrap();
        let mut child = OwnedChild::start(&mut program.command(&directory), &directory);
        let mut data = None;
        let mut control = None;
        until(
            &mut child,
            "both actual readers must start before either receives data",
            || {
                if data.is_none() {
                    data = writer(&directory.join("input.txt"));
                }
                if control.is_none() {
                    control = writer(&directory.join("control.txt"));
                }
                (data.is_some() && control.is_some()).then_some(())
            },
        );
        let mut data = data;
        let mut control = control.unwrap();
        control
            .write_all(if case == "cancel" {
                b"cancel"
            } else if case == "parent-defect" {
                b"parent-defect"
            } else {
                b"continue"
            })
            .unwrap();
        drop(control);
        if case == "cancel" {
            until(
                &mut child,
                "parent must request cancellation while data read is pending",
                || {
                    read_trace(&directory)
                        .contains("cancel-request\n")
                        .then_some(())
                },
            );
            // Cancellation is unavailable for the current OS call. Release one
            // read, without EOF, so the next chunk checkpoint can observe it.
            if let Err(error) = data.as_mut().unwrap().write_all(b"x") {
                assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
            }
        } else {
            if case == "parent-defect" {
                until(
                    &mut child,
                    "parent defect while child I/O is pending",
                    || {
                        read_trace(&directory)
                            .contains("parent-stop\n")
                            .then_some(())
                    },
                );
            }
            if let Err(error) = data.as_mut().unwrap().write_all(payload(case)) {
                assert_eq!(case, "parent-defect");
                assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
            }
            drop(data.take());
        }
        until(&mut child, "scope must enter its masked finalizer", || {
            let trace = read_trace(&directory);
            if case == "cancel" && !trace.contains("scope-close\n") {
                // The trace is written before interrupt is scheduled. A first
                // chunk may be consumed before that request reaches the worker,
                // which can then be waiting in another uninterruptible OS read.
                // Keep driving bounded read checkpoints without supplying EOF.
                match data.as_mut().unwrap().write(b"x") {
                    Ok(_) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::BrokenPipe
                        ) => {}
                    Err(error) => panic!("{}-{case}: waking pending read: {error}", program.name),
                }
            }
            trace.contains("scope-close\n").then_some(())
        });
        assert!(child.child.try_wait().unwrap().is_none());
        assert!(
            writer(&directory.join("input.txt")).is_none(),
            "OS reader remains during finalizer"
        );
        if case == "cancel" {
            assert_eq!(
                data.as_mut().unwrap().write(b"late").unwrap_err().kind(),
                std::io::ErrorKind::BrokenPipe
            );
        }
        drop(data);
        let finish = until(&mut child, "masked finalizer awaits finish", || {
            writer(&directory.join("finish.txt"))
        });
        drop(finish);
        let output = child.wait(Duration::from_secs(15));
        let trace = format!(
            "scope-open\n{}scope-close\nreleased\n{}",
            if case == "cancel" {
                "cancel-request\n"
            } else if case == "parent-defect" {
                "parent-stop\n"
            } else {
                ""
            },
            if case == "success" {
                "joined\n"
            } else if case == "cancel" {
                "cancelled\n"
            } else {
                ""
            }
        );
        program.check(&directory, output, case, &trace);
    }
}
