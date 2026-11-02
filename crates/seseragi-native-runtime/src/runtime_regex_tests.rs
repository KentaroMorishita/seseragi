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

    fn strings(value: &V) -> Vec<String> {
        let V::Array(values) = value else { panic!() };
        values.get().iter().map(|v| native_text(v).to_owned()).collect()
    }

    #[test]
    fn all_matches_advance_by_scalars_keep_absolute_anchors_and_include_the_end() {
        for (pattern, text, expected) in [
            ("", "", vec![(0, 0, "")]),
            ("", "a🙂é", vec![(0, 0, ""), (1, 1, ""), (5, 5, ""), (6, 6, ""), (8, 8, "")]),
            ("a*", "aa🙂a", vec![(0, 2, "aa"), (2, 2, ""), (6, 7, "a"), (7, 7, "")]),
            ("aa", "aaa", vec![(0, 2, "aa")]),
            ("a|aa", "xaa", vec![(1, 2, "a"), (2, 3, "a")]),
            ("aa|a", "xaa", vec![(1, 3, "aa")]),
            ("^a|^b", "ab", vec![(0, 1, "a")]),
            ("\\A|\\z", "雫🙂", vec![(0, 0, ""), (7, 7, "")]),
            ("z", "雫🙂", vec![]),
        ] {
            let result = native_regex_operation("std/regex::findAll", vec![compiled(pattern), V::text(text)]);
            let V::Array(values) = &result else { panic!() };
            let actual: Vec<_> = values.get().iter().map(|hit| {
                let V::Int(start) = hit.field("span").field("start") else { panic!() };
                let V::Int(end) = hit.field("span").field("end") else { panic!() };
                (start, end, native_text(&hit.field("text")).to_owned())
            }).collect();
            let expected: Vec<_> = expected.into_iter().map(|(start, end, text)| (start, end, text.to_owned())).collect();
            assert_eq!(actual, expected, "{pattern:?}/{text:?}");
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn all_matches_reset_captures_and_keep_named_absence_and_utf8_spans() {
        let result = native_regex_operation("std/regex::findAll", vec![compiled("(?<word>a(b)?)(?<empty>)|(?<other>雫)"), V::text("ab🙂a雫")]);
        let V::Array(values) = &result else { panic!() };
        assert_eq!(values.get().len(), 3);
        for (index, start, end) in [(0, 0, 2), (1, 6, 7), (2, 7, 10)] {
            let hit = &values.get()[index];
            assert!(matches!(hit.field("span").field("start"), V::Int(n) if n == start));
            assert!(matches!(hit.field("span").field("end"), V::Int(n) if n == end));
            let optional = hit.field("captures").index(1);
            assert_eq!(optional.tagged("std/prelude::Just"), index == 0);
            let empty = hit.field("captures").index(2);
            assert_eq!(empty.tagged("std/prelude::Just"), index != 2);
            if index != 2 {
                assert_eq!(native_text(&empty.payload().field("text")), "");
                assert!(matches!(empty.payload().field("span").field("start"), V::Int(n) if n == end));
                assert!(matches!(empty.payload().field("span").field("end"), V::Int(n) if n == end));
            }
            let entries = native_map_operation("std/map::entries", vec![], vec![hit.field("named")]);
            for (slot, name) in ["word", "empty", "other"].iter().enumerate() {
                assert_eq!(native_text(&entries.index(slot).index(0)), *name);
                assert_eq!(entries.index(slot).index(1).tagged("std/prelude::Just"), if slot == 2 { index == 2 } else { index != 2 });
            }
        }
        let saved = result.index(2);
        drop(result);
        assert_eq!(native_text(&saved.field("captures").index(3).payload().field("text")), "雫");
        drop(saved);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn split_keeps_empty_parts_without_inserting_captures() {
        for (pattern, text, expected) in [
            ("", "", vec!["", ""]),
            ("", "a🙂é", vec!["", "a", "🙂", "e", "́", ""]),
            ("(,)", ",a,,b,", vec!["", "a", "", "b", ""]),
            (",+", ",a,,b,", vec!["", "a", "b", ""]),
            ("a*", "a🙂", vec!["", "", "🙂", ""]),
            ("z", "", vec![""]),
            ("z", "雫🙂", vec!["雫🙂"]),
        ] {
            assert_eq!(strings(&native_regex_operation("std/regex::split", vec![compiled(pattern), V::text(text)])), expected, "{pattern:?}/{text:?}");
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn replacement_is_literal_can_delete_matches_and_keeps_unmatched_scalars() {
        for (pattern, replacement, text, expected) in [
            ("", "$1\\$&${name}", "a🙂", "$1\\$&${name}a$1\\$&${name}🙂$1\\$&${name}"),
            ("(?<digits>[0-9]+)", "$1\\", "a12b3", "a$1\\b$1\\"),
            ("[0-9]+", "", "a12b3", "ab"),
            ("a*", "_", "a🙂", "__🙂_"),
            ("", "_", "", "_"),
            ("z", "_", "", ""),
            ("z", "_", "雫🙂", "雫🙂"),
            (".", "\0🙂", "ab", "\0🙂\0🙂"),
        ] {
            let result = native_regex_operation("std/regex::replaceAll", vec![compiled(pattern), V::text(replacement), V::text(text)]);
            assert_eq!(native_text(&result), expected, "{pattern:?}/{text:?}");
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn replacement_callbacks_run_once_in_order_skip_absence_and_allow_reentry() {
        enable_cycle_collection();
        let calls = V::reference(V::text(""));
        let saved = V::reference(V::Unit);
        let callback = V::fun(vec![calls.clone(), saved.clone(), compiled(".")], |captures, hit| {
            let start = hit.field("span").field("start").render();
            let previous = captures[0].read_reference();
            captures[0].write_reference(V::text(&format!("{}{start},", native_text(&previous))));
            captures[1].write_reference(hit.clone());
            let nested = native_regex_operation("std/regex::replaceAll", vec![captures[2].clone(), V::text("_"), V::text("雫🙂")]);
            assert_eq!(native_text(&nested), "__");
            collect_cycles().unwrap();
            V::text(&format!("[{start}]"))
        });
        let result = native_regex_operation("std/regex::replaceAllWith", vec![compiled("(?<empty>)"), callback.clone(), V::text("a🙂b")]);
        assert_eq!(native_text(&result), "[0]a[1]🙂[5]b[6]");
        assert_eq!(native_text(&calls.read_reference()), "0,1,5,6,");
        for text in ["", "雫🙂"] {
            let result = native_regex_operation("std/regex::replaceAllWith", vec![compiled("z"), callback.clone(), V::text(text)]);
            assert_eq!(native_text(&result), text);
        }
        assert_eq!(native_text(&calls.read_reference()), "0,1,5,6,");
        assert!(matches!(saved.read_reference().field("captures").index(0).payload().field("span").field("start"), V::Int(6)));
        drop((result, callback, calls, saved));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn replacement_callback_defects_stop_later_calls_and_release_captured_cycles() {
        enable_cycle_collection();
        let calls = V::reference(V::text(""));
        let cycle = V::reference(V::Unit);
        let callback = V::fun(vec![calls.clone(), cycle.clone()], |captures, hit| {
            let V::Int(start) = hit.field("span").field("start") else { panic!() };
            let previous = captures[0].read_reference();
            captures[0].write_reference(V::text(&format!("{}{start},", native_text(&previous))));
            assert_ne!(start, 5, "replacement stop");
            V::text("_")
        });
        cycle.write_reference(V::Tuple(Shared::new(vec![cycle.clone(), callback.clone()])));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| native_regex_operation("std/regex::replaceAllWith", vec![compiled(""), callback.clone(), V::text("a🙂b")])));
        assert!(result.is_err());
        assert_eq!(native_text(&calls.read_reference()), "0,1,5,");
        drop((callback, calls, cycle));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
