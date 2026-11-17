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
    connection: std::sync::Arc<std::sync::Mutex<Option<std::net::TcpStream>>>,
    completion: std::sync::mpsc::Receiver<NativeHttpReply>,
}

fn native_http_failure(tag: &'static str, message: &str) -> NativeExit {
    NativeExit::failure(V::adt(tag, V::text(message)))
}

// The Content-Length a request declares must match its body, as on TS.
fn native_http_declared_length(
    headers: &[(String, String)],
    actual: usize,
) -> Result<(), NativeExit> {
    let values: Vec<_> = headers
        .iter()
        .filter(|(name, _)| name == "content-length")
        .collect();
    let Some((_, value)) = values.first() else {
        return Ok(());
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
    let declared = match value.parse::<i64>() {
        Ok(declared) if declared <= 9_007_199_254_740_991 => declared,
        _ => {
            return Err(native_http_failure(
                "std/http::HttpProtocolFailure",
                "Content-Length exceeds the safe integer range",
            ))
        }
    };
    if declared != actual as i64 {
        return Err(NativeExit::failure(V::adt(
            "std/http::HttpRequestLengthMismatch",
            native_http_record(vec![
                ("declared", V::Int(declared)),
                ("actual", V::Int(actual as i64)),
            ]),
        )));
    }
    Ok(())
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
                    body,
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
        if let Some(stream) = self.connection.lock().unwrap().as_ref() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
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

struct NativeHttpTarget {
    host: String,
    port: u16,
    authority: String,
    path: String,
}

fn native_http_target(url: &str) -> Result<NativeHttpTarget, NativeHttpReply> {
    if url.starts_with("https://") {
        return Err(NativeHttpReply::Failure(
            "std/http::HttpTlsFailure",
            "https needs TLS, which the native target does not support yet".into(),
        ));
    }
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        NativeHttpReply::Failure("std/http::HttpProtocolFailure", "invalid HTTP URL".into())
    })?;
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
        host: host.into(),
        port: port.and_then(|port| port.parse().ok()).unwrap_or(80),
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

fn native_http_exchange(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: Vec<u8>,
    limit: u64,
    cancelled: &std::sync::atomic::AtomicBool,
    connection: &std::sync::Mutex<Option<std::net::TcpStream>>,
) -> NativeHttpReply {
    use std::io::Write;
    let target = match native_http_target(url) {
        Ok(target) => target,
        Err(reply) => return reply,
    };
    let is_cancelled = || cancelled.load(std::sync::atomic::Ordering::Acquire);
    let (connected, connecting) = std::sync::mpsc::channel();
    let (host, port) = (target.host.clone(), target.port);
    std::thread::spawn(move || {
        // Name resolution also blocks in the OS, so it runs here too.
        let addresses = match std::net::ToSocketAddrs::to_socket_addrs(&(host.as_str(), port)) {
            Ok(addresses) => addresses.collect::<Vec<_>>(),
            Err(error) => {
                let _ = connected.send(Err(("std/http::HttpDnsFailure", error.to_string())));
                return;
            }
        };
        if addresses.is_empty() {
            let _ = connected.send(Err((
                "std/http::HttpDnsFailure",
                format!("no address for {host}"),
            )));
            return;
        }
        let mut last = String::new();
        for address in addresses {
            match std::net::TcpStream::connect(address) {
                Ok(stream) => {
                    let _ = connected.send(Ok(stream));
                    return;
                }
                Err(error) => last = error.to_string(),
            }
        }
        let _ = connected.send(Err(("std/http::HttpConnectionFailure", last)));
    });
    let stream = loop {
        if is_cancelled() {
            return NativeHttpReply::Cancelled;
        }
        match connecting.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(Ok(stream)) => break stream,
            Ok(Err((tag, message))) => return NativeHttpReply::Failure(tag, message),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return NativeHttpReply::Failure(
                    "std/http::HttpConnectionFailure",
                    "connection attempt ended without a result".into(),
                )
            }
        }
    };
    if let Ok(clone) = stream.try_clone() {
        *connection.lock().unwrap() = Some(clone);
    }
    if is_cancelled() {
        return NativeHttpReply::Cancelled;
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
    if !declared && (!body.is_empty() || matches!(method, "POST" | "PUT" | "PATCH")) {
        head.extend(format!("content-length: {}\r\n", body.len()).bytes());
    }
    head.extend(b"connection: close\r\n\r\n");
    let failure = |error: std::io::Error, tag: &'static str| {
        if is_cancelled() {
            NativeHttpReply::Cancelled
        } else {
            NativeHttpReply::Failure(tag, error.to_string())
        }
    };
    // The request is written on its own thread, so a final response that
    // arrives before the body is fully sent (such as 413) is still read.
    // Closing the connection after the response stops the writer.
    let Ok(mut writer) = stream.try_clone() else {
        return NativeHttpReply::Failure(
            "std/http::HttpConnectionFailure",
            "cannot share the connection".into(),
        );
    };
    let written = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer_done = written.clone();
    std::thread::spawn(move || {
        let _ = writer
            .write_all(&head)
            .and_then(|_| writer.write_all(&body));
        writer_done.store(true, std::sync::atomic::Ordering::Release);
    });
    let mut reader = std::io::BufReader::with_capacity(NATIVE_HTTP_CHUNK_BYTES, stream);
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
            let _ = reader.get_mut().shutdown(std::net::Shutdown::Both);
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

type NativeHttpHead = (u16, Vec<(String, String)>);

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
    let status = match line.as_slice() {
        [b'H', b'T', b'T', b'P', b'/', b'1', b'.', b'0' | b'1', b' ', a, b, c, rest @ ..]
            if [a, b, c].iter().all(|d| d.is_ascii_digit())
                && (rest.is_empty() || (rest[0] == b' ' && valid_text(&rest[1..]))) =>
        {
            (*a - b'0') as u16 * 100 + (*b - b'0') as u16 * 10 + (*c - b'0') as u16
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
    Ok((status, headers))
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
    let chunked = transfer
        .iter()
        .flat_map(|(_, value)| value.split(','))
        .filter(|coding| coding.trim().eq_ignore_ascii_case("chunked"))
        .count();
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
    connection: &std::sync::Mutex<Option<std::net::TcpStream>>,
) -> std::io::Result<()> {
    match connection.lock().unwrap().as_ref() {
        Some(stream) => stream.shutdown(std::net::Shutdown::Write),
        None => Ok(()),
    }
}

fn native_http_read_response(
    reader: &mut impl std::io::BufRead,
    head_request: bool,
    limit: u64,
    on_final_head: &mut dyn FnMut(),
) -> Result<(u16, Vec<(String, String)>, Vec<u8>), NativeHttpRead> {
    let mut informational = 0;
    let (status, headers) = loop {
        let (status, headers) = native_http_read_head(reader)?;
        native_http_framing(&headers)?;
        if !(100..200).contains(&status) {
            on_final_head();
            break (status, headers);
        }
        informational += 1;
        if informational > NATIVE_HTTP_INFORMATIONAL_LIMIT {
            return native_http_protocol("HTTP informational response limit exceeded");
        }
    };
    let values = |name: &str| -> Vec<String> {
        headers
            .iter()
            .filter(|(header, _)| header == name)
            .map(|(_, value)| value.clone())
            .collect()
    };
    let mut body = Vec::new();
    if head_request || matches!(status, 204 | 304) {
        return Ok((status, headers, body));
    }
    let codings: Vec<String> = values("transfer-encoding")
        .iter()
        .flat_map(|value| value.split(','))
        .map(|coding| coding.trim().to_ascii_lowercase())
        .filter(|coding| !coding.is_empty())
        .collect();
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
                loop {
                    let line = native_http_read_line(reader, &mut budget)?;
                    if line.is_empty() {
                        break;
                    }
                    native_http_header_line(&line)?;
                }
                break;
            }
            while remaining > 0 {
                let want = remaining.min(buffer.len());
                let read = reader.read(&mut buffer[..want])?;
                if read == 0 {
                    return native_http_protocol("connection closed inside a chunk");
                }
                native_http_append(&mut body, &buffer[..read], limit)?;
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
            native_http_append(&mut body, &buffer[..read], limit)?;
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
            native_http_append(&mut body, &buffer[..read], limit)?;
        }
    }
    let encoded = values("content-encoding")
        .iter()
        .flat_map(|value| value.split(','))
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
    fn targets_keep_the_authority_and_refuse_https() {
        let target = native_http_target("http://[::1]:8080?q").ok().unwrap();
        assert_eq!(
            (target.host.as_str(), target.port, target.path.as_str()),
            ("::1", 8080, "/?q")
        );
        assert_eq!(target.authority, "[::1]:8080");
        let target = native_http_target("http://a/p?#").ok().unwrap();
        assert_eq!(target.path, "/p?");
        assert!(matches!(
            native_http_target("https://a/"),
            Err(NativeHttpReply::Failure("std/http::HttpTlsFailure", _))
        ));
    }
}
