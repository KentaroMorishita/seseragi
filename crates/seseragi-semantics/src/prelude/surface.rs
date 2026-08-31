use serde::Serialize;
use seseragi_syntax::TypeParameter;

#[cfg(test)]
use crate::TypedType;

use super::{
    standard_instance_constraint_specs, trait_by_name, trait_method_signature,
    PreludeTraitMethodSignature, SPECIAL_STANDARD_INSTANCES, STANDARD_INSTANCES, TRAITS,
    TRAIT_METHODS,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleSurface {
    schema: u32,
    kind: &'static str,
    language_version: &'static str,
    module: &'static str,
    traits: Vec<StandardTraitSurface>,
    instances: Vec<StandardInstanceSurface>,
    builtin_instances: Vec<StandardBuiltinInstanceSurface>,
    coherence: StandardCoherenceSurface,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardTraitSurface {
    name: &'static str,
    canonical: &'static str,
    type_parameters: Vec<TypeParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supertrait: Option<&'static str>,
    #[serde(default, skip_serializing_if = "is_false")]
    deriving: bool,
    methods: Vec<StandardTraitMethodSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardTraitMethodSurface {
    name: &'static str,
    canonical: &'static str,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    operators: Vec<&'static str>,
    signature: PreludeTraitMethodSignature,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardInstanceSurface {
    #[serde(rename = "trait")]
    trait_name: &'static str,
    trait_canonical: &'static str,
    type_constructor: &'static str,
    type_constructor_canonical: String,
    type_constructor_arity: u32,
    identity: &'static str,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    constraints: Vec<StandardInstanceConstraintSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardInstanceConstraintSurface {
    #[serde(rename = "trait")]
    trait_name: &'static str,
    trait_canonical: &'static str,
    type_argument_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardBuiltinInstanceSurface {
    #[serde(rename = "trait")]
    trait_name: &'static str,
    trait_canonical: &'static str,
    arguments: Vec<&'static str>,
    identity: &'static str,
    dispatch: super::PreludeSpecialInstanceDispatch,
    #[serde(default, skip_serializing_if = "is_false")]
    strict_equality_compatible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardCoherenceSurface {
    standard_heads: &'static str,
    user_overlap: &'static str,
    diagnostic: &'static str,
}

fn is_false(value: &bool) -> bool {
    !value
}

pub fn standard_prelude_surface() -> StandardModuleSurface {
    StandardModuleSurface {
        schema: 1,
        kind: "standard-module-surface",
        language_version: seseragi_project::IMPLEMENTED_LANGUAGE_VERSION,
        module: "std/prelude",
        traits: TRAITS
            .iter()
            .map(|trait_spec| StandardTraitSurface {
                name: trait_spec.name,
                canonical: trait_spec.canonical,
                type_parameters: trait_spec
                    .type_parameters
                    .iter()
                    .map(|parameter| {
                        if parameter.arity == 0 {
                            TypeParameter::value(parameter.name)
                        } else {
                            TypeParameter::constructor(parameter.name, parameter.arity)
                        }
                    })
                    .collect(),
                supertrait: trait_spec.supertrait,
                deriving: trait_spec.deriving,
                methods: TRAIT_METHODS
                    .iter()
                    .filter(|method| method.trait_name == trait_spec.name)
                    .map(|method| StandardTraitMethodSurface {
                        name: method.name,
                        canonical: method.canonical,
                        operators: method.operators.to_vec(),
                        signature: trait_method_signature(method),
                    })
                    .collect(),
            })
            .collect(),
        instances: STANDARD_INSTANCES
            .iter()
            .map(|instance| {
                let trait_spec = trait_by_name(instance.trait_name)
                    .expect("standard instance trait must exist in the Prelude registry");
                StandardInstanceSurface {
                    trait_name: instance.trait_name,
                    trait_canonical: trait_spec.canonical,
                    type_constructor: instance.type_name,
                    type_constructor_canonical: instance
                        .type_canonical
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("std/prelude::{}", instance.type_name)),
                    type_constructor_arity: instance.type_arity,
                    identity: instance.identity,
                    constraints: standard_instance_constraint_specs(instance.identity)
                        .iter()
                        .map(|constraint| {
                            let required = trait_by_name(constraint.trait_name)
                                .expect("standard instance constraint trait must exist");
                            StandardInstanceConstraintSurface {
                                trait_name: constraint.trait_name,
                                trait_canonical: required.canonical,
                                type_argument_index: constraint.type_argument_index,
                            }
                        })
                        .collect(),
                }
            })
            .collect(),
        builtin_instances: SPECIAL_STANDARD_INSTANCES
            .iter()
            .map(|instance| {
                let trait_spec = trait_by_name(instance.trait_name)
                    .expect("builtin instance trait must exist in the Prelude registry");
                StandardBuiltinInstanceSurface {
                    trait_name: instance.trait_name,
                    trait_canonical: trait_spec.canonical,
                    arguments: instance.arguments.to_vec(),
                    identity: instance.identity,
                    dispatch: instance.dispatch,
                    strict_equality_compatible: instance.strict_equality_compatible,
                }
            })
            .collect(),
        coherence: StandardCoherenceSurface {
            standard_heads: "sealed",
            user_overlap: "compile-error",
            diagnostic: "trait.instance-duplicate",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_registered_traits_methods_instances_and_coherence() {
        let surface = standard_prelude_surface();

        assert_eq!(surface.language_version, "0.1.0");
        assert_eq!(surface.traits.len(), 23);
        assert_eq!(
            surface
                .traits
                .iter()
                .flat_map(|trait_spec| &trait_spec.methods)
                .count(),
            24
        );
        assert_eq!(surface.instances.len(), 155);
        assert_eq!(surface.builtin_instances.len(), 24);
        for identity in [
            "std/int::Eq",
            "std/int::Zero",
            "std/string::Add",
            "std/float::Pow",
            "std/array::Iterable",
            "std/list::Reducible",
            "std/range::Reducible",
        ] {
            assert!(surface
                .builtin_instances
                .iter()
                .any(|instance| instance.identity == identity));
        }
        for identity in [
            "Show<std/bytes::ByteError>",
            "Debug<std/bytes::BytesSliceError>",
            "Show<std/text::Utf8DecodeError>",
            "Show<std/effect::ScheduleError>",
            "Show<std/effect::ParallelismError>",
            "Debug<std/effect::ParallelismError>",
            "Show<std/stream::BufferCapacityError>",
            "Debug<std/stream::BufferCapacityError>",
            "Show<std/queue::QueueCreateError>",
            "Debug<std/queue::QueueCreateError>",
            "Show<std/queue::QueueClosed>",
            "Debug<std/queue::QueueClosed>",
            "Show<std/semaphore::SemaphoreCreateError>",
            "Debug<std/semaphore::SemaphoreCreateError>",
            "Show<std/path::PathError>",
            "Show<std/process::ProcessSignal>",
            "Debug<std/process::ProcessSignal>",
            "Show<std/process::ProcessError>",
            "Debug<std/process::ProcessError>",
            "Show<std/child-process::ChildProcessConfigError>",
            "Debug<std/child-process::ChildProcessError>",
            "Show<std/child-process::ChildExitStatus>",
            "Show<std/random::RandomRangeError>",
            "Debug<std/random::RandomRangeError>",
            "Show<std/random::RandomConfigError>",
            "Debug<std/random::RandomConfigError>",
            "Show<std/entropy::EntropyConfigError>",
            "Debug<std/entropy::EntropyConfigError>",
            "Show<std/entropy::EntropyError>",
            "Debug<std/entropy::EntropyError>",
            "Debug<std/fs::FileSystemError>",
            "Show<std/fs::FileTextError>",
            "Debug<std/time::DurationError>",
            "Show<std/time::DateTimeError>",
            "Debug<std/time::TimeZoneError>",
            "Show<std/stdin::StdinConfigError>",
            "Debug<std/log::LogError>",
            "Show<std/http::HttpBuildError>",
            "Debug<std/http::HttpError>",
            "Show<std/web/navigation::UrlBuildError>",
            "Debug<std/web/navigation::NavigationError>",
            "Show<std/web/storage::StorageArea>",
            "Debug<std/web/storage::StorageError>",
            "std/int::JsonEncode",
            "std/either::JsonDecode",
            "std/json::JsonEncode",
            "std/stream::Functor",
            "std/stream::Applicative",
            "std/stream::Monad",
        ] {
            assert!(surface
                .instances
                .iter()
                .any(|instance| instance.identity == identity));
        }
        assert_eq!(surface.coherence.standard_heads, "sealed");

        let monoid = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Monoid")
            .expect("Monoid must be part of the standard Prelude surface");
        assert_eq!(monoid.type_parameters, vec![TypeParameter::value("A")]);
        assert_eq!(monoid.supertrait, Some("Semigroup"));

        let debug = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Debug")
            .expect("Debug must be part of the standard Prelude surface");
        assert_eq!(debug.type_parameters, vec![TypeParameter::value("A")]);
        assert_eq!(debug.methods[0].name, "debug");
        assert_eq!(
            debug.methods[0].signature.result,
            TypedType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }
        );

        let array_show = surface
            .instances
            .iter()
            .find(|instance| instance.identity == "std/array::Show")
            .expect("Array Show must be part of the standard Prelude surface");
        assert_eq!(array_show.constraints.len(), 1);
        assert_eq!(array_show.constraints[0].trait_name, "Show");
        assert_eq!(array_show.constraints[0].type_argument_index, 0);

        for name in [
            "Eq",
            "Ord",
            "Hash",
            "Show",
            "Debug",
            "Zero",
            "One",
            "Semigroup",
            "Monoid",
            "JsonEncode",
            "JsonDecode",
            "Functor",
            "Applicative",
            "Monad",
            "Iterable",
            "Reducible",
            "Traversable",
            "Add",
            "Sub",
            "Mul",
            "Div",
            "Rem",
            "Pow",
        ] {
            assert!(surface
                .traits
                .iter()
                .any(|trait_spec| trait_spec.name == name));
        }

        let reducible = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Reducible")
            .expect("Reducible must be part of the standard Prelude surface");
        assert_eq!(
            reducible.type_parameters,
            vec![TypeParameter::value("C"), TypeParameter::value("A")]
        );
        assert_eq!(reducible.supertrait, Some("Iterable"));

        let traversable = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Traversable")
            .expect("Traversable must be part of the standard Prelude surface");
        assert_eq!(traversable.supertrait, Some("Functor"));
        assert_eq!(traversable.methods[0].signature.constraints.len(), 1);
        assert_eq!(
            traversable.methods[0].signature.constraints[0].name,
            "Applicative"
        );

        let deriving = surface
            .traits
            .iter()
            .filter(|trait_spec| trait_spec.deriving)
            .map(|trait_spec| trait_spec.name)
            .collect::<Vec<_>>();
        assert_eq!(
            deriving,
            vec![
                "Eq",
                "Ord",
                "Hash",
                "Show",
                "Debug",
                "JsonEncode",
                "JsonDecode"
            ]
        );
    }
}
