//! Lowering must report missing checker facts as `Err`, never panic. Each case
//! checks a real source, removes one fact the checker normally provides, and
//! expects an internal error located at the Core node that needed it.

use seseragi_lowering::{
    lower_core_module_to_typescript_ir, lower_core_module_to_typescript_ir_with_plan,
    lower_typed_module, CoreExpr, CoreInstanceImplementation, CoreModule, CoreTemplatePart,
    LoweringInternalError, LoweringInternalErrorKind, LoweringStage, SourceSpan,
    TypeScriptLoweringError, TypeScriptOutputPlan,
};
use seseragi_semantics::type_module;

const SOURCE_NAME: &str = "internal.ssrg";

fn core(source: &str) -> CoreModule {
    let diagnostics = seseragi_semantics::semantic_diagnostics(SOURCE_NAME, source);
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != seseragi_syntax::DiagnosticSeverity::Error),
        "{diagnostics:?}"
    );
    let core =
        lower_typed_module(type_module(SOURCE_NAME, source)).expect("valid test lowering input");
    // The untouched module lowers, so each failure comes from the removed fact.
    lower_core_module_to_typescript_ir(core.clone()).expect("valid test lowering input");
    core
}

/// Removes selected trait evidence from calls, operators and interpolations.
fn strip_evidence(expr: &mut CoreExpr) {
    match expr {
        CoreExpr::Template { parts, .. } => {
            for part in parts {
                if let CoreTemplatePart::Interpolation {
                    value, evidence, ..
                } = part
                {
                    *evidence = None;
                    strip_evidence(value);
                }
            }
        }
        CoreExpr::Binary {
            left,
            right,
            evidence,
            ..
        } => {
            evidence.clear();
            strip_evidence(left);
            strip_evidence(right);
        }
        CoreExpr::Call {
            arguments,
            evidence,
            ..
        } => {
            evidence.clear();
            arguments.iter_mut().for_each(strip_evidence);
        }
        CoreExpr::Variable { evidence, .. } => evidence.clear(),
        _ => {}
    }
}

fn without_call_evidence(mut core: CoreModule) -> CoreModule {
    core.bindings
        .iter_mut()
        .for_each(|binding| strip_evidence(&mut binding.value));
    core.functions
        .iter_mut()
        .for_each(|function| strip_evidence(&mut function.body));
    core
}

fn internal_error(core: CoreModule) -> LoweringInternalError {
    match lower_core_module_to_typescript_ir_with_plan(core, &TypeScriptOutputPlan::default()) {
        Err(TypeScriptLoweringError::Internal(error)) => error,
        other => panic!("expected an internal lowering error, got {other:?}"),
    }
}

fn span(source: &str, text: &str) -> SourceSpan {
    let start = source.find(text).expect("span text");
    SourceSpan {
        source: SOURCE_NAME.to_owned(),
        start,
        end: start + text.len(),
    }
}

#[test]
fn template_interpolation_without_show_evidence_is_an_expression_error() {
    let source = "pub fn label value: Int -> String = `value=${value}`\n";
    let error = internal_error(without_call_evidence(core(source)));
    assert_eq!(error.stage, LoweringStage::Expression);
    assert_eq!(error.kind, LoweringInternalErrorKind::MissingTraitEvidence);
    assert_eq!(error.origin, Some(span(source, "`value=${value}`")));
    assert!(
        error.to_string().contains("template interpolation"),
        "{error}"
    );
}

#[test]
fn comparison_without_ord_evidence_reports_the_binary_expression() {
    let source = "pub fn small value: Int -> Bool = value < 10\n";
    let error = internal_error(without_call_evidence(core(source)));
    assert_eq!(error.stage, LoweringStage::Expression);
    assert_eq!(error.kind, LoweringInternalErrorKind::MissingTraitEvidence);
    assert_eq!(error.origin, Some(span(source, "value < 10")));
}

#[test]
fn partial_comparison_without_ord_evidence_reports_the_operator_reference() {
    let source = "pub let less: Int -> Bool = (<) 1\n";
    let error = internal_error(without_call_evidence(core(source)));
    assert_eq!(error.kind, LoweringInternalErrorKind::MissingTraitEvidence);
    let origin = error.origin.expect("located error");
    let call = span(source, "(<) 1");
    assert!(
        call.start <= origin.start && origin.end <= call.end,
        "{origin:?} is outside {call:?}"
    );
}

#[test]
fn derived_show_without_payload_evidence_reports_the_instance() {
    let source =
        "type Box deriving Show = | Box Int\npub fn label value: Box -> String = show value\n";
    let core = core(source);
    let instance_origin = core
        .instances
        .iter()
        .find(|instance| instance.trait_name == "Show")
        .expect("derived Show instance")
        .origin
        .clone();
    let mut core = core;
    for instance in &mut core.instances {
        if let CoreInstanceImplementation::DerivedShow {
            payload_evidence, ..
        } = &mut instance.implementation
        {
            payload_evidence.clear();
        }
    }
    let error = internal_error(core);
    assert_eq!(error.stage, LoweringStage::Instance);
    assert_eq!(
        error.kind,
        LoweringInternalErrorKind::MissingDerivedEvidence
    );
    assert_eq!(error.origin, Some(instance_origin));
}

fn typescript(source: &str) -> seseragi_lowering::TypeScriptModule {
    let module = lower_core_module_to_typescript_ir(core(source)).expect("valid TypeScript IR");
    seseragi_lowering::emit_typescript_module(module.clone(), source)
        .expect("valid TypeScript output");
    module
}

fn assert_failure(
    error: LoweringInternalError,
    stage: LoweringStage,
    kind: LoweringInternalErrorKind,
    origin: SourceSpan,
) {
    assert_eq!(error.stage, stage);
    assert_eq!(error.kind, kind);
    assert_eq!(error.origin, Some(origin));
}

#[test]
fn type_conversion_without_import_binding_reports_the_function_declaration() {
    let source =
        "pub fn recurse value: Int -> Int = if value == 0 then 0 else recurse (value - 1)\n";
    let mut module = core(source);
    let origin = module.functions[0].origin.clone();
    // A retained recursive annotation still needs its checked type binding.
    module.functions[0].value_type = Some(seseragi_lowering::CoreType::ExternalNamed {
        name: "Lost".into(),
        canonical: "lost/module::Lost".into(),
        arguments: vec![],
    });
    assert_failure(
        internal_error(module),
        LoweringStage::Type,
        LoweringInternalErrorKind::MissingTypeInformation,
        origin,
    );
}

#[test]
fn comprehension_without_a_generator_reports_the_comprehension() {
    let source = "pub fn values items: Array<Int> -> Array<Int> = [item | item <- items]\n";
    let mut module = core(source);
    let CoreExpr::ArrayComprehension {
        clauses, origin, ..
    } = &mut module.functions[0].body
    else {
        panic!("comprehension");
    };
    let origin = origin.clone();
    clauses.clear();
    assert_failure(
        internal_error(module),
        LoweringStage::Expression,
        LoweringInternalErrorKind::UnexpectedCoreShape,
        origin,
    );
}

#[test]
fn nested_comprehension_without_dictionary_reports_the_inner_generator() {
    let source = "pub fn values items: Array<Int> -> Array<Int> = [inner | outer <- items, inner <- [outer]]\n";
    let mut module = core(source);
    let CoreExpr::ArrayComprehension { clauses, .. } = &mut module.functions[0].body else {
        panic!("comprehension");
    };
    let seseragi_lowering::CoreComprehensionClause::Generator {
        evidence, origin, ..
    } = &mut clauses[1]
    else {
        panic!("inner generator");
    };
    let origin = origin.clone();
    evidence.evidence = seseragi_lowering::CoreInstanceEvidence::Local {
        identity: "lost-iterable".into(),
        type_arguments: vec![],
        evidence_arguments: vec![],
    };
    assert_failure(
        internal_error(module),
        LoweringStage::Expression,
        LoweringInternalErrorKind::MissingTraitEvidence,
        origin,
    );
}

#[test]
fn core_conversion_without_generalized_binding_reports_the_declaration() {
    let source = "let render = \\value -> show value\n";
    let mut typed = type_module(SOURCE_NAME, source);
    lower_typed_module(typed.clone()).expect("valid Typed HIR");
    let seseragi_semantics::TypedDecl::Let {
        bindings,
        scheme,
        origin,
        ..
    } = &mut typed.declarations[0]
    else {
        panic!("generalized binding");
    };
    assert!(!scheme.constraints.is_empty());
    let origin = SourceSpan {
        source: SOURCE_NAME.into(),
        start: origin.start,
        end: origin.end,
    };
    bindings.clear();
    assert_failure(
        lower_typed_module(typed).unwrap_err(),
        LoweringStage::Core,
        LoweringInternalErrorKind::UnexpectedTypedHirShape,
        origin,
    );
}

#[test]
fn core_conversion_of_generalized_pattern_reports_the_inner_pattern() {
    let source = "pub fn render -> String = { let display = \\value -> show value; display 42 }\n";
    let mut typed = type_module(SOURCE_NAME, source);
    lower_typed_module(typed.clone()).expect("valid Typed HIR");
    let seseragi_semantics::TypedDecl::Fn {
        body: seseragi_semantics::TypedExpr::Block { statements, .. },
        ..
    } = &mut typed.declarations[0]
    else {
        panic!("block");
    };
    let seseragi_semantics::TypedBlockStatement::Let {
        pattern,
        constraints,
        ..
    } = &mut statements[0]
    else {
        panic!("local let");
    };
    assert!(!constraints.is_empty());
    let seseragi_semantics::TypedPattern::Binding {
        type_ref, origin, ..
    } = pattern
    else {
        panic!("binding pattern");
    };
    let expected = SourceSpan {
        source: SOURCE_NAME.into(),
        start: origin.start,
        end: origin.end,
    };
    *pattern = seseragi_semantics::TypedPattern::Wildcard {
        type_ref: type_ref.clone(),
        origin: *origin,
    };
    assert_failure(
        lower_typed_module(typed).unwrap_err(),
        LoweringStage::Core,
        LoweringInternalErrorKind::UnexpectedTypedHirShape,
        expected,
    );
}

#[test]
fn derived_display_emission_without_dictionary_import_reports_the_instance() {
    let source = "type Box deriving Show = | Box Int\n";
    let mut module = typescript(source);
    let origin = module.instances[0].origin.clone();
    module
        .type_imports
        .retain(|import| import.feature != "core.show.dictionary");
    assert_failure(
        seseragi_lowering::emit_typescript_module(module, source).unwrap_err(),
        LoweringStage::Emission,
        LoweringInternalErrorKind::MissingDerivedEvidence,
        origin,
    );
}

#[test]
fn derived_instance_emission_without_head_reports_the_instance() {
    let source = "type Box deriving Show = | Box Int\n";
    let mut module = typescript(source);
    let origin = module.instances[0].origin.clone();
    module.instances[0].arguments.clear();
    assert_failure(
        seseragi_lowering::emit_typescript_module(module, source).unwrap_err(),
        LoweringStage::Emission,
        LoweringInternalErrorKind::MissingDerivedEvidence,
        origin,
    );
}

#[test]
fn derived_instance_emission_rejects_inconsistent_trait_names() {
    for source in [
        "type Box deriving Show = | Box Int\n",
        "type Box deriving Eq = | Box Int\n",
        "type Box deriving JsonEncode = | Box Int\n",
    ] {
        let mut module = typescript(source);
        let origin = module.instances[0].origin.clone();
        module.instances[0].trait_name = "Lost".into();
        assert_failure(
            seseragi_lowering::emit_typescript_module(module, source).unwrap_err(),
            LoweringStage::Emission,
            LoweringInternalErrorKind::MissingDerivedEvidence,
            origin,
        );
    }
}

#[test]
fn derived_json_newtype_emission_without_payload_reports_the_instance() {
    let source = "newtype Id deriving JsonEncode = Int\n";
    let mut module = typescript(source);
    let origin = module.instances[0].origin.clone();
    let seseragi_lowering::TypeScriptInstanceImplementation::DerivedJson { variants, .. } =
        &mut module.instances[0].implementation
    else {
        panic!("JSON newtype");
    };
    variants[0].payload = None;
    assert_failure(
        seseragi_lowering::emit_typescript_module(module, source).unwrap_err(),
        LoweringStage::Emission,
        LoweringInternalErrorKind::MissingDerivedEvidence,
        origin,
    );
}

#[test]
fn erased_newtype_emission_without_constructor_reports_the_type_declaration() {
    let source = "pub newtype Id = Int\n";
    let core = core(source);
    let symbol = core.adts[0].variants[0].symbol.clone();
    let mut module = seseragi_lowering::lower_core_module_to_typescript_ir_with_options(
        core.clone(),
        &TypeScriptOutputPlan::default(),
        &seseragi_lowering::TypeScriptLoweringOptions {
            erased_newtype_constructors: [symbol].into(),
            ..Default::default()
        },
    )
    .expect("valid erased newtype");
    seseragi_lowering::emit_typescript_module(module.clone(), source).expect("valid erased output");
    let origin = module.adts[0].origin.clone();
    assert!(module.adts[0].erased_newtype);
    module.adts[0].variants.clear();
    assert_failure(
        seseragi_lowering::emit_typescript_module_with_output_paths(
            module,
            source,
            Default::default(),
        )
        .unwrap_err(),
        LoweringStage::Emission,
        LoweringInternalErrorKind::UnexpectedTypeScriptShape,
        origin,
    );
}

#[test]
fn partial_kind_emission_reports_the_witness_source() {
    let source = "pub struct Holder<F<_>, A> { value: F<A> }\npub fn read holder: Holder<Either<_, Int>, String> -> Either<String, Int> = holder.value\n";
    let mut module = typescript(source);
    let kinds = module.higher_kinds.as_mut().expect("higher kinds");
    let origin = kinds.partial_origins[0]
        .clone()
        .expect("partial witness source");
    kinds.partials[0].fill(false);
    assert_failure(
        seseragi_lowering::emit_typescript_module(module, source).unwrap_err(),
        LoweringStage::Emission,
        LoweringInternalErrorKind::UnexpectedTypeScriptShape,
        origin,
    );
}

#[test]
fn simple_typescript_entry_returns_internal_errors() {
    let source = "pub fn label value: Int -> String = `value=${value}`\n";
    let error =
        lower_core_module_to_typescript_ir(without_call_evidence(core(source))).unwrap_err();
    let TypeScriptLoweringError::Internal(error) = error else {
        panic!("internal error");
    };
    assert_failure(
        error,
        LoweringStage::Expression,
        LoweringInternalErrorKind::MissingTraitEvidence,
        span(source, "`value=${value}`"),
    );
}

#[test]
fn erased_derived_instance_emission_without_payload_reports_the_instance() {
    for trait_name in [
        "Show",
        "Debug",
        "Eq",
        "Ord",
        "Hash",
        "JsonEncode",
        "JsonDecode",
    ] {
        let source =
            "newtype Id deriving Eq, Ord, Hash, Show, Debug, JsonEncode, JsonDecode = Int\n";
        let core = core(&source);
        let constructor = core.adts[0].variants[0].symbol.clone();
        let mut module = seseragi_lowering::lower_core_module_to_typescript_ir_with_options(
            core,
            &TypeScriptOutputPlan::default(),
            &seseragi_lowering::TypeScriptLoweringOptions {
                erased_newtype_constructors: [constructor].into(),
                ..Default::default()
            },
        )
        .expect("checked erased newtype");
        seseragi_lowering::emit_typescript_module(module.clone(), &source)
            .expect("valid derived output");
        let instance = module
            .instances
            .iter_mut()
            .find(|instance| instance.trait_name == trait_name)
            .unwrap_or_else(|| panic!("missing derived {trait_name} instance"));
        let origin = instance.origin.clone();
        use seseragi_lowering::TypeScriptInstanceImplementation as Implementation;
        let variants = match &mut instance.implementation {
            Implementation::DerivedShow { variants, .. }
            | Implementation::DerivedJson { variants, .. }
            | Implementation::DerivedStructural { variants, .. } => variants,
            other => panic!("derived newtype: {other:?}"),
        };
        variants[0].payload = None;
        assert_failure(
            seseragi_lowering::emit_typescript_module(module, &source).unwrap_err(),
            LoweringStage::Emission,
            LoweringInternalErrorKind::MissingDerivedEvidence,
            origin,
        );
    }
}

#[test]
fn structural_emission_without_constraint_head_reports_the_instance() {
    let source = "type Box<A> deriving Eq = | Box A\n";
    let mut module = typescript(source);
    let instance = &mut module.instances[0];
    let origin = instance.origin.clone();
    assert_eq!(instance.constraints.len(), 1);
    instance.constraints[0].arguments.clear();
    assert_failure(
        seseragi_lowering::emit_typescript_module(module, source).unwrap_err(),
        LoweringStage::Emission,
        LoweringInternalErrorKind::MissingDerivedEvidence,
        origin,
    );
}

#[test]
fn type_conversion_in_a_local_annotation_reports_the_local_declaration() {
    let source = "pub fn answer -> Int = { fn identity value: Int -> Int = value; identity 42 }\n";
    let mut module = core(source);
    let CoreExpr::Sequence { statements, .. } = &mut module.functions[0].body else {
        panic!("block");
    };
    let seseragi_lowering::CoreStatement::LocalFunction {
        return_type,
        origin,
        ..
    } = &mut statements[0]
    else {
        panic!("local function");
    };
    let origin = origin.clone();
    *return_type = Some(seseragi_lowering::CoreType::ExternalNamed {
        name: "Lost".into(),
        canonical: "lost/module::Lost".into(),
        arguments: vec![],
    });
    assert_failure(
        internal_error(module),
        LoweringStage::Type,
        LoweringInternalErrorKind::MissingTypeInformation,
        origin,
    );
}

#[test]
fn partial_witness_positions_round_trip_without_changing_existing_shapes() {
    let source = "pub struct Holder<F<_>, A> { value: F<A> }\npub fn read holder: Holder<Either<_, Int>, String> -> Either<String, Int> = holder.value\n";
    let module = typescript(source);
    let kinds = module.higher_kinds.expect("partial witnesses");
    let json = serde_json::to_value(&kinds).unwrap();
    assert_eq!(json["partials"], serde_json::json!(kinds.partials));
    assert_eq!(
        serde_json::from_value::<seseragi_lowering::TypeScriptKinds>(json.clone()).unwrap(),
        kinds
    );
    let mut older = json;
    older.as_object_mut().unwrap().remove("partialOrigins");
    let restored = serde_json::from_value::<seseragi_lowering::TypeScriptKinds>(older).unwrap();
    assert_eq!(restored.partials, kinds.partials);
    assert!(restored.partial_origins.is_empty());
}
