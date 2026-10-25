use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static DEV_SERVER_TEST_LOCK: Mutex<()> = Mutex::new(());
const RELOAD_ENDPOINT: &str = "/__seseragi_dev/version";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn test_directory(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "seseragi-dev-{name}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            copy_directory(&path, &target);
        } else {
            fs::copy(path, target).unwrap();
        }
    }
}

fn update_lock(package: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["lock", "update"])
        .arg(package)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn available_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn request(port: u16, path: &str) -> Option<(u16, String)> {
    request_with_host(port, path, "127.0.0.1")
}

fn request_with_host(port: u16, path: &str, host: &str) -> Option<(u16, String)> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    if let Err(error) = stream.read_to_string(&mut response) {
        eprintln!("HTTP request for {path} did not finish: {error}");
        return None;
    }
    let status = response
        .lines()
        .next()?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()?;
    let body = response.split_once("\r\n\r\n")?.1.to_owned();
    Some((status, body))
}

fn wait_for(timeout: Duration, mut condition: impl FnMut() -> bool, message: &str) {
    wait_for_with_diagnostics(timeout, &mut condition, message, String::new);
}

fn wait_for_value<T>(
    timeout: Duration,
    mut observe: impl FnMut() -> Option<T>,
    message: &str,
) -> T {
    let mut observed = None;
    wait_for(
        timeout,
        || {
            observed = observe();
            observed.is_some()
        },
        message,
    );
    observed.expect("a successful wait retains its observed value")
}

fn wait_for_with_diagnostics(
    timeout: Duration,
    mut condition: impl FnMut() -> bool,
    message: &str,
    mut diagnostics: impl FnMut() -> String,
) {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if condition() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("timed out waiting for {message}\n{}", diagnostics());
}

struct DevServer {
    child: Child,
    log_path: PathBuf,
}

impl DevServer {
    fn spawn(command: &mut Command, log_path: PathBuf) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Self {
            child: command.spawn().unwrap(),
            log_path,
        }
    }
}

impl Deref for DevServer {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.child
    }
}

impl DerefMut for DevServer {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}

impl Drop for DevServer {
    fn drop(&mut self) {
        if thread::panicking() {
            eprintln!(
                "dev server {} log {}:\n{}",
                self.child.id(),
                self.log_path.display(),
                fs::read_to_string(&self.log_path).unwrap_or_default()
            );
        }
        // A live, unreaped Child owns this PID and its dedicated group.
        // Do not reap the leader between TERM and KILL: its reserved PID
        // prevents reuse while remaining group members are reclaimed.
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.wait();
            return;
        }
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .args(["-TERM", "--", &format!("-{}", self.child.id())])
                .status();
            thread::sleep(Duration::from_millis(100));
            let _ = Command::new("kill")
                .args(["-KILL", "--", &format!("-{}", self.child.id())])
                .status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(unix)]
#[test]
fn dev_test_guard_reaps_its_process_group_during_unwind() {
    let directory = test_directory("panic-cleanup");
    let log_path = directory.join("dev.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut server = DevServer::spawn(
        Command::new("sh")
            .args(["-c", "sleep 60 & printf '%s\\n' $!; wait"])
            .stdout(Stdio::from(log)),
        log_path.clone(),
    );
    let parent = server.id();
    let mut descendant = None;
    wait_for(
        Duration::from_secs(5),
        || {
            descendant = fs::read_to_string(&log_path)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok());
            descendant.is_some()
        },
        "owned process group descendant",
    );
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _owned = &mut server;
        panic!("exercise owned dev process cleanup on test failure");
    }));
    assert!(result.is_err());
    for pid in [parent, descendant.unwrap()] {
        wait_for(
            Duration::from_secs(5),
            || {
                !Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .output()
                    .unwrap()
                    .status
                    .success()
            },
            "owned process exit after unwind",
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

fn stop(child: &mut Child) {
    #[cfg(unix)]
    {
        let status = Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap();
        assert!(status.success());
        wait_for(
            Duration::from_secs(5),
            || child.try_wait().unwrap().is_some(),
            "graceful dev shutdown",
        );
        assert!(child.wait().unwrap().success());
    }
    #[cfg(not(unix))]
    {
        child.kill().unwrap();
        child.wait().unwrap();
    }
}

fn build_image(directory: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn visit(
        root: &Path,
        directory: &Path,
        result: &mut std::collections::BTreeMap<String, Vec<u8>>,
    ) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    visit(directory, directory, &mut result);
    result
}

fn served_resource(port: u16, path: &str) -> (String, Vec<u8>) {
    try_served_resource(port, path).expect("complete served resource")
}

fn try_served_resource(port: u16, path: &str) -> Option<(String, Vec<u8>)> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .ok()?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).ok()?;
    let boundary = response.windows(4).position(|bytes| bytes == b"\r\n\r\n")?;
    let headers = String::from_utf8(response[..boundary].to_vec()).unwrap();
    assert!(headers.starts_with("HTTP/1.1 200 OK"), "{headers}");
    let body = response[boundary + 4..].to_vec();
    if let Some(length) = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("Content-Length")
            .then(|| value.trim().parse::<usize>().unwrap())
    }) {
        if body.len() != length {
            return None;
        }
    }
    Some((headers, body))
}

#[test]
fn a_timed_out_binary_response_is_retried_without_accepting_partial_bytes() {
    fn read_request(stream: &mut TcpStream) {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            assert!(request.len() < 8192);
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        assert!(request.starts_with(b"GET /program.wasm HTTP/1.1\r\n"));
    }
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let (release, held) = std::sync::mpsc::sync_channel(1);
    let server = thread::spawn(move || {
        let (mut first, _) = wait_for_value(
            Duration::from_secs(5),
            || listener.accept().ok(),
            "first binary request",
        );
        read_request(&mut first);
        first
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\na")
            .unwrap();
        held.recv_timeout(Duration::from_secs(5)).unwrap();
        drop(first);
        let (mut second, _) = wait_for_value(
            Duration::from_secs(5),
            || listener.accept().ok(),
            "retried binary request",
        );
        read_request(&mut second);
        second
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\n")
            .unwrap();
        second.write_all(&[0, 255, 1]).unwrap();
    });
    assert!(try_served_resource(port, "/program.wasm").is_none());
    release.send(()).unwrap();
    let (_, bytes) = wait_for_value(
        Duration::from_secs(5),
        || try_served_resource(port, "/program.wasm"),
        "complete retried binary response",
    );
    assert_eq!(bytes, [0, 255, 1]);
    server.join().unwrap();
}

fn run_program_probe(engine: &str, script: &Path, directory: &Path) {
    let log_path = directory.join(format!("{engine}-probe.log"));
    let log = fs::File::create(&log_path).unwrap();
    let mut child = DevServer::spawn(
        Command::new(engine)
            .arg(script)
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        log_path.clone(),
    );
    wait_for(
        Duration::from_secs(20),
        || child.try_wait().unwrap().is_some(),
        "actual program lifetime probe",
    );
    assert!(
        child.wait().unwrap().success(),
        "{}",
        fs::read_to_string(log_path).unwrap()
    );
}

#[test]
fn initial_program_diagnostics_recover_without_restarting_dev() {
    let _server = DEV_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for external in [false, true] {
        let directory = test_directory(if external {
            "initial-program-dependency"
        } else {
            "initial-program-module"
        });
        let project = directory.join("project");
        copy_directory(
            &repository_root()
                .join("examples/spec/fixtures/rust-backend-projects/program-wasm-package"),
            &project,
        );
        let broken = if external {
            let dependency = directory.join("data");
            fs::create_dir_all(dependency.join("src")).unwrap();
            fs::write(
                dependency.join("seseragi.toml"),
                "[package]\nname=\"fixture/watch-data\"\nversion=\"0.0.0\"\nlanguage=\">=0.1.0 <0.2.0\"\n[run]\nentry=\"constants\"\n[exports]\nconstants=\"constants\"\n",
            )
            .unwrap();
            let constants = dependency.join("src/constants.ssrg");
            fs::copy(project.join("src/constants.ssrg"), &constants).unwrap();
            let manifest = project.join("seseragi.toml");
            let source = fs::read_to_string(&manifest).unwrap();
            fs::write(
                manifest,
                format!("{source}\n[dependencies]\ndata={{package=\"fixture/watch-data\",path=\"../data\"}}\n"),
            )
            .unwrap();
            let calculation = project.join("src/calculation.ssrg");
            let source = fs::read_to_string(&calculation).unwrap();
            assert!(source.contains("\"./constants\""));
            fs::write(
                calculation,
                source.replace("\"./constants\"", "\"data/constants\""),
            )
            .unwrap();
            constants
        } else {
            project.join("src/calculation.ssrg")
        };
        update_lock(&project);
        let original = fs::read_to_string(&broken).unwrap();
        let (invalid, diagnostic) = if external {
            (
                "pub let offset: Int = missingInitialProgramName\n".to_owned(),
                "SES-N0001",
            )
        } else {
            (
                format!("{original}\nlet initialProgramError: Int = \"not an int\"\n"),
                "SES-T0101",
            )
        };
        fs::write(&broken, invalid).unwrap();
        let log_path = directory.join("dev.log");
        let log = fs::File::create(&log_path).unwrap();
        let port = available_port();
        let mut child = DevServer::spawn(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("dev")
                .arg(&project)
                .args(["--port", &port.to_string()])
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log)),
            log_path.clone(),
        );
        let fallback = wait_for_value(
            Duration::from_secs(10),
            || {
                assert!(
                    child.try_wait().unwrap().is_none(),
                    "{}",
                    fs::read_to_string(&log_path).unwrap()
                );
                request(port, "/").and_then(|(status, body)| (status == 503).then_some(body))
            },
            "initial program diagnostic fallback",
        );
        assert!(fallback.contains("Build failed") && fallback.contains(RELOAD_ENDPOINT));
        let diagnostics = fs::read_to_string(&log_path).unwrap();
        assert!(diagnostics.contains(diagnostic), "{diagnostics}");
        assert!(diagnostics.contains(broken.file_name().unwrap().to_str().unwrap()));
        assert!(!project.join(".seseragi/dev").exists());
        assert_eq!(request(port, RELOAD_ENDPOINT).unwrap().1, "0\n");
        fs::write(broken, original).unwrap();
        let recovered_version = wait_for_value(
            Duration::from_secs(45),
            || {
                let (status, _) = request(port, "/")?;
                if status != 200 {
                    return None;
                }
                let (status, version) = request(port, RELOAD_ENDPOINT)?;
                if status != 200 {
                    return None;
                }
                let version = version.trim().parse::<u64>().unwrap();
                assert!(version > 0);
                Some(version)
            },
            "first complete site after program diagnostics",
        );
        assert!(recovered_version > 0);
        let output = project.join(".seseragi/dev/programs/calculation");
        assert!(fs::read(output.join("program.wasm"))
            .unwrap()
            .starts_with(b"\0asm"));
        stop(&mut child);
        let probe = directory.join("recovered.mjs");
        fs::write(&probe, r#"import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {loadProgram} from './project/.seseragi/dev/programs/calculation/index.mjs';
const bytes=await readFile(new URL('./project/.seseragi/dev/programs/calculation/program.wasm',import.meta.url));
const program=await loadProgram(bytes), prefix=program.exports.label('recovered: '), callback=prefix(40);
prefix.dispose();assert.equal(callback(2),'recovered: 42');callback.dispose();program.dispose();
assert.equal(program.ownership().managedLiveAtClose,0);
"#).unwrap();
        for engine in ["bun", "node"] {
            run_program_probe(engine, &probe, &directory);
        }
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn initial_program_selection_error_remains_fatal() {
    let directory = test_directory("initial-program-selection");
    let project = directory.join("project");
    copy_directory(
        &repository_root()
            .join("examples/spec/fixtures/rust-backend-projects/program-wasm-package"),
        &project,
    );
    let manifest = project.join("seseragi.toml");
    let source = fs::read_to_string(&manifest).unwrap();
    assert!(source.contains("exports = [\"label\","));
    fs::write(
        manifest,
        source.replacen(
            "exports = [\"label\",",
            "exports = [\"missingInitialProgramExport\",",
            1,
        ),
    )
    .unwrap();
    update_lock(&project);
    let log_path = directory.join("dev.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("dev")
            .arg(&project)
            .args(["--port", &available_port().to_string()])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        log_path.clone(),
    );
    wait_for(
        Duration::from_secs(10),
        || child.try_wait().unwrap().is_some(),
        "invalid program selection rejection",
    );
    assert!(!child.wait().unwrap().success());
    let diagnostics = fs::read_to_string(log_path).unwrap();
    assert!(
        diagnostics.contains("web.programs.calculation")
            && diagnostics.contains("SES-RUST-0001")
            && diagnostics.contains("selected function is missing"),
        "{diagnostics}"
    );
    assert!(!project.join(".seseragi/dev").exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn watches_program_wasm_path_dependencies_keeps_failed_builds_and_closes_real_instances() {
    let _server = DEV_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let directory = test_directory("program-wasm");
    let project = directory.join("project");
    copy_directory(
        &repository_root()
            .join("examples/spec/fixtures/rust-backend-projects/program-wasm-package"),
        &project,
    );
    let dependency = directory.join("data");
    fs::create_dir_all(dependency.join("src")).unwrap();
    fs::write(dependency.join("seseragi.toml"), "[package]\nname = \"fixture/watch-data\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n\n[run]\nentry = \"constants\"\n\n[exports]\nconstants = \"constants\"\n").unwrap();
    let constants = dependency.join("src/constants.ssrg");
    fs::copy(project.join("src/constants.ssrg"), &constants).unwrap();
    let manifest = project.join("seseragi.toml");
    let mut source = fs::read_to_string(&manifest).unwrap();
    source.push_str(
        "\n[dependencies]\ndata = { package = \"fixture/watch-data\", path = \"../data\" }\n",
    );
    fs::write(&manifest, source).unwrap();
    let calculation = project.join("src/calculation.ssrg");
    let source = fs::read_to_string(&calculation).unwrap();
    assert!(source.contains("\"./constants\""));
    fs::write(
        &calculation,
        source.replace("\"./constants\"", "\"data/constants\""),
    )
    .unwrap();
    update_lock(&project);
    let port = available_port();
    let log_path = directory.join("dev.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("dev")
            .arg(&project)
            .args(["--port", &port.to_string()])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        log_path.clone(),
    );
    wait_for(
        Duration::from_secs(45),
        || {
            assert!(
                child.try_wait().unwrap().is_none(),
                "{}",
                fs::read_to_string(&log_path).unwrap()
            );
            request(port, "/").is_some_and(|r| r.0 == 200)
        },
        "initial program WASM Web build",
    );
    let output = project.join(".seseragi/dev");
    let receipt_path = output.join("programs/calculation/program-wasm.json");
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    let selected = seseragi_project::load_local_project_module(
        &project,
        &seseragi_project::ModulePath::parse("calculation").unwrap(),
    )
    .unwrap();
    let expected = selected
        .graph()
        .topological_order()
        .unwrap()
        .iter()
        .map(seseragi_project::logical_module_id)
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 2);
    assert_eq!(receipt["modules"], serde_json::json!(expected));
    let (_, loader) = request(port, "/programs/calculation/index.mjs").unwrap();
    assert!(loader.contains("loadOwnedProgram"));
    let (headers, bytes) = served_resource(port, "/programs/calculation/index.mjs");
    assert!(headers.contains("Content-Type: text/javascript; charset=utf-8"));
    assert_eq!(
        bytes,
        fs::read(output.join("programs/calculation/index.mjs")).unwrap()
    );
    let (headers, bytes) = served_resource(port, "/programs/calculation/program.wasm");
    assert!(headers.contains("Content-Type: application/wasm"));
    assert_eq!(
        bytes,
        fs::read(output.join("programs/calculation/program.wasm")).unwrap()
    );
    let (_, lifecycle) = request(port, "/programs/__lifecycle.mjs").unwrap();
    assert!(lifecycle.contains("seseragi:before-reload"));
    let (_, index) = request(port, "/").unwrap();
    assert!(index.contains("dispatchEvent(closing)"));
    let initial_version = request(port, RELOAD_ENDPOINT).unwrap().1;
    let initial = build_image(&output);
    fs::write(&constants, "pub let offset: Int = missingWatchValue\n").unwrap();
    wait_for(
        Duration::from_secs(45),
        || {
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("Build failed")
        },
        "program compile failure",
    );
    assert_eq!(build_image(&output), initial);
    assert_eq!(request(port, RELOAD_ENDPOINT).unwrap().1, initial_version);
    fs::write(
        &constants,
        "let initial = 1\npub let offset: Int = initial\n",
    )
    .unwrap();
    let (next_version, rebuilt_program) = wait_for_value(
        Duration::from_secs(45),
        || {
            let (status, version) = request(port, RELOAD_ENDPOINT)?;
            if status != 200 || version == initial_version {
                return None;
            }
            let program = fs::read(output.join("programs/calculation/program.wasm")).unwrap();
            Some((version, program))
        },
        "external pure program dependency rebuild",
    );
    assert_ne!(
        rebuilt_program,
        initial["programs/calculation/program.wasm"]
    );
    let main = project.join("src/main.ssrg");
    let source = fs::read_to_string(&main).unwrap();
    fs::write(
        &main,
        source.replace("Seseragi program WASM", "Watched program WASM"),
    )
    .unwrap();
    let rebuilt_ui = wait_for_value(
        Duration::from_secs(45),
        || {
            let (status, version) = request(port, RELOAD_ENDPOINT)?;
            if status != 200 || version == next_version {
                return None;
            }
            let source = String::from_utf8(fs::read(output.join("assets/app.js")).ok()?).unwrap();
            source.contains("Watched program WASM").then_some(source)
        },
        "generated UI rebuild",
    );
    assert!(rebuilt_ui.contains("Watched program WASM"));
    stop(&mut child);

    let script = directory.join("lifetime.mjs");
    fs::write(&script, r#"import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {loadProgram,ProgramWasmClosed} from './project/.seseragi/dev/programs/calculation/index.mjs';
import {closePrograms,createProgramLifetime,createProgramApplication} from './project/.seseragi/dev/programs/__lifecycle.mjs';
import {loadProgram as rawLoadProgram} from './project/.seseragi/dev/programs/calculation/program.mjs';
const bytes=await readFile(new URL('./project/.seseragi/dev/programs/calculation/program.wasm',import.meta.url));
const first=await loadProgram(bytes),second=await loadProgram(bytes);
const prefix=second.exports.label('合計: '),saved=prefix(40);prefix.dispose();
assert.equal(saved(2),'合計: 43');first.dispose();
const instantiate=WebAssembly.instantiate,compile=WebAssembly.compile;let compileCalls=0;
WebAssembly.compile=(...args)=>{compileCalls++;return Reflect.apply(compile,WebAssembly,args);};
const gate=Promise.withResolvers(),ready=Promise.withResolvers();let late;
WebAssembly.instantiate=async(...args)=>{late=await Reflect.apply(instantiate,WebAssembly,args);ready.resolve();await gate.promise;return late;};
try{
  const loading=loadProgram(bytes);assert.equal(compileCalls,1);await ready.promise;
  let closed=false;const closing=closePrograms().then(()=>{closed=true;});
  assert.equal(second.ownership().managedLiveAtClose,0);
  assert.throws(()=>saved(2),ProgramWasmClosed);
  assert.throws(()=>second.exports.validate(1),ProgramWasmClosed);
  await Promise.resolve();assert.equal(closed,false);gate.resolve();
  await assert.rejects(loading,ProgramWasmClosed);await closing;
  assert.equal(late.exports.ssrg_live(),0);assert.equal(late.exports.ssrg_callbacks(),0);
  await assert.rejects(()=>loadProgram(bytes),ProgramWasmClosed);
  assert.equal(first.ownership().managedLiveAtClose,0);
  assert.equal(second.ownership().callbacks,0);
  const startupLifetime=createProgramLifetime(),application=createProgramApplication(startupLifetime);
  const primary=new Error('application startup after WASM acquisition');let startupOwner;
  await assert.rejects(application.run(async()=>{
    startupOwner=await startupLifetime.load(()=>rawLoadProgram(bytes),()=>new ProgramWasmClosed('closed application'));
    const prefix=startupOwner.exports.label('startup: '),callback=prefix(40);prefix.dispose();
    assert.equal(callback(2),'startup: 43');throw primary;
  }),error=>error===primary);
  assert.equal(startupOwner.ownership().managedLiveAtClose,0);
  assert.equal(startupOwner.ownership().callbacks,0);await application.close();
  console.log('real program: late load drained, stale callbacks rejected, managed roots0');
}finally{WebAssembly.instantiate=instantiate;WebAssembly.compile=compile;gate.resolve();}
"#).unwrap();
    for engine in ["bun", "node"] {
        run_program_probe(engine, &script, &directory);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn watches_host_effect_programs_and_drains_the_old_application_before_reload() {
    let _server = DEV_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let directory = test_directory("host-effect-program-wasm");
    let project = directory.join("project");
    copy_directory(
        &repository_root()
            .join("examples/spec/fixtures/rust-backend-projects/program-wasm-host-effect-package"),
        &project,
    );
    update_lock(&project);
    let port = available_port();
    let log_path = directory.join("dev.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("dev")
            .arg(&project)
            .args(["--port", &port.to_string()])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        log_path.clone(),
    );
    wait_for(
        Duration::from_secs(45),
        || {
            assert!(
                child.try_wait().unwrap().is_none(),
                "{}",
                fs::read_to_string(&log_path).unwrap()
            );
            request(port, "/").is_some_and(|response| response.0 == 200)
        },
        "initial host Effect program WASM build",
    );
    let output = project.join(".seseragi/dev");
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(output.join("programs/calculation/program-wasm.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["abi"], "seseragi-program-wasm-host-effect-v1");
    let initial_version = request(port, RELOAD_ENDPOINT).unwrap().1;
    let initial = build_image(&output);
    let before = directory.join("before");
    copy_directory(&output.join("programs"), &before.join("programs"));
    let calculation = project.join("src/calculation.ssrg");
    let original = fs::read_to_string(&calculation).unwrap();
    assert!(original.contains("\"processed:\""));
    fs::write(
        &calculation,
        format!("{original}\npub let watchBroken: Int = missingWatchValue\n"),
    )
    .unwrap();
    wait_for(
        Duration::from_secs(45),
        || {
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("Build failed")
        },
        "host Effect program compile failure",
    );
    assert_eq!(build_image(&output), initial);
    assert_eq!(request(port, RELOAD_ENDPOINT).unwrap().1, initial_version);
    fs::write(
        &calculation,
        original.replace("\"processed:\"", "\"watched:\""),
    )
    .unwrap();
    let paths = [
        "/programs/calculation/index.mjs",
        "/programs/calculation/program.wasm",
        "/programs/__lifecycle.mjs",
    ];
    let (rebuilt, served, index) = wait_for_value(
        Duration::from_secs(45),
        || {
            assert!(
                child.try_wait().unwrap().is_none(),
                "{}",
                fs::read_to_string(&log_path).unwrap()
            );
            let (status, version) = request(port, RELOAD_ENDPOINT)?;
            if status != 200 || version == initial_version {
                return None;
            }
            // A source write can cause another watch snapshot while a build
            // finishes. Keep binary delivery in the existing 45-second wait;
            // each HTTP attempt still has its two-second timeout.
            let served = paths
                .iter()
                .map(|path| try_served_resource(port, path))
                .collect::<Option<Vec<_>>>()?;
            let index = request(port, "/")?;
            assert_eq!(index.0, 200);
            Some((
                fs::read(output.join("programs/calculation/program.wasm")).unwrap(),
                served,
                index.1,
            ))
        },
        "host Effect program recovery and rebuild",
    );
    assert_ne!(rebuilt, initial["programs/calculation/program.wasm"]);
    for (path, (_, bytes)) in paths.into_iter().zip(served) {
        assert_eq!(bytes, fs::read(output.join(&path[1..])).unwrap());
    }
    assert!(index.contains("dispatchEvent(closing)"));
    let after = directory.join("after");
    copy_directory(&output.join("programs"), &after.join("programs"));
    stop(&mut child);

    for (name, cleanup_defect) in [("success", false), ("defect", true)] {
        let case = directory.join(name);
        fs::create_dir(&case).unwrap();
        copy_directory(&before, &case.join("before"));
        copy_directory(&after, &case.join("after"));
        fs::write(
            case.join("watch-case.json"),
            serde_json::to_vec(&serde_json::json!({"cleanupDefect":cleanup_defect})).unwrap(),
        )
        .unwrap();
        let script = case.join("watch.mjs");
        fs::copy(
            repository_root()
                .join("crates/seseragi-cli/tests/fixtures/program-wasm-host-watch.mjs"),
            &script,
        )
        .unwrap();
        for engine in ["bun", "node"] {
            run_program_probe(engine, &script, &case);
            assert!(fs::read_to_string(case.join(format!("{engine}-probe.log")))
                .unwrap()
                .contains("host watch event: old cleanup drained; next image verified"));
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn serves_rebuilds_recovers_and_shuts_down_a_canonical_web_project() {
    let _server = DEV_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let directory = test_directory("lifecycle");
    let project = directory.join("project-flow-app");
    copy_directory(
        &repository_root().join("examples/samples/project-flow-app"),
        &project,
    );
    update_lock(&project);
    let log = fs::File::create(directory.join("dev.log")).unwrap();
    let port = available_port();
    let mut child = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args([
                "dev",
                project.to_str().unwrap(),
                "--port",
                &port.to_string(),
            ])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        directory.join("dev.log"),
    );

    wait_for(
        Duration::from_secs(20),
        || request(port, "/").is_some_and(|response| response.0 == 200),
        "initial web build",
    );
    let (_, index) = request(port, "/").unwrap();
    assert!(index.contains("/__seseragi_dev/version"));
    let (_, source_map) = request(port, "/assets/app.js.map").unwrap();
    assert!(source_map.contains("sources"));
    for host in [
        "attacker.example".to_owned(),
        format!("127.0.0.1.attacker.example:{port}"),
        "attacker.example\r\nHost: 127.0.0.1".to_owned(),
        "127.0.0.1\r\nhOsT: attacker.example".to_owned(),
    ] {
        for path in [
            "/",
            "/assets/app.js",
            "/assets/app.js.map",
            "/__seseragi_dev/version",
        ] {
            let (status, body) = request_with_host(port, path, &host).unwrap();
            assert_eq!(status, 403, "host {host:?}, path {path}");
            assert_eq!(body, "Forbidden request host\n");
        }
    }
    assert_eq!(
        request_with_host(port, "/", &format!("localhost:{port}"))
            .unwrap()
            .0,
        200
    );
    let initial_version = request(port, "/__seseragi_dev/version").unwrap().1;
    let production_output = project.join("dist");
    assert!(!production_output.exists());

    let source_path = project.join("src/app.ssrg");
    let original = fs::read_to_string(&source_path).unwrap();
    let changed = original.replace(
        "Make room for the next release.",
        "Make room for the next live release.",
    );
    fs::write(&source_path, &changed).unwrap();
    wait_for(
        Duration::from_secs(20),
        || {
            let Some((_, version)) = request(port, "/__seseragi_dev/version") else {
                return false;
            };
            let bundle_updated = request(port, "/assets/app.js").is_some_and(|response| {
                response.1.contains("Make room for the next live release.")
            });
            if version == initial_version || !bundle_updated {
                return false;
            }
            true
        },
        "successful rebuild",
    );
    fs::write(
        &source_path,
        format!("{changed}\nlet brokenDevValue = missingDevName\n"),
    )
    .unwrap();
    let mut preserved_version = None;
    wait_for_with_diagnostics(
        Duration::from_secs(20),
        || {
            let log = fs::read_to_string(directory.join("dev.log")).unwrap_or_default();
            if !log.contains("missingDevName") || !log.contains("Build failed") {
                return false;
            }
            // A source write can overlap a watch snapshot, so another successful
            // build may finish after the first updated bundle was observed.
            // A failed build must preserve the last successful reload, not
            // necessarily that first observation.
            let Some(last_success) = log.lines().rev().find_map(|line| {
                line.strip_prefix("Built web app (")?
                    .rsplit_once("; reload ")?
                    .1
                    .strip_suffix(')')?
                    .parse::<u64>()
                    .ok()
            }) else {
                return false;
            };
            let expected = format!("{last_success}\n");
            let version_unchanged = request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 == expected);
            let previous_build_available =
                request(port, "/").is_some_and(|response| response.0 == 200);
            let previous_bundle_available =
                request(port, "/assets/app.js").is_some_and(|response| {
                    response.0 == 200
                        && response.1.contains("Make room for the next live release.")
                        && !response.1.contains("missingDevName")
                });
            if version_unchanged && previous_build_available && previous_bundle_available {
                preserved_version = Some(expected);
                return true;
            }
            false
        },
        "failed rebuild preserving the previous output",
        || {
            format!(
                "actual version: {:?}; index: {:?}\n{}",
                request(port, "/__seseragi_dev/version"),
                request(port, "/").map(|r| r.0),
                fs::read_to_string(directory.join("dev.log")).unwrap_or_default()
            )
        },
    );
    let preserved_version = preserved_version.unwrap();
    assert!(child.try_wait().unwrap().is_none());

    fs::write(&source_path, &changed).unwrap();
    wait_for(
        Duration::from_secs(20),
        || {
            request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 != preserved_version)
        },
        "recovery rebuild",
    );
    stop(&mut child);

    let log = fs::read_to_string(directory.join("dev.log")).unwrap();
    assert!(log.contains("Built web app"));
    assert!(log.contains("error[SES-N0001]"), "{log}");
    assert!(log.contains("Build failed"));
    assert!(log.contains("Stopped dev server"));
    assert!(!production_output.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn watches_a_new_path_dependency_even_when_its_first_build_fails() {
    let _server = DEV_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let directory = test_directory("path-dependency-recovery");
    let project = directory.join("project-flow-app");
    let dependency = directory.join("release-copy");
    copy_directory(
        &repository_root().join("examples/samples/project-flow-app"),
        &project,
    );
    update_lock(&project);
    let log = fs::File::create(directory.join("dev.log")).unwrap();
    let port = available_port();
    let mut child = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args([
                "dev",
                project.to_str().unwrap(),
                "--port",
                &port.to_string(),
            ])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        directory.join("dev.log"),
    );

    wait_for(
        Duration::from_secs(20),
        || request(port, "/").is_some_and(|response| response.0 == 200),
        "initial web build",
    );
    let initial_version = request(port, "/__seseragi_dev/version").unwrap().1;

    fs::create_dir_all(dependency.join("src")).unwrap();
    fs::write(
        dependency.join("seseragi.toml"),
        concat!(
            "[package]\n",
            "name = \"samples/release-copy\"\n",
            "version = \"0.0.0\"\n",
            "language = \">=0.1.0 <0.2.0\"\n\n",
            "[exports]\n",
            "\".\" = \"lib\"\n",
        ),
    )
    .unwrap();
    fs::write(
        dependency.join("src/lib.ssrg"),
        "pub fn releaseCopy unit: Unit -> String = missingDependencyName\n",
    )
    .unwrap();
    let dependency = dependency.canonicalize().unwrap();

    let app_path = project.join("src/app.ssrg");
    let app = fs::read_to_string(&app_path).unwrap();
    fs::write(
        &app_path,
        app.replacen(
            "import * as signals from \"std/signal\"\n",
            concat!(
                "import * as signals from \"std/signal\"\n",
                "import { releaseCopy } from \"release-copy\"\n",
            ),
            1,
        )
        .replace(
            "children: \"Make room for the next release.\"",
            "children: releaseCopy ()",
        ),
    )
    .unwrap();
    let manifest_path = project.join("seseragi.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    let manifest_with_dependency = format!(
        "{manifest}\n[dependencies]\nrelease-copy = {{ package = \"samples/release-copy\", path = \"../release-copy\" }}\n"
    );
    fs::write(&manifest_path, &manifest_with_dependency).unwrap();
    update_lock(&project);

    wait_for(
        Duration::from_secs(20),
        || {
            let failed = fs::read_to_string(directory.join("dev.log")).is_ok_and(|contents| {
                contents.contains("missingDependencyName")
                    && contents.contains(&format!("Watching {}", dependency.display()))
            });
            let version_unchanged = request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 == initial_version);
            let previous_build_available =
                request(port, "/").is_some_and(|response| response.0 == 200);
            failed && version_unchanged && previous_build_available
        },
        "failed build and refreshed dependency watch root",
    );
    assert!(child.try_wait().unwrap().is_none());

    fs::write(
        dependency.join("src/lib.ssrg"),
        "pub fn releaseCopy unit: Unit -> String = \"Recovered from dependency\"\n",
    )
    .unwrap();
    let mut recovered_version = None;
    wait_for(
        Duration::from_secs(20),
        || {
            let Some((_, version)) = request(port, "/__seseragi_dev/version") else {
                return false;
            };
            let bundle_updated = request(port, "/assets/app.js")
                .is_some_and(|response| response.1.contains("Recovered from dependency"));
            if version == initial_version || !bundle_updated {
                return false;
            }
            recovered_version = Some(version);
            true
        },
        "path dependency recovery rebuild",
    );
    let recovered_version = recovered_version.unwrap();
    fs::write(&manifest_path, "[package\n").unwrap();
    wait_for(
        Duration::from_secs(20),
        || {
            let refresh_failed = fs::read_to_string(directory.join("dev.log"))
                .is_ok_and(|contents| contents.contains("failed to refresh watched package graph"));
            let version_unchanged = request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 == recovered_version);
            let previous_build_available =
                request(port, "/").is_some_and(|response| response.0 == 200);
            refresh_failed && version_unchanged && previous_build_available
        },
        "transient invalid manifest diagnostic",
    );
    assert!(child.try_wait().unwrap().is_none());

    fs::write(&manifest_path, manifest_with_dependency).unwrap();
    wait_for(
        Duration::from_secs(20),
        || {
            request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 != recovered_version)
        },
        "recovery after transient invalid manifest",
    );
    stop(&mut child);

    let log = fs::read_to_string(directory.join("dev.log")).unwrap();
    assert!(log.contains("missingDependencyName"), "{log}");
    assert!(log.contains("Build failed"), "{log}");
    assert!(log.contains("reload 2"), "{log}");
    assert!(log.contains("reload 3"), "{log}");
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reports_port_conflicts_without_starting_a_second_server() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let package = repository_root().join("examples/samples/project-flow-app");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args([
            "dev",
            package.to_str().unwrap(),
            "--port",
            &port.to_string(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("dev server could not bind"), "{stderr}");
    assert!(stderr.contains(&port.to_string()), "{stderr}");
}

#[test]
fn recovers_when_the_initial_build_has_compiler_diagnostics() {
    let _server = DEV_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let directory = test_directory("initial-failure");
    let project = directory.join("project-flow-app");
    copy_directory(
        &repository_root().join("examples/samples/project-flow-app"),
        &project,
    );
    update_lock(&project);
    let source_path = project.join("src/app.ssrg");
    let source = fs::read_to_string(&source_path).unwrap();
    fs::write(
        &source_path,
        format!("{source}\nlet brokenInitialDevValue = missingInitialDevName\n"),
    )
    .unwrap();
    let log = fs::File::create(directory.join("dev.log")).unwrap();
    let port = available_port();
    let mut child = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args([
                "dev",
                project.to_str().unwrap(),
                "--port",
                &port.to_string(),
            ])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        directory.join("dev.log"),
    );

    let mut initial_fallback = None;
    wait_for(
        Duration::from_secs(10),
        || {
            request(port, "/").is_some_and(|(status, body)| {
                if status == 503 {
                    initial_fallback = Some(body);
                    true
                } else {
                    false
                }
            })
        },
        "initial diagnostic fallback",
    );
    let fallback = initial_fallback.unwrap();
    assert!(fallback.contains("Build failed"));
    assert!(fallback.contains("/__seseragi_dev/version"));
    assert!(child.try_wait().unwrap().is_none());

    fs::write(&source_path, source).unwrap();
    wait_for(
        Duration::from_secs(20),
        || request(port, "/").is_some_and(|response| response.0 == 200),
        "first successful build after diagnostics",
    );
    stop(&mut child);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_process_target_packages_before_entering_the_watch_loop() {
    let directory = test_directory("process-target");
    let package = directory.join("std-parity-portable");
    copy_directory(
        &repository_root().join("examples/spec/fixtures/projects/std-parity-portable"),
        &package,
    );
    update_lock(&package);
    let log_path = directory.join("refused-dev.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["dev", package.to_str().unwrap(), "--port", "0"])
            .stdout(Stdio::from(
                fs::File::create(directory.join("refused-dev.stdout")).unwrap(),
            ))
            .stderr(Stdio::from(log)),
        log_path.clone(),
    );
    wait_for(
        Duration::from_secs(20),
        || child.try_wait().unwrap().is_some(),
        "process target refusal before watching",
    );
    assert_eq!(child.wait().unwrap().code(), Some(2));
    assert!(fs::read_to_string(&log_path)
        .unwrap()
        .contains("dev does not support the `process` target"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn documents_dev_in_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("seseragi dev [path/to/package]"));
    assert!(stdout.contains("--host 127.0.0.1"));
    assert!(stdout.contains("--port 3000"));
    assert!(stdout.contains("--open"));
}

#[test]
fn rebuilds_custom_shell_and_extensionless_public_assets_with_build_parity() {
    let _server = DEV_SERVER_TEST_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let directory = test_directory("custom-web-assets");
    let project = directory.join("project");
    copy_directory(
        &repository_root().join("examples/spec/fixtures/projects/web-assets"),
        &project,
    );
    fs::write(project.join("public/large.txt"), "x".repeat(2_000_000)).unwrap();
    update_lock(&project);
    let production = directory.join("production");
    let build = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["build", "--target", "web"])
        .arg(&project)
        .arg("--out-dir")
        .arg(&production)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let log = fs::File::create(directory.join("dev.log")).unwrap();
    let port = available_port();
    let mut server = DevServer::spawn(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("dev")
            .arg(&project)
            .args(["--port", &port.to_string()])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log)),
        directory.join("dev.log"),
    );
    wait_for(
        Duration::from_secs(30),
        || request(port, "/").is_some_and(|r| r.0 == 200),
        "custom document",
    );
    assert_eq!(request(port, "/large.txt").unwrap().1.len(), 2_000_000);
    let html = request(port, "/").unwrap().1;
    assert!(html.contains("<title>Seseragi custom document</title>"));
    let generated = fs::read_to_string(project.join(".seseragi/dev/index.html")).unwrap();
    assert_eq!(
        generated,
        fs::read_to_string(production.join("index.html")).unwrap()
    );
    assert_eq!(
        request(port, "/images/read%20me.txt").unwrap(),
        (200, "nested public asset\n".to_owned())
    );
    assert_eq!(request(port, "/status").unwrap().1, "ready\n");
    assert_eq!(request(port, "/%2e%2e/seseragi.toml").unwrap().0, 400);

    fs::write(project.join("public/status"), "changed\n").unwrap();
    let index = project.join("web/index.html");
    fs::write(
        &index,
        fs::read_to_string(&index)
            .unwrap()
            .replace("Seseragi custom document", "Rebuilt document"),
    )
    .unwrap();
    wait_for(
        Duration::from_secs(30),
        || {
            request(port, "/").is_some_and(|r| r.1.contains("<title>Rebuilt document</title>"))
                && request(port, "/status").is_some_and(|r| r.1 == "changed\n")
        },
        "HTML and extensionless asset rebuild",
    );
    fs::remove_file(project.join("public/robots.txt")).unwrap();
    wait_for(
        Duration::from_secs(30),
        || {
            assert!(
                server.try_wait().unwrap().is_none(),
                "dev server exited during deletion: {}",
                fs::read_to_string(directory.join("dev.log")).unwrap()
            );
            request(port, "/robots.txt").is_some_and(|r| r.0 == 404)
        },
        "deleted asset removal",
    );
    stop(&mut server);
    fs::remove_dir_all(directory).unwrap();
}
