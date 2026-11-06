//! Lowering must report missing checker facts as `Err`, never panic. Each case
//! checks a real source, removes one fact the checker normally provides, and
//! expects an internal error located at the Core node that needed it.

use seseragi_lowering::{
    lower_core_module_to_typescript_ir, lower_core_module_to_typescript_ir_with_plan,
    lower_typed_module, CoreExpr, CoreInstanceImplementation, CoreModule, CoreTemplatePart,
    SourceSpan, TypeScriptInternalError, TypeScriptInternalErrorKind, TypeScriptLoweringError,
    TypeScriptLoweringStage, TypeScriptOutputPlan,
};
use seseragi_semantics::type_module;

const SOURCE_NAME: &str = "internal.ssrg";

fn core(source: &str) -> CoreModule {
    let core = lower_typed_module(type_module(SOURCE_NAME, source));
    // The untouched module lowers, so each failure comes from the removed fact.
    lower_core_module_to_typescript_ir(core.clone());
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

fn internal_error(core: CoreModule) -> TypeScriptInternalError {
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
    assert_eq!(error.stage, TypeScriptLoweringStage::Expression);
    assert_eq!(
        error.kind,
        TypeScriptInternalErrorKind::MissingTraitEvidence
    );
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
    assert_eq!(error.stage, TypeScriptLoweringStage::Expression);
    assert_eq!(
        error.kind,
        TypeScriptInternalErrorKind::MissingTraitEvidence
    );
    assert_eq!(error.origin, Some(span(source, "value < 10")));
}

#[test]
fn partial_comparison_without_ord_evidence_reports_the_operator_reference() {
    let source = "pub let less: Int -> Bool = (<) 1\n";
    let error = internal_error(without_call_evidence(core(source)));
    assert_eq!(
        error.kind,
        TypeScriptInternalErrorKind::MissingTraitEvidence
    );
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
    assert_eq!(error.stage, TypeScriptLoweringStage::Instance);
    assert_eq!(
        error.kind,
        TypeScriptInternalErrorKind::MissingDerivedEvidence
    );
    assert_eq!(error.origin, Some(instance_origin));
}
