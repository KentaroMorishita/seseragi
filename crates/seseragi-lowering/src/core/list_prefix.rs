use super::{CoreDecisionBranch, CoreDecisionProjection, CoreDecisionTest, CoreType};
use serde::{Deserialize, Serialize};

/// Branch-local proof for an immutable canonical List at the decision root.
/// Each cursor is established by its own Cons check, in order, before any
/// source guard runs. No cursor or unchecked partial projection crosses arms.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreListPrefixPlan {
    pub length: usize,
    pub minimum: bool,
}

pub fn core_list_prefix_plan<'a>(
    scrutinee_type: &CoreType,
    branch: &CoreDecisionBranch,
    bound_type_names: impl IntoIterator<Item = &'a str>,
) -> Option<CoreListPrefixPlan> {
    // Built-in List has this distinguished Core type. User nominal names are
    // module-qualified; unexpanded aliases and external/higher-kinded shapes
    // deliberately retain the original decision path.
    if bound_type_names.into_iter().any(|name| name == "List")
        || !matches!(scrutinee_type, CoreType::Named { name, arguments }
        if name == "List" && arguments.len() == 1)
    {
        return None;
    }
    let CoreDecisionTest::ListLength {
        path,
        length,
        minimum,
        ..
    } = branch.tests.first()?
    else {
        return None;
    };
    if !path.is_empty() || *length < 2 || *length > 4096 {
        return None;
    }
    let covered = |path: &[CoreDecisionProjection]| match path.first() {
        None => true,
        Some(CoreDecisionProjection::ListElement { index }) => index < length,
        Some(CoreDecisionProjection::ListRest { start }) => start <= length,
        _ => false,
    };
    for test in &branch.tests[1..] {
        let path = match test {
            CoreDecisionTest::Integer { path, .. }
            | CoreDecisionTest::String { path, .. }
            | CoreDecisionTest::Boolean { path, .. }
            | CoreDecisionTest::Constructor { path, .. }
            | CoreDecisionTest::ArrayLength { path, .. }
            | CoreDecisionTest::ListLength { path, .. } => path,
            CoreDecisionTest::Invalid { .. } => return None,
        };
        if !covered(path) {
            return None;
        }
    }
    if !branch.bindings.iter().all(|binding| covered(&binding.path)) {
        return None;
    }
    Some(CoreListPrefixPlan {
        length: *length,
        minimum: *minimum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision() -> (CoreType, CoreDecisionBranch) {
        let module = crate::lower_typed_module(seseragi_semantics::type_module(
            "prefix/main.ssrg",
            "pub fn probe xs: List<Int> -> Int = match xs { `[a, b, ...tail] when a == 1 -> b\n _ -> 0 }",
        ));
        let super::super::CoreExpr::Decision {
            scrutinee_type,
            branches,
            ..
        } = &module.functions[0].body
        else {
            panic!("decision")
        };
        (scrutinee_type.clone(), branches[0].clone())
    }

    #[test]
    fn proves_only_the_checked_root_prefix_and_preserves_source_branch() {
        let (ty, branch) = decision();
        let original = branch.clone();
        assert_eq!(
            core_list_prefix_plan(&ty, &branch, []),
            Some(CoreListPrefixPlan {
                length: 2,
                minimum: true
            })
        );
        assert_eq!(branch, original);
        assert_eq!(core_list_prefix_plan(&ty, &branch, ["List"]), None);
        assert!(branch.guard.is_some());
        for ty in [
            CoreType::Hole,
            CoreType::Named {
                name: "prefix/main::List".into(),
                arguments: vec![CoreType::Hole],
            },
            CoreType::Named {
                name: "Alias".into(),
                arguments: vec![CoreType::Hole],
            },
            CoreType::Named {
                name: "List".into(),
                arguments: vec![],
            },
            CoreType::ExternalNamed {
                name: "List".into(),
                canonical: "user::List".into(),
                arguments: vec![CoreType::Hole],
            },
        ] {
            assert_eq!(core_list_prefix_plan(&ty, &branch, []), None);
        }
    }

    #[test]
    fn rejects_unchecked_root_traversals_and_out_of_bounds_projections() {
        let (ty, branch) = decision();
        let mut changed = branch.clone();
        changed.tests.remove(0);
        assert_eq!(core_list_prefix_plan(&ty, &changed, []), None);
        for path in [
            vec![CoreDecisionProjection::ListElement { index: 2 }],
            vec![CoreDecisionProjection::ListRest { start: 3 }],
            vec![CoreDecisionProjection::AdtPayload],
        ] {
            let mut changed = branch.clone();
            changed.bindings[0].path = path;
            assert_eq!(core_list_prefix_plan(&ty, &changed, []), None);
        }
        let mut changed = branch.clone();
        let CoreDecisionTest::ListLength { path, .. } = &mut changed.tests[0] else {
            panic!("length")
        };
        path.push(CoreDecisionProjection::AdtPayload);
        assert_eq!(core_list_prefix_plan(&ty, &changed, []), None);
        for length in [0, 1, 4097] {
            let mut changed = branch.clone();
            let CoreDecisionTest::ListLength { length: n, .. } = &mut changed.tests[0] else {
                panic!("length")
            };
            *n = length;
            assert_eq!(core_list_prefix_plan(&ty, &changed, []), None);
        }
    }
}
