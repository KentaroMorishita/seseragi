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
    pub(super) scrutinee_type: super::TypeScriptType,
    pub(super) tests: Vec<TypeScriptDecisionTest>,
    pub(super) bindings: Vec<TypeScriptDecisionBinding>,
}

pub(super) fn lower_core_pattern_decision(
    pattern: CorePattern,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptPatternDecision {
    let scrutinee_type = type_ref_from_core_type(&core_pattern_type(&pattern), imported_types);
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
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptExpr {
    TypeScriptExpr::Decision {
        scrutinee: Box::new(lower_core_expr_to_typescript(
            scrutinee,
            imported_values,
            imported_types,
        )),
        scrutinee_type: type_ref_from_core_type(&scrutinee_type, imported_types),
        branches: branches
            .into_iter()
            .map(|branch| lower_branch(branch, imported_values, imported_types))
            .collect(),
        type_ref: type_ref_from_core_type(&type_ref, imported_types),
    }
}

fn lower_branch(
    branch: CoreDecisionBranch,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptDecisionBranch {
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
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptDecisionBinding {
    TypeScriptDecisionBinding {
        name: safe_identifier(&binding.name),
        type_ref: type_ref_from_core_type(&binding.type_ref, imported_types),
        path: binding.path.into_iter().map(lower_projection).collect(),
    }
}

fn lower_test(test: CoreDecisionTest) -> TypeScriptDecisionTest {
    match test {
        CoreDecisionTest::Integer { path, value, .. } => TypeScriptDecisionTest::BigintEquals {
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
    imported_types: &BTreeMap<String, String>,
) {
    match pattern {
        CorePattern::Integer { value, .. } => tests.push(TypeScriptDecisionTest::BigintEquals {
            path: path.clone(),
            value,
        }),
        CorePattern::String { value, .. } => tests.push(TypeScriptDecisionTest::StringEquals {
            path: path.clone(),
            value,
        }),
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
