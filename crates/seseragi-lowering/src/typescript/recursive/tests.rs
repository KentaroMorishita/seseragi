use super::{annotation, cyclic_members, providerless_effect_type};
use crate::{lower_typed_module, CoreInstanceConstraint, CoreModule, CoreType, TypeScriptType};
use seseragi_semantics::{type_module, ExternalTypeBinding};
use seseragi_syntax::TypeParameter;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn cycles_exclude_callers_and_sinks_but_include_saved_value_edges() {
    let edges = BTreeMap::from([
        ("caller", vec!["first", "sink"]),
        ("first", vec!["saved"]),
        ("saved", vec!["second"]),
        ("second", vec!["first", "sink"]),
        ("self", vec!["self"]),
        ("sink", vec![]),
    ])
    .into_iter()
    .map(|(name, refs)| {
        (
            name.to_owned(),
            refs.into_iter().map(str::to_owned).collect(),
        )
    })
    .collect();
    assert_eq!(
        cyclic_members(&edges),
        BTreeSet::from([
            "first".into(),
            "saved".into(),
            "second".into(),
            "self".into()
        ])
    );
}

#[test]
fn a_long_acyclic_graph_does_not_use_recursive_compiler_stack() {
    let edges = (0..10000)
        .map(|n| {
            (
                n.to_string(),
                if n == 9999 {
                    BTreeSet::new()
                } else {
                    BTreeSet::from([(n + 1).to_string()])
                },
            )
        })
        .collect();
    assert!(cyclic_members(&edges).is_empty());
}

#[test]
fn effect_dependency_analysis_does_not_weaken_pure_capture_refusals() {
    let module=lower_typed_module(type_module("mutual.ssrg", "import * as effects from \"std/effect\"\neffect fn checkpoint _unit: Unit -> String = do { effects.yieldNow (); wait () }\nfn wait -> Effect<{}, Never, String> = effects.defer checkpoint\n"));
    assert!(crate::core_free_variables(&module.functions[0].body).is_none());
    let refs = crate::core_analysis::core_term_dependencies(&module.functions[0].body).unwrap();
    assert!(refs.names.contains(&module.functions[1].symbol));
}

fn module() -> CoreModule {
    lower_typed_module(type_module(
        "recursive.ssrg",
        "fn wait -> Effect<{}, Never, Unit> = wait ()\n",
    ))
}

fn external(canonical: &str) -> CoreType {
    CoreType::ExternalNamed {
        name: "same display spelling".into(),
        canonical: canonical.into(),
        arguments: Vec::new(),
    }
}

fn result(ty: &mut CoreType) -> &mut CoreType {
    match ty {
        CoreType::Function { result: next, .. } => result(next),
        _ => ty,
    }
}

fn channels(ty: &mut CoreType) -> &mut Vec<CoreType> {
    match result(ty) {
        CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => arguments,
        _ => panic!("expected a checked Effect result"),
    }
}

fn accepted(module: &CoreModule) -> bool {
    providerless_effect_type(module, &module.functions[0]).is_some()
}

#[test]
fn closed_canonical_contract_keeps_source_and_returned_function_stages() {
    let mut module = module();
    assert!(accepted(&module));
    let original = module.functions[0].value_type.take().unwrap();
    let CoreType::Function { parameter, result } = original else {
        panic!("expected checked source parameter")
    };
    module.functions[0].value_type = Some(CoreType::Function {
        parameter: parameter.clone(),
        result: Box::new(CoreType::Function { parameter, result }),
    });
    let checked = providerless_effect_type(&module, &module.functions[0]).unwrap();
    assert_eq!(module.functions[0].parameters.len(), 1);
    let names = super::TypeScriptTypeContext::default()
        .with_runtime_type_name("std/prelude::Effect".into(), "Work".into());
    assert!(matches!(
        annotation(&checked, &names),
        Some(TypeScriptType::Function { result, .. })
            if matches!(result.as_ref(), TypeScriptType::Function { result, .. }
                if matches!(result.as_ref(), TypeScriptType::Reference { name, arguments }
                    if name == "Work" && arguments.len() == 3))
    ));
}

#[test]
fn declaration_guards_keep_effect_functions_generics_and_dictionaries_out() {
    let original = module();
    let mut effect = original.clone();
    effect.functions[0].is_effect = true;
    assert!(!accepted(&effect));
    for parameter in [
        TypeParameter::value("Unit"),
        TypeParameter::constructor("Effect", 3),
    ] {
        let mut generic = original.clone();
        generic.functions[0].type_parameters.push(parameter);
        assert!(!accepted(&generic));
    }
    let mut dictionary = original;
    dictionary.functions[0]
        .constraints
        .push(CoreInstanceConstraint {
            name: "Show".into(),
            trait_identity: Some("std/prelude::Show".into()),
            arguments: vec![external("std/prelude::Unit")],
        });
    assert!(!accepted(&dictionary));
}

#[test]
fn missing_proof_holes_open_rows_and_unsaturated_kinds_do_not_supply_annotations() {
    let original = module();
    let mut missing = original.clone();
    missing.functions[0].value_type = None;
    assert!(!accepted(&missing));
    for replacement in [
        CoreType::Hole,
        CoreType::ExternalNamed {
            name: "Maybe".into(),
            canonical: "std/prelude::Maybe".into(),
            arguments: Vec::new(),
        },
    ] {
        let mut invalid = original.clone();
        channels(invalid.functions[0].value_type.as_mut().unwrap())[2] = replacement;
        assert!(!accepted(&invalid));
    }
    let mut open = original;
    channels(open.functions[0].value_type.as_mut().unwrap())[0] = CoreType::Record {
        closed: false,
        fields: Vec::new(),
    };
    assert!(!accepted(&open));
}

#[test]
fn canonical_identity_and_kind_override_display_spelling() {
    let original = module();
    let mut canonical = original.clone();
    channels(canonical.functions[0].value_type.as_mut().unwrap())[2] =
        external("std/prelude::Unit");
    assert!(accepted(&canonical));
    let mut nominal = original.clone();
    channels(nominal.functions[0].value_type.as_mut().unwrap())[2] = external("user/main::Unit");
    assert!(!accepted(&nominal));
    let mut wrong_kind = canonical;
    wrong_kind.external_type_bindings.push(ExternalTypeBinding {
        spelling: "RealUnit".into(),
        canonical: "std/prelude::Unit".into(),
        arity: Some(1),
        provider: None,
    });
    assert!(!accepted(&wrong_kind));
}

#[test]
fn lexical_nominals_and_ambiguous_import_owners_are_not_prelude_types() {
    let original = module();
    let local = lower_typed_module(type_module("local.ssrg", "type Unit = | LocalUnit\n"));
    let mut nominal = original.clone();
    nominal.adts.extend(local.adts);
    channels(nominal.functions[0].value_type.as_mut().unwrap())[2] = CoreType::Named {
        name: "Unit".into(),
        arguments: Vec::new(),
    };
    assert!(!accepted(&nominal));
    let mut ambiguous = original;
    channels(ambiguous.functions[0].value_type.as_mut().unwrap())[2] = CoreType::Named {
        name: "Alias".into(),
        arguments: Vec::new(),
    };
    for canonical in ["std/prelude::Unit", "user/main::Unit"] {
        ambiguous.external_type_bindings.push(ExternalTypeBinding {
            spelling: "Alias".into(),
            canonical: canonical.into(),
            arity: Some(0),
            provider: None,
        });
    }
    assert!(!accepted(&ambiguous));
}

#[test]
fn source_parameter_type_and_kind_must_match_the_checked_contract() {
    let original = module();
    let mut wrong_parameter = original.clone();
    wrong_parameter.functions[0].parameters[0].type_ref = external("std/prelude::String");
    assert!(!accepted(&wrong_parameter));
    let mut wrong_kind = original.clone();
    wrong_kind.functions[0].parameters[0].kind = "dictionary".into();
    assert!(!accepted(&wrong_kind));
    let mut too_many = original;
    let parameter = too_many.functions[0].parameters[0].clone();
    too_many.functions[0].parameters.push(parameter);
    assert!(!accepted(&too_many));
}

#[test]
fn nested_effects_use_the_same_closed_environment_and_owner_proof() {
    let mut nested = module();
    let inner = result(nested.functions[0].value_type.as_mut().unwrap()).clone();
    channels(nested.functions[0].value_type.as_mut().unwrap())[2] = inner;
    assert!(accepted(&nested));
    let channels = channels(nested.functions[0].value_type.as_mut().unwrap());
    let (CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. }) =
        &mut channels[2]
    else {
        panic!("expected nested Effect")
    };
    arguments[0] = CoreType::Hole;
    assert!(!accepted(&nested));
}
