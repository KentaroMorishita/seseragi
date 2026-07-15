use crate::{
    CoreCallEvidence, CoreDecisionBranch, CoreExpr, CoreInstanceEvidence,
    CoreInstanceImplementation, CoreModule, CoreStatement,
};
use std::collections::BTreeSet;

pub(super) fn imported_instance_evidence(module: &CoreModule) -> BTreeSet<(String, String)> {
    let mut imported = BTreeSet::new();
    for binding in &module.bindings {
        collect_expr(&binding.value, &mut imported);
    }
    for function in &module.functions {
        collect_expr(&function.body, &mut imported);
    }
    for instance in &module.instances {
        match &instance.implementation {
            CoreInstanceImplementation::DerivedShow {
                payload_evidence, ..
            } => {
                for payload in payload_evidence {
                    collect_evidence(&payload.evidence, &mut imported);
                }
            }
            CoreInstanceImplementation::UserDefined { methods } => {
                for method in methods {
                    collect_expr(&method.body, &mut imported);
                }
            }
        }
    }
    imported
}

fn collect_expr(expr: &CoreExpr, imported: &mut BTreeSet<(String, String)>) {
    match expr {
        CoreExpr::Variable { evidence, .. } => collect_call_evidence(evidence, imported),
        CoreExpr::Call {
            arguments,
            evidence,
            ..
        } => {
            collect_call_evidence(evidence, imported);
            for argument in arguments {
                collect_expr(argument, imported);
            }
        }
        CoreExpr::Tuple { elements, .. } | CoreExpr::Array { elements, .. } => {
            for element in elements {
                collect_expr(element, imported);
            }
        }
        CoreExpr::Binary {
            left,
            right,
            evidence,
            ..
        } => {
            collect_call_evidence(evidence, imported);
            collect_expr(left, imported);
            collect_expr(right, imported);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr(condition, imported);
            collect_expr(then_branch, imported);
            collect_expr(else_branch, imported);
        }
        CoreExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            collect_expr(scrutinee, imported);
            for branch in branches {
                collect_branch(branch, imported);
            }
        }
        CoreExpr::EffectOperation { arguments, .. } | CoreExpr::EffectInvoke { arguments, .. } => {
            for argument in arguments {
                collect_expr(argument, imported);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    CoreStatement::Effect { value }
                    | CoreStatement::PureLet { value, .. }
                    | CoreStatement::Bind { value, .. } => collect_expr(value, imported),
                }
            }
            collect_expr(result, imported);
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
    }
}

fn collect_branch(branch: &CoreDecisionBranch, imported: &mut BTreeSet<(String, String)>) {
    if let Some(guard) = &branch.guard {
        collect_expr(guard, imported);
    }
    collect_expr(&branch.value, imported);
}

fn collect_call_evidence(evidence: &[CoreCallEvidence], imported: &mut BTreeSet<(String, String)>) {
    for selected in evidence {
        collect_evidence(&selected.evidence, imported);
    }
}

fn collect_evidence(evidence: &CoreInstanceEvidence, imported: &mut BTreeSet<(String, String)>) {
    match evidence {
        CoreInstanceEvidence::Imported {
            identity,
            provider_module,
        } => {
            imported.insert((provider_module.clone(), identity.clone()));
        }
        CoreInstanceEvidence::Local {
            evidence_arguments, ..
        } => collect_call_evidence(evidence_arguments, imported),
        CoreInstanceEvidence::Standard { .. } | CoreInstanceEvidence::Parameter { .. } => {}
    }
}
