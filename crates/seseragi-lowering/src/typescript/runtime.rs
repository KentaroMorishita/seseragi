use crate::sum_ops::runtime_sum_constructor;
use crate::{
    effect_ops::runtime_effect_operation, int_ops::runtime_int_operation, CoreExpr, CoreStatement,
    CoreType,
};

use super::{push_import_unique, push_unique, TypeScriptImport};

pub(super) fn collect_expr_runtime_requirements(expr: &CoreExpr, requirements: &mut Vec<String>) {
    match expr {
        CoreExpr::Unit { .. } => push_unique(requirements, "core.unit"),
        CoreExpr::Int64 { .. } => push_unique(requirements, "core.int64"),
        CoreExpr::String { .. } => push_unique(requirements, "core.string"),
        CoreExpr::Boolean { .. } => push_unique(requirements, "core.bool"),
        CoreExpr::Variable { name, type_ref, .. } => {
            if matches!(type_ref, CoreType::Function { .. }) {
                if let Some(operation) = runtime_int_operation(name) {
                    push_unique(requirements, operation.runtime_feature);
                }
            }
            if let Some(constructor) = runtime_sum_constructor(name) {
                push_unique(requirements, constructor.runtime_feature);
            }
            collect_type_runtime_requirement(type_ref, requirements);
        }
        CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            ..
        } => {
            if let Some(constructor) = runtime_sum_constructor(callee) {
                push_unique(requirements, constructor.runtime_feature);
            }
            collect_type_runtime_requirement(type_ref, requirements);
            for argument in arguments {
                collect_expr_runtime_requirements(argument, requirements);
            }
        }
        CoreExpr::Tuple {
            elements, type_ref, ..
        }
        | CoreExpr::Array {
            elements, type_ref, ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            for element in elements {
                collect_expr_runtime_requirements(element, requirements);
            }
        }
        CoreExpr::Binary {
            operator,
            left,
            right,
            type_ref,
            ..
        } => {
            collect_type_runtime_requirement(type_ref, requirements);
            if is_int_type(type_ref) {
                if let Some(operation) = runtime_int_operation(operator) {
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
        CoreExpr::Variable { name, type_ref, .. } => {
            if matches!(type_ref, CoreType::Function { .. }) {
                if let Some(operation) = runtime_int_operation(name) {
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
        }
        CoreExpr::Call {
            callee, arguments, ..
        } => {
            if let Some(constructor) = runtime_sum_constructor(callee) {
                push_sum_constructor_import(imports, constructor);
            }
            for argument in arguments {
                collect_expr_runtime_imports(argument, imports);
            }
        }
        CoreExpr::Tuple { elements, .. } | CoreExpr::Array { elements, .. } => {
            for element in elements {
                collect_expr_runtime_imports(element, imports);
            }
        }
        CoreExpr::Binary {
            operator,
            left,
            right,
            type_ref,
            ..
        } => {
            if is_int_type(type_ref) {
                if let Some(operation) = runtime_int_operation(operator) {
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
                "Maybe" => push_unique(requirements, "core.maybe"),
                "Either" => push_unique(requirements, "core.either"),
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
