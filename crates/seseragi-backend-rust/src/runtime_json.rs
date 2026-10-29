fn native_json_invalid(offset: usize, message: &str) -> V {
    V::adt(
        "std/json::InvalidJsonSyntax",
        V::Record(Shared::new(BTreeMap::from([
            (
                "offset".into(),
                checked_int(i64::try_from(offset).expect("JSON offset fits Int")),
            ),
            ("message".into(), V::text(message)),
        ]))),
    )
}
struct NativeJsonParser<'a> {
    source: &'a str,
    index: usize,
}
impl NativeJsonParser<'_> {
    fn byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.index).copied()
    }
    fn whitespace(&mut self) {
        while matches!(self.byte(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.index += 1;
        }
    }
    fn fail(&self, message: &str) -> V {
        native_json_invalid(self.index, message)
    }
    fn literal(&mut self, text: &str, value: V) -> Result<V, V> {
        if !self.source[self.index..].starts_with(text) {
            return Err(self.fail(&format!("expected {text}")));
        }
        self.index += text.len();
        Ok(value)
    }
    fn number(&mut self) -> Result<V, V> {
        let start = self.index;
        if self.byte() == Some(b'-') {
            self.index += 1;
        }
        match self.byte() {
            Some(b'0') => {
                self.index += 1;
                if self.byte().is_some_and(|b| b.is_ascii_digit()) {
                    return Err(self.fail("leading zero in JSON number"));
                }
            }
            Some(b'1'..=b'9') => {
                while self.byte().is_some_and(|b| b.is_ascii_digit()) {
                    self.index += 1;
                }
            }
            _ => return Err(self.fail("expected digit in JSON number")),
        }
        if self.byte() == Some(b'.') {
            self.index += 1;
            if !self.byte().is_some_and(|b| b.is_ascii_digit()) {
                return Err(self.fail("expected digit after decimal point"));
            }
            while self.byte().is_some_and(|b| b.is_ascii_digit()) {
                self.index += 1;
            }
        }
        if matches!(self.byte(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.byte(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            if !self.byte().is_some_and(|b| b.is_ascii_digit()) {
                return Err(self.fail("expected exponent digit"));
            }
            while self.byte().is_some_and(|b| b.is_ascii_digit()) {
                self.index += 1;
            }
        }
        let parsed = native_decimal_parse(&self.source[start..self.index]);
        assert!(
            parsed.tagged("std/prelude::Right"),
            "Rust ABI: checked JSON decimal syntax"
        );
        Ok(V::adt("std/json::JsonNumber", parsed.payload()))
    }
    fn hex(&mut self, start: usize) -> Result<u32, V> {
        let Some(digits) = self.source.as_bytes().get(self.index..self.index + 4) else {
            return Err(native_json_invalid(start, "invalid Unicode escape"));
        };
        if !digits.iter().all(u8::is_ascii_hexdigit) {
            return Err(native_json_invalid(start, "invalid Unicode escape"));
        }
        let value = u32::from_str_radix(std::str::from_utf8(digits).unwrap(), 16).unwrap();
        self.index += 4;
        Ok(value)
    }
    fn escape(&mut self) -> Result<char, V> {
        let start = self.index;
        self.index += 1;
        let escaped = self.byte();
        if escaped.is_some() {
            self.index += 1;
        }
        Ok(match escaped {
            Some(b'"') => '"',
            Some(b'\\') => '\\',
            Some(b'/') => '/',
            Some(b'b') => '\u{8}',
            Some(b'f') => '\u{c}',
            Some(b'n') => '\n',
            Some(b'r') => '\r',
            Some(b't') => '\t',
            Some(b'u') => {
                let first = self.hex(start)?;
                if (0xd800..=0xdbff).contains(&first) {
                    if !self.source.as_bytes()[self.index..].starts_with(b"\\u") {
                        return Err(native_json_invalid(
                            start,
                            "unpaired escaped high surrogate",
                        ));
                    }
                    self.index += 2;
                    let second = self.hex(start)?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(native_json_invalid(
                            start,
                            "unpaired escaped high surrogate",
                        ));
                    }
                    char::from_u32(0x10000 + ((first - 0xd800) << 10) + second - 0xdc00).unwrap()
                } else if (0xdc00..=0xdfff).contains(&first) {
                    return Err(native_json_invalid(start, "unpaired escaped low surrogate"));
                } else {
                    char::from_u32(first).unwrap()
                }
            }
            _ => return Err(native_json_invalid(start, "invalid JSON escape")),
        })
    }
    fn string(&mut self) -> Result<String, V> {
        assert_eq!(self.byte(), Some(b'"'));
        self.index += 1;
        let mut output = String::new();
        while let Some(byte) = self.byte() {
            if byte == b'"' {
                self.index += 1;
                return Ok(output);
            }
            if byte == b'\\' {
                output.push(self.escape()?);
                continue;
            }
            if byte <= 0x1f {
                return Err(self.fail("unescaped control character"));
            }
            // Source String already consists of Unicode scalars; escaped UTF-16
            // code units are checked separately before constructing a char.
            let value = self.source[self.index..].chars().next().unwrap();
            output.push(value);
            self.index += value.len_utf8();
        }
        Err(self.fail("unterminated JSON string"))
    }
    fn field(
        &mut self,
        names: &mut std::collections::BTreeSet<String>,
        path: &[V],
    ) -> Result<String, V> {
        if self.byte() != Some(b'"') {
            return Err(self.fail("expected object field"));
        }
        let name = self.string()?;
        if !names.insert(name.clone()) {
            return Err(V::adt(
                "std/json::DuplicateJsonField",
                V::Record(Shared::new(BTreeMap::from([
                    ("field".into(), V::text(&name)),
                    ("path".into(), V::Array(Shared::new(path.to_vec()))),
                ]))),
            ));
        }
        self.whitespace();
        if self.byte() != Some(b':') {
            return Err(self.fail("expected ':'"));
        }
        self.index += 1;
        self.whitespace();
        Ok(name)
    }
}
enum NativeJsonFrame {
    Array {
        values: Vec<V>,
        path_len: usize,
    },
    Object {
        entries: Vec<(String, V)>,
        names: std::collections::BTreeSet<String>,
        key: String,
        path_len: usize,
    },
}
fn native_json_object(entries: Vec<(String, V)>) -> V {
    let dictionaries = vec![
        native_operator_dictionary("std/string::Eq"),
        native_hash_dictionary("std/string::Hash"),
    ];
    let mut map = native_map_operation("std/map::empty", vec![], vec![V::Unit]);
    for (name, value) in entries {
        map = native_map_operation(
            "std/map::insert",
            dictionaries.clone(),
            vec![V::text(&name), value, map],
        );
    }
    V::adt("std/json::JsonObject", map)
}
fn native_json_parse_value(source: &str) -> Result<V, V> {
    let mut parser = NativeJsonParser { source, index: 0 };
    let mut frames = Vec::new();
    let mut path = Vec::new();
    parser.whitespace();
    'next: loop {
        let mut value = match parser.byte() {
            Some(b'"') => V::adt("std/json::JsonString", V::text(&parser.string()?)),
            Some(b't') => parser.literal("true", V::adt("std/json::JsonBool", V::Bool(true)))?,
            Some(b'f') => parser.literal("false", V::adt("std/json::JsonBool", V::Bool(false)))?,
            Some(b'n') => parser.literal("null", V::adt("std/json::JsonNull", V::Unit))?,
            Some(b'-' | b'0'..=b'9') => parser.number()?,
            Some(b'[') => {
                parser.index += 1;
                parser.whitespace();
                if parser.byte() == Some(b']') {
                    parser.index += 1;
                    V::adt("std/json::JsonArray", V::Array(Shared::new(vec![])))
                } else {
                    frames.push(NativeJsonFrame::Array {
                        values: vec![],
                        path_len: path.len(),
                    });
                    path.push(V::adt("std/json::JsonIndex", V::Int(0)));
                    continue 'next;
                }
            }
            Some(b'{') => {
                parser.index += 1;
                parser.whitespace();
                if parser.byte() == Some(b'}') {
                    parser.index += 1;
                    native_json_object(vec![])
                } else {
                    let mut names = std::collections::BTreeSet::new();
                    let key = parser.field(&mut names, &path)?;
                    let path_len = path.len();
                    path.push(V::adt("std/json::JsonField", V::text(&key)));
                    frames.push(NativeJsonFrame::Object {
                        entries: vec![],
                        names,
                        key,
                        path_len,
                    });
                    continue 'next;
                }
            }
            None => return Err(parser.fail("expected JSON value")),
            _ => return Err(parser.fail("invalid JSON value")),
        };
        loop {
            match frames.last_mut() {
                Some(NativeJsonFrame::Array { values, path_len }) => {
                    path.truncate(*path_len);
                    values.push(value);
                    parser.whitespace();
                    if parser.byte() == Some(b']') {
                        parser.index += 1;
                        let Some(NativeJsonFrame::Array { values, .. }) = frames.pop() else {
                            unreachable!()
                        };
                        value = V::adt("std/json::JsonArray", V::Array(Shared::new(values)));
                    } else {
                        if parser.byte() != Some(b',') {
                            return Err(parser.fail("expected ',' or ']'"));
                        }
                        parser.index += 1;
                        parser.whitespace();
                        if parser.byte() == Some(b']') {
                            return Err(parser.fail("trailing array comma"));
                        }
                        path.push(V::adt(
                            "std/json::JsonIndex",
                            checked_int(i64::try_from(values.len()).expect("JSON index fits Int")),
                        ));
                        continue 'next;
                    }
                }
                Some(NativeJsonFrame::Object {
                    entries,
                    names,
                    key,
                    path_len,
                }) => {
                    path.truncate(*path_len);
                    entries.push((std::mem::take(key), value));
                    parser.whitespace();
                    if parser.byte() == Some(b'}') {
                        parser.index += 1;
                        let Some(NativeJsonFrame::Object { entries, .. }) = frames.pop() else {
                            unreachable!()
                        };
                        value = native_json_object(entries);
                    } else {
                        if parser.byte() != Some(b',') {
                            return Err(parser.fail("expected ',' or '}'"));
                        }
                        parser.index += 1;
                        parser.whitespace();
                        if parser.byte() == Some(b'}') {
                            return Err(parser.fail("trailing object comma"));
                        }
                        *key = parser.field(names, &path)?;
                        path.push(V::adt("std/json::JsonField", V::text(key)));
                        continue 'next;
                    }
                }
                None => {
                    parser.whitespace();
                    if parser.index != source.len() {
                        return Err(parser.fail("unexpected trailing input"));
                    }
                    return Ok(value);
                }
            }
        }
    }
}
fn native_json_parse(source: &str) -> V {
    match native_json_parse_value(source) {
        Ok(value) => native_right(value),
        Err(error) => native_left(error),
    }
}
fn native_json_quote(value: &str, output: &mut String) {
    output.push('"');
    for c in value.chars() {
        match c {
            '\u{8}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{c}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\0'..='\u{1f}' => {
                use std::fmt::Write;
                write!(output, "\\u{:04X}", c as u32).unwrap();
            }
            _ => output.push(c),
        }
    }
    output.push('"');
}
fn native_json_stringify(value: V) -> String {
    enum Task {
        Value(V),
        Quoted(V),
        Text(&'static str),
    }
    let mut tasks = vec![Task::Value(value)];
    let mut output = String::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Text(value) => output.push_str(value),
            Task::Quoted(V::Text(value)) => native_json_quote(value.get(), &mut output),
            Task::Quoted(_) => panic!("Rust ABI: JSON object key"),
            Task::Value(value) => {
                let V::Adt(data) = value else {
                    panic!("Rust ABI: expected Json")
                };
                let (tag, payload) = data.get();
                match *tag {
                    "std/json::JsonNull" => output.push_str("null"),
                    "std/json::JsonBool" => {
                        let V::Bool(value) = payload else {
                            panic!("Rust ABI: JSON Bool")
                        };
                        output.push_str(if *value { "true" } else { "false" });
                    }
                    "std/json::JsonNumber" => {
                        let V::Decimal(value) = payload else {
                            panic!("Rust ABI: JSON Decimal")
                        };
                        output.push_str(&native_decimal_format(value.get()));
                    }
                    "std/json::JsonString" => {
                        let V::Text(value) = payload else {
                            panic!("Rust ABI: JSON String")
                        };
                        native_json_quote(value.get(), &mut output);
                    }
                    "std/json::JsonArray" => {
                        let V::Array(values) = payload else {
                            panic!("Rust ABI: JSON Array")
                        };
                        output.push('[');
                        tasks.push(Task::Text("]"));
                        for (index, value) in values.get().iter().enumerate().rev() {
                            tasks.push(Task::Value(value.clone()));
                            if index > 0 {
                                tasks.push(Task::Text(","));
                            }
                        }
                    }
                    "std/json::JsonObject" => {
                        let map = native_map_value(payload);
                        let mut id = map.first;
                        let mut entries = vec![];
                        while let Some(current) = id {
                            let entry = native_map_entry(map, current);
                            entries.push((entry.key.clone(), entry.value.clone()));
                            id = entry.next;
                        }
                        output.push('{');
                        tasks.push(Task::Text("}"));
                        for (index, (key, value)) in entries.into_iter().enumerate().rev() {
                            tasks.push(Task::Value(value));
                            tasks.push(Task::Text(":"));
                            tasks.push(Task::Quoted(key));
                            if index > 0 {
                                tasks.push(Task::Text(","));
                            }
                        }
                    }
                    _ => panic!("Rust ABI: unknown Json tag"),
                }
            }
        }
    }
    output
}
fn native_json_operation(name: &'static str, mut arguments: Vec<V>) -> V {
    match name {
        "std/json::parse" => {
            let V::Text(value) = arguments.remove(0) else {
                panic!("Rust ABI: JSON parse String")
            };
            native_json_parse(value.get())
        }
        "std/json::stringify" => V::text(&native_json_stringify(arguments.remove(0))),
        "std/json::JsonNull" => V::adt(name, V::Unit),
        "std/json::JsonBool"
        | "std/json::JsonNumber"
        | "std/json::JsonString"
        | "std/json::JsonArray"
        | "std/json::JsonObject"
        | "std/json::JsonField"
        | "std/json::JsonIndex"
        | "std/json::InvalidJsonSyntax"
        | "std/json::DuplicateJsonField" => V::adt(name, arguments.remove(0)),
        _ => panic!("Rust ABI: unsupported JSON operation"),
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;

    #[test]
    fn json_keeps_exact_numbers_unicode_scalars_order_and_minimal_escapes() {
        clear_dictionaries();
        for (source, expected) in [
            ("null", "null"),
            (" true\n", "true"),
            ("false", "false"),
            ("-0.000e99999999999999999999999", "0"),
            ("1.25E-3", "0.00125"),
            (
                "9007199254740992000000000000001.100",
                "9007199254740992000000000000001.1",
            ),
            (
                r#"{ "snow":"雪🙂", "a":[true,null,1.200] }"#,
                r#"{"snow":"雪🙂","a":[true,null,1.2]}"#,
            ),
            (r#""\u96ea\uD83D\uDE42\/""#, r#""雪🙂/""#),
            (
                r#""\u0000\u000b\b\t\n\f\r\"\\""#,
                r#""\u0000\u000B\b\t\n\f\r\"\\""#,
            ),
            ("[]", "[]"),
            ("{}", "{}"),
            ("[1,2,3]", "[1,2,3]"),
        ] {
            let parsed = native_json_parse(source);
            assert!(
                parsed.tagged("std/prelude::Right"),
                "{source}:{}",
                parsed.render()
            );
            assert_eq!(
                native_json_stringify(parsed.payload()),
                expected,
                "{source}"
            );
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn syntax_keeps_the_first_utf8_byte_and_exact_message() {
        clear_dictionaries();
        for (source, offset, message) in [
            ("", 0, "expected JSON value"),
            ("tru", 0, "expected true"),
            ("truex", 4, "unexpected trailing input"),
            ("+1", 0, "invalid JSON value"),
            ("01", 1, "leading zero in JSON number"),
            ("-", 1, "expected digit in JSON number"),
            ("1.", 2, "expected digit after decimal point"),
            ("1e+", 3, "expected exponent digit"),
            ("[1,]", 3, "trailing array comma"),
            (r#"{"a":1,}"#, 7, "trailing object comma"),
            (r#"{x:1}"#, 1, "expected object field"),
            (r#"{"x" 1}"#, 5, "expected ':'"),
            ("[1 2]", 3, "expected ',' or ']'"),
            ("[0", 2, "expected ',' or ']'"),
            ("\"雪", 4, "unterminated JSON string"),
            ("\"a\n\"", 2, "unescaped control character"),
            (r#""雫\x""#, 4, "invalid JSON escape"),
            (r#""\u12X4""#, 1, "invalid Unicode escape"),
            (r#""\uD800""#, 1, "unpaired escaped high surrogate"),
            (r#""\uDC00""#, 1, "unpaired escaped low surrogate"),
            (r#""\uD800\u0041""#, 1, "unpaired escaped high surrogate"),
            (r#"["雫",]"#, 7, "trailing array comma"),
        ] {
            let result = native_json_parse(source);
            assert!(result.tagged("std/prelude::Left"), "{source}");
            let error = result.payload();
            assert!(error.tagged("std/json::InvalidJsonSyntax"), "{source}");
            assert_eq!(
                error.payload().field("offset").render(),
                offset.to_string(),
                "{source}"
            );
            assert_eq!(
                error.payload().field("message").render(),
                message,
                "{source}"
            );
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn duplicate_fields_use_decoded_names_and_the_enclosing_root_path_before_colon() {
        clear_dictionaries();
        let result = native_json_parse(r#"{"root":[{"雪":1,"\u96EA":}]}"#);
        assert!(result.tagged("std/prelude::Left"));
        let error = result.payload();
        assert!(error.tagged("std/json::DuplicateJsonField"));
        assert_eq!(error.payload().field("field").render(), "雪");
        let path = error.payload().field("path");
        let V::Array(values) = &path else { panic!() };
        assert_eq!(values.get().len(), 2);
        assert!(path.index(0).tagged("std/json::JsonField"));
        assert_eq!(path.index(0).payload().render(), "root");
        assert!(path.index(1).tagged("std/json::JsonIndex"));
        assert_eq!(path.index(1).payload().render(), "0");
        drop((result, error, path));
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn deep_data_uses_heap_frames_and_all_parser_error_paths_release_partial_values() {
        clear_dictionaries();
        let depth = 20_000;
        let text = "[".repeat(depth) + "0" + &"]".repeat(depth);
        let parsed = native_json_parse(&text);
        assert!(parsed.tagged("std/prelude::Right"));
        assert_eq!(native_json_stringify(parsed.payload()), text);
        drop(parsed);
        let partial =
            "[".repeat(depth) + r#"{"ok":"retained only during parse","bad":true,"bad":null}"#;
        let bad = native_json_parse(&partial);
        assert!(bad.tagged("std/prelude::Left"));
        drop(bad);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn saved_json_keeps_ref_cycles_and_host_limit_defects_reclaim_every_value() {
        enable_cycle_collection();
        clear_dictionaries();
        let parsed = native_json_parse(r#"{"answer":42,"snow":"雪🙂"}"#).payload();
        let reference = V::reference(parsed.clone());
        let formatter = V::fun(vec![reference.clone(), parsed.clone()], |c, _| {
            V::text(&native_json_stringify(c[1].clone()))
        });
        assert_eq!(
            formatter.apply(V::Unit).render(),
            r#"{"answer":42,"snow":"雪🙂"}"#
        );
        reference.write_reference(V::Tuple(Shared::new(vec![
            reference.clone(),
            formatter.clone(),
            parsed.clone(),
        ])));
        drop((reference, formatter, parsed));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
        let value =
            native_json_parse(r#"{"big":1e999999999999999999999999999999999999,"later":["雪"]}"#)
                .payload();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| native_json_stringify(
                value
            )))
            .is_err()
        );
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
