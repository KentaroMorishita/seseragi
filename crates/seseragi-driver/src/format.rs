use seseragi_formatter::{format_cst, FormattedSource};
use seseragi_syntax::{
    lex, parse_cst_from_tokens, parse_diagnostics, DiagnosticArtifact, DiagnosticSeverity,
};

/// Formats one source snapshot through the same lossless frontend artifacts
/// used by compiler, LSP, and playground adapters.
pub fn format_module(
    source_name: &str,
    source: &str,
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
    Ok(format_cst(&tokens, &cst))
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

        assert_eq!(formatted.text, source);
        assert_eq!(after.typed_hir, original.typed_hir);
        assert_eq!(after.core_ir, original.core_ir);
        assert_eq!(after.typescript_ir, original.typescript_ir);
        assert_eq!(after.generated.typescript, original.generated.typescript);
    }

    #[test]
    fn returns_shared_parse_diagnostics_instead_of_formatting_recovery_nodes() {
        let source = "pub let answer: Int =\n";
        let diagnostics = format_module("broken.ssrg", source).expect_err("invalid source");

        assert_eq!(diagnostics.source, "broken.ssrg");
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
    }
}
