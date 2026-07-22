use crate::CompileInput;
use seseragi_project::{link_module, standard_module_target};
use seseragi_semantics::{
    analysis_document, analyze_linked_module, diagnostics_only_analysis, AnalysisDocument,
    AnalyzedModule,
};
use seseragi_syntax::{
    parse_diagnostics, parse_unlinked_module_interface, DiagnosticArtifact, DiagnosticSeverity,
};
use std::collections::BTreeMap;

/// Runs the shared compiler frontend without lowering, code generation, or
/// Effect execution and returns the reusable position-query snapshot.
pub fn analyze_module(input: CompileInput<'_>) -> AnalysisDocument {
    match analyze_module_frontend(input) {
        Ok(analyzed) => {
            analysis_document(analyzed.diagnostics, analyzed.resolved, &analyzed.typed_hir)
        }
        Err(diagnostics) => {
            diagnostics_only_analysis(input.source_name(), input.module_id(), diagnostics)
        }
    }
}

pub(crate) fn analyze_module_frontend(
    input: CompileInput<'_>,
) -> Result<AnalyzedModule, DiagnosticArtifact> {
    let mut diagnostics = parse_diagnostics(input.source_name(), input.source());
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let unlinked =
        parse_unlinked_module_interface(input.source_name(), input.module_id(), input.source());
    let targets = unlinked
        .imports
        .iter()
        .filter_map(|import| {
            standard_module_target(&import.specifier)
                .map(|target| (import.specifier.clone(), target))
        })
        .collect::<BTreeMap<_, _>>();
    let linked = match link_module(unlinked, &targets) {
        Ok(linked) => linked,
        Err(errors) => {
            crate::dependencies::append_link_diagnostics(errors, &mut diagnostics);
            return Err(diagnostics);
        }
    };
    analyze_linked_module(diagnostics, linked, input.source())
}

fn has_errors(diagnostics: &DiagnosticArtifact) -> bool {
    diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_does_not_require_lowering_or_an_entry_point() {
        let source =
            "// 雫\nfn add left: Int -> right: Int -> Int = left + right\nlet addOne = add 1\n";
        let analysis = analyze_module(CompileInput::new("main.ssrg", "analysis/shared", source));

        assert!(analysis.diagnostics.diagnostics.is_empty());
        let add_reference = source.rfind("add 1").unwrap();
        let add_symbol = analysis.symbol_at(add_reference).unwrap();
        assert_eq!(add_symbol.name, "add");
        assert_eq!(
            analysis.type_at(add_reference).unwrap().type_name,
            "Int -> Int -> Int"
        );
        assert_eq!(
            analysis.definition_of(add_reference),
            Some(add_symbol.definition)
        );
        let callable = analysis.callable_at(add_reference).unwrap();
        assert_eq!(callable.parameters.len(), 2);
        assert_eq!(callable.result, "Int");

        let applied_argument = source.rfind('1').unwrap();
        let partial = analysis.callable_at(applied_argument).unwrap();
        assert_eq!(partial.remaining_parameters.len(), 1);
        assert_eq!(
            partial.remaining_parameters[0].name.as_deref(),
            Some("right")
        );
        assert_eq!(partial.remaining_parameters[0].type_name, "Int");

        let visible = analysis
            .visible_symbols(applied_argument)
            .into_iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>();
        assert!(visible.contains(&"add"));
        assert!(visible.contains(&"addOne"));

        for expected in ["join", "sum", "forEach", "map"] {
            assert!(analysis
                .standard_library_catalog()
                .iter()
                .any(|item| item.name == expected));
        }
    }

    #[test]
    fn invalid_source_still_returns_shared_diagnostics_and_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/invalid",
            "pub let broken: Int =\n",
        ));

        assert!(!analysis.diagnostics.diagnostics.is_empty());
        assert!(analysis.symbols.is_empty());
        assert!(analysis
            .standard_library_catalog()
            .iter()
            .any(|item| item.name == "join"));
    }

    #[test]
    fn imported_member_definition_points_to_its_source_import() {
        let source = concat!(
            "import * as html from \"std/web/html\"\n",
            "let page = html.div { children: \"Hi\" }\n",
        );
        let analysis = analyze_module(CompileInput::new("main.ssrg", "analysis/import", source));
        let member = source.rfind("div").unwrap();
        let definition = analysis.definition_of(member).unwrap();

        assert_eq!(analysis.symbol_at(member).unwrap().name, "div");
        assert!(definition.start < member);
        assert!(definition.end > definition.start);
    }
}
