//! A narrow checked contract for ordinary recursive functions returning Effects.
//! This does not infer a return type from the body or recognize a rendered TS name.
use std::collections::{BTreeMap, BTreeSet};

use crate::core_analysis::core_term_dependencies;
use crate::{CoreFunction, CoreModule, CoreType};

use super::types::TypeScriptTypeContext;
use super::TypeScriptType;

pub(super) fn providerless_effect_types(module: &CoreModule) -> BTreeMap<String, CoreType> {
    // Canonical top-level identities also include saved values between functions.
    // Local binders and imported modules cannot create an edge by spelling.
    let bodies = module
        .functions
        .iter()
        .map(|function| (&function.symbol, &function.body))
        .chain(
            module
                .bindings
                .iter()
                .map(|binding| (&binding.symbol, &binding.value)),
        )
        .collect::<BTreeMap<_, _>>();
    let edges = bodies
        .iter()
        .map(|(symbol, body)| {
            let references = core_term_dependencies(body)
                .map(|free| {
                    free.names
                        .into_iter()
                        .filter(|name| bodies.contains_key(name))
                        .collect()
                })
                .unwrap_or_default();
            ((*symbol).clone(), references)
        })
        .collect();
    let cyclic = cyclic_members(&edges);
    module
        .functions
        .iter()
        .filter_map(|function| {
            cyclic
                .contains(&function.symbol)
                .then(|| providerless_effect_type(module, function))
                .flatten()
                .map(|ty| (function.symbol.clone(), ty))
        })
        .collect()
}

// Iterative strongly connected components keep compiler stack and graph work
// bounded by the source symbol/edge graph, rather than searching from every fn.
fn cyclic_members(edges: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut order = Vec::new();
    for root in edges.keys() {
        let mut stack = vec![(root.clone(), false)];
        while let Some((symbol, expanded)) = stack.pop() {
            if expanded {
                order.push(symbol);
            } else if seen.insert(symbol.clone()) {
                stack.push((symbol.clone(), true));
                stack.extend(
                    edges[&symbol]
                        .iter()
                        .rev()
                        .map(|next| (next.clone(), false)),
                );
            }
        }
    }
    let mut reverse = edges
        .keys()
        .map(|name| (name.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (symbol, references) in edges {
        for next in references {
            reverse
                .get_mut(next)
                .expect("only known top-level edges")
                .insert(symbol.clone());
        }
    }
    seen.clear();
    let mut cyclic = BTreeSet::new();
    for root in order.into_iter().rev() {
        if seen.contains(&root) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![root];
        while let Some(symbol) = stack.pop() {
            if seen.insert(symbol.clone()) {
                stack.extend(reverse[&symbol].iter().cloned());
                component.push(symbol);
            }
        }
        if component.len() > 1 || edges[&component[0]].contains(&component[0]) {
            cyclic.extend(component);
        }
    }
    cyclic
}

fn providerless_effect_type(module: &CoreModule, function: &CoreFunction) -> Option<CoreType> {
    if function.is_effect
        || function.parameters.is_empty()
        || !function.type_parameters.is_empty()
        || !function.constraints.is_empty()
    {
        return None;
    }
    let checked = closed_type(module, function.value_type.as_ref()?)?;
    let mut result = &checked;
    for parameter in &function.parameters {
        let CoreType::Function {
            parameter: expected,
            result: next,
        } = result
        else {
            return None;
        };
        let actual = closed_type(module, &parameter.type_ref)?;
        if expected.as_ref() != &actual
            || !matches!(parameter.kind.as_str(), "named" | "implicit")
            || (parameter.kind == "implicit" && !canonical_is(&actual, "std/prelude::Unit"))
        {
            return None;
        }
        result = next;
    }
    // Source arity remains declaration metadata; returned callable stages are
    // inspected only to establish the eventual closed Effect result.
    while let CoreType::Function { result: next, .. } = result {
        result = next;
    }
    canonical_is(result, "std/prelude::Effect").then_some(checked)
}

fn canonical_is(ty: &CoreType, owner: &str) -> bool {
    matches!(ty, CoreType::ExternalNamed { canonical, .. } if canonical == owner)
}

fn owner(module: &CoreModule, name: &str) -> Option<String> {
    if name.contains("::") {
        return Some(name.to_owned());
    }
    if module
        .adts
        .iter()
        .map(|item| &item.name)
        .chain(module.structs.iter().map(|item| &item.name))
        .chain(module.aliases.iter().map(|item| &item.name))
        .any(|local| local == name)
    {
        return None;
    }
    let mut bindings = module
        .external_type_bindings
        .iter()
        .filter(|binding| binding.spelling == name);
    if let Some(first) = bindings.next() {
        let canonical = &first.canonical;
        let spelling = canonical.strip_prefix("std/prelude::")?;
        let arity = seseragi_semantics::prelude_type_constructor_arity(spelling)?;
        if first.arity.is_some_and(|actual| actual != arity)
            || bindings.any(|binding| {
                binding.canonical != *canonical
                    || binding.arity.is_some_and(|actual| actual != arity)
            })
        {
            return None;
        }
        return Some(canonical.clone());
    }
    // Bare checked names resolve through the lexical module before the
    // language Prelude. Use its declared kind, after excluding local/imported
    // owners, rather than accepting a same-spelling nominal or scoped binder.
    seseragi_semantics::prelude_type_constructor_arity(name)?;
    Some(format!("std/prelude::{name}"))
}

fn closed_type(module: &CoreModule, ty: &CoreType) -> Option<CoreType> {
    match ty {
        CoreType::Function { parameter, result } => Some(CoreType::Function {
            parameter: Box::new(closed_type(module, parameter)?),
            result: Box::new(closed_type(module, result)?),
        }),
        CoreType::Tuple { elements } => Some(CoreType::Tuple {
            elements: elements
                .iter()
                .map(|element| closed_type(module, element))
                .collect::<Option<_>>()?,
        }),
        CoreType::Named { name, arguments } => named_type(module, owner(module, name)?, arguments),
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => named_type(module, canonical.clone(), arguments),
        _ => None,
    }
}

fn named_type(module: &CoreModule, canonical: String, arguments: &[CoreType]) -> Option<CoreType> {
    let name = canonical.strip_prefix("std/prelude::")?;
    let arity = seseragi_semantics::prelude_type_constructor_arity(name)?;
    if module.external_type_bindings.iter().any(|binding| {
        binding.canonical == canonical && binding.arity.is_some_and(|actual| actual != arity)
    }) {
        return None;
    }
    let arguments = match (name, arguments) {
        ("Unit" | "Never" | "Bool" | "Int" | "Float" | "Char" | "String", []) => Vec::new(),
        ("Maybe", [_]) | ("Either", [_, _]) => arguments
            .iter()
            .map(|argument| closed_type(module, argument))
            .collect::<Option<_>>()?,
        (
            "Effect",
            [requirements @ CoreType::Record {
                closed: true,
                fields,
            }, failure, success],
        ) if fields.is_empty() => {
            vec![
                requirements.clone(),
                closed_type(module, failure)?,
                closed_type(module, success)?,
            ]
        }
        _ => return None,
    };
    Some(CoreType::ExternalNamed {
        name: name.to_owned(),
        canonical,
        arguments,
    })
}

/// Only the canonical closed tree returned above reaches this renderer. Its
/// Effect import name comes from the same runtime import plan as the annotation.
pub(super) fn annotation(
    checked: &CoreType,
    names: &TypeScriptTypeContext,
) -> Option<TypeScriptType> {
    Some(match checked {
        CoreType::Function { parameter, result } => TypeScriptType::Function {
            parameter: Box::new(annotation(parameter, names)?),
            result: Box::new(annotation(result, names)?),
        },
        CoreType::Tuple { elements } => TypeScriptType::Tuple {
            elements: elements
                .iter()
                .map(|element| annotation(element, names))
                .collect::<Option<_>>()?,
        },
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => match (canonical.as_str(), arguments.as_slice()) {
            ("std/prelude::Unit", []) => TypeScriptType::Undefined,
            ("std/prelude::Never", []) => TypeScriptType::Never,
            ("std/prelude::Bool", []) => TypeScriptType::Boolean,
            ("std/prelude::Int" | "std/prelude::Float", []) => TypeScriptType::Number,
            ("std/prelude::Char" | "std/prelude::String", []) => TypeScriptType::String,
            ("std/prelude::Maybe", [value]) => TypeScriptType::Maybe {
                element: Box::new(annotation(value, names)?),
            },
            ("std/prelude::Either", [error, value]) => TypeScriptType::Either {
                error: Box::new(annotation(error, names)?),
                value: Box::new(annotation(value, names)?),
            },
            (
                "std/prelude::Effect",
                [CoreType::Record {
                    closed: true,
                    fields,
                }, error, value],
            ) if fields.is_empty() => TypeScriptType::Reference {
                name: names.get("std/prelude::Effect")?.clone(),
                arguments: vec![
                    TypeScriptType::Record { fields: Vec::new() },
                    annotation(error, names)?,
                    annotation(value, names)?,
                ],
            },
            _ => return None,
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
