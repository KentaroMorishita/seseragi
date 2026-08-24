use crate::local_project::{compile_path, LocalProjectCompilation};
use seseragi_project::ProjectCommand;
use seseragi_runtime::{build_local_project, BuildTarget};
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
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
                    if let Err(error) = serve(stream, &output, &version, build_available) {
                        eprintln!("seseragi dev: {error}");
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(format!("dev server accept failed: {error}")),
            }
        }

        let next = watch_snapshot(&watch_roots)?;
        if next != files {
            let rebuilt = match rebuild(&project, &output, &version) {
                Ok(rebuilt) => rebuilt,
                Err(error) => {
                    eprintln!("seseragi dev: {error}");
                    eprintln!("Build failed");
                    false
                }
            };
            build_available = rebuilt || build_available;
            let (refreshed_roots, refresh_succeeded) = refresh_watch_roots(&project, &watch_roots);
            for root in refreshed_roots
                .iter()
                .filter(|root| !watch_roots.contains(root))
            {
                println!("Watching {}", root.display());
            }
            watch_roots = refreshed_roots;
            if refresh_succeeded {
                files = watch_snapshot(&watch_roots)?;
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
    let compiled = match compile_path(project, ProjectCommand::Dev, None)? {
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
    ) -> Result<(), String> {
        for entry in fs::read_dir(directory)
            .map_err(|error| format!("failed to watch {}: {error}", directory.display()))?
        {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read watched directory {}: {error}",
                    directory.display()
                )
            })?;
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("watched path is under root");
            if ignored(relative) {
                continue;
            }
            let metadata = entry.metadata().map_err(|error| {
                format!("failed to inspect watched path {}: {error}", path.display())
            })?;
            if metadata.is_dir() {
                visit(root, &path, files)?;
            } else if watched(relative) {
                let fingerprint = fingerprint(&path)?;
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
        visit(root, root, &mut files)?;
    }
    Ok(files)
}

fn fingerprint(path: &Path) -> Result<u64, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read watched path {}: {error}", path.display()))?;
    Ok(bytes.into_iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    }))
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
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| format!("failed to configure client: {error}"))?;
    let mut request = [0_u8; 8192];
    let length = match stream.read(&mut request) {
        Ok(0) => return Ok(()),
        Ok(length) => length,
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            return Ok(())
        }
        Err(error) => return Err(format!("failed to read request: {error}")),
    };
    let request = String::from_utf8_lossy(&request[..length]);
    let Some(line) = request.lines().next() else {
        return Ok(());
    };
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let request_path = parts.next().unwrap_or("");
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
    let relative = safe_path(request_path).ok_or_else(|| "invalid request path".to_owned())?;
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

fn safe_path(request_path: &str) -> Option<PathBuf> {
    let trimmed = request_path.trim_start_matches('/');
    let path = if trimmed.is_empty() {
        "index.html"
    } else {
        trimmed
    };
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
