use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct OwnedChild {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
    record: Option<PathBuf>,
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
        let mut owned = Self {
            child: command.spawn().unwrap(),
            stdout,
            stderr,
            record: None,
        };
        owned.record = std::env::var_os("SESERAGI_GENERIC_IMPL_PROCESS_EVIDENCE").map(|root| {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = PathBuf::from(root);
            fs::create_dir_all(&root).unwrap();
            let path = root.join(format!(
                "{}-{}.json",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::write(
                &path,
                serde_json::to_vec_pretty(&serde_json::json!({
                "pid":owned.child.id(), "command":format!("{command:?}"), "waited":false
                }))
                .unwrap(),
            )
            .unwrap();
            path
        });
        owned
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
        let output = Output {
            status,
            stdout: fs::read(&self.stdout).unwrap(),
            stderr: fs::read(&self.stderr).unwrap(),
        };
        if let Some(record) = &self.record {
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(record).unwrap()).unwrap();
            value["waited"] = true.into();
            value["exit"] = output.status.code().into();
            value["stdout"] = String::from_utf8_lossy(&output.stdout).into_owned().into();
            value["stderr"] = String::from_utf8_lossy(&output.stderr).into_owned().into();
            fs::write(record, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        }
        output
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
pub fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
pub fn bounded(command: &mut Command, directory: &Path) -> Output {
    // Native compilation can be slow on CI hosts running many tests in parallel.
    OwnedChild::start(command, directory).wait(Duration::from_secs(600))
}
pub fn runtime(name: &str, directory: &Path) -> PathBuf {
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
