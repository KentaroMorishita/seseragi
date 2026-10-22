use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreModule,
    CoreType, TypeScriptFunction, TypeScriptType,
};
use seseragi_project::{link_module, standard_module_target};
use seseragi_semantics::analyze_linked_module;
use seseragi_syntax::{parse_diagnostics, parse_unlinked_module_interface};
use std::collections::BTreeMap;

const WAIT: &str = "import * as effects from \"std/effect\"\nfn wait -> Effect<{}, Never, Unit> = effects.defer (\\_ -> wait ())\n";

fn core(source: &str) -> CoreModule {
    let unlinked = parse_unlinked_module_interface("recursive.ssrg", "fixture/recursive", source);
    let linked = link_module(
        unlinked,
        &BTreeMap::from([(
            "std/effect".to_owned(),
            standard_module_target("std/effect").unwrap(),
        )]),
    )
    .unwrap();
    let analyzed =
        analyze_linked_module(parse_diagnostics("recursive.ssrg", source), linked, source).unwrap();
    lower_typed_module(analyzed.typed_hir)
}

#[test]
fn checked_wait_defer_keeps_a_closed_annotation_and_effect_type_import() {
    let module = core(WAIT);
    assert!(!module.functions[0].is_effect);
    assert_eq!(module.functions[0].parameters.len(), 1);
    let typescript = lower_core_module_to_typescript_ir(module);
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            is_effect: false,
            value_type: Some(TypeScriptType::Function { parameter, result }),
            parameters,
            ..
        } if parameters.len() == 1
            && parameter.as_ref() == &TypeScriptType::Undefined
            && matches!(result.as_ref(), TypeScriptType::Reference { name, arguments }
                if name == "Effect"
                    && arguments.as_slice() == [
                        TypeScriptType::Record { fields: Vec::new() },
                        TypeScriptType::Never,
                        TypeScriptType::Undefined,
                    ])
    ));
    assert_eq!(
        typescript
            .type_imports
            .iter()
            .filter(|import| import.feature == "effect.core.type")
            .count(),
        1
    );
    let generated = emit_typescript_module(typescript, WAIT);
    assert!(generated.typescript.contains("type Effect as Effect"));
    assert!(generated
        .typescript
        .contains("const wait: (argument: undefined) => Effect<"));
    assert!(generated.typescript.contains("wait(undefined)"));
}

#[test]
fn nested_checked_effect_result_keeps_its_channels_and_one_import() {
    let source = "import * as effects from \"std/effect\"\nfn wait -> Effect<{}, Never, Effect<{}, Never, Unit>> = effects.defer (\\_ -> wait ())\n";
    let typescript = lower_core_module_to_typescript_ir(core(source));
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            value_type: Some(TypeScriptType::Function { result, .. }),
            ..
        } if matches!(result.as_ref(), TypeScriptType::Reference { arguments, .. }
            if matches!(&arguments[2], TypeScriptType::Reference { name, arguments }
                if name == "Effect" && arguments[1] == TypeScriptType::Never
                    && arguments[2] == TypeScriptType::Undefined))
    ));
    assert_eq!(
        typescript
            .type_imports
            .iter()
            .filter(|import| import.feature == "effect.core.type")
            .count(),
        1
    );
}

#[test]
fn canonical_effect_annotation_freshens_only_its_runtime_import() {
    let mut module = core(WAIT);
    let CoreType::Function { result, .. } = module.functions[0].value_type.as_mut().unwrap() else {
        panic!("expected checked function contract")
    };
    let arguments = match result.as_ref() {
        CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
            arguments.clone()
        }
        _ => panic!("expected checked Effect result"),
    };
    **result = CoreType::ExternalNamed {
        name: "ActualEffect".into(),
        canonical: "std/prelude::Effect".into(),
        arguments,
    };
    let nominal = lower_typed_module(seseragi_semantics::type_module(
        "nominal.ssrg",
        "type Effect = | LocalEffect\n",
    ));
    module.adts.extend(nominal.adts);
    let typescript = lower_core_module_to_typescript_ir(module);
    let import = typescript
        .type_imports
        .iter()
        .find(|import| import.feature == "effect.core.type")
        .unwrap();
    assert_eq!(import.local, "__ssrg$Effect");
    assert_eq!(typescript.adts[0].name, "Effect");
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            value_type: Some(TypeScriptType::Function { result, .. }),
            ..
        } if matches!(result.as_ref(), TypeScriptType::Reference { name, .. }
            if name == &import.local)
    ));
    let generated = emit_typescript_module(typescript, WAIT);
    assert!(generated
        .typescript
        .contains("type Effect as __ssrg$Effect"));
    assert!(generated.typescript.contains("type Effect ="));
}
