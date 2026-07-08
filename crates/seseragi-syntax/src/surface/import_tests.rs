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
                    alias: None,
                },
                SurfaceImportItem {
                    namespace: "operator".to_owned(),
                    name: "<+>".to_owned(),
                    alias: None,
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
                alias: Some("parseJson".to_owned()),
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
                alias: Some("text".to_owned()),
            }],
            span: ByteSpan { start: 0, end: 32 },
        }]
    );
}
