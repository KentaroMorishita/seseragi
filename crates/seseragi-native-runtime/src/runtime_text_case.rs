fn native_text_case_mapping(
    table: &'static [(u32, &'static [u32])],
    scalar: char,
) -> Option<&'static [u32]> {
    table
        .binary_search_by_key(&u32::from(scalar), |(point, _)| *point)
        .ok()
        .map(|index| table[index].1)
}

fn native_text_case_property(ranges: &[[u32; 3]], scalar: char) -> bool {
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
        .is_ok()
}

fn native_text_push_case(output: &mut String, scalar: char, mapping: Option<&[u32]>) {
    match mapping {
        Some(points) => output.extend(
            points
                .iter()
                .map(|point| char::from_u32(*point).expect("pinned Unicode scalar")),
        ),
        None => output.push(scalar),
    }
}

fn native_text_case(name: &str, text: &str) -> String {
    use native_text_case_data::{CASED, CASE_IGNORABLE, FINAL_SIGMA, FULL_FOLD, LOWER, UPPER};

    let mut output = String::with_capacity(text.len());
    let table = match name {
        "std/text::toLower" => LOWER,
        "std/text::toUpper" => UPPER,
        "std/text::caseFold" => FULL_FOLD,
        _ => panic!("Rust ABI: unsupported text casing operation"),
    };
    if name != "std/text::toLower" {
        for scalar in text.chars() {
            native_text_push_case(&mut output, scalar, native_text_case_mapping(table, scalar));
        }
        return output;
    }

    let scalars = text.chars().collect::<Vec<_>>();
    let mut followed_by_cased = vec![false; scalars.len()];
    let mut following = false;
    for (index, &scalar) in scalars.iter().enumerate().rev() {
        followed_by_cased[index] = following;
        if !native_text_case_property(CASE_IGNORABLE, scalar) {
            following = native_text_case_property(CASED, scalar);
        }
    }
    let mut preceded_by_cased = false;
    for (index, scalar) in scalars.into_iter().enumerate() {
        let mapping = if preceded_by_cased && !followed_by_cased[index] {
            native_text_case_mapping(FINAL_SIGMA, scalar)
                .or_else(|| native_text_case_mapping(LOWER, scalar))
        } else {
            native_text_case_mapping(LOWER, scalar)
        };
        native_text_push_case(&mut output, scalar, mapping);
        if !native_text_case_property(CASE_IGNORABLE, scalar) {
            preceded_by_cased = native_text_case_property(CASED, scalar);
        }
    }
    output
}

#[cfg(test)]
mod text_case_tests {
    use super::*;

    #[test]
    fn casing_preserves_input_and_uses_full_locale_independent_mappings() {
        clear_dictionaries();
        for (source, lower, upper, fold) in [
            ("", "", "", ""),
            ("AZ az 09", "az az 09", "AZ AZ 09", "az az 09"),
            (
                "ΟΣ ΟΣΑ Σ İ",
                "ος οσα σ i\u{307}",
                "ΟΣ ΟΣΑ Σ İ",
                "οσ οσα σ i\u{307}",
            ),
            (
                "Ο'Σ\u{301}! Ο'Σ\u{301}Α",
                "ο'ς\u{301}! ο'σ\u{301}α",
                "Ο'Σ\u{301}! Ο'Σ\u{301}Α",
                "ο'σ\u{301}! ο'σ\u{301}α",
            ),
            (
                "Straße ﬃ ᾀ",
                "straße ﬃ ᾀ",
                "STRASSE FFI ἈΙ",
                "strasse ffi ἀι",
            ),
            ("Iİıi", "ii\u{307}ıi", "IİII", "ii\u{307}ıi"),
            ("\u{1fbe}Σ", "\u{1fbe}ς", "ΙΣ", "ισ"),
            (
                "\u{a7ce}\u{a7cf}\u{16ea0}\u{16ebb}",
                "\u{a7cf}\u{a7cf}\u{16ebb}\u{16ebb}",
                "\u{a7ce}\u{a7ce}\u{16ea0}\u{16ea0}",
                "\u{a7cf}\u{a7cf}\u{16ebb}\u{16ebb}",
            ),
            (
                "\0🙂\u{10ffff}",
                "\0🙂\u{10ffff}",
                "\0🙂\u{10ffff}",
                "\0🙂\u{10ffff}",
            ),
        ] {
            let input = V::text(source);
            for (name, expected) in [("toLower", lower), ("toUpper", upper), ("caseFold", fold)] {
                let result =
                    native_text_operation(&format!("std/text::{name}"), vec![input.clone()]);
                assert_eq!(native_text(&result), expected, "{name}: {source:?}");
                assert_eq!(native_text(&input), source);
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn final_sigma_context_is_linear_across_long_ignorable_runs() {
        let marks = "\u{301}".repeat(20_000);
        for (source, expected) in [
            (format!("A{marks}Σ{marks}!"), format!("a{marks}ς{marks}!")),
            (format!("A{marks}Σ{marks}A"), format!("a{marks}σ{marks}a")),
            (format!("{marks}Σ{marks}"), format!("{marks}σ{marks}")),
            (
                format!("A{}", "Σ\u{301}".repeat(20_000)),
                format!("a{}ς\u{301}", "σ\u{301}".repeat(19_999)),
            ),
        ] {
            assert_eq!(native_text_case("std/text::toLower", &source), expected);
        }
    }

    #[test]
    fn every_scalar_matches_the_pinned_default_full_casing_and_folding_sources() {
        fn rows(source: &str) -> impl Iterator<Item = Vec<&str>> {
            source.lines().filter_map(|line| {
                let value = line.split('#').next().unwrap().trim();
                (!value.is_empty()).then(|| value.split(';').map(str::trim).collect())
            })
        }
        fn point(hex: &str) -> u32 {
            u32::from_str_radix(hex, 16).unwrap()
        }
        fn mapping(hex: &str) -> String {
            hex.split_whitespace()
                .map(|hex| char::from_u32(point(hex)).unwrap())
                .collect()
        }

        let mut lower = BTreeMap::new();
        let mut upper = BTreeMap::new();
        let mut fold = BTreeMap::new();
        for row in rows(include_str!("../../../runtime/unicode/ucd/UnicodeData.txt")) {
            if !row[13].is_empty() {
                lower.insert(point(row[0]), mapping(row[13]));
            }
            if !row[12].is_empty() {
                upper.insert(point(row[0]), mapping(row[12]));
            }
        }
        for row in rows(include_str!(
            "../../../runtime/unicode/ucd/SpecialCasing.txt"
        )) {
            if row[4].is_empty() {
                lower.insert(point(row[0]), mapping(row[1]));
                upper.insert(point(row[0]), mapping(row[3]));
            }
        }
        for row in rows(include_str!("../../../runtime/unicode/ucd/CaseFolding.txt")) {
            if matches!(row[1], "C" | "F") {
                fold.insert(point(row[0]), mapping(row[2]));
            }
        }
        for scalar in (0..=0x10ffff).filter_map(char::from_u32) {
            let source = scalar.to_string();
            for (name, table) in [
                ("toLower", &lower),
                ("toUpper", &upper),
                ("caseFold", &fold),
            ] {
                let expected = table.get(&u32::from(scalar)).unwrap_or(&source);
                assert_eq!(
                    native_text_case(&format!("std/text::{name}"), &source),
                    *expected,
                    "{name}: U+{:X}",
                    u32::from(scalar)
                );
            }
        }
    }
}
