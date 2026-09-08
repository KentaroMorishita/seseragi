use crate::typescript::types::TypeScriptTypeContext;
use crate::{
    CoreDecisionBinding, CoreDecisionBranch, CoreDecisionProjection, CoreDecisionTest, CoreExpr,
    CorePattern, CoreType,
};
use std::collections::BTreeMap;

use super::expr::lower_core_expr_to_typescript;
use super::names::{local_name, safe_identifier};
use super::types::type_ref_from_core_type;
use super::{
    TypeScriptDecisionBinding, TypeScriptDecisionBranch, TypeScriptDecisionProjection,
    TypeScriptDecisionTest, TypeScriptExpr,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TypeScriptPatternDecision {
    pub(super) private_representation: bool,
    pub(super) scrutinee_type: super::TypeScriptType,
    pub(super) tests: Vec<TypeScriptDecisionTest>,
    pub(super) bindings: Vec<TypeScriptDecisionBinding>,
}

pub(super) fn lower_core_pattern_decision(
    pattern: CorePattern,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptPatternDecision {
    let scrutinee_type = type_ref_from_core_type(
        &core_pattern_type(&pattern),
        &imported_types.with_private_representation(),
    );
    let private_representation =
        scrutinee_type != type_ref_from_core_type(&core_pattern_type(&pattern), imported_types);
    let mut tests = Vec::new();
    let mut bindings = Vec::new();
    lower_pattern(
        pattern,
        &mut Vec::new(),
        &mut tests,
        &mut bindings,
        imported_types,
    );
    TypeScriptPatternDecision {
        private_representation,
        scrutinee_type,
        tests,
        bindings,
    }
}

pub(super) fn lower_core_decision(
    scrutinee: CoreExpr,
    scrutinee_type: CoreType,
    branches: Vec<CoreDecisionBranch>,
    type_ref: CoreType,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptExpr {
    let erases_newtype = branches.iter().flat_map(|branch| &branch.tests).any(|test| matches!(test, CoreDecisionTest::Constructor { constructor, .. } if imported_types.is_erased_newtype(constructor)));
    let public_type = type_ref_from_core_type(&scrutinee_type, imported_types);
    let private_type = type_ref_from_core_type(
        &scrutinee_type,
        &imported_types.with_private_representation(),
    );
    let scrutinee = lower_core_expr_to_typescript(scrutinee, imported_values, imported_types);
    let scrutinee = if public_type != private_type {
        super::types::assert_private_representation(scrutinee, private_type.clone())
    } else {
        scrutinee
    };
    let mut branches = branches
        .into_iter()
        .map(|branch| lower_branch(branch, imported_values, imported_types))
        .collect::<Vec<_>>();
    if erases_newtype && branches.len() == 1 {
        let branch = &mut branches[0];
        if let TypeScriptExpr::Identifier { name } = &scrutinee {
            if branch.tests.is_empty()
                && branch.guard.is_none()
                && branch
                    .bindings
                    .iter()
                    .all(|binding| binding.path.is_empty())
                && !contains_scope(&branch.value)
            {
                let renames = branch
                    .bindings
                    .iter()
                    .map(|binding| (binding.name.clone(), name.clone()))
                    .collect();
                substitute_bindings(&mut branch.value, &renames);
                return TypeScriptExpr::CheckedResult {
                    value: Box::new(branch.value.clone()),
                    type_ref: type_ref_from_core_type(&type_ref, imported_types),
                };
            }
        }
    }
    TypeScriptExpr::Decision {
        scrutinee: Box::new(scrutinee),
        scrutinee_type: private_type,
        branches,
        type_ref: type_ref_from_core_type(&type_ref, imported_types),
    }
}

// Called only for scope-free expressions after the scrutinee is an immutable
// identifier. Runtime references are a separate namespace and never substituted.
fn substitute_bindings(expr: &mut TypeScriptExpr, renames: &BTreeMap<String, String>) {
    match expr {
        TypeScriptExpr::Identifier { name } => {
            if let Some(replacement) = renames.get(name) {
                *name = replacement.clone();
            }
        }
        TypeScriptExpr::Call {
            callee, arguments, ..
        }
        | TypeScriptExpr::TypeApplicationCall {
            callee, arguments, ..
        } => {
            if let Some(replacement) = renames.get(callee) {
                *callee = replacement.clone();
            }
            for argument in arguments {
                substitute_bindings(argument, renames);
            }
        }
        TypeScriptExpr::RuntimeCall { arguments, .. }
        | TypeScriptExpr::ForeignTaskCall { arguments, .. }
        | TypeScriptExpr::Tuple {
            elements: arguments,
        }
        | TypeScriptExpr::Array {
            elements: arguments,
            ..
        } => {
            for argument in arguments {
                substitute_bindings(argument, renames);
            }
        }
        TypeScriptExpr::FieldAccess {
            receiver: value, ..
        }
        | TypeScriptExpr::OptionalFieldAccess {
            receiver: value, ..
        }
        | TypeScriptExpr::CheckedResult { value, .. }
        | TypeScriptExpr::Await { value }
        | TypeScriptExpr::Unary { operand: value, .. } => substitute_bindings(value, renames),
        TypeScriptExpr::Binary { left, right, .. } => {
            substitute_bindings(left, renames);
            substitute_bindings(right, renames);
        }
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            substitute_bindings(condition, renames);
            substitute_bindings(then_branch, renames);
            substitute_bindings(else_branch, renames);
        }
        TypeScriptExpr::DictionaryCall {
            dictionary,
            arguments,
            ..
        } => {
            substitute_bindings(dictionary, renames);
            for argument in arguments {
                substitute_bindings(argument, renames);
            }
        }
        TypeScriptExpr::Record { items, .. } => {
            for item in items {
                match item {
                    super::TypeScriptRecordValueItem::Field { value, .. }
                    | super::TypeScriptRecordValueItem::Spread { value } => {
                        substitute_bindings(value, renames)
                    }
                }
            }
        }
        TypeScriptExpr::Lambda { .. }
        | TypeScriptExpr::Sequence { .. }
        | TypeScriptExpr::MonadDo { .. }
        | TypeScriptExpr::Decision { .. } => unreachable!("scope-free substitution"),
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::Number { .. }
        | TypeScriptExpr::String { .. }
        | TypeScriptExpr::Boolean { .. }
        | TypeScriptExpr::RuntimeReference { .. }
        | TypeScriptExpr::CurriedRuntimeReference { .. } => {}
    }
}

fn lower_branch(
    mut branch: CoreDecisionBranch,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptDecisionBranch {
    let erased_paths = branch
        .tests
        .iter()
        .filter_map(|test| match test {
            CoreDecisionTest::Constructor {
                path, constructor, ..
            } if imported_types.is_erased_newtype(constructor) => Some(path.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    fn erase_projection(
        path: &mut Vec<CoreDecisionProjection>,
        erased: &[Vec<CoreDecisionProjection>],
    ) {
        let original = path.clone();
        *path = original
            .iter()
            .enumerate()
            .filter(|(index, projection)| {
                !matches!(projection, CoreDecisionProjection::AdtPayload)
                    || !erased
                        .iter()
                        .any(|prefix| prefix.as_slice() == &original[..*index])
            })
            .map(|(_, projection)| projection.clone())
            .collect();
    }
    branch.tests.retain(|test| !matches!(test, CoreDecisionTest::Constructor { constructor, .. } if imported_types.is_erased_newtype(constructor)));
    for test in &mut branch.tests {
        match test {
            CoreDecisionTest::Integer { path, .. }
            | CoreDecisionTest::String { path, .. }
            | CoreDecisionTest::Boolean { path, .. }
            | CoreDecisionTest::Constructor { path, .. }
            | CoreDecisionTest::ArrayLength { path, .. }
            | CoreDecisionTest::ListLength { path, .. } => erase_projection(path, &erased_paths),
            CoreDecisionTest::Invalid { .. } => {}
        }
    }
    for binding in &mut branch.bindings {
        erase_projection(&mut binding.path, &erased_paths);
    }
    TypeScriptDecisionBranch {
        tests: branch.tests.into_iter().map(lower_test).collect(),
        bindings: branch
            .bindings
            .into_iter()
            .map(|binding| lower_binding(binding, imported_types))
            .collect(),
        guard: branch
            .guard
            .map(|guard| lower_core_expr_to_typescript(guard, imported_values, imported_types)),
        value: lower_core_expr_to_typescript(branch.value, imported_values, imported_types),
    }
}

fn lower_binding(
    binding: CoreDecisionBinding,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptDecisionBinding {
    TypeScriptDecisionBinding {
        name: safe_identifier(&binding.name),
        type_ref: type_ref_from_core_type(&binding.type_ref, imported_types),
        path: binding.path.into_iter().map(lower_projection).collect(),
    }
}

fn lower_test(test: CoreDecisionTest) -> TypeScriptDecisionTest {
    match test {
        CoreDecisionTest::Integer { path, value, .. } => TypeScriptDecisionTest::NumberEquals {
            path: path.into_iter().map(lower_projection).collect(),
            value,
        },
        CoreDecisionTest::String { path, value, .. } => TypeScriptDecisionTest::StringEquals {
            path: path.into_iter().map(lower_projection).collect(),
            value,
        },
        CoreDecisionTest::Boolean { path, value, .. } => TypeScriptDecisionTest::BooleanEquals {
            path: path.into_iter().map(lower_projection).collect(),
            value,
        },
        CoreDecisionTest::Constructor {
            path, constructor, ..
        } => TypeScriptDecisionTest::TagEquals {
            path: path.into_iter().map(lower_projection).collect(),
            tag: local_name(&constructor),
        },
        CoreDecisionTest::ArrayLength {
            path,
            length,
            minimum,
            ..
        } => TypeScriptDecisionTest::ArrayLength {
            path: path.into_iter().map(lower_projection).collect(),
            length,
            minimum,
        },
        CoreDecisionTest::ListLength {
            path,
            length,
            minimum,
            ..
        } => TypeScriptDecisionTest::ListLength {
            path: path.into_iter().map(lower_projection).collect(),
            length,
            minimum,
        },
        CoreDecisionTest::Invalid { .. } => TypeScriptDecisionTest::Invalid,
    }
}

fn lower_projection(projection: CoreDecisionProjection) -> TypeScriptDecisionProjection {
    match projection {
        CoreDecisionProjection::TupleElement { index } => {
            TypeScriptDecisionProjection::TupleElement { index }
        }
        CoreDecisionProjection::ArrayElement { index } => {
            TypeScriptDecisionProjection::ArrayElement { index }
        }
        CoreDecisionProjection::ArrayRest { start } => {
            TypeScriptDecisionProjection::ArrayRest { start }
        }
        CoreDecisionProjection::ListElement { index } => {
            TypeScriptDecisionProjection::ListElement { index }
        }
        CoreDecisionProjection::ListRest { start } => {
            TypeScriptDecisionProjection::ListRest { start }
        }
        CoreDecisionProjection::RecordField { name } => {
            TypeScriptDecisionProjection::RecordField { name }
        }
        CoreDecisionProjection::AdtPayload => TypeScriptDecisionProjection::AdtPayload,
    }
}

fn core_pattern_type(pattern: &CorePattern) -> CoreType {
    match pattern {
        CorePattern::Integer { type_ref, .. }
        | CorePattern::String { type_ref, .. }
        | CorePattern::Char { type_ref, .. }
        | CorePattern::Boolean { type_ref, .. }
        | CorePattern::Binding { type_ref, .. }
        | CorePattern::Wildcard { type_ref, .. }
        | CorePattern::Constructor { type_ref, .. }
        | CorePattern::Tuple { type_ref, .. }
        | CorePattern::Array { type_ref, .. }
        | CorePattern::List { type_ref, .. }
        | CorePattern::Record { type_ref, .. } => type_ref.clone(),
        CorePattern::Invalid { .. } => CoreType::Hole,
    }
}

fn lower_pattern(
    pattern: CorePattern,
    path: &mut Vec<TypeScriptDecisionProjection>,
    tests: &mut Vec<TypeScriptDecisionTest>,
    bindings: &mut Vec<TypeScriptDecisionBinding>,
    imported_types: &TypeScriptTypeContext,
) {
    match pattern {
        CorePattern::Integer { value, .. } => tests.push(TypeScriptDecisionTest::NumberEquals {
            path: path.clone(),
            value,
        }),
        CorePattern::String { value, .. } | CorePattern::Char { value, .. } => {
            tests.push(TypeScriptDecisionTest::StringEquals {
                path: path.clone(),
                value,
            })
        }
        CorePattern::Boolean { value, .. } => {
            tests.push(TypeScriptDecisionTest::BooleanEquals {
                path: path.clone(),
                value,
            });
        }
        CorePattern::Binding { name, type_ref, .. } => bindings.push(TypeScriptDecisionBinding {
            name: safe_identifier(&name),
            type_ref: type_ref_from_core_type(&type_ref, imported_types),
            path: path.clone(),
        }),
        CorePattern::Wildcard { .. } => {}
        CorePattern::Constructor {
            symbol, argument, ..
        } => {
            if imported_types.is_erased_newtype(&symbol) {
                if let Some(argument) = argument {
                    lower_pattern(*argument, path, tests, bindings, imported_types);
                }
                return;
            }
            tests.push(TypeScriptDecisionTest::TagEquals {
                path: path.clone(),
                tag: local_name(&symbol),
            });
            if let Some(argument) = argument {
                path.push(TypeScriptDecisionProjection::AdtPayload);
                lower_pattern(*argument, path, tests, bindings, imported_types);
                path.pop();
            }
        }
        CorePattern::Tuple { elements, .. } => {
            for (index, element) in elements.into_iter().enumerate() {
                path.push(TypeScriptDecisionProjection::TupleElement { index });
                lower_pattern(element, path, tests, bindings, imported_types);
                path.pop();
            }
        }
        CorePattern::Array { elements, rest, .. } => {
            let length = elements.len();
            tests.push(TypeScriptDecisionTest::ArrayLength {
                path: path.clone(),
                length,
                minimum: rest.is_some(),
            });
            for (index, element) in elements.into_iter().enumerate() {
                path.push(TypeScriptDecisionProjection::ArrayElement { index });
                lower_pattern(element, path, tests, bindings, imported_types);
                path.pop();
            }
            if let Some(rest) = rest {
                path.push(TypeScriptDecisionProjection::ArrayRest { start: length });
                lower_pattern(*rest, path, tests, bindings, imported_types);
                path.pop();
            }
        }
        CorePattern::List { elements, rest, .. } => {
            let length = elements.len();
            tests.push(TypeScriptDecisionTest::ListLength {
                path: path.clone(),
                length,
                minimum: rest.is_some(),
            });
            for (index, element) in elements.into_iter().enumerate() {
                path.push(TypeScriptDecisionProjection::ListElement { index });
                lower_pattern(element, path, tests, bindings, imported_types);
                path.pop();
            }
            if let Some(rest) = rest {
                path.push(TypeScriptDecisionProjection::ListRest { start: length });
                lower_pattern(*rest, path, tests, bindings, imported_types);
                path.pop();
            }
        }
        CorePattern::Record { fields, .. } => {
            for field in fields {
                path.push(TypeScriptDecisionProjection::RecordField { name: field.name });
                lower_pattern(field.pattern, path, tests, bindings, imported_types);
                path.pop();
            }
        }
        CorePattern::Invalid { .. } => tests.push(TypeScriptDecisionTest::Invalid),
    }
}

// Substitution is restricted to scope-free expressions so a nested binder can
// never capture the scrutinee name or shadow a pattern binding.
fn contains_scope(expr: &TypeScriptExpr) -> bool {
    match expr {
        TypeScriptExpr::Lambda { .. }
        | TypeScriptExpr::Sequence { .. }
        | TypeScriptExpr::MonadDo { .. }
        | TypeScriptExpr::Decision { .. } => true,
        TypeScriptExpr::Tuple { elements } | TypeScriptExpr::Array { elements, .. } => {
            elements.iter().any(contains_scope)
        }
        TypeScriptExpr::FieldAccess { receiver, .. }
        | TypeScriptExpr::OptionalFieldAccess { receiver, .. } => contains_scope(receiver),
        TypeScriptExpr::Record { items, .. } => items.iter().any(|item| match item {
            super::TypeScriptRecordValueItem::Field { value, .. }
            | super::TypeScriptRecordValueItem::Spread { value } => contains_scope(value),
        }),
        TypeScriptExpr::Binary { left, right, .. } => contains_scope(left) || contains_scope(right),
        TypeScriptExpr::Unary { operand, .. } => contains_scope(operand),
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            contains_scope(condition) || contains_scope(then_branch) || contains_scope(else_branch)
        }
        TypeScriptExpr::Call { arguments, .. }
        | TypeScriptExpr::TypeApplicationCall { arguments, .. }
        | TypeScriptExpr::ForeignTaskCall { arguments, .. }
        | TypeScriptExpr::RuntimeCall { arguments, .. } => arguments.iter().any(contains_scope),
        TypeScriptExpr::DictionaryCall {
            dictionary,
            arguments,
            ..
        } => contains_scope(dictionary) || arguments.iter().any(contains_scope),
        TypeScriptExpr::CheckedResult { value, .. } | TypeScriptExpr::Await { value } => {
            contains_scope(value)
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::Number { .. }
        | TypeScriptExpr::String { .. }
        | TypeScriptExpr::Boolean { .. }
        | TypeScriptExpr::Identifier { .. }
        | TypeScriptExpr::RuntimeReference { .. }
        | TypeScriptExpr::CurriedRuntimeReference { .. } => false,
    }
}
