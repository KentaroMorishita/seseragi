use super::{CoreCallEvidence, CoreInstanceEvidence, CoreType};
use std::collections::BTreeSet;

/// Certify only the compiler's pure tuple Show/Debug factory family. A closed
/// user instance or an arbitrary Standard identity is not a purity proof.
/// This permits lazy reuse at the original read, never eager evaluation.
pub fn closed_standard_display_key<'a>(
    selected: &CoreCallEvidence,
    bound_type_names: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
        return None;
    };
    if !matches!(identity.as_str(), "std/tuple::Show" | "std/tuple::Debug") {
        return None;
    }
    let bound = bound_type_names.into_iter().collect::<BTreeSet<_>>();
    certify(selected, &bound, 0)
}

fn certify(selected: &CoreCallEvidence, bound: &BTreeSet<&str>, depth: usize) -> Option<String> {
    if depth > 64 {
        return None;
    }
    let method = selected.constraint.name.as_str();
    if !matches!(method, "Show" | "Debug")
        || selected.constraint.trait_identity.as_deref() != Some(&format!("std/prelude::{method}"))
    {
        return None;
    }
    let [ty] = selected.constraint.arguments.as_slice() else {
        return None;
    };
    let CoreInstanceEvidence::Standard {
        identity,
        type_arguments,
        evidence_arguments,
    } = &selected.evidence
    else {
        return None;
    };
    let type_key = display_type_key(ty, bound, depth)?;
    if identity == &format!("std/tuple::{method}") {
        let [tuple @ CoreType::Tuple { elements }] = type_arguments.as_slice() else {
            return None;
        };
        if Some(type_key.clone()) != display_type_key(tuple, bound, depth)
            || elements.len() != evidence_arguments.len()
        {
            return None;
        }
        for (element, child) in elements.iter().zip(evidence_arguments) {
            let [child_type] = child.constraint.arguments.as_slice() else {
                return None;
            };
            if child.constraint.name != method
                || display_type_key(element, bound, depth + 1)?
                    != display_type_key(child_type, bound, depth + 1)?
            {
                return None;
            }
            certify(child, bound, depth + 1)?;
        }
        return Some(format!("{method}:{type_key}"));
    }
    if !type_arguments.is_empty() || !evidence_arguments.is_empty() {
        return None;
    }
    for primitive in ["Unit", "Bool", "Int", "Float", "Char", "String"] {
        let canonical = format!("std/prelude::{primitive}");
        if type_key == canonical && identity == &format!("{method}<{canonical}>") {
            return Some(format!("{method}:{type_key}"));
        }
    }
    None
}

// Bound recursion before structural comparison; do not clone or recursively
// compare an arbitrary unproved type just to decide whether it is closed.
fn display_type_key(ty: &CoreType, bound: &BTreeSet<&str>, depth: usize) -> Option<String> {
    if depth > 64 {
        return None;
    }
    if let CoreType::Tuple { elements } = ty {
        if elements.len() < 2 {
            return None;
        }
        let children = elements
            .iter()
            .map(|element| display_type_key(element, bound, depth + 1))
            .collect::<Option<Vec<_>>>()?;
        return Some(format!("Tuple[{}]", children.join(",")));
    }
    for primitive in ["Unit", "Bool", "Int", "Float", "Char", "String"] {
        let canonical = format!("std/prelude::{primitive}");
        let proved = match ty {
            CoreType::Named { name, arguments } => {
                !bound.contains(name.as_str()) && name == primitive && arguments.is_empty()
            }
            CoreType::ExternalNamed {
                canonical: actual,
                arguments,
                ..
            } => actual == &canonical && arguments.is_empty(),
            _ => false,
        };
        if proved {
            return Some(canonical);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selected(method: &str) -> CoreCallEvidence {
        let source =
            format!("pub fn kernel value: Int -> String = {method} (value, value + 1, True)");
        let module = crate::lower_typed_module(seseragi_semantics::type_module(
            "display/main.ssrg",
            &source,
        ));
        let super::super::CoreExpr::Call { evidence, .. } = &module.functions[0].body else {
            panic!("call")
        };
        evidence[0].clone()
    }

    #[test]
    fn proves_only_closed_canonical_standard_display_factories() {
        let show = selected("show");
        let debug = selected("debug");
        let key = closed_standard_display_key(&show, []).expect("closed tuple");
        assert_ne!(Some(key.clone()), closed_standard_display_key(&debug, []));
        assert_eq!(Some(key), closed_standard_display_key(&show, ["A"]));
        assert_eq!(None, closed_standard_display_key(&show, ["Int"]));
        let original = show.clone();
        let _ = closed_standard_display_key(&show, []);
        assert_eq!(show, original);
    }

    #[test]
    fn unproved_factories_and_deep_type_trees_fall_back_without_cloning_them() {
        let source = "pub fn generic<A> value: A -> String where Show<A> = show (value, True)";
        let module =
            crate::lower_typed_module(seseragi_semantics::type_module("display/main.ssrg", source));
        let super::super::CoreExpr::Call { evidence, .. } = &module.functions[0].body else {
            panic!("call")
        };
        assert_eq!(closed_standard_display_key(&evidence[0], ["A"]), None);
        let mut ty = CoreType::Named {
            name: "Int".into(),
            arguments: vec![],
        };
        for _ in 0..66 {
            ty = CoreType::Tuple {
                elements: vec![
                    ty,
                    CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    },
                ],
            };
        }
        assert_eq!(display_type_key(&ty, &BTreeSet::new(), 0), None);
        let mut selected = selected("show");
        selected.constraint.arguments = vec![ty];
        assert_eq!(closed_standard_display_key(&selected, []), None);
    }

    #[test]
    fn refuses_forged_owner_arity_type_child_and_dynamic_evidence() {
        let original = selected("show");
        let mut changed = original.clone();
        changed.constraint.trait_identity = Some("user::Show".into());
        assert_eq!(closed_standard_display_key(&changed, []), None);
        let mut changed = original.clone();
        changed.constraint.trait_identity = None;
        assert_eq!(closed_standard_display_key(&changed, []), None);
        let mut changed = original.clone();
        changed.constraint.arguments.clear();
        assert_eq!(closed_standard_display_key(&changed, []), None);
        let mut changed = original.clone();
        changed.constraint.arguments.push(CoreType::Hole);
        assert_eq!(closed_standard_display_key(&changed, []), None);
        for index in 0..7 {
            let mut changed = original.clone();
            let CoreInstanceEvidence::Standard {
                identity,
                type_arguments,
                evidence_arguments,
            } = &mut changed.evidence
            else {
                panic!("standard")
            };
            match index {
                0 => *identity = "user::Show".into(),
                1 => type_arguments.clear(),
                2 => evidence_arguments.clear(),
                3 => {
                    evidence_arguments[0].constraint.arguments = vec![CoreType::Named {
                        name: "user::Int".into(),
                        arguments: vec![],
                    }]
                }
                4 => evidence_arguments[0].constraint.trait_identity = Some("user::Show".into()),
                5 => evidence_arguments[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
                6 => {
                    evidence_arguments[0].evidence = CoreInstanceEvidence::Local {
                        identity: "user::Show".into(),
                        type_arguments: vec![],
                        evidence_arguments: vec![],
                    }
                }
                _ => unreachable!(),
            }
            assert_eq!(
                closed_standard_display_key(&changed, []),
                None,
                "mutation {index}"
            );
        }
    }
}
