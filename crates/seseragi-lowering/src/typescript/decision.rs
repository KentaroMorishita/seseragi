use crate::{
    CoreDecisionBinding, CoreDecisionBranch, CoreDecisionProjection, CoreDecisionTest, CoreExpr,
    CoreType,
};
use std::collections::BTreeMap;

use super::expr::lower_core_expr_to_typescript;
use super::names::{local_name, safe_identifier};
use super::types::type_ref_from_core_type;
use super::{
    TypeScriptDecisionBinding, TypeScriptDecisionBranch, TypeScriptDecisionProjection,
    TypeScriptDecisionTest, TypeScriptExpr,
};

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
        CoreDecisionTest::Invalid { .. } => TypeScriptDecisionTest::Invalid,
    }
}

fn lower_projection(projection: CoreDecisionProjection) -> TypeScriptDecisionProjection {
    match projection {
        CoreDecisionProjection::TupleElement { index } => {
            TypeScriptDecisionProjection::TupleElement { index }
        }
        CoreDecisionProjection::AdtPayload => TypeScriptDecisionProjection::AdtPayload,
    }
}
