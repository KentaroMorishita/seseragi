use seseragi_dts::{
    convert_entry, interpret_declaration, project_graph, ConversionInput, HostMetadata,
};
use seseragi_interop::SemanticGraph;

const SETTINGS: &str = "schema=1\n[entries.api]\ndeclaration='api.d.ts'\nspecifier='./api.js'\noutput='api'\nevaluation='pure'\n";

fn convert(source: &str, settings: &str, previous: Option<&str>) -> seseragi_dts::ConversionResult {
    convert_entry(&ConversionInput {
        settings,
        entry: "api",
        declaration: source,
        host: HostMetadata {
            manifest: "{}",
            resolved_package: None,
        },
        previous_metadata: previous,
    })
    .unwrap()
}

#[test]
fn direct_jsdoc_retains_text_and_exact_origins_through_the_common_graph_and_binding() {
    let source = concat!(
        "// 元のAPI 🙂\n",
        "/** Description.\n * @deprecated Use \"current\".\n * Keep the value.\n * @since 9.9.9\n */\n",
        "export declare function old(value: string): string;\n",
        "export declare function current(value: string): string;\n",
        "/** @deprecated Use Modern. */\nexport interface Old { readonly value: string; }\n",
        "/** @deprecated Use modern namespace. */\nexport declare namespace oldNamespace {\n",
        "  export function sibling(): string;\n",
        "  /** @deprecated Use fresh. */\n  export function nested(): string;\n}\n",
    );
    let result = convert(source, SETTINGS, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let graph = result.semantic.unwrap();
    let notice = &graph.symbols["old"].deprecations[0];
    assert_eq!(notice.message, "Use \"current\".\nKeep the value.");
    assert_eq!(notice.origin.source, "api.d.ts");
    assert!(source[notice.origin.start..notice.origin.end].starts_with("/** Description."));
    assert!(source[notice.declaration.start..notice.declaration.end].starts_with("function old"));
    assert!(graph.symbols["current"].deprecations.is_empty());
    assert!(graph.symbols["oldNamespace.sibling"]
        .deprecations
        .is_empty());
    assert_eq!(
        graph.symbols["oldNamespace.nested"].deprecations[0].message,
        "Use fresh."
    );
    let decoded = SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap();
    assert_eq!(decoded, graph);
    let generated = result.generated.unwrap();
    assert!(
        generated
            .source
            .contains("deprecated \"Use \\\"current\\\".\\nKeep the value.\" pure fn old"),
        "{}",
        generated.source
    );
    assert!(generated
        .source
        .contains("deprecated \"Use Modern.\" opaque type Old"));
    assert!(generated
        .source
        .contains("deprecated \"Use modern namespace.\" namespace oldNamespace"));
    assert!(!generated.source.contains("since"));
    assert_eq!(
        project_graph(&decoded, SETTINGS, "api")
            .unwrap()
            .source
            .as_deref(),
        Some(generated.source.as_str())
    );
    let metadata: serde_json::Value = serde_json::from_str(&generated.metadata).unwrap();
    let symbols = metadata["symbols"].as_array().unwrap();
    let old = symbols
        .iter()
        .find(|symbol| symbol["publicPath"] == "old")
        .unwrap();
    assert_eq!(
        old["deprecations"],
        serde_json::to_value(&graph.symbols["old"].deprecations).unwrap()
    );
    assert!(old["deprecations"][0].get("since").is_none());
    assert!(symbols
        .iter()
        .find(|symbol| symbol["publicPath"] == "current")
        .unwrap()
        .get("deprecations")
        .is_none());
}

#[test]
fn overload_selection_never_inherits_another_signatures_notice() {
    let settings = format!("{SETTINGS}\n[entries.api.symbols.pick.overloads.oldPick]\nsignature=0\n[entries.api.symbols.pick.overloads.currentPick]\nsignature=1\n");
    let source = "/** @deprecated Use currentPick. */\nexport declare function pick(value: string): string;\nexport declare function pick(value: boolean): boolean;\n";
    let result = convert(source, &settings, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let graph = result.semantic.unwrap();
    assert_eq!(graph.symbols["pick"].deprecations.len(), 1);
    let binding = result.generated.unwrap();
    assert!(
        binding
            .source
            .contains("deprecated \"Use currentPick.\" pure fn oldPick"),
        "{}",
        binding.source
    );
    assert!(binding.source.contains("pure fn currentPick"));
    let metadata: serde_json::Value = serde_json::from_str(&binding.metadata).unwrap();
    let old = metadata["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .find(|symbol| symbol["localName"] == "oldPick")
        .unwrap();
    let current = metadata["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .find(|symbol| symbol["localName"] == "currentPick")
        .unwrap();
    assert_eq!(old["deprecations"].as_array().unwrap().len(), 1);
    assert!(current.get("deprecations").is_none());
    assert_eq!(old["hostName"], "pick");
    assert_eq!(current["hostName"], "pick");
}

#[test]
fn aliases_keep_original_declaration_notice_and_ignore_export_list_comment() {
    let source = concat!(
        "/** @deprecated Use current. */\ndeclare function old(value: string): string;\n",
        "declare function ordinary(value: string): string;\n",
        "/** @deprecated This is not a declaration tag. */\nexport { old as renamed, ordinary };\n",
    );
    let result = convert(source, SETTINGS, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let graph = result.semantic.unwrap();
    assert_eq!(
        graph.symbols["renamed"].deprecations[0].message,
        "Use current."
    );
    assert!(graph.symbols["ordinary"].deprecations.is_empty());
    let origin = &graph.symbols["renamed"].deprecations[0].declaration;
    assert!(source[origin.start..origin.end].starts_with("function old"));
    assert!(result
        .generated
        .unwrap()
        .source
        .contains("deprecated \"Use current.\" pure fn renamed"));
}

#[test]
fn empty_or_duplicate_tags_are_explicit_errors_and_lookalikes_do_not_attach() {
    for comment in [
        "/** @deprecated */",
        "/** @deprecated Use current.\n * @deprecated Another. */",
    ] {
        let result = convert(
            &format!("{comment}\nexport declare function old(): string;"),
            SETTINGS,
            None,
        );
        assert!(result.generated.is_none());
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("@deprecated")));
    }
    let source = concat!(
        "/* @deprecated Ordinary block. */\nexport declare function first(): string;\n",
        "/** @deprecatedLonger Not a tag. */\nexport declare function second(): string;\n",
        "/** @deprecated Distant. */\n// intervening ordinary comment\nexport declare function third(): string;\n",
        "export type Text = '@deprecated literal';\n",
    );
    let result = convert(source, SETTINGS, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert!(result
        .semantic
        .unwrap()
        .symbols
        .values()
        .all(|symbol| symbol.deprecations.is_empty()));
    let binding = result.generated.unwrap();
    assert!(!binding.source.contains("deprecated"));
    assert!(!binding.metadata.contains("deprecations"));
}

#[test]
fn changed_migration_metadata_is_reported_without_changing_callable_signature_or_host() {
    let source =
        "/** @deprecated Use current. */\nexport declare function old(value: string): string;\n";
    let before = convert(source, SETTINGS, None).generated.unwrap();
    let after = convert(
        &source.replace("Use current.", "Use next."),
        SETTINGS,
        Some(&before.metadata),
    )
    .generated
    .unwrap();
    let before_metadata: serde_json::Value = serde_json::from_str(&before.metadata).unwrap();
    let after_metadata: serde_json::Value = serde_json::from_str(&after.metadata).unwrap();
    for key in ["identity", "signature", "hostName", "localName"] {
        assert_eq!(
            before_metadata["symbols"][0][key],
            after_metadata["symbols"][0][key]
        );
    }
    assert_eq!(before_metadata["hostModule"], after_metadata["hostModule"]);
    let report: serde_json::Value = serde_json::from_str(&after.report).unwrap();
    assert_eq!(report["changed"].as_array().unwrap().len(), 1);
    assert!(report["added"].as_array().unwrap().is_empty());
    assert!(report["removed"].as_array().unwrap().is_empty());
}

#[test]
fn invalid_metadata_provenance_cannot_be_persisted_or_projected() {
    let graph = interpret_declaration(
        "/** @deprecated Use current. */\nexport declare function old(): string;",
        "api.d.ts",
        SETTINGS,
    )
    .unwrap();
    for corruption in 0..4 {
        let mut invalid = graph.clone();
        let notice = &mut invalid.symbols.get_mut("old").unwrap().deprecations[0];
        match corruption {
            0 => notice.message.clear(),
            1 => notice.origin.end = usize::MAX,
            2 => notice.declaration.start += 1,
            3 => notice.origin.source = "another.d.ts".to_owned(),
            _ => unreachable!(),
        }
        assert!(invalid.to_json().is_err());
        assert!(project_graph(&invalid, SETTINGS, "api").is_err());
    }
}

#[test]
fn control_characters_use_seseragi_scalar_escapes_instead_of_json_escapes() {
    let source = "/** @deprecated Keep x\u{8}y\u{c}z. */\nexport declare function old(): string;";
    let result = convert(source, SETTINGS, None);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let binding = result.generated.unwrap();
    assert!(
        binding.source.contains("x\\u{8}y\\u{c}z"),
        "{}",
        binding.source
    );
    assert!(!binding.source.contains("\\b"));
    assert!(!binding.source.contains("\\f"));
}
