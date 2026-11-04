// Keep parsing, thread priority, and capture resets aligned with runtime/ts/src/regex.ts.
pub struct NativeRegex {
    instructions: Vec<NativeRegexInstruction>,
    start: usize,
    capture_count: usize,
    counter_count: usize,
    named: Vec<(String, usize)>,
    options: NativeRegexOptions,
}

impl TraceValue for NativeRegex {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
struct NativeRegexOptions {
    case_insensitive: bool,
    multiline: bool,
    dot_matches_newline: bool,
}

#[derive(Clone, Copy)]
enum NativeRegexAssertion {
    Start,
    End,
    AbsoluteStart,
    AbsoluteEnd,
}

#[derive(Clone)]
enum NativeRegexProperty {
    Alphabetic,
    Whitespace,
    Categories(u32, u32),
}

#[derive(Clone)]
enum NativeRegexTerm {
    Range(u32, u32),
    Digit(bool),
    Space(bool),
    Word(bool),
    Property(NativeRegexProperty, bool),
}

#[derive(Clone)]
struct NativeRegexMatcher {
    terms: Vec<NativeRegexTerm>,
    negated: bool,
    dot: bool,
}

fn native_regex_category(scalar: char) -> u32 {
    native_unicode_range(native_unicode_data::GENERAL_CATEGORY, scalar, 28)
}

fn native_regex_alphabetic(scalar: char) -> bool {
    native_unicode_range(native_unicode_data::ALPHABETIC, scalar, 0) != 0
}

fn native_regex_whitespace(scalar: char) -> bool {
    native_unicode_range(native_unicode_data::WHITE_SPACE, scalar, 0) != 0
}

fn native_regex_property(name: &str) -> Option<NativeRegexProperty> {
    use NativeRegexProperty::*;
    let normalized: String = name
        .chars()
        .filter(|c| !matches!(c, '_' | '-' | ' '))
        .map(|c| c.to_ascii_lowercase())
        .collect();
    Some(match normalized.as_str() {
        "alphabetic" => Alphabetic,
        "whitespace" => Whitespace,
        "uppercaseletter" | "lu" => Categories(0, 0),
        "lowercaseletter" | "ll" => Categories(1, 1),
        "titlecaseletter" | "lt" => Categories(2, 2),
        "modifierletter" | "lm" => Categories(3, 3),
        "otherletter" | "lo" => Categories(4, 4),
        "nonspacingmark" | "mn" => Categories(5, 5),
        "spacingmark" | "mc" => Categories(6, 6),
        "enclosingmark" | "me" => Categories(7, 7),
        "decimalnumber" | "nd" => Categories(8, 8),
        "letternumber" | "nl" => Categories(9, 9),
        "othernumber" | "no" => Categories(10, 10),
        "connectorpunctuation" | "pc" => Categories(11, 11),
        "dashpunctuation" | "pd" => Categories(12, 12),
        "openpunctuation" | "ps" => Categories(13, 13),
        "closepunctuation" | "pe" => Categories(14, 14),
        "initialpunctuation" | "pi" => Categories(15, 15),
        "finalpunctuation" | "pf" => Categories(16, 16),
        "otherpunctuation" | "po" => Categories(17, 17),
        "mathsymbol" | "sm" => Categories(18, 18),
        "currencysymbol" | "sc" => Categories(19, 19),
        "modifiersymbol" | "sk" => Categories(20, 20),
        "othersymbol" | "so" => Categories(21, 21),
        "spaceseparator" | "zs" => Categories(22, 22),
        "lineseparator" | "zl" => Categories(23, 23),
        "paragraphseparator" | "zp" => Categories(24, 24),
        "control" | "cc" => Categories(25, 25),
        "format" | "cf" => Categories(26, 26),
        "privateuse" | "co" => Categories(27, 27),
        "unassigned" | "cn" => Categories(28, 28),
        "letter" | "l" => Categories(0, 4),
        "mark" | "m" => Categories(5, 7),
        "number" | "n" => Categories(8, 10),
        "punctuation" | "p" => Categories(11, 17),
        "symbol" | "s" => Categories(18, 21),
        "separator" | "z" => Categories(22, 24),
        "other" | "c" => Categories(25, 28),
        _ => return None,
    })
}

fn native_regex_fold_equivalents(scalar: char) -> &'static [char] {
    static GROUPS: std::sync::OnceLock<BTreeMap<char, Vec<char>>> = std::sync::OnceLock::new();
    let groups = GROUPS.get_or_init(|| {
        let mut adjacency: BTreeMap<char, Vec<char>> = BTreeMap::new();
        for &(from, mapping) in native_text_case_data::SIMPLE_FOLD {
            let from = char::from_u32(from).unwrap();
            let to = char::from_u32(mapping[0]).unwrap();
            adjacency.entry(from).or_default().push(to);
            adjacency.entry(to).or_default().push(from);
        }
        let mut groups = BTreeMap::new();
        for &point in adjacency.keys() {
            if groups.contains_key(&point) {
                continue;
            }
            let mut group = vec![point];
            let mut cursor = 0;
            while cursor < group.len() {
                for &next in &adjacency[&group[cursor]] {
                    if !group.contains(&next) {
                        group.push(next);
                    }
                }
                cursor += 1;
            }
            for &member in &group {
                groups.insert(member, group.clone());
            }
        }
        groups
    });
    groups.get(&scalar).map_or(&[], Vec::as_slice)
}

fn native_regex_line_terminator(scalar: char) -> bool {
    matches!(scalar, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

impl NativeRegexMatcher {
    fn matches(&self, scalar: char, options: NativeRegexOptions) -> bool {
        if self.dot {
            return options.dot_matches_newline || !native_regex_line_terminator(scalar);
        }
        let folded = if options.case_insensitive {
            native_regex_fold_equivalents(scalar)
        } else {
            &[]
        };
        let equivalents = if folded.is_empty() {
            std::slice::from_ref(&scalar)
        } else {
            folded
        };
        let matched = self.terms.iter().any(|term| {
            let negated = match term {
                NativeRegexTerm::Range(_, _) => false,
                NativeRegexTerm::Digit(n)
                | NativeRegexTerm::Space(n)
                | NativeRegexTerm::Word(n)
                | NativeRegexTerm::Property(_, n) => *n,
            };
            let matched = equivalents.iter().any(|&point| match term {
                NativeRegexTerm::Range(start, end) => (*start..=*end).contains(&(point as u32)),
                NativeRegexTerm::Digit(_) => point.is_ascii_digit(),
                NativeRegexTerm::Space(_) => native_regex_whitespace(point),
                NativeRegexTerm::Word(_) => {
                    native_regex_alphabetic(point)
                        || matches!(native_regex_category(point), 5..=8 | 11)
                }
                NativeRegexTerm::Property(property, _) => match property {
                    NativeRegexProperty::Alphabetic => native_regex_alphabetic(point),
                    NativeRegexProperty::Whitespace => native_regex_whitespace(point),
                    NativeRegexProperty::Categories(start, end) => {
                        (*start..=*end).contains(&native_regex_category(point))
                    }
                },
            });
            matched != negated
        });
        matched != self.negated
    }
}

fn native_regex_record(fields: Vec<(&str, V)>) -> V {
    V::Record(Shared::new(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    ))
}

fn native_regex_compile(options: NativeRegexOptions, pattern: &str) -> V {
    let mut parser = NativeRegexParser::new(pattern);
    match parser.parse() {
        Err(error) => native_left(error.value()),
        Ok(node) => {
            let mut regex = NativeRegex {
                instructions: vec![],
                start: 0,
                capture_count: parser.captures,
                counter_count: 0,
                named: parser.named,
                options,
            };
            let fragment = regex.compile_node(&node);
            let accept = regex.emit(NativeRegexInstruction::Accept);
            regex.patch(fragment.exits, accept);
            regex.start = fragment.start;
            native_right(V::Regex(Shared::new(regex)))
        }
    }
}

fn native_regex_capture(text: &str, boundaries: &[usize], start: usize, end: usize) -> V {
    let (start, end) = (boundaries[start], boundaries[end]);
    native_regex_record(vec![
        (
            "span",
            native_regex_record(vec![
                ("start", V::Int(start as i64)),
                ("end", V::Int(end as i64)),
            ]),
        ),
        ("text", V::text(&text[start..end])),
    ])
}

fn native_regex_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries: Vec<_> = text.char_indices().map(|s| s.0).collect();
    boundaries.push(text.len());
    boundaries
}

fn native_regex_public_match(
    regex: &NativeRegex,
    text: &str,
    boundaries: &[usize],
    matched: NativeRegexMatch,
) -> V {
    let whole = native_regex_capture(text, boundaries, matched.start, matched.end);
    let captures: Vec<_> = matched
        .captures
        .chunks_exact(2)
        .map(|slots| match (slots[0], slots[1]) {
            (Some(start), Some(end)) => V::adt(
                "std/prelude::Just",
                native_regex_capture(text, boundaries, start, end),
            ),
            _ => V::adt("std/prelude::Nothing", V::Unit),
        })
        .collect();
    let mut named = native_map_operation("std/map::empty", vec![], vec![V::Unit]);
    if !regex.named.is_empty() {
        let dictionaries = vec![
            native_operator_dictionary("std/string::Eq"),
            native_hash_dictionary("std/string::Hash"),
        ];
        for (name, index) in &regex.named {
            named = native_map_operation(
                "std/map::insert",
                dictionaries.clone(),
                vec![V::text(name), captures[*index].clone(), named],
            );
        }
    }
    native_regex_record(vec![
        ("span", whole.field("span")),
        ("text", whole.field("text")),
        ("captures", V::Array(Shared::new(captures))),
        ("named", named),
    ])
}

pub fn native_regex_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/regex::defaultOptions", [V::Unit]) => native_regex_record(vec![
            ("caseInsensitive", V::Bool(false)),
            ("multiline", V::Bool(false)),
            ("dotMatchesNewline", V::Bool(false)),
        ]),
        ("std/regex::compile", [pattern]) => {
            native_regex_compile(NativeRegexOptions::default(), native_text(pattern))
        }
        ("std/regex::compileWith", [options, pattern]) => native_regex_compile(
            NativeRegexOptions {
                case_insensitive: options.field("caseInsensitive").boolean(),
                multiline: options.field("multiline").boolean(),
                dot_matches_newline: options.field("dotMatchesNewline").boolean(),
            },
            native_text(pattern),
        ),
        ("std/regex::isMatch" | "std/regex::find", [V::Regex(regex), text]) => {
            let text = native_text(text);
            let matched = regex.get().find(&text.chars().collect::<Vec<_>>());
            if name == "std/regex::isMatch" {
                return V::Bool(matched.is_some());
            }
            match matched {
                Some(matched) => V::adt(
                    "std/prelude::Just",
                    native_regex_public_match(
                        regex.get(),
                        text,
                        &native_regex_boundaries(text),
                        matched,
                    ),
                ),
                None => V::adt("std/prelude::Nothing", V::Unit),
            }
        }
        ("std/regex::findAll" | "std/regex::split", [V::Regex(regex), text]) => {
            let text = native_text(text);
            let boundaries = native_regex_boundaries(text);
            let matches = regex.get().find_all(&text.chars().collect::<Vec<_>>());
            if name == "std/regex::findAll" {
                return V::Array(Shared::new(
                    matches
                        .into_iter()
                        .map(|matched| {
                            native_regex_public_match(regex.get(), text, &boundaries, matched)
                        })
                        .collect(),
                ));
            }
            let mut parts = Vec::new();
            let mut previous = 0;
            for matched in matches {
                parts.push(V::text(&text[previous..boundaries[matched.start]]));
                previous = boundaries[matched.end];
            }
            parts.push(V::text(&text[previous..]));
            V::Array(Shared::new(parts))
        }
        (
            "std/regex::replaceAll" | "std/regex::replaceAllWith",
            [V::Regex(regex), replacement, text],
        ) => {
            let text = native_text(text);
            let boundaries = native_regex_boundaries(text);
            let matches = regex.get().find_all(&text.chars().collect::<Vec<_>>());
            let mut output = String::new();
            let mut previous = 0;
            for matched in matches {
                output.push_str(&text[previous..boundaries[matched.start]]);
                previous = boundaries[matched.end];
                if name == "std/regex::replaceAllWith" {
                    let value = replacement.apply(native_regex_public_match(
                        regex.get(),
                        text,
                        &boundaries,
                        matched,
                    ));
                    output.push_str(native_text(&value));
                } else {
                    output.push_str(native_text(replacement));
                }
            }
            output.push_str(&text[previous..]);
            V::text(&output)
        }
        ("std/regex::escape", [text]) => {
            let mut escaped = String::new();
            for scalar in native_text(text).chars() {
                match scalar {
                    '\\' | '.' | '^' | '$' | '|' | '?' | '*' | '+' | '(' | ')' | '[' | ']'
                    | '{' | '}' => {
                        escaped.push('\\');
                        escaped.push(scalar);
                    }
                    '\n' => escaped.push_str("\\n"),
                    '\r' => escaped.push_str("\\r"),
                    '\t' => escaped.push_str("\\t"),
                    '\0' => escaped.push_str("\\u{0}"),
                    _ => escaped.push(scalar),
                }
            }
            V::text(&escaped)
        }
        _ => panic!("Rust ABI: unadmitted Regex operation or arguments"),
    }
}
