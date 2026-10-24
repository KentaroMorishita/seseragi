use seseragi_driver::{compile_module, compile_project, CompileInput, ProjectModuleInput};
use seseragi_project::ModuleGraph;
use seseragi_semantics::{TypedDecl, TypedParameter, TypedType};
use seseragi_syntax::InterfaceType;

fn cases() -> impl Iterator<Item = (&'static str, &'static str, &'static str)> {
    [
        ("Unit", "Int", "42"),
        ("Bool", "Int", "42"),
        ("Int", "Bool", "True"),
        ("Float", "Int", "42"),
        ("Char", "Int", "42"),
        ("String", "Int", "42"),
    ]
    .into_iter()
}
fn primitive_literal(name: &str) -> &'static str {
    match name {
        "Unit" => "()",
        "Bool" => "True",
        "Int" => "7",
        "Float" => "1.5",
        "Char" => "'x'",
        "String" => "\"hello\"",
        _ => unreachable!(),
    }
}
fn declarations(name: &str, payload: &str, literal: &str) -> String {
    format!("pub struct {name} {{value: {payload}}}\npub fn read value: {name} -> {payload} = value.value\npub let nominal: {name} = {name} {{value: {literal}}}\nlet genuineUnit = ()\nlet genuineBool = True\nlet genuineInt = 42\nlet genuineFloat = 1.5\nlet genuineChar = 'x'\nlet genuineString = \"hello\"\n")
}
fn record_evidence(label: &str, modules: &[(&str, &str)]) {
    if let Some(root) = std::env::var_os("SESERAGI_NOMINAL_ANNOTATION_EVIDENCE") {
        let root = std::path::PathBuf::from(root).join(label);
        std::fs::create_dir_all(&root).unwrap();
        for (name, source) in modules {
            std::fs::write(root.join(name), source).unwrap();
        }
    }
}
#[test]
fn local_nominal_builtin_spellings_keep_owned_annotations_and_primitive_values() {
    for (name, payload, literal) in cases() {
        for alias in [false, true] {
            let mut source = declarations(name, payload, literal);
            if alias {
                source.push_str(&format!("alias Named = {name}\npub fn aliased value: Named -> {payload} = value.value\n"));
            }
            source.push_str(&format!("pub effect fn genuine = succeed {}\npub effect fn main = do {{println (show (read nominal)); primitive <- genuine (); println (show primitive)}}\n", primitive_literal(name)));
            let compiled = compile_module(CompileInput::new(
                "main.ssrg",
                "fixture/annotations",
                &source,
            ))
            .unwrap();
            let parameter = compiled
                .typed_hir
                .declarations
                .iter()
                .find_map(|declaration| match declaration {
                    TypedDecl::Fn {
                        symbol, parameters, ..
                    } if symbol.ends_with("::read") => parameters.first(),
                    _ => None,
                })
                .unwrap();
            assert!(
                matches!(parameter, TypedParameter::Named {type_ref: TypedType::ExternalNamed {canonical, ..}, ..} if canonical == &format!("fixture/annotations::{name}")),
                "{parameter:?}"
            );
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("read = (value: {name})")),
                "{}",
                compiled.generated.typescript
            );
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("as unknown as {name}")),
                "{}",
                compiled.generated.typescript
            );
            for (binding, annotation) in [
                ("genuineUnit", "undefined"),
                ("genuineBool", "boolean"),
                ("genuineInt", "number"),
                ("genuineFloat", "number"),
                ("genuineChar", "string"),
                ("genuineString", "string"),
            ] {
                assert!(
                    compiled
                        .generated
                        .typescript
                        .contains(&format!("{binding}: {annotation}")),
                    "{}",
                    compiled.generated.typescript
                );
            }
            record_evidence(
                &format!("local-{name}-{alias}"),
                &[("main.ts", &compiled.generated.typescript)],
            );
        }
    }
}
#[test]
fn imported_nominal_and_primitive_exports_from_same_provider_stay_distinct() {
    for (name, payload, literal) in cases() {
        let domain = format!(
            "{}pub effect fn genuine = succeed {}\n",
            declarations(name, payload, literal),
            primitive_literal(name)
        );
        let source = format!("import {{{name}, read, nominal, genuine}} from \"./domain\"\npub fn inspect value: {name} -> {payload} = read value\npub fn construct -> {name} = {name} {{value: {literal}}}\npub effect fn main = do {{println (show (read nominal)); primitive <- genuine (); println (show primitive)}}\n");
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/annotations::domain".to_owned(),
                std::iter::empty::<(String, String)>(),
            )
            .unwrap();
        graph
            .add_module(
                "fixture/annotations::main".to_owned(),
                [("./domain".into(), "fixture/annotations::domain".into())],
            )
            .unwrap();
        let compiled = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/annotations::domain",
                    domain,
                    "domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/annotations::main",
                    source,
                    "main.js",
                ),
            ],
        )
        .unwrap();
        let genuine = compiled.modules["fixture/annotations::domain"]
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "genuine")
            .unwrap();
        let InterfaceType::Function { result, .. } = &genuine.scheme.type_ref else {
            panic!("{:?}", genuine.scheme)
        };
        let InterfaceType::Named {
            name: effect,
            arguments,
        } = result.as_ref()
        else {
            panic!("{result:?}")
        };
        assert_eq!(effect, "Effect");
        assert!(
            matches!(&arguments[2], InterfaceType::ExternalNamed {name: primitive, canonical, provider_module, arguments, ..} if primitive == name && canonical == &format!("std/prelude::{name}") && provider_module == "std/prelude" && arguments.is_empty()),
            "{arguments:?}"
        );
        let main = &compiled.modules["fixture/annotations::main"];
        assert!(
            main.generated
                .typescript
                .contains(&format!("inspect = (value: {name})")),
            "{}",
            main.generated.typescript
        );
        let constructed = main
            .core_ir
            .functions
            .iter()
            .find(|f| f.symbol.ends_with("::construct"))
            .unwrap();
        let seseragi_lowering::CoreExpr::Record { type_ref, .. } = &constructed.body else {
            panic!("checked nominal construction: {:?}", constructed.body);
        };
        assert!(
            matches!(type_ref, seseragi_lowering::CoreType::ExternalNamed {canonical, arguments, ..} if canonical == &format!("fixture/annotations::domain::{name}") && arguments.is_empty()),
            "{type_ref:?}"
        );
        assert!(
            main.generated
                .typescript
                .contains(&format!("as unknown as {name}")),
            "{}",
            main.generated.typescript
        );
        record_evidence(
            &format!("imported-{name}"),
            &[
                (
                    "domain.ts",
                    &compiled.modules["fixture/annotations::domain"]
                        .generated
                        .typescript,
                ),
                ("main.ts", &main.generated.typescript),
            ],
        );
    }
}
#[test]
fn registered_collection_names_and_transparent_aliases_preserve_annotations() {
    for (name, literal) in [("Array", "[1]"), ("List", "`[1]")] {
        let source = format!("pub struct {name}<A> {{value: A}}\npub fn read<A> value: {name}<A> -> A = value.value\nlet nominal = {name} {{value: 42}}\nlet genuine = {literal}\npub fn identity<A> value: A -> A = value\npub fn inspect -> Int = {{let values = identity {literal}; match values {{{prefix}[head, ...tail] -> head; {prefix}[] -> 0}}}}\n", prefix=if name == "List" {"`"} else {""});
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "fixture/annotations",
            &source,
        ))
        .unwrap();
        assert!(
            compiled
                .generated
                .typescript
                .contains(&format!("read = <A,>(value: {name}<A>)")),
            "{}",
            compiled.generated.typescript
        );
        if name == "List" {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains("type List as __ssrg$List"),
                "{}",
                compiled.generated.typescript
            );
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains("genuine: __ssrg$List<number>"),
                "{}",
                compiled.generated.typescript
            );
        } else {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains("genuine: ReadonlyArray<number>"),
                "{}",
                compiled.generated.typescript
            );
        }
        record_evidence(
            &format!("local-{name}"),
            &[("main.ts", &compiled.generated.typescript)],
        );
        let domain = format!("pub struct {name}<A> {{value:A}}\npub fn read<A> value: {name}<A> -> A = value.value\npub let nominal: {name}<Int> = {name}{{value:42}}\npub effect fn genuine=succeed {literal}\n");
        let main = format!("import {{{name}, read, nominal, genuine}} from \"./domain\"\npub effect fn main=do {{println(show(read nominal)); values <- genuine (); println(show(match values {{{prefix}[head,...tail] -> head; {prefix}[] -> 0}}))}}\n", prefix=if name == "List" {"`"} else {""});
        let provider = compile_module(CompileInput::new(
            "domain.ssrg",
            "fixture/annotations::domain",
            &domain,
        ))
        .unwrap();
        record_evidence(
            &format!("provider-{name}"),
            &[
                ("domain.ts", &provider.generated.typescript),
                (
                    "typed-interface.json",
                    &serde_json::to_string_pretty(&provider.typed_interface).unwrap(),
                ),
            ],
        );
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/annotations::domain".into(),
                std::iter::empty::<(String, String)>(),
            )
            .unwrap();
        graph
            .add_module(
                "fixture/annotations::main".into(),
                [("./domain".into(), "fixture/annotations::domain".into())],
            )
            .unwrap();
        let imported = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/annotations::domain",
                    domain,
                    "domain.js",
                ),
                ProjectModuleInput::new("main.ssrg", "fixture/annotations::main", main, "main.js"),
            ],
        )
        .unwrap();
        record_evidence(
            &format!("imported-{name}"),
            &[
                (
                    "domain.ts",
                    &imported.modules["fixture/annotations::domain"]
                        .generated
                        .typescript,
                ),
                (
                    "main.ts",
                    &imported.modules["fixture/annotations::main"]
                        .generated
                        .typescript,
                ),
            ],
        );
    }
    let compiled = compile_module(CompileInput::new(
        "main.ssrg",
        "fixture/annotations",
        "alias Bool = Int\npub fn read value: Bool -> Int = value + 1\n",
    ))
    .unwrap();
    assert!(
        compiled
            .generated
            .typescript
            .contains("read = (value: number)"),
        "{}",
        compiled.generated.typescript
    );
}

#[test]
fn generic_nominal_bool_does_not_change_the_arity_of_genuine_bool_annotations() {
    let source = "pub opaque struct Bool<A> {value:A}\npub fn read<A> value:Bool<A> ->A=value.value\nlet nominal=Bool{value:42}\nlet genuine=True\npub effect fn main=do {println(show(read nominal)); if genuine then println \"true\" else println \"false\"}\n";
    let compiled = compile_module(CompileInput::new(
        "main.ssrg",
        "fixture/annotations",
        source,
    ))
    .unwrap();
    assert!(
        compiled
            .generated
            .typescript
            .contains("read = <A,>(value: Bool<A>)"),
        "{}",
        compiled.generated.typescript
    );
    assert!(
        compiled.generated.typescript.contains("genuine: boolean"),
        "{}",
        compiled.generated.typescript
    );
    record_evidence(
        "generic-Bool",
        &[("main.ts", &compiled.generated.typescript)],
    );
}

#[test]
fn foreign_opaque_builtin_spelling_uses_its_actual_emitted_alias_and_brand() {
    let source = "foreign \"typescript\" from \"./host.mjs\" {\n opaque type Bool\n pure fn obtain unit: Unit -> Bool\n}\nfn identity<A> value:A ->A=value\nfn forward ->Bool=identity(obtain ())\n";
    let compiled = compile_module(CompileInput::new(
        "main.ssrg",
        "fixture/annotations",
        source,
    ))
    .unwrap();
    let ts = &compiled.generated.typescript;
    assert!(ts.contains("__ssrg$foreign$brand$Bool"), "{ts}");
    assert!(ts.contains("invokePure<Bool>"), "{ts}");
    assert!(ts.contains("[\"unit\"], \"opaque\""), "{ts}");
    record_evidence(
        "foreign-opaque-Bool",
        &[
            ("main.ts", ts),
            ("host.mjs", "export const obtain = () => ({value:42});\n"),
        ],
    );
}

#[test]
fn imported_adt_and_newtype_builtin_spellings_keep_owner_in_annotations_and_constructor_calls() {
    for (kind, producer, constructor) in [
        ("adt", "pub type Bool = | Flag Int\n", "Flag"),
        ("newtype", "pub newtype Bool = Int\n", "Bool"),
    ] {
        for alias in [false, true] {
            let (type_import, name, constructor_import, constructor_name) = if alias {
                (
                    "Bool as Owned",
                    "Owned",
                    format!("{constructor} as Make"),
                    "Make",
                )
            } else {
                ("Bool", "Bool", constructor.into(), constructor)
            };
            let imports = if kind == "newtype" {
                type_import.to_owned()
            } else {
                format!("{type_import}, {constructor_import}")
            };
            let constructor_name = if kind == "newtype" {
                name
            } else {
                constructor_name
            };
            let source = format!("import {{{imports}}} from \"./domain\"\npub fn read value: {name} -> Int = match value {{{constructor_name} n -> n}}\nlet genuine = True\npub effect fn main = do {{println (show (read ({constructor_name} 42))); if genuine then println \"true\" else println \"false\"}}\n");
            let mut graph = ModuleGraph::new();
            graph
                .add_module(
                    "fixture/annotations::domain".into(),
                    std::iter::empty::<(String, String)>(),
                )
                .unwrap();
            graph
                .add_module(
                    "fixture/annotations::main".into(),
                    [("./domain".into(), "fixture/annotations::domain".into())],
                )
                .unwrap();
            let compiled = compile_project(
                graph,
                [
                    ProjectModuleInput::new(
                        "domain.ssrg",
                        "fixture/annotations::domain",
                        producer,
                        "domain.js",
                    ),
                    ProjectModuleInput::new(
                        "main.ssrg",
                        "fixture/annotations::main",
                        source,
                        "main.js",
                    ),
                ],
            )
            .unwrap();
            let main = &compiled.modules["fixture/annotations::main"];
            let parameter = main
                .typed_hir
                .declarations
                .iter()
                .find_map(|declaration| match declaration {
                    TypedDecl::Fn {
                        symbol, parameters, ..
                    } if symbol.ends_with("::read") => parameters.first(),
                    _ => None,
                })
                .unwrap();
            assert!(
                matches!(parameter, TypedParameter::Named {type_ref: TypedType::ExternalNamed {canonical, ..}, ..} if canonical == "fixture/annotations::domain::Bool"),
                "{parameter:?}"
            );
            assert!(
                main.generated
                    .typescript
                    .contains(&format!("read = (value: {name}")),
                "{}",
                main.generated.typescript
            );
            assert!(
                main.generated.typescript.contains("genuine: boolean"),
                "{}",
                main.generated.typescript
            );
            record_evidence(
                &format!("imported-{kind}-Bool-{alias}"),
                &[
                    (
                        "domain.ts",
                        &compiled.modules["fixture/annotations::domain"]
                            .generated
                            .typescript,
                    ),
                    ("main.ts", &main.generated.typescript),
                ],
            );
        }
    }
}

#[test]
fn exported_aliases_preserve_nominal_primitive_owners_through_three_modules() {
    let names = ["Unit", "Bool", "Int", "Float", "Char", "String"];
    let mut domain = "pub struct Marker { tokens: Array<Never> }\n".to_owned();
    let mut bridge = "import * as domain from \"./domain\"\n".to_owned();
    let mut main = "import * as domain from \"./domain\"\nimport {".to_owned();
    main.push_str(
        &names
            .iter()
            .map(|name| format!("Alias{name} as Renamed{name}"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    main.push_str("} from \"./bridge\"\n");
    for name in names {
        domain.push_str(&format!("pub struct {name} {{ value: Marker }}\n"));
        bridge.push_str(&format!("pub alias Alias{name} = domain.{name}\n"));
        main.push_str(&format!(
            "pub fn read{name} value: Renamed{name} -> domain.Marker = value.value\n"
        ));
    }
    main.push_str("let genuineUnit = ()\nlet genuineBool = True\nlet genuineInt = 42\nlet genuineFloat = 1.5\nlet genuineChar = 'x'\nlet genuineString = \"hello\"\npub effect fn main = do { println \"audit\" }\n");
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/annotations::domain".into(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/annotations::bridge".into(),
            [("./domain".into(), "fixture/annotations::domain".into())],
        )
        .unwrap();
    graph
        .add_module(
            "fixture/annotations::main".into(),
            [
                ("./domain".into(), "fixture/annotations::domain".into()),
                ("./bridge".into(), "fixture/annotations::bridge".into()),
            ],
        )
        .unwrap();
    let compiled = compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "domain.ssrg",
                "fixture/annotations::domain",
                domain,
                "domain.js",
            ),
            ProjectModuleInput::new(
                "bridge.ssrg",
                "fixture/annotations::bridge",
                bridge,
                "bridge.js",
            ),
            ProjectModuleInput::new("main.ssrg", "fixture/annotations::main", main, "main.js"),
        ],
    )
    .unwrap();
    for name in names {
        let declaration = compiled.modules["fixture/annotations::main"]
            .typed_hir
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                TypedDecl::Fn {
                    symbol, parameters, ..
                } if symbol.ends_with(&format!("::read{name}")) => parameters.first(),
                _ => None,
            })
            .unwrap();
        assert!(
            matches!(declaration, TypedParameter::Named {type_ref:TypedType::ExternalNamed {canonical, ..}, ..} if canonical == &format!("fixture/annotations::domain::{name}")),
            "{declaration:?}"
        );
    }
    record_evidence(
        "transitive-six-primitive-aliases",
        &[
            (
                "domain.ts",
                &compiled.modules["fixture/annotations::domain"]
                    .generated
                    .typescript,
            ),
            (
                "bridge.ts",
                &compiled.modules["fixture/annotations::bridge"]
                    .generated
                    .typescript,
            ),
            (
                "main.ts",
                &compiled.modules["fixture/annotations::main"]
                    .generated
                    .typescript,
            ),
        ],
    );
}
