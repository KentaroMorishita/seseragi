use crate::CoreAdt;
use seseragi_syntax::Visibility;
use std::collections::BTreeMap;

use super::names::{local_name, safe_identifier};
use super::runtime::collect_type_runtime_requirement;
use super::types::type_ref_from_core_type;
use super::{push_unique, TypeScriptAdt, TypeScriptAdtVariant};

pub(super) fn lower_core_adt_to_typescript(
    adt: CoreAdt,
    imported_types: &BTreeMap<String, String>,
    runtime_requirements: &mut Vec<String>,
) -> TypeScriptAdt {
    push_unique(runtime_requirements, "core.adt");
    let exported = adt.visibility == Visibility::Public;
    let constructors_exported = exported && !adt.opaque;
    TypeScriptAdt {
        exported,
        name: local_name(&adt.symbol),
        type_parameters: adt
            .type_parameters
            .into_iter()
            .map(|parameter| safe_identifier(&parameter))
            .collect(),
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
                        .map(|payload| type_ref_from_core_type(payload, imported_types)),
                    origin: variant.origin,
                }
            })
            .collect(),
        origin: adt.origin,
    }
}
