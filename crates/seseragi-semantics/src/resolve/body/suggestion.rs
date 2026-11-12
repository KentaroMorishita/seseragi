//! Close names for an unresolved reference. A candidate is a name the
//! reference could use where it stands: a binding in an enclosing scope, a
//! Prelude name, or an export of the module a namespace alias names.
use super::Resolver;
use crate::{
    prelude::{PURE_FUNCTION_NAMES, STANDALONE_TYPE_NAMES, SUM_TYPES, TRAIT_METHODS},
    ScopeId, SymbolNamespace,
};
use std::collections::BTreeSet;

impl Resolver {
    /// The closest name in `namespace` visible from `scope`, if one is close.
    pub(super) fn closest_visible_name(
        &self,
        scope: ScopeId,
        namespace: SymbolNamespace,
        spelling: &str,
    ) -> Option<String> {
        if spelling.contains('.') {
            return None;
        }
        let mut chain = BTreeSet::new();
        let mut current = Some(scope);
        while let Some(scope) = current {
            chain.insert(scope);
            current = self
                .scopes
                .get(scope.0 as usize)
                .and_then(|scope| scope.parent);
        }
        let mut candidates = self
            .names
            .keys()
            .filter(|(scope, candidate_namespace, _)| {
                *candidate_namespace == namespace && chain.contains(scope)
            })
            .map(|(_, _, name)| name.as_str())
            .collect::<BTreeSet<_>>();
        match namespace {
            SymbolNamespace::Value => {
                // Boolean literals are tokens, not bindings, but are written
                // where a value name is.
                candidates.extend(["True", "False"]);
                candidates.extend(PURE_FUNCTION_NAMES.iter().copied());
                candidates.extend(
                    crate::effect_ops::known_effect_operations()
                        .map(|operation| operation.surface_name),
                );
                candidates.extend(
                    SUM_TYPES
                        .iter()
                        .flat_map(|sum_type| sum_type.variants.iter().map(|variant| variant.name)),
                );
                candidates.extend(TRAIT_METHODS.iter().map(|method| method.name));
            }
            SymbolNamespace::Type => {
                candidates.extend(STANDALONE_TYPE_NAMES.iter().copied());
                candidates.extend(SUM_TYPES.iter().map(|sum_type| sum_type.name));
            }
            _ => return None,
        }
        closest_name(spelling, candidates)
    }
}

/// The candidate nearest to `requested` by edit distance, when it is close
/// enough to be a misspelling: one edit for names up to four characters, two
/// up to eight, three beyond. Ties go to the alphabetically first name.
pub(super) fn closest_name<'a>(
    requested: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let length = requested.chars().count();
    let maximum = match length {
        0 | 1 => return None,
        2..=4 => 1,
        5..=8 => 2,
        _ => 3,
    };
    candidates
        .into_iter()
        .filter(|candidate| {
            *candidate != requested
                && candidate
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_alphabetic())
        })
        .map(|candidate| (edit_distance(requested, candidate), candidate))
        .filter(|(distance, candidate)| {
            *distance <= maximum && *distance < candidate.chars().count()
        })
        .min()
        .map(|(_, candidate)| candidate.to_owned())
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = Vec::with_capacity(right.len() + 1);
        current.push(left_index + 1);
        for (right_index, right_char) in right.iter().enumerate() {
            let substitution = previous[right_index] + usize::from(left_char != *right_char);
            current.push(
                (previous[right_index + 1] + 1)
                    .min(current[right_index] + 1)
                    .min(substitution),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::closest_name;

    #[test]
    fn suggests_only_a_close_spelling() {
        let names = ["println", "print", "show", "True", "length"];
        assert_eq!(closest_name("prinln", names).as_deref(), Some("println"));
        assert_eq!(closest_name("true", names).as_deref(), Some("True"));
        assert_eq!(closest_name("lenght", names).as_deref(), Some("length"));
        assert_eq!(closest_name("frobnicate", names), None);
        assert_eq!(closest_name("x", ["y"]), None);
        assert_eq!(closest_name("ab", ["b"]), None);
    }
}
