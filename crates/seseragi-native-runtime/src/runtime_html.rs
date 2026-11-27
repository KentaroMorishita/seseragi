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

// HTML and SVG nodes are trees until `renderToString` or `renderDocument`
// writes them, as on TS, so a shared subtree stays shared. A node is
// `\0html.node` (or `\0svg.node` before `toHtml`) around one of:
// - `\0html.text`: the unescaped text;
// - `\0html.element`: (namespace, tag, props, children, void);
// - `\0html.fragment`: the children.
// Children are kept without their node wrapper. Building checks only what the
// TS constructors check; attributes are checked when they are written.

fn native_html_tree<'a>(value: &'a V, kind: &str) -> Option<&'a V> {
    match value {
        V::Adt(node) if node.get().0 == kind => Some(&node.get().1),
        _ => None,
    }
}

fn native_html_text_tree(value: &str) -> V {
    V::adt("\0html.text", V::text(value))
}

/// The children TS's `normalizeChildren` accepts: text, one node, or an Array
/// or a List of nodes.
fn native_html_children(value: Option<&V>) -> Vec<V> {
    let node = |value: &V, message: &str| -> V {
        native_html_tree(value, "\0html.node")
            .unwrap_or_else(|| panic!("{message}"))
            .clone()
    };
    match value {
        // Unit is `undefined` and a Char is a string on TS.
        None | Some(V::Unit) => Vec::new(),
        Some(V::Text(text)) => vec![native_html_text_tree(text.get())],
        Some(V::Char(ch)) => vec![native_html_text_tree(&ch.to_string())],
        Some(value @ V::Adt(_)) => vec![node(value, "unsupported HTML children value")],
        Some(V::Array(values)) => values
            .get()
            .iter()
            .map(|value| node(value, "HTML child arrays may contain only Html values"))
            .collect(),
        Some(mut cursor @ (V::ListEmpty | V::List(_))) => {
            let mut children = Vec::new();
            while let V::List(cell) = cursor {
                children.push(node(
                    &cell.get().head,
                    "HTML child lists may contain only Html values",
                ));
                cursor = &cell.get().tail;
            }
            children
        }
        Some(_) => panic!("unsupported HTML children value"),
    }
}

/// The children TS's SVG `unwrapChildren` accepts: text, one scene node, or an
/// Array of them. A List is refused when the node is built.
fn native_svg_children(value: Option<&V>) -> Vec<V> {
    let node = |value: &V, message: &str| -> V {
        native_html_tree(value, "\0svg.node")
            .unwrap_or_else(|| panic!("{message}"))
            .clone()
    };
    match value {
        // Unit is `undefined` and a Char is a string on TS.
        None | Some(V::Unit) => Vec::new(),
        Some(V::Text(text)) => vec![native_html_text_tree(text.get())],
        Some(V::Char(ch)) => vec![native_html_text_tree(&ch.to_string())],
        Some(value @ V::Adt(_)) => vec![node(value, "unsupported SVG children value")],
        Some(V::Array(values)) => values
            .get()
            .iter()
            .map(|value| node(value, "SVG child arrays may contain only Svg values"))
            .collect(),
        Some(_) => panic!("unsupported SVG children value"),
    }
}

fn native_html_element_tree(
    namespace: &str,
    tag: &str,
    props: V,
    children: Vec<V>,
    void: bool,
) -> V {
    V::adt(
        "\0html.element",
        V::Tuple(Shared::new(vec![
            V::text(namespace),
            V::text(tag),
            props,
            V::Array(Shared::new(children)),
            V::Bool(void),
        ])),
    )
}

fn native_html_element(tag: &str, value: V, void: bool) -> V {
    let fields = value.record_fields();
    if void && fields.contains_key("children") {
        panic!("void HTML element {tag} cannot have children");
    }
    let children = if void {
        Vec::new()
    } else if tag == "textarea" {
        // TS gives a textarea its value, or "", as its children.
        match fields.get("value") {
            None | Some(V::Unit) => vec![native_html_text_tree("")],
            value => native_html_children(value),
        }
    } else {
        native_html_children(fields.get("children"))
    };
    V::adt(
        "\0html.node",
        native_html_element_tree("html", tag, value, children, void),
    )
}

fn native_svg_element(tag: &str, value: V) -> V {
    let children = native_svg_children(value.record_fields().get("children"));
    V::adt(
        "\0svg.node",
        native_html_element_tree("svg", tag, value, children, false),
    )
}

/// One attribute, checked and written as the TS renderer does.
fn native_html_attribute(
    output: &mut String,
    names: &mut std::collections::BTreeSet<String>,
    name: &str,
    value: Option<&V>,
    kind: &str,
) {
    // Unit is `undefined` on TS, so the attribute is left out.
    let Some(value) = value.filter(|value| !matches!(value, V::Unit)) else {
        return;
    };
    names.insert(name.to_ascii_lowercase());
    let text = match kind {
        "bool" => {
            if matches!(value, V::Bool(true)) {
                output.push(' ');
                output.push_str(name);
            }
            return;
        }
        "string" => match value {
            V::Text(text) => text.get().clone(),
            V::Char(ch) => ch.to_string(),
            _ => panic!("HTML attribute {name} must be a string"),
        },
        "enum" => match value {
            V::Bool(value) => value.to_string(),
            _ => panic!("HTML attribute {name} must be a boolean"),
        },
        "int" => match value {
            V::Int(value) => value.to_string(),
            V::Byte(value) => value.to_string(),
            V::Float(value) if value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_991.0 => {
                native_number_text(*value)
            }
            _ => panic!("HTML attribute {name} must be an Int"),
        },
        "number" => match value {
            V::Int(value) => value.to_string(),
            V::Byte(value) => value.to_string(),
            V::Float(value) if value.is_finite() => native_number_text(*value),
            _ => panic!("SVG attribute {name} must be a finite number"),
        },
        "style" => match native_html_tree(value, "\0html.style") {
            Some(css) => native_text(css).to_owned(),
            None => panic!("HTML style must be created with html.style"),
        },
        "url" => match native_html_tree(value, "\0html.url") {
            Some(url) => native_text(url).to_owned(),
            None => panic!("HTML URL attribute {name} must be created with html.parseWebUrl"),
        },
        _ => unreachable!("Rust ABI: HTML attribute kind"),
    };
    output.push_str(&format!(" {name}=\"{}\"", native_html_escape(&text, true)));
}

/// Every attribute of an element, in the TS renderer's order.
fn native_html_attributes(
    output: &mut String,
    namespace: &str,
    tag: &str,
    props: &BTreeMap<String, V>,
) {
    let mut names = std::collections::BTreeSet::new();
    let mut write = |property: &str, name: &str, kind: &str| {
        native_html_attribute(output, &mut names, name, props.get(property), kind);
    };
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
        write(property, name, kind);
    }
    if namespace == "svg" {
        for (property, name) in [
            ("viewBox", "viewBox"),
            ("preserveAspectRatio", "preserveAspectRatio"),
            ("transform", "transform"),
            ("d", "d"),
            ("points", "points"),
            ("fill", "fill"),
            ("stroke", "stroke"),
            ("strokeWidth", "stroke-width"),
            ("strokeLinecap", "stroke-linecap"),
            ("strokeLinejoin", "stroke-linejoin"),
            ("textAnchor", "text-anchor"),
            ("dominantBaseline", "dominant-baseline"),
            ("pointerEvents", "pointer-events"),
        ] {
            write(property, name, "string");
        }
        for name in [
            "x", "y", "x1", "y1", "x2", "y2", "cx", "cy", "r", "rx", "ry", "width", "height",
            "opacity",
        ] {
            write(name, name, "number");
        }
    }
    let attributes: &[(&str, &str, &str)] = match tag {
        "button" => &[
            ("disabled", "disabled", "bool"),
            ("buttonType", "type", "string"),
            ("name", "name", "string"),
            ("value", "value", "string"),
            ("autoFocus", "autofocus", "bool"),
        ],
        "form" => &[
            ("name", "name", "string"),
            ("autoComplete", "autocomplete", "string"),
        ],
        "input" => &[
            ("value", "value", "string"),
            ("checked", "checked", "bool"),
            ("name", "name", "string"),
            ("disabled", "disabled", "bool"),
            ("required", "required", "bool"),
            ("readOnly", "readonly", "bool"),
            ("multiple", "multiple", "bool"),
            ("placeholder", "placeholder", "string"),
            ("autoComplete", "autocomplete", "string"),
            ("autoFocus", "autofocus", "bool"),
            ("min", "min", "string"),
            ("max", "max", "string"),
            ("step", "step", "string"),
            ("pattern", "pattern", "string"),
            ("inputType", "type", "string"),
        ],
        "textarea" => &[
            ("name", "name", "string"),
            ("disabled", "disabled", "bool"),
            ("required", "required", "bool"),
            ("readOnly", "readonly", "bool"),
            ("placeholder", "placeholder", "string"),
            ("autoComplete", "autocomplete", "string"),
            ("autoFocus", "autofocus", "bool"),
            ("rows", "rows", "int"),
            ("cols", "cols", "int"),
        ],
        "label" => &[("htmlFor", "for", "string")],
        "a" => &[
            ("href", "href", "url"),
            ("target", "target", "string"),
            ("rel", "rel", "string"),
            ("download", "download", "bool"),
        ],
        "img" => &[
            ("src", "src", "url"),
            ("alt", "alt", "string"),
            ("width", "width", "int"),
            ("height", "height", "int"),
            ("loading", "loading", "string"),
        ],
        "source" => &[
            ("src", "src", "url"),
            ("media", "media", "string"),
            ("mimeType", "type", "string"),
        ],
        "audio" => &[("src", "src", "url")],
        "video" => &[
            ("src", "src", "url"),
            ("width", "width", "int"),
            ("height", "height", "int"),
        ],
        "meta" => &[
            ("charSet", "charset", "string"),
            ("name", "name", "string"),
            ("content", "content", "string"),
            ("httpEquiv", "http-equiv", "string"),
        ],
        "link" => &[("rel", "rel", "string"), ("href", "href", "url")],
        "select" => &[
            ("name", "name", "string"),
            ("value", "value", "string"),
            ("disabled", "disabled", "bool"),
            ("required", "required", "bool"),
            ("multiple", "multiple", "bool"),
            ("autoFocus", "autofocus", "bool"),
        ],
        "option" => &[
            ("value", "value", "string"),
            ("selected", "selected", "bool"),
            ("disabled", "disabled", "bool"),
        ],
        "th" | "td" => &[("colSpan", "colspan", "int"), ("rowSpan", "rowspan", "int")],
        "details" | "dialog" => &[("open", "open", "bool")],
        _ => &[],
    };
    for &(property, name, kind) in attributes {
        // `buttonType` and `inputType` fall back as on TS.
        let fallback = match (tag, property) {
            ("button", "buttonType") => Some("button"),
            ("input", "inputType") => Some("text"),
            _ => None,
        };
        let value = props
            .get(property)
            .filter(|value| !matches!(value, V::Unit));
        match (value, fallback) {
            (None, Some(fallback)) => {
                native_html_attribute(output, &mut names, name, Some(&V::text(fallback)), kind)
            }
            (value, _) => native_html_attribute(output, &mut names, name, value, kind),
        }
    }
    match props.get("attributes") {
        None | Some(V::Unit) => {}
        Some(V::Array(items)) => {
            for item in items.get() {
                let Some(attribute) = native_html_tree(item, "\0html.attribute") else {
                    panic!("HTML custom attributes must be created with html.attribute")
                };
                let name = attribute.field("name");
                let name = native_text(&name);
                if !names.insert(name.to_ascii_lowercase()) {
                    panic!("duplicate HTML custom attribute {name}");
                }
                output.push_str(&format!(
                    " {name}=\"{}\"",
                    native_html_escape(native_text(&attribute.field("value")), true)
                ));
            }
        }
        Some(_) => panic!("HTML custom attributes must be an Array"),
    }
}

/// Writes a tree depth first, with an explicit stack so a deep tree cannot
/// exhaust the native stack.
fn native_html_render(tree: &V) -> String {
    enum Step<'a> {
        Tree(&'a V),
        Close(&'a str),
    }
    let mut output = String::new();
    let mut steps = vec![Step::Tree(tree)];
    while let Some(step) = steps.pop() {
        let tree = match step {
            Step::Close(tag) => {
                output.push_str("</");
                output.push_str(tag);
                output.push('>');
                continue;
            }
            Step::Tree(tree) => tree,
        };
        let V::Adt(node) = tree else {
            panic!("Rust ABI: HTML tree")
        };
        match node.get().0 {
            "\0html.text" => {
                output.push_str(&native_html_escape(native_text(&node.get().1), false))
            }
            "\0html.fragment" => {
                let V::Array(children) = &node.get().1 else {
                    panic!("Rust ABI: HTML fragment")
                };
                steps.extend(children.get().iter().rev().map(Step::Tree));
            }
            "\0html.element" => {
                let V::Tuple(parts) = &node.get().1 else {
                    panic!("Rust ABI: HTML element")
                };
                let [namespace, tag, props, V::Array(children), V::Bool(void)] =
                    parts.get().as_slice()
                else {
                    panic!("Rust ABI: HTML element parts")
                };
                let tag = native_text(tag);
                output.push('<');
                output.push_str(tag);
                native_html_attributes(
                    &mut output,
                    native_text(namespace),
                    tag,
                    props.record_fields(),
                );
                output.push('>');
                if !*void {
                    steps.push(Step::Close(tag));
                    steps.extend(children.get().iter().rev().map(Step::Tree));
                }
            }
            _ => panic!("Rust ABI: HTML tree kind"),
        }
    }
    output
}

const NATIVE_HTML_RESERVED_ATTRIBUTES: [&str; 56] = [
    "alt",
    "attributes",
    "autocomplete",
    "autofocus",
    "buttontype",
    "checked",
    "children",
    "class",
    "classname",
    "cols",
    "colspan",
    "contenteditable",
    "dir",
    "disabled",
    "download",
    "draggable",
    "for",
    "height",
    "hidden",
    "htmlfor",
    "href",
    "id",
    "inputtype",
    "key",
    "lang",
    "loading",
    "max",
    "media",
    "min",
    "mimetype",
    "multiple",
    "name",
    "charset",
    "content",
    "http-equiv",
    "open",
    "pattern",
    "placeholder",
    "readonly",
    "rel",
    "elementref",
    "required",
    "role",
    "rows",
    "rowspan",
    "selected",
    "src",
    "step",
    "style",
    "tabindex",
    "target",
    "title",
    "type",
    "value",
    "width",
    "srcdoc",
];

const NATIVE_HTML_URL_ATTRIBUTES: [&str; 16] = [
    "action",
    "background",
    "cite",
    "classid",
    "codebase",
    "data",
    "formaction",
    "href",
    "icon",
    "longdesc",
    "manifest",
    "poster",
    "profile",
    "src",
    "usemap",
    "xlink:href",
];

/// `html.attribute name value`, checked as on TS.
pub fn native_html_custom_attribute(name: V, value: V) -> V {
    let text = native_text(&name);
    let mut chars = text.chars();
    let valid = chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == ':')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | ':' | '-'));
    let normalized = text.to_ascii_lowercase();
    if !valid
        || ((normalized.starts_with("data-") || normalized.starts_with("aria-"))
            && (text != normalized || normalized.len() <= 5))
    {
        return native_left(V::adt("std/web/html::InvalidAttributeName", name));
    }
    if normalized.starts_with("on")
        || normalized == "data-ssrg"
        || normalized.starts_with("data-ssrg-")
        || NATIVE_HTML_RESERVED_ATTRIBUTES.contains(&normalized.as_str())
    {
        return native_left(V::adt("std/web/html::ReservedAttributeName", name));
    }
    if NATIVE_HTML_URL_ATTRIBUTES.contains(&normalized.as_str()) {
        let parsed = native_html_parse_url(value.clone());
        if parsed.tagged("std/prelude::Left") {
            return parsed;
        }
    }
    native_right(V::adt(
        "\0html.attribute",
        V::Record(Shared::new(BTreeMap::from([
            ("name".to_owned(), name),
            ("value".to_owned(), value),
        ]))),
    ))
}

pub fn native_html_operation(name: &str, value: V, void: bool) -> V {
    match name {
        "std/web/svg::toHtml" => match native_html_tree(&value, "\0svg.node") {
            Some(tree) => V::adt("\0html.node", tree.clone()),
            None => panic!("expected std/web/svg Svg"),
        },
        name if name.starts_with("std/web/svg::") => {
            native_svg_element(&name["std/web/svg::".len()..], value)
        }
        "style" => native_html_style(value),
        "parseWebUrl" => native_html_parse_url(value),
        "text" => V::adt("\0html.node", native_html_text_tree(native_text(&value))),
        "fragment" => V::adt(
            "\0html.node",
            V::adt(
                "\0html.fragment",
                V::Array(Shared::new(native_html_children(Some(&value)))),
            ),
        ),
        "renderToString" | "renderDocument" => {
            let Some(tree) = native_html_tree(&value, "\0html.node") else {
                panic!("Rust ABI: expected HTML node")
            };
            let text = native_html_render(tree);
            V::text(&if name == "renderDocument" {
                format!("<!doctype html>{text}")
            } else {
                text
            })
        }
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
    let Some(domain) = native_url_host(host) else {
        return false;
    };
    if domain.starts_with('[') {
        return true;
    }
    domain
        .split('.')
        .all(|label| match label.strip_prefix("xn--") {
            None => true,
            Some(encoded) => native_url_punycode(encoded)
                .and_then(|decoded| native_url_domain_to_ascii(&decoded))
                .is_some_and(|roundtrip| roundtrip == label),
        })
}
