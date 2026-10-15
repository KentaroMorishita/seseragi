use seseragi_syntax::{
    decode_integer_literal, lex, parse_diagnostics, IntegerLiteralError, TokenKind,
};

#[test]
fn decodes_all_bases_separators_and_safe_boundaries_without_host_rounding() {
    for (raw, value) in [
        ("0", 0),
        ("42", 42),
        ("1_000_000", 1_000_000),
        ("0b1010_0110", 166),
        ("0B1010_0110", 166),
        ("0o755", 493),
        ("0O7_55", 493),
        ("0xFF_00", 65280),
        ("0Xff_00", 65280),
        ("0x0000", 0),
        ("0b0_0", 0),
        ("0o0_0", 0),
    ] {
        assert_eq!(decode_integer_literal(raw), Ok(value), "{raw}");
    }
    for value in [
        0,
        1,
        42,
        65535,
        1_114_111,
        9_007_199_254_740_990,
        9_007_199_254_740_991,
    ] {
        for raw in [
            value.to_string(),
            format!("0b{value:b}"),
            format!("0o{value:o}"),
            format!("0x{value:X}"),
        ] {
            assert_eq!(decode_integer_literal(&raw), Ok(value), "{raw}");
            let source = format!("pub let value: Int = -{raw}\n");
            assert!(
                parse_diagnostics("number.ssrg", &source)
                    .diagnostics
                    .is_empty(),
                "{raw}"
            );
        }
    }
}

#[test]
fn malformed_numbers_are_single_tokens_with_full_source_diagnostics() {
    for raw in [
        "00",
        "01",
        "0_0",
        "1_",
        "1__2",
        "42suffix",
        "42λ",
        "0z12",
        "0b",
        "0B",
        "0b_1",
        "0b2",
        "0b102",
        "0b1__0",
        "0b1_",
        "0o",
        "0O",
        "0o_7",
        "0o8",
        "0o78",
        "0o7__0",
        "0o7_",
        "0x",
        "0X",
        "0x_FF",
        "0xG",
        "0xFFz",
        "0xF__F",
        "0xFF_",
        "0x1.8p+2",
        "0x1.fp-2",
        "0x1p+2",
        "0x1.5",
        "0x1_2suffix",
    ] {
        assert_eq!(
            decode_integer_literal(raw),
            Err(IntegerLiteralError::InvalidSpelling),
            "{raw}"
        );
        let tokens = lex("number.ssrg", raw);
        assert_eq!(tokens.tokens[0].raw, raw, "{raw}");
        assert_eq!(tokens.tokens[0].kind, TokenKind::LiteralInteger, "{raw}");
        assert_eq!(tokens.reconstructed_text(), raw);
        for (prefix, suffix) in [
            ("pub let 値: Int = ", "\n"),
            ("pub let 値: Int = -", "\n"),
            ("let 値 = `${", "}`\n"),
            (
                "pub fn value n: Int -> Int = match n { ",
                " -> 1; _ -> 2 }\n",
            ),
        ] {
            let diagnostics = parse_diagnostics("number.ssrg", &format!("{prefix}{raw}{suffix}"));
            assert_eq!(diagnostics.diagnostics.len(), 1, "{raw}: {diagnostics:#?}");
            let diagnostic = &diagnostics.diagnostics[0];
            assert_eq!(diagnostic.code, "SES-P0203", "{raw}");
            assert_eq!(diagnostic.message_key, "literal.int-invalid", "{raw}");
            assert_eq!(diagnostic.primary.start, prefix.len(), "{raw}");
            assert_eq!(diagnostic.primary.end, prefix.len() + raw.len(), "{raw}");
        }
    }
}

#[test]
fn rejects_safe_range_overflow_in_each_base_including_very_long_tokens() {
    let overflow = 9_007_199_254_740_992_i64;
    for raw in [
        overflow.to_string(),
        format!("0b{overflow:b}"),
        format!("0o{overflow:o}"),
        format!("0x{overflow:X}"),
        format!("0x{}", "F".repeat(1000)),
    ] {
        assert_eq!(
            decode_integer_literal(&raw),
            Err(IntegerLiteralError::OutsideSafeRange)
        );
        let source = format!("pub let value: Int = -{raw}\n");
        let diagnostics = parse_diagnostics("overflow.ssrg", &source);
        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0203");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "literal.int-outside-range"
        );
    }
}

#[test]
fn numeric_tokens_preserve_range_member_operator_and_float_boundaries() {
    let source = "0x1..0x3 0b1..=0o7 0xFF.face 42.member 0x1e+2 -0o52 1e+2 1.0suffix 1e2e3";
    let stream = lex("boundaries.ssrg", source);
    let raw = stream
        .tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::TriviaSpace | TokenKind::Eof))
        .map(|t| t.raw.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        raw,
        [
            "0x1",
            "..",
            "0x3",
            "0b1",
            "..=",
            "0o7",
            "0xFF",
            ".",
            "face",
            "42",
            ".",
            "member",
            "0x1e",
            "+",
            "2",
            "-",
            "0o52",
            "1e+2",
            "1.0suffix",
            "1e2e3"
        ]
    );
    assert_eq!(stream.reconstructed_text(), source);
    for raw in ["1.0suffix", "1e2e3", "1e+", "1e_2"] {
        let diagnostics =
            parse_diagnostics("bad-float.ssrg", &format!("pub let value: Float = {raw}\n"));
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.code == "SES-P0203"),
            "{raw}: {diagnostics:#?}"
        );
    }
}
