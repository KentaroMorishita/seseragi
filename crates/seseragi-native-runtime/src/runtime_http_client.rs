// `std/http.sendBytes` and `sendEmpty` on the native target: one HTTP/1.1
// exchange over a TCP connection on a worker thread. https needs TLS, which
// the native runtime does not carry, so it fails with `HttpTlsFailure`.
// Redirects are not followed and no `Accept-Encoding` is sent, as on the TS
// target. A compressed response body is not decoded natively yet.

const NATIVE_HTTP_CHUNK_BYTES: usize = 64 * 1024;
const NATIVE_HTTP_HEAD_BYTES: usize = 16 * 1024;
const NATIVE_HTTP_INFORMATIONAL_LIMIT: usize = 16;

#[derive(Debug, PartialEq)]
enum NativeHttpReply {
    Response {
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    },
    Failure(&'static str, String),
    LimitExceeded,
    Cancelled,
}

struct NativeHttpTask {
    limit: i64,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    connection: std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<dyn NativeHttpShutdown>>>>,
    completion: std::sync::mpsc::Receiver<NativeHttpReply>,
}

fn native_http_failure(tag: &'static str, message: &str) -> NativeExit {
    NativeExit::failure(V::adt(tag, V::text(message)))
}

// The Content-Length a request declares, checked before anything is sent.
fn native_http_declared(headers: &[(String, String)]) -> Result<Option<i64>, NativeExit> {
    let values: Vec<_> = headers
        .iter()
        .filter(|(name, _)| name == "content-length")
        .collect();
    let Some((_, value)) = values.first() else {
        return Ok(None);
    };
    let digits = values.len() == 1
        && !value.is_empty()
        && value.bytes().all(|b| b.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'));
    if !digits {
        return Err(native_http_failure(
            "std/http::HttpProtocolFailure",
            "invalid Content-Length header",
        ));
    }
    match value.parse::<i64>() {
        Ok(declared) if declared <= 9_007_199_254_740_991 => Ok(Some(declared)),
        _ => Err(native_http_failure(
            "std/http::HttpProtocolFailure",
            "Content-Length exceeds the safe integer range",
        )),
    }
}

fn native_http_length_mismatch(declared: i64, actual: i64) -> V {
    V::adt(
        "std/http::HttpRequestLengthMismatch",
        native_http_record(vec![
            ("declared", V::Int(declared)),
            ("actual", V::Int(actual)),
        ]),
    )
}

// The Content-Length a request declares must match its body, as on TS.
fn native_http_declared_length(
    headers: &[(String, String)],
    actual: usize,
) -> Result<(), NativeExit> {
    match native_http_declared(headers)? {
        Some(declared) if declared != actual as i64 => Err(NativeExit::failure(
            native_http_length_mismatch(declared, actual as i64),
        )),
        _ => Ok(()),
    }
}

// A request body: whole bytes for `sendBytes` and `sendEmpty`, or the parts
// an exchange's body Fiber hands over one at a time.
enum NativeHttpBody {
    Bytes(Vec<u8>),
    Parts {
        parts: std::sync::mpsc::Receiver<NativeHttpPart>,
        // The length a body knows in advance; None sends it chunked.
        length: Option<u64>,
        // Woken after each part is taken, so a waiting hand-over retries.
        runtime: std::thread::Thread,
    },
}

enum NativeHttpPart {
    Chunk(Vec<u8>),
    End,
}

// Writes the parts of a body as they arrive, in chunked framing when its
// length is unknown. Returns once the body ends, a write fails, or the
// exchange stops handing parts over.
fn native_http_write_parts(
    writer: &mut impl std::io::Write,
    parts: &std::sync::mpsc::Receiver<NativeHttpPart>,
    chunked: bool,
    runtime: &std::thread::Thread,
) -> std::io::Result<bool> {
    loop {
        let part = parts.recv();
        runtime.unpark();
        match part {
            Ok(NativeHttpPart::Chunk(bytes)) if chunked => {
                for piece in bytes.chunks(NATIVE_HTTP_CHUNK_BYTES) {
                    let mut frame = format!("{:x}\r\n", piece.len()).into_bytes();
                    frame.extend_from_slice(piece);
                    frame.extend_from_slice(b"\r\n");
                    writer.write_all(&frame)?;
                }
            }
            Ok(NativeHttpPart::Chunk(bytes)) => writer.write_all(&bytes)?,
            Ok(NativeHttpPart::End) => {
                if chunked {
                    writer.write_all(b"0\r\n\r\n")?;
                }
                return Ok(true);
            }
            Err(_) => return Ok(false),
        }
    }
}

impl NativeHttpTask {
    // Captures: the body limit, the request body and the Request.
    fn start(captures: Vec<V>) -> Result<Self, NativeExit> {
        let limit = native_http_int(native_http_inner(&captures[0], "std/http::HttpBodyLimit"));
        let body = native_bytes(&captures[1]).to_vec();
        let request = native_http_inner(&captures[2], "std/http::Request");
        let method = native_text(native_http_inner(
            &request.field("method"),
            "std/http::Method",
        ))
        .to_owned();
        let url = native_text(native_http_inner(
            &request.field("url"),
            "std/http::HttpUrl",
        ))
        .to_owned();
        let headers = native_http_headers(&request.field("headers"));
        native_http_declared_length(&headers, body.len())?;
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let connection = std::sync::Arc::new(std::sync::Mutex::new(None));
        let (send, completion) = std::sync::mpsc::channel();
        let runtime = std::thread::current();
        let (worker_cancelled, worker_connection) = (cancelled.clone(), connection.clone());
        std::thread::Builder::new()
            .name("seseragi-http".into())
            .spawn(move || {
                let reply = native_http_exchange(
                    &method,
                    &url,
                    &headers,
                    NativeHttpBody::Bytes(body),
                    limit as u64,
                    &worker_cancelled,
                    &worker_connection,
                );
                let _ = send.send(reply);
                runtime.unpark();
            })
            .map_err(|error| {
                NativeExit::defects(vec![format!("cannot start HTTP worker: {error}")])
            })?;
        Ok(Self {
            limit,
            cancelled,
            connection,
            completion,
        })
    }

    fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
        native_http_shutdown_both(&self.connection);
    }

    fn poll(&mut self) -> Option<NativeOutcome> {
        let reply = match self.completion.try_recv() {
            Ok(reply) => reply,
            Err(std::sync::mpsc::TryRecvError::Empty) => return None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                return Some(Err(NativeExit::defects(vec![
                    "HTTP worker returned no result".into(),
                ])))
            }
        };
        if self.cancelled.load(std::sync::atomic::Ordering::Acquire) {
            return Some(Err(NativeExit::Cancelled));
        }
        Some(match reply {
            NativeHttpReply::Response {
                status,
                headers,
                body,
            } => Ok(native_http_value(
                "std/http::Response",
                native_http_record(vec![
                    (
                        "status",
                        native_http_value("std/http::Status", V::Int(status as i64)),
                    ),
                    ("headers", native_http_headers_value(headers)),
                    ("body", V::Bytes(Shared::new(body))),
                ]),
            )),
            NativeHttpReply::Failure(tag, message) => Err(native_http_failure(tag, &message)),
            NativeHttpReply::LimitExceeded => Err(NativeExit::failure(V::adt(
                "std/http::HttpResponseBodyLimitExceeded",
                native_http_record(vec![("limitBytes", V::Int(self.limit))]),
            ))),
            NativeHttpReply::Cancelled => Err(NativeExit::Cancelled),
        })
    }
}

impl Drop for NativeHttpTask {
    fn drop(&mut self) {
        // A dropped exchange closes its connection; the worker then ends.
        self.cancel();
    }
}

pub struct NativeHttpTarget {
    pub https: bool,
    pub host: String,
    pub port: u16,
    pub authority: String,
    pub path: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NativeHttpConnectError {
    Dns(String),
    Tls(String),
    Protocol(String),
    Connection(String),
    Cancelled,
}

impl From<NativeHttpConnectError> for NativeHttpReply {
    fn from(error: NativeHttpConnectError) -> Self {
        match error {
            NativeHttpConnectError::Dns(message) => {
                NativeHttpReply::Failure("std/http::HttpDnsFailure", message)
            }
            NativeHttpConnectError::Tls(message) => {
                NativeHttpReply::Failure("std/http::HttpTlsFailure", message)
            }
            NativeHttpConnectError::Protocol(message) => {
                NativeHttpReply::Failure("std/http::HttpProtocolFailure", message)
            }
            NativeHttpConnectError::Connection(message) => {
                NativeHttpReply::Failure("std/http::HttpConnectionFailure", message)
            }
            NativeHttpConnectError::Cancelled => NativeHttpReply::Cancelled,
        }
    }
}

pub trait NativeHttpShutdown: Send + Sync {
    fn shutdown_write(&self) -> std::io::Result<()>;
    fn shutdown_both(&self) -> std::io::Result<()>;
}

impl NativeHttpShutdown for std::net::TcpStream {
    fn shutdown_write(&self) -> std::io::Result<()> {
        self.shutdown(std::net::Shutdown::Write)
    }

    fn shutdown_both(&self) -> std::io::Result<()> {
        self.shutdown(std::net::Shutdown::Both)
    }
}

pub struct NativeHttpTransport {
    pub reader: Box<dyn std::io::Read + Send>,
    pub writer: Box<dyn std::io::Write + Send>,
    pub shutdown: std::sync::Arc<dyn NativeHttpShutdown>,
}

pub trait NativeHttpConnector: Send + Sync {
    fn connect(
        &self,
        target: &NativeHttpTarget,
        cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<NativeHttpTransport, NativeHttpConnectError>;
}

struct NativePlainHttpConnector;

static NATIVE_HTTP_CONNECTOR: std::sync::OnceLock<std::sync::Arc<dyn NativeHttpConnector>> =
    std::sync::OnceLock::new();

pub fn native_http_register_connector(
    connector: std::sync::Arc<dyn NativeHttpConnector>,
) -> Result<(), &'static str> {
    NATIVE_HTTP_CONNECTOR
        .set(connector)
        .map_err(|_| "HTTP connector is already registered")
}

fn native_http_active_connector() -> &'static dyn NativeHttpConnector {
    match NATIVE_HTTP_CONNECTOR.get() {
        Some(connector) => connector.as_ref(),
        None => &NativePlainHttpConnector,
    }
}

fn native_http_target(url: &str) -> Result<NativeHttpTarget, NativeHttpReply> {
    let (https, default_port, rest) = if let Some(rest) = url.strip_prefix("https://") {
        (true, 443, rest)
    } else if let Some(rest) = url.strip_prefix("http://") {
        (false, 80, rest)
    } else {
        return Err(NativeHttpReply::Failure(
            "std/http::HttpProtocolFailure",
            "invalid HTTP URL".into(),
        ));
    };
    // An empty fragment can remain in a parsed URL; it is never sent.
    let rest = rest.split('#').next().unwrap_or("");
    let end = rest.find(['/', '?']).unwrap_or(rest.len());
    let (authority, path) = rest.split_at(end);
    let (host, port) = if let Some(close) = authority.find(']') {
        let port = authority[close + 1..].strip_prefix(':');
        (&authority[1..close], port)
    } else {
        match authority.rfind(':') {
            Some(colon) => (&authority[..colon], Some(&authority[colon + 1..])),
            None => (authority, None),
        }
    };
    Ok(NativeHttpTarget {
        https,
        host: host.into(),
        port: port
            .and_then(|port| port.parse().ok())
            .unwrap_or(default_port),
        authority: authority.into(),
        path: if path.is_empty() {
            "/".into()
        } else if path.starts_with('?') {
            format!("/{path}")
        } else {
            path.into()
        },
    })
}

pub fn native_http_connect_tcp(
    target: &NativeHttpTarget,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<std::net::TcpStream, NativeHttpConnectError> {
    let is_cancelled = || cancelled.load(std::sync::atomic::Ordering::Acquire);
    let (connected, connecting) = std::sync::mpsc::channel();
    let (host, port) = (target.host.clone(), target.port);
    let connector_cancelled = cancelled.clone();
    std::thread::spawn(move || {
        // After cancellation no further lookup or attempt starts. An attempt
        // already blocked in the OS ends at the OS connect timeout, and its
        // connection, if any, is dropped unused.
        let stopped = || connector_cancelled.load(std::sync::atomic::Ordering::Acquire);
        if stopped() {
            return;
        }
        // Name resolution also blocks in the OS, so it runs here too.
        let addresses = match std::net::ToSocketAddrs::to_socket_addrs(&(host.as_str(), port)) {
            Ok(addresses) => addresses.collect::<Vec<_>>(),
            Err(error) => {
                let _ = connected.send(Err(NativeHttpConnectError::Dns(error.to_string())));
                return;
            }
        };
        if addresses.is_empty() {
            let _ = connected.send(Err(NativeHttpConnectError::Dns(format!(
                "no address for {host}"
            ))));
            return;
        }
        let mut last = String::new();
        for address in addresses {
            if stopped() {
                return;
            }
            match std::net::TcpStream::connect(address) {
                Ok(stream) => {
                    if stopped() {
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                        return;
                    }
                    let _ = connected.send(Ok(stream));
                    return;
                }
                Err(error) => last = error.to_string(),
            }
        }
        let _ = connected.send(Err(NativeHttpConnectError::Connection(last)));
    });
    loop {
        if is_cancelled() {
            return Err(NativeHttpConnectError::Cancelled);
        }
        match connecting.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(Ok(stream)) => return Ok(stream),
            Ok(Err(error)) => return Err(error),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(NativeHttpConnectError::Connection(
                    "connection attempt ended without a result".into(),
                ))
            }
        }
    }
}

impl NativeHttpConnector for NativePlainHttpConnector {
    fn connect(
        &self,
        target: &NativeHttpTarget,
        cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<NativeHttpTransport, NativeHttpConnectError> {
        if target.https {
            return Err(NativeHttpConnectError::Tls(
                "https needs TLS, which the native target does not support yet".into(),
            ));
        }
        let stream = native_http_connect_tcp(target, cancelled)?;
        let Ok(writer) = stream.try_clone() else {
            return Err(NativeHttpConnectError::Connection(
                "cannot share the connection".into(),
            ));
        };
        let Ok(shutdown) = stream.try_clone() else {
            return Err(NativeHttpConnectError::Connection(
                "cannot share the connection".into(),
            ));
        };
        Ok(NativeHttpTransport {
            reader: Box::new(stream),
            writer: Box::new(writer),
            shutdown: std::sync::Arc::new(shutdown),
        })
    }
}

// Connects and starts writing one request. The returned flag turns true once
// the whole request has been written.
type NativeHttpOpened = (
    std::io::BufReader<Box<dyn std::io::Read + Send>>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
);

fn native_http_open(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: NativeHttpBody,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    connection: &std::sync::Mutex<Option<std::sync::Arc<dyn NativeHttpShutdown>>>,
) -> Result<NativeHttpOpened, NativeHttpReply> {
    native_http_open_with(
        native_http_active_connector(),
        method,
        url,
        headers,
        body,
        cancelled,
        connection,
    )
}

fn native_http_open_with(
    connector: &dyn NativeHttpConnector,
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: NativeHttpBody,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    connection: &std::sync::Mutex<Option<std::sync::Arc<dyn NativeHttpShutdown>>>,
) -> Result<NativeHttpOpened, NativeHttpReply> {
    use std::io::Write;
    let target = native_http_target(url)?;
    let is_cancelled = || cancelled.load(std::sync::atomic::Ordering::Acquire);
    let NativeHttpTransport {
        reader,
        mut writer,
        shutdown,
    } = connector.connect(&target, cancelled)?;
    *connection.lock().unwrap() = Some(shutdown);
    if is_cancelled() {
        native_http_shutdown_both(connection);
        return Err(NativeHttpReply::Cancelled);
    }
    // Header values are Latin-1, as the TS hosts write them.
    let mut head = format!(
        "{method} {} HTTP/1.1\r\nhost: {}\r\n",
        target.path, target.authority
    )
    .into_bytes();
    for (name, value) in headers {
        head.extend(name.bytes());
        head.extend(b": ");
        head.extend(value.chars().map(|c| c as u32 as u8));
        head.extend(b"\r\n");
    }
    let declared = headers.iter().any(|(name, _)| name == "content-length");
    // A body of known length declares it, as the TS HTTP/1 writer does; any
    // other body is sent chunked.
    let mut chunked = false;
    match &body {
        NativeHttpBody::Bytes(bytes) => {
            if !declared && (!bytes.is_empty() || matches!(method, "POST" | "PUT" | "PATCH")) {
                head.extend(format!("content-length: {}\r\n", bytes.len()).bytes());
            }
        }
        NativeHttpBody::Parts { length, .. } => {
            if !declared {
                match length {
                    Some(length) => head.extend(format!("content-length: {length}\r\n").bytes()),
                    None => {
                        chunked = true;
                        head.extend(b"transfer-encoding: chunked\r\n");
                    }
                }
            }
        }
    }
    head.extend(b"connection: close\r\n\r\n");
    // The request is written on its own thread, so a final response that
    // arrives before the body is fully sent (such as 413) is still read.
    // Closing the connection after the response stops the writer.
    let written = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer_done = written.clone();
    std::thread::spawn(move || {
        let finished = writer.write_all(&head).and_then(|_| match &body {
            NativeHttpBody::Bytes(bytes) => writer.write_all(bytes).map(|_| true),
            NativeHttpBody::Parts { parts, runtime, .. } => {
                native_http_write_parts(&mut writer, parts, chunked, runtime)
            }
        });
        if matches!(finished, Ok(true)) {
            writer_done.store(true, std::sync::atomic::Ordering::Release);
        }
    });
    Ok((
        std::io::BufReader::with_capacity(NATIVE_HTTP_CHUNK_BYTES, reader),
        written,
    ))
}

// `sendBytes` and `sendEmpty`: one request and its whole response.
fn native_http_exchange(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: NativeHttpBody,
    limit: u64,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    connection: &std::sync::Mutex<Option<std::sync::Arc<dyn NativeHttpShutdown>>>,
) -> NativeHttpReply {
    native_http_exchange_with(
        native_http_active_connector(),
        method,
        url,
        headers,
        body,
        limit,
        cancelled,
        connection,
    )
}

#[allow(clippy::too_many_arguments)]
fn native_http_exchange_with(
    connector: &dyn NativeHttpConnector,
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: NativeHttpBody,
    limit: u64,
    cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    connection: &std::sync::Mutex<Option<std::sync::Arc<dyn NativeHttpShutdown>>>,
) -> NativeHttpReply {
    let (mut reader, written) =
        match native_http_open_with(connector, method, url, headers, body, cancelled, connection) {
            Ok(opened) => opened,
            Err(reply) => return reply,
        };
    let is_cancelled = || cancelled.load(std::sync::atomic::Ordering::Acquire);
    let failure = |error: std::io::Error, tag: &'static str| {
        if is_cancelled() {
            NativeHttpReply::Cancelled
        } else {
            NativeHttpReply::Failure(tag, error.to_string())
        }
    };
    // A final response head stops a request body still being sent, as the TS
    // HTTP/1 writer does; the response is still read to its end. A finished
    // request is left alone, since some servers close on a half-closed
    // connection.
    let mut stop_writer = || {
        if !written.load(std::sync::atomic::Ordering::Acquire) {
            let _ = native_http_stop_writing(connection);
        }
    };
    let response =
        native_http_read_response(&mut reader, method == "HEAD", limit, &mut stop_writer);
    match response {
        Err(NativeHttpRead::Io(error)) => failure(error, "std/http::HttpConnectionFailure"),
        Err(NativeHttpRead::Protocol(message)) => {
            if is_cancelled() {
                NativeHttpReply::Cancelled
            } else {
                NativeHttpReply::Failure("std/http::HttpProtocolFailure", message)
            }
        }
        Err(NativeHttpRead::Limit) => NativeHttpReply::LimitExceeded,
        Ok((status, headers, body)) => {
            native_http_shutdown_both(connection);
            NativeHttpReply::Response {
                status,
                headers,
                body,
            }
        }
    }
}

enum NativeHttpRead {
    Io(std::io::Error),
    Protocol(String),
    Limit,
}
impl From<std::io::Error> for NativeHttpRead {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

fn native_http_protocol<T>(message: &str) -> Result<T, NativeHttpRead> {
    Err(NativeHttpRead::Protocol(message.into()))
}

fn native_http_read_line(
    reader: &mut impl std::io::BufRead,
    budget: &mut usize,
) -> Result<Vec<u8>, NativeHttpRead> {
    use std::io::BufRead;
    let mut line = Vec::new();
    let read =
        std::io::Read::take(&mut *reader, *budget as u64 + 1).read_until(b'\n', &mut line)?;
    if read == 0 {
        return native_http_protocol("connection closed before the response head ended");
    }
    if read > *budget {
        return native_http_protocol("HTTP response head is too large");
    }
    *budget -= read;
    if line.last() != Some(&b'\n') {
        return native_http_protocol("connection closed before the response head ended");
    }
    line.pop();
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    Ok(line)
}

// The minor HTTP/1 version, the status and the headers of one response head.
type NativeHttpHead = (u8, u16, Vec<(String, String)>);

fn native_http_field_text(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .all(|b| *b == b'\t' || (0x20..=0x7e).contains(b) || *b >= 0x80)
}

// A header or trailer line, checked as the TS HTTP/1 reader does.
fn native_http_header_line(line: &[u8]) -> Result<(String, String), NativeHttpRead> {
    let colon = match line.iter().position(|b| *b == b':') {
        Some(colon) if colon > 0 => colon,
        _ => return native_http_protocol("invalid HTTP header"),
    };
    let name = String::from_utf8_lossy(&line[..colon]).to_ascii_lowercase();
    if !native_http_token(&name) || !native_http_field_text(&line[colon + 1..]) {
        return native_http_protocol("invalid HTTP header");
    }
    // Header bytes are Latin-1, as the TS hosts read them.
    let value: String = line[colon + 1..].iter().map(|b| *b as char).collect();
    Ok((name, value.trim_matches([' ', '\t']).to_owned()))
}

fn native_http_read_head(
    reader: &mut impl std::io::BufRead,
) -> Result<NativeHttpHead, NativeHttpRead> {
    let mut budget = NATIVE_HTTP_HEAD_BYTES;
    let line = native_http_read_line(reader, &mut budget)?;
    // `HTTP/1.x NNN` with an optional reason, as the TS HTTP/1 reader takes.
    let valid_text = native_http_field_text;
    let (minor, status) = match line.as_slice() {
        [b'H', b'T', b'T', b'P', b'/', b'1', b'.', minor @ (b'0' | b'1'), b' ', a, b, c, rest @ ..]
            if [a, b, c].iter().all(|d| d.is_ascii_digit())
                && (rest.is_empty() || (rest[0] == b' ' && valid_text(&rest[1..]))) =>
        {
            (
                *minor - b'0',
                (*a - b'0') as u16 * 100 + (*b - b'0') as u16 * 10 + (*c - b'0') as u16,
            )
        }
        _ => return native_http_protocol("invalid HTTP status line"),
    };
    if status < 100 {
        return native_http_protocol("invalid HTTP response status");
    }
    let mut headers = Vec::new();
    loop {
        let line = native_http_read_line(reader, &mut budget)?;
        if line.is_empty() {
            break;
        }
        headers.push(native_http_header_line(&line)?);
    }
    Ok((minor, status, headers))
}

// Transfer codings in order, skipping quoted commas and parameters, as the
// TS HTTP/1 reader's `transferCodings` does.
fn native_http_transfer_codings(value: &str) -> Result<Vec<String>, NativeHttpRead> {
    let mut codings = Vec::new();
    let (mut start, mut quoted, mut escaped) = (0, false, false);
    let mut append = |end: usize, start: &mut usize| {
        let coding = value[*start..end].split(';').next().unwrap_or("");
        codings.push(coding.trim().to_ascii_lowercase());
        *start = end + 1;
    };
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
        } else if quoted && character == '\\' {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if character == ',' && !quoted {
            append(index, &mut start);
        }
    }
    if quoted {
        return native_http_protocol("invalid HTTP Transfer-Encoding parameter");
    }
    append(value.len(), &mut start);
    Ok(codings)
}

// Framing headers are checked on every head, as on the TS HTTP/1 reader.
fn native_http_framing(headers: &[(String, String)]) -> Result<(), NativeHttpRead> {
    let transfer: Vec<_> = headers
        .iter()
        .filter(|(name, _)| name == "transfer-encoding")
        .collect();
    let lengths: Vec<_> = headers
        .iter()
        .filter(|(name, _)| name == "content-length")
        .collect();
    if !transfer.is_empty() && !lengths.is_empty() {
        return native_http_protocol(
            "HTTP response contains both Transfer-Encoding and Content-Length",
        );
    }
    if let Some((_, value)) = lengths.first() {
        let digits = !value.is_empty()
            && value.bytes().all(|b| b.is_ascii_digit())
            && (value == "0" || !value.starts_with('0'));
        if lengths.len() != 1
            || !digits
            || value
                .parse::<u64>()
                .map_or(true, |length| length > 9_007_199_254_740_991)
        {
            return native_http_protocol("invalid response Content-Length");
        }
    }
    let mut chunked = 0;
    for (_, value) in &transfer {
        chunked += native_http_transfer_codings(value)?
            .iter()
            .filter(|coding| *coding == "chunked")
            .count();
    }
    if chunked > 1 {
        return native_http_protocol(
            "HTTP response applies chunked transfer coding more than once",
        );
    }
    Ok(())
}

fn native_http_append(body: &mut Vec<u8>, chunk: &[u8], limit: u64) -> Result<(), NativeHttpRead> {
    // The limit is checked before a chunk is retained.
    if chunk.len() as u64 > limit - body.len() as u64 {
        return Err(NativeHttpRead::Limit);
    }
    body.extend_from_slice(chunk);
    Ok(())
}

fn native_http_stop_writing(
    connection: &std::sync::Mutex<Option<std::sync::Arc<dyn NativeHttpShutdown>>>,
) -> std::io::Result<()> {
    match connection.lock().unwrap().as_ref() {
        Some(handle) => handle.shutdown_write(),
        None => Ok(()),
    }
}

fn native_http_shutdown_both(
    connection: &std::sync::Mutex<Option<std::sync::Arc<dyn NativeHttpShutdown>>>,
) {
    if let Some(handle) = connection.lock().unwrap().as_ref() {
        let _ = handle.shutdown_both();
    }
}

// One step of a response, as `std/http.exchange` reports it.
#[derive(Debug, PartialEq)]
enum NativeHttpEvent {
    Head {
        informational: bool,
        minor: u8,
        status: u16,
        headers: Vec<(String, String)>,
    },
    Chunk(Vec<u8>),
    Trailers(Vec<(String, String)>),
}

// Reads one response and reports each head, body chunk (at most 64 KiB) and
// non-empty trailer block in order, as the TS HTTP/1 reader does.
fn native_http_read_events(
    reader: &mut impl std::io::BufRead,
    head_request: bool,
    on_final_head: &mut dyn FnMut(),
    emit: &mut dyn FnMut(NativeHttpEvent) -> Result<(), NativeHttpRead>,
) -> Result<(), NativeHttpRead> {
    let mut informational = 0;
    let headers = loop {
        let (minor, status, headers) = native_http_read_head(reader)?;
        native_http_framing(&headers)?;
        let final_head = !(100..200).contains(&status);
        if final_head {
            on_final_head();
        } else {
            informational += 1;
            if informational > NATIVE_HTTP_INFORMATIONAL_LIMIT {
                return native_http_protocol("HTTP informational response limit exceeded");
            }
        }
        emit(NativeHttpEvent::Head {
            informational: !final_head,
            minor,
            status,
            headers: headers.clone(),
        })?;
        if final_head {
            if head_request || matches!(status, 204 | 304) {
                return Ok(());
            }
            break headers;
        }
    };
    let values = |name: &str| -> Vec<String> {
        headers
            .iter()
            .filter(|(header, _)| header == name)
            .map(|(_, value)| value.clone())
            .collect()
    };
    let mut codings = Vec::new();
    for value in values("transfer-encoding") {
        codings.extend(
            native_http_transfer_codings(&value)?
                .into_iter()
                .filter(|coding| !coding.is_empty()),
        );
    }
    let mut buffer = vec![0u8; NATIVE_HTTP_CHUNK_BYTES];
    let chunked = codings.last().map(String::as_str) == Some("chunked");
    if chunked {
        loop {
            let mut budget = NATIVE_HTTP_HEAD_BYTES;
            let line = native_http_read_line(reader, &mut budget)?;
            let size = line.split(|b| *b == b';').next().unwrap_or(&[]);
            if size.is_empty() || !size.iter().all(u8::is_ascii_hexdigit) {
                return native_http_protocol("invalid chunk size");
            }
            let Ok(mut remaining) = usize::from_str_radix(std::str::from_utf8(size).unwrap(), 16)
            else {
                return native_http_protocol("chunk size is too large");
            };
            if remaining == 0 {
                // Trailers are checked like headers and end with an empty line.
                let mut trailers = Vec::new();
                loop {
                    let line = native_http_read_line(reader, &mut budget)?;
                    if line.is_empty() {
                        break;
                    }
                    trailers.push(native_http_header_line(&line)?);
                }
                if !trailers.is_empty() {
                    emit(NativeHttpEvent::Trailers(trailers))?;
                }
                return Ok(());
            }
            while remaining > 0 {
                let want = remaining.min(buffer.len());
                let read = reader.read(&mut buffer[..want])?;
                if read == 0 {
                    return native_http_protocol("connection closed inside a chunk");
                }
                emit(NativeHttpEvent::Chunk(buffer[..read].to_vec()))?;
                remaining -= read;
            }
            let mut budget = 2;
            if !native_http_read_line(reader, &mut budget)?.is_empty() {
                return native_http_protocol("missing chunk terminator");
            }
        }
    } else if let (true, Some(length)) = (codings.is_empty(), values("content-length").first()) {
        let mut remaining: u64 = length.parse().unwrap();
        while remaining > 0 {
            let want = (remaining as usize).min(buffer.len());
            let read = reader.read(&mut buffer[..want])?;
            if read == 0 {
                return native_http_protocol("connection closed before the body ended");
            }
            emit(NativeHttpEvent::Chunk(buffer[..read].to_vec()))?;
            remaining -= read as u64;
        }
    } else {
        // No length, or a final transfer coding other than chunked: the body
        // runs until the connection closes.
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            emit(NativeHttpEvent::Chunk(buffer[..read].to_vec()))?;
        }
    }
    Ok(())
}

// `sendBytes` and `sendEmpty`: the events of one exchange collected into a
// response, with the body limit checked before each chunk is kept.
fn native_http_read_response(
    reader: &mut impl std::io::BufRead,
    head_request: bool,
    limit: u64,
    on_final_head: &mut dyn FnMut(),
) -> Result<(u16, Vec<(String, String)>, Vec<u8>), NativeHttpRead> {
    let mut head = None;
    let mut body = Vec::new();
    native_http_read_events(reader, head_request, on_final_head, &mut |event| {
        match event {
            NativeHttpEvent::Head {
                informational: false,
                status,
                headers,
                ..
            } => head = Some((status, headers)),
            NativeHttpEvent::Chunk(chunk) => native_http_append(&mut body, &chunk, limit)?,
            _ => {}
        }
        Ok(())
    })?;
    let Some((status, headers)) = head else {
        return native_http_protocol("HTTP response has no final head");
    };
    let encoded = headers
        .iter()
        .filter(|(name, _)| name == "content-encoding")
        .flat_map(|(_, value)| value.split(','))
        .map(|coding| coding.trim().to_ascii_lowercase())
        .filter(|coding| !coding.is_empty())
        .collect::<Vec<_>>();
    // The TS hosts decode these codings; the native runtime cannot yet, so it
    // refuses rather than return different bytes. Unknown codings pass as is.
    if !body.is_empty()
        && !encoded.is_empty()
        && encoded.iter().all(|coding| {
            matches!(
                coding.as_str(),
                "gzip" | "x-gzip" | "deflate" | "x-deflate" | "br"
            )
        })
    {
        return native_http_protocol(
            "compressed response bodies are not decoded on the native target",
        );
    }
    Ok((status, headers, body))
}

#[cfg(test)]
mod http_client_tests {
    use super::*;

    fn read(
        raw: &[u8],
        head: bool,
        limit: u64,
    ) -> Result<(u16, Vec<(String, String)>, Vec<u8>), String> {
        native_http_read_response(&mut std::io::BufReader::new(raw), head, limit, &mut || {})
            .map_err(|error| match error {
                NativeHttpRead::Io(error) => format!("io {error}"),
                NativeHttpRead::Protocol(message) => format!("protocol {message}"),
                NativeHttpRead::Limit => "limit".into(),
            })
    }

    #[test]
    fn responses_read_lengths_chunks_and_informational_heads() {
        let (status, headers, body) = read(
            b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nX-A: 1\r\nContent-Length: 3\r\n\r\nabcdef",
            false,
            10,
        )
        .unwrap();
        assert_eq!((status, body.as_slice()), (200, &b"abc"[..]));
        assert_eq!(headers[0], ("x-a".into(), "1".into()));
        let (_, _, body) = read(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n2;x=y\r\nde\r\n0\r\nT: 1\r\n\r\n",
            false,
            10,
        )
        .unwrap();
        assert_eq!(body, b"abcde");
        let (_, _, body) = read(b"HTTP/1.0 200 OK\r\n\r\nto the end", false, 100).unwrap();
        assert_eq!(body, b"to the end");
        let (_, _, body) = read(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\n", true, 1).unwrap();
        assert!(body.is_empty());
    }

    #[test]
    fn limits_and_malformed_responses_fail_before_the_body_is_kept() {
        assert_eq!(
            read(
                b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nabcd",
                false,
                3
            )
            .unwrap_err(),
            "limit"
        );
        for raw in [
            &b"HTTP/2 200 OK\r\n\r\n"[..],
            b"HTTP/1.1 20 OK\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nbad header\r\n\r\n",
            b"HTTP/1.1 200 OK\r\n folded\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nab",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: 1\r\n\r\nx",
            b"HTTP/1.1 101 Switching\r\n\r\n",
            b"HTTP/1.1 200",
        ] {
            assert!(
                read(raw, false, 100).unwrap_err().starts_with("protocol"),
                "{raw:?}"
            );
        }
        let informational = b"HTTP/1.1 103 Early Hints\r\n\r\n".repeat(17);
        let mut raw = informational.clone();
        raw.extend_from_slice(b"HTTP/1.1 200 OK\r\n\r\n");
        assert!(read(&raw, false, 100)
            .unwrap_err()
            .contains("informational"));
        let mut raw = informational[..informational.len() / 17 * 16].to_vec();
        raw.extend_from_slice(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
        assert_eq!(read(&raw, false, 100).unwrap().0, 200);
        for raw in [
            &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 1\r\n\r\n"[..],
            b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\nx",
            b"HTTP/1.1 200 OK\r\nContent-Length: 9007199254740992\r\n\r\n",
            b"HTTP/1.1 204 OK\r\nContent-Length: x\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nX-A: a\x00b\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked, chunked\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: x;note=\"open, chunked\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nBad Trailer\r\n\r\n",
        ] {
            assert!(
                read(raw, false, 100).unwrap_err().starts_with("protocol"),
                "{raw:?}"
            );
        }
        let (_, _, body) = read(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\nraw",
            false,
            100,
        )
        .unwrap();
        assert_eq!(body, b"raw");
        let (_, _, body) = read(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: x;note=\"a, chunked, b\", chunked\r\n\r\n2\r\nok\r\n0\r\n\r\n",
            false,
            100,
        )
        .unwrap();
        assert_eq!(body, b"ok");
        let (_, _, body) = read(
            b"HTTP/1.1 200 OK\r\nContent-Encoding: zstd\r\n\r\nx",
            false,
            100,
        )
        .unwrap();
        assert_eq!(body, b"x");
    }

    #[test]
    fn declared_lengths_match_the_ts_checks() {
        let header = |value: &str| vec![("content-length".to_string(), value.to_string())];
        assert!(native_http_declared_length(&[], 3).is_ok());
        assert!(native_http_declared_length(&header("3"), 3).is_ok());
        for (value, tag) in [
            ("2", "HttpRequestLengthMismatch"),
            ("03", "HttpProtocolFailure"),
            ("", "HttpProtocolFailure"),
            ("9007199254740992", "HttpProtocolFailure"),
        ] {
            let Err(NativeExit::Failure(error)) = native_http_declared_length(&header(value), 3)
            else {
                panic!("{value}")
            };
            assert_eq!(native_error_tag(&error.value), tag, "{value}");
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn targets_keep_the_authority_and_plain_connector_refuses_https() {
        let target = native_http_target("http://[::1]:8080?q").ok().unwrap();
        assert_eq!(
            (
                target.https,
                target.host.as_str(),
                target.port,
                target.path.as_str()
            ),
            (false, "::1", 8080, "/?q")
        );
        assert_eq!(target.authority, "[::1]:8080");
        let target = native_http_target("http://a/p?#").ok().unwrap();
        assert_eq!(target.path, "/p?");
        let https_target = native_http_target("https://example.com/v1?x=1")
            .ok()
            .unwrap();
        assert_eq!(
            (
                https_target.https,
                https_target.host.as_str(),
                https_target.port,
                https_target.path.as_str()
            ),
            (true, "example.com", 443, "/v1?x=1")
        );
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        assert!(matches!(
            NativePlainHttpConnector.connect(&https_target, &cancelled),
            Err(NativeHttpConnectError::Tls(_))
        ));
    }

    #[test]
    fn custom_connector_runs_https_exchange_and_invokes_shutdown() {
        struct RecordingShutdown {
            writes: std::sync::atomic::AtomicUsize,
            both: std::sync::atomic::AtomicUsize,
        }
        impl NativeHttpShutdown for RecordingShutdown {
            fn shutdown_write(&self) -> std::io::Result<()> {
                self.writes
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            fn shutdown_both(&self) -> std::io::Result<()> {
                self.both.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }
        struct SharedSink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for SharedSink {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        struct MemoryConnector {
            response: Vec<u8>,
            sent: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
            shutdown: std::sync::Arc<RecordingShutdown>,
            seen_https: std::sync::Arc<std::sync::atomic::AtomicBool>,
        }
        impl NativeHttpConnector for MemoryConnector {
            fn connect(
                &self,
                target: &NativeHttpTarget,
                _cancelled: &std::sync::Arc<std::sync::atomic::AtomicBool>,
            ) -> Result<NativeHttpTransport, NativeHttpConnectError> {
                self.seen_https
                    .store(target.https, std::sync::atomic::Ordering::SeqCst);
                Ok(NativeHttpTransport {
                    reader: Box::new(std::io::Cursor::new(self.response.clone())),
                    writer: Box::new(SharedSink(self.sent.clone())),
                    shutdown: self.shutdown.clone(),
                })
            }
        }

        let sent = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let shutdown = std::sync::Arc::new(RecordingShutdown {
            writes: std::sync::atomic::AtomicUsize::new(0),
            both: std::sync::atomic::AtomicUsize::new(0),
        });
        let seen_https = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let connector = MemoryConnector {
            response: b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello".to_vec(),
            sent: sent.clone(),
            shutdown: shutdown.clone(),
            seen_https: seen_https.clone(),
        };
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let connection = std::sync::Mutex::new(None);
        let reply = native_http_exchange_with(
            &connector,
            "GET",
            "https://example.com/hello",
            &[("x-test".into(), "1".into())],
            NativeHttpBody::Bytes(Vec::new()),
            16,
            &cancelled,
            &connection,
        );
        assert!(seen_https.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(
            reply,
            NativeHttpReply::Response {
                status: 200,
                headers: vec![("content-length".into(), "5".into())],
                body: b"hello".to_vec(),
            }
        );
        assert!(shutdown.both.load(std::sync::atomic::Ordering::SeqCst) >= 1);

        let limit_reply = native_http_exchange_with(
            &connector,
            "GET",
            "https://example.com/hello",
            &[],
            NativeHttpBody::Bytes(Vec::new()),
            4,
            &cancelled,
            &connection,
        );
        assert_eq!(limit_reply, NativeHttpReply::LimitExceeded);
    }
}
