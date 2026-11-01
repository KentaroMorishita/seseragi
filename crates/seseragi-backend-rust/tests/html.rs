use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

fn named(name: &str) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments: vec![],
    }
}

#[test]
fn style_requires_canonical_types_and_matching_checked_evidence() {
    let core = compile_core_module(CompileInput::new("html.ssrg", "native/html", "import * as html from \"std/web/html\"\npub fn probe -> html.Style = html.style { color: \"red\" }\n")).unwrap().core_ir;
    let artifact = emit_rust_module(&core, "probe").unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "html"));
    for mutation in 0..10 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            type_ref,
            arguments,
            evidence,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => *type_ref = named("fake::Style"),
            2 => evidence.clear(),
            3 => evidence[0].constraint.trait_identity = Some("fake::StyleRecord".into()),
            4 => evidence[0].constraint.arguments = vec![named("String")],
            5 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "fake::StyleRecord<Record>".into();
            }
            6 => {
                let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(named("String"));
            }
            7 => {
                let duplicate = evidence[0].clone();
                evidence.push(duplicate);
            }
            8 => {
                let duplicate = arguments[0].clone();
                arguments.push(duplicate);
            }
            9 => {
                let CoreExpr::Record {
                    type_ref: CoreType::Record { fields, .. },
                    ..
                } = &mut arguments[0]
                else {
                    panic!()
                };
                fields[0].type_ref = named("Int");
            }
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some(), "mutation {mutation}: {error:?}");
    }
}

#[test]
fn html_validates_props_and_children_without_admitting_program_wasm() {
    let core = compile_core_module(CompileInput::new("html.ssrg", "native/html", "import * as html from \"std/web/html\"\npub fn probe -> html.Html<Unit> = html.p { children: \"<hello>\" }\n")).unwrap().core_ir;
    emit_rust_module(&core, "probe").unwrap();
    assert!(emit_program_wasm(&[&core], &core.module, &["probe".into()]).is_err());
    for mutation in 0..5 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            type_ref,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => *callee = "std/web/html::img".into(),
            1 => *type_ref = named("fake::Html"),
            2 => evidence[0].constraint.arguments[0] = named("Int"),
            3 => {
                let CoreExpr::Record {
                    type_ref: CoreType::Record { fields, .. },
                    ..
                } = &mut arguments[0]
                else {
                    panic!()
                };
                fields[0].name = "unknown".into();
            }
            4 => evidence.clear(),
            _ => unreachable!(),
        }
        assert!(
            emit_rust_module(&changed, "probe").is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn rendering_an_unannotated_global_uses_its_checked_initializer_type() {
    let core = compile_core_module(CompileInput::new(
        "html.ssrg",
        "native/html",
        include_str!("../../../examples/tour/lessons/web-component-children/exercise.ssrg"),
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "main").unwrap();
    let mut changed = core.clone();
    changed.bindings[0].value = CoreExpr::String {
        value: "<untrusted>".into(),
        origin: changed.bindings[0].origin.clone(),
    };
    assert!(emit_rust_module(&changed, "main").is_err());
}

#[test]
fn unused_html_operations_still_require_native_target_admission() {
    let core = compile_core_module(CompileInput::new("html.ssrg", "native/html", "import * as html from \"std/web/html\"\npub fn probe -> Unit = { let ignored = html.style { color: \"red\" }; () }\n")).unwrap().core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let error = emit_program_wasm(&[&core], &core.module, &["probe".into()]).unwrap_err();
    assert!(error.message.contains("native HTML"), "{error:?}");
}

#[test]
fn ordinary_records_keep_the_plain_representation_with_and_without_html() {
    for html in ["", "import * as html from \"std/web/html\"\npub fn styled -> html.Style = html.style { color: \"red\", zIndex: \"2\" }\n"] {
        let source = format!("{html}\nalias Point = {{ x: Int, y: Int, name: String }}\npub fn point p: Point -> Point = {{ ...p, x: p.x + 1 }}\npub fn label -> {{ label: String }} = {{ label: \"p\" }}\npub fn unrelated -> {{ color: String, zIndex: String }} = {{ color: \"blue\", zIndex: \"3\" }}\npub fn probe -> Point = point {{ x: 0, y: 0, name: \"p\" }}\n");
        let core = compile_core_module(CompileInput::new("records.ssrg", "native/html", &source)).unwrap().core_ir;
        let artifact = emit_rust_module(&core, "probe").unwrap();
        assert!(!artifact.source.contains("\0html-record:"));
        for function in artifact.source.lines().filter(|line| line.starts_with("fn v_") && line.contains("BTreeMap::new()")) {
            assert!(!function.contains("NativeStyleRecord::new()"), "{function}");
            assert!(!function.contains(".record_fields()"), "{function}");
            assert!(function.contains("V::Record(Shared::new("), "{function}");
        }
        let ordered_functions = artifact.source.lines().filter(|line| line.starts_with("fn v_") && line.contains("NativeStyleRecord::new()")).count();
        assert_eq!(ordered_functions, if html.is_empty() { 0 } else { 2 });
    }
}

#[test]
fn style_rejects_a_record_node_with_a_forged_scalar_type() {
    let mut core = compile_core_module(CompileInput::new(
        "html.ssrg",
        "native/html",
        "import * as html from \"std/web/html\"\npub fn probe -> html.Style = html.style { color: \"red\" }\n",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Call { arguments, .. } = &mut core.functions[0].body else {
        panic!()
    };
    let CoreExpr::Record { type_ref, .. } = &mut arguments[0] else {
        panic!()
    };
    *type_ref = named("std/prelude::String");
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.origin.is_some());
}
