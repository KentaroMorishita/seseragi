use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn assert_target_mismatch(output: &std::process::Output) {
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "seseragi: target mismatch before execution",
        "SES-K0203 provider.target-mismatch",
        "required capabilities: dom",
        "selected target: process",
        "selected target capabilities: console, stdin",
        "missing capabilities: dom",
        "available target contracts: browser",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in {stderr}"
        );
    }
    assert!(!stderr.contains("runtime defect"));
}

#[test]
fn rejects_unsupported_dom_before_single_file_and_project_execution() {
    let fixtures = repository_root().join("crates/seseragi-cli/tests/fixtures");
    for path in [
        fixtures.join("target-mismatch.ssrg"),
        fixtures.join("target-mismatch-project"),
        repository_root().join("examples/spec/fixtures/projects/std-parity-target"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(path)
            .output()
            .unwrap();
        assert_target_mismatch(&output);
    }
}

#[test]
fn runs_the_phase_one_program_without_fixture_metadata() {
    let root = repository_root();
    let program = root.join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"rock\nscissors\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Player 1 wins!\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn renders_typed_failure_and_preserves_the_program_exit_class() {
    let root = repository_root();
    let program = root.join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"lizard\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "UnknownHand lizard\n"
    );
}

#[test]
fn runs_the_manifest_discovered_split_phase_one_program() {
    let package = repository_root()
        .join("examples/spec/artifacts/project-schema-1/rock-paper-scissors-cli-split");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(".")
        .current_dir(package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"paper\nrock\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Player 1 wins!\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn renders_a_split_package_typed_failure() {
    let package = repository_root()
        .join("examples/spec/artifacts/project-schema-1/rock-paper-scissors-cli-split");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"lizard\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "InputFailure UnknownHand lizard\n"
    );
}

#[test]
fn runs_a_local_path_dependency_package() {
    let package =
        repository_root().join("examples/spec/fixtures/projects/package-path-dependency-basic");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_the_portable_standard_parity_package() {
    let package = repository_root().join("examples/spec/fixtures/projects/std-parity-portable");
    let expected = std::fs::read_to_string(package.join("expected.stdout")).unwrap();
    for entry in [&package, &package.join("src/main.ssrg")] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(entry)
            .output()
            .unwrap();

        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }
}

#[test]
fn runs_imported_derived_json_codecs() {
    let package =
        repository_root().join("examples/spec/fixtures/projects/imported-derived-json-codecs");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_effect_temporal_control() {
    let package = repository_root().join("examples/spec/fixtures/projects/effect-temporal-control");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_effect_resource_scope() {
    let package = repository_root().join("examples/spec/fixtures/projects/effect-resource-scope");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_the_small_response_http_client_surface_from_seseragi_source() {
    let listener = TcpListener::bind(("127.0.0.1", 41287)).unwrap();
    let server = std::thread::spawn(move || {
        let (mut connection, _) = listener.accept().unwrap();
        connection
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        loop {
            let count = connection.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            let Some(head_end) = request.windows(4).position(|part| part == b"\r\n\r\n") else {
                continue;
            };
            let head = String::from_utf8_lossy(&request[..head_end + 4]);
            let content_length = head
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= head_end + 4 + content_length {
                break;
            }
        }
        let request = String::from_utf8_lossy(&request);
        assert!(
            request.starts_with("POST /upload HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains("x-seseragi: small-response\r\n"),
            "{request}"
        );
        assert!(request.ends_with("seseragi"), "{request}");
        connection
            .write_all(
                b"HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\nContent-Length: 5\r\nConnection: close\r\n\r\nready",
            )
            .unwrap();
    });

    let package =
        repository_root().join("examples/spec/fixtures/projects/provider-http-client-e2e");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_a_real_bun_http_provider_from_seseragi_source() {
    let package =
        repository_root().join("examples/spec/fixtures/projects/provider-http-server-e2e");
    let source = std::fs::read_to_string(package.join("src/main.ssrg")).unwrap();
    assert!(!source.contains("runtime-bun"));
    assert!(!source.contains("seseragi/runtime-bun#http-server"));
    run_http_server_fixture(
        &package,
        b"{\"name\":\"Mio\"}",
        "201",
        "{\"id\":42,\"name\":\"Mio\"}",
    );
    run_http_server_fixture(&package, b"not json", "400", "invalid json");
    run_http_server_fixture(&package, &[0xff], "400", "invalid utf-8");
}

fn run_http_server_fixture(
    package: &Path,
    body: &[u8],
    expected_status: &str,
    expected_body: &str,
) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let connection = (0..2400).find_map(|_| match TcpStream::connect(("127.0.0.1", 41286)) {
        Ok(stream) => Some(stream),
        Err(_) => {
            std::thread::sleep(Duration::from_millis(25));
            None
        }
    });
    let Some(mut connection) = connection else {
        let _ = child.kill();
        let output = child.wait_with_output().unwrap();
        panic!(
            "Seseragi HTTP provider did not start\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };
    let mut request = format!(
        "POST /users?source=e2e HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(body);
    connection.write_all(&request).unwrap();
    let mut response = String::new();
    connection.read_to_string(&mut response).unwrap();
    assert!(
        response.starts_with(&format!("HTTP/1.1 {expected_status}")),
        "{response}"
    );
    if expected_status == "201" {
        assert!(
            response
                .to_ascii_lowercase()
                .contains("content-type: application/json; charset=utf-8"),
            "{response}"
        );
        assert!(
            response
                .to_ascii_lowercase()
                .contains("x-seseragi-method: post"),
            "{response}"
        );
    }
    assert!(response.contains(expected_body), "{response}");

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_only_entry_reachable_modules_in_a_local_project() {
    let package = repository_root().join("examples/spec/fixtures/projects/entry-rooted-runtime");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_logical_conditions_with_branch_values_and_short_circuiting() {
    let package = repository_root().join("examples/spec/fixtures/projects/logical-short-circuit");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_canonical_reduce_with_curried_lambdas() {
    let package = repository_root().join("examples/spec/fixtures/projects/prelude-reduce-lambda");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn reports_compiler_diagnostics_with_source_ranges() {
    let program = repository_root()
        .join("examples/spec/artifacts/semantic-diagnostics-schema-1/unknown-pure-name/main.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&program)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&format!("{}:1:", program.display())));
    assert!(stderr.contains("error[SES-N0001]: Name could not be resolved"));
    assert!(stderr.contains("pub fn useMissing value: Int -> Int = missing"));
    assert!(stderr.contains("= help:"));
    assert!(!stderr.contains("name.unresolved"));
}

#[test]
fn formats_a_file_and_supports_check_mode() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("seseragi-format-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join("main.ssrg");
    std::fs::write(
        &source_path,
        "pub fn identity value: Int -> Int =   \r\n      value   \r\n",
    )
    .unwrap();

    let before = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["format", "--check"])
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(before.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&before.stderr).contains("not canonically formatted"));

    let formatted = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("format")
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(formatted.status.code(), Some(0));
    assert_eq!(
        std::fs::read_to_string(&source_path).unwrap(),
        "pub fn identity value: Int -> Int = value\n"
    );

    let after = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["format", "--check"])
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(after.status.code(), Some(0));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn phase_one_goal_program_passes_format_check() {
    let program = repository_root()
        .join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["format", "--check"])
        .arg(program)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
