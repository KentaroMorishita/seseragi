pub fn native_text_operation(name: &str, arguments: Vec<V>) -> V {
    match name {
        "std/text::encodeUtf8" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.encodeUtf8 arity");
            V::Bytes(Shared::new(native_text(&arguments[0]).as_bytes().to_vec()))
        }
        "std/text::decodeUtf8" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.decodeUtf8 arity");
            match std::str::from_utf8(native_bytes(&arguments[0])) {
                Ok(text) => native_right(V::text(text)),
                Err(error) => native_left(native_utf8_decode_error(error)),
            }
        }
        "std/text::decodeUtf8Lossy" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.decodeUtf8Lossy arity");
            V::text(&String::from_utf8_lossy(native_bytes(&arguments[0])))
        }
        "std/text::isEmpty" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.isEmpty arity");
            V::Bool(native_text(&arguments[0]).is_empty())
        }
        "std/text::toLower" | "std/text::toUpper" | "std/text::caseFold" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text casing arity");
            V::text(&native_text_case(name, native_text(&arguments[0])))
        }
        "std/text::lengthScalars" | "std/text::lengthBytes" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text length arity");
            let text = native_text(&arguments[0]);
            checked_int(if name == "std/text::lengthScalars" {
                text.chars().count() as i64
            } else {
                text.len() as i64
            })
        }
        "std/text::concat" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.concat arity");
            let V::Array(values) = &arguments[0] else {
                panic!("Rust ABI: text.concat expected Array<String>")
            };
            let mut output = String::new();
            for value in values.get() {
                output.push_str(native_text(value));
            }
            V::text(&output)
        }
        "std/text::scalarAt" => {
            assert_eq!(arguments.len(), 2, "Rust ABI: text.scalarAt arity");
            let V::Int(index) = arguments[0] else {
                panic!("Rust ABI: text.scalarAt expected Int")
            };
            let text = native_text(&arguments[1]);
            match usize::try_from(index)
                .ok()
                .and_then(|i| text.chars().nth(i))
            {
                Some(scalar) => V::adt("std/prelude::Just", V::Char(scalar)),
                None => V::adt("std/prelude::Nothing", V::Unit),
            }
        }
        "std/text::sliceScalars" => {
            assert_eq!(arguments.len(), 3, "Rust ABI: text.sliceScalars arity");
            let (V::Int(start), V::Int(end)) = (&arguments[0], &arguments[1]) else {
                panic!("Rust ABI: text.sliceScalars expected Int bounds")
            };
            let text = native_text(&arguments[2]);
            let (mut size, mut first, mut last) = (0, 0, 0);
            for (offset, _) in text.char_indices() {
                if size == *start {
                    first = offset;
                }
                if size == *end {
                    last = offset;
                }
                size += 1;
            }
            if size == *start {
                first = text.len();
            }
            if size == *end {
                last = text.len();
            }
            if *start < 0 || start > end || *end > size {
                native_left(V::adt(
                    "std/text::InvalidScalarRange",
                    V::Record(Shared::new(BTreeMap::from([
                        ("start".into(), V::Int(*start)),
                        ("end".into(), V::Int(*end)),
                        ("length".into(), V::Int(size)),
                    ]))),
                ))
            } else {
                native_right(V::text(&text[first..last]))
            }
        }
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
        "std/text::words" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.words arity");
            V::Array(Shared::new(
                native_text(&arguments[0])
                    .split(native_text_whitespace)
                    .filter(|word| !word.is_empty())
                    .map(V::text)
                    .collect(),
            ))
        }
        "std/text::trim" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text.trim arity");
            V::text(native_text(&arguments[0]).trim_matches(native_text_whitespace))
        }
        "std/text::trimStart" | "std/text::trimEnd" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: text trim arity");
            let text = native_text(&arguments[0]);
            V::text(if name == "std/text::trimStart" {
                text.trim_start_matches(native_text_whitespace)
            } else {
                text.trim_end_matches(native_text_whitespace)
            })
        }
        "std/text::contains" => {
            assert_eq!(arguments.len(), 2, "Rust ABI: text.contains arity");
            V::Bool(native_text(&arguments[1]).contains(native_text(&arguments[0])))
        }
        "std/text::startsWith" | "std/text::endsWith" => {
            assert_eq!(
                arguments.len(),
                2,
                "Rust ABI: text boundary predicate arity"
            );
            let pattern = native_text(&arguments[0]);
            let text = native_text(&arguments[1]);
            V::Bool(if name == "std/text::startsWith" {
                text.starts_with(pattern)
            } else {
                text.ends_with(pattern)
            })
        }
        "std/text::replace" => {
            assert_eq!(arguments.len(), 3, "Rust ABI: text.replace arity");
            V::text(&native_text(&arguments[2]).replacen(
                native_text(&arguments[0]),
                native_text(&arguments[1]),
                1,
            ))
        }
        "std/text::replaceAll" => {
            assert_eq!(arguments.len(), 3, "Rust ABI: text.replaceAll arity");
            V::text(
                &native_text(&arguments[2])
                    .replace(native_text(&arguments[0]), native_text(&arguments[1])),
            )
        }
        _ => panic!("Rust ABI: unsupported text operation"),
    }
}

fn native_utf8_decode_error(error: std::str::Utf8Error) -> V {
    V::adt(
        "std/text::InvalidUtf8",
        V::Record(Shared::new(BTreeMap::from([(
            "offset".into(),
            V::Int(error.valid_up_to() as i64),
        )]))),
    )
}

// Unicode 17 White_Space, shared with std/text's pinned property table.
// In particular, FEFF is content and NEL is whitespace.
fn native_text_whitespace(scalar: char) -> bool {
    matches!(scalar, '\u{9}'..='\u{d}' | '\u{20}' | '\u{85}' | '\u{a0}'
        | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}'
        | '\u{202f}' | '\u{205f}' | '\u{3000}')
}

#[cfg(test)]
mod utf8_tests {
    use super::*;

    #[test]
    fn encoding_preserves_empty_nul_combining_bom_and_scalar_boundaries() {
        clear_dictionaries();
        for (text, expected) in [
            ("", &b""[..]),
            ("A\0\n\u{7f}", &b"A\0\n\x7f"[..]),
            ("e\u{301}🙂", &[101, 204, 129, 240, 159, 153, 130][..]),
            ("\u{feff}A\u{feff}", &[239, 187, 191, 65, 239, 187, 191][..]),
            (
                "\u{80}\u{7ff}\u{800}\u{d7ff}\u{e000}\u{ffff}\u{10000}\u{10ffff}",
                &[
                    194, 128, 223, 191, 224, 160, 128, 237, 159, 191, 238, 128, 128, 239, 191, 191,
                    240, 144, 128, 128, 244, 143, 191, 191,
                ][..],
            ),
        ] {
            let input = V::text(text);
            let bytes = native_text_operation("std/text::encodeUtf8", vec![input.clone()]);
            assert_eq!(native_bytes(&bytes), expected);
            assert_eq!(native_text(&input), text);
            let strict = native_text_operation("std/text::decodeUtf8", vec![bytes.clone()]);
            let lossy = native_text_operation("std/text::decodeUtf8Lossy", vec![bytes]);
            assert!(strict.tagged("std/prelude::Right"));
            assert_eq!(native_text(&strict.payload()), text);
            assert_eq!(native_text(&lossy), text);
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn decoding_reports_the_sequence_start_and_replaces_each_maximal_invalid_subsequence() {
        clear_dictionaries();
        for (input, offset, expected) in [
            (&[128][..], 0, "�"),
            (&[255][..], 0, "�"),
            (&[192, 128][..], 0, "��"),
            (&[224, 128, 128][..], 0, "���"),
            (&[237, 160, 128][..], 0, "���"),
            (&[240, 128, 128, 128][..], 0, "����"),
            (&[244, 144, 128, 128][..], 0, "����"),
            (&[245, 128, 128, 128][..], 0, "����"),
            (&[194][..], 0, "�"),
            (&[194, 65][..], 0, "�A"),
            (&[224, 160][..], 0, "�"),
            (&[225, 128, 65][..], 0, "�A"),
            (&[225, 65, 128][..], 0, "�A�"),
            (&[240, 144, 128][..], 0, "�"),
            (&[241, 128, 128, 65][..], 0, "�A"),
            (&[241, 128, 65, 128][..], 0, "�A�"),
            (&[226, 130, 194, 162][..], 0, "�¢"),
            (&[226, 130, 240, 159, 153, 130][..], 0, "�🙂"),
            (&[97, 195, 169, 226, 130, 65, 255][..], 3, "aé�A�"),
            (&[240, 159, 153, 130, 255, 128][..], 4, "🙂��"),
            (&[239, 187, 191, 226, 130][..], 3, "\u{feff}�"),
            (&[226, 130, 239, 187, 191][..], 0, "�\u{feff}"),
        ] {
            let bytes = V::Bytes(Shared::new(input.to_vec()));
            let strict = native_text_operation("std/text::decodeUtf8", vec![bytes.clone()]);
            assert!(strict.tagged("std/prelude::Left"), "{input:?}");
            let error = strict.payload();
            assert!(error.tagged("std/text::InvalidUtf8"), "{input:?}");
            assert_eq!(
                error.payload().field("offset").render(),
                offset.to_string(),
                "{input:?}"
            );
            let lossy = native_text_operation("std/text::decodeUtf8Lossy", vec![bytes.clone()]);
            assert_eq!(native_text(&lossy), expected, "{input:?}");
            assert_eq!(native_bytes(&bytes), input);
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn decoding_each_single_byte_is_independent_and_does_not_retain_previous_input() {
        clear_dictionaries();
        for byte in 0..=255 {
            let bytes = V::Bytes(Shared::new(vec![byte]));
            let strict = native_text_operation("std/text::decodeUtf8", vec![bytes.clone()]);
            let lossy = native_text_operation("std/text::decodeUtf8Lossy", vec![bytes]);
            if byte < 128 {
                assert!(strict.tagged("std/prelude::Right"));
                assert_eq!(native_text(&strict.payload()), char::from(byte).to_string());
                assert_eq!(native_text(&lossy), char::from(byte).to_string());
            } else {
                assert!(strict.tagged("std/prelude::Left"));
                let error = strict.payload();
                assert!(error.tagged("std/text::InvalidUtf8"));
                assert_eq!(error.payload().field("offset").render(), "0");
                assert_eq!(native_text(&lossy), "�");
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}

#[cfg(test)]
mod text_tests {
    use super::*;

    #[test]
    fn lengths_and_concat_preserve_scalar_byte_and_empty_boundaries() {
        clear_dictionaries();
        for (text, scalars, bytes) in [
            ("", 0, 0),
            ("\0", 1, 1),
            ("é", 1, 2),
            ("e\u{301}", 2, 3),
            ("A👍🏽e\u{301}\0", 6, 13),
            ("\u{feff}\u{10ffff}", 2, 7),
        ] {
            let input = V::text(text);
            assert_eq!(
                native_text_operation("std/text::isEmpty", vec![input.clone()]).boolean(),
                text.is_empty()
            );
            for (name, expected) in [("lengthScalars", scalars), ("lengthBytes", bytes)] {
                assert_eq!(
                    native_text_operation(&format!("std/text::{name}"), vec![input.clone()])
                        .render(),
                    expected.to_string()
                );
            }
            let values = V::Array(Shared::new(vec![
                V::text(""),
                input.clone(),
                V::text("🙂"),
                V::text(""),
            ]));
            let result = native_text_operation("std/text::concat", vec![values.clone()]);
            assert_eq!(native_text(&values.index(1)), text);
            drop((values, input));
            assert_eq!(native_text(&result), format!("{text}🙂"));
        }
        let empty = native_text_operation("std/text::concat", vec![V::Array(Shared::new(vec![]))]);
        assert_eq!(native_text(&empty), "");
        drop(empty);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn scalar_indices_and_ranges_return_exact_values_and_typed_error_payloads() {
        clear_dictionaries();
        for text in ["", "雪", "A👍🏽e\u{301}\0", "\u{feff}\u{10ffff}"] {
            let scalars = text.chars().collect::<Vec<_>>();
            let size = scalars.len() as i64;
            let indices = [
                -9_007_199_254_740_991,
                -1,
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                9_007_199_254_740_991,
            ];
            for index in indices {
                let value =
                    native_text_operation("std/text::scalarAt", vec![V::Int(index), V::text(text)]);
                if index >= 0 && index < size {
                    assert!(value.tagged("std/prelude::Just"));
                    assert_eq!(native_char(&value.payload()), scalars[index as usize]);
                } else {
                    assert!(value.tagged("std/prelude::Nothing"));
                }
                for end in indices {
                    let value = native_text_operation(
                        "std/text::sliceScalars",
                        vec![V::Int(index), V::Int(end), V::text(text)],
                    );
                    if index >= 0 && index <= end && end <= size {
                        assert!(value.tagged("std/prelude::Right"));
                        let expected = scalars[index as usize..end as usize]
                            .iter()
                            .collect::<String>();
                        assert_eq!(native_text(&value.payload()), expected);
                    } else {
                        assert!(value.tagged("std/prelude::Left"));
                        let error = value.payload();
                        assert!(error.tagged("std/text::InvalidScalarRange"));
                        let payload = error.payload();
                        for (name, expected) in [("start", index), ("end", end), ("length", size)] {
                            assert_eq!(payload.field(name).render(), expected.to_string());
                        }
                    }
                }
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn words_and_one_sided_trim_use_every_pinned_whitespace_scalar() {
        clear_dictionaries();
        for scalar in "\t\n\u{b}\u{c}\r \u{85}\u{a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}".chars() {
            let text = format!("{scalar}{scalar}雫,{scalar}🙂!\u{feff}\0{scalar}{scalar}");
            assert_eq!(
                strings(native_text_operation("std/text::words", vec![V::text(&text)])),
                ["雫,", "🙂!\u{feff}\0"]
            );
            for (name, expected) in [
                ("trimStart", format!("雫,{scalar}🙂!\u{feff}\0{scalar}{scalar}")),
                ("trimEnd", format!("{scalar}{scalar}雫,{scalar}🙂!\u{feff}\0")),
            ] {
                let result = native_text_operation(&format!("std/text::{name}"), vec![V::text(&text)]);
                assert_eq!(native_text(&result), expected);
                let empty = native_text_operation(&format!("std/text::{name}"), vec![V::text(&format!("{scalar}{scalar}"))]);
                assert_eq!(native_text(&empty), "");
            }
        }
        for text in ["", "\u{feff}", "\u{180e}\u{200b}\u{2060}\0"] {
            for name in ["trimStart", "trimEnd"] {
                assert_eq!(
                    native_text(&native_text_operation(
                        &format!("std/text::{name}"),
                        vec![V::text(text)]
                    )),
                    text
                );
            }
            assert_eq!(
                strings(native_text_operation(
                    "std/text::words",
                    vec![V::text(text)]
                )),
                if text.is_empty() { vec![] } else { vec![text] }
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn replace_all_is_literal_nonoverlapping_and_inserts_at_scalar_boundaries() {
        clear_dictionaries();
        for (needle, replacement, text, expected) in [
            ("aa", "$&", "aaaaa", "$&$&a"),
            ("aba", "X", "ababa", "Xba"),
            ("", "/", "A👍🏽e\u{301}", "/A/👍/🏽/e/\u{301}/"),
            ("", "/", "", "/"),
            ("", "", "雪🙂", "雪🙂"),
            ("雪", "川", "雪🙂雪", "川🙂川"),
            ("🙂", "", "🙂🙂", ""),
            ("x", "y", "", ""),
            ("longer", "x", "long", "long"),
            ("é", "x", "e\u{301}", "e\u{301}"),
            (".", "$1\\", "a.b.", "a$1\\b$1\\"),
            ("\0", "$`$'", "a\0🙂\0", "a$`$'🙂$`$'"),
        ] {
            let result = native_text_operation(
                "std/text::replaceAll",
                vec![V::text(needle), V::text(replacement), V::text(text)],
            );
            assert_eq!(native_text(&result), expected);
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn prefix_and_suffix_are_literal_and_keep_unicode_empty_and_nul_boundaries() {
        clear_dictionaries();
        for (pattern, text, starts, ends) in [
            ("", "", true, true),
            ("", "雪🙂", true, true),
            ("雪", "", false, false),
            ("雪🙂", "雪🙂", true, true),
            ("雪", "雪🙂", true, false),
            ("🙂", "雪🙂", false, true),
            ("中", "前中後", false, false),
            ("longer", "long", false, false),
            ("é", "e\u{301}", false, false),
            ("e", "e\u{301}", true, false),
            ("\u{301}", "e\u{301}", false, true),
            ("\0", "\0雪\0", true, true),
            ("snow", "Snow", false, false),
        ] {
            for (operation, expected) in [("startsWith", starts), ("endsWith", ends)] {
                let output = native_text_operation(
                    &format!("std/text::{operation}"),
                    vec![V::text(pattern), V::text(text)],
                );
                assert_eq!(
                    output.boolean(),
                    expected,
                    "{operation}/{pattern:?}/{text:?}"
                );
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

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
