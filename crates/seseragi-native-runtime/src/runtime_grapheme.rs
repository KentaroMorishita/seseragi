fn native_grapheme_boundaries(text: &str) -> impl Iterator<Item = usize> + '_ {
    use native_unicode_data::{EXTENDED_PICTOGRAPHIC, GRAPHEME_BREAK, INDIC_CONJUNCT_BREAK};

    // Values follow the pinned Grapheme_Cluster_Break table used by the TS runtime.
    const CR: u32 = 1;
    const LF: u32 = 2;
    const CONTROL: u32 = 3;
    const EXTEND: u32 = 4;
    const ZWJ: u32 = 5;
    const RI: u32 = 6;
    const PREPEND: u32 = 7;
    const SPACING_MARK: u32 = 8;
    const L: u32 = 9;
    const V: u32 = 10;
    const T: u32 = 11;
    const LV: u32 = 12;
    const LVT: u32 = 13;

    let (mut previous, mut regional_odd, mut emoji, mut indic) = (u32::MAX, false, 0, 0);
    let interior = text.char_indices().filter_map(move |(byte, scalar)| {
        let current = native_unicode_range(GRAPHEME_BREAK, scalar, 0);
        let conjunct = native_unicode_range(INDIC_CONJUNCT_BREAK, scalar, 0);
        let pictographic = native_unicode_range(EXTENDED_PICTOGRAPHIC, scalar, 0) != 0;
        let joined = if previous == CR && current == LF {
            true
        } else if matches!(previous, CR | LF | CONTROL) || matches!(current, CR | LF | CONTROL) {
            false
        } else {
            matches!(
                (previous, current),
                (L, L | V | LV | LVT)
                    | (LV | V, V | T)
                    | (LVT | T, T)
                    | (_, EXTEND | ZWJ | SPACING_MARK)
                    | (PREPEND, _)
            ) || (conjunct == 1 && indic == 2)
                || (pictographic && emoji == 2)
                || (previous == RI && current == RI && regional_odd)
        };

        regional_odd = current == RI && !regional_odd;
        emoji = if pictographic || (current == EXTEND && emoji == 1) {
            1
        } else if current == ZWJ && emoji == 1 {
            2
        } else {
            0
        };
        indic = if conjunct == 1 {
            1
        } else if conjunct == 3 && indic != 0 {
            2
        } else if conjunct == 2 {
            indic
        } else {
            0
        };
        previous = current;
        (byte != 0 && !joined).then_some(byte)
    });
    std::iter::once(0)
        .chain(interior)
        .chain((!text.is_empty()).then_some(text.len()))
}

pub fn native_grapheme_operation(name: &str, arguments: Vec<V>) -> V {
    match name {
        "std/text/grapheme::length"
        | "std/text/grapheme::clusters"
        | "std/text/grapheme::byteBoundaries" => {
            let [text] = arguments.as_slice() else {
                panic!("Rust ABI: grapheme unary arity")
            };
            let text = native_text(text);
            let boundaries = native_grapheme_boundaries(text);
            match name {
                "std/text/grapheme::length" => checked_int((boundaries.count() - 1) as i64),
                "std/text/grapheme::byteBoundaries" => V::Array(Shared::new(
                    boundaries.map(|byte| checked_int(byte as i64)).collect(),
                )),
                _ => {
                    let mut previous = 0;
                    V::Array(Shared::new(
                        boundaries
                            .skip(1)
                            .map(|byte| {
                                let cluster = V::text(&text[previous..byte]);
                                previous = byte;
                                cluster
                            })
                            .collect(),
                    ))
                }
            }
        }
        "std/text/grapheme::at" => {
            let [V::Int(index), text] = arguments.as_slice() else {
                panic!("Rust ABI: grapheme.at expects Int and String")
            };
            let text = native_text(text);
            let cluster = usize::try_from(*index).ok().and_then(|index| {
                let mut boundaries = native_grapheme_boundaries(text);
                let start = boundaries.nth(index)?;
                let end = boundaries.next()?;
                Some(V::text(&text[start..end]))
            });
            match cluster {
                Some(cluster) => V::adt("std/prelude::Just", cluster),
                None => V::adt("std/prelude::Nothing", V::Unit),
            }
        }
        "std/text/grapheme::slice" => {
            let [V::Int(start), V::Int(end), text] = arguments.as_slice() else {
                panic!("Rust ABI: grapheme.slice expects Int bounds and String")
            };
            let text = native_text(text);
            let (mut size, mut first, mut last) = (0, 0, 0);
            for (index, byte) in native_grapheme_boundaries(text).enumerate() {
                size = index as i64;
                if size == *start {
                    first = byte;
                }
                if size == *end {
                    last = byte;
                }
            }
            if *start < 0 || start > end || *end > size {
                native_left(V::adt(
                    "std/text/grapheme::InvalidGraphemeRange",
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
        _ => panic!("Rust ABI: unsupported grapheme operation"),
    }
}

#[cfg(test)]
mod grapheme_tests {
    use super::*;

    fn integer(value: &V) -> i64 {
        let V::Int(value) = value else { panic!() };
        *value
    }

    fn operation(name: &str, mut arguments: Vec<V>, text: &str) -> V {
        arguments.push(V::text(text));
        native_text_operation(&format!("std/text/grapheme::{name}"), arguments)
    }

    fn assert_clusters(text: &str, clusters: &[&str]) {
        let mut expected = vec![0];
        for cluster in clusters {
            expected.push(expected.last().unwrap() + cluster.len());
        }
        assert_eq!(
            native_grapheme_boundaries(text).collect::<Vec<_>>(),
            expected,
            "{text:?}"
        );
        assert_eq!(
            integer(&operation("length", vec![], text)),
            clusters.len() as i64
        );
        let V::Array(actual) = operation("byteBoundaries", vec![], text) else {
            panic!()
        };
        assert_eq!(
            actual.get().iter().map(integer).collect::<Vec<_>>(),
            expected.iter().map(|&byte| byte as i64).collect::<Vec<_>>()
        );
        let V::Array(actual) = operation("clusters", vec![], text) else {
            panic!()
        };
        assert_eq!(
            actual.get().iter().map(native_text).collect::<Vec<_>>(),
            clusters
        );
        assert_eq!(clusters.concat(), text);
    }

    #[test]
    fn all_pinned_extended_grapheme_break_cases_match_clusters_and_utf8_offsets() {
        let mut count = 0;
        for line in
            include_str!("../../../runtime/unicode/ucd/auxiliary/GraphemeBreakTest.txt").lines()
        {
            let value = line.split('#').next().unwrap().trim();
            if value.is_empty() {
                continue;
            }
            let clusters = value
                .split('÷')
                .filter_map(|group| {
                    let cluster = group
                        .split_whitespace()
                        .filter(|token| *token != "×")
                        .map(|hex| char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap())
                        .collect::<String>();
                    (!cluster.is_empty()).then_some(cluster)
                })
                .collect::<Vec<_>>();
            assert_clusters(
                &clusters.concat(),
                &clusters.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            count += 1;
        }
        assert!(count > 700);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn at_and_slice_preserve_graphemes_and_exact_invalid_range_payloads() {
        for clusters in [
            vec![],
            vec!["a"],
            vec!["A", "👍🏽", "e\u{301}"],
            vec!["\0", "\r\n", "é", "e\u{301}", "\u{10ffff}"],
            vec!["👩‍💻", "🇯🇵", "🇺", "क्‍क", "\u{600}a"],
        ] {
            let text = clusters.concat();
            assert_clusters(&text, &clusters);
            let size = clusters.len() as i64;
            let mut indices = vec![-9_007_199_254_740_991, -1];
            indices.extend(0..=size + 1);
            indices.push(9_007_199_254_740_991);
            for &start in &indices {
                let at = operation("at", vec![V::Int(start)], &text);
                if (0..size).contains(&start) {
                    assert!(at.tagged("std/prelude::Just"));
                    assert_eq!(native_text(&at.payload()), clusters[start as usize]);
                } else {
                    assert!(at.tagged("std/prelude::Nothing"));
                }
                for &end in &indices {
                    let slice = operation("slice", vec![V::Int(start), V::Int(end)], &text);
                    if 0 <= start && start <= end && end <= size {
                        assert!(slice.tagged("std/prelude::Right"));
                        assert_eq!(
                            native_text(&slice.payload()),
                            clusters[start as usize..end as usize].concat()
                        );
                    } else {
                        assert!(slice.tagged("std/prelude::Left"));
                        let error = slice.payload();
                        assert!(error.tagged("std/text/grapheme::InvalidGraphemeRange"));
                        let payload = error.payload();
                        assert_eq!(integer(&payload.field("start")), start);
                        assert_eq!(integer(&payload.field("end")), end);
                        assert_eq!(integer(&payload.field("length")), size);
                    }
                }
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn long_combining_emoji_indic_and_regional_runs_keep_streaming_state() {
        let combined = format!("a{}", "\u{301}".repeat(100_000));
        assert_clusters(&combined, &[&combined]);
        let emoji = format!("👩{}‍💻", "\u{301}".repeat(100_000));
        assert_clusters(&emoji, &[&emoji]);
        let indic = format!("क{}्क", "\u{93c}".repeat(100_000));
        assert_clusters(&indic, &[&indic]);
        let flags = "🇦".repeat(100_001);
        let mut expected = vec!["🇦🇦"; 50_000];
        expected.push("🇦");
        assert_clusters(&flags, &expected);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
