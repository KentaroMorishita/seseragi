use crate::{
    ExternalTypeBinding, ResolvedModule, ResolvedSymbol, SymbolId, SymbolKind, SymbolNamespace,
    TypedModuleDependency, TypedParameter, TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceDecl, TypeRef};
use std::collections::BTreeMap;

use super::functions::TopLevelPureFunction;
use super::semantic_types::{SemanticTypeCatalog, SemanticTypeKey, SemanticValueType};
use super::surface_expr::surface_expression_type_hint;
use super::type_ref::typed_type_from_type_ref;

mod imported_effects;
mod imported_types;
mod imports;
mod module_dependencies;
mod type_bindings;

pub(crate) struct TypedResolution<'a> {
    resolved: &'a ResolvedModule,
    top_level_values: BTreeMap<SymbolId, TypedType>,
    callables: BTreeMap<SymbolId, TopLevelPureFunction>,
    imported_effects: BTreeMap<SymbolId, imported_effects::ImportedEffectFunction>,
    semantic_values: BTreeMap<SymbolId, SemanticTypeKey>,
    semantic_types: SemanticTypeCatalog,
}

impl<'a> TypedResolution<'a> {
    pub(crate) fn new(resolved: &'a ResolvedModule) -> Self {
        let semantic_types = SemanticTypeCatalog::new(resolved);
        Self {
            resolved,
            top_level_values: collect_top_level_value_types(resolved),
            callables: collect_callables(resolved, &semantic_types),
            imported_effects: imported_effects::collect_imported_effects(resolved),
            semantic_values: collect_semantic_value_types(resolved, &semantic_types),
            semantic_types,
        }
    }

    pub(crate) fn declaration_symbol(
        &self,
        origin: ByteSpan,
        kind: SymbolKind,
    ) -> Option<&ResolvedSymbol> {
        self.resolved
            .symbols
            .iter()
            .find(|symbol| symbol.kind == kind && symbol.origin == origin)
    }

    pub(crate) fn target(&self, origin: ByteSpan, namespace: SymbolNamespace) -> Option<SymbolId> {
        self.resolved
            .references
            .iter()
            .find(|reference| reference.namespace == namespace && reference.origin == origin)
            .and_then(|reference| reference.target)
    }

    pub(crate) fn candidates(&self, origin: ByteSpan, namespace: SymbolNamespace) -> &[SymbolId] {
        self.resolved
            .references
            .iter()
            .find(|reference| reference.namespace == namespace && reference.origin == origin)
            .map(|reference| reference.candidates.as_slice())
            .unwrap_or_default()
    }

    pub(crate) fn symbol(&self, id: SymbolId) -> Option<&ResolvedSymbol> {
        self.resolved.symbols.iter().find(|symbol| symbol.id == id)
    }

    pub(crate) fn top_level_value_type(&self, id: SymbolId) -> Option<&TypedType> {
        self.top_level_values.get(&id)
    }

    pub(crate) fn callable(&self, id: SymbolId) -> Option<&TopLevelPureFunction> {
        self.callables.get(&id)
    }

    pub(crate) fn imported_effect(
        &self,
        id: SymbolId,
    ) -> Option<&imported_effects::ImportedEffectFunction> {
        self.imported_effects.get(&id)
    }

    pub(crate) fn semantic_value_key(&self, id: SymbolId) -> SemanticTypeKey {
        self.semantic_values
            .get(&id)
            .cloned()
            .unwrap_or(SemanticTypeKey::Invalid)
    }

    pub(crate) fn semantic_types(&self) -> &SemanticTypeCatalog {
        &self.semantic_types
    }

    pub(crate) fn resolved(&self) -> &ResolvedModule {
        self.resolved
    }

    pub(crate) fn dependency_instance(
        &self,
        trait_name: &str,
        type_identity: &str,
    ) -> Option<&crate::ResolvedDependencyInstance> {
        self.resolved.dependency_instances.iter().find(|instance| {
            instance.trait_name == trait_name && instance.type_identity == type_identity
        })
    }

    pub(crate) fn external_type_bindings(&self) -> Vec<ExternalTypeBinding> {
        type_bindings::collect_external_type_bindings(self.resolved)
    }

    pub(crate) fn module_dependencies(&self) -> Vec<TypedModuleDependency> {
        module_dependencies::collect_module_dependencies(self.resolved)
    }

    pub(crate) fn semantic_value_from_type_ref(&self, type_ref: &TypeRef) -> SemanticValueType {
        SemanticValueType {
            type_ref: typed_type_from_type_ref(type_ref),
            key: self
                .semantic_types
                .key_from_type_ref(self.resolved, type_ref),
        }
    }

    pub(crate) fn semantic_value_from_typed_type(&self, type_ref: &TypedType) -> SemanticValueType {
        SemanticValueType {
            type_ref: type_ref.clone(),
            key: self
                .semantic_types
                .key_from_typed_type(self.resolved, type_ref),
        }
    }

    pub(crate) fn parameter_types(
        &self,
        parameters: &[TypedParameter],
    ) -> BTreeMap<SymbolId, SemanticValueType> {
        parameters
            .iter()
            .filter_map(|parameter| {
                let (origin, type_ref) = match parameter {
                    TypedParameter::Named {
                        origin, type_ref, ..
                    } => (*origin, type_ref.clone()),
                    TypedParameter::ImplicitUnit { .. } => return None,
                };
                let symbol = self.declaration_symbol(origin, SymbolKind::Parameter)?;
                Some((
                    symbol.id,
                    SemanticValueType {
                        type_ref,
                        key: self.semantic_value_key(symbol.id),
                    },
                ))
            })
            .collect()
    }
}

fn collect_top_level_value_types(resolved: &ResolvedModule) -> BTreeMap<SymbolId, TypedType> {
    resolved
        .declarations
        .iter()
        .filter_map(|declaration| {
            let SurfaceDecl::Let {
                name_span,
                type_ref,
                body,
                ..
            } = declaration
            else {
                return None;
            };
            let symbol = resolved
                .symbols
                .iter()
                .find(|symbol| symbol.kind == SymbolKind::Let && symbol.origin == *name_span)?;
            let type_ref = type_ref
                .as_ref()
                .map(typed_type_from_type_ref)
                .or_else(|| body.as_ref().and_then(surface_expression_type_hint))?;
            Some((symbol.id, type_ref))
        })
        .collect()
}

fn collect_callables(
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
) -> BTreeMap<SymbolId, TopLevelPureFunction> {
    let mut callables = BTreeMap::new();
    for declaration in &resolved.declarations {
        match declaration {
            SurfaceDecl::Fn {
                name_span,
                type_parameters,
                parameters,
                return_type,
                constraints,
                ..
            } if constraints.is_empty() && !parameters.is_empty() => {
                let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::Function && symbol.origin == *name_span
                }) else {
                    continue;
                };
                let semantic_parameters = parameters
                    .iter()
                    .map(|parameter| {
                        semantic_types.key_from_type_ref(resolved, &parameter.type_ref)
                    })
                    .collect::<Vec<_>>();
                let parameters = parameters
                    .iter()
                    .map(|parameter| typed_type_from_type_ref(&parameter.type_ref))
                    .collect::<Vec<_>>();
                let result = typed_type_from_type_ref(return_type);
                let Some(canonical) = symbol.canonical.clone() else {
                    continue;
                };
                callables.insert(
                    symbol.id,
                    TopLevelPureFunction {
                        symbol: canonical,
                        trait_identity: None,
                        trait_method: None,
                        type_parameters: type_parameters.clone(),
                        constraints: Vec::new(),
                        parameters,
                        semantic_parameters,
                        result,
                        semantic_result: semantic_types.key_from_type_ref(resolved, return_type),
                    },
                );
            }
            _ => {}
        }
    }
    for (constructor, signature) in semantic_types.constructor_signatures() {
        callables.insert(
            constructor,
            TopLevelPureFunction {
                symbol: signature.symbol,
                trait_identity: None,
                trait_method: None,
                type_parameters: signature.type_parameters,
                constraints: Vec::new(),
                parameters: signature
                    .parameters
                    .iter()
                    .map(|parameter| parameter.type_ref.clone())
                    .collect(),
                semantic_parameters: signature
                    .parameters
                    .into_iter()
                    .map(|parameter| parameter.key)
                    .collect(),
                result: signature.result.type_ref,
                semantic_result: signature.result.key,
            },
        );
    }
    collect_local_trait_methods(resolved, semantic_types, &mut callables);
    callables.extend(imports::collect_imported_callables(resolved));
    for symbol in &resolved.symbols {
        if symbol.canonical.as_deref() == Some("std/prelude::reduce") {
            callables.insert(symbol.id, standard_reduce_callable());
        }
    }
    callables
}

fn standard_reduce_callable() -> TopLevelPureFunction {
    let accumulator = TypedType::Named {
        name: "B".to_owned(),
        arguments: Vec::new(),
    };
    let element = TypedType::Named {
        name: "A".to_owned(),
        arguments: Vec::new(),
    };
    let collection = TypedType::Named {
        name: "C".to_owned(),
        arguments: Vec::new(),
    };
    TopLevelPureFunction {
        symbol: "std/prelude::reduce".to_owned(),
        trait_identity: None,
        trait_method: None,
        type_parameters: vec!["C".to_owned(), "A".to_owned(), "B".to_owned()],
        constraints: vec![crate::TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![collection.clone(), element.clone()],
        }],
        parameters: vec![
            accumulator.clone(),
            TypedType::Function {
                parameter: Box::new(accumulator.clone()),
                result: Box::new(TypedType::Function {
                    parameter: Box::new(element),
                    result: Box::new(accumulator.clone()),
                }),
            },
            collection,
        ],
        semantic_parameters: vec![SemanticTypeKey::Other; 3],
        result: accumulator,
        semantic_result: SemanticTypeKey::Other,
    }
}

fn collect_local_trait_methods(
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
    callables: &mut BTreeMap<SymbolId, TopLevelPureFunction>,
) {
    for declaration in &resolved.declarations {
        let SurfaceDecl::Trait {
            name,
            name_span,
            type_parameters,
            methods,
            ..
        } = declaration
        else {
            continue;
        };
        let Some(trait_identity) = resolved.symbols.iter().find_map(|symbol| {
            (symbol.kind == SymbolKind::Trait && symbol.origin == *name_span)
                .then(|| symbol.canonical.clone())
                .flatten()
        }) else {
            continue;
        };
        for method in methods {
            let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                symbol.kind == SymbolKind::TraitMethod && symbol.origin == method.name_span
            }) else {
                continue;
            };
            let Some(canonical) = symbol.canonical.clone() else {
                continue;
            };
            let mut callable_type_parameters = type_parameters.clone();
            callable_type_parameters.extend(method.type_parameters.clone());
            callables.insert(
                symbol.id,
                TopLevelPureFunction {
                    symbol: canonical,
                    trait_identity: Some(trait_identity.clone()),
                    trait_method: Some(method.name.clone()),
                    type_parameters: callable_type_parameters,
                    constraints: vec![crate::TypedConstraint {
                        name: name.clone(),
                        arguments: type_parameters
                            .iter()
                            .map(|parameter| TypedType::Named {
                                name: parameter.clone(),
                                arguments: Vec::new(),
                            })
                            .collect(),
                    }],
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
                },
            );
        }
    }
}

fn collect_semantic_value_types(
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
) -> BTreeMap<SymbolId, SemanticTypeKey> {
    let mut values = BTreeMap::new();
    for declaration in &resolved.declarations {
        match declaration {
            SurfaceDecl::Let {
                name_span,
                type_ref: Some(type_ref),
                ..
            } => {
                if let Some(symbol) = resolved
                    .symbols
                    .iter()
                    .find(|symbol| symbol.kind == SymbolKind::Let && symbol.origin == *name_span)
                {
                    values.insert(
                        symbol.id,
                        semantic_types.key_from_type_ref(resolved, type_ref),
                    );
                }
            }
            SurfaceDecl::Fn {
                parameters,
                name_span,
                return_type,
                ..
            } => {
                for parameter in parameters {
                    if let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                        symbol.kind == SymbolKind::Parameter && symbol.origin == parameter.name_span
                    }) {
                        values.insert(
                            symbol.id,
                            semantic_types.key_from_type_ref(resolved, &parameter.type_ref),
                        );
                    }
                }
                if let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::Function && symbol.origin == *name_span
                }) {
                    values.insert(
                        symbol.id,
                        semantic_types.key_from_type_ref(resolved, return_type),
                    );
                }
            }
            SurfaceDecl::Trait { methods, .. } | SurfaceDecl::Instance { methods, .. } => {
                for method in methods {
                    for parameter in &method.parameters {
                        if let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                            symbol.kind == SymbolKind::Parameter
                                && symbol.origin == parameter.name_span
                        }) {
                            values.insert(
                                symbol.id,
                                semantic_types.key_from_type_ref(resolved, &parameter.type_ref),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for (constructor, signature) in semantic_types.constructor_signatures() {
        values.insert(constructor, signature.result.key);
    }
    values
}

pub(super) fn contains_function_type(type_ref: &TypedType) -> bool {
    match type_ref {
        TypedType::Function { .. } => true,
        TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
            arguments.iter().any(contains_function_type)
        }
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| contains_function_type(&field.type_ref)),
        TypedType::Tuple { elements } => elements.iter().any(contains_function_type),
        TypedType::Hole => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_unconstrained_generic_and_higher_order_functions() {
        let resolved = crate::resolve_module(
            "artifact/functions/main.ssrg",
            "fn identity<A> value: A -> A = value\nfn constrained<A> value: A -> A\nwhere Eq<A> = value\nfn apply f: (Int -> Int) -> value: Int -> Int = f value\n",
        );

        let semantic_types = SemanticTypeCatalog::new(&resolved);
        let callables = collect_callables(&resolved, &semantic_types);
        assert_eq!(callables.len(), 2);
        assert!(callables
            .values()
            .any(|callable| callable.symbol == "artifact/functions::identity"
                && callable.type_parameters == ["A"]));
        assert!(callables
            .values()
            .any(|callable| callable.symbol == "artifact/functions::apply"
                && callable.parameters.len() == 2));
    }
}
