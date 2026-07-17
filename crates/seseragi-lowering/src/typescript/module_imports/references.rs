use crate::{
    CoreComprehensionClause, CoreDecisionBranch, CoreExpr, CoreModule, CorePattern, CoreStatement,
    CoreTemplatePart, CoreType,
};
use std::collections::BTreeSet;

pub(super) fn referenced_value_symbols(module: &CoreModule) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    for binding in &module.bindings {
        collect_expr_value_symbols(&binding.value, &mut values);
    }
    for function in &module.functions {
        collect_expr_value_symbols(&function.body, &mut values);
    }
    for instance in &module.instances {
        if let crate::CoreInstanceImplementation::UserDefined { methods } = &instance.implementation
        {
            for method in methods {
                collect_expr_value_symbols(&method.body, &mut values);
            }
        }
    }
    values
}

fn collect_expr_value_symbols(expr: &CoreExpr, values: &mut BTreeSet<String>) {
    match expr {
        CoreExpr::Variable { name, .. } => {
            values.insert(name.clone());
        }
        CoreExpr::Call {
            callee, arguments, ..
        } => {
            values.insert(callee.clone());
            for argument in arguments {
                collect_expr_value_symbols(argument, values);
            }
        }
        CoreExpr::Tuple { elements, .. }
        | CoreExpr::Array { elements, .. }
        | CoreExpr::List { elements, .. } => {
            for element in elements {
                collect_expr_value_symbols(element, values);
            }
        }
        CoreExpr::FieldAccess { receiver, .. } | CoreExpr::OptionalFieldAccess { receiver, .. } => {
            collect_expr_value_symbols(receiver, values)
        }
        CoreExpr::Record { items, .. } => {
            for item in items {
                collect_expr_value_symbols(item.value(), values);
            }
        }
        CoreExpr::Template { parts, .. } => {
            for part in parts {
                if let CoreTemplatePart::Interpolation { value, .. } = part {
                    collect_expr_value_symbols(value, values);
                }
            }
        }
        CoreExpr::ArrayComprehension {
            element, clauses, ..
        }
        | CoreExpr::ListComprehension {
            element, clauses, ..
        } => {
            collect_expr_value_symbols(element, values);
            for clause in clauses {
                let expression = match clause {
                    CoreComprehensionClause::Generator { source, .. } => source,
                    CoreComprehensionClause::Guard { condition, .. } => condition,
                };
                collect_expr_value_symbols(expression, values);
            }
        }
        CoreExpr::Binary { left, right, .. } => {
            collect_expr_value_symbols(left, values);
            collect_expr_value_symbols(right, values);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_value_symbols(condition, values);
            collect_expr_value_symbols(then_branch, values);
            collect_expr_value_symbols(else_branch, values);
        }
        CoreExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            collect_expr_value_symbols(scrutinee, values);
            for branch in branches {
                collect_branch_value_symbols(branch, values);
            }
        }
        CoreExpr::EffectOperation { arguments, .. } => {
            for argument in arguments {
                collect_expr_value_symbols(argument, values);
            }
        }
        CoreExpr::EffectInvoke {
            callee, arguments, ..
        } => {
            values.insert(callee.clone());
            for argument in arguments {
                collect_expr_value_symbols(argument, values);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    CoreStatement::Effect { value }
                    | CoreStatement::PureLet { value, .. }
                    | CoreStatement::Bind { value, .. } => {
                        collect_expr_value_symbols(value, values);
                    }
                }
            }
            collect_expr_value_symbols(result, values);
        }
        CoreExpr::MonadDo {
            statements, result, ..
        } => {
            for statement in statements {
                let value = match statement {
                    crate::CoreMonadDoStatement::Expression { value }
                    | crate::CoreMonadDoStatement::PureLet { value, .. }
                    | crate::CoreMonadDoStatement::Bind { value, .. } => value,
                };
                collect_expr_value_symbols(value, values);
            }
            collect_expr_value_symbols(result, values);
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
    }
}

fn collect_branch_value_symbols(branch: &CoreDecisionBranch, values: &mut BTreeSet<String>) {
    if let Some(guard) = &branch.guard {
        collect_expr_value_symbols(guard, values);
    }
    collect_expr_value_symbols(&branch.value, values);
}

pub(super) struct ReferencedTypes {
    pub(super) names: BTreeSet<String>,
    pub(super) external: BTreeSet<String>,
}

pub(super) fn referenced_types(module: &CoreModule) -> ReferencedTypes {
    let mut references = ReferencedTypes {
        names: BTreeSet::new(),
        external: BTreeSet::new(),
    };
    for adt in &module.adts {
        for variant in &adt.variants {
            if let Some(payload) = &variant.payload {
                collect_type_names(payload, &mut references);
            }
        }
    }
    for instance in &module.instances {
        for argument in &instance.arguments {
            collect_type_names(argument, &mut references);
        }
        for constraint in &instance.constraints {
            for argument in &constraint.arguments {
                collect_type_names(argument, &mut references);
            }
        }
        if let crate::CoreInstanceImplementation::UserDefined { methods } = &instance.implementation
        {
            for method in methods {
                for parameter in &method.parameters {
                    collect_type_names(&parameter.type_ref, &mut references);
                }
                collect_expr_type_names(&method.body, &mut references);
            }
        }
    }
    for binding in &module.bindings {
        collect_expr_type_names(&binding.value, &mut references);
    }
    for function in &module.functions {
        for parameter in &function.parameters {
            collect_type_names(&parameter.type_ref, &mut references);
        }
        collect_expr_type_names(&function.body, &mut references);
    }
    references
}

fn collect_expr_type_names(expr: &CoreExpr, references: &mut ReferencedTypes) {
    match expr {
        CoreExpr::Variable { type_ref, .. } | CoreExpr::Call { type_ref, .. } => {
            collect_type_names(type_ref, references);
        }
        CoreExpr::Template { parts, .. } => {
            for part in parts {
                if let CoreTemplatePart::Interpolation { value, .. } = part {
                    collect_expr_type_names(value, references);
                }
            }
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
            collect_type_names(type_ref, references);
            for element in elements {
                collect_expr_type_names(element, references);
            }
        }
        CoreExpr::FieldAccess {
            receiver, type_ref, ..
        }
        | CoreExpr::OptionalFieldAccess {
            receiver, type_ref, ..
        } => {
            collect_type_names(type_ref, references);
            collect_expr_type_names(receiver, references);
        }
        CoreExpr::Record {
            items, type_ref, ..
        } => {
            collect_type_names(type_ref, references);
            for item in items {
                collect_expr_type_names(item.value(), references);
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
            collect_type_names(type_ref, references);
            collect_expr_type_names(element, references);
            for clause in clauses {
                match clause {
                    CoreComprehensionClause::Generator {
                        pattern, source, ..
                    } => {
                        collect_pattern_type_names(pattern, references);
                        collect_expr_type_names(source, references);
                    }
                    CoreComprehensionClause::Guard { condition, .. } => {
                        collect_expr_type_names(condition, references);
                    }
                }
            }
        }
        CoreExpr::Binary {
            left,
            right,
            type_ref,
            ..
        } => {
            collect_type_names(type_ref, references);
            collect_expr_type_names(left, references);
            collect_expr_type_names(right, references);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            ..
        } => {
            collect_type_names(type_ref, references);
            collect_expr_type_names(condition, references);
            collect_expr_type_names(then_branch, references);
            collect_expr_type_names(else_branch, references);
        }
        CoreExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            ..
        } => {
            collect_type_names(scrutinee_type, references);
            collect_type_names(type_ref, references);
            collect_expr_type_names(scrutinee, references);
            for branch in branches {
                for binding in &branch.bindings {
                    collect_type_names(&binding.type_ref, references);
                }
                if let Some(guard) = &branch.guard {
                    collect_expr_type_names(guard, references);
                }
                collect_expr_type_names(&branch.value, references);
            }
        }
        CoreExpr::EffectOperation {
            requirements,
            failure,
            success,
            arguments,
            ..
        }
        | CoreExpr::EffectInvoke {
            requirements,
            failure,
            success,
            arguments,
            ..
        } => {
            collect_type_names(requirements, references);
            collect_type_names(failure, references);
            collect_type_names(success, references);
            for argument in arguments {
                collect_expr_type_names(argument, references);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    CoreStatement::Effect { value } => collect_expr_type_names(value, references),
                    CoreStatement::PureLet {
                        type_ref, value, ..
                    }
                    | CoreStatement::Bind {
                        type_ref, value, ..
                    } => {
                        collect_type_names(type_ref, references);
                        collect_expr_type_names(value, references);
                    }
                }
            }
            collect_expr_type_names(result, references);
        }
        CoreExpr::MonadDo {
            statements,
            result,
            type_ref,
            ..
        } => {
            collect_type_names(type_ref, references);
            for statement in statements {
                match statement {
                    crate::CoreMonadDoStatement::Expression { value } => {
                        collect_expr_type_names(value, references)
                    }
                    crate::CoreMonadDoStatement::PureLet {
                        type_ref, value, ..
                    }
                    | crate::CoreMonadDoStatement::Bind {
                        type_ref, value, ..
                    } => {
                        collect_type_names(type_ref, references);
                        collect_expr_type_names(value, references);
                    }
                }
            }
            collect_expr_type_names(result, references);
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
    }
}

fn collect_pattern_type_names(pattern: &CorePattern, references: &mut ReferencedTypes) {
    match pattern {
        CorePattern::Integer { type_ref, .. }
        | CorePattern::String { type_ref, .. }
        | CorePattern::Boolean { type_ref, .. }
        | CorePattern::Binding { type_ref, .. }
        | CorePattern::Wildcard { type_ref, .. } => collect_type_names(type_ref, references),
        CorePattern::Constructor {
            argument, type_ref, ..
        } => {
            collect_type_names(type_ref, references);
            if let Some(argument) = argument {
                collect_pattern_type_names(argument, references);
            }
        }
        CorePattern::Tuple {
            elements, type_ref, ..
        } => {
            collect_type_names(type_ref, references);
            for element in elements {
                collect_pattern_type_names(element, references);
            }
        }
        CorePattern::Invalid { .. } => {}
    }
}

fn collect_type_names(type_ref: &CoreType, references: &mut ReferencedTypes) {
    match type_ref {
        CoreType::Named { name, arguments } => {
            references.names.insert(name.clone());
            for argument in arguments {
                collect_type_names(argument, references);
            }
        }
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => {
            references.external.insert(canonical.clone());
            for argument in arguments {
                collect_type_names(argument, references);
            }
        }
        CoreType::Record { fields, .. } => {
            for field in fields {
                collect_type_names(&field.type_ref, references);
            }
        }
        CoreType::Tuple { elements } => {
            for element in elements {
                collect_type_names(element, references);
            }
        }
        CoreType::Function { parameter, result } => {
            collect_type_names(parameter, references);
            collect_type_names(result, references);
        }
        CoreType::Hole => {}
    }
}
