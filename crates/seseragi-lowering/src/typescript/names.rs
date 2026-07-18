const RESERVED_TYPESCRIPT_WORDS: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

pub(super) fn safe_identifier(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    for (index, char) in name.chars().enumerate() {
        let valid = if index == 0 {
            char == '_' || char == '$' || unicode_ident::is_xid_start(char)
        } else {
            char == '_' || char == '$' || unicode_ident::is_xid_continue(char)
        };
        if valid {
            output.push(char);
        } else if char == '\'' {
            // Apostrophes are valid Seseragi identifier continuations but not
            // JavaScript identifier characters. `$` cannot occur in a source
            // identifier, so this spelling stays distinct from every source
            // name instead of colliding with an underscore replacement.
            output.push_str("$prime");
        } else {
            output.push('_');
        }
    }

    if output.is_empty()
        || output
            .chars()
            .next()
            .is_some_and(|char| char.is_ascii_digit())
        || RESERVED_TYPESCRIPT_WORDS.contains(&output.as_str())
    {
        output.insert(0, '_');
    }

    output
}

pub(super) fn local_name(symbol: &str) -> String {
    if let Some(name) = canonical_operator_name(symbol) {
        return name;
    }
    let relative = symbol.rsplit_once("::").map_or(symbol, |(_, name)| name);
    operator_name(relative).unwrap_or_else(|| safe_identifier(relative))
}

/// Produces a stable backend binding for a value owned by `module`.
/// Top-level values keep their source spelling, while nested semantic values
/// (such as inherent methods) retain their owner path to avoid collisions.
pub(super) fn module_value_name(module: &str, symbol: &str) -> String {
    let relative = symbol
        .strip_prefix(module)
        .and_then(|relative| relative.strip_prefix("::"));
    match relative {
        Some(relative) => {
            if let Some(name) = operator_name(relative) {
                name
            } else if relative.contains("::") {
                safe_identifier(&format!("__ssrg$method${}", relative.replace("::", "$")))
            } else {
                safe_identifier(relative)
            }
        }
        None => local_name(symbol),
    }
}

fn operator_name(relative: &str) -> Option<String> {
    let spelling = relative.strip_prefix("operator(")?.strip_suffix(')')?;
    let encoded = spelling
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Some(format!("__ssrg$operator${encoded}"))
}

fn canonical_operator_name(symbol: &str) -> Option<String> {
    if symbol.starts_with("operator(") {
        return operator_name(symbol);
    }
    let (_, suffix) = symbol.rsplit_once("::operator(")?;
    operator_name(&format!("operator({suffix}"))
}

#[cfg(test)]
mod tests {
    use super::{local_name, module_value_name, safe_identifier};

    #[test]
    fn preserves_simple_identifiers() {
        assert_eq!(safe_identifier("value"), "value");
    }

    #[test]
    fn prefixes_reserved_words() {
        assert_eq!(safe_identifier("default"), "_default");
    }

    #[test]
    fn replaces_invalid_identifier_characters() {
        assert_eq!(safe_identifier("operator(<+>)"), "operator_____");
    }

    #[test]
    fn preserves_unicode_and_encodes_identifier_apostrophes() {
        assert_eq!(safe_identifier("次の値'"), "次の値$prime");
        assert_ne!(safe_identifier("value'"), safe_identifier("value_"));
    }

    #[test]
    fn removes_the_module_qualification_at_the_backend_boundary() {
        assert_eq!(local_name("artifact/example::answer"), "answer");
        assert_eq!(
            local_name("artifact/example::operator(<^>)"),
            "__ssrg$operator$3c5e3e"
        );
        assert_eq!(
            local_name("artifact/example::operator(<::>)"),
            "__ssrg$operator$3c3a3a3e"
        );
    }

    #[test]
    fn retains_nested_value_owners_at_the_backend_boundary() {
        assert_eq!(
            module_value_name("artifact/example", "artifact/example::Box::map"),
            "__ssrg$method$Box$map"
        );
        assert_eq!(
            module_value_name("artifact/example", "artifact/example::answer"),
            "answer"
        );
    }

    #[test]
    fn gives_custom_operators_distinct_stable_backend_names() {
        assert_eq!(
            module_value_name("artifact/example", "artifact/example::operator(<+>)"),
            "__ssrg$operator$3c2b3e"
        );
        assert_eq!(
            module_value_name("artifact/example", "artifact/example::operator(<*>)"),
            "__ssrg$operator$3c2a3e"
        );
    }
}
