use std::collections::BTreeMap;

use seseragi_syntax::{SurfaceDecl, SurfaceImplMember};

use crate::{ResolvedModule, SymbolId, TypedConstraint};

use super::super::functions::TopLevelPureFunction;
use super::super::semantic_types::{SemanticTypeCatalog, SemanticTypeKey};
use super::super::type_ref::typed_type_from_type_ref;
use super::imported_types::ImportedTypeContext;

#[derive(Default)]
pub(super) struct InherentMethodCatalog {
    methods: BTreeMap<(SymbolId, String), Vec<TopLevelPureFunction>>,
}

impl InherentMethodCatalog {
    pub(super) fn new(resolved: &ResolvedModule, semantic_types: &SemanticTypeCatalog) -> Self {
        let mut catalog = Self::default();
        for declaration in &resolved.declarations {
            let SurfaceDecl::Impl {
                type_parameters,
                target,
                constraints,
                members,
                ..
            } = declaration
            else {
                continue;
            };
            let target_key = semantic_types.key_from_type_ref(resolved, target);
            let Some(owner) = local_owner(&target_key) else {
                continue;
            };
            let Some(owner_identity) = resolved
                .symbols
                .iter()
                .find(|symbol| symbol.id == owner)
                .and_then(|symbol| symbol.canonical.as_deref())
            else {
                continue;
            };

            for member in members {
                let SurfaceImplMember::Method { method, .. } = member else {
                    continue;
                };
                let mut callable_type_parameters = type_parameters.clone();
                callable_type_parameters.extend(method.type_parameters.clone());
                let callable_constraints = constraints
                    .iter()
                    .chain(&method.constraints)
                    .map(|constraint| TypedConstraint {
                        name: constraint.name.clone(),
                        arguments: constraint
                            .arguments
                            .iter()
                            .map(typed_type_from_type_ref)
                            .collect(),
                    })
                    .collect::<Vec<_>>();
                let constraint_identities = constraints
                    .iter()
                    .chain(&method.constraints)
                    .map(|constraint| super::constraint_identity(resolved, constraint.name_span))
                    .collect::<Vec<_>>();
                let callable = TopLevelPureFunction {
                    symbol: format!("{owner_identity}::{}", method.name),
                    trait_identity: None,
                    trait_method: None,
                    type_parameters: callable_type_parameters,
                    constraints: callable_constraints,
                    constraint_identities,
                    parameters: method
                        .parameters
                        .iter()
                        .map(|parameter| typed_type_from_type_ref(&parameter.type_ref))
                        .collect(),
                    semantic_parameters: method
                        .parameters
                        .iter()
                        .map(|parameter| {
                            semantic_types.key_from_type_ref(resolved, &parameter.type_ref)
                        })
                        .collect(),
                    result: typed_type_from_type_ref(&method.return_type),
                    semantic_result: semantic_types
                        .key_from_type_ref(resolved, &method.return_type),
                };
                catalog
                    .methods
                    .entry((owner, method.name.clone()))
                    .or_default()
                    .push(callable);
            }
        }
        let imported_types = ImportedTypeContext::new(resolved);
        for import in resolved.imports.iter().filter(|import| {
            import.export.namespace == "method"
                && import.export.declaration_kind.as_deref() == Some("inherent-method")
        }) {
            let Some(owner) = import.member_owner else {
                continue;
            };
            let Some(callable) = super::imports::imported_callable(&imported_types, import) else {
                continue;
            };
            catalog
                .methods
                .entry((owner, import.export.name.clone()))
                .or_default()
                .push(callable);
        }
        catalog
    }

    pub(super) fn unique(
        &self,
        receiver: &SemanticTypeKey,
        name: &str,
    ) -> Option<&TopLevelPureFunction> {
        let owner = local_owner(receiver)?;
        let methods = self.methods.get(&(owner, name.to_owned()))?;
        (methods.len() == 1).then(|| &methods[0])
    }
}

fn local_owner(key: &SemanticTypeKey) -> Option<SymbolId> {
    match key {
        SemanticTypeKey::Adt { owner, .. } | SemanticTypeKey::Struct { owner, .. } => Some(*owner),
        _ => None,
    }
}
