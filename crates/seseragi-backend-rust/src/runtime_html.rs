fn native_html_escape(value: &str, attribute: bool) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        output.push_str(match ch {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' if attribute => "&quot;",
            '\'' if attribute => "&#39;",
            _ => {
                output.push(ch);
                continue;
            }
        });
    }
    output
}

fn native_html_value<'a>(value: &'a V, kind: &str) -> &'a str {
    let V::Adt(node) = value else {
        panic!("Rust ABI: expected HTML {kind}")
    };
    assert_eq!(node.get().0, kind, "Rust ABI: HTML value owner");
    native_text(&node.get().1)
}

fn native_html_children(value: &V) -> String {
    match value {
        V::Text(text) => native_html_escape(text.get(), false),
        V::Adt(_) => native_html_value(value, "\0html.node").into(),
        V::Array(values) => values
            .get()
            .iter()
            .map(|v| native_html_value(v, "\0html.node"))
            .collect(),
        V::ListEmpty | V::List(_) => {
            let mut output = String::new();
            let mut cursor = value;
            while let V::List(cell) = cursor {
                output.push_str(native_html_value(&cell.get().head, "\0html.node"));
                cursor = &cell.get().tail;
            }
            assert!(matches!(cursor, V::ListEmpty), "Rust ABI: HTML child List");
            output
        }
        _ => panic!("Rust ABI: unsupported HTML children"),
    }
}

fn native_html_kebab(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        if ch.is_ascii_uppercase() {
            output.push('-');
            output.push(ch.to_ascii_lowercase());
        } else {
            output.push(ch);
        }
    }
    output
}

fn native_html_style(value: V) -> V {
    let mut properties = Vec::new();
    for (name, value) in value.style_fields() {
        if name == "variables" {
            for (name, value) in value.style_fields() {
                properties.push(format!(
                    "--{}: {}",
                    native_html_kebab(name),
                    native_text(value)
                ));
            }
        } else {
            properties.push(format!(
                "{}: {}",
                native_html_kebab(name),
                native_text(value)
            ));
        }
    }
    V::adt("\0html.style", V::text(&properties.join("; ")))
}

fn native_html_attribute(
    output: &mut String,
    props: &BTreeMap<String, V>,
    property: &str,
    name: &str,
    kind: &str,
    default: Option<&str>,
) {
    let value = props.get(property);
    let rendered = match (kind, value) {
        ("bool", Some(V::Bool(true))) => {
            output.push(' ');
            output.push_str(name);
            return;
        }
        ("bool", Some(V::Bool(false)) | None) => return,
        ("bool", _) => panic!("Rust ABI: HTML boolean attribute"),
        ("enum", Some(V::Bool(value))) => value.to_string(),
        ("int", Some(V::Int(value))) => value.to_string(),
        ("url", Some(value)) => native_html_value(value, "\0html.url").into(),
        ("style", Some(value)) => native_html_value(value, "\0html.style").into(),
        ("string", Some(value)) => native_text(value).into(),
        (_, None) => match default {
            Some(value) => value.into(),
            None => return,
        },
        _ => panic!("Rust ABI: HTML attribute type"),
    };
    output.push_str(&format!(
        " {name}=\"{}\"",
        native_html_escape(&rendered, true)
    ));
}

fn native_html_element(tag: &str, value: V, void: bool) -> V {
    let props = value.record_fields();
    assert!(
        !void || !props.contains_key("children"),
        "Rust ABI: void HTML children"
    );
    let mut output = format!("<{tag}");
    for (property, name, kind) in [
        ("id", "id", "string"),
        ("class", "class", "string"),
        ("title", "title", "string"),
        ("hidden", "hidden", "bool"),
        ("style", "style", "style"),
        ("role", "role", "string"),
        ("tabIndex", "tabindex", "int"),
        ("lang", "lang", "string"),
        ("dir", "dir", "string"),
        ("draggable", "draggable", "enum"),
        ("contentEditable", "contenteditable", "enum"),
        ("inert", "inert", "bool"),
        ("ariaLabel", "aria-label", "string"),
        ("ariaBusy", "aria-busy", "enum"),
        ("ariaExpanded", "aria-expanded", "enum"),
        ("ariaHidden", "aria-hidden", "enum"),
        ("ariaSelected", "aria-selected", "enum"),
    ] {
        native_html_attribute(&mut output, props, property, name, kind, None);
    }
    let attributes: &[(&str, &str, &str, Option<&str>)] = match tag {
        "button" => &[
            ("disabled", "disabled", "bool", None),
            ("buttonType", "type", "string", Some("button")),
            ("name", "name", "string", None),
            ("value", "value", "string", None),
            ("autoFocus", "autofocus", "bool", None),
        ],
        "form" => &[
            ("name", "name", "string", None),
            ("autoComplete", "autocomplete", "string", None),
        ],
        "input" => &[
            ("value", "value", "string", None),
            ("checked", "checked", "bool", None),
            ("name", "name", "string", None),
            ("disabled", "disabled", "bool", None),
            ("required", "required", "bool", None),
            ("readOnly", "readonly", "bool", None),
            ("multiple", "multiple", "bool", None),
            ("placeholder", "placeholder", "string", None),
            ("autoComplete", "autocomplete", "string", None),
            ("autoFocus", "autofocus", "bool", None),
            ("min", "min", "string", None),
            ("max", "max", "string", None),
            ("step", "step", "string", None),
            ("pattern", "pattern", "string", None),
            ("inputType", "type", "string", Some("text")),
        ],
        "textarea" => &[
            ("name", "name", "string", None),
            ("disabled", "disabled", "bool", None),
            ("required", "required", "bool", None),
            ("readOnly", "readonly", "bool", None),
            ("placeholder", "placeholder", "string", None),
            ("autoComplete", "autocomplete", "string", None),
            ("autoFocus", "autofocus", "bool", None),
            ("rows", "rows", "int", None),
            ("cols", "cols", "int", None),
        ],
        "label" => &[("htmlFor", "for", "string", None)],
        "a" => &[
            ("href", "href", "url", None),
            ("target", "target", "string", None),
            ("rel", "rel", "string", None),
            ("download", "download", "bool", None),
        ],
        "img" => &[
            ("src", "src", "url", None),
            ("alt", "alt", "string", None),
            ("width", "width", "int", None),
            ("height", "height", "int", None),
            ("loading", "loading", "string", None),
        ],
        "source" => &[
            ("src", "src", "url", None),
            ("media", "media", "string", None),
            ("mimeType", "type", "string", None),
        ],
        "audio" => &[("src", "src", "url", None)],
        "video" => &[
            ("src", "src", "url", None),
            ("width", "width", "int", None),
            ("height", "height", "int", None),
        ],
        "meta" => &[
            ("charSet", "charset", "string", None),
            ("name", "name", "string", None),
            ("content", "content", "string", None),
            ("httpEquiv", "http-equiv", "string", None),
        ],
        "link" => &[
            ("rel", "rel", "string", None),
            ("href", "href", "url", None),
        ],
        "select" => &[
            ("name", "name", "string", None),
            ("value", "value", "string", None),
            ("disabled", "disabled", "bool", None),
            ("required", "required", "bool", None),
            ("multiple", "multiple", "bool", None),
            ("autoFocus", "autofocus", "bool", None),
        ],
        "option" => &[
            ("value", "value", "string", None),
            ("selected", "selected", "bool", None),
            ("disabled", "disabled", "bool", None),
        ],
        "th" | "td" => &[
            ("colSpan", "colspan", "int", None),
            ("rowSpan", "rowspan", "int", None),
        ],
        "details" | "dialog" => &[("open", "open", "bool", None)],
        _ => &[],
    };
    for &(property, name, kind, default) in attributes {
        native_html_attribute(&mut output, props, property, name, kind, default);
    }
    if let Some(value) = props.get("attributes") {
        let V::Array(attributes) = value else {
            panic!("Rust ABI: HTML custom attributes Array")
        };
        let mut names = std::collections::BTreeSet::new();
        for value in attributes.get() {
            let V::Adt(attribute) = value else {
                panic!("Rust ABI: HTML custom attribute")
            };
            assert_eq!(
                attribute.get().0,
                "\0html.attribute",
                "Rust ABI: HTML attribute owner"
            );
            let name = attribute.get().1.field("name");
            let name = native_text(&name);
            assert!(
                names.insert(name.to_ascii_lowercase()),
                "Rust ABI: duplicate HTML attribute"
            );
            output.push_str(&format!(
                " {name}=\"{}\"",
                native_html_escape(native_text(&attribute.get().1.field("value")), true)
            ));
        }
    }
    output.push('>');
    if !void {
        if tag == "textarea" {
            if let Some(value) = props.get("value") {
                output.push_str(&native_html_escape(native_text(value), false));
            }
        } else if let Some(children) = props.get("children") {
            output.push_str(&native_html_children(children));
        }
        output.push_str(&format!("</{tag}>"));
    }
    V::adt("\0html.node", V::text(&output))
}

fn native_html_operation(name: &str, value: V, void: bool) -> V {
    match name {
        "style" => native_html_style(value),
        "parseWebUrl" => native_html_parse_url(value),
        "text" => V::adt(
            "\0html.node",
            V::text(&native_html_escape(native_text(&value), false)),
        ),
        "fragment" => V::adt("\0html.node", V::text(&native_html_children(&value))),
        "renderToString" => V::text(native_html_value(&value, "\0html.node")),
        "renderDocument" => V::text(&format!(
            "<!doctype html>{}",
            native_html_value(&value, "\0html.node")
        )),
        tag => native_html_element(tag, value, void),
    }
}

fn native_html_parse_url(value: V) -> V {
    if native_html_safe_url(native_text(&value)) {
        native_right(V::adt("\0html.url", value))
    } else {
        native_left(V::adt("std/web/html::UnsafeWebUrlScheme", value))
    }
}

fn native_html_safe_url(value: &str) -> bool {
    if value.chars().any(|ch| ch.is_ascii_control()) {
        return false;
    }
    let value = value.trim_matches(' ');
    let scheme_end = value.find([':', '/', '?', '#', '\\']);
    let (protocol, rest) = match scheme_end {
        Some(end) if value.as_bytes()[end] == b':' => {
            let scheme = &value[..end];
            if scheme.starts_with(|ch: char| ch.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
            {
                (scheme.to_ascii_lowercase(), &value[end + 1..])
            } else {
                return true;
            }
        }
        _ => ("https".into(), value),
    };
    if !matches!(protocol.as_str(), "http" | "https" | "mailto" | "tel") {
        return false;
    }
    if matches!(protocol.as_str(), "mailto" | "tel") && !rest.starts_with("//") {
        return true;
    }
    let explicit = scheme_end.is_some_and(|end| value.as_bytes()[end] == b':');
    if protocol == "https"
        && explicit
        && rest
            .chars()
            .take_while(|ch| matches!(ch, '/' | '\\'))
            .count()
            < 2
    {
        return true;
    }
    let special = matches!(protocol.as_str(), "http" | "https");
    let authority = if !special {
        &rest[2..]
    } else if rest.starts_with(['/', '\\']) {
        let count = rest
            .chars()
            .take_while(|ch| matches!(ch, '/' | '\\'))
            .count();
        if count < 2 && !explicit {
            return true;
        }
        &rest[count..]
    } else if explicit {
        rest
    } else {
        return true;
    };
    let authority = authority
        .split(|ch| matches!(ch, '/' | '?' | '#') || (special && ch == '\\'))
        .next()
        .unwrap();
    let authority = if let Some((credentials, host)) = authority.rsplit_once('@') {
        if !credentials.is_empty() && credentials != ":" {
            return false;
        }
        host
    } else {
        authority
    };
    if authority.is_empty() {
        return !special;
    }
    let (host, port) = if authority.starts_with('[') {
        let Some(end) = authority.find(']') else {
            return false;
        };
        if authority[1..end].parse::<std::net::Ipv6Addr>().is_err() {
            return false;
        }
        let port = &authority[end + 1..];
        if !port.is_empty() && !port.starts_with(':') {
            return false;
        }
        (&authority[..=end], port.strip_prefix(':'))
    } else {
        authority
            .rsplit_once(':')
            .map_or((authority, None), |(host, port)| (host, Some(port)))
    };
    if port.is_some_and(|p| {
        let digits = p.trim_start_matches('0');
        !p.bytes().all(|b| b.is_ascii_digit())
            || (!digits.is_empty() && digits.parse::<u16>().is_err())
    }) {
        return false;
    }
    if host.is_empty()
        || host.chars().any(|ch| {
            matches!(ch, ' ' | '<' | '>' | '^' | '|' | '[' | ']' | '\\')
                && !authority.starts_with('[')
        })
    {
        return false;
    }
    if special {
        native_html_url_host(host)
    } else {
        !host
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '<' | '>' | '^' | '|'))
    }
}

fn native_html_url_host(host: &str) -> bool {
    if host.starts_with('[') {
        return true;
    }
    let mut decoded = Vec::new();
    let mut bytes = host.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let Some(high) = bytes.next().and_then(|b| (b as char).to_digit(16)) else {
                return false;
            };
            let Some(low) = bytes.next().and_then(|b| (b as char).to_digit(16)) else {
                return false;
            };
            decoded.push((high * 16 + low) as u8);
        } else {
            decoded.push(byte);
        }
    }
    let Ok(host) = String::from_utf8(decoded) else {
        return false;
    };
    if host.chars().any(|ch| {
        ch.is_whitespace()
            || ch.is_ascii_control()
            || matches!(
                ch,
                '#' | '%' | '/' | ':' | '<' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '|'
            )
    }) {
        return false;
    }
    let parts: Vec<_> = host.strip_suffix('.').unwrap_or(&host).split('.').collect();
    // Internationalized hosts require IDNA validation; do not accept unchecked labels.
    if !host.is_ascii()
        || parts
            .iter()
            .any(|part| part.to_ascii_lowercase().starts_with("xn--"))
    {
        return false;
    }
    fn number(part: &str) -> Option<u64> {
        if part.is_empty() {
            return None;
        }
        let (digits, radix) = if part.starts_with("0x") || part.starts_with("0X") {
            (&part[2..], 16)
        } else if part.len() >= 2 && part.starts_with('0') {
            (&part[1..], 8)
        } else {
            (part, 10)
        };
        if digits.is_empty() {
            Some(0)
        } else {
            u64::from_str_radix(digits, radix).ok()
        }
    }
    let last = parts.last().unwrap();
    let hex = last.strip_prefix("0x").or_else(|| last.strip_prefix("0X"));
    let numeric = number(last).is_some()
        || hex.is_some_and(|digits| digits.bytes().all(|b| b.is_ascii_hexdigit()))
        || (!last.is_empty() && last.bytes().all(|b| b.is_ascii_digit()));
    if numeric {
        if parts.len() > 4 || parts.iter().any(|part| part.is_empty()) {
            return false;
        }
        for (index, part) in parts.iter().enumerate() {
            let Some(value) = number(part) else {
                return false;
            };
            let limit = if index + 1 == parts.len() {
                1u64 << (8 * (5 - parts.len()))
            } else {
                256
            };
            if value >= limit {
                return false;
            }
        }
    }
    true
}
