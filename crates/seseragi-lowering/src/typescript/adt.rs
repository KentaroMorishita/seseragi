use crate::typescript::types::TypeScriptTypeContext;
use crate::CoreAdt;
use seseragi_syntax::Visibility;

use super::names::local_name;
use super::runtime::collect_type_runtime_requirement;
use super::types::type_ref_from_core_type;
use super::{push_unique, TypeScriptAdt, TypeScriptAdtVariant};

pub(super) fn lower_core_adt_to_typescript(
    adt: CoreAdt,
    imported_types: &TypeScriptTypeContext,
    runtime_requirements: &mut Vec<String>,
) -> TypeScriptAdt {
    push_unique(runtime_requirements, "core.adt");
    let exported = adt.visibility == Visibility::Public;
    let constructors_exported = exported && !adt.opaque;
    let scoped_types = imported_types.with_parameters(&adt.type_parameters);
    TypeScriptAdt {
        erased_newtype: adt.newtype
            && adt
                .variants
                .iter()
                .any(|variant| imported_types.is_erased_newtype(&variant.symbol)),
        exported,
        name: local_name(&adt.symbol),
        type_parameters: adt.type_parameters,
        variants: adt
            .variants
            .into_iter()
            .map(|variant| {
                if let Some(payload) = &variant.payload {
                    collect_type_runtime_requirement(payload, runtime_requirements);
                }
                TypeScriptAdtVariant {
                    exported: constructors_exported,
                    name: local_name(&variant.symbol),
                    tag: variant.name,
                    payload: variant
                        .payload
                        .as_ref()
                        .map(|payload| type_ref_from_core_type(payload, &scoped_types)),
                    origin: variant.origin,
                }
            })
            .collect(),
        origin: adt.origin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreAdtVariant, CoreType, SourceSpan, TypeScriptType};
    use seseragi_syntax::TypeParameter;

    #[test]
    fn adt_payloads_scope_constructor_and_value_parameters() {
        let origin = SourceSpan {
            source: "higher.ssrg".into(),
            start: 0,
            end: 1,
        };
        let named = |name: &str| CoreType::Named {
            name: name.into(),
            arguments: vec![],
        };
        let adt = CoreAdt {
            symbol: "app::Higher".into(),
            name: "Higher".into(),
            visibility: Visibility::Public,
            opaque: false,
            newtype: false,
            type_parameters: vec![
                TypeParameter::constructor("F", 1),
                TypeParameter::value("A"),
            ],
            variants: vec![CoreAdtVariant {
                symbol: "app::Higher".into(),
                name: "Higher".into(),
                payload: Some(CoreType::Tuple {
                    elements: vec![
                        CoreType::Named {
                            name: "F".into(),
                            arguments: vec![named("A")],
                        },
                        named("A"),
                    ],
                }),
                origin: origin.clone(),
            }],
            origin,
        };
        // An outer constructor spelling A must be shadowed by the ADT's own
        // value parameter. F<A> uses the existing HKT erasure policy.
        let types =
            TypeScriptTypeContext::default().with_parameters(&[TypeParameter::constructor("A", 1)]);
        let lowered = lower_core_adt_to_typescript(adt, &types, &mut vec![]);
        assert_eq!(
            lowered.variants[0].payload,
            Some(TypeScriptType::Tuple {
                elements: vec![
                    TypeScriptType::Unknown,
                    TypeScriptType::Reference {
                        name: "A".into(),
                        arguments: vec![]
                    }
                ]
            })
        );
        assert_eq!(lowered.type_parameters[0].arity, 1);
        assert_eq!(lowered.type_parameters[1].arity, 0);
    }
}
