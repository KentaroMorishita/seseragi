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
