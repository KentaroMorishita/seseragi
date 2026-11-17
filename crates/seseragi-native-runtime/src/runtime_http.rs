// `std/http` values. Each opaque type keeps its owner, as on the TS target
// where they are branded: a Method is its text, a Status its code, Headers a
// list of lowercase names and values in insertion order.

const NATIVE_HTTP_DEFAULT_BODY_LIMIT: i64 = 8 * 1024 * 1024;
const NATIVE_HTTP_MANAGED_HEADERS: [&str; 7] = [
    "connection",
    "proxy-connection",
    "keep-alive",
    "transfer-encoding",
    "upgrade",
    "te",
    "host",
];

fn native_http_value(owner: &'static str, value: V) -> V {
    V::adt(owner, value)
}
fn native_http_inner<'a>(value: &'a V, owner: &str) -> &'a V {
    let V::Adt(adt) = value else {
        panic!("Rust ABI: expected {owner}")
    };
    assert_eq!(adt.get().0, owner, "Rust ABI: {owner} owner");
    &adt.get().1
}
fn native_http_int(value: &V) -> i64 {
    let V::Int(value) = value else {
        panic!("Rust ABI: expected Int")
    };
    *value
}
fn native_http_record(fields: Vec<(&str, V)>) -> V {
    V::Record(Shared::new(
        fields
            .into_iter()
            .map(|(name, value)| (name.into(), value))
            .collect(),
    ))
}
fn native_http_build_error(tag: &'static str, payload: V) -> V {
    native_left(V::adt(tag, payload))
}

fn native_http_token(text: &str) -> bool {
    !text.is_empty()
        && text.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

fn native_http_headers(value: &V) -> Vec<(String, String)> {
    let V::Array(entries) = native_http_inner(value, "std/http::Headers") else {
        panic!("Rust ABI: expected Headers entries")
    };
    entries
        .get()
        .iter()
        .map(|entry| {
            let V::Tuple(pair) = entry else {
                panic!("Rust ABI: expected a header entry")
            };
            (
                native_text(&pair.get()[0]).to_owned(),
                native_text(&pair.get()[1]).to_owned(),
            )
        })
        .collect()
}
fn native_http_headers_value(entries: Vec<(String, String)>) -> V {
    native_http_value(
        "std/http::Headers",
        V::Array(Shared::new(
            entries
                .into_iter()
                .map(|(name, value)| V::Tuple(Shared::new(vec![V::text(&name), V::text(&value)])))
                .collect(),
        )),
    )
}

// The error a header name and value would have, or the lowercase name.
fn native_http_validate_header(name: &str, value: &str) -> Result<String, V> {
    if !native_http_token(name) {
        return Err(V::adt("std/http::InvalidHeaderName", V::text(name)));
    }
    let normalized = name.to_ascii_lowercase();
    if NATIVE_HTTP_MANAGED_HEADERS.contains(&normalized.as_str()) || normalized.starts_with(':') {
        return Err(V::adt("std/http::ManagedHttpHeader", V::text(&normalized)));
    }
    // The TS target counts the offset in code points.
    if let Some(offset) = value
        .chars()
        .position(|c| matches!(c as u32, 0 | 10 | 13) || c as u32 > 255)
    {
        return Err(V::adt(
            "std/http::InvalidHeaderValue",
            native_http_record(vec![
                ("name", V::text(&normalized)),
                ("offset", V::Int(offset as i64)),
            ]),
        ));
    }
    Ok(normalized)
}

fn native_http_append_header(name: &V, value: &V, headers: &V) -> V {
    match native_http_validate_header(native_text(name), native_text(value)) {
        Err(error) => native_left(error),
        Ok(normalized) => {
            let mut entries = native_http_headers(headers);
            entries.push((normalized, native_text(value).to_owned()));
            native_right(native_http_headers_value(entries))
        }
    }
}

fn native_http_set_header(name: &V, value: &V, headers: &V) -> V {
    match native_http_validate_header(native_text(name), native_text(value)) {
        Err(error) => native_left(error),
        Ok(normalized) => {
            let entries = native_http_headers(headers);
            let first = entries.iter().position(|(name, _)| *name == normalized);
            let mut retained: Vec<_> = entries
                .into_iter()
                .filter(|(name, _)| *name != normalized)
                .collect();
            let at = first.unwrap_or(retained.len());
            retained.insert(at, (normalized, native_text(value).to_owned()));
            native_right(native_http_headers_value(retained))
        }
    }
}

fn native_http_remove_header(name: &V, headers: &V) -> V {
    let normalized = native_text(name).to_lowercase();
    native_http_headers_value(
        native_http_headers(headers)
            .into_iter()
            .filter(|(name, _)| *name != normalized)
            .collect(),
    )
}

fn native_http_request(method: V, url: V, headers: V) -> V {
    native_http_value(
        "std/http::Request",
        native_http_record(vec![("method", method), ("url", url), ("headers", headers)]),
    )
}

fn native_http_parse_url(text: &str) -> V {
    // Offsets count UTF-16 units; every unit before the first refused one is
    // ASCII, so the byte index is the same.
    let bytes = text.as_bytes();
    for (offset, byte) in bytes.iter().enumerate() {
        if *byte > 127
            || (*byte == b'%'
                && !(bytes.get(offset + 1).is_some_and(u8::is_ascii_hexdigit)
                    && bytes.get(offset + 2).is_some_and(u8::is_ascii_hexdigit)))
        {
            return native_http_build_error(
                "std/http::InvalidHttpUrl",
                native_http_record(vec![("offset", V::Int(offset as i64))]),
            );
        }
    }
    match native_url_parse(text) {
        None => native_http_build_error(
            "std/http::InvalidHttpUrl",
            native_http_record(vec![("offset", V::Int(0))]),
        ),
        Some(NativeHttpUrl::Other(scheme)) => {
            native_http_build_error("std/http::UnsupportedHttpScheme", V::text(&scheme))
        }
        Some(NativeHttpUrl::Http {
            href,
            credentials,
            fragment,
        }) => {
            if credentials {
                native_http_build_error("std/http::HttpUrlContainsUserInfo", V::Unit)
            } else if fragment {
                native_http_build_error("std/http::HttpUrlContainsFragment", V::Unit)
            } else {
                native_right(native_http_value(
                    "std/http::HttpUrl",
                    V::text(&native_http_uppercase_escapes(&href)),
                ))
            }
        }
    }
}

// The TS target writes every percent escape with uppercase digits.
fn native_http_uppercase_escapes(text: &str) -> String {
    let mut bytes = text.as_bytes().to_vec();
    for i in 0..bytes.len() {
        if bytes[i] == b'%'
            && bytes.get(i + 1).is_some_and(u8::is_ascii_hexdigit)
            && bytes.get(i + 2).is_some_and(u8::is_ascii_hexdigit)
        {
            bytes[i + 1] = bytes[i + 1].to_ascii_uppercase();
            bytes[i + 2] = bytes[i + 2].to_ascii_uppercase();
        }
    }
    String::from_utf8(bytes).expect("ASCII URL")
}

fn native_http_error_message(error: &V) -> String {
    let tag = native_error_tag(error);
    let payload = error.payload();
    match tag {
        "HttpClientUnavailable" => tag.into(),
        "HttpRequestLengthMismatch" => format!(
            "{tag}: declared {}, actual {}",
            native_http_int(&payload.field("declared")),
            native_http_int(&payload.field("actual"))
        ),
        "HttpResponseBodyLimitExceeded" => format!(
            "{tag}: {} bytes",
            native_http_int(&payload.field("limitBytes"))
        ),
        _ => format!("{tag}: {}", native_text(&payload)),
    }
}

fn native_http_status_class(status: &V, low: i64) -> V {
    let code = native_http_int(native_http_inner(status, "std/http::Status"));
    V::Bool(code >= low && code < low + 100)
}

pub fn native_http_operation(name: &str, arguments: Vec<V>) -> V {
    let a = &arguments;
    match name {
        "std/http::get" => native_http_value("std/http::Method", V::text("GET")),
        "std/http::head" => native_http_value("std/http::Method", V::text("HEAD")),
        "std/http::post" => native_http_value("std/http::Method", V::text("POST")),
        "std/http::put" => native_http_value("std/http::Method", V::text("PUT")),
        "std/http::patch" => native_http_value("std/http::Method", V::text("PATCH")),
        "std/http::delete" => native_http_value("std/http::Method", V::text("DELETE")),
        "std/http::options" => native_http_value("std/http::Method", V::text("OPTIONS")),
        "std/http::connect" => native_http_value("std/http::Method", V::text("CONNECT")),
        "std/http::trace" => native_http_value("std/http::Method", V::text("TRACE")),
        "std/http::HttpVersionUnknown" => V::adt("std/http::HttpVersionUnknown", V::Unit),
        "std/http::Http1_0" => V::adt("std/http::Http1_0", V::Unit),
        "std/http::Http1_1" => V::adt("std/http::Http1_1", V::Unit),
        "std/http::Http2" => V::adt("std/http::Http2", V::Unit),
        "std/http::Http3" => V::adt("std/http::Http3", V::Unit),
        "std/http::emptyHeaders" => native_http_headers_value(Vec::new()),
        "std/http::customMethod" => {
            let text = native_text(&a[0]);
            if native_http_token(text) && text == text.to_uppercase() {
                native_right(native_http_value("std/http::Method", a[0].clone()))
            } else {
                native_http_build_error("std/http::InvalidHttpMethod", a[0].clone())
            }
        }
        "std/http::methodText" => native_http_inner(&a[0], "std/http::Method").clone(),
        "std/http::status" => {
            let code = native_http_int(&a[0]);
            if (100..=999).contains(&code) {
                native_right(native_http_value("std/http::Status", a[0].clone()))
            } else {
                native_http_build_error("std/http::InvalidHttpStatus", a[0].clone())
            }
        }
        "std/http::statusCode" => native_http_inner(&a[0], "std/http::Status").clone(),
        "std/http::isInformational" => native_http_status_class(&a[0], 100),
        "std/http::isSuccess" => native_http_status_class(&a[0], 200),
        "std/http::isRedirection" => native_http_status_class(&a[0], 300),
        "std/http::isClientError" => native_http_status_class(&a[0], 400),
        "std/http::isServerError" => native_http_status_class(&a[0], 500),
        "std/http::parseUrl" => native_http_parse_url(native_text(&a[0])),
        "std/http::renderUrl" => native_http_inner(&a[0], "std/http::HttpUrl").clone(),
        "std/http::appendHeader" => native_http_append_header(&a[0], &a[1], &a[2]),
        "std/http::setHeader" => native_http_set_header(&a[0], &a[1], &a[2]),
        "std/http::removeHeader" => native_http_remove_header(&a[0], &a[1]),
        "std/http::headerValues" => {
            let normalized = native_text(&a[0]).to_lowercase();
            V::Array(Shared::new(
                native_http_headers(&a[1])
                    .into_iter()
                    .filter(|(name, _)| *name == normalized)
                    .map(|(_, value)| V::text(&value))
                    .collect(),
            ))
        }
        "std/http::headerEntries" => native_http_inner(&a[0], "std/http::Headers").clone(),
        "std/http::request" => native_http_request(
            a[0].clone(),
            a[1].clone(),
            native_http_headers_value(Vec::new()),
        ),
        "std/http::withRequestHeader" => {
            let request = native_http_inner(&a[2], "std/http::Request");
            match native_http_append_header(&a[0], &a[1], &request.field("headers")) {
                V::Adt(result) if result.get().0 == "std/prelude::Right" => {
                    native_right(native_http_request(
                        request.field("method"),
                        request.field("url"),
                        result.get().1.clone(),
                    ))
                }
                error => error,
            }
        }
        "std/http::withoutRequestHeader" => {
            let request = native_http_inner(&a[1], "std/http::Request");
            native_http_request(
                request.field("method"),
                request.field("url"),
                native_http_remove_header(&a[0], &request.field("headers")),
            )
        }
        "std/http::bodyLimit" => {
            let bytes = native_http_int(&a[0]);
            if bytes > 0 && bytes <= 9_007_199_254_740_991 {
                native_right(native_http_value("std/http::HttpBodyLimit", a[0].clone()))
            } else {
                native_http_build_error("std/http::InvalidHttpBodyLimit", a[0].clone())
            }
        }
        "std/http::defaultBodyLimit" => native_http_value(
            "std/http::HttpBodyLimit",
            V::Int(NATIVE_HTTP_DEFAULT_BODY_LIMIT),
        ),
        "std/http::responseStatus" => {
            native_http_inner(&a[0], "std/http::Response").field("status")
        }
        "std/http::responseHeaders" => {
            native_http_inner(&a[0], "std/http::Response").field("headers")
        }
        "std/http::responseBody" => native_http_inner(&a[0], "std/http::Response").field("body"),
        "std/http::sendBytes" => V::effect(Instruction::HttpSend, arguments),
        "std/http::sendEmpty" => V::effect(
            Instruction::HttpSend,
            vec![a[0].clone(), V::Bytes(Shared::new(Vec::new())), a[1].clone()],
        ),
        "std/http::errorMessage" => V::text(&native_http_error_message(&a[0])),
        _ => panic!("Rust ABI: unsupported std/http operation {name}"),
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;

    fn op(name: &str, arguments: Vec<V>) -> V {
        native_http_operation(name, arguments)
    }
    fn right(value: V) -> V {
        assert!(value.tagged("std/prelude::Right"), "{}", value.render());
        value.payload()
    }
    fn left_tag(value: V) -> String {
        assert!(value.tagged("std/prelude::Left"), "{}", value.render());
        native_error_tag(&value.payload()).to_owned()
    }

    #[test]
    fn headers_keep_order_lowercase_names_and_refuse_managed_ones() {
        let empty = op("std/http::emptyHeaders", vec![]);
        let headers = right(op(
            "std/http::appendHeader",
            vec![V::text("X-A"), V::text("1"), empty],
        ));
        let headers = right(op(
            "std/http::appendHeader",
            vec![V::text("x-b"), V::text("2"), headers],
        ));
        let headers = right(op(
            "std/http::appendHeader",
            vec![V::text("X-a"), V::text("3"), headers],
        ));
        let headers = right(op(
            "std/http::setHeader",
            vec![V::text("x-A"), V::text("4"), headers],
        ));
        assert_eq!(
            native_http_headers(&headers),
            [("x-a".into(), "4".into()), ("x-b".into(), "2".into())]
        );
        for (name, value, tag) in [
            ("", "v", "InvalidHeaderName"),
            ("a b", "v", "InvalidHeaderName"),
            ("Host", "v", "ManagedHttpHeader"),
            ("x", "a\u{100}", "InvalidHeaderValue"),
            ("x", "a\r", "InvalidHeaderValue"),
        ] {
            assert_eq!(
                left_tag(op(
                    "std/http::appendHeader",
                    vec![V::text(name), V::text(value), headers.clone()]
                )),
                tag
            );
        }
        let removed = op("std/http::removeHeader", vec![V::text("X-A"), headers]);
        assert_eq!(native_http_headers(&removed), [("x-b".into(), "2".into())]);
        drop(removed);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn methods_statuses_and_limits_follow_the_ts_ranges() {
        assert_eq!(
            left_tag(op("std/http::customMethod", vec![V::text("get")])),
            "InvalidHttpMethod"
        );
        right(op("std/http::customMethod", vec![V::text("PURGE")]));
        for code in [99, 1000, -1] {
            assert_eq!(
                left_tag(op("std/http::status", vec![V::Int(code)])),
                "InvalidHttpStatus"
            );
        }
        let status = right(op("std/http::status", vec![V::Int(404)]));
        assert!(matches!(
            op("std/http::isClientError", vec![status]),
            V::Bool(true)
        ));
        for bytes in [0, -1, 9_007_199_254_740_992] {
            assert_eq!(
                left_tag(op("std/http::bodyLimit", vec![V::Int(bytes)])),
                "InvalidHttpBodyLimit"
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn parse_url_reports_the_first_refused_offset_and_kind() {
        for (input, tag) in [
            ("http://a/é", "InvalidHttpUrl"),
            ("http://a/%zz", "InvalidHttpUrl"),
            ("not a url", "InvalidHttpUrl"),
            ("ftp://a/", "UnsupportedHttpScheme"),
            ("http://u@a/", "HttpUrlContainsUserInfo"),
            ("http://a/#x", "HttpUrlContainsFragment"),
        ] {
            assert_eq!(
                left_tag(op("std/http::parseUrl", vec![V::text(input)])),
                tag,
                "{input}"
            );
        }
        let error = op("std/http::parseUrl", vec![V::text("http://a/%zz")]).payload();
        assert_eq!(native_http_int(&error.payload().field("offset")), 9);
        let url = right(op("std/http::parseUrl", vec![V::text("HTTP://A/%7e")]));
        assert_eq!(
            native_text(&op("std/http::renderUrl", vec![url])),
            "http://a/%7E"
        );
        drop(error);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
