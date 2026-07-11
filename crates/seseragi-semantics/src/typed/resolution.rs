use crate::{
    ResolvedModule, ResolvedSymbol, SymbolId, SymbolKind, SymbolNamespace, TypedParameter,
    TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceDecl};
use std::collections::BTreeMap;

use super::functions::TopLevelPureFunction;
use super::semantic_types::{SemanticTypeCatalog, SemanticTypeKey, SemanticValueType};
use super::surface_expr::surface_expression_type_hint;
use super::type_ref::typed_type_from_type_ref;

pub(crate) struct TypedResolution<'a> {
    resolved: &'a ResolvedModule,
    top_level_values: BTreeMap<SymbolId, TypedType>,
    callables: BTreeMap<SymbolId, TopLevelPureFunction>,
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

    pub(crate) fn symbol(&self, id: SymbolId) -> Option<&ResolvedSymbol> {
        self.resolved.symbols.iter().find(|symbol| symbol.id == id)
    }

    pub(crate) fn top_level_value_type(&self, id: SymbolId) -> Option<&TypedType> {
        self.top_level_values.get(&id)
    }

    pub(crate) fn callable(&self, id: SymbolId) -> Option<&TopLevelPureFunction> {
        self.callables.get(&id)
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
            } if type_parameters.is_empty() && constraints.is_empty() && !parameters.is_empty() => {
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
                if parameters.iter().any(contains_function_type) || contains_function_type(&result)
                {
                    continue;
                }
                let Some(canonical) = symbol.canonical.clone() else {
                    continue;
                };
                callables.insert(
                    symbol.id,
                    TopLevelPureFunction {
                        symbol: canonical,
                        type_parameters: Vec::new(),
                        parameters,
                        semantic_parameters,
                        result,
                        semantic_result: semantic_types.key_from_type_ref(resolved, return_type),
                    },
                );
            }
            SurfaceDecl::Type {
                name,
                type_parameters,
                variants,
                ..
            } => {
                let result = TypedType::Named {
                    name: name.clone(),
                    arguments: type_parameters
                        .iter()
                        .map(|parameter| TypedType::Named {
                            name: parameter.clone(),
                            arguments: Vec::new(),
                        })
                        .collect(),
                };
                for variant in variants {
                    let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                        symbol.kind == SymbolKind::Constructor && symbol.origin == variant.name_span
                    }) else {
                        continue;
                    };
                    let Some(canonical) = symbol.canonical.clone() else {
                        continue;
                    };
                    callables.insert(
                        symbol.id,
                        TopLevelPureFunction {
                            symbol: canonical,
                            type_parameters: type_parameters.clone(),
                            parameters: variant
                                .payload
                                .as_ref()
                                .map(typed_type_from_type_ref)
                                .into_iter()
                                .collect(),
                            semantic_parameters: semantic_types
                                .constructor(symbol.id)
                                .and_then(|(_, variant)| {
                                    variant.payload.as_ref().map(|payload| payload.key.clone())
                                })
                                .into_iter()
                                .collect(),
                            result: result.clone(),
                            semantic_result: semantic_types
                                .constructor(symbol.id)
                                .map(|(owner, _)| {
                                    semantic_types.polymorphic_adt_key(owner, &result)
                                })
                                .unwrap_or(SemanticTypeKey::Invalid),
                        },
                    );
                }
            }
            _ => {}
        }
    }
    callables
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
            _ => {}
        }
    }
    for symbol in &resolved.symbols {
        if symbol.kind != SymbolKind::Constructor {
            continue;
        }
        if let Some((owner, _)) = semantic_types.constructor(symbol.id) {
            let result = resolved
                .declarations
                .iter()
                .find_map(|declaration| {
                    let SurfaceDecl::Type {
                        name,
                        type_parameters,
                        variants,
                        ..
                    } = declaration
                    else {
                        return None;
                    };
                    variants
                        .iter()
                        .any(|variant| variant.name_span == symbol.origin)
                        .then(|| TypedType::Named {
                            name: name.clone(),
                            arguments: type_parameters
                                .iter()
                                .map(|parameter| TypedType::Named {
                                    name: parameter.clone(),
                                    arguments: Vec::new(),
                                })
                                .collect(),
                        })
                })
                .unwrap_or(TypedType::Hole);
            values.insert(
                symbol.id,
                semantic_types.polymorphic_adt_key(owner, &result),
            );
        }
    }
    values
}

fn contains_function_type(type_ref: &TypedType) -> bool {
    match type_ref {
        TypedType::Function { .. } => true,
        TypedType::Named { arguments, .. } => arguments.iter().any(contains_function_type),
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
    fn excludes_generic_and_higher_order_functions_from_direct_call_signatures() {
        let resolved = crate::resolve_module(
            "artifact/functions/main.ssrg",
            "fn identity<A> value: A -> A = value\nfn apply f: (Int -> Int) -> value: Int -> Int = f value\n",
        );

        let semantic_types = SemanticTypeCatalog::new(&resolved);
        assert!(collect_callables(&resolved, &semantic_types).is_empty());
    }
}
