//! Compiler-side Unicode operations from the same pinned UCD as the runtime.

use crate::unicode_data::{CASED, CASE_IGNORABLE, FINAL_SIGMA, LOWER, UPPERCASE, WHITE_SPACE};
pub use seseragi_release::{UNICODE_VERSION, UNICODE_VERSION_TUPLE};
pub use unicode_ident::{is_xid_continue, is_xid_start};

fn property(ranges: &[[u32; 3]], value: char) -> bool {
    let point = u32::from(value);
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

pub fn is_whitespace(value: char) -> bool {
    property(WHITE_SPACE, value)
}

pub fn is_uppercase(value: char) -> bool {
    property(UPPERCASE, value)
}

fn mapping(table: &'static [(u32, &'static [u32])], value: char) -> Option<&'static [u32]> {
    table
        .binary_search_by_key(&u32::from(value), |(point, _)| *point)
        .ok()
        .map(|index| table[index].1)
}

fn push_mapping(output: &mut String, value: char, mapping: Option<&[u32]>) {
    match mapping {
        Some(points) => output.extend(
            points
                .iter()
                .map(|point| char::from_u32(*point).expect("pinned Unicode scalar")),
        ),
        None => output.push(value),
    }
}

pub fn lowercase_first(value: &str) -> String {
    let mut chars = value.chars();
    let mut output = String::with_capacity(value.len());
    if let Some(first) = chars.next() {
        push_mapping(&mut output, first, mapping(LOWER, first));
    }
    output.extend(chars);
    output
}

/// Unicode default lowercase, including locale-independent Final_Sigma context.
pub fn lowercase(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut suffix = vec![false; chars.len()];
    let mut cased = false;
    for (index, value) in chars.iter().enumerate().rev() {
        suffix[index] = cased;
        if !property(CASE_IGNORABLE, *value) {
            cased = property(CASED, *value);
        }
    }
    let mut output = String::with_capacity(value.len());
    let mut before = false;
    for (index, value) in chars.into_iter().enumerate() {
        let mapped = if before && !suffix[index] {
            mapping(FINAL_SIGMA, value).or_else(|| mapping(LOWER, value))
        } else {
            mapping(LOWER, value)
        };
        push_mapping(&mut output, value, mapped);
        if !property(CASE_IGNORABLE, value) {
            before = property(CASED, value);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_match_every_scalar_in_the_pinned_derived_core_properties() {
        assert_eq!(unicode_ident::UNICODE_VERSION, UNICODE_VERSION_TUPLE);
        let mut start = vec![false; 0x110000];
        let mut continuation = vec![false; 0x110000];
        let mut uppercase = vec![false; 0x110000];
        for line in include_str!("../../../runtime/unicode/ucd/DerivedCoreProperties.txt").lines() {
            let fields = line
                .split('#')
                .next()
                .unwrap()
                .split(';')
                .map(str::trim)
                .collect::<Vec<_>>();
            let table = match fields.get(1).copied() {
                Some("XID_Start") => &mut start,
                Some("XID_Continue") => &mut continuation,
                Some("Uppercase") => &mut uppercase,
                _ => continue,
            };
            let range = fields[0]
                .split("..")
                .map(|point| usize::from_str_radix(point, 16).unwrap())
                .collect::<Vec<_>>();
            for value in &mut table[range[0]..=range.get(1).copied().unwrap_or(range[0])] {
                *value = true;
            }
        }
        for point in 0..0x110000 {
            if let Some(scalar) = char::from_u32(point as u32) {
                assert_eq!(is_xid_start(scalar), start[point], "XID_Start U+{point:X}");
                assert_eq!(
                    is_xid_continue(scalar),
                    continuation[point],
                    "XID_Continue U+{point:X}"
                );
                assert_eq!(
                    is_uppercase(scalar),
                    uppercase[point],
                    "Uppercase U+{point:X}"
                );
                // Rust string trimming is also used in non-lexical tooling.
                // Its White_Space set must agree with our pinned projection.
                assert_eq!(
                    is_whitespace(scalar),
                    scalar.is_whitespace(),
                    "White_Space U+{point:X}"
                );
            }
        }
    }

    #[test]
    fn casing_and_whitespace_use_pinned_data_not_rust_std_unicode() {
        assert_eq!(lowercase("ΟΣ ΟΣΑ Σ İ"), "ος οσα σ i\u{307}");
        assert_eq!(lowercase_first("İService"), "i\u{307}Service");
        assert_eq!(lowercase("\u{a7ce}"), "\u{a7cf}");
        assert!(is_whitespace('\u{85}'));
        assert!(!is_whitespace('\u{feff}'));
        let tokens = crate::lex("unicode.ssrg", "\u{a7ce} \u{a7cf}");
        assert_eq!(tokens.tokens[0].kind, crate::TokenKind::IdentifierUpper);
        assert_eq!(tokens.tokens[2].kind, crate::TokenKind::IdentifierLower);
    }
}
