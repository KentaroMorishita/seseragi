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
    symbol
        .rsplit_once("::")
        .map(|(_, name)| safe_identifier(name))
        .unwrap_or_else(|| safe_identifier(symbol))
}

#[cfg(test)]
mod tests {
    use super::{local_name, safe_identifier};

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
    }
}
