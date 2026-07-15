use crate::{
    CoreCallEvidence, CoreComprehensionClause, CoreDecisionBranch, CoreExpr, CoreInstanceEvidence,
    CoreInstanceImplementation, CoreModule, CoreStatement, TypeScriptLoweringError,
    TypeScriptOutputPlan, TypeScriptSourceImport, TypeScriptSourceImportBinding,
};
use std::collections::{BTreeMap, BTreeSet};

use super::super::names::safe_identifier;
use super::names::fresh_name;
use super::push_binding;
use super::types::inferred_import_origin;

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

pub(super) fn lower_imported_instance_imports(
    module: &CoreModule,
    imported: &BTreeSet<(String, String)>,
    plan: &TypeScriptOutputPlan,
    imports: &mut Vec<TypeScriptSourceImport>,
    used_values: &mut BTreeSet<String>,
) -> Result<BTreeMap<(String, String), String>, TypeScriptLoweringError> {
    let mut names = BTreeMap::new();
    for (provider_module, identity) in imported {
        let specifier = plan.specifier_for(provider_module).ok_or_else(|| {
            TypeScriptLoweringError::MissingInstanceOutputSpecifier {
                module: provider_module.clone(),
                identity: identity.clone(),
            }
        })?;
        let dictionary_export = plan
            .instance_export_for(provider_module, identity)
            .ok_or_else(|| TypeScriptLoweringError::MissingInstanceOutput {
                module: provider_module.clone(),
                identity: identity.clone(),
            })?;
        let local = fresh_name(&safe_identifier(dictionary_export), used_values);
        used_values.insert(local.clone());
        names.insert((provider_module.clone(), identity.clone()), local.clone());

        let index = imports
            .iter()
            .position(|group| group.module == *provider_module)
            .unwrap_or_else(|| {
                imports.push(TypeScriptSourceImport {
                    module: provider_module.clone(),
                    specifier: specifier.to_owned(),
                    runtime_edge: false,
                    bindings: Vec::new(),
                    origin: inferred_import_origin(module),
                });
                imports.len() - 1
            });
        let origin = imports[index].origin.clone();
        push_binding(
            &mut imports[index],
            TypeScriptSourceImportBinding {
                imported: dictionary_export.to_owned(),
                local,
                source_local: identity.clone(),
                canonical: identity.clone(),
                type_only: false,
                origin,
            },
        );
    }
    Ok(names)
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
        CoreExpr::ArrayComprehension {
            element, clauses, ..
        } => {
            collect_expr(element, imported);
            for clause in clauses {
                match clause {
                    CoreComprehensionClause::Generator {
                        source, evidence, ..
                    } => {
                        collect_call_evidence(std::slice::from_ref(evidence), imported);
                        collect_expr(source, imported);
                    }
                    CoreComprehensionClause::Guard { condition, .. } => {
                        collect_expr(condition, imported);
                    }
                }
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
