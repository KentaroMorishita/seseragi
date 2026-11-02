#[cfg(test)]
mod regex_tests {
    use super::*;

    fn compiled(pattern: &str) -> V {
        let result = native_regex_operation("std/regex::compile", vec![V::text(pattern)]);
        assert!(
            result.tagged("std/prelude::Right"),
            "{pattern}: {}",
            result.render()
        );
        result.payload()
    }

    fn find(pattern: &str, text: &str) -> V {
        native_regex_operation("std/regex::find", vec![compiled(pattern), V::text(text)])
    }

    #[test]
    fn compile_errors_preserve_kinds_payloads_and_utf8_offsets() {
        for (pattern, kind, offset, payload) in [
            ("é[", "UnexpectedRegexEnd", 3, None),
            ("雫🙂)", "UnexpectedRegexToken", 7, Some(")")),
            (
                "(?<名前>a)(?<名前>b)",
                "DuplicateCaptureName",
                15,
                Some("名前"),
            ),
            (
                "(a)\\1",
                "UnsupportedRegexFeature",
                3,
                Some("backreference"),
            ),
            ("a+?", "UnsupportedRegexFeature", 2, Some("lazy quantifier")),
            (
                "a++",
                "UnsupportedRegexFeature",
                2,
                Some("possessive quantifier"),
            ),
            ("(?<=a)", "UnsupportedRegexFeature", 1, Some("look-around")),
            ("a{3,2}", "InvalidRegexQuantifier", 1, None),
            ("a{9007199254740992}", "InvalidRegexQuantifier", 1, None),
            ("\\p{Unknown}", "InvalidRegexEscape", 0, None),
            ("[z-a]", "InvalidRegexRange", 2, None),
            ("[\\d-a]", "InvalidRegexRange", 3, None),
            ("\\u{d800}", "InvalidRegexEscape", 0, None),
            ("\\u{12", "InvalidRegexEscape", 0, None),
            ("\\p{Lu", "UnexpectedRegexEnd", 5, None),
        ] {
            let result = native_regex_operation("std/regex::compile", vec![V::text(pattern)]);
            assert!(result.tagged("std/prelude::Left"), "{pattern}");
            let error = result.payload();
            let actual = error.field("kind");
            assert!(
                actual.tagged(&format!("std/regex::{kind}")),
                "{pattern}: {}",
                actual.render()
            );
            assert!(
                matches!(error.field("offset"), V::Int(actual) if actual == offset),
                "{pattern}"
            );
            match payload {
                Some(value) if kind == "UnexpectedRegexToken" => {
                    assert!(matches!(actual.payload(), V::Char(c) if c.to_string() == value))
                }
                Some(value) => assert_eq!(native_text(&actual.payload()), value),
                None => assert!(matches!(actual.payload(), V::Unit)),
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn ordered_alternatives_and_greedy_repetitions_return_the_first_match() {
        for (pattern, input, expected) in [
            ("a|aa", "xaa", "a"),
            ("aa|a", "xaa", "aa"),
            ("a{2,4}", "xaaaaa", "aaaa"),
            ("a{2,}", "xaaaa", "aaaa"),
            ("(?:ab)?c+", "zabccc", "abccc"),
            ("(a?)*", "aa", "aa"),
            ("", "🙂", ""),
            ("a{0}", "a", ""),
        ] {
            let matched = find(pattern, input);
            assert!(matched.tagged("std/prelude::Just"), "{pattern}");
            assert_eq!(
                native_text(&matched.payload().field("text")),
                expected,
                "{pattern}"
            );
        }
        assert!(find("a{2}", "a").tagged("std/prelude::Nothing"));
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn empty_iterations_preserve_the_last_consuming_captures_and_skip_large_counts() {
        for (pattern, input, expected) in [
            ("(a?)*", "a", Some("a")),
            ("(a?)*", "", None),
            ("(a?)+", "", Some("")),
            ("(a?){1000000000}", "", Some("")),
            ("(){9007199254740991}", "", Some("")),
        ] {
            let matched = find(pattern, input).payload();
            let captured = matched.field("captures").index(0);
            match expected {
                Some(expected) => {
                    assert!(captured.tagged("std/prelude::Just"));
                    assert_eq!(native_text(&captured.payload().field("text")), expected);
                }
                None => assert!(captured.tagged("std/prelude::Nothing")),
            }
        }
        let matched = find("((a)|b)+", "ab").payload();
        assert_eq!(
            native_text(&matched.field("captures").index(0).payload().field("text")),
            "b"
        );
        assert!(matched
            .field("captures")
            .index(1)
            .tagged("std/prelude::Nothing"));
        drop(matched);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn captures_copy_utf8_spans_and_named_entries_keep_declaration_order_and_absence() {
        let matched = find("(?<word>\\w+)(?:-(a+))?", "👍 é́-aa").payload();
        let span = matched.field("span");
        assert!(matches!(span.field("start"), V::Int(5)));
        assert!(matches!(span.field("end"), V::Int(12)));
        let capture = matched.field("captures").index(0).payload();
        assert_eq!(native_text(&capture.field("text")), "é́");
        assert!(matches!(capture.field("span").field("start"), V::Int(5)));
        assert!(matches!(capture.field("span").field("end"), V::Int(9)));
        let absent = find("(?<z>b)?(?<a>a)?(?<m>c)?", "a").payload();
        let entries = native_map_operation("std/map::entries", vec![], vec![absent.field("named")]);
        for (i, name) in ["z", "a", "m"].iter().enumerate() {
            assert_eq!(native_text(&entries.index(i).index(0)), *name);
            assert_eq!(
                entries.index(i).index(1).tagged("std/prelude::Just"),
                i == 1
            );
        }
        drop((matched, span, capture, absent, entries));
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn simple_fold_equivalence_and_properties_use_the_pinned_unicode_tables() {
        let options = NativeRegexOptions {
            case_insensitive: true,
            ..Default::default()
        };
        for (pattern, text, expected) in [
            ("^[A-ZΣ]+$", "aςσ", true),
            ("^[K]+$", "kKK", true),
            ("^[S]+$", "ſsS", true),
            ("^ß$", "ẞ", true),
            ("^ß$", "ss", false),
            ("^I$", "İ", false),
            ("^\\P{Uppercase_Letter}+$", "a", false),
            ("^\\P{Uppercase_Letter}+$", "1", true),
            ("^[^A-Z]+$", "a", false),
        ] {
            let regex = native_regex_compile(options, pattern).payload();
            let result = native_regex_operation("std/regex::isMatch", vec![regex, V::text(text)]);
            assert!(
                matches!(result, V::Bool(actual) if actual == expected),
                "{pattern}: {text}"
            );
        }
        for (pattern, text, expected) in [
            ("^\\w+$", "漢́٣_‿", true),
            ("^\\d+$", "٣", false),
            ("^\\p{Decimal_Number}+$", "٣", true),
            ("^\\s$", "\u{85}", true),
            ("^\\s$", "\u{feff}", false),
            ("^.$", "🙂", true),
        ] {
            let result = native_regex_operation(
                "std/regex::isMatch",
                vec![compiled(pattern), V::text(text)],
            );
            assert!(
                matches!(result, V::Bool(actual) if actual == expected),
                "{pattern}"
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn long_ambiguous_failure_does_not_backtrack_exponentially() {
        let input = format!("{}c", "a".repeat(40_000));
        let result = native_regex_operation(
            "std/regex::isMatch",
            vec![compiled("^(a|aa)*b$"), V::text(&input)],
        );
        assert!(matches!(result, V::Bool(false)));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn escape_round_trips_all_metacharacters_and_control_characters() {
        let literal = "a.^$|?*+()[]{}\\\n\r\t\0🙂";
        let escaped = native_regex_operation("std/regex::escape", vec![V::text(literal)]);
        assert_eq!(
            native_text(&escaped),
            "a\\.\\^\\$\\|\\?\\*\\+\\(\\)\\[\\]\\{\\}\\\\\\n\\r\\t\\u{0}🙂"
        );
        let pattern = format!("\\A{}\\z", native_text(&escaped));
        let regex = compiled(&pattern);
        for text in [literal, ""] {
            let result =
                native_regex_operation("std/regex::isMatch", vec![regex.clone(), V::text(text)]);
            assert!(matches!(result, V::Bool(actual) if actual == (text == literal)));
        }
        drop((regex, escaped));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn compiled_patterns_survive_saved_captures_and_release_reference_cycles() {
        enable_cycle_collection();
        let regex = compiled("雫🙂");
        let cell = V::reference(regex.clone());
        let search = V::fun(vec![regex.clone(), cell.clone()], |captures, input| {
            native_regex_operation("std/regex::find", vec![captures[0].clone(), input])
        });
        drop(regex);
        let matched = search.apply(V::text("x雫🙂")).payload();
        assert_eq!(native_text(&matched.field("text")), "雫🙂");
        cell.write_reference(V::Tuple(Shared::new(vec![cell.clone(), search.clone()])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((cell, search));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(native_text(&matched.field("text")), "雫🙂");
        drop(matched);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
