use seseragi_driver::{compile_project, CompiledProject, ProjectCompileError, ProjectModuleInput};
use seseragi_project::ModuleGraph;

const DOMAIN: &str =
    include_str!("../../../examples/spec/fixtures/projects/opaque-struct/src/domain.ssrg");

fn project(main: &str) -> Result<CompiledProject, ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/opaque::domain".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/opaque::facade".to_owned(),
            [("./domain".to_owned(), "fixture/opaque::domain".to_owned())],
        )
        .unwrap();
    graph
        .add_module(
            "fixture/opaque::main".to_owned(),
            [("./facade".to_owned(), "fixture/opaque::facade".to_owned())],
        )
        .unwrap();
    compile_project(graph, [
        ProjectModuleInput::new("domain.ssrg", "fixture/opaque::domain", DOMAIN, "dist/domain.js"),
        ProjectModuleInput::new("facade.ssrg", "fixture/opaque::facade", "pub import { Secret, Box, create, read, unpack, update, box, unbox, nested, collect } from \"./domain\"", "dist/facade.js"),
        ProjectModuleInput::new("main.ssrg", "fixture/opaque::main", main, "dist/main.js"),
    ])
}

#[test]
fn preserves_opaque_identity_and_evidence_through_reexport() {
    let compiled = project(
        r#"
import { Secret, Box, create, read, unpack, update, box, unbox, nested, collect } from "./facade"
pub let secret: Secret = create 7
pub let value = (read secret, unpack secret, show secret, secret == create 7)
pub let generic: Box<Int> = box 42
pub let values = (unbox generic, nested (box secret), collect [secret])
"#,
    )
    .unwrap();
    for module in ["fixture/opaque::domain", "fixture/opaque::facade"] {
        for export in &compiled.modules[module].typed_interface.exports {
            if export.name == "Secret" || export.name == "Box" {
                assert_eq!(export.declaration_kind.as_deref(), Some("opaque-struct"));
                assert!(export.representation.is_none());
            }
        }
    }
    let ts = &compiled.modules["fixture/opaque::domain"]
        .generated
        .typescript;
    let public = ts
        .split("export type Secret = {")
        .nth(1)
        .unwrap()
        .split("};")
        .next()
        .unwrap();
    assert!(!public.contains("value"), "{public}");
    assert!(ts.contains("__ssrg$representation$Secret"));
}

#[test]
fn rejects_every_external_representation_route_including_empty_patterns() {
    for body in [
        "pub fn forge -> Secret = Secret {}",
        "pub fn forge -> Secret = Secret { value: 1 }",
        "pub fn readOutside x: Secret -> Int = x.value",
        "pub fn unpackOutside x: Secret -> Int = { let Secret { value } = x; value }",
        "pub fn emptyPattern x: Secret -> Unit = { let Secret {} = x; () }",
        "pub fn spread x: Secret -> Secret = Secret { ...x }",
        "pub fn updateOutside x: Secret -> Secret = Secret { ...x, value: 2 }",
        "pub fn genericOutside x: Box<Int> -> Int = x.value",
    ] {
        let source = format!("import {{ Secret, Box }} from \"./facade\"\n{body}");
        let failure = project(&source).unwrap_err();
        assert!(
            !format!("{failure:?}").contains("expression.invalid"),
            "{body} {failure:?}"
        );
    }
}

#[test]
fn opaque_domain_compiles_independently() {
    let result = seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        "domain.ssrg",
        "fixture/opaque::domain",
        DOMAIN,
    ));
    assert!(result.is_ok(), "{result:#?}");
}
