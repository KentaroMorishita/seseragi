use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, CoreExpr, CoreFunction, CoreModule,
    CoreParameter, CoreType, SourceSpan, TypeScriptExpr, TypeScriptFunction,
};
use seseragi_syntax::Visibility;

fn named(name: &str) -> CoreType {
    CoreType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn origin() -> SourceSpan {
    SourceSpan {
        source: "artifact/from-either/main.ssrg".to_owned(),
        start: 0,
        end: 0,
    }
}

#[test]
fn lowers_from_either_to_a_cold_runtime_helper_call() {
    let either = CoreType::Named {
        name: "Either".to_owned(),
        arguments: vec![named("String"), named("String")],
    };
    let module = CoreModule {
        schema: 1,
        stage: "core-ir".to_owned(),
        module: "artifact/from-either".to_owned(),
        adts: Vec::new(),
        bindings: Vec::new(),
        functions: vec![CoreFunction {
            symbol: "artifact/from-either::lift".to_owned(),
            visibility: Visibility::Public,
            origin: origin(),
            parameters: vec![CoreParameter {
                id: "candidate".to_owned(),
                kind: "named".to_owned(),
                type_ref: either.clone(),
            }],
            body: CoreExpr::EffectOperation {
                operation: "effect.fromEither".to_owned(),
                requirements: CoreType::Record {
                    fields: Vec::new(),
                    closed: true,
                },
                failure: named("String"),
                success: named("String"),
                arguments: vec![CoreExpr::Variable {
                    name: "candidate".to_owned(),
                    type_ref: either,
                    origin: origin(),
                }],
                origin: origin(),
            },
        }],
    };

    let typescript = lower_core_module_to_typescript_ir(module);

    assert_eq!(
        typescript.runtime_requirements,
        vec!["core.either", "core.string", "effect.core.fromEither"]
    );
    assert_eq!(typescript.imports.len(), 1);
    assert_eq!(typescript.imports[0].feature, "effect.core.fromEither");
    assert_eq!(typescript.imports[0].local, "_ssrg_effect_fromEither");
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            is_async: false,
            body: TypeScriptExpr::RuntimeCall { callee, arguments },
            ..
        } if callee == "_ssrg_effect_fromEither"
            && matches!(arguments.as_slice(), [TypeScriptExpr::Identifier { name }] if name == "candidate")
    ));

    let bundle = emit_typescript_module(typescript, "");
    assert!(bundle.typescript.contains(
        "import { fromEither as _ssrg_effect_fromEither } from \"@seseragi/runtime/effect\""
    ));
    assert!(bundle
        .typescript
        .contains("=> _ssrg_effect_fromEither(candidate)"));
    assert!(!bundle.typescript.contains("await"));
}
