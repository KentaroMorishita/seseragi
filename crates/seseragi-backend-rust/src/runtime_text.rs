fn native_text_operation(name: &str, arguments: Vec<V>) -> V {
    match name {
        "std/text::lines" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.lines arity");
            let text = native_text(&arguments[0]);
            let mut scalars = text.char_indices().peekable();
            let mut previous = 0;
            let mut output = Vec::new();
            while let Some((offset, scalar)) = scalars.next() {
                if matches!(scalar, '\r' | '\n' | '\u{85}' | '\u{2028}' | '\u{2029}') {
                    output.push(V::text(&text[previous..offset]));
                    previous = offset + scalar.len_utf8();
                    if scalar == '\r' && scalars.peek().is_some_and(|(_, next)| *next == '\n') {
                        let (offset, _) = scalars.next().unwrap();
                        previous = offset + 1;
                    }
                }
            }
            // Match std/text: a terminal line break does not add a final empty
            // line, and empty input has no lines. Interior empty lines remain.
            if previous < text.len() {
                output.push(V::text(&text[previous..]));
            }
            V::Array(Shared::new(output))
        }
        "std/text::join" => {
            assert_eq!(arguments.len(), 2, "Rust ABI: text.join arity");
            let separator = native_text(&arguments[0]);
            let V::Array(values) = &arguments[1] else {
                panic!("Rust ABI: text.join expected Array<String>")
            };
            let mut output = String::new();
            for (index, value) in values.get().iter().enumerate() {
                if index != 0 {
                    output.push_str(separator);
                }
                output.push_str(native_text(value));
            }
            V::text(&output)
        }
        "std/text::split" => {
            assert_eq!(arguments.len(), 2, "Rust ABI: text.split arity");
            let separator = native_text(&arguments[0]);
            let text = native_text(&arguments[1]);
            let output = if separator.is_empty() {
                text.chars()
                    .map(|scalar| V::text(&scalar.to_string()))
                    .collect()
            } else {
                text.split(separator).map(V::text).collect()
            };
            V::Array(Shared::new(output))
        }
        "std/text::trim" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.trim arity");
            V::text(native_text(&arguments[0]).trim_matches(native_text_whitespace))
        }
        "std/text::contains" => {
            assert_eq!(arguments.len(), 2, "Rust ABI: text.contains arity");
            V::Bool(native_text(&arguments[1]).contains(native_text(&arguments[0])))
        }
        "std/text::replace" => {
            assert_eq!(arguments.len(), 3, "Rust ABI: text.replace arity");
            V::text(&native_text(&arguments[2]).replacen(
                native_text(&arguments[0]),
                native_text(&arguments[1]),
                1,
            ))
        }
        _ => panic!("Rust ABI: unsupported text operation"),
    }
}

// Unicode 17 White_Space, shared with std/text's pinned property table.
// In particular, FEFF is content and NEL is whitespace.
fn native_text_whitespace(scalar: char) -> bool {
    matches!(scalar, '\u{9}'..='\u{d}' | '\u{20}' | '\u{85}' | '\u{a0}'
        | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}'
        | '\u{202f}' | '\u{205f}' | '\u{3000}')
}

#[cfg(test)]
mod text_tests {
    use super::*;

    fn strings(value: V) -> Vec<String> {
        let V::Array(values) = value else { panic!() };
        values
            .get()
            .iter()
            .map(|value| native_text(value).to_owned())
            .collect()
    }

    #[test]
    fn lines_match_standard_unicode_boundaries_without_normalizing_contents() {
        clear_dictionaries();
        for (input, expected) in [
            ("", vec![]),
            ("a", vec!["a"]),
            ("\n", vec![""]),
            ("a\n", vec!["a"]),
            ("\n\n", vec!["", ""]),
            ("a\r\n\r\nb\r", vec!["a", "", "b"]),
            ("a\n\rb", vec!["a", "", "b"]),
            (
                "雫🙂\u{85}e\u{301}\u{2028}\0\u{2029}終",
                vec!["雫🙂", "e\u{301}", "\0", "終"],
            ),
            ("\r\n", vec![""]),
        ] {
            assert_eq!(
                strings(native_text_operation(
                    "std/text::lines",
                    vec![V::text(input)]
                )),
                expected
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn join_keeps_empty_elements_separator_and_owned_text_lifetimes() {
        clear_dictionaries();
        for separator in ["", "|", "\0🙂"] {
            let items = ["", "雫", "é", "e\u{301}", ""];
            let array = V::Array(Shared::new(
                items.iter().map(|item| V::text(item)).collect(),
            ));
            let joined = native_text_operation("std/text::join", vec![V::text(separator), array]);
            assert_eq!(native_text(&joined), items.join(separator));
            drop(joined);
        }
        let empty = native_text_operation(
            "std/text::join",
            vec![V::text("|"), V::Array(Shared::new(vec![]))],
        );
        assert_eq!(native_text(&empty), "");
        drop(empty);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn split_preserves_empty_fields_scalars_and_independent_results() {
        clear_dictionaries();
        for (separator, input, expected) in [
            (",", ",雫🙂,,", vec!["", "雫🙂", "", ""]),
            ("", "雫🙂e\u{301}", vec!["雫", "🙂", "e", "\u{301}"]),
            ("", "", vec![]),
            (",", "", vec![""]),
            ("aa", "aaaaa", vec!["", "", "a"]),
            ("\0", "a\0b\0", vec!["a", "b", ""]),
        ] {
            let result =
                native_text_operation("std/text::split", vec![V::text(separator), V::text(input)]);
            assert_eq!(strings(result), expected);
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn literal_search_replace_and_pinned_whitespace_keep_the_text_contract() {
        clear_dictionaries();
        let whitespace = "\t\n\u{b}\u{c}\r \u{85}\u{a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}";
        for (input, expected) in [
            (format!("{whitespace}雫🙂{whitespace}"), "雫🙂"),
            (whitespace.to_owned(), ""),
            ("\u{feff} 雫 \u{feff}".to_owned(), "\u{feff} 雫 \u{feff}"),
            ("\u{200b}雫\u{200b}".to_owned(), "\u{200b}雫\u{200b}"),
        ] {
            assert_eq!(
                native_text(&native_text_operation(
                    "std/text::trim",
                    vec![V::text(&input)]
                )),
                expected
            );
        }
        for (needle, input, expected) in [
            ("", "", true),
            ("🙂", "雫🙂", true),
            ("a+b", "aaab", false),
            ("\0", "a\0b", true),
        ] {
            let V::Bool(actual) =
                native_text_operation("std/text::contains", vec![V::text(needle), V::text(input)])
            else {
                panic!()
            };
            assert_eq!(actual, expected);
        }
        for (needle, replacement, input, expected) in [
            ("a", "$&$1", "aba", "$&$1ba"),
            ("", "🙂", "雫", "🙂雫"),
            ("aa", "x", "aaaa", "xaa"),
            ("absent", "x", "雫🙂", "雫🙂"),
        ] {
            assert_eq!(
                native_text(&native_text_operation(
                    "std/text::replace",
                    vec![V::text(needle), V::text(replacement), V::text(input)]
                )),
                expected
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
