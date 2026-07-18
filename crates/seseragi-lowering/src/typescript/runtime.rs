use crate::collection_ops::{runtime_collection_operation, runtime_iterable_operation};
use crate::iterator_ops::runtime_iterator_comprehension_operation;
use crate::iterator_ops::runtime_iterator_operation;
use crate::list_ops::runtime_list_literal_operation;
use crate::range_ops::runtime_range_operation;
use crate::sum_ops::runtime_sum_constructor;
use crate::web_html_ops::runtime_web_html_operation;
use crate::{
    effect_ops::runtime_effect_operation, int_ops::runtime_int_operation_with_evidence,
    prelude_ops::runtime_prelude_dictionary_for_identity,
    show_ops::runtime_show_dictionary_for_identity, CoreCallEvidence, CoreComprehensionClause,
    CoreExpr, CoreInstanceEvidence, CoreStatement, CoreTemplatePart, CoreType,
};

use super::{push_import_unique, push_unique, TypeScriptImport};

pub(super) fn collect_expr_runtime_requirements(expr: &CoreExpr, requirements: &mut Vec<String>) {
    match expr {
        CoreExpr::Unit { .. } => push_unique(requirements, "core.unit"),
        CoreExpr::Int64 { .. } => push_unique(requirements, "core.int64"),
        CoreExpr::String { .. } => push_unique(requirements, "core.string"),
        CoreExpr::Template { parts, .. } => {
            push_unique(requirements, "core.string");
            for part in parts {
                if let CoreTemplatePart::Interpolation {
                    value, evidence, ..
                } = part
                {
                    if let Some(evidence) = evidence {
                        collect_evidence_runtime_requirements(
                            std::slice::from_ref(evidence),
                            requirements,
                        );
                    }
                    collect_expr_runtime_requirements(value, requirements);
                }
            }
        }
        CoreExpr::Boolean { .. } => push_unique(requirements, "core.bool"),
        CoreExpr::Variable {
            name,
            evidence,
            type_ref,
            ..
        } => {
            collect_evidence_runtime_requirements(evidence, requirements);
            if matches!(type_ref, CoreType::Function { .. }) {
                if let Some(operation) = runtime_int_operation_with_evidence(name, evidence) {
                    push_unique(requirements, operation.runtime_feature);
                }
            }
            if let Some(constructor) = runtime_sum_constructor(name) {
                push_unique(requirements, constructor.runtime_feature);
            }
            if let Some(operation) = runtime_web_html_operation(name) {
                push_unique(requirements, operation.runtime_feature);
            }
            collect_type_runtime_requirement(type_ref, requirements);
        }
        CoreExpr::Call {
            callee,
            arguments,
            evidence,
            type_ref,
            ..
        } => {
            collect_evidence_runtime_requirements(evidence, requirements);
            if let Some(operation) = runtime_collection_operation(callee, evidence) {
                push_unique(requirements, operation.runtime_feature);
            } else if let Some(operation) = runtime_iterator_operation(callee) {
                push_unique(requirements, operation.runtime_feature);
            } else if let Some(constructor) = runtime_sum_constructor(callee) {
                push_unique(requirements, constructor.runtime_feature);
            } else if let Some(operation) = runtime_web_html_operation(callee) {
                push_unique(requirements, operation.runtime_feature);
            }
            collect_type_runtime_requirement(type_ref, requirements);
            for argument in arguments {
                collect_expr_runtime_requirements(argument, requirements);
            }
        }
        CoreExpr::Lambda {
            parameter,
            body,
            type_ref,
            ..
        } => {
            collect_type_runtime_requirement(&parameter.type_ref, requirements);
            collect_type_runtime_requirement(type_ref, requirements);
            collect_expr_runtime_requirements(body, requirements);
        }
        CoreExpr::Tuple {
            elements, type_ref, ..
        }
        | CoreExpr::Array {
            elements, type_ref, ..
        }
        | CoreExpr::List {
            elements, type_ref, ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            if matches!(expr, CoreExpr::List { .. }) {
                push_unique(
                    requirements,
                    runtime_list_literal_operation().runtime_feature,
                );
            }
            for element in elements {
                collect_expr_runtime_requirements(element, requirements);
            }
        }
        CoreExpr::FieldAccess {
            receiver, type_ref, ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            collect_expr_runtime_requirements(receiver, requirements);
        }
        CoreExpr::OptionalFieldAccess {
            receiver, type_ref, ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            for name in ["std/prelude::Nothing", "std/prelude::Just"] {
                let constructor = runtime_sum_constructor(name)
                    .expect("standard Maybe constructor must have a runtime feature");
                push_unique(requirements, constructor.runtime_feature);
            }
            collect_expr_runtime_requirements(receiver, requirements);
        }
        CoreExpr::Record {
            items, type_ref, ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            for item in items {
                collect_expr_runtime_requirements(item.value(), requirements);
            }
        }
        CoreExpr::ArrayComprehension {
            element,
            clauses,
            type_ref,
            ..
        }
        | CoreExpr::ListComprehension {
            element,
            clauses,
            type_ref,
            ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            if matches!(expr, CoreExpr::ListComprehension { .. }) {
                push_unique(
                    requirements,
                    runtime_list_literal_operation().runtime_feature,
                );
            }
            collect_expr_runtime_requirements(element, requirements);
            for (index, clause) in clauses.iter().enumerate() {
                match clause {
                    CoreComprehensionClause::Generator {
                        source, evidence, ..
                    } => {
                        collect_evidence_runtime_requirements(
                            std::slice::from_ref(evidence),
                            requirements,
                        );
                        let flatten = clauses[index + 1..].iter().any(|clause| {
                            matches!(clause, CoreComprehensionClause::Generator { .. })
                        });
                        let feature = runtime_iterable_operation(evidence, flatten)
                            .map(|operation| operation.runtime_feature)
                            .unwrap_or_else(|| {
                                runtime_iterator_comprehension_operation(flatten).runtime_feature
                            });
                        push_unique(requirements, feature);
                        collect_expr_runtime_requirements(source, requirements);
                    }
                    CoreComprehensionClause::Guard { condition, .. } => {
                        collect_expr_runtime_requirements(condition, requirements);
                    }
                }
            }
        }
        CoreExpr::Binary {
            operator,
            left,
            right,
            evidence,
            type_ref,
            ..
        } => {
            collect_evidence_runtime_requirements(evidence, requirements);
            collect_type_runtime_requirement(type_ref, requirements);
            if let Some(operation) = runtime_range_operation(operator) {
                push_unique(requirements, operation.runtime_feature);
            } else if is_int_type(type_ref) {
                if let Some(operation) = runtime_int_operation_with_evidence(operator, evidence) {
                    push_unique(requirements, operation.runtime_feature);
                }
            }
            collect_expr_runtime_requirements(left, requirements);
            collect_expr_runtime_requirements(right, requirements);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            collect_expr_runtime_requirements(condition, requirements);
            collect_expr_runtime_requirements(then_branch, requirements);
            collect_expr_runtime_requirements(else_branch, requirements);
        }
        CoreExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            ..
        } => {
            collect_type_runtime_requirement(scrutinee_type, requirements);
            collect_type_runtime_requirement(type_ref, requirements);
            collect_expr_runtime_requirements(scrutinee, requirements);
            for branch in branches {
                if branch
                    .tests
                    .iter()
                    .any(|test| matches!(test, crate::CoreDecisionTest::Constructor { .. }))
                {
                    push_unique(requirements, "core.adt");
                }
                for binding in &branch.bindings {
                    collect_type_runtime_requirement(&binding.type_ref, requirements);
                }
                if let Some(guard) = &branch.guard {
                    collect_expr_runtime_requirements(guard, requirements);
                }
                collect_expr_runtime_requirements(&branch.value, requirements);
            }
        }
        CoreExpr::EffectOperation {
            operation,
            arguments,
            requirements: environment,
            failure,
            success,
            ..
        } => {
            if let Some(operation) = runtime_effect_operation(operation) {
                push_unique(requirements, operation.runtime_feature);
            }
            collect_type_runtime_requirement(environment, requirements);
            collect_type_runtime_requirement(failure, requirements);
            collect_type_runtime_requirement(success, requirements);
            for argument in arguments {
                collect_expr_runtime_requirements(argument, requirements);
            }
        }
        CoreExpr::EffectInvoke {
            arguments,
            requirements: environment,
            failure,
            success,
            ..
        } => {
            collect_type_runtime_requirement(environment, requirements);
            collect_type_runtime_requirement(failure, requirements);
            collect_type_runtime_requirement(success, requirements);
            for argument in arguments {
                collect_expr_runtime_requirements(argument, requirements);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            if statements.iter().any(statement_is_monadic) {
                push_unique(requirements, "effect.core.flatMap");
            }
            for statement in statements {
                collect_statement_runtime_requirements(statement, requirements);
            }
            collect_expr_runtime_requirements(result, requirements);
        }
        CoreExpr::MonadDo {
            statements,
            result,
            evidence,
            type_ref,
            ..
        } => {
            collect_evidence_runtime_requirements(std::slice::from_ref(evidence), requirements);
            collect_type_runtime_requirement(type_ref, requirements);
            for statement in statements {
                match statement {
                    crate::CoreMonadDoStatement::Expression { value } => {
                        collect_expr_runtime_requirements(value, requirements)
                    }
                    crate::CoreMonadDoStatement::PureLet {
                        type_ref, value, ..
                    }
                    | crate::CoreMonadDoStatement::Bind {
                        type_ref, value, ..
                    } => {
                        collect_type_runtime_requirement(type_ref, requirements);
                        collect_expr_runtime_requirements(value, requirements);
                    }
                }
            }
            collect_expr_runtime_requirements(result, requirements);
        }
    }
}

fn collect_statement_runtime_requirements(
    statement: &CoreStatement,
    requirements: &mut Vec<String>,
) {
    match statement {
        CoreStatement::Effect { value }
        | CoreStatement::PureLet { value, .. }
        | CoreStatement::Bind { value, .. } => {
            collect_expr_runtime_requirements(value, requirements);
        }
    }
}

pub(super) fn collect_expr_runtime_imports(expr: &CoreExpr, imports: &mut Vec<TypeScriptImport>) {
    match expr {
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => {
            if let Some(operation) = runtime_effect_operation(operation) {
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            }
            for argument in arguments {
                collect_expr_runtime_imports(argument, imports);
            }
        }
        CoreExpr::EffectInvoke { arguments, .. } => {
            for argument in arguments {
                collect_expr_runtime_imports(argument, imports);
            }
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
        CoreExpr::Template { parts, .. } => {
            for part in parts {
                if let CoreTemplatePart::Interpolation {
                    value, evidence, ..
                } = part
                {
                    if let Some(evidence) = evidence {
                        collect_evidence_runtime_imports(std::slice::from_ref(evidence), imports);
                    }
                    collect_expr_runtime_imports(value, imports);
                }
            }
        }
        CoreExpr::Variable {
            name,
            evidence,
            type_ref,
            ..
        } => {
            collect_evidence_runtime_imports(evidence, imports);
            if matches!(type_ref, CoreType::Function { .. }) {
                if let Some(operation) = runtime_int_operation_with_evidence(name, evidence) {
                    push_import_unique(
                        imports,
                        TypeScriptImport {
                            feature: operation.runtime_feature.to_owned(),
                            local: operation.local_name.to_owned(),
                        },
                    );
                }
            }
            if let Some(constructor) = runtime_sum_constructor(name) {
                push_sum_constructor_import(imports, constructor);
            }
            if let Some(operation) = runtime_web_html_operation(name) {
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            }
        }
        CoreExpr::Call {
            callee,
            arguments,
            evidence,
            ..
        } => {
            collect_evidence_runtime_imports(evidence, imports);
            if let Some(operation) = runtime_collection_operation(callee, evidence) {
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            } else if let Some(operation) = runtime_iterator_operation(callee) {
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            } else if let Some(constructor) = runtime_sum_constructor(callee) {
                push_sum_constructor_import(imports, constructor);
            } else if let Some(operation) = runtime_web_html_operation(callee) {
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            }
            for argument in arguments {
                collect_expr_runtime_imports(argument, imports);
            }
        }
        CoreExpr::Lambda { body, .. } => collect_expr_runtime_imports(body, imports),
        CoreExpr::Tuple { elements, .. } | CoreExpr::Array { elements, .. } => {
            for element in elements {
                collect_expr_runtime_imports(element, imports);
            }
        }
        CoreExpr::FieldAccess { receiver, .. } => {
            collect_expr_runtime_imports(receiver, imports);
        }
        CoreExpr::OptionalFieldAccess { receiver, .. } => {
            for name in ["std/prelude::Nothing", "std/prelude::Just"] {
                let constructor = runtime_sum_constructor(name)
                    .expect("standard Maybe constructor must have a runtime import");
                push_sum_constructor_import(imports, constructor);
            }
            collect_expr_runtime_imports(receiver, imports);
        }
        CoreExpr::Record { items, .. } => {
            for item in items {
                collect_expr_runtime_imports(item.value(), imports);
            }
        }
        CoreExpr::List { elements, .. } => {
            let operation = runtime_list_literal_operation();
            push_import_unique(
                imports,
                TypeScriptImport {
                    feature: operation.runtime_feature.to_owned(),
                    local: operation.local_name.to_owned(),
                },
            );
            for element in elements {
                collect_expr_runtime_imports(element, imports);
            }
        }
        CoreExpr::ArrayComprehension {
            element, clauses, ..
        }
        | CoreExpr::ListComprehension {
            element, clauses, ..
        } => {
            if matches!(expr, CoreExpr::ListComprehension { .. }) {
                let operation = runtime_list_literal_operation();
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            }
            collect_expr_runtime_imports(element, imports);
            for (index, clause) in clauses.iter().enumerate() {
                match clause {
                    CoreComprehensionClause::Generator {
                        source, evidence, ..
                    } => {
                        collect_evidence_runtime_imports(std::slice::from_ref(evidence), imports);
                        let flatten = clauses[index + 1..].iter().any(|clause| {
                            matches!(clause, CoreComprehensionClause::Generator { .. })
                        });
                        let (feature, local) = runtime_iterable_operation(evidence, flatten)
                            .map(|operation| (operation.runtime_feature, operation.local_name))
                            .unwrap_or_else(|| {
                                let operation = runtime_iterator_comprehension_operation(flatten);
                                (operation.runtime_feature, operation.local_name)
                            });
                        push_import_unique(
                            imports,
                            TypeScriptImport {
                                feature: feature.to_owned(),
                                local: local.to_owned(),
                            },
                        );
                        collect_expr_runtime_imports(source, imports);
                    }
                    CoreComprehensionClause::Guard { condition, .. } => {
                        collect_expr_runtime_imports(condition, imports);
                    }
                }
            }
        }
        CoreExpr::Binary {
            operator,
            left,
            right,
            evidence,
            type_ref,
            ..
        } => {
            collect_evidence_runtime_imports(evidence, imports);
            if let Some(operation) = runtime_range_operation(operator) {
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            } else if is_int_type(type_ref) {
                if let Some(operation) = runtime_int_operation_with_evidence(operator, evidence) {
                    push_import_unique(
                        imports,
                        TypeScriptImport {
                            feature: operation.runtime_feature.to_owned(),
                            local: operation.local_name.to_owned(),
                        },
                    );
                }
            }
            collect_expr_runtime_imports(left, imports);
            collect_expr_runtime_imports(right, imports);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_runtime_imports(condition, imports);
            collect_expr_runtime_imports(then_branch, imports);
            collect_expr_runtime_imports(else_branch, imports);
        }
        CoreExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            collect_expr_runtime_imports(scrutinee, imports);
            for branch in branches {
                if let Some(guard) = &branch.guard {
                    collect_expr_runtime_imports(guard, imports);
                }
                collect_expr_runtime_imports(&branch.value, imports);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            if statements.iter().any(statement_is_monadic) {
                let operation = runtime_effect_operation("effect.flatMap")
                    .expect("effect.flatMap runtime operation must be registered");
                push_import_unique(
                    imports,
                    TypeScriptImport {
                        feature: operation.runtime_feature.to_owned(),
                        local: operation.local_name.to_owned(),
                    },
                );
            }
            for statement in statements {
                collect_statement_runtime_imports(statement, imports);
            }
            collect_expr_runtime_imports(result, imports);
        }
        CoreExpr::MonadDo {
            statements,
            result,
            evidence,
            ..
        } => {
            collect_evidence_runtime_imports(std::slice::from_ref(evidence), imports);
            for statement in statements {
                let value = match statement {
                    crate::CoreMonadDoStatement::Expression { value }
                    | crate::CoreMonadDoStatement::PureLet { value, .. }
                    | crate::CoreMonadDoStatement::Bind { value, .. } => value,
                };
                collect_expr_runtime_imports(value, imports);
            }
            collect_expr_runtime_imports(result, imports);
        }
    }
}

fn collect_evidence_runtime_requirements(
    evidence: &[CoreCallEvidence],
    requirements: &mut Vec<String>,
) {
    for selected in evidence {
        match &selected.evidence {
            CoreInstanceEvidence::Local {
                evidence_arguments, ..
            }
            | CoreInstanceEvidence::Imported {
                evidence_arguments, ..
            } => collect_evidence_runtime_requirements(evidence_arguments, requirements),
            CoreInstanceEvidence::Standard { identity } => {
                if let Some(dictionary) = runtime_show_dictionary_for_identity(identity) {
                    push_unique(requirements, dictionary.runtime_feature);
                } else if let Some(dictionary) = runtime_prelude_dictionary_for_identity(identity) {
                    push_unique(requirements, dictionary.runtime_feature);
                }
            }
            CoreInstanceEvidence::Parameter { .. } => {}
        }
    }
}

fn collect_evidence_runtime_imports(
    evidence: &[CoreCallEvidence],
    imports: &mut Vec<TypeScriptImport>,
) {
    for selected in evidence {
        match &selected.evidence {
            CoreInstanceEvidence::Local {
                evidence_arguments, ..
            }
            | CoreInstanceEvidence::Imported {
                evidence_arguments, ..
            } => collect_evidence_runtime_imports(evidence_arguments, imports),
            CoreInstanceEvidence::Standard { identity } => {
                if let Some(dictionary) = runtime_show_dictionary_for_identity(identity) {
                    push_import_unique(
                        imports,
                        TypeScriptImport {
                            feature: dictionary.runtime_feature.to_owned(),
                            local: dictionary.local_name.to_owned(),
                        },
                    );
                } else if let Some(dictionary) = runtime_prelude_dictionary_for_identity(identity) {
                    push_import_unique(
                        imports,
                        TypeScriptImport {
                            feature: dictionary.runtime_feature.to_owned(),
                            local: dictionary.local_name.to_owned(),
                        },
                    );
                }
            }
            CoreInstanceEvidence::Parameter { .. } => {}
        }
    }
}

fn push_sum_constructor_import(
    imports: &mut Vec<TypeScriptImport>,
    constructor: crate::sum_ops::RuntimeSumConstructor,
) {
    push_import_unique(
        imports,
        TypeScriptImport {
            feature: constructor.runtime_feature.to_owned(),
            local: constructor.local_name.to_owned(),
        },
    );
}

fn is_int_type(type_ref: &CoreType) -> bool {
    matches!(type_ref, CoreType::Named { name, arguments } if name == "Int" && arguments.is_empty())
}

fn collect_statement_runtime_imports(
    statement: &CoreStatement,
    imports: &mut Vec<TypeScriptImport>,
) {
    match statement {
        CoreStatement::Effect { value }
        | CoreStatement::PureLet { value, .. }
        | CoreStatement::Bind { value, .. } => {
            collect_expr_runtime_imports(value, imports);
        }
    }
}

fn statement_is_monadic(statement: &CoreStatement) -> bool {
    matches!(
        statement,
        CoreStatement::Effect { .. } | CoreStatement::Bind { .. }
    )
}

pub(super) fn collect_type_runtime_requirement(
    type_ref: &CoreType,
    requirements: &mut Vec<String>,
) {
    match type_ref {
        CoreType::Named { name, arguments } => {
            match name.as_str() {
                "Int" => push_unique(requirements, "core.int64"),
                "String" => push_unique(requirements, "core.string"),
                "Bool" => push_unique(requirements, "core.bool"),
                "Unit" => push_unique(requirements, "core.unit"),
                "Effect" => push_unique(requirements, "effect.core.type"),
                "Maybe" => push_unique(requirements, "core.maybe"),
                "Either" => push_unique(requirements, "core.either"),
                "Range" => push_unique(requirements, "core.range"),
                "Iterator" => push_unique(requirements, "core.iterator"),
                "List" => push_unique(requirements, "core.list"),
                _ => {}
            }
            for argument in arguments {
                collect_type_runtime_requirement(argument, requirements);
            }
        }
        CoreType::ExternalNamed { arguments, .. } => {
            for argument in arguments {
                collect_type_runtime_requirement(argument, requirements);
            }
        }
        CoreType::Record { fields, .. } => {
            for field in fields {
                collect_type_runtime_requirement(&field.type_ref, requirements);
            }
        }
        CoreType::Tuple { elements } => {
            for element in elements {
                collect_type_runtime_requirement(element, requirements);
            }
        }
        CoreType::Function { parameter, result } => {
            collect_type_runtime_requirement(parameter, requirements);
            collect_type_runtime_requirement(result, requirements);
        }
        CoreType::Hole => {}
    }
}

#[cfg(test)]
mod tests {
    use super::collect_type_runtime_requirement;
    use crate::CoreType;

    #[test]
    fn collects_standard_sum_and_nested_payload_requirements() {
        let type_ref = CoreType::Named {
            name: "Either".to_owned(),
            arguments: vec![
                CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                },
                CoreType::Named {
                    name: "Maybe".to_owned(),
                    arguments: vec![CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    }],
                },
            ],
        };
        let mut requirements = Vec::new();

        collect_type_runtime_requirement(&type_ref, &mut requirements);

        assert_eq!(
            requirements,
            vec!["core.either", "core.string", "core.maybe", "core.int64"]
        );
    }
}
