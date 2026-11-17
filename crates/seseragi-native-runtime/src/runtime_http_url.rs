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
        return native_url_file(rest).then_some(NativeHttpUrl::Other(scheme));
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

// A file URL is invalid only through its host, which takes no port or
// credentials. A Windows drive letter in the host position is a path.
fn native_url_file(rest: &str) -> bool {
    let Some(rest) = rest
        .strip_prefix(['/', '\\'])
        .and_then(|r| r.strip_prefix(['/', '\\']))
    else {
        return true;
    };
    let end = rest.find(['/', '\\', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..end];
    let drive = host.len() == 2
        && host.as_bytes()[0].is_ascii_alphabetic()
        && matches!(host.as_bytes()[1], b':' | b'|');
    host.is_empty() || drive || native_url_host(host).is_some()
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
        // A port needs a host, and must fit in 16 bits.
        let digits = port.trim_start_matches('0');
        if host.is_empty()
            || !port.bytes().all(|b| b.is_ascii_digit())
            || digits.len() > 5
            || (!digits.is_empty() && digits.parse::<u32>().map_or(true, |p| p > 65535))
        {
            return false;
        }
    }
    if host.starts_with('[') {
        return native_url_ipv6(&host[1..host.len() - 1]).is_some();
    }
    // Only the forbidden host code points refuse an opaque host; other
    // controls and DEL are allowed there, unlike in a domain.
    !host.bytes().any(|b| {
        b == 0
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
    let domain = String::from_utf8(decoded).ok()?;
    let domain = if domain.is_ascii() {
        domain.to_ascii_lowercase()
    } else {
        native_url_domain_to_ascii(&domain)?
    };
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
    if domain
        .split('.')
        .any(|label| label.starts_with("xn--") && native_url_punycode(&label[4..]).is_none())
    {
        return None;
    }
    if native_url_ends_in_number(&domain) {
        return native_url_ipv4(&domain);
    }
    Some(domain)
}

// A small part of IDNA ToASCII for a host that is not ASCII after
// percent-decoding: full stops and fullwidth ASCII map to ASCII, letters
// lowercase, and each label that is still not ASCII becomes Punycode. Full
// IDNA needs Unicode mapping and normalization tables the native runtime does
// not carry, so a host with a combining mark, which normalization could
// change, is refused rather than encoded differently from the TS target.
fn native_url_domain_to_ascii(domain: &str) -> Option<String> {
    // Characters IDNA maps through compatibility tables (ligatures, width
    // and presentation forms, letterlike and enclosed symbols, superscripts,
    // math letters and similar) are refused: encoding them unmapped would
    // name a different host than the TS target.
    let compatibility = |c: char| {
        matches!(c as u32,
            0x00aa | 0x00b2 | 0x00b3 | 0x00b5 | 0x00b9 | 0x00ba | 0x00bc..=0x00be
            | 0x0132 | 0x0133 | 0x013f | 0x0140 | 0x0149 | 0x017f | 0x01c4..=0x01cc
            | 0x01f1..=0x01f3 | 0x02b0..=0x02ff | 0x1d2c..=0x1dbf | 0x2000..=0x209f | 0x20a8 | 0x2100..=0x218f | 0x2460..=0x24ff | 0x2f00..=0x2fdf
            | 0x3000..=0x303f | 0x3200..=0x33ff | 0xf900..=0xfaff | 0xfb00..=0xfdff
            | 0xfe10..=0xfe1f | 0xfe30..=0xfe4f | 0xfe50..=0xfeff | 0xff00..=0xffef
            | 0x1d400..=0x1d7ff | 0x1f100..=0x1f2ff | 0x2f800..=0x2fa1f)
    };
    let mut mapped = String::new();
    for c in domain.chars() {
        match c as u32 {
            // IDNA ignores these: soft hyphen, combining grapheme joiner,
            // Mongolian variation selectors, zero width space, word joiner and
            // invisible operators, variation selectors and the BOM.
            0x00ad
            | 0x034f
            | 0x180b..=0x180d
            | 0x180f
            | 0x200b
            | 0x2060..=0x2064
            | 0xfe00..=0xfe0f
            | 0xfeff
            | 0x1bca0..=0x1bca3
            | 0xe0100..=0xe01ef => {}
            0x3002 | 0xff0e | 0xff61 => mapped.push('.'),
            0xff01..=0xff5e => mapped.push(char::from_u32(c as u32 - 0xfee0).unwrap()),
            // Cherokee letters fold to the uppercase forms in IDNA.
            0x13a0..=0x13f5 => mapped.push(c),
            0x13f8..=0x13fd => mapped.push(char::from_u32(c as u32 - 8).unwrap()),
            0xab70..=0xabbf => mapped.push(char::from_u32(c as u32 - 0xab70 + 0x13a0).unwrap()),
            _ if compatibility(c) => return None,
            _ => mapped.extend(c.to_lowercase()),
        }
    }
    if mapped.chars().any(|c| {
        c.is_control()
            || c.is_whitespace()
            || matches!(c as u32,
                0x0300..=0x036f | 0x1ab0..=0x1aff | 0x1dc0..=0x1dff | 0x20d0..=0x20ff | 0xfe20..=0xfe2f)
    }) {
        return None;
    }
    let labels = mapped
        .split('.')
        .map(|label| {
            if label.is_ascii() {
                Some(label.to_owned())
            } else {
                Some(format!("xn--{}", native_url_punycode_encode(label)?))
            }
        })
        .collect::<Option<Vec<_>>>()?;
    Some(labels.join("."))
}

// Encodes a label as Punycode (RFC 3492).
fn native_url_punycode_encode(label: &str) -> Option<String> {
    let (base, tmin, tmax, skew, damp) = (36u32, 1u32, 26u32, 38u32, 700u32);
    let adapt = |delta: u32, points: u32, first: bool| {
        let mut delta = if first { delta / damp } else { delta / 2 };
        delta += delta / points;
        let mut k = 0;
        while delta > ((base - tmin) * tmax) / 2 {
            delta /= base - tmin;
            k += base;
        }
        k + (base - tmin + 1) * delta / (delta + skew)
    };
    let digit = |d: u32| {
        (if d < 26 {
            b'a' + d as u8
        } else {
            b'0' + (d - 26) as u8
        }) as char
    };
    let input: Vec<u32> = label.chars().map(|c| c as u32).collect();
    let mut output: String = label.chars().filter(char::is_ascii).collect();
    let basic = output.len() as u32;
    let mut handled = basic;
    if basic > 0 {
        output.push('-');
    }
    let (mut n, mut delta, mut bias) = (128u32, 0u32, 72u32);
    while (handled as usize) < input.len() {
        let m = *input.iter().filter(|c| **c >= n).min()?;
        delta = delta.checked_add((m - n).checked_mul(handled + 1)?)?;
        n = m;
        for c in &input {
            if *c < n {
                delta = delta.checked_add(1)?;
            }
            if *c == n {
                let mut q = delta;
                let mut k = base;
                loop {
                    let t = if k <= bias {
                        tmin
                    } else if k >= bias + tmax {
                        tmax
                    } else {
                        k - bias
                    };
                    if q < t {
                        break;
                    }
                    output.push(digit(t + (q - t) % (base - t)));
                    q = (q - t) / (base - t);
                    k += base;
                }
                output.push(digit(q));
                bias = adapt(delta, handled + 1, handled == basic);
                delta = 0;
                handled += 1;
            }
        }
        delta += 1;
        n += 1;
    }
    Some(output)
}

// Decodes a Punycode label (RFC 3492). As in the Bun URL parser, a label that
// fails to decode, decodes to nothing but ASCII, or decodes to a control or
// space, is not a valid host. Full IDNA validation (mapping and
// normalization) is not done natively.
fn native_url_punycode(input: &str) -> Option<String> {
    let (base, tmin, tmax, skew, damp) = (36u32, 1u32, 26u32, 38u32, 700u32);
    let adapt = |delta: u32, points: u32, first: bool| {
        let mut delta = if first { delta / damp } else { delta / 2 };
        delta += delta / points;
        let mut k = 0;
        while delta > ((base - tmin) * tmax) / 2 {
            delta /= base - tmin;
            k += base;
        }
        k + (base - tmin + 1) * delta / (delta + skew)
    };
    let (basic, rest) = match input.rfind('-') {
        Some(dash) => (&input[..dash], &input[dash + 1..]),
        None => ("", input),
    };
    let mut output: Vec<char> = basic.chars().collect();
    let (mut n, mut i, mut bias) = (128u32, 0u32, 72u32);
    let mut digits = rest.bytes().peekable();
    while digits.peek().is_some() {
        let (previous, mut weight, mut k) = (i, 1u32, base);
        loop {
            let digit = match digits.next()? {
                b @ b'a'..=b'z' => (b - b'a') as u32,
                b @ b'0'..=b'9' => (b - b'0') as u32 + 26,
                _ => return None,
            };
            i = i.checked_add(digit.checked_mul(weight)?)?;
            let t = if k <= bias {
                tmin
            } else if k >= bias + tmax {
                tmax
            } else {
                k - bias
            };
            if digit < t {
                break;
            }
            weight = weight.checked_mul(base - t)?;
            k += base;
        }
        let points = output.len() as u32 + 1;
        bias = adapt(i - previous, points, previous == 0);
        n = n.checked_add(i / points)?;
        i %= points;
        // A decoded label holds no ASCII, controls or spaces. Other IDNA
        // mappings are not checked, so a valid host is never refused.
        let c =
            char::from_u32(n).filter(|c| !c.is_ascii() && !c.is_control() && !c.is_whitespace())?;
        output.insert(i as usize, c);
        i += 1;
    }
    let decoded: String = output.into_iter().collect();
    (!decoded.is_ascii()).then_some(decoded)
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
                    || matches!(
                        b,
                        b'"' | b'#' | b'<' | b'>' | b'?' | b'^' | b'`' | b'{' | b'}'
                    )
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
            ("http://%C3%A9.com/", Some("http://xn--9ca.com/")),
            ("http://%C3%89.com/", Some("http://xn--9ca.com/")),
            ("http://%EF%BD%81.com/", Some("http://a.com/")),
            ("http://a%E3%80%82b/", Some("http://a.b/")),
            ("http://%E2%98%83.com/", Some("http://xn--n3h.com/")),
            ("http://%F0%9F%92%A9.la/", Some("http://xn--ls8h.la/")),
            ("http://M%C3%BCnchen.DE/", Some("http://xn--mnchen-3ya.de/")),
            (
                "http://%E4%BE%8B%E3%81%88.jp/",
                Some("http://xn--r8jz45g.jp/"),
            ),
            ("http://%E1%8E%A0.com/", Some("http://xn--58d.com/")),
            ("http://%EA%AD%B0.com/", Some("http://xn--58d.com/")),
            ("http://%C3%9F.com/", Some("http://xn--zca.com/")),
            ("http://%CF%82.com/", Some("http://xn--3xa.com/")),
            (
                "http://%E6%97%A5%E6%9C%AC.com/",
                Some("http://xn--wgv71a.com/"),
            ),
            ("http://a%C2%AD.com/", Some("http://a.com/")),
            ("http://a%EF%B8%8F.com/", Some("http://a.com/")),
            ("http://a%E2%80%8B.com/", Some("http://a.com/")),
            ("http://a%CD%8F.com/", Some("http://a.com/")),
            ("http://a%E2%80%8D.com/", None),
            // Compatibility mappings need IDNA tables; they are refused.
            ("http://%EF%AC%83.com/", None),
            ("http://%E2%91%A0.com/", None),
            ("http://%E2%84%8C.com/", None),
            ("http://%F0%9D%90%80.com/", None),
            ("http://%C2%B2.com/", None),
            ("http://%C5%BF.com/", None),
            ("http://%EF%BD%B1.com/", None),
            ("http://%FF.com/", None),
            ("http://%C2%A0.com/", None),
            // Normalization would compose this; it is refused natively.
            ("http://e%CC%81.com/", None),
            ("http://xn--9ca.com/", Some("http://xn--9ca.com/")),
            (
                "http://XN--MNCHEN-3YA.de/",
                Some("http://xn--mnchen-3ya.de/"),
            ),
            ("http://xn--ls8h.la/", Some("http://xn--ls8h.la/")),
            ("http://xn--58d.com/", Some("http://xn--58d.com/")),
            ("http://xn--a.com/", None),
            ("http://xn--.com/", None),
            ("http://xn--abc-.com/", None),
            ("http://xn--zz/", None),
            ("mailto:a@b", Some("other mailto")),
            ("ftp://a/", Some("other ftp")),
            ("ftp://", None),
            ("1http://a/", None),
            ("example.com", None),
            ("http://a/^", Some("http://a/%5E")),
            ("file://[invalid]/", None),
            ("file://a:80/", None),
            ("file://u@a/", None),
            ("file://C:/x", Some("other file")),
            ("file:///x", Some("other file")),
            ("file://host/x", Some("other file")),
            ("foo://a:99999999999999999999/", None),
            ("foo://:1/", None),
            ("foo://a:1/", Some("other foo")),
            ("foo:///x", Some("other foo")),
            ("foo://a\u{7f}b/", Some("other foo")),
            ("foo://a\u{1}b/", Some("other foo")),
            ("foo://a%zzb/", Some("other foo")),
            ("foo://a\0b/", None),
            ("foo://a^b/", None),
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
