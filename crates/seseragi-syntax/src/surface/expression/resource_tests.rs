use super::*;
use crate::{parse_cst, parse_diagnostics, parse_import_free_module_interface, parse_surface_ast};

#[test]
fn parses_each_grouped_operand_once_at_eight_to_twelve_levels() {
    for depth in 8..=12 {
        for operator in ["+", "<+>"] {
            let mut expression = "1".to_owned();
            for _ in 0..depth {
                expression = format!("({expression} {operator} 2 * 3)");
            }
            PREFIX_PARSES.with(|count| count.set(0));
            let surface =
                parse_surface_ast("linear.ssrg", &format!("fn probe -> Int = {expression}\n"));
            let Some(crate::SurfaceDecl::Fn {
                body: Some(body), ..
            }) = surface.declarations.first()
            else {
                panic!("function body missing");
            };
            assert!(matches!(body, SurfaceExpr::Grouped { .. }));
            PREFIX_PARSES.with(|count| {
                assert_eq!(
                    count.get(),
                    3 * depth + 1,
                    "depth={depth}, operator={operator}"
                )
            });
        }
    }
}

#[test]
fn malformed_grouped_operands_do_not_trigger_speculative_retries() {
    let depth = 12;
    let source = format!(
        "fn probe -> Int = {}1 + {}\n",
        "(".repeat(depth),
        ")".repeat(depth)
    );
    PREFIX_PARSES.with(|count| count.set(0));
    let surface = parse_surface_ast("invalid.ssrg", &source);
    assert!(matches!(
        surface.declarations.first(),
        Some(crate::SurfaceDecl::Fn {
            body: Some(SurfaceExpr::Error { .. }),
            ..
        })
    ));
    PREFIX_PARSES.with(|count| assert!(count.get() <= depth + 2));
    assert!(parse_diagnostics("invalid.ssrg", &source)
        .diagnostics
        .iter()
        .any(|d| d.message_key == "parser.expected-expression"));
}

fn assert_resource_diagnostic(source: &str) {
    let artifact = parse_diagnostics("bounds-雫.ssrg", source);
    assert_eq!(artifact.source, "bounds-雫.ssrg");
    assert_eq!(artifact.diagnostics.len(), 1, "{source}");
    let diagnostic = &artifact.diagnostics[0];
    assert_eq!(diagnostic.code, "SES-P0002", "{source}");
    assert_eq!(diagnostic.message_key, "parser.resource-limit");
    assert!(diagnostic.primary.start <= diagnostic.primary.end);
    assert!(diagnostic.primary.end <= source.len());
    assert!(source.is_char_boundary(diagnostic.primary.start));
    assert!(source.is_char_boundary(diagnostic.primary.end));
    assert!(diagnostic.message().contains("limit"));
    assert!(parse_cst("bounds.ssrg", source)
        .errors
        .iter()
        .any(|error| error.code == "SES-P0002"));
    let interface = parse_import_free_module_interface("bounds.ssrg", "bounds", source).unwrap();
    assert!(interface.exports.is_empty());
}

#[test]
fn bounds_recursion_across_new_range_parsers_and_template_lexing() {
    let count = crate::resource::RECURSION_LIMIT + 8;
    for (open, close) in [
        ("(", ")"),
        ("[", "]"),
        ("{ ", " }"),
        ("do { ", " }"),
        ("match 0 { _ -> ", " }"),
        ("[0 | _ <- ", "]"),
        ("\\x -> ", ""),
        ("`${", "}`"),
    ] {
        let source = format!(
            "// 雫🙂\nfn probe -> Int = {}0{}\n",
            open.repeat(count),
            close.repeat(count)
        );
        assert_resource_diagnostic(&source);
    }
}

#[test]
fn bounds_type_and_pattern_recursion_before_building_interface_artifacts() {
    let count = crate::resource::RECURSION_LIMIT + 8;
    assert_resource_diagnostic(&format!(
        "fn probe -> {}Int{} = 0\n",
        "Array<".repeat(count),
        ">".repeat(count)
    ));
    assert_resource_diagnostic(&format!("let {}value = 0\n", "Some ".repeat(count)));
    assert_resource_diagnostic(&format!(
        "foreign \"typescript\" from \"./api.js\" {{\n{}pure fn probe -> Int\n{} }}\n",
        "namespace Nested {\n".repeat(count),
        "}\n".repeat(count)
    ));
}

#[test]
fn bounds_flat_ast_construction_including_error_paths() {
    let count = crate::resource::EXPRESSION_DEPTH_LIMIT + 8;
    for expression in [
        vec!["1"; count].join(" + "),
        vec!["1"; count].join(" ** "),
        format!("{}0", "f ".repeat(count)),
        format!("value{}", ".field".repeat(count)),
        format!("\\{}-> 0", "arg ".repeat(count)),
    ] {
        assert_resource_diagnostic(&format!("fn probe -> Int = {expression}\n"));
        assert_resource_diagnostic(&format!("fn probe -> Int = {expression} +\n"));
    }
}

#[test]
fn accepts_wide_shallow_syntax_and_restores_a_failed_parse_budget() {
    let source = format!("fn probe -> Array<Int> = [{}]\n", vec!["1"; 512].join(", "));
    assert!(parse_diagnostics("wide.ssrg", &source)
        .diagnostics
        .is_empty());
    {
        let _scope = crate::resource::ParseScope::new();
        crate::resource::spend_all_but_one();
        let limited = parse_diagnostics("work.ssrg", "fn probe -> Int = 1\n");
        assert_eq!(limited.diagnostics[0].message_key, "parser.resource-limit");
        assert_eq!(limited.diagnostics[0].code, "SES-P0002");
    }
    assert!(parse_diagnostics("after.ssrg", "fn probe -> Int = 1\n")
        .diagnostics
        .is_empty());
}

#[test]
fn lexes_deep_templates_without_using_recursive_scanner_frames() {
    let count = 256;
    let source = format!("{}0{}", "`${".repeat(count), "}`".repeat(count));
    let stream = crate::lex("template.ssrg", &source);
    assert_eq!(stream.tokens[0].kind, TokenKind::LiteralTemplate);
    assert_eq!(stream.tokens[0].end, source.len());
    assert_resource_diagnostic(&format!("fn probe -> String = {source}\n"));
}

#[test]
fn accepts_the_group_recursion_boundary_and_rejects_the_next_level() {
    let accepted = crate::resource::RECURSION_LIMIT / 2 - 1;
    for depth in [24, 30, accepted] {
        let source = format!(
            "fn probe -> Int = {}1{}\n",
            "(".repeat(depth),
            ")".repeat(depth)
        );
        assert!(
            parse_diagnostics("accepted.ssrg", &source)
                .diagnostics
                .is_empty(),
            "depth={depth}"
        );
    }
    let depth = accepted + 1;
    assert_resource_diagnostic(&format!(
        "fn probe -> Int = {}1{}\n",
        "(".repeat(depth),
        ")".repeat(depth)
    ));
}

#[test]
fn shares_resource_limits_with_unterminated_nested_template_diagnostics() {
    let source = format!(
        "fn probe -> String = {}0\n",
        "`${".repeat(crate::resource::RECURSION_LIMIT + 8)
    );
    let diagnostics = parse_diagnostics("unterminated.ssrg", &source);
    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-P0002");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "parser.resource-limit"
    );
    assert!(diagnostics.diagnostics[0].primary.end <= source.len());
}

#[test]
fn parse_budgets_are_thread_local_and_restore_during_unwind() {
    let other_thread =
        std::thread::spawn(|| parse_diagnostics("other.ssrg", "fn probe -> Int = 1\n").diagnostics);
    let caught = std::panic::catch_unwind(|| {
        let _scope = crate::resource::ParseScope::new();
        crate::resource::spend_all_but_one();
        assert_eq!(
            parse_diagnostics("limited.ssrg", "fn probe -> Int = 1\n").diagnostics[0].code,
            "SES-P0002"
        );
        panic!("exercise parse scope unwind");
    });
    assert!(caught.is_err());
    assert!(other_thread.join().unwrap().is_empty());
    assert!(parse_diagnostics("restored.ssrg", "fn probe -> Int = 1\n")
        .diagnostics
        .is_empty());
}

#[test]
fn resource_cst_token_ranges_remain_ordered_at_zero_width_eof() {
    for prefixes in [62, 63, 64] {
        for suffix in ["", "\n", " \n"] {
            let source = format!("// 雫\nlet result = {}{suffix}", "! ".repeat(prefixes));
            let tokens = crate::lex("eof.ssrg", &source);
            let eof = tokens
                .tokens
                .iter()
                .position(|token| token.kind == TokenKind::Eof)
                .unwrap();
            let cst = parse_cst("eof.ssrg", &source);
            for error in &cst.errors {
                assert!(
                    error.start_token <= error.end_token,
                    "prefixes={prefixes}, error={error:?}"
                );
                assert!(
                    error.end_token <= eof,
                    "prefixes={prefixes}, error={error:?}"
                );
            }
            let resource_errors = cst
                .errors
                .iter()
                .filter(|error| error.code == "SES-P0002")
                .collect::<Vec<_>>();
            assert_eq!(resource_errors.len(), usize::from(prefixes >= 63));
            if prefixes == 63 {
                for error in resource_errors {
                    assert_eq!(error.start_token, eof);
                    assert_eq!(error.end_token, eof);
                }
            }
        }
    }
}

#[test]
fn bounds_projected_custom_infix_depth_before_semantic_reassociation() {
    let chain = |steps: usize| format!("1{}", " <+> 1".repeat(steps));
    let accepted = format!("fn probe -> Int = {}\n", chain(63));
    assert!(parse_diagnostics("flat.ssrg", &accepted)
        .diagnostics
        .is_empty());
    assert_resource_diagnostic(&format!("fn probe -> Int = {}\n", chain(64)));
    // Each individual flat sequence fits. Combining them must account for
    // the applications produced by resolution, rather than only two shallow
    // InfixChain nodes in the syntax artifact.
    assert_resource_diagnostic(&format!(
        "fn probe -> Int = ({}){}\n",
        chain(40),
        " <+> 1".repeat(40)
    ));
}
