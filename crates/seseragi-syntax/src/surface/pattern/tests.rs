use super::*;
use crate::{parse_cst, parse_diagnostics, parse_surface_ast, SurfaceDecl};

fn parse_pattern(source: &str) -> SurfacePattern {
    let _scope = crate::resource::ParseScope::new();
    let stream = crate::lex("pattern.ssrg", source);
    let end = stream
        .tokens
        .iter()
        .position(|token| token.kind == TokenKind::Eof)
        .unwrap();
    parse_pattern_range(&stream.tokens, 0, end)
}

fn name(name: &str, start: usize) -> SurfacePattern {
    let span = ByteSpan {
        start,
        end: start + name.len(),
    };
    SurfacePattern::Name {
        name: name.to_owned(),
        name_span: span,
        span,
    }
}

fn constructor(
    name: &str,
    start: usize,
    end: usize,
    argument: Option<SurfacePattern>,
) -> SurfacePattern {
    SurfacePattern::Constructor {
        name: name.to_owned(),
        name_span: ByteSpan {
            start,
            end: start + name.len(),
        },
        argument: argument.map(Box::new),
        span: ByteSpan { start, end },
    }
}

#[test]
fn constructor_prefixes_preserve_names_spans_and_terminal_patterns() {
    assert_eq!(parse_pattern("Leaf"), constructor("Leaf", 0, 4, None));
    assert_eq!(
        parse_pattern("Branch tree.Leaf value"),
        constructor(
            "Branch",
            0,
            22,
            Some(constructor("tree.Leaf", 7, 22, Some(name("value", 17))))
        )
    );
    assert_eq!(
        parse_pattern("tree.Branch Leaf"),
        constructor(
            "tree.Branch",
            0,
            16,
            Some(constructor("Leaf", 12, 16, None))
        )
    );
    assert_eq!(
        parse_pattern("Branch (Leaf value)"),
        constructor(
            "Branch",
            0,
            19,
            Some(constructor("Leaf", 8, 18, Some(name("value", 13))))
        )
    );
    let field = SurfaceRecordPatternField {
        name: "field".to_owned(),
        name_span: ByteSpan { start: 14, end: 19 },
        optional: false,
        pattern: name("field", 14),
        span: ByteSpan { start: 14, end: 19 },
    };
    assert_eq!(
        parse_pattern("Branch Node { field }"),
        constructor(
            "Branch",
            0,
            21,
            Some(SurfacePattern::Struct {
                name: "Node".to_owned(),
                name_span: ByteSpan { start: 7, end: 11 },
                fields: vec![field],
                span: ByteSpan { start: 7, end: 21 },
            })
        )
    );
    for (source, field_start, record_start, record_end) in [
        ("Branch ({ field })", 10, 8, 17),
        ("tree.Node { field }", 12, 10, 19),
    ] {
        let constructor_name = source.split(' ').next().unwrap();
        let span = ByteSpan {
            start: field_start,
            end: field_start + 5,
        };
        assert_eq!(
            parse_pattern(source),
            constructor(
                constructor_name,
                0,
                source.len(),
                Some(SurfacePattern::Record {
                    fields: vec![SurfaceRecordPatternField {
                        name: "field".to_owned(),
                        name_span: span,
                        optional: false,
                        pattern: name("field", field_start),
                        span,
                    }],
                    span: ByteSpan {
                        start: record_start,
                        end: record_end,
                    },
                })
            )
        );
    }
    // Trivia is ignored when identifying a qualified prefix, but its original
    // source extent remains the name span.
    let source = "tree . Leaf // argument\nvalue";
    let parsed = parse_pattern(source);
    let SurfacePattern::Constructor {
        name,
        name_span,
        argument,
        span,
    } = parsed
    else {
        panic!("qualified constructor missing");
    };
    assert_eq!(name, "tree.Leaf");
    assert_eq!(name_span, ByteSpan { start: 0, end: 11 });
    assert_eq!(
        span,
        ByteSpan {
            start: 0,
            end: source.len()
        }
    );
    assert_eq!(
        argument.as_deref(),
        Some(&self::name("value", source.rfind("value").unwrap()))
    );
}

#[test]
fn constructor_chains_scan_each_source_token_once() {
    for count in [8, 12, 64, 128, 260] {
        let source = format!("{}value", "Branch ".repeat(count));
        SIGNIFICANT_TOKEN_VISITS.with(|visits| visits.set(0));
        let pattern = parse_pattern(&source);
        // One identifier and one space per prefix, plus the terminal binding.
        SIGNIFICANT_TOKEN_VISITS.with(|visits| assert_eq!(visits.get(), 2 * count + 1));
        let mut current = &pattern;
        for index in 0..count {
            let SurfacePattern::Constructor {
                name,
                name_span,
                argument: Some(argument),
                span,
            } = current
            else {
                panic!("constructor {index} missing at length {count}");
            };
            assert_eq!(name, "Branch");
            assert_eq!(name_span.start, index * 7);
            assert_eq!(name_span.end, index * 7 + 6);
            assert_eq!(span.end, source.len());
            current = argument;
        }
        assert_eq!(current, &name("value", count * 7));
    }
}

#[test]
fn coverage_limit_patterns_remain_valid_syntax() {
    let chain = format!("{}Leaf value", "Branch ".repeat(260));
    let mut elements = vec!["_".to_owned(); 199];
    elements.push(format!("{}Leaf value", "Branch ".repeat(60)));
    for pattern in [chain, format!("[{}]", elements.join(", "))] {
        let source =
            format!("fn inspect tree: Tree -> Int = match tree {{ {pattern} -> 1; _ -> 0 }}\n");
        assert!(
            parse_diagnostics("coverage.ssrg", &source)
                .diagnostics
                .is_empty(),
            "{source}"
        );
    }
}

#[test]
fn pattern_tree_depth_is_bounded_before_clone_and_drop() {
    // A bounded native stack exercises surface construction, recursive
    // diagnostics, clone, bindings, CST and drop at the accepted tree limit.
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let source = format!(
                "let {}value = 0\n",
                "Branch ".repeat(crate::resource::PATTERN_DEPTH_LIMIT - 1)
            );
            assert!(parse_diagnostics("accepted.ssrg", &source)
                .diagnostics
                .is_empty());
            assert!(parse_cst("accepted.ssrg", &source).errors.is_empty());
            let surface = parse_surface_ast("accepted.ssrg", &source);
            let cloned = surface.clone();
            let SurfaceDecl::Let { pattern, .. } = &cloned.declarations[0] else {
                panic!("let pattern missing");
            };
            let bindings = pattern.bindings();
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].name, "value");
            drop(cloned);
            drop(surface);
            for pattern in [
                format!(
                    "{}value",
                    "Branch ".repeat(crate::resource::PATTERN_DEPTH_LIMIT)
                ),
                // Each local chain fits, but the collection root and outer
                // prefixes must count against the same logical depth.
                format!("{}[{}value]", "Branch ".repeat(260), "Branch ".repeat(260)),
            ] {
                let limited = parse_diagnostics("limited.ssrg", &format!("let {pattern} = 0\n"));
                assert_eq!(limited.diagnostics.len(), 1);
                assert_eq!(limited.diagnostics[0].code, "SES-P0002");
                assert_eq!(limited.diagnostics[0].message_key, "parser.resource-limit");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn nested_pattern_containers_keep_the_native_frame_limit() {
    for (open, close) in [("(", ")"), ("[", "]"), ("{ field: ", " }")] {
        let source = format!(
            "let {}value{} = 0\n",
            open.repeat(crate::resource::RECURSION_LIMIT + 8),
            close.repeat(crate::resource::RECURSION_LIMIT + 8)
        );
        let limited = parse_diagnostics("containers.ssrg", &source);
        assert_eq!(limited.diagnostics.len(), 1);
        assert_eq!(limited.diagnostics[0].message_key, "parser.resource-limit");
    }
}
