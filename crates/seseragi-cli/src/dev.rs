use crate::local_project::prepare_package_build;
use crate::target::Target;
use seseragi_project::ProjectCommand;
use seseragi_runtime::{build_local_project_with_programs, BuildTarget};
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
const CLIENT_IO_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) fn dev(arguments: &[String]) -> Result<i32, String> {
    let mut options = DevOptions::parse(arguments)?;
    // A package whose run.target is wasm rebuilds its program library like
    // `--target wasm`, without a dev server.
    if options.target != Some(Target::Native)
        && crate::build::wasm_package(&options.path, options.target.and_then(Target::project))?
            .is_some()
    {
        if options.web_options {
            return Err("--host, --port and --open require --target web".into());
        }
        options.target = Some(Target::Project(seseragi_project::ProjectTarget::Wasm));
    }
    let serverless = matches!(
        options.target,
        Some(Target::Native | Target::Project(seseragi_project::ProjectTarget::Wasm))
    );
    if options.target != Some(Target::Native) && !options.path.is_dir() {
        return Err("dev requires a package directory".to_owned());
    }
    let package = if options.target == Some(Target::Native) {
        crate::local_project::containing_package(&options.path)
    } else {
        None
    };
    let input = package.as_deref().unwrap_or(&options.path);
    let project = input.canonicalize().map_err(|error| {
        format!(
            "failed to resolve package {}: {error}",
            options.path.display()
        )
    })?;
    let root = if project.is_dir() {
        project.as_path()
    } else {
        project.parent().unwrap()
    };
    let output = root.join(".seseragi/dev");
    let listener = if serverless {
        None
    } else {
        let manifest_path = project.join("seseragi.toml");
        let manifest_source = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
        let manifest = seseragi_project::parse_manifest(&manifest_source)
            .map_err(|error| format!("{}: {error}", error.code()))?;
        seseragi_project::select_project_target(
            ProjectCommand::Dev,
            options.target.and_then(Target::project),
            manifest.run.as_ref().and_then(|run| run.target.as_ref()),
            None,
        )
        .map_err(|error| error.to_string())?;
        let listener =
            TcpListener::bind((options.host.as_str(), options.port)).map_err(|error| {
                format!(
                    "dev server could not bind {}:{}: {error}",
                    options.host, options.port
                )
            })?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("failed to configure dev server: {error}"))?;
        Some(listener)
    };
    let url = listener
        .as_ref()
        .map(|listener| {
            listener
                .local_addr()
                .map(|address| format!("http://{}:{}/", options.host, address.port()))
                .map_err(|error| format!("failed to read dev server address: {error}"))
        })
        .transpose()?;

    let stopping = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&stopping);
    ctrlc::set_handler(move || signal.store(true, Ordering::SeqCst))
        .map_err(|error| format!("failed to install shutdown handler: {error}"))?;

    let mut watch_roots = watched_package_roots(&project).unwrap_or_else(|_| vec![root.to_owned()]);
    let mut files = Some(watch_snapshot(&watch_roots)?);
    let version = AtomicU64::new(0);
    let rebuild = || rebuild_target(&project, &output, &version, options.target);
    let mut build_available = if serverless {
        rebuild().unwrap_or_else(|error| {
            eprintln!("seseragi dev: {error}\nBuild failed");
            false
        })
    } else {
        rebuild()?
    };
    if let Some(url) = &url {
        println!("Dev server: {url}");
    }
    println!("Watching {}", project.display());
    if options.open {
        open_browser(url.as_deref().expect("web dev has a URL"))?;
    }

    while !stopping.load(Ordering::SeqCst) {
        if let Some(listener) = &listener {
            if !serve_pending_requests(listener, &stopping, |stream| {
                if let Err(error) = serve(
                    stream,
                    &output,
                    &version,
                    build_available,
                    &options.host,
                    &stopping,
                ) {
                    if !stopping.load(Ordering::SeqCst) {
                        eprintln!("seseragi dev: {error}");
                    }
                }
            })? {
                break;
            }
        }
        build_available =
            rebuild_if_changed(&project, &mut watch_roots, &mut files, &stopping, rebuild)?
                || build_available;
        if stopping.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(WATCH_INTERVAL);
    }
    println!(
        "{}",
        if serverless {
            "Stopped watching"
        } else {
            "Stopped dev server"
        }
    );
    Ok(0)
}

fn rebuild_if_changed(
    project: &Path,
    watch_roots: &mut Vec<PathBuf>,
    files: &mut Option<BTreeMap<PathBuf, WatchStamp>>,
    stopping: &AtomicBool,
    mut rebuild: impl FnMut() -> Result<bool, String>,
) -> Result<bool, String> {
    let next = watch_snapshot(watch_roots)?;
    if stopping.load(Ordering::SeqCst) || files.as_ref() == Some(&next) {
        return Ok(false);
    }
    let mut attempt = || match rebuild() {
        Ok(rebuilt) => rebuilt,
        Err(error) => {
            eprintln!("seseragi dev: {error}");
            eprintln!("Build failed");
            false
        }
    };
    let mut rebuilt = attempt();
    if stopping.load(Ordering::SeqCst) {
        return Ok(false);
    }
    let (refreshed_roots, refresh_succeeded) = refresh_watch_roots(project, watch_roots);
    let roots_changed = &refreshed_roots != watch_roots;
    for root in refreshed_roots
        .iter()
        .filter(|root| !watch_roots.contains(root))
    {
        println!("Watching {}", root.display());
    }
    // A lockfile can become valid between a failed rebuild and graph refresh.
    // Rebuild against a newly discovered package before acknowledging the change.
    if !rebuilt && refresh_succeeded && roots_changed && !stopping.load(Ordering::SeqCst) {
        rebuilt = attempt();
    }
    *watch_roots = refreshed_roots;
    // A failed graph refresh must stay pending even if a manifest is restored
    // to identical contents or only the unwatched lockfile is repaired.
    // Otherwise, acknowledge only inputs observed before building: edits made
    // during the build or graph refresh still belong to the next poll.
    *files = refresh_succeeded.then_some(next);
    Ok(rebuilt)
}

fn serve_pending_requests(
    listener: &TcpListener,
    stopping: &AtomicBool,
    mut serve: impl FnMut(TcpStream),
) -> Result<bool, String> {
    loop {
        if stopping.load(Ordering::SeqCst) {
            return Ok(false);
        }
        match listener.accept() {
            Ok((stream, _)) => serve(stream),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                return Ok(!stopping.load(Ordering::SeqCst));
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(format!("dev server accept failed: {error}")),
        }
    }
}

struct DevOptions {
    path: PathBuf,
    host: String,
    port: u16,
    open: bool,
    target: Option<Target>,
    web_options: bool,
}

impl DevOptions {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut path = None;
        let mut host = DEFAULT_HOST.to_owned();
        let mut port = DEFAULT_PORT;
        let mut open = false;
        let mut target = None;
        let mut web_options = false;
        let mut index = 0;
        while index < arguments.len() {
            match arguments[index].as_str() {
                "--target" => {
                    if target.is_some() {
                        return Err("--target may only be specified once".into());
                    }
                    index += 1;
                    let value = arguments
                        .get(index)
                        .ok_or("--target requires web, wasm or native")?;
                    let selected = Target::parse(value)?;
                    if selected == Target::Project(seseragi_project::ProjectTarget::Process) {
                        return Err("dev does not support the `process` target".into());
                    }
                    target = Some(selected);
                }
                "--host" => {
                    web_options = true;
                    index += 1;
                    host = arguments
                        .get(index)
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| "--host requires an address".to_owned())?
                        .clone();
                }
                "--port" => {
                    web_options = true;
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
        let web_options = web_options || open;
        if matches!(
            target,
            Some(Target::Native | Target::Project(seseragi_project::ProjectTarget::Wasm))
        ) && web_options
        {
            return Err("--host, --port and --open require --target web".into());
        }
        Ok(Self {
            path: path.unwrap_or_else(|| PathBuf::from(".")),
            target,
            host,
            port,
            open,
            web_options,
        })
    }
}

fn rebuild_target(
    project: &Path,
    output: &Path,
    version: &AtomicU64,
    target: Option<Target>,
) -> Result<bool, String> {
    if target == Some(Target::Project(seseragi_project::ProjectTarget::Wasm)) {
        let started = Instant::now();
        // The program library is published beside the web dev output, which
        // a web dev session of the same package replaces as a whole.
        let Some(built) = crate::experimental::program_wasm::build_package_program(
            project,
            ProjectCommand::Dev,
            None,
            &output.with_file_name("dev-wasm"),
        )?
        else {
            eprintln!("Build failed ({})", elapsed(started));
            return Ok(false);
        };
        println!("Built {} ({})", built.display(), elapsed(started));
        return Ok(true);
    }
    if target != Some(Target::Native) {
        return rebuild(project, output, version, target.and_then(Target::project));
    }
    let started = Instant::now();
    crate::experimental::build_native(project, output, ProjectCommand::Dev, Default::default())?;
    println!("Built native app ({})", elapsed(started));
    Ok(true)
}

fn rebuild(
    project: &Path,
    output: &Path,
    version: &AtomicU64,
    target: Option<seseragi_project::ProjectTarget>,
) -> Result<bool, String> {
    let started = Instant::now();
    let Some(prepared) = prepare_package_build(project, ProjectCommand::Dev, target, None)? else {
        eprintln!("Build failed ({})", elapsed(started));
        return Ok(false);
    };
    build_local_project_with_programs(
        &prepared.project.compiled,
        output,
        BuildTarget::Web,
        prepared.project.process_run_options,
        seseragi_runtime::artifact::ArtifactOptions::default(),
        &prepared.programs,
    )
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
    if project.is_file() {
        return Ok(vec![project
            .parent()
            .ok_or("source file has no parent")?
            .to_owned()]);
    }
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
                let Some(stamp) = watch_stamp(&path, &metadata)? else {
                    // An editor or asset deletion can race the directory scan.
                    continue;
                };
                files.insert(path, stamp);
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

fn watch_stamp(path: &Path, metadata: &fs::Metadata) -> Result<Option<WatchStamp>, String> {
    if !metadata.is_file() {
        return Ok(Some(WatchStamp {
            modified: metadata.modified().ok(),
            length: metadata.len(),
            fingerprint: 0,
        }));
    }
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
    // An editor can write between metadata lookup and the read. Derive the
    // entire regular-file stamp from the same bytes, so stale size/mtime alone
    // cannot queue another long build after these contents were already built.
    Ok(Some(WatchStamp {
        modified: None,
        length: bytes.len() as u64,
        fingerprint: bytes.into_iter().fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        }),
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
    stream: TcpStream,
    output: &Path,
    version: &AtomicU64,
    build_available: bool,
    configured_host: &str,
    stopping: &AtomicBool,
) -> Result<(), String> {
    // Accepted sockets can inherit the nonblocking listener state on macOS.
    // Complete large bundles/assets with bounded blocking writes instead of
    // abandoning the response at the first full socket buffer.
    stream
        .set_nonblocking(false)
        .map_err(|error| format!("failed to configure client: {error}"))?;
    stream
        .set_write_timeout(Some(CLIENT_IO_POLL_INTERVAL))
        .map_err(|error| format!("failed to configure client: {error}"))?;
    stream
        .set_read_timeout(Some(CLIENT_IO_POLL_INTERVAL))
        .map_err(|error| format!("failed to configure client: {error}"))?;
    let local_address = stream
        .local_addr()
        .map_err(|error| format!("failed to read client address: {error}"))?;
    let mut stream = ClientIo {
        stream,
        stopping,
        deadline: Instant::now() + Duration::from_secs(5),
    };
    let request = match read_request_head(&mut stream, stopping) {
        Ok(Some(request)) => request,
        Ok(None) => return Ok(()),
        Err(_) if stopping.load(Ordering::SeqCst) => return Ok(()),
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
            version.load(Ordering::SeqCst),
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
        body = inject_reload(&body, version.load(Ordering::SeqCst));
    }
    response(&mut stream, 200, content_type(&path), &body, method)
}

struct ClientIo<'a, S> {
    stream: S,
    stopping: &'a AtomicBool,
    deadline: Instant,
}

impl<S: Read> Read for ClientIo<'_, S> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(bytes)
    }
}

impl<S: Write> Write for ClientIo<'_, S> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        loop {
            // write_all retries Interrupted itself, so use ConnectionAborted
            // when stopping. Each socket wait is short, and partial writes must
            // not restart the deadline for the whole response.
            if self.stopping.load(Ordering::SeqCst) {
                return Err(std::io::Error::new(
                    ErrorKind::ConnectionAborted,
                    "dev server stopping",
                ));
            }
            if Instant::now() >= self.deadline {
                return Err(std::io::Error::new(
                    ErrorKind::TimedOut,
                    "dev response timed out",
                ));
            }
            match self.stream.write(bytes) {
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                    ) => {}
                result => return result,
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

fn read_request_head(
    stream: &mut impl Read,
    stopping: &AtomicBool,
) -> std::io::Result<Option<String>> {
    let started = Instant::now();
    let mut request = [0_u8; 8192];
    let mut length = 0;
    while started.elapsed() < Duration::from_secs(1) && length < request.len() {
        if stopping.load(Ordering::SeqCst) {
            return Err(std::io::Error::new(
                ErrorKind::ConnectionAborted,
                "dev server stopping",
            ));
        }
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
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) => {}
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

fn inject_reload(index: &[u8], build_version: u64) -> Vec<u8> {
    const SCRIPT: &str = concat!(
        "<script type=\"module\">\n",
        "const endpoint = '/__seseragi_dev/version';\n",
        "const version = __SESERAGI_BUILD_VERSION__;\n",
        "let reloading = false;\n",
        "setInterval(async () => {\n",
        "  try {\n",
        "    const next = await (await fetch(endpoint, { cache: 'no-store' })).text();\n",
        "    if (next !== version && !reloading) {\n",
        "      reloading = true;\n",
        "      const closing = new Event('seseragi:before-reload', { cancelable: true });\n",
        "      if (globalThis.dispatchEvent(closing)) location.reload();\n",
        "    }\n",
        "  } catch {}\n",
        "}, 250);\n",
        "</script>\n",
    );
    // Pin the version served with this HTML. Fetching the initial version in
    // the browser could acknowledge a newer build without ever loading its UI.
    let script = SCRIPT.replace(
        "__SESERAGI_BUILD_VERSION__",
        &serde_json::to_string(&format!("{build_version}\n")).expect("version serializes"),
    );
    let source = String::from_utf8_lossy(index);
    if let Some(position) = source.rfind("</body>") {
        let mut output = String::with_capacity(source.len() + script.len());
        output.push_str(&source[..position]);
        output.push_str(&script);
        output.push_str(&source[position..]);
        output.into_bytes()
    } else {
        [index, script.as_bytes()].concat()
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
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
    stream: &mut impl Write,
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

    fn watch_project() -> (tempfile::TempDir, PathBuf, PathBuf, String) {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("project");
        let dependency = directory.path().join("release-copy");
        for root in [&project, &dependency] {
            fs::create_dir_all(root.join("src")).unwrap();
        }
        fs::write(dependency.join("seseragi.toml"), "[package]\nname = \"fixture/release-copy\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[exports]\n\".\" = \"lib\"\n").unwrap();
        fs::write(dependency.join("src/lib.ssrg"), "pub let value: Int = 1\n").unwrap();
        let manifest = "[package]\nname = \"fixture/watch-app\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"web\"\n\n[dependencies]\nrelease-copy = { package = \"fixture/release-copy\", path = \"../release-copy\" }\n";
        fs::write(project.join("seseragi.toml"), manifest).unwrap();
        fs::write(project.join("src/main.ssrg"), "pub let value: Int = 42\n").unwrap();
        write_watch_lock(&project);
        (
            directory,
            project.canonicalize().unwrap(),
            dependency.canonicalize().unwrap(),
            manifest.into(),
        )
    }

    fn write_watch_lock(project: &Path) {
        crate::lock::write(
            project,
            &seseragi_project::generate_lockfile(project).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn watch_retries_an_identical_manifest_after_failed_graph_refresh() {
        let (_directory, project, dependency, manifest) = watch_project();
        let mut roots = watched_package_roots(&project).unwrap();
        let mut files = Some(watch_snapshot(&roots).unwrap());
        let original = files.clone();
        let stopping = AtomicBool::new(false);
        let attempts = std::cell::Cell::new(0);
        let attempt = || {
            attempts.set(attempts.get() + 1);
            watched_package_roots(&project).map(|_| true)
        };
        fs::write(project.join("seseragi.toml"), "[package\n").unwrap();
        assert!(!rebuild_if_changed(&project, &mut roots, &mut files, &stopping, attempt).unwrap());
        assert!(
            roots.contains(&dependency),
            "failed refresh must retain the dependency"
        );
        fs::write(project.join("seseragi.toml"), manifest).unwrap();
        assert_eq!(Some(watch_snapshot(&roots).unwrap()), original);
        assert!(
            rebuild_if_changed(&project, &mut roots, &mut files, &stopping, attempt).unwrap(),
            "restoring identical manifest contents must retry after a failed graph refresh"
        );
        assert_eq!(attempts.get(), 2);
        assert!(!rebuild_if_changed(&project, &mut roots, &mut files, &stopping, attempt).unwrap());
        assert_eq!(
            attempts.get(),
            2,
            "successful recovery must resume normal change polling"
        );
    }

    #[test]
    fn watch_retries_a_lock_only_repair_without_a_new_content_change() {
        let (_directory, project, _dependency, manifest) = watch_project();
        let mut roots = watched_package_roots(&project).unwrap();
        let mut files = Some(watch_snapshot(&roots).unwrap());
        let stopping = AtomicBool::new(false);
        let attempts = std::cell::Cell::new(0);
        let attempt = || {
            attempts.set(attempts.get() + 1);
            watched_package_roots(&project).map(|_| true)
        };
        fs::write(project.join("seseragi.toml"), "[package\n").unwrap();
        assert!(!rebuild_if_changed(&project, &mut roots, &mut files, &stopping, attempt).unwrap());
        fs::write(project.join("seseragi.toml"), manifest).unwrap();
        fs::write(project.join("seseragi.lock"), "invalid lockfile").unwrap();
        assert!(!rebuild_if_changed(&project, &mut roots, &mut files, &stopping, attempt).unwrap());
        assert_eq!(
            attempts.get(),
            2,
            "the failed refresh must remain pending after contents are restored"
        );
        let before_lock_repair = watch_snapshot(&roots).unwrap();
        write_watch_lock(&project);
        assert_eq!(watch_snapshot(&roots).unwrap(), before_lock_repair);
        assert!(rebuild_if_changed(&project, &mut roots, &mut files, &stopping, attempt).unwrap());
        assert_eq!(attempts.get(), 3);
    }

    #[test]
    fn watch_keeps_a_manifest_repair_during_a_failed_build_pending() {
        let (_directory, project, _dependency, manifest) = watch_project();
        let mut roots = watched_package_roots(&project).unwrap();
        let original_roots = roots.clone();
        let mut files = Some(watch_snapshot(&roots).unwrap());
        let stopping = AtomicBool::new(false);
        fs::write(project.join("seseragi.toml"), "[package\n").unwrap();
        assert!(
            !rebuild_if_changed(&project, &mut roots, &mut files, &stopping, || {
                fs::write(project.join("seseragi.toml"), &manifest).unwrap();
                Ok(false)
            })
            .unwrap()
        );
        assert_eq!(roots, original_roots);
        assert!(
            rebuild_if_changed(&project, &mut roots, &mut files, &stopping, || {
                watched_package_roots(&project).map(|_| true)
            })
            .unwrap(),
            "graph refresh must not consume a repair made during a failed build"
        );
    }

    #[test]
    fn watch_discovers_a_new_dependency_after_a_failed_build_and_observes_its_repair() {
        let (_directory, project, dependency, manifest) = watch_project();
        let manifest_path = project.join("seseragi.toml");
        fs::write(
            &manifest_path,
            manifest.split("\n[dependencies]").next().unwrap(),
        )
        .unwrap();
        write_watch_lock(&project);
        let mut roots = watched_package_roots(&project).unwrap();
        let mut files = Some(watch_snapshot(&roots).unwrap());
        let stopping = AtomicBool::new(false);
        assert!(!roots.contains(&dependency));
        fs::write(&manifest_path, manifest).unwrap();
        let attempts = std::cell::Cell::new(0);
        assert!(
            !rebuild_if_changed(&project, &mut roots, &mut files, &stopping, || {
                attempts.set(attempts.get() + 1);
                if attempts.get() == 1 {
                    assert!(
                        watched_package_roots(&project).is_err(),
                        "the build sees the stale lockfile"
                    );
                    write_watch_lock(&project);
                }
                Ok(false)
            })
            .unwrap()
        );
        assert!(roots.contains(&dependency));
        assert_eq!(
            attempts.get(),
            2,
            "a new graph discovered after the failure must get its own attempt"
        );
        // Consume the newly observed package, still with compiler diagnostics.
        assert!(
            !rebuild_if_changed(&project, &mut roots, &mut files, &stopping, || Ok(false)).unwrap()
        );
        let mut repaired = false;
        assert!(
            !rebuild_if_changed(&project, &mut roots, &mut files, &stopping, || {
                repaired = true;
                Ok(true)
            })
            .unwrap()
        );
        assert!(!repaired);
        fs::write(dependency.join("src/lib.ssrg"), "pub let value: Int = 2\n").unwrap();
        assert!(
            rebuild_if_changed(&project, &mut roots, &mut files, &stopping, || {
                repaired = true;
                Ok(true)
            })
            .unwrap()
        );
        assert!(repaired, "the dependency repair must trigger another build");
    }

    struct Writer<F>(F);

    impl<F: FnMut(&[u8]) -> std::io::Result<usize>> Write for Writer<F> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            (self.0)(bytes)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct Reader<F>(F);

    impl<F: FnMut(&mut [u8]) -> std::io::Result<usize>> Read for Reader<F> {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            (self.0)(bytes)
        }
    }

    #[test]
    fn client_io_retries_transient_errors_and_partial_writes_without_losing_response_bytes() {
        let stopping = AtomicBool::new(false);
        let mut transient = [
            ErrorKind::Interrupted,
            ErrorKind::WouldBlock,
            ErrorKind::TimedOut,
        ]
        .into_iter();
        let mut written = Vec::new();
        let mut client = ClientIo {
            stream: Writer(|bytes: &[u8]| {
                if let Some(error) = transient.next() {
                    return Err(error.into());
                }
                let count = bytes.len().min(3);
                written.extend_from_slice(&bytes[..count]);
                Ok(count)
            }),
            stopping: &stopping,
            deadline: Instant::now() + Duration::from_secs(5),
        };
        response(&mut client, 200, "text/plain", b"bundle", "GET").unwrap();
        assert_eq!(written, b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 6\r\nCache-Control: no-store\r\nConnection: close\r\n\r\nbundle");
    }

    #[test]
    fn client_io_shutdown_interrupts_full_buffers_and_partial_responses() {
        for partial in [false, true] {
            let stopping = AtomicBool::new(false);
            let mut attempts = 0;
            let mut client = ClientIo {
                stream: Writer(|_: &[u8]| {
                    attempts += 1;
                    stopping.store(true, Ordering::SeqCst);
                    if partial {
                        Ok(1)
                    } else {
                        Err(ErrorKind::WouldBlock.into())
                    }
                }),
                stopping: &stopping,
                deadline: Instant::now() + Duration::from_secs(5),
            };
            let error = client.write_all(b"large response").unwrap_err();
            assert_eq!(error.kind(), ErrorKind::ConnectionAborted);
            assert_eq!(attempts, 1);
        }
    }

    #[test]
    fn client_io_deadline_is_not_restarted_by_another_write() {
        let stopping = AtomicBool::new(false);
        let mut client = ClientIo {
            stream: Writer(|_: &[u8]| panic!("an expired response must not write again")),
            stopping: &stopping,
            deadline: Instant::now(),
        };
        for _ in 0..2 {
            assert_eq!(
                client.write_all(b"bundle").unwrap_err().kind(),
                ErrorKind::TimedOut
            );
        }
    }

    #[test]
    fn client_io_preserves_disconnected_peer_errors_without_retrying() {
        let stopping = AtomicBool::new(false);
        let mut attempts = 0;
        let mut client = ClientIo {
            stream: Writer(|_: &[u8]| {
                attempts += 1;
                Err(ErrorKind::ConnectionReset.into())
            }),
            stopping: &stopping,
            deadline: Instant::now() + Duration::from_secs(5),
        };
        assert_eq!(
            client.write_all(b"bundle").unwrap_err().kind(),
            ErrorKind::ConnectionReset
        );
        assert_eq!(attempts, 1);
    }

    #[test]
    fn client_io_shutdown_interrupts_an_incomplete_request_head() {
        let stopping = AtomicBool::new(false);
        let mut attempts = 0;
        let mut reader = Reader(|_: &mut [u8]| {
            attempts += 1;
            stopping.store(true, Ordering::SeqCst);
            Err(ErrorKind::WouldBlock.into())
        });
        assert_eq!(
            read_request_head(&mut reader, &stopping)
                .unwrap_err()
                .kind(),
            ErrorKind::ConnectionAborted
        );
        assert_eq!(attempts, 1);
    }

    #[test]
    fn client_io_reads_fragmented_headers_after_transient_errors() {
        let mut transient = [
            ErrorKind::Interrupted,
            ErrorKind::WouldBlock,
            ErrorKind::TimedOut,
        ]
        .into_iter();
        let mut chunks = [
            b"GET / HTTP/1.1\r\nHo".as_slice(),
            b"st: localhost\r\n\r\n".as_slice(),
        ]
        .into_iter();
        let mut reader = Reader(|bytes: &mut [u8]| {
            if let Some(error) = transient.next() {
                return Err(error.into());
            }
            let chunk = chunks.next().unwrap();
            bytes[..chunk.len()].copy_from_slice(chunk);
            Ok(chunk.len())
        });
        assert_eq!(
            read_request_head(&mut reader, &AtomicBool::new(false))
                .unwrap()
                .as_deref(),
            Some("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        );
    }

    fn queued_connections(count: usize) -> (TcpListener, Vec<TcpStream>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let clients = (0..count)
            .map(|_| TcpStream::connect(listener.local_addr().unwrap()).unwrap())
            .collect();
        (listener, clients)
    }

    #[test]
    fn queued_requests_stop_before_the_next_request_or_change_poll() {
        let (listener, _clients) = queued_connections(2);
        let stopping = AtomicBool::new(false);
        let mut served = 0;
        let poll_changes = serve_pending_requests(&listener, &stopping, |_| {
            served += 1;
            stopping.store(true, Ordering::SeqCst);
        })
        .unwrap();
        assert!(
            !poll_changes,
            "shutdown must not begin another watch/build cycle"
        );
        assert_eq!(served, 1, "queued requests must not delay shutdown");
    }

    #[test]
    fn shutdown_before_accept_leaves_queued_requests_unprocessed() {
        let (listener, _clients) = queued_connections(1);
        let stopping = AtomicBool::new(true);
        let mut served = 0;
        assert!(!serve_pending_requests(&listener, &stopping, |_| served += 1).unwrap());
        assert_eq!(served, 0);
    }

    #[test]
    fn drained_requests_continue_to_the_change_poll_without_shutdown() {
        let (listener, _clients) = queued_connections(2);
        let stopping = AtomicBool::new(false);
        let mut served = 0;
        assert!(serve_pending_requests(&listener, &stopping, |_| served += 1).unwrap());
        assert_eq!(served, 2);
    }

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
                &AtomicBool::new(false),
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
        let index = inject_reload(b"<body>app</body>", 7);
        let index = String::from_utf8(index).unwrap();
        assert!(index.contains(RELOAD_PATH));
        assert!(index.ends_with("</body>"));
        assert!(index.contains("const version = \"7\\n\";"));
        assert!(!index.contains("let version = await"));
        assert!(index.contains("dispatchEvent(closing)"));
        let failed = inject_reload(b"<body>Build failed</body>", 0);
        assert!(String::from_utf8(failed).unwrap().contains(RELOAD_PATH));
    }

    #[test]
    fn watch_file_stamps_do_not_requeue_builds_for_metadata_captured_before_an_edit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("main.ssrg");
        fs::write(&path, "let value = 1\n").unwrap();
        let before_edit = fs::metadata(&path).unwrap();
        let initial = watch_stamp(&path, &before_edit).unwrap();
        let changed = "let value = 42\n";
        fs::write(&path, changed).unwrap();
        assert_ne!(before_edit.len(), changed.len() as u64);
        let observed_during_edit = watch_stamp(&path, &before_edit).unwrap();
        assert_ne!(observed_during_edit, initial);
        let next_poll = watch_snapshot(&[directory.path().to_owned()]).unwrap();
        assert_eq!(observed_during_edit.as_ref(), next_poll.get(&path));

        let file = fs::File::options().write(true).open(&path).unwrap();
        file.set_times(
            fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        )
        .unwrap();
        assert_eq!(
            watch_snapshot(&[directory.path().to_owned()]).unwrap(),
            next_poll
        );
        fs::write(&path, "let value = 43\n").unwrap();
        assert_ne!(
            watch_snapshot(&[directory.path().to_owned()]).unwrap(),
            next_poll
        );
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
        let metadata = enumerated.metadata().unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(watch_stamp(&enumerated.path(), &metadata).unwrap(), None);
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
