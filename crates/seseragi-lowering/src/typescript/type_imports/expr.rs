use crate::{CoreComprehensionClause, CoreExpr, CorePattern, CoreStatement, CoreTemplatePart};
use seseragi_semantics::ExternalTypeBinding;

use super::super::TypeScriptTypeImport;
use super::collect_type_imports;

pub(super) fn collect_expr_type_imports(
    expr: &CoreExpr,
    bindings: &[ExternalTypeBinding],
    requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptTypeImport>,
) {
    match expr {
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. } => {}
        CoreExpr::Template { parts, .. } => {
            for part in parts {
                if let CoreTemplatePart::Interpolation { value, .. } = part {
                    collect_expr_type_imports(value, bindings, requirements, imports);
                }
            }
        }
        CoreExpr::Variable { type_ref, .. } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
        }
        CoreExpr::Call {
            arguments,
            type_ref,
            ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_exprs(arguments, bindings, requirements, imports);
        }
        CoreExpr::Lambda {
            parameter,
            body,
            type_ref,
            ..
        } => {
            collect_type_imports(&parameter.type_ref, bindings, requirements, imports);
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(body, bindings, requirements, imports);
        }
        CoreExpr::Tuple {
            elements, type_ref, ..
        }
        | CoreExpr::Array {
            elements, type_ref, ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_exprs(elements, bindings, requirements, imports);
        }
        CoreExpr::FieldAccess {
            receiver, type_ref, ..
        }
        | CoreExpr::OptionalFieldAccess {
            receiver, type_ref, ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(receiver, bindings, requirements, imports);
        }
        CoreExpr::Record {
            items, type_ref, ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            for item in items {
                collect_expr_type_imports(item.value(), bindings, requirements, imports);
            }
        }
        CoreExpr::List { elements, .. } => {
            // List literals are constructed through `fromArray`, whose result
            // type is inferred. A runtime `List` type import is only needed
            // when a source signature or another emitted annotation names it.
            collect_exprs(elements, bindings, requirements, imports);
        }
        CoreExpr::ArrayComprehension {
            element,
            clauses,
            type_ref,
            ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(element, bindings, requirements, imports);
            for clause in clauses {
                match clause {
                    CoreComprehensionClause::Generator {
                        pattern, source, ..
                    } => {
                        collect_pattern_type_imports(pattern, bindings, requirements, imports);
                        collect_expr_type_imports(source, bindings, requirements, imports);
                    }
                    CoreComprehensionClause::Guard { condition, .. } => {
                        collect_expr_type_imports(condition, bindings, requirements, imports);
                    }
                }
            }
        }
        CoreExpr::ListComprehension {
            element, clauses, ..
        } => {
            collect_expr_type_imports(element, bindings, requirements, imports);
            for clause in clauses {
                match clause {
                    CoreComprehensionClause::Generator {
                        pattern, source, ..
                    } => {
                        collect_pattern_type_imports(pattern, bindings, requirements, imports);
                        collect_expr_type_imports(source, bindings, requirements, imports);
                    }
                    CoreComprehensionClause::Guard { condition, .. } => {
                        collect_expr_type_imports(condition, bindings, requirements, imports);
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
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(left, bindings, requirements, imports);
            collect_expr_type_imports(right, bindings, requirements, imports);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(condition, bindings, requirements, imports);
            collect_expr_type_imports(then_branch, bindings, requirements, imports);
            collect_expr_type_imports(else_branch, bindings, requirements, imports);
        }
        CoreExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            ..
        } => {
            collect_type_imports(scrutinee_type, bindings, requirements, imports);
            collect_type_imports(type_ref, bindings, requirements, imports);
            collect_expr_type_imports(scrutinee, bindings, requirements, imports);
            for branch in branches {
                for binding in &branch.bindings {
                    collect_type_imports(&binding.type_ref, bindings, requirements, imports);
                }
                if let Some(guard) = &branch.guard {
                    collect_expr_type_imports(guard, bindings, requirements, imports);
                }
                collect_expr_type_imports(&branch.value, bindings, requirements, imports);
            }
        }
        CoreExpr::EffectOperation {
            arguments, success, ..
        }
        | CoreExpr::EffectInvoke {
            arguments, success, ..
        } => {
            collect_type_imports(success, bindings, requirements, imports);
            collect_exprs(arguments, bindings, requirements, imports);
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                let value = match statement {
                    CoreStatement::Effect { value } => value,
                    CoreStatement::PureLet {
                        type_ref, value, ..
                    }
                    | CoreStatement::Bind {
                        type_ref, value, ..
                    } => {
                        collect_type_imports(type_ref, bindings, requirements, imports);
                        value
                    }
                    CoreStatement::LocalFunction {
                        constraints,
                        parameters,
                        body,
                        ..
                    } => {
                        for constraint in constraints {
                            for argument in &constraint.arguments {
                                collect_type_imports(argument, bindings, requirements, imports);
                            }
                        }
                        for parameter in parameters {
                            collect_type_imports(
                                &parameter.type_ref,
                                bindings,
                                requirements,
                                imports,
                            );
                        }
                        body
                    }
                };
                collect_expr_type_imports(value, bindings, requirements, imports);
            }
            collect_expr_type_imports(result, bindings, requirements, imports);
        }
        CoreExpr::MonadDo {
            statements,
            result,
            type_ref,
            ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            for statement in statements {
                let value = match statement {
                    crate::CoreMonadDoStatement::Expression { value } => value,
                    crate::CoreMonadDoStatement::PureLet {
                        type_ref, value, ..
                    }
                    | crate::CoreMonadDoStatement::Bind {
                        type_ref, value, ..
                    } => {
                        collect_type_imports(type_ref, bindings, requirements, imports);
                        value
                    }
                };
                collect_expr_type_imports(value, bindings, requirements, imports);
            }
            collect_expr_type_imports(result, bindings, requirements, imports);
        }
    }
}

fn collect_pattern_type_imports(
    pattern: &CorePattern,
    bindings: &[ExternalTypeBinding],
    requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptTypeImport>,
) {
    match pattern {
        CorePattern::Integer { .. }
        | CorePattern::String { .. }
        | CorePattern::Boolean { .. }
        | CorePattern::Wildcard { .. }
        | CorePattern::Invalid { .. } => {}
        CorePattern::Binding { type_ref, .. } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
        }
        CorePattern::Constructor {
            argument, type_ref, ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            if let Some(argument) = argument {
                collect_pattern_type_imports(argument, bindings, requirements, imports);
            }
        }
        CorePattern::Tuple { elements, .. } => {
            for element in elements {
                collect_pattern_type_imports(element, bindings, requirements, imports);
            }
        }
        CorePattern::Record {
            fields, type_ref, ..
        } => {
            collect_type_imports(type_ref, bindings, requirements, imports);
            for field in fields {
                collect_pattern_type_imports(&field.pattern, bindings, requirements, imports);
            }
        }
    }
}

fn collect_exprs(
    expressions: &[CoreExpr],
    bindings: &[ExternalTypeBinding],
    requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptTypeImport>,
) {
    for expression in expressions {
        collect_expr_type_imports(expression, bindings, requirements, imports);
    }
}
