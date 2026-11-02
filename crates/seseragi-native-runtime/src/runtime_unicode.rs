fn native_unicode_range(ranges: &[[u32; 3]], scalar: char, fallback: u32) -> u32 {
    let point = u32::from(scalar);
    ranges
        .binary_search_by(|range| {
            if point < range[0] {
                std::cmp::Ordering::Greater
            } else if point > range[1] {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .map_or(fallback, |index| ranges[index][2])
}

fn native_unicode_decompose(scalar: char, compatibility: bool, output: &mut Vec<char>) {
    use native_unicode_data::{CANONICAL_DECOMPOSITION, COMPATIBILITY_DECOMPOSITION};

    let point = u32::from(scalar);
    if (0xac00..0xd7a4).contains(&point) {
        let syllable = point - 0xac00;
        output.push(char::from_u32(0x1100 + syllable / 588).unwrap());
        output.push(char::from_u32(0x1161 + (syllable % 588) / 28).unwrap());
        if syllable % 28 != 0 {
            output.push(char::from_u32(0x11a7 + syllable % 28).unwrap());
        }
        return;
    }
    let mapping = compatibility
        .then(|| native_text_case_mapping(COMPATIBILITY_DECOMPOSITION, scalar))
        .flatten()
        .or_else(|| native_text_case_mapping(CANONICAL_DECOMPOSITION, scalar));
    if let Some(mapping) = mapping {
        for &point in mapping {
            native_unicode_decompose(char::from_u32(point).unwrap(), compatibility, output);
        }
    } else {
        output.push(scalar);
    }
}

fn native_unicode_composite(first: char, second: char) -> Option<char> {
    let (first, second) = (u32::from(first), u32::from(second));
    if (0x1100..0x1113).contains(&first) && (0x1161..0x1176).contains(&second) {
        return char::from_u32(0xac00 + (first - 0x1100) * 588 + (second - 0x1161) * 28);
    }
    if (0xac00..0xd7a4).contains(&first)
        && (first - 0xac00) % 28 == 0
        && (0x11a8..0x11c3).contains(&second)
    {
        return char::from_u32(first + second - 0x11a7);
    }
    native_unicode_data::COMPOSITIONS
        .binary_search_by_key(&(first, second), |(pair, _)| *pair)
        .ok()
        .and_then(|index| char::from_u32(native_unicode_data::COMPOSITIONS[index].1))
}

fn native_unicode_normalize(form: &str, text: &str) -> String {
    let compatibility = matches!(form, "NFKC" | "NFKD");
    let mut decomposed = Vec::new();
    for scalar in text.chars() {
        native_unicode_decompose(scalar, compatibility, &mut decomposed);
    }
    let class = |scalar| native_unicode_range(native_unicode_data::COMBINING_CLASS, scalar, 0);
    let mut ordered = Vec::with_capacity(decomposed.len());
    let mut marks: BTreeMap<u32, Vec<char>> = BTreeMap::new();
    for scalar in decomposed {
        let ccc = class(scalar);
        if ccc == 0 {
            for (_, bucket) in std::mem::take(&mut marks) {
                ordered.extend(bucket);
            }
            ordered.push(scalar);
        } else {
            marks.entry(ccc).or_default().push(scalar);
        }
    }
    for (_, bucket) in marks {
        ordered.extend(bucket);
    }
    if matches!(form, "NFD" | "NFKD") {
        return ordered.into_iter().collect();
    }
    let mut composed = Vec::with_capacity(ordered.len());
    let mut starter = None;
    let mut last_class = 0;
    for scalar in ordered {
        let ccc = class(scalar);
        let combined = starter
            .filter(|_| last_class == 0 || last_class < ccc)
            .and_then(|index| native_unicode_composite(composed[index], scalar));
        if let Some(combined) = combined {
            composed[starter.unwrap()] = combined;
        } else {
            if ccc == 0 {
                starter = Some(composed.len());
            }
            composed.push(scalar);
            last_class = ccc;
        }
    }
    composed.into_iter().collect()
}

pub fn native_unicode_operation(name: &str, arguments: Vec<V>) -> V {
    use native_unicode_data::{ALPHABETIC, CATEGORY_TAGS, GENERAL_CATEGORY, VERSION, WHITE_SPACE};

    match name {
        "std/text/unicode::version" => {
            assert!(
                matches!(arguments.as_slice(), [V::Unit]),
                "Rust ABI: Unicode version argument"
            );
            V::text(VERSION)
        }
        "std/text/unicode::normalize" | "std/text/unicode::isNormalized" => {
            let [form, text] = arguments.as_slice() else {
                panic!("Rust ABI: Unicode normalization arity")
            };
            let form = match form {
                V::Adt(value) => match value.get().0 {
                    "std/text/unicode::NFC" => "NFC",
                    "std/text/unicode::NFD" => "NFD",
                    "std/text/unicode::NFKC" => "NFKC",
                    "std/text/unicode::NFKD" => "NFKD",
                    _ => panic!("Rust ABI: expected canonical NormalizationForm"),
                },
                _ => panic!("Rust ABI: expected NormalizationForm"),
            };
            let text = native_text(text);
            let normalized = native_unicode_normalize(form, text);
            if name == "std/text/unicode::normalize" {
                V::text(&normalized)
            } else {
                V::Bool(normalized == text)
            }
        }
        "std/text/unicode::fullCaseFold" => {
            assert_eq!(arguments.len(), 1, "Rust ABI: Unicode fullCaseFold arity");
            V::text(&native_text_case(
                "std/text::caseFold",
                native_text(&arguments[0]),
            ))
        }
        _ => {
            let [V::Char(scalar)] = arguments.as_slice() else {
                panic!("Rust ABI: Unicode property expects Char")
            };
            let category = native_unicode_range(GENERAL_CATEGORY, *scalar, 28);
            match name {
                "std/text/unicode::generalCategory" => {
                    V::adt(CATEGORY_TAGS[category as usize], V::Unit)
                }
                "std/text/unicode::simpleCaseFold" => V::Char(
                    native_text_case_mapping(native_text_case_data::SIMPLE_FOLD, *scalar)
                        .map_or(*scalar, |mapping| char::from_u32(mapping[0]).unwrap()),
                ),
                "std/text/unicode::isAlphabetic" => {
                    V::Bool(native_unicode_range(ALPHABETIC, *scalar, 0) != 0)
                }
                "std/text/unicode::isWhitespace" => {
                    V::Bool(native_unicode_range(WHITE_SPACE, *scalar, 0) != 0)
                }
                "std/text/unicode::isDecimalDigit" => V::Bool(category == 8),
                "std/text/unicode::isMark" => V::Bool((5..=7).contains(&category)),
                _ => panic!("Rust ABI: unsupported Unicode operation"),
            }
        }
    }
}

pub fn native_unicode_category_ord_dictionary() -> V {
    fn index(value: &V) -> usize {
        let V::Adt(value) = value else {
            panic!("Rust ABI: expected UnicodeGeneralCategory")
        };
        native_unicode_data::CATEGORY_TAGS
            .iter()
            .position(|tag| *tag == value.get().0)
            .expect("Rust ABI: expected canonical UnicodeGeneralCategory")
    }
    V::dict(vec![
        (
            "eq",
            V::fun(vec![], |_, left| {
                V::fun(vec![left], |c, right| {
                    V::Bool(index(&c[0]) == index(&right))
                })
            }),
        ),
        (
            "compare",
            V::fun(vec![], |_, left| {
                V::fun(vec![left], |c, right| {
                    V::adt(
                        match index(&c[0]).cmp(&index(&right)) {
                            std::cmp::Ordering::Less => "std/prelude::Less",
                            std::cmp::Ordering::Equal => "std/prelude::Equal",
                            std::cmp::Ordering::Greater => "std/prelude::Greater",
                        },
                        V::Unit,
                    )
                })
            }),
        ),
    ])
}

#[cfg(test)]
mod unicode_tests {
    use super::*;

    fn rows(source: &str) -> impl Iterator<Item = Vec<&str>> {
        source.lines().filter_map(|line| {
            let value = line.split('#').next().unwrap().trim();
            (!value.is_empty() && !value.starts_with('@'))
                .then(|| value.split(';').map(str::trim).collect())
        })
    }

    fn point(hex: &str) -> u32 {
        u32::from_str_radix(hex, 16).unwrap()
    }

    fn scalars(hex: &str) -> String {
        hex.split_whitespace()
            .map(|hex| char::from_u32(point(hex)).unwrap())
            .collect()
    }

    #[test]
    fn normalization_matches_every_pinned_conformance_row() {
        let expected_columns = [[1, 1, 1, 3, 3], [2, 2, 2, 4, 4], [3; 5], [4; 5]];
        let mut count = 0;
        for row in rows(include_str!(
            "../../../runtime/unicode/ucd/NormalizationTest.txt"
        )) {
            let columns = row[..5]
                .iter()
                .map(|value| scalars(value))
                .collect::<Vec<_>>();
            for (form_index, form) in ["NFC", "NFD", "NFKC", "NFKD"].iter().enumerate() {
                for column in 0..5 {
                    assert_eq!(
                        native_unicode_normalize(form, &columns[column]),
                        columns[expected_columns[form_index][column]],
                        "{form}, column {column}: {}",
                        row[0]
                    );
                }
            }
            count += 1;
        }
        assert!(count > 19_000);
    }

    #[test]
    fn long_combining_runs_keep_stable_order_and_composition_blocking() {
        let source = format!("a{}", "\u{301}\u{323}".repeat(20_000));
        let decomposed = format!("a{}{}", "\u{323}".repeat(20_000), "\u{301}".repeat(20_000));
        assert_eq!(native_unicode_normalize("NFD", &source), decomposed);
        assert_eq!(native_unicode_normalize("NFD", &decomposed), decomposed);
        assert_eq!(
            native_unicode_normalize("NFC", "A\u{305}\u{301}"),
            "A\u{305}\u{301}"
        );
        for form in ["NFC", "NFD", "NFKC", "NFKD"] {
            for source in [
                "",
                "\0\u{d7ff}\u{e000}\u{10ffff}",
                "\u{301}\u{323}",
                "각ﬃ①é",
            ] {
                let normalized = native_unicode_normalize(form, source);
                assert_eq!(native_unicode_normalize(form, &normalized), normalized);
            }
        }
    }

    #[test]
    fn every_scalar_property_matches_the_pinned_sources() {
        let codes = [
            "Lu", "Ll", "Lt", "Lm", "Lo", "Mn", "Mc", "Me", "Nd", "Nl", "No", "Pc", "Pd", "Ps",
            "Pe", "Pi", "Pf", "Po", "Sm", "Sc", "Sk", "So", "Zs", "Zl", "Zp", "Cc", "Cf", "Co",
            "Cn",
        ];
        let mut expected = vec![28u8; 0x110000];
        let mut start = None;
        for row in rows(include_str!("../../../runtime/unicode/ucd/UnicodeData.txt")) {
            let end = point(row[0]) as usize;
            if row[1].ends_with(", First>") {
                start = Some(end);
                continue;
            }
            let first = start.take().unwrap_or(end);
            if let Some(category) = codes.iter().position(|code| *code == row[2]) {
                expected[first..=end].fill(category as u8);
            }
        }
        for (source, property, bit) in [
            (
                include_str!("../../../runtime/unicode/ucd/DerivedCoreProperties.txt"),
                "Alphabetic",
                0x20,
            ),
            (
                include_str!("../../../runtime/unicode/ucd/PropList.txt"),
                "White_Space",
                0x40,
            ),
        ] {
            for row in rows(source).filter(|row| row[1] == property) {
                let mut range = row[0].split("..");
                let start = point(range.next().unwrap());
                let end = range.next().map(point).unwrap_or(start);
                for value in &mut expected[start as usize..=end as usize] {
                    *value |= bit;
                }
            }
        }
        for scalar in (0..=0x10ffff).filter_map(char::from_u32) {
            let expected = expected[scalar as usize];
            let category = u32::from(expected & 0x1f);
            assert_eq!(
                native_unicode_range(native_unicode_data::GENERAL_CATEGORY, scalar, 28),
                category,
                "{scalar:?}"
            );
            for (operation, expected) in [
                ("isAlphabetic", expected & 0x20 != 0),
                ("isWhitespace", expected & 0x40 != 0),
                ("isDecimalDigit", category == 8),
                ("isMark", (5..=7).contains(&category)),
            ] {
                assert_eq!(
                    native_unicode_operation(
                        &format!("std/text/unicode::{operation}"),
                        vec![V::Char(scalar)]
                    )
                    .boolean(),
                    expected,
                    "{operation}: {scalar:?}"
                );
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn simple_folding_matches_every_scalar_without_full_or_turkic_expansions() {
        let mut simple = BTreeMap::new();
        for row in rows(include_str!("../../../runtime/unicode/ucd/CaseFolding.txt")) {
            if matches!(row[1], "C" | "S") {
                let mapping = scalars(row[2]);
                assert_eq!(mapping.chars().count(), 1);
                simple.insert(point(row[0]), mapping.chars().next().unwrap());
            }
        }
        assert!(simple.len() > 1_500);
        for scalar in (0..=0x10ffff).filter_map(char::from_u32) {
            let folded =
                native_unicode_operation("std/text/unicode::simpleCaseFold", vec![V::Char(scalar)]);
            assert_eq!(
                native_char(&folded),
                *simple.get(&u32::from(scalar)).unwrap_or(&scalar),
                "{scalar:?}"
            );
        }
    }

    #[test]
    fn public_operations_preserve_values_version_and_category_declaration_order() {
        let manifest = include_str!("../../../runtime/unicode/manifest.json");
        let version = manifest
            .split("\"version\"")
            .nth(1)
            .unwrap()
            .split('"')
            .nth(1)
            .unwrap();
        let actual = native_unicode_operation("std/text/unicode::version", vec![V::Unit]);
        assert_eq!(native_text(&actual), version);
        let text = V::text("Straße İ Σς ﬃ");
        let folded = native_unicode_operation("std/text/unicode::fullCaseFold", vec![text.clone()]);
        assert_eq!(native_text(&folded), "strasse i\u{307} σσ ffi");
        assert_eq!(native_text(&text), "Straße İ Σς ﬃ");
        let dictionary = native_unicode_category_ord_dictionary();
        for (left, &a) in native_unicode_data::CATEGORY_TAGS.iter().enumerate() {
            for (right, &b) in native_unicode_data::CATEGORY_TAGS.iter().enumerate() {
                let result = dictionary
                    .field("compare")
                    .apply(V::adt(a, V::Unit))
                    .apply(V::adt(b, V::Unit));
                assert!(result.tagged(match left.cmp(&right) {
                    std::cmp::Ordering::Less => "std/prelude::Less",
                    std::cmp::Ordering::Equal => "std/prelude::Equal",
                    std::cmp::Ordering::Greater => "std/prelude::Greater",
                }));
                assert_eq!(
                    dictionary
                        .field("eq")
                        .apply(V::adt(a, V::Unit))
                        .apply(V::adt(b, V::Unit))
                        .boolean(),
                    left == right
                );
            }
        }
        drop((actual, text, folded, dictionary));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
