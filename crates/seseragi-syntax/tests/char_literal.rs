use seseragi_syntax::{decode_char_literal, lex, parse_diagnostics, TokenKind};

#[test]
fn decodes_exact_scalar_and_preserves_raw_tokens() {
    for (literal, expected) in [
        ("'a'", "a"),
        ("'瀬'", "瀬"),
        (r"'\u{03BB}'", "λ"),
        (r"'\0'", "\0"),
        (r"'\''", "'"),
        (r"'\\'", "\\"),
        (r"'\n'", "\n"),
        (r"'\u{10FFFF}'", "\u{10ffff}"),
    ] {
        assert_eq!(decode_char_literal(literal).unwrap(), expected);
        let source = format!("pub let value = {literal}\n");
        let tokens = lex("char.ssrg", &source);
        assert_eq!(tokens.reconstructed_text(), source);
        assert!(tokens
            .tokens
            .iter()
            .any(|t| t.kind == TokenKind::LiteralChar && t.raw == literal));
        assert!(
            parse_diagnostics("char.ssrg", &source)
                .diagnostics
                .is_empty(),
            "{literal}"
        );
    }
}

#[test]
fn char_diagnostics_report_scalar_count_escape_and_unclosed_ranges() {
    for (literal, code, start, end) in [
        ("''", "SES-P0202", 0, 2),
        ("'ab'", "SES-P0202", 0, 4),
        (r"'e\u{0301}'", "SES-P0202", 0, 11),
        (r"'\u{D800}'", "SES-P0201", 1, 9),
        (r"'\u{DFFF}'", "SES-P0201", 1, 9),
        (r"'\u{110000}'", "SES-P0201", 1, 11),
        (r"'\q'", "SES-P0201", 1, 3),
        ("'a", "SES-P0001", 0, 2),
        (r"'\'", "SES-P0001", 0, 3),
    ] {
        let prefix = "let value = ";
        let diagnostics = parse_diagnostics("char.ssrg", &format!("{prefix}{literal}"));
        assert_eq!(
            diagnostics.diagnostics.len(),
            1,
            "{literal}: {diagnostics:#?}"
        );
        let diagnostic = &diagnostics.diagnostics[0];
        assert_eq!(diagnostic.code, code, "{literal}");
        assert_eq!(diagnostic.primary.start, prefix.len() + start, "{literal}");
        assert_eq!(diagnostic.primary.end, prefix.len() + end, "{literal}");
    }
}

#[test]
fn apostrophe_stays_in_identifiers_but_separate_argument_is_char() {
    let tokens = lex("char.ssrg", "account' classify 'a'");
    assert_eq!(tokens.tokens[0].raw, "account'");
    assert_eq!(tokens.tokens[0].kind, TokenKind::IdentifierLower);
    assert_eq!(tokens.tokens[4].kind, TokenKind::LiteralChar);
}

#[test]
fn invalid_char_escapes_are_rejected_inside_and_outside_templates() {
    for literal in [r#"'\"'"#, r"'\`'", r"'\x41'", r"'\u{}'", r"'\u{0000041}'"] {
        let source = format!("let value = {literal}");
        let diagnostics = parse_diagnostics("char.ssrg", &source);
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.code == "SES-P0201"),
            "{diagnostics:#?}"
        );
        let source = format!("let value = `${{{literal}}}`");
        let diagnostics = parse_diagnostics("char.ssrg", &source);
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.code == "SES-P0201"),
            "{source}: {diagnostics:#?}"
        );
    }
}
