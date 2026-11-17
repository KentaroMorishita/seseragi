// std/http values that need no HttpClient match on every target.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn http_values_match_bun_node_and_native() {
    let scratch =
        Scratch(std::env::temp_dir().join(format!("seseragi-native-http-{}", std::process::id())));
    fs::create_dir(&scratch.0).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for (name, sample) in [("http-values", false)] {
        let input = if sample {
            root.join(format!("examples/samples/{name}/main.ssrg"))
        } else {
            root.join(format!("examples/spec/fixtures/rust-backend/{name}.ssrg"))
        };
        let directory = scratch.0.join(name);
        fs::create_dir(&directory).unwrap();
        let expected = if sample {
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("run")
                    .arg(&input),
                &directory,
            ))
            .stdout
        } else {
            fs::read(input.with_extension("stdout")).unwrap()
        };
        let input = process::copy_input(&input, &directory);
        let native = directory.join("native");
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&input)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &directory,
        ));
        let output = successful(bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &directory,
        ));
        assert_eq!(output.stdout, expected, "{name}/native");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
            "{name}: {output:?}"
        );
        let ts = directory.join("ts");
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&input)
                .args(["--target", "process", "--out-dir"])
                .arg(&ts),
            &directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(ts.join("artifact-manifest.json")).unwrap()).unwrap();
        let entry = ts.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, &directory))
                    .arg(&entry)
                    .env("PATH", ""),
                &directory,
            ));
            assert_eq!(output.stdout, expected, "{name}/{engine}");
            assert!(output.stderr.is_empty(), "{name}/{engine}: {output:?}");
        }
    }
}

// A local HTTP/1.1 server. Every connection gets one response and closes.
fn serve() -> String {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::sync::{Arc, Condvar, Mutex};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    // Whether the connection of the last /hang request has been closed.
    let hang: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(true), Condvar::new()));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let hang = hang.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    return;
                }
                let mut parts = line.split_whitespace();
                let method = parts.next().unwrap_or("").to_owned();
                let path = parts.next().unwrap_or("").to_owned();
                let mut length = 0;
                let mut declared = false;
                let mut chunked = false;
                loop {
                    let mut header = String::new();
                    if reader.read_line(&mut header).unwrap_or(0) == 0 || header.trim().is_empty() {
                        break;
                    }
                    if let Some((name, value)) = header.split_once(':') {
                        if name.eq_ignore_ascii_case("content-length") {
                            length = value.trim().parse().unwrap_or(0);
                            declared = true;
                        }
                        if name.eq_ignore_ascii_case("transfer-encoding") {
                            chunked = value.to_ascii_lowercase().contains("chunked");
                        }
                    }
                }
                let plain = |status: &str, headers: &str, body: &[u8]| {
                    let mut response = format!(
                        "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .into_bytes();
                    response.extend_from_slice(body);
                    response
                };
                // Answers before the request body ends, then reads the rest
                // of the request until the client stops sending, so closing
                // the connection never discards the response.
                let early = |stream: &mut std::net::TcpStream,
                             reader: &mut BufReader<std::net::TcpStream>,
                             response: Vec<u8>| {
                    let _ = stream.write_all(&response);
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                    let mut rest = [0u8; 4096];
                    while matches!(reader.read(&mut rest), Ok(read) if read > 0) {}
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                };
                if path == "/reject" {
                    // The body is never read.
                    early(
                        &mut stream,
                        &mut reader,
                        plain("413 Payload Too Large", "", b"too large"),
                    );
                    return;
                }
                if path == "/first" {
                    // Only the first chunk of the body is read.
                    let mut size = String::new();
                    let _ = reader.read_line(&mut size);
                    let size = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
                    let mut chunk = vec![0; size];
                    let _ = reader.read_exact(&mut chunk);
                    let mut answer = b"first ".to_vec();
                    answer.extend_from_slice(&chunk);
                    early(&mut stream, &mut reader, plain("200 OK", "", &answer));
                    return;
                }
                let mut body = vec![0; length];
                let _ = reader.read_exact(&mut body);
                // A request body of unknown length arrives chunked.
                let mut sizes = Vec::new();
                while chunked {
                    let mut size = String::new();
                    if reader.read_line(&mut size).unwrap_or(0) == 0 {
                        break;
                    }
                    let size =
                        usize::from_str_radix(size.trim().split(';').next().unwrap_or(""), 16)
                            .unwrap_or(0);
                    let mut chunk = vec![0; size + 2];
                    if reader.read_exact(&mut chunk).is_err() {
                        break;
                    }
                    if size == 0 {
                        break;
                    }
                    sizes.push(size.to_string());
                    body.extend_from_slice(&chunk[..size]);
                }
                if path == "/hang" {
                    // Never answer; record when the client closes.
                    *hang.0.lock().unwrap() = false;
                    let mut rest = [0u8; 64];
                    while matches!(reader.read(&mut rest), Ok(read) if read > 0) {}
                    *hang.0.lock().unwrap() = true;
                    hang.1.notify_all();
                    return;
                }
                let closed = || {
                    let (closed, changed) = &*hang;
                    let guard = changed
                        .wait_timeout_while(
                            closed.lock().unwrap(),
                            std::time::Duration::from_secs(5),
                            |closed| !*closed,
                        )
                        .unwrap()
                        .0;
                    if *guard {
                        "closed"
                    } else {
                        "open"
                    }
                };
                let response = match path.as_str() {
                    "/text" => plain("200 OK", "", b"hello"),
                    "/trailers" => b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n2\r\nab\r\n2\r\ncd\r\n0\r\nX-T: 1\r\n\r\n".to_vec(),
                    "/early" => {
                        let mut response = b"HTTP/1.1 103 Early Hints\r\nLink: </a>\r\n\r\n".to_vec();
                        response.extend(plain("200 OK", "", b"early"));
                        response
                    }
                    "/closed" => plain("200 OK", "", closed().as_bytes()),
                    "/chunked" => b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n2\r\nab\r\n2\r\ncd\r\n0\r\n\r\n".to_vec(),
                    "/empty" => b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n".to_vec(),
                    "/headers" => plain("200 OK", "X-Multi: a\r\nX-Multi: b\r\n", b"ok"),
                    "/big" => plain("200 OK", "", &[b'x'; 100]),
                    // How the request body was framed.
                    "/framing" => {
                        let framing = if chunked {
                            format!("chunks {}", sizes.join(","))
                        } else if declared {
                            format!("length {length}")
                        } else {
                            "none".into()
                        };
                        plain("200 OK", "", framing.as_bytes())
                    }
                    "/echo" => {
                        let mut echoed = format!("{method} {} ", body.len()).into_bytes();
                        echoed.extend_from_slice(&body);
                        plain("200 OK", "", &echoed)
                    }
                    _ => plain("404 Not Found", "", b"not found"),
                };
                let _ = stream.write_all(&response);
                let _ = stream.shutdown(std::net::Shutdown::Both);
            });
        }
    });
    base
}

#[test]
fn http_exchanges_match_bun_node_and_native() {
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "seseragi-native-http-client-{}",
        std::process::id()
    )));
    fs::create_dir(&scratch.0).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let base = serve();
    let source = root.join("examples/spec/fixtures/rust-backend/http-client.ssrg");
    let expected = fs::read(source.with_extension("stdout")).unwrap();
    let directory = scratch.0.join("http-client");
    fs::create_dir(&directory).unwrap();
    let input = process::copy_input(&source, &directory);
    let native = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&input)
            .args(["--entry", "main"])
            .args([
                "--provider",
                "std/clock::Clock=seseragi/runtime-native#clock",
            ])
            .args(["--out-dir"])
            .arg(&native),
        &directory,
    ));
    // Only the native run sends request bodies that never end; see the
    // fixture's `endless`.
    let output = successful(bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .arg(&base)
            .arg("endless")
            .env("PATH", ""),
        &directory,
    ));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}exchange /reject: [started 413]too large\nexchange /first: [started 200]first ab\n",
            String::from_utf8_lossy(&expected)
        ),
        "native"
    );
    let ts = directory.join("ts");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("build")
            .arg(&input)
            .args(["--target", "process", "--out-dir"])
            .arg(&ts),
        &directory,
    ));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(ts.join("artifact-manifest.json")).unwrap()).unwrap();
    let entry = ts.join(manifest["entry"].as_str().unwrap());
    for engine in ["bun", "node"] {
        let output = successful(bounded(
            Command::new(runtime(engine, &directory))
                .arg(&entry)
                .arg(&base)
                .env("PATH", ""),
            &directory,
        ));
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&expected),
            "{engine}"
        );
    }
}
