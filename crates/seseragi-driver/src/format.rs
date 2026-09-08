use seseragi_formatter::{format_cst_with_options, FormatOptions, FormattedSource};
use seseragi_syntax::{
    lex, parse_cst_from_tokens, parse_diagnostics, DiagnosticArtifact, DiagnosticSeverity,
};

/// Formats one source snapshot through the same lossless frontend artifacts
/// used by compiler, LSP, and playground adapters.
pub fn format_module(
    source_name: &str,
    source: &str,
) -> Result<FormattedSource, DiagnosticArtifact> {
    format_module_with_options(source_name, source, FormatOptions::default())
}

pub fn format_module_with_options(
    source_name: &str,
    source: &str,
    options: FormatOptions,
) -> Result<FormattedSource, DiagnosticArtifact> {
    let diagnostics = parse_diagnostics(source_name, source);
    if diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    {
        return Err(diagnostics);
    }

    let tokens = lex(source_name, source);
    let cst = parse_cst_from_tokens(tokens.clone());
    Ok(format_cst_with_options(&tokens, &cst, options))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compile_module, CompileInput};

    #[test]
    fn formats_without_changing_phase_one_compilation() {
        let source = include_str!(
            "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg"
        );
        let formatted = format_module("main.ssrg", source).expect("valid source");
        let original = compile_module(CompileInput::new("main.ssrg", "app/main", source))
            .expect("original compiles");
        let after = compile_module(CompileInput::new("main.ssrg", "app/main", &formatted.text))
            .expect("formatted source compiles");

        let converged = format_module("main.ssrg", &formatted.text).expect("canonical source");
        assert!(!converged.changed);
        assert_eq!(converged.text, formatted.text);
        assert_eq!(after.typed_hir, original.typed_hir);
        assert_eq!(after.core_ir, original.core_ir);
        assert_eq!(after.typescript_ir, original.typescript_ir);
        assert_eq!(after.generated.typescript, original.generated.typescript);
    }

    #[test]
    fn preserves_bodyless_declarations_through_the_shared_driver() {
        let input = include_str!(
            "../../seseragi-formatter/tests/fixtures/declaration-boundaries.input.ssrg"
        );
        let expected = include_str!(
            "../../seseragi-formatter/tests/fixtures/declaration-boundaries.expected.ssrg"
        );

        let formatted = format_module("main.ssrg", input).expect("valid declaration fixture");
        assert_eq!(formatted.text, expected);

        let converged =
            format_module("main.ssrg", &formatted.text).expect("valid formatted fixture");
        assert!(!converged.changed);
        assert_eq!(converged.text, expected);
    }

    #[test]
    fn returns_shared_parse_diagnostics_instead_of_formatting_recovery_nodes() {
        let source = "pub let answer: Int =\n";
        let diagnostics = format_module("broken.ssrg", source).expect_err("invalid source");

        assert_eq!(diagnostics.source, "broken.ssrg");
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
    }

    #[test]
    fn passes_explicit_line_width_to_the_formatter_core() {
        let source =
            "let labels = [\"formatter\", \"playground\", \"curriculum\", \"diagnostics\"]\n";
        let default = format_module("main.ssrg", source).expect("valid source");
        let narrow = format_module_with_options("main.ssrg", source, FormatOptions::new(48))
            .expect("valid source");

        assert_ne!(narrow.text, default.text);
        assert!(narrow.text.contains("[\n"), "{}", narrow.text);
        let converged =
            format_module_with_options("main.ssrg", &narrow.text, FormatOptions::new(48))
                .expect("formatted source remains valid");
        assert!(!converged.changed, "{}", converged.text);
    }
}

/// Formats complete nodes intersecting a byte range and verifies that layout
/// leaves the shared surface parser's structure unchanged.
pub fn format_module_range(
    source_name: &str,
    source: &str,
    range: std::ops::Range<usize>,
    options: FormatOptions,
) -> Vec<seseragi_formatter::FormatEdit> {
    let tokens = lex(source_name, source);
    let cst = parse_cst_from_tokens(tokens.clone());
    let edits = seseragi_formatter::format_cst_range(&tokens, &cst, range, options);
    let mut result = source.to_owned();
    for edit in edits.iter().rev() {
        result.replace_range(edit.range.clone(), &edit.text);
    }
    fn shape(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(fields) => {
                if fields.len() == 2
                    && fields
                        .get("start")
                        .is_some_and(serde_json::Value::is_number)
                    && fields.get("end").is_some_and(serde_json::Value::is_number)
                {
                    fields.insert("start".into(), 0.into());
                    fields.insert("end".into(), 0.into());
                } else {
                    for child in fields.values_mut() {
                        shape(child);
                    }
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    shape(item);
                }
            }
            _ => {}
        }
    }
    let mut before = serde_json::to_value(seseragi_syntax::parse_surface_ast(source_name, source))
        .expect("serializable surface AST");
    let mut after = serde_json::to_value(seseragi_syntax::parse_surface_ast(source_name, &result))
        .expect("serializable surface AST");
    shape(&mut before);
    shape(&mut after);
    if before == after {
        edits
    } else {
        vec![]
    }
}

#[cfg(test)]
mod range_tests {
    use super::*;
    fn apply(source: &str, selected: &str, expected: &str) {
        let start = source.find(selected).unwrap();
        let edits = format_module_range(
            "range.ssrg",
            source,
            start..start + selected.len(),
            FormatOptions::default(),
        );
        let mut result = source.to_owned();
        for edit in edits.iter().rev() {
            result.replace_range(edit.range.clone(), &edit.text);
        }
        assert_eq!(result, expected, "edits: {edits:?}");
        let second = format_module_range(
            "range.ssrg",
            &result,
            start..(start + selected.len()).min(result.len()),
            FormatOptions::default(),
        );
        assert!(second.is_empty(), "range must converge: {second:?}");
    }
    #[test]
    fn range_formats_only_complete_selected_nodes() {
        apply("let a=1+2\nlet b=3+4\n", "1+2", "let a=1 + 2\nlet b=3+4\n");
        apply(
            "let a={ left:1+2, right:3+4 }\n",
            "left:1+2",
            "let a={ left: 1 + 2, right:3+4 }\n",
        );
        apply("let a=[1+2,3+4]\n", "1+2", "let a=[1 + 2,3+4]\n");
        apply("let a=`[1+2,3+4]\n", "1+2", "let a=`[1 + 2,3+4]\n");
        apply(
            "let a=match value {\n    Just x->x+1\n _->0\n}\n",
            "Just x->x+1",
            "let a=match value {\n  Just x -> x + 1\n _->0\n}\n",
        );
        apply(
            "effect fn main = do {\n    let a=1+2\n println a\n}\n",
            "let a=1+2",
            "effect fn main = do {\n  let a = 1 + 2\n println a\n}\n",
        );
    }
    #[test]
    fn range_preserves_custom_fixity_and_document_layout() {
        let source = "operator infixr 4 <+> left: Int -> right: Int -> Int = left + right\npub let result=1<+>2<+>3\n";
        let start = source.find("1<+>").unwrap();
        let edits = format_module_range(
            "main.ssrg",
            source,
            start..source.len(),
            FormatOptions::default(),
        );
        assert!(!edits.is_empty());
        let mut result = source.to_owned();
        for edit in edits.iter().rev() {
            result.replace_range(edit.range.clone(), &edit.text);
        }
        let before =
            crate::compile_module(crate::CompileInput::new("main.ssrg", "app/main", source))
                .unwrap();
        let after =
            crate::compile_module(crate::CompileInput::new("main.ssrg", "app/main", &result))
                .unwrap();
        assert_eq!(before.generated.typescript, after.generated.typescript);
        assert_eq!(
            format_module("main.ssrg", source).unwrap().text,
            format_module("main.ssrg", &result).unwrap().text
        );
    }
    #[test]
    fn range_formats_a_multiline_signature_without_its_body() {
        apply(
            "fn add\n  first:Int ->\n second:Int ->Int =first+second\n",
            "fn add\n  first:Int ->\n second:Int ->Int =",
            "fn add first: Int -> second: Int -> Int =first+second\n",
        );
    }
    #[test]
    fn range_preserves_function_neighbors_comments_and_multiple_declarations() {
        apply(
            "fn add value:Int ->Int =value+1\nlet b=3+4\n",
            "value+1",
            "fn add value:Int ->Int =value + 1\nlet b=3+4\n",
        );
        apply(
            "let a=1+2 // keep comment\nlet b=3+4\n",
            "1+2",
            "let a=1 + 2 // keep comment\nlet b=3+4\n",
        );
        apply(
            "let a=1+2\nlet b=3+4\nlet c=5+6\n",
            "let a=1+2\nlet b=3+4",
            "let a = 1 + 2\nlet b = 3 + 4\nlet c=5+6\n",
        );
        apply(
            "let a=1+2\nlet b=3+4\nlet c=5+6\n",
            "1+2\nlet b=3+4",
            "let a=1 + 2\nlet b = 3 + 4\nlet c=5+6\n",
        );
        let source = "let a=\"😀\"++\"x\"\n";
        let inside_scalar = source.find('😀').unwrap() + 1;
        assert!(format_module_range(
            "range.ssrg",
            source,
            inside_scalar..inside_scalar,
            FormatOptions::default()
        )
        .is_empty());
    }
    #[test]
    fn range_handles_partial_unicode_and_recovery() {
        apply(
            "let a=123+456\nlet b=7+8\n",
            "23+45",
            "let a=123 + 456\nlet b=7+8\n",
        );
        apply(
            "let a=\"😀\"++\"瀬\"\n",
            "\"😀\"++\"瀬\"",
            "let a=\"😀\" ++ \"瀬\"\n",
        );
        apply("let broken =\n", "broken", "let broken =\n");
        apply(
            "let a=1+2\nlet broken =\n",
            "1+2",
            "let a=1 + 2\nlet broken =\n",
        );
    }
}
