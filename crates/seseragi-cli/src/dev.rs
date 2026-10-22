use crate::local_project::{compile_path, LocalProjectCompilation};
use seseragi_project::ProjectCommand;
use seseragi_runtime::{build_local_project, BuildTarget};
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const RELOAD_PATH: &str = "/__seseragi_dev/version";
const WATCH_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) fn dev(arguments: &[String]) -> Result<i32, String> {
    let options = DevOptions::parse(arguments)?;
    if !options.path.is_dir() {
        return Err("dev requires a package directory".to_owned());
    }
    let project = options.path.canonicalize().map_err(|error| {
        format!(
            "failed to resolve package {}: {error}",
            options.path.display()
        )
    })?;
    let output = project.join(".seseragi/dev");
    let listener = TcpListener::bind((options.host.as_str(), options.port)).map_err(|error| {
        format!(
            "dev server could not bind {}:{}: {error}",
            options.host, options.port
        )
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("failed to configure dev server: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("failed to read dev server address: {error}"))?;
    let url = format!("http://{}:{}/", options.host, address.port());

    let stopping = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&stopping);
    ctrlc::set_handler(move || signal.store(true, Ordering::SeqCst))
        .map_err(|error| format!("failed to install shutdown handler: {error}"))?;

    let mut watch_roots = watched_package_roots(&project).unwrap_or_else(|_| vec![project.clone()]);
    let mut files = watch_snapshot(&watch_roots)?;
    let version = AtomicU64::new(0);
    let mut build_available = rebuild(&project, &output, &version)?;
    println!("Dev server: {url}");
    println!("Watching {}", project.display());
    if options.open {
        open_browser(&url)?;
    }

    while !stopping.load(Ordering::SeqCst) {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(error) =
                        serve(stream, &output, &version, build_available, &options.host)
                    {
                        eprintln!("seseragi dev: {error}");
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(format!("dev server accept failed: {error}")),
            }
        }

        let next = watch_snapshot(&watch_roots)?;
        if next != files {
            let mut rebuilt = match rebuild(&project, &output, &version) {
                Ok(rebuilt) => rebuilt,
                Err(error) => {
                    eprintln!("seseragi dev: {error}");
                    eprintln!("Build failed");
                    false
                }
            };
            let (refreshed_roots, refresh_succeeded) = refresh_watch_roots(&project, &watch_roots);
            let roots_changed = refreshed_roots != watch_roots;
            for root in refreshed_roots
                .iter()
                .filter(|root| !watch_roots.contains(root))
            {
                println!("Watching {}", root.display());
            }
            // A lockfile can become valid between a failed rebuild and graph
            // refresh. When that refresh also discovers a new package root,
            // rebuild once against the stable graph before consuming its
            // snapshot; otherwise the dependency change has no later event to
            // trigger recovery.
            if !rebuilt && refresh_succeeded && roots_changed {
                rebuilt = match rebuild(&project, &output, &version) {
                    Ok(rebuilt) => rebuilt,
                    Err(error) => {
                        eprintln!("seseragi dev: {error}");
                        eprintln!("Build failed");
                        false
                    }
                };
            }
            build_available = rebuilt || build_available;
            watch_roots = refreshed_roots;
            if refresh_succeeded {
                // Only acknowledge what this attempt observed before building.
                // A repair/edit during compilation or graph refresh still needs
                // another build, even when it leaves the package roots unchanged.
                // Newly discovered roots likewise remain visible to the next poll.
                files = next;
            }
        }
        std::thread::sleep(WATCH_INTERVAL);
    }
    println!("Stopped dev server");
    Ok(0)
}

struct DevOptions {
    path: PathBuf,
    host: String,
    port: u16,
    open: bool,
}

impl DevOptions {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut path = None;
        let mut host = DEFAULT_HOST.to_owned();
        let mut port = DEFAULT_PORT;
        let mut open = false;
        let mut index = 0;
        while index < arguments.len() {
            match arguments[index].as_str() {
                "--host" => {
                    index += 1;
                    host = arguments
                        .get(index)
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| "--host requires an address".to_owned())?
                        .clone();
                }
                "--port" => {
                    index += 1;
                    let value = arguments
                        .get(index)
                        .ok_or_else(|| "--port requires a number".to_owned())?;
                    port = value
                        .parse::<u16>()
                        .map_err(|_| format!("invalid dev port `{value}`"))?;
                }
                "--open" => open = true,
                argument if argument.starts_with('-') => {
                    return Err(format!("unknown dev option `{argument}`"));
                }
                argument if path.is_none() => path = Some(PathBuf::from(argument)),
                argument => return Err(format!("unexpected dev argument `{argument}`")),
            }
            index += 1;
        }
        Ok(Self {
            path: path.unwrap_or_else(|| PathBuf::from(".")),
            host,
            port,
            open,
        })
    }
}

fn rebuild(project: &Path, output: &Path, version: &AtomicU64) -> Result<bool, String> {
    let started = Instant::now();
    let compiled = match compile_path(
        project,
        ProjectCommand::Dev,
        None,
        seseragi_runtime::DiagnosticFormat::Text,
    )? {
        LocalProjectCompilation::Compiled(compiled) => compiled,
        LocalProjectCompilation::Diagnostics => {
            eprintln!("Build failed ({})", elapsed(started));
            return Ok(false);
        }
    };
    build_local_project(&compiled.compiled, output, BuildTarget::Web)
        .map_err(|error| error.to_string())?;
    let next = version.fetch_add(1, Ordering::SeqCst) + 1;
    println!("Built web app ({}; reload {next})", elapsed(started));
    Ok(true)
}

fn elapsed(started: Instant) -> String {
    format!("{} ms", started.elapsed().as_millis())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WatchStamp {
    modified: Option<SystemTime>,
    length: u64,
    fingerprint: u64,
}

fn watched_package_roots(project: &Path) -> Result<Vec<PathBuf>, String> {
    seseragi_project::read_and_validate_development_lockfile(project)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let graph = seseragi_project::discover_local_package_graph(project)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    Ok(graph
        .packages()
        .map(|(_, package)| package.root().to_owned())
        .collect())
}

fn refresh_watch_roots(project: &Path, current: &[PathBuf]) -> (Vec<PathBuf>, bool) {
    match watched_package_roots(project) {
        Ok(roots) => (roots, true),
        Err(error) => {
            eprintln!("seseragi dev: failed to refresh watched package graph: {error}");
            let mut roots = current
                .iter()
                .filter(|root| root.is_dir())
                .cloned()
                .collect::<Vec<_>>();
            if !roots.iter().any(|root| root == project) {
                roots.push(project.to_owned());
            }
            roots.sort();
            roots.dedup();
            (roots, false)
        }
    }
}

fn watch_snapshot(roots: &[PathBuf]) -> Result<BTreeMap<PathBuf, WatchStamp>, String> {
    fn visit(
        root: &Path,
        directory: &Path,
        files: &mut BTreeMap<PathBuf, WatchStamp>,
        web: Option<&seseragi_project::ManifestWeb>,
    ) -> Result<(), String> {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(format!("failed to watch {}: {error}", directory.display())),
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(format!(
                        "failed to read watched directory {}: {error}",
                        directory.display()
                    ))
                }
            };
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("watched path is under root");
            if ignored(relative) {
                continue;
            }
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(format!(
                        "failed to inspect watched path {}: {error}",
                        path.display()
                    ))
                }
            };
            if metadata.is_dir() {
                visit(root, &path, files, web)?;
            } else if watched(relative)
                || web.is_some_and(|web| {
                    web.index
                        .as_ref()
                        .is_some_and(|index| relative == Path::new(index.as_str()))
                        || web
                            .public
                            .as_ref()
                            .is_some_and(|public| relative.starts_with(public.as_str()))
                })
            {
                let fingerprint = if metadata.is_file() {
                    let Some(fingerprint) = fingerprint(&path)? else {
                        // An editor or asset deletion can race the directory scan.
                        continue;
                    };
                    fingerprint
                } else {
                    0
                };
                files.insert(
                    path,
                    WatchStamp {
                        modified: metadata.modified().ok(),
                        length: metadata.len(),
                        fingerprint,
                    },
                );
            }
        }
        Ok(())
    }

    let mut files = BTreeMap::new();
    for root in roots {
        let web = fs::read_to_string(root.join("seseragi.toml"))
            .ok()
            .and_then(|source| seseragi_project::parse_manifest(&source).ok())
            .and_then(|manifest| manifest.web);
        visit(root, root, &mut files, web.as_ref())?;
    }
    Ok(files)
}

fn fingerprint(path: &Path) -> Result<Option<u64>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to read watched path {}: {error}",
                path.display()
            ))
        }
    };
    Ok(Some(
        bytes.into_iter().fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        }),
    ))
}

fn ignored(path: &Path) -> bool {
    matches!(
        path.components().next(),
        Some(Component::Normal(name))
            if name == ".git" || name == ".seseragi" || name == "dist" || name == "node_modules"
    )
}

fn watched(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("seseragi.toml")
        || path.extension().and_then(|extension| extension.to_str()) == Some("ssrg")
}

fn serve(
    mut stream: TcpStream,
    output: &Path,
    version: &AtomicU64,
    build_available: bool,
    configured_host: &str,
) -> Result<(), String> {
    // Accepted sockets can inherit the nonblocking listener state on macOS.
    // Complete large bundles/assets with bounded blocking writes instead of
    // abandoning the response at the first full socket buffer.
    stream
        .set_nonblocking(false)
        .map_err(|error| format!("failed to configure client: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("failed to configure client: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| format!("failed to configure client: {error}"))?;
    let request = match read_request_head(&mut stream) {
        Ok(Some(request)) => request,
        Ok(None) => return Ok(()),
        Err(_) => {
            // The incomplete request may be HEAD; do not send a response body.
            return response(&mut stream, 400, "text/plain; charset=utf-8", b"", "GET");
        }
    };
    let Some(line) = request.lines().next() else {
        return Ok(());
    };
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let request_path = parts.next().unwrap_or("");
    let local_address = stream
        .local_addr()
        .map_err(|error| format!("failed to read client address: {error}"))?;
    // Binding to loopback alone does not authorize a browser's origin: a
    // rebinding hostname can resolve to that same address. Never resolve Host
    // or trust proxy headers when deciding which authority may read artifacts.
    if !valid_request_host(&request, configured_host, local_address) {
        return response(
            &mut stream,
            403,
            "text/plain; charset=utf-8",
            b"Forbidden request host\n",
            method,
        );
    }
    if !matches!(method, "GET" | "HEAD") {
        return response(
            &mut stream,
            405,
            "text/plain; charset=utf-8",
            b"Method not allowed\n",
            method,
        );
    }
    let request_path = request_path.split('?').next().unwrap_or(request_path);
    if request_path == RELOAD_PATH {
        let body = format!("{}\n", version.load(Ordering::SeqCst));
        return response(
            &mut stream,
            200,
            "text/plain; charset=utf-8",
            body.as_bytes(),
            method,
        );
    }
    if !build_available {
        let body = inject_reload(
            b"<!doctype html><title>Seseragi build failed</title><body><h1>Build failed</h1><p>Fix the compiler diagnostics; this page will become available after the next successful build.</p></body>",
        );
        return response(&mut stream, 503, "text/html; charset=utf-8", &body, method);
    }
    let Some(relative) = safe_path(request_path) else {
        return response(
            &mut stream,
            400,
            "text/plain; charset=utf-8",
            b"Invalid request path\n",
            method,
        );
    };
    let path = output.join(&relative);
    let metadata = fs::metadata(&path).ok();
    let path = if metadata.as_ref().is_some_and(|value| value.is_file()) {
        path
    } else if !relative.to_string_lossy().contains('.') {
        output.join("index.html")
    } else {
        return response(
            &mut stream,
            404,
            "text/plain; charset=utf-8",
            b"Not found\n",
            method,
        );
    };
    let mut body =
        fs::read(&path).map_err(|error| format!("failed to serve {}: {error}", path.display()))?;
    if path.file_name().and_then(|name| name.to_str()) == Some("index.html") {
        body = inject_reload(&body);
    }
    response(&mut stream, 200, content_type(&path), &body, method)
}

fn read_request_head(stream: &mut TcpStream) -> std::io::Result<Option<String>> {
    let started = Instant::now();
    let mut request = [0_u8; 8192];
    let mut length = 0;
    while started.elapsed() < Duration::from_secs(1) && length < request.len() {
        match stream.read(&mut request[length..]) {
            Ok(0) if length == 0 => return Ok(None),
            Ok(0) => break,
            Ok(read) => {
                length += read;
                if let Some(end) = request[..length]
                    .windows(4)
                    .position(|part| part == b"\r\n\r\n")
                {
                    return String::from_utf8(request[..end + 4].to_vec())
                        .map(Some)
                        .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error));
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        ErrorKind::InvalidData,
        "incomplete or oversized request headers",
    ))
}

fn valid_request_host(request: &str, configured_host: &str, local_address: SocketAddr) -> bool {
    let mut host = None;
    for line in request
        .split("\r\n")
        .skip(1)
        .take_while(|line| !line.is_empty())
    {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
            || value
                .bytes()
                .any(|byte| (byte < 32 && byte != b'\t') || byte == 127)
        {
            return false;
        }
        if name.eq_ignore_ascii_case("host") {
            if host.is_some() {
                return false;
            }
            host = Some(value.trim_matches([' ', '\t']));
        }
    }
    host.is_some_and(|host| allowed_authority(host, configured_host, local_address))
}

fn allowed_authority(authority: &str, configured_host: &str, local_address: SocketAddr) -> bool {
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let Some((host, suffix)) = rest.split_once(']') else {
            return false;
        };
        if host.parse::<std::net::Ipv6Addr>().is_err() {
            return false;
        }
        let port = if suffix.is_empty() {
            None
        } else if let Some(port) = suffix.strip_prefix(':') {
            Some(port)
        } else {
            return false;
        };
        (host, port)
    } else {
        match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        }
    };
    if port.is_some_and(|port| {
        port.is_empty()
            || !port.bytes().all(|byte| byte.is_ascii_digit())
            || port.parse::<u16>().ok() != Some(local_address.port())
    }) {
        return false;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return ip.to_canonical() == local_address.ip().to_canonical()
            || configured_host
                .parse::<IpAddr>()
                .map(|ip| ip.to_canonical())
                .ok()
                == Some(ip.to_canonical());
    }
    let host = host.strip_suffix('.').unwrap_or(host);
    let configured_host = configured_host.strip_suffix('.').unwrap_or(configured_host);
    !host.is_empty()
        && host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".-_".contains(&byte))
        && (host.eq_ignore_ascii_case(configured_host)
            || (local_address.ip().to_canonical().is_loopback()
                && host.eq_ignore_ascii_case("localhost")))
}

fn safe_path(request_path: &str) -> Option<PathBuf> {
    let decoded = decode_url_path(request_path)?;
    let trimmed = decoded.trim_start_matches('/');
    let path = if trimmed.is_empty() {
        "index.html"
    } else {
        trimmed
    };
    if path.contains('\\') || path.contains(':') || path.chars().any(char::is_control) {
        return None;
    }
    let path = Path::new(path);
    if path
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Some(path.to_owned())
    } else {
        None
    }
}

fn decode_url_path(value: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut input = value.as_bytes().iter().copied();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = (input.next()? as char).to_digit(16)?;
            let low = (input.next()? as char).to_digit(16)?;
            bytes.push((high * 16 + low) as u8);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

fn inject_reload(index: &[u8]) -> Vec<u8> {
    const SCRIPT: &str = concat!(
        "<script type=\"module\">\n",
        "const endpoint = '/__seseragi_dev/version';\n",
        "let version = await (await fetch(endpoint, { cache: 'no-store' })).text();\n",
        "setInterval(async () => {\n",
        "  try {\n",
        "    const next = await (await fetch(endpoint, { cache: 'no-store' })).text();\n",
        "    if (next !== version) location.reload();\n",
        "  } catch {}\n",
        "}, 250);\n",
        "</script>\n",
    );
    let source = String::from_utf8_lossy(index);
    if let Some(position) = source.rfind("</body>") {
        let mut output = String::with_capacity(source.len() + SCRIPT.len());
        output.push_str(&source[..position]);
        output.push_str(SCRIPT);
        output.push_str(&source[position..]);
        output.into_bytes()
    } else {
        [index, SCRIPT.as_bytes()].concat()
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("map") | Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("txt") => "text/plain; charset=utf-8",
        Some("webmanifest") => "application/manifest+json",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    method: &str,
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        503 => "Service Unavailable",
        _ => "Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .and_then(|()| {
        if method == "HEAD" {
            Ok(())
        } else {
            stream.write_all(body)
        }
    })
    .map_err(|error| format!("failed to write response: {error}"))
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", ""]);
        command
    };
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err("--open is not supported on this host".to_owned());
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    command
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to open browser: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorizes_only_explicit_or_connected_hosts() {
        let local = "127.0.0.1:3000".parse().unwrap();
        for host in [
            "127.0.0.1",
            "127.0.0.1:3000",
            "LOCALHOST:3000",
            "localhost.",
        ] {
            assert!(allowed_authority(host, "127.0.0.1", local), "{host}");
        }
        for host in [
            "attacker.example",
            "localhost.attacker.example",
            "127.0.0.1.attacker.example",
            "localhost@attacker.example",
            "localhost/path",
            "localhost%00",
            "localhost:3001",
            "localhost:",
            "localhost:+3000",
            "localhost:65536",
            "[::1]junk",
            "[localhost]",
            "",
            ":3000",
            "[::1",
            "::1",
            "127.0.0.2",
            "localhost,attacker.example",
        ] {
            assert!(!allowed_authority(host, "127.0.0.1", local), "{host}");
        }
        assert!(allowed_authority("APP.TEST.:3000", "app.test", local));
        assert!(!allowed_authority("other.test", "app.test", local));
        let lan = "192.0.2.10:4321".parse().unwrap();
        assert!(allowed_authority("192.0.2.10:4321", "0.0.0.0", lan));
        assert!(allowed_authority("0.0.0.0:4321", "0.0.0.0", lan));
        assert!(!allowed_authority("attacker.example", "0.0.0.0", lan));
        assert!(!allowed_authority("localhost", "0.0.0.0", lan));
        let ipv6 = "[::1]:4321".parse().unwrap();
        assert!(allowed_authority("[0:0:0:0:0:0:0:1]:4321", "::1", ipv6));
        assert!(allowed_authority("localhost:4321", "::1", ipv6));
        let mapped = "[::ffff:127.0.0.1]:4321".parse().unwrap();
        assert!(allowed_authority("127.0.0.1:4321", "::", mapped));
        assert!(allowed_authority("localhost:4321", "::", mapped));
    }

    #[test]
    fn requires_one_well_formed_host_header() {
        let local = "127.0.0.1:3000".parse().unwrap();
        assert!(valid_request_host(
            "GET / HTTP/1.1\r\nhOsT:\tLOCALHOST:3000 \t\r\n\r\n",
            "127.0.0.1",
            local
        ));
        for headers in [
            "",
            "Host: attacker.example\r\nX-Forwarded-Host: localhost\r\n",
            "Host: localhost\r\nHost: attacker.example\r\n",
            "Host: attacker.example\r\nhOsT: localhost\r\n",
            "Host: localhost\r\nHost: localhost\r\n",
            "Host: \r\n",
            "Host : localhost\r\n",
            " Host: localhost\r\n",
            "Host: localhost\r\n continuation\r\n",
            "Host: localhost\0\r\n",
            "Host: localhost\r\nBad header\r\n",
        ] {
            assert!(
                !valid_request_host(
                    &format!("GET / HTTP/1.1\r\n{headers}\r\n"),
                    "127.0.0.1",
                    local
                ),
                "{headers:?}"
            );
        }
    }

    fn request_chunks(chunks: &[&[u8]], build_available: bool) -> String {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            // Valid reload/failure requests do not need a filesystem output.
            serve(
                stream,
                Path::new("missing-dev-test-output"),
                &AtomicU64::new(7),
                build_available,
                "127.0.0.1",
            )
            .unwrap();
        });
        let mut client = TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        for (index, chunk) in chunks.iter().enumerate() {
            if index != 0 {
                std::thread::sleep(Duration::from_millis(20));
            }
            client.write_all(chunk).unwrap();
        }
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut result = String::new();
        client.read_to_string(&mut result).unwrap();
        server.join().unwrap();
        result
    }

    #[test]
    fn complete_headers_are_checked_before_reload_and_build_failure_responses() {
        let valid = request_chunks(
            &[
                b"GET /__seseragi_dev/version HTTP/1.1\r\nHo",
                b"st: local",
                b"host\r\n\r\n",
            ],
            true,
        );
        assert!(valid.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(valid.ends_with("\r\n\r\n7\n"));
        let valid_head = request_chunks(
            &[b"HEAD /__seseragi_dev/version HTTP/1.1\r\nHost: localhost\r\n\r\n"],
            true,
        );
        assert!(valid_head.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(valid_head.ends_with("\r\n\r\n"));
        for available in [true, false] {
            for path in ["/", RELOAD_PATH] {
                let first = format!("GET {path} HTTP/1.1\r\nHost: ");
                let result =
                    request_chunks(&[first.as_bytes(), b"attacker.example\r\n\r\n"], available);
                assert!(result.starts_with("HTTP/1.1 403 Forbidden\r\n"), "{result}");
            }
        }
        let failure = request_chunks(&[b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"], false);
        assert!(failure.starts_with("HTTP/1.1 503 Service Unavailable\r\n"));
        for invalid in [
            b"GET / HTTP/1.1\r\nHost: localhost".as_slice(),
            b"HEAD / HTTP/1.1\r\nHost: localhost".as_slice(),
            &vec![b'x'; 8192],
        ] {
            let result = request_chunks(&[invalid], false);
            assert!(
                result.starts_with("HTTP/1.1 400 Bad Request\r\n"),
                "{result}"
            );
            assert!(result.ends_with("\r\n\r\n"));
            assert!(result.contains("Content-Length: 0\r\n"));
        }
    }

    #[test]
    fn parses_options_and_rejects_invalid_ports() {
        let options = DevOptions::parse(&[
            "app".to_owned(),
            "--host".to_owned(),
            "0.0.0.0".to_owned(),
            "--port".to_owned(),
            "4000".to_owned(),
            "--open".to_owned(),
        ])
        .unwrap();
        assert_eq!(options.path, Path::new("app"));
        assert_eq!(options.host, "0.0.0.0");
        assert_eq!(options.port, 4000);
        assert!(options.open);
        assert!(DevOptions::parse(&["--port".to_owned(), "nope".to_owned()]).is_err());
    }

    #[test]
    fn confines_static_paths_and_injects_reload_client() {
        assert_eq!(
            safe_path("/assets/app.js"),
            Some(PathBuf::from("assets/app.js"))
        );
        assert_eq!(safe_path("/"), Some(PathBuf::from("index.html")));
        assert_eq!(safe_path("/../secret"), None);
        let index = inject_reload(b"<body>app</body>");
        let index = String::from_utf8(index).unwrap();
        assert!(index.contains(RELOAD_PATH));
        assert!(index.ends_with("</body>"));
        let failed = inject_reload(b"<body>Build failed</body>");
        assert!(String::from_utf8(failed).unwrap().contains(RELOAD_PATH));
    }

    #[test]
    fn watch_snapshots_survive_files_deleted_during_enumeration() {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "seseragi-watch-delete-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("main.ssrg");
        fs::write(&path, "let value = 1\n").unwrap();
        let enumerated = fs::read_dir(&directory).unwrap().next().unwrap().unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(fingerprint(&enumerated.path()).unwrap(), None);
        let stopping = Arc::new(AtomicBool::new(false));
        let writer_stop = Arc::clone(&stopping);
        let writer = std::thread::spawn(move || {
            while !writer_stop.load(Ordering::SeqCst) {
                fs::write(&path, "let value = 1\n").unwrap();
                fs::remove_file(&path).unwrap();
            }
        });
        let mut failure = None;
        for _ in 0..1000 {
            if let Err(error) = watch_snapshot(&[directory.clone()]) {
                failure = Some(error);
                break;
            }
        }
        stopping.store(true, Ordering::SeqCst);
        writer.join().unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(failure, None);
    }

    #[test]
    fn keeps_existing_roots_when_package_graph_refresh_fails() {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "seseragi-dev-watch-roots-{}-{unique}",
            std::process::id()
        ));
        let project = directory.join("project");
        let dependency = directory.join("dependency");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&dependency).unwrap();
        fs::write(project.join("seseragi.toml"), "[package\n").unwrap();

        let (roots, succeeded) = refresh_watch_roots(&project, &[dependency.clone()]);
        assert_eq!(roots, vec![dependency, project]);
        assert!(!succeeded);
        fs::remove_dir_all(directory).unwrap();
    }
}
