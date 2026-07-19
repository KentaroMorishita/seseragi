use serde::Serialize;
use seseragi_syntax::TypeParameter;

use super::{
    trait_by_name, trait_method_signature, PreludeTraitMethodSignature, STANDARD_INSTANCES, TRAITS,
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
    methods: Vec<StandardTraitMethodSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardTraitMethodSurface {
    name: &'static str,
    canonical: &'static str,
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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardCoherenceSurface {
    standard_heads: &'static str,
    user_overlap: &'static str,
    diagnostic: &'static str,
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
                type_parameters: vec![TypeParameter::constructor(trait_spec.type_parameter, 1)],
                supertrait: trait_spec.supertrait,
                methods: TRAIT_METHODS
                    .iter()
                    .filter(|method| method.trait_name == trait_spec.name)
                    .map(|method| StandardTraitMethodSurface {
                        name: method.name,
                        canonical: method.canonical,
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
        assert_eq!(surface.traits.len(), 3);
        assert_eq!(
            surface
                .traits
                .iter()
                .flat_map(|trait_spec| &trait_spec.methods)
                .count(),
            4
        );
        assert_eq!(surface.instances.len(), 17);
        assert_eq!(surface.coherence.standard_heads, "sealed");
    }
}
