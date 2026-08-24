use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
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
    let started = Instant::now();
    while started.elapsed() < timeout {
        if condition() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("timed out waiting for {message}");
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

#[test]
fn serves_rebuilds_recovers_and_shuts_down_a_canonical_web_project() {
    let directory = test_directory("lifecycle");
    let project = directory.join("project-flow-app");
    copy_directory(
        &repository_root().join("examples/samples/project-flow-app"),
        &project,
    );
    update_lock(&project);
    let log = fs::File::create(directory.join("dev.log")).unwrap();
    let port = available_port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args([
            "dev",
            project.to_str().unwrap(),
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();

    wait_for(
        Duration::from_secs(20),
        || request(port, "/").is_some_and(|response| response.0 == 200),
        "initial web build",
    );
    let (_, index) = request(port, "/").unwrap();
    assert!(index.contains("/__seseragi_dev/version"));
    let (_, source_map) = request(port, "/assets/app.js.map").unwrap();
    assert!(source_map.contains("sources"));
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
            request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 != initial_version)
        },
        "successful rebuild",
    );
    let rebuilt_version = request(port, "/__seseragi_dev/version").unwrap().1;
    let (_, bundle) = request(port, "/assets/app.js").unwrap();
    assert!(bundle.contains("Make room for the next live release."));

    fs::write(&source_path, format!("{changed}\nmissingDevName\n")).unwrap();
    thread::sleep(Duration::from_secs(2));
    assert_eq!(
        request(port, "/__seseragi_dev/version").unwrap().1,
        rebuilt_version
    );
    assert_eq!(request(port, "/").unwrap().0, 200);
    assert!(child.try_wait().unwrap().is_none());

    fs::write(&source_path, &changed).unwrap();
    wait_for(
        Duration::from_secs(20),
        || {
            request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 != rebuilt_version)
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
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args([
            "dev",
            project.to_str().unwrap(),
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();

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
            fs::read_to_string(directory.join("dev.log")).is_ok_and(|contents| {
                contents.contains("missingDependencyName")
                    && contents.contains(&format!("Watching {}", dependency.display()))
            })
        },
        "failed build and refreshed dependency watch root",
    );
    assert_eq!(
        request(port, "/__seseragi_dev/version").unwrap().1,
        initial_version
    );
    assert_eq!(request(port, "/").unwrap().0, 200);
    assert!(child.try_wait().unwrap().is_none());

    fs::write(
        dependency.join("src/lib.ssrg"),
        "pub fn releaseCopy unit: Unit -> String = \"Recovered from dependency\"\n",
    )
    .unwrap();
    wait_for(
        Duration::from_secs(20),
        || {
            request(port, "/__seseragi_dev/version")
                .is_some_and(|response| response.1 != initial_version)
        },
        "path dependency recovery rebuild",
    );
    let (_, bundle) = request(port, "/assets/app.js").unwrap();
    assert!(bundle.contains("Recovered from dependency"));

    let recovered_version = request(port, "/__seseragi_dev/version").unwrap().1;
    fs::write(&manifest_path, "[package\n").unwrap();
    wait_for(
        Duration::from_secs(20),
        || {
            fs::read_to_string(directory.join("dev.log"))
                .is_ok_and(|contents| contents.contains("failed to refresh watched package graph"))
        },
        "transient invalid manifest diagnostic",
    );
    assert_eq!(
        request(port, "/__seseragi_dev/version").unwrap().1,
        recovered_version
    );
    assert_eq!(request(port, "/").unwrap().0, 200);
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
    let directory = test_directory("initial-failure");
    let project = directory.join("project-flow-app");
    copy_directory(
        &repository_root().join("examples/samples/project-flow-app"),
        &project,
    );
    update_lock(&project);
    let source_path = project.join("src/app.ssrg");
    let source = fs::read_to_string(&source_path).unwrap();
    fs::write(&source_path, format!("{source}\nmissingInitialDevName\n")).unwrap();
    let log = fs::File::create(directory.join("dev.log")).unwrap();
    let port = available_port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args([
            "dev",
            project.to_str().unwrap(),
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();

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
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["dev", package.to_str().unwrap(), "--port", "0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
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
