use super::*;

#[test]
fn parses_operator_import_items() {
    let module = parse_surface_ast(
        "main.ssrg",
        "import { identity, operator <+> } from \"./dep\"\n",
    );

    assert_eq!(
        module.imports,
        vec![SurfaceImport {
            visibility: Visibility::Private,
            specifier: "./dep".to_owned(),
            items: vec![
                SurfaceImportItem {
                    namespace: "value".to_owned(),
                    name: "identity".to_owned(),
                    name_span: ByteSpan { start: 9, end: 17 },
                    alias: None,
                    alias_span: None,
                },
                SurfaceImportItem {
                    namespace: "operator".to_owned(),
                    name: "<+>".to_owned(),
                    name_span: ByteSpan { start: 28, end: 31 },
                    alias: None,
                    alias_span: None,
                },
            ],
            span: ByteSpan { start: 0, end: 46 },
        }]
    );
}

#[test]
fn parses_aliased_import_items() {
    let module = parse_surface_ast("main.ssrg", "import { parse as parseJson } from \"json\"\n");

    assert_eq!(
        module.imports,
        vec![SurfaceImport {
            visibility: Visibility::Private,
            specifier: "json".to_owned(),
            items: vec![SurfaceImportItem {
                namespace: "value".to_owned(),
                name: "parse".to_owned(),
                name_span: ByteSpan { start: 9, end: 14 },
                alias: Some("parseJson".to_owned()),
                alias_span: Some(ByteSpan { start: 18, end: 27 }),
            }],
            span: ByteSpan { start: 0, end: 41 },
        }]
    );
}

#[test]
fn parses_namespace_import_items() {
    let module = parse_surface_ast("main.ssrg", "import * as text from \"std/text\"\n");

    assert_eq!(
        module.imports,
        vec![SurfaceImport {
            visibility: Visibility::Private,
            specifier: "std/text".to_owned(),
            items: vec![SurfaceImportItem {
                namespace: "namespace".to_owned(),
                name: "*".to_owned(),
                name_span: ByteSpan { start: 7, end: 8 },
                alias: Some("text".to_owned()),
                alias_span: Some(ByteSpan { start: 12, end: 16 }),
            }],
            span: ByteSpan { start: 0, end: 32 },
        }]
    );
}

#[test]
fn preserves_public_import_visibility() {
    let module = parse_surface_ast(
        "facade.ssrg",
        "pub import { userId } from \"./model/user\"\n",
    );

    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].visibility, Visibility::Public);
    assert_eq!(module.imports[0].items[0].name, "userId");
    assert_eq!(module.imports[0].span, ByteSpan { start: 0, end: 41 });
}

#[test]
fn rejects_keyword_as_namespace_import_alias() {
    let source = "import * as effect from \"std/effect\"\n\nlet x = effect.succeed ()\n";
    let artifact = crate::parse_diagnostics("main.ssrg", source);
    let alias_error = artifact.diagnostics.iter().find(|diagnostic| {
        diagnostic.message_key == "parser.invalid-declaration-name"
            && diagnostic.primary.start == 12
            && diagnostic.primary.end == 18
    });
    assert!(
        alias_error.is_some(),
        "expected invalid-declaration-name at `effect` alias, got: {:#?}",
        artifact.diagnostics
    );
}

#[test]
fn rejects_reserved_word_or_uppercase_as_namespace_import_alias() {
    for (source, expected_start, expected_end) in [
        ("import * as type from \"std/text\"\n", 12, 16),
        ("import * as Effect from \"std/effect\"\n", 12, 18),
    ] {
        let artifact = crate::parse_diagnostics("main.ssrg", source);
        assert!(
            artifact.diagnostics.iter().any(|diagnostic| {
                diagnostic.message_key == "parser.invalid-declaration-name"
                    && diagnostic.primary.start == expected_start
                    && diagnostic.primary.end == expected_end
            }),
            "expected invalid-declaration-name for `{source}`, got: {:#?}",
            artifact.diagnostics
        );
    }
}

#[test]
fn rejects_keyword_as_named_import_alias() {
    for (source, expected_start, expected_end) in [
        ("import { succeed as effect } from \"std/effect\"\n", 20, 26),
        ("import { succeed as from } from \"std/effect\"\n", 20, 24),
        ("import * as from from \"std/effect\"\n", 12, 16),
        (
            "import { succeed as bad-name } from \"std/effect\"\n",
            20,
            28,
        ),
        ("import * as bad-name from \"std/effect\"\n", 12, 20),
    ] {
        let artifact = crate::parse_diagnostics("main.ssrg", source);
        assert!(
            artifact.diagnostics.iter().any(|diagnostic| {
                diagnostic.message_key == "parser.invalid-declaration-name"
                    && diagnostic.primary.start == expected_start
                    && diagnostic.primary.end == expected_end
            }),
            "expected invalid-declaration-name for `{source}`, got: {:#?}",
            artifact.diagnostics
        );
    }
}

#[test]
fn does_not_split_malformed_alias_into_extra_import_items() {
    let module = parse_surface_ast(
        "main.ssrg",
        "import { succeed as bad-name } from \"std/effect\"\n",
    );
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.imports[0].items.len(), 1);
    assert_eq!(module.imports[0].items[0].name, "succeed");
    assert_eq!(module.imports[0].items[0].alias, None);
}

#[test]
fn accepts_valid_namespace_and_named_import_aliases() {
    let source =
        "import * as effects from \"std/effect\"\nimport { succeed as ok } from \"std/effect\"\n";
    let artifact = crate::parse_diagnostics("main.ssrg", source);
    assert!(
        artifact.diagnostics.is_empty(),
        "expected no diagnostics for valid import aliases, got: {:#?}",
        artifact.diagnostics
    );
}
