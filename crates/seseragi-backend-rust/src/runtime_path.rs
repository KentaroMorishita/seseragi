// Portable paths retain their canonical opaque owner. They are not String
// aliases, and lexical normalization never resolves a symlink.
fn native_text(value: &V) -> &str {
    let V::Text(text) = value else {
        panic!("Rust ABI: expected String")
    };
    text.get()
}
fn native_path(text: &str) -> V {
    V::adt("std/path::Path", V::text(text))
}
fn native_path_text(value: &V) -> &str {
    let V::Adt(path) = value else {
        panic!("Rust ABI: expected Path")
    };
    assert_eq!(path.get().0, "std/path::Path", "Rust ABI: Path owner");
    native_text(&path.get().1)
}
fn native_left(value: V) -> V {
    V::adt("std/prelude::Left", value)
}
fn native_right(value: V) -> V {
    V::adt("std/prelude::Right", value)
}
fn native_maybe(value: Option<V>) -> V {
    match value {
        Some(value) => V::adt("std/prelude::Just", value),
        None => V::adt("std/prelude::Nothing", V::Unit),
    }
}
fn native_path_error(tag: &'static str, payload: V) -> V {
    V::adt(tag, payload)
}
fn native_path_validation(text: &str) -> Option<V> {
    if text.is_empty() {
        return Some(native_path_error("std/path::EmptyPath", V::Unit));
    }
    for (character, name) in [
        ('\0', "std/path::PathContainsNul"),
        ('\\', "std/path::PathContainsBackslash"),
    ] {
        if let Some(index) = text.find(character) {
            let offset = text[..index].encode_utf16().count();
            return Some(native_path_error(
                name,
                V::Record(Shared::new(BTreeMap::from([(
                    "offset".into(),
                    V::Int(offset as i64),
                )]))),
            ));
        }
    }
    let bytes = text.as_bytes();
    if bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && bytes.get(2) != Some(&b'/')
    {
        return Some(native_path_error("std/path::InvalidDriveRoot", V::Unit));
    }
    if let Some(unc) = text.strip_prefix("//") {
        let mut parts = unc.split('/');
        if !(0..2).all(|_| {
            parts
                .next()
                .is_some_and(|p| !p.is_empty() && p != "." && p != "..")
        }) {
            return Some(native_path_error("std/path::InvalidUncRoot", V::Unit));
        }
    }
    None
}
struct NativePathParts<'a> {
    prefix: String,
    segments: Vec<&'a str>,
    relative: bool,
}
fn native_path_parts(text: &str) -> NativePathParts<'_> {
    let (prefix, rest, relative) = if let Some(unc) = text.strip_prefix("//") {
        let mut values = unc.splitn(3, '/');
        let server = values.next().unwrap();
        let share = values.next().unwrap();
        (
            format!("//{server}/{share}/"),
            values.next().unwrap_or(""),
            false,
        )
    } else if text.len() >= 3
        && text.as_bytes()[0].is_ascii_alphabetic()
        && &text.as_bytes()[1..3] == b":/"
    {
        (text[..3].into(), &text[3..], false)
    } else if let Some(rest) = text.strip_prefix('/') {
        ("/".into(), rest, false)
    } else {
        (String::new(), text, true)
    };
    let mut segments: Vec<_> = rest.split('/').collect();
    while segments.last() == Some(&"") {
        segments.pop();
    }
    NativePathParts {
        prefix,
        segments,
        relative,
    }
}
fn native_path_normalize(text: &str) -> String {
    let parts = native_path_parts(text);
    let mut normalized = Vec::new();
    for segment in parts.segments {
        match segment {
            "" | "." => (),
            ".." => {
                if normalized.last().is_some_and(|p| *p != "..") {
                    normalized.pop();
                } else if parts.relative {
                    normalized.push("..");
                }
            }
            other => normalized.push(other),
        }
    }
    if normalized.is_empty() {
        if parts.relative {
            ".".into()
        } else {
            parts.prefix
        }
    } else {
        format!("{}{}", parts.prefix, normalized.join("/"))
    }
}
fn native_path_append(base: &str, child: &str) -> String {
    format!("{}/{child}", base.trim_end_matches('/'))
}
fn native_path_operation(name: &str, args: &[V]) -> V {
    match name {
        "std/path::current" => {
            assert!(matches!(args[0], V::Unit));
            native_path(".")
        }
        "std/path::parse" => {
            let text = native_text(&args[0]);
            native_path_validation(text)
                .map(native_left)
                .unwrap_or_else(|| native_right(native_path(text)))
        }
        "std/path::render" => V::text(native_path_text(&args[0])),
        "std/path::normalize" => native_path(&native_path_normalize(native_path_text(&args[0]))),
        "std/path::isAbsolute" => V::Bool(!native_path_parts(native_path_text(&args[0])).relative),
        "std/path::child" => {
            let child = native_text(&args[0]);
            let base = native_path_text(&args[1]);
            if child.is_empty() || matches!(child, "." | "..") || child.contains(['/', '\\', '\0'])
            {
                native_left(native_path_error(
                    "std/path::InvalidPathSegment",
                    args[0].clone(),
                ))
            } else {
                native_right(native_path(&native_path_append(base, child)))
            }
        }
        "std/path::join" => {
            let child = native_path_text(&args[0]);
            let base = native_path_text(&args[1]);
            if !native_path_parts(child).relative {
                native_left(native_path_error("std/path::AbsoluteChildPath", V::Unit))
            } else if child == "." {
                native_right(args[1].clone())
            } else if base == "." {
                native_right(args[0].clone())
            } else {
                native_right(native_path(&native_path_append(base, child)))
            }
        }
        "std/path::parent" => {
            let text = native_path_text(&args[0]);
            let mut parts = native_path_parts(text);
            if (parts.relative && text == ".") || parts.segments.is_empty() {
                return native_maybe(None);
            }
            parts.segments.pop();
            let parent = if parts.segments.is_empty() {
                if parts.relative {
                    ".".into()
                } else {
                    parts.prefix
                }
            } else {
                format!("{}{}", parts.prefix, parts.segments.join("/"))
            };
            native_maybe(Some(native_path(&parent)))
        }
        "std/path::fileName" | "std/path::extension" => {
            let parts = native_path_parts(native_path_text(&args[0]));
            let file = parts.segments.last().copied();
            let value = if name.ends_with("extension") {
                file.and_then(|s| {
                    s.rfind('.')
                        .filter(|i| *i > 0 && *i + 1 < s.len())
                        .map(|i| &s[i + 1..])
                })
            } else {
                file
            };
            native_maybe(value.map(V::text))
        }
        _ => panic!("Rust ABI: unsupported Path operation"),
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;
    #[test]
    fn portable_roots_normalization_and_error_offsets_keep_the_path_contract() {
        for (input, expected) in [
            ("./a/../b", "b"),
            ("../../a", "../../a"),
            ("/../../a", "/a"),
            ("C:/a/../", "C:/"),
            ("//server/share/a/..", "//server/share/"),
            ("A💡", "A💡"),
        ] {
            let parsed = native_path_operation("std/path::parse", &[V::text(input)]);
            assert!(parsed.tagged("std/prelude::Right"));
            let normalized = native_path_operation("std/path::normalize", &[parsed.payload()]);
            assert_eq!(native_path_text(&normalized), expected);
        }
        let error = native_path_validation("💡\0").unwrap();
        assert!(error.tagged("std/path::PathContainsNul"));
        assert_eq!(error.payload().field("offset").render(), "2");
        let invalid =
            native_path_operation("std/path::child", &[V::text("../file"), native_path(".")]);
        assert!(invalid.tagged("std/prelude::Left"));
        assert!(invalid.payload().tagged("std/path::InvalidPathSegment"));
    }
}
