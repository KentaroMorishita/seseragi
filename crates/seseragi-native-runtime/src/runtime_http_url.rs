// `std/http.parseUrl` follows the WHATWG URL parser of the TS target for the
// inputs it can accept: ASCII text whose `%` signs start valid escapes. Only
// what `parseUrl` observes is kept: the scheme, the presence of credentials
// and of a fragment, and the serialized URL.

#[derive(Debug, PartialEq)]
enum NativeHttpUrl {
    Http {
        href: String,
        credentials: bool,
        fragment: bool,
    },
    // A URL the TS parser accepts, with a scheme other than http and https.
    Other(String),
}

const NATIVE_URL_SPECIAL: [(&str, Option<u16>); 6] = [
    ("http", Some(80)),
    ("https", Some(443)),
    ("ws", Some(80)),
    ("wss", Some(443)),
    ("ftp", Some(21)),
    ("file", None),
];

fn native_url_parse(input: &str) -> Option<NativeHttpUrl> {
    let trimmed = input.trim_matches(|c: char| c <= ' ');
    let text: String = trimmed
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    let colon = text.find(':')?;
    let scheme = &text[..colon];
    let mut letters = scheme.chars();
    if !letters.next()?.is_ascii_alphabetic()
        || !letters.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return None;
    }
    let scheme = scheme.to_ascii_lowercase();
    let rest = &text[colon + 1..];
    let Some(&(_, default_port)) = NATIVE_URL_SPECIAL.iter().find(|(name, _)| *name == scheme)
    else {
        return native_url_opaque(rest).then_some(NativeHttpUrl::Other(scheme));
    };
    if scheme == "file" {
        return Some(NativeHttpUrl::Other(scheme));
    }
    // A special scheme skips any slashes and backslashes before its authority.
    let rest = rest.trim_start_matches(['/', '\\']);
    let end = rest.find(['/', '\\', '?', '#']).unwrap_or(rest.len());
    let (authority, rest) = rest.split_at(end);
    let (credentials, host_port) = match authority.rfind('@') {
        Some(at) => {
            let (user, password) = match authority[..at].find(':') {
                Some(colon) => (&authority[..colon], &authority[colon + 1..at]),
                None => (&authority[..at], ""),
            };
            (
                !user.is_empty() || !password.is_empty(),
                &authority[at + 1..],
            )
        }
        None => (false, authority),
    };
    let (host, port) = native_url_host_port(host_port)?;
    let host = native_url_host(host)?;
    let port = match port {
        None | Some("") => None,
        Some(port) => {
            if !port.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let value = port.trim_start_matches('0');
            if value.len() > 5 {
                return None;
            }
            let value: u32 = if value.is_empty() {
                0
            } else {
                value.parse().ok()?
            };
            if value > 65535 {
                return None;
            }
            (Some(value as u16) != default_port).then_some(value as u16)
        }
    };
    let (path_and_query, fragment) = match rest.find('#') {
        Some(hash) => (&rest[..hash], Some(&rest[hash + 1..])),
        None => (rest, None),
    };
    let (path, query) = match path_and_query.find('?') {
        Some(question) => (
            &path_and_query[..question],
            Some(&path_and_query[question + 1..]),
        ),
        None => (path_and_query, None),
    };
    let mut href = format!("{scheme}://{host}");
    if let Some(port) = port {
        href.push_str(&format!(":{port}"));
    }
    href.push_str(&native_url_path(path));
    if let Some(query) = query {
        href.push('?');
        native_url_encode(&mut href, query, |b| {
            b < 0x21 || b > 0x7e || matches!(b, b'"' | b'#' | b'<' | b'>' | b'\'')
        });
    }
    if let Some(fragment) = fragment {
        href.push('#');
        native_url_encode(&mut href, fragment, |b| {
            b < 0x21 || b > 0x7e || matches!(b, b'"' | b'<' | b'>' | b'`')
        });
    }
    if scheme != "http" && scheme != "https" {
        return Some(NativeHttpUrl::Other(scheme));
    }
    Some(NativeHttpUrl::Http {
        href,
        credentials,
        fragment: fragment.is_some_and(|fragment| !fragment.is_empty()),
    })
}

// A URL without a special scheme. Only an authority can make it invalid.
fn native_url_opaque(rest: &str) -> bool {
    let Some(rest) = rest.strip_prefix("//") else {
        return true;
    };
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    let host_port = authority
        .rfind('@')
        .map_or(authority, |at| &authority[at + 1..]);
    if authority.contains('@') && host_port.is_empty() {
        return false;
    }
    let Some((host, port)) = native_url_host_port(host_port) else {
        return false;
    };
    if let Some(port) = port {
        if !port.bytes().all(|b| b.is_ascii_digit())
            || port.trim_start_matches('0').parse::<u32>().unwrap_or(0) > 65535
        {
            return false;
        }
    }
    if host.starts_with('[') {
        return native_url_ipv6(&host[1..host.len() - 1]).is_some();
    }
    !host.bytes().any(|b| {
        b < 0x20
            || b == 0x7f
            || matches!(
                b,
                b' ' | b'#'
                    | b'/'
                    | b':'
                    | b'<'
                    | b'>'
                    | b'?'
                    | b'@'
                    | b'['
                    | b'\\'
                    | b']'
                    | b'^'
                    | b'|'
            )
    })
}

fn native_url_host_port(text: &str) -> Option<(&str, Option<&str>)> {
    if text.starts_with('[') {
        let close = text.find(']')?;
        let rest = &text[close + 1..];
        return match rest.strip_prefix(':') {
            Some(port) => Some((&text[..=close], Some(port))),
            None if rest.is_empty() => Some((&text[..=close], None)),
            None => None,
        };
    }
    Some(match text.find(':') {
        Some(colon) => (&text[..colon], Some(&text[colon + 1..])),
        None => (text, None),
    })
}

fn native_url_host(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    if let Some(inner) = text.strip_prefix('[') {
        let inner = inner.strip_suffix(']')?;
        return Some(format!("[{}]", native_url_ipv6(inner)?));
    }
    let decoded = native_url_percent_decode(text);
    // Percent-decoding to a non-ASCII domain needs IDNA, which is not
    // supported natively. Such a host is refused.
    let domain = String::from_utf8(decoded).ok()?;
    if !domain.is_ascii() {
        return None;
    }
    let domain = domain.to_ascii_lowercase();
    if domain.is_empty()
        || domain.bytes().any(|b| {
            b <= 0x20
                || b == 0x7f
                || matches!(
                    b,
                    b'#' | b'%'
                        | b'/'
                        | b':'
                        | b'<'
                        | b'>'
                        | b'?'
                        | b'@'
                        | b'['
                        | b'\\'
                        | b']'
                        | b'^'
                        | b'|'
                )
        })
    {
        return None;
    }
    if native_url_ends_in_number(&domain) {
        return native_url_ipv4(&domain);
    }
    Some(domain)
}

fn native_url_ends_in_number(domain: &str) -> bool {
    let mut labels: Vec<&str> = domain.split('.').collect();
    if labels.last() == Some(&"") {
        if labels.len() == 1 {
            return false;
        }
        labels.pop();
    }
    let last = labels.last().copied().unwrap_or("");
    if !last.is_empty() && last.bytes().all(|b| b.is_ascii_digit()) {
        return true;
    }
    let hex = last.strip_prefix("0x").or_else(|| last.strip_prefix("0X"));
    hex.is_some_and(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn native_url_ipv4_number(text: &str) -> Option<u64> {
    if text.is_empty() {
        return None;
    }
    let (digits, radix) =
        if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            (hex, 16)
        } else if text.len() > 1 && text.starts_with('0') {
            (&text[1..], 8)
        } else {
            (text, 10)
        };
    if digits.is_empty() {
        return Some(0);
    }
    if !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    // Any value above 2^32 fails, so saturation keeps the comparison exact.
    Some(digits.chars().fold(0u64, |value, c| {
        value
            .saturating_mul(radix as u64)
            .saturating_add(c.to_digit(radix).unwrap() as u64)
            .min(1 << 40)
    }))
}

fn native_url_ipv4(domain: &str) -> Option<String> {
    let mut parts: Vec<&str> = domain.split('.').collect();
    if parts.last() == Some(&"") {
        parts.pop();
    }
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    let numbers = parts
        .iter()
        .map(|part| native_url_ipv4_number(part))
        .collect::<Option<Vec<_>>>()?;
    let (last, leading) = numbers.split_last().unwrap();
    if leading.iter().any(|n| *n > 255) || *last >= 256u64.pow(5 - numbers.len() as u32) {
        return None;
    }
    let mut address = *last;
    for (index, n) in leading.iter().enumerate() {
        address += n * 256u64.pow(3 - index as u32);
    }
    Some(
        (0..4)
            .map(|i| ((address >> (24 - 8 * i)) & 255).to_string())
            .collect::<Vec<_>>()
            .join("."),
    )
}

fn native_url_ipv6(text: &str) -> Option<String> {
    let mut pieces = [0u16; 8];
    let bytes = text.as_bytes();
    let (mut index, mut pointer, mut compress) = (0usize, 0usize, None);
    if bytes.first() == Some(&b':') {
        if bytes.get(1) != Some(&b':') {
            return None;
        }
        index = 2;
        pointer = 1;
        compress = Some(1);
    }
    while index < bytes.len() {
        if pointer == 8 {
            return None;
        }
        if bytes[index] == b':' {
            if compress.is_some() {
                return None;
            }
            index += 1;
            pointer += 1;
            compress = Some(pointer);
            continue;
        }
        let (mut value, mut length) = (0u16, 0);
        while length < 4 && index < bytes.len() && bytes[index].is_ascii_hexdigit() {
            value = value * 16 + (bytes[index] as char).to_digit(16).unwrap() as u16;
            index += 1;
            length += 1;
        }
        if index < bytes.len() && bytes[index] == b'.' {
            if length == 0 || pointer > 6 {
                return None;
            }
            index -= length;
            let mut seen = 0;
            while index < bytes.len() {
                let mut part: Option<u16> = None;
                if seen > 0 {
                    if bytes[index] == b'.' && seen < 4 {
                        index += 1;
                    } else {
                        return None;
                    }
                }
                if index >= bytes.len() || !bytes[index].is_ascii_digit() {
                    return None;
                }
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    let digit = (bytes[index] - b'0') as u16;
                    part = match part {
                        None => Some(digit),
                        Some(0) => return None,
                        Some(previous) => Some(previous * 10 + digit),
                    };
                    if part? > 255 {
                        return None;
                    }
                    index += 1;
                }
                pieces[pointer] = pieces[pointer] * 256 + part?;
                seen += 1;
                if seen == 2 || seen == 4 {
                    pointer += 1;
                }
            }
            if seen != 4 {
                return None;
            }
            break;
        }
        if index < bytes.len() && bytes[index] == b':' {
            index += 1;
            if index >= bytes.len() {
                return None;
            }
        } else if index < bytes.len() {
            return None;
        }
        pieces[pointer] = value;
        pointer += 1;
    }
    if let Some(compress) = compress {
        let mut swaps = pointer - compress;
        pointer = 7;
        while pointer != 0 && swaps > 0 {
            pieces.swap(pointer, compress + swaps - 1);
            pointer -= 1;
            swaps -= 1;
        }
    } else if pointer != 8 {
        return None;
    }
    // The longest run of two or more zero pieces is written as `::`.
    let (mut best, mut best_length, mut start) = (None, 1, None);
    for (i, piece) in pieces.iter().enumerate() {
        if *piece == 0 {
            let first = *start.get_or_insert(i);
            if i + 1 - first > best_length {
                best = Some(first);
                best_length = i + 1 - first;
            }
        } else {
            start = None;
        }
    }
    let mut output = String::new();
    let mut i = 0;
    while i < 8 {
        if Some(i) == best {
            output.push_str(if i == 0 { "::" } else { ":" });
            i += best_length;
            continue;
        }
        output.push_str(&format!("{:x}", pieces[i]));
        if i != 7 {
            output.push(':');
        }
        i += 1;
    }
    Some(output)
}

fn native_url_percent_decode(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if let (Some(high), Some(low)) = (
                bytes.get(i + 1).and_then(|b| (*b as char).to_digit(16)),
                bytes.get(i + 2).and_then(|b| (*b as char).to_digit(16)),
            ) {
                output.push((high * 16 + low) as u8);
                i += 3;
                continue;
            }
        }
        output.push(bytes[i]);
        i += 1;
    }
    output
}

fn native_url_encode(output: &mut String, text: &str, encode: impl Fn(u8) -> bool) {
    for byte in text.bytes() {
        if encode(byte) {
            output.push_str(&format!("%{byte:02X}"));
        } else {
            output.push(byte as char);
        }
    }
}

fn native_url_path(path: &str) -> String {
    let mut segments: Vec<String> = Vec::new();
    let parts: Vec<&str> = path.split(['/', '\\']).collect();
    // The text before the first separator is empty for a path that starts
    // with one; an authority without a path has a single empty segment.
    let parts = if parts.len() > 1 {
        &parts[1..]
    } else {
        &parts[..]
    };
    for (index, part) in parts.iter().enumerate() {
        let last = index + 1 == parts.len();
        let lower = part.to_ascii_lowercase();
        if matches!(lower.as_str(), ".." | ".%2e" | "%2e." | "%2e%2e") {
            segments.pop();
            if last {
                segments.push(String::new());
            }
        } else if matches!(lower.as_str(), "." | "%2e") {
            if last {
                segments.push(String::new());
            }
        } else {
            let mut segment = String::new();
            native_url_encode(&mut segment, part, |b| {
                b < 0x21
                    || b > 0x7e
                    || matches!(b, b'"' | b'#' | b'<' | b'>' | b'?' | b'`' | b'{' | b'}')
            });
            segments.push(segment);
        }
    }
    if segments.is_empty() {
        return "/".into();
    }
    format!("/{}", segments.join("/"))
}

#[cfg(test)]
mod http_url_tests {
    use super::*;

    fn href(input: &str) -> Option<String> {
        match native_url_parse(input)? {
            NativeHttpUrl::Http { href, .. } => Some(href),
            NativeHttpUrl::Other(scheme) => Some(format!("other {scheme}")),
        }
    }

    #[test]
    fn urls_serialize_like_the_whatwg_parser() {
        for (input, expected) in [
            ("http://Example.COM", Some("http://example.com/")),
            ("HTTPS://example.com:443/a", Some("https://example.com/a")),
            ("http://example.com:0080/", Some("http://example.com/")),
            ("http://example.com:8080", Some("http://example.com:8080/")),
            ("http:example.com/x", Some("http://example.com/x")),
            ("http:\\\\example.com\\a\\b", Some("http://example.com/a/b")),
            ("  http://a/\tb\n ", Some("http://a/b")),
            ("http://a/b/../c/./d/..", Some("http://a/c/")),
            ("http://a/%2e%2E/x/%2E", Some("http://a/x/")),
            ("http://a/a b?c d'e", Some("http://a/a%20b?c%20d%27e")),
            ("http://a/?", Some("http://a/?")),
            ("http://a/#", Some("http://a/#")),
            ("http://0x7f.1/", Some("http://127.0.0.1/")),
            ("http://2130706433", Some("http://127.0.0.1/")),
            ("http://[0:0::1]:81/", Some("http://[::1]:81/")),
            ("http://[1:0:0:2::3:0]/", Some("http://[1::2:0:0:3:0]/")),
            (
                "http://[::ffff:192.168.0.1]/",
                Some("http://[::ffff:c0a8:1]/"),
            ),
            ("http://@a/", Some("http://a/")),
            ("http://example.com:65536/", None),
            ("http://exa mple.com/", None),
            ("http://", None),
            ("http://a:b/", None),
            ("http://256.0.0.1/", None),
            ("http://[1::2::3]/", None),
            ("http://%41.com/", Some("http://a.com/")),
            ("mailto:a@b", Some("other mailto")),
            ("ftp://a/", Some("other ftp")),
            ("ftp://", None),
            ("1http://a/", None),
            ("example.com", None),
            ("http://a/{x}|`y`", Some("http://a/%7Bx%7D|%60y%60")),
            ("http://a?b#c d", Some("http://a/?b#c%20d")),
            ("http://09/", None),
            ("http://1.2.3.4.5/", None),
            ("http://a./", Some("http://a./")),
            ("http://0x/", Some("http://0.0.0.0/")),
            ("http://a/..", Some("http://a/")),
            ("http://a/./", Some("http://a/")),
            ("http://a:/", Some("http://a/")),
        ] {
            assert_eq!(href(input).as_deref(), expected, "{input}");
        }
    }

    #[test]
    fn credentials_and_fragments_are_reported() {
        let Some(NativeHttpUrl::Http {
            credentials,
            fragment,
            ..
        }) = native_url_parse("http://u:p@a/#x")
        else {
            panic!()
        };
        assert!(credentials && fragment);
        let Some(NativeHttpUrl::Http {
            credentials,
            fragment,
            ..
        }) = native_url_parse("http://:@a/#")
        else {
            panic!()
        };
        assert!(!credentials && !fragment);
    }
}
