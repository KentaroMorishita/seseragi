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
        _ => panic!("Rust ABI: unsupported text operation"),
    }
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
}
