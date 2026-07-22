use crate::{
    ExternalTypeBinding, ResolvedModule, ResolvedSymbol, SymbolId, SymbolKind, SymbolNamespace,
    TypedModuleDependency, TypedParameter, TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceDecl, SurfaceImplMember, SurfaceRequirement, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

use super::functions::TopLevelPureFunction;
use super::semantic_types::{SemanticTypeCatalog, SemanticTypeKey, SemanticValueType};
use super::surface_expr::surface_expression_type_hint;
use super::type_ref::typed_type_from_type_ref;

mod imported_effects;
mod imported_types;
mod imports;
mod inherent_methods;
mod module_dependencies;
mod type_bindings;

pub(crate) struct TypedResolution<'a> {
    resolved: &'a ResolvedModule,
    top_level_values: BTreeMap<SymbolId, TypedType>,
    callables: BTreeMap<SymbolId, TopLevelPureFunction>,
    inherent_methods: inherent_methods::InherentMethodCatalog,
    imported_effects: BTreeMap<SymbolId, imported_effects::ImportedEffectFunction>,
    semantic_values: BTreeMap<SymbolId, SemanticTypeKey>,
    semantic_types: SemanticTypeCatalog,
}

impl<'a> TypedResolution<'a> {
    pub(crate) fn new(resolved: &'a ResolvedModule) -> Self {
        let semantic_types = SemanticTypeCatalog::new(resolved);
        Self {
            resolved,
            top_level_values: collect_top_level_value_types(resolved, &semantic_types),
            callables: collect_callables(resolved, &semantic_types),
            inherent_methods: inherent_methods::InherentMethodCatalog::new(
                resolved,
                &semantic_types,
            ),
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

    pub(crate) fn inherent_method(
        &self,
        receiver: &SemanticTypeKey,
        name: &str,
    ) -> Option<&TopLevelPureFunction> {
        self.inherent_methods.unique(receiver, name)
    }

    pub(crate) fn inherent_method_symbol(&self, target: &TypeRef, name: &str) -> Option<String> {
        let key = self.semantic_types.key_from_type_ref(self.resolved, target);
        self.inherent_methods
            .unique(&key, name)
            .map(|method| method.symbol.clone())
    }

    pub(crate) fn local_nominal_owner(&self, target: &TypeRef) -> Option<SymbolId> {
        match self.semantic_types.key_from_type_ref(self.resolved, target) {
            SemanticTypeKey::Adt { owner, .. } | SemanticTypeKey::Struct { owner, .. } => {
                Some(owner)
            }
            _ => None,
        }
    }

    pub(crate) fn same_semantic_type(&self, left: &TypeRef, right: &TypeRef) -> bool {
        self.semantic_types.key_from_type_ref(self.resolved, left)
            == self.semantic_types.key_from_type_ref(self.resolved, right)
    }

    pub(crate) fn type_has_canonical_identity(&self, type_ref: &TypeRef, canonical: &str) -> bool {
        let TypeRef::Named { span, .. } = type_ref else {
            return false;
        };
        self.target(*span, SymbolNamespace::Type)
            .and_then(|target| self.symbol(target))
            .and_then(|symbol| symbol.canonical.as_deref())
            == Some(canonical)
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
            instance.trait_name == trait_name
                && instance.type_identity.as_deref() == Some(type_identity)
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
            type_ref: semantic_typed_type_from_type_ref(
                type_ref,
                self.resolved,
                &self.semantic_types,
            ),
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

    pub(crate) fn semantic_value_from_imported_type(
        &self,
        type_ref: seseragi_syntax::InterfaceType,
        module: &str,
        type_parameters: &[seseragi_syntax::TypeParameter],
    ) -> Option<SemanticValueType> {
        let type_parameters = type_parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<BTreeSet<_>>();
        imported_types::ImportedTypeContext::new(self.resolved).semantic_value(
            type_ref,
            module,
            &type_parameters,
            &self.external_type_bindings(),
        )
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

fn collect_top_level_value_types(
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
) -> BTreeMap<SymbolId, TypedType> {
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
                .map(|type_ref| {
                    semantic_typed_type_from_type_ref(type_ref, resolved, semantic_types)
                })
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
            } => {
                let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::Function && symbol.origin == *name_span
                }) else {
                    continue;
                };
                let (parameters, semantic_parameters) =
                    callable_parameter_types(parameters, resolved, semantic_types);
                let result =
                    semantic_typed_type_from_type_ref(return_type, resolved, semantic_types);
                let constraint_identities = constraints
                    .iter()
                    .map(|constraint| constraint_identity(resolved, constraint.name_span))
                    .collect();
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
                        constraints: constraints
                            .iter()
                            .map(|constraint| crate::TypedConstraint {
                                name: constraint.name.clone(),
                                arguments: constraint
                                    .arguments
                                    .iter()
                                    .map(|argument| {
                                        semantic_typed_type_from_type_ref(
                                            argument,
                                            resolved,
                                            semantic_types,
                                        )
                                    })
                                    .collect(),
                            })
                            .collect(),
                        constraint_identities,
                        parameters,
                        semantic_parameters,
                        result,
                        semantic_result: semantic_types.key_from_type_ref(resolved, return_type),
                    },
                );
            }
            SurfaceDecl::EffectFn {
                name_span,
                type_parameters,
                parameters,
                return_type: Some(return_type),
                requirements,
                failure,
                inferred_contract: false,
                constraints,
                ..
            } => {
                let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::EffectFunction && symbol.origin == *name_span
                }) else {
                    continue;
                };
                let Some(canonical) = symbol.canonical.clone() else {
                    continue;
                };
                let (parameters, semantic_parameters) =
                    callable_parameter_types(parameters, resolved, semantic_types);
                callables.insert(
                    symbol.id,
                    TopLevelPureFunction {
                        symbol: canonical,
                        trait_identity: None,
                        trait_method: None,
                        type_parameters: type_parameters.clone(),
                        constraints: constraints
                            .iter()
                            .map(|constraint| crate::TypedConstraint {
                                name: constraint.name.clone(),
                                arguments: constraint
                                    .arguments
                                    .iter()
                                    .map(|argument| {
                                        semantic_typed_type_from_type_ref(
                                            argument,
                                            resolved,
                                            semantic_types,
                                        )
                                    })
                                    .collect(),
                            })
                            .collect(),
                        constraint_identities: constraints
                            .iter()
                            .map(|constraint| constraint_identity(resolved, constraint.name_span))
                            .collect(),
                        parameters,
                        semantic_parameters,
                        result: explicit_effect_value_type(
                            return_type,
                            requirements,
                            failure.as_ref(),
                            resolved,
                            semantic_types,
                        ),
                        semantic_result: SemanticTypeKey::Other,
                    },
                );
            }
            SurfaceDecl::Operator {
                spelling_span,
                type_parameters,
                parameters,
                return_type,
                constraints,
                ..
            } if !parameters.is_empty() => {
                let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::Operator && symbol.origin == *spelling_span
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
                        trait_identity: None,
                        trait_method: None,
                        type_parameters: type_parameters.clone(),
                        constraints: constraints
                            .iter()
                            .map(|constraint| crate::TypedConstraint {
                                name: constraint.name.clone(),
                                arguments: constraint
                                    .arguments
                                    .iter()
                                    .map(|argument| {
                                        semantic_typed_type_from_type_ref(
                                            argument,
                                            resolved,
                                            semantic_types,
                                        )
                                    })
                                    .collect(),
                            })
                            .collect(),
                        constraint_identities: constraints
                            .iter()
                            .map(|constraint| constraint_identity(resolved, constraint.name_span))
                            .collect(),
                        parameters: parameters
                            .iter()
                            .map(|parameter| {
                                semantic_typed_type_from_type_ref(
                                    &parameter.type_ref,
                                    resolved,
                                    semantic_types,
                                )
                            })
                            .collect(),
                        semantic_parameters: parameters
                            .iter()
                            .map(|parameter| {
                                semantic_types.key_from_type_ref(resolved, &parameter.type_ref)
                            })
                            .collect(),
                        result: semantic_typed_type_from_type_ref(
                            return_type,
                            resolved,
                            semantic_types,
                        ),
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
                type_parameters: signature
                    .type_parameters
                    .into_iter()
                    .map(seseragi_syntax::TypeParameter::value)
                    .collect(),
                constraints: Vec::new(),
                constraint_identities: Vec::new(),
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
        if symbol.namespace == SymbolNamespace::Operator {
            if let Some(operator) = seseragi_syntax::standard_trait_operator(&symbol.spelling) {
                callables.insert(symbol.id, standard_trait_operator_callable(operator));
                continue;
            }
        }
        if let Some(method) = symbol
            .canonical
            .as_deref()
            .and_then(crate::prelude::trait_method_by_canonical)
        {
            callables.insert(symbol.id, standard_trait_method_callable(method));
            continue;
        }
        match symbol.canonical.as_deref() {
            Some("std/prelude::reduce") => {
                callables.insert(symbol.id, standard_reduce_callable());
            }
            Some("std/prelude::join") => {
                callables.insert(symbol.id, standard_join_callable());
            }
            Some("std/prelude::unfold") => {
                callables.insert(symbol.id, standard_unfold_callable());
            }
            Some("std/prelude::next") => {
                callables.insert(symbol.id, standard_next_callable(resolved, semantic_types));
            }
            _ => {}
        }
    }
    callables
}

fn callable_parameter_types(
    parameters: &[seseragi_syntax::SurfaceParameter],
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
) -> (Vec<TypedType>, Vec<SemanticTypeKey>) {
    if parameters.is_empty() {
        return (
            vec![TypedType::Named {
                name: "Unit".to_owned(),
                arguments: Vec::new(),
            }],
            vec![SemanticTypeKey::Other],
        );
    }
    (
        parameters
            .iter()
            .map(|parameter| {
                semantic_typed_type_from_type_ref(&parameter.type_ref, resolved, semantic_types)
            })
            .collect(),
        parameters
            .iter()
            .map(|parameter| semantic_types.key_from_type_ref(resolved, &parameter.type_ref))
            .collect(),
    )
}

fn explicit_effect_value_type(
    return_type: &TypeRef,
    requirements: &[SurfaceRequirement],
    failure: Option<&TypeRef>,
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
) -> TypedType {
    let fields = requirements
        .iter()
        .map(|requirement| match requirement {
            SurfaceRequirement::Field { name, type_ref, .. } => crate::TypedRecordField {
                name: name.clone(),
                optional: false,
                type_ref: semantic_typed_type_from_type_ref(type_ref, resolved, semantic_types),
            },
            SurfaceRequirement::Shorthand { name, .. } => crate::TypedRecordField {
                name: lower_first(name),
                optional: false,
                type_ref: TypedType::Named {
                    name: name.clone(),
                    arguments: Vec::new(),
                },
            },
        })
        .collect();
    TypedType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            TypedType::Record {
                closed: true,
                fields,
            },
            failure.map_or_else(
                || TypedType::Named {
                    name: "Never".to_owned(),
                    arguments: Vec::new(),
                },
                |failure| semantic_typed_type_from_type_ref(failure, resolved, semantic_types),
            ),
            semantic_typed_type_from_type_ref(return_type, resolved, semantic_types),
        ],
    }
}

fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn semantic_typed_type_from_type_ref(
    type_ref: &TypeRef,
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
) -> TypedType {
    match type_ref {
        TypeRef::Named {
            name,
            arguments,
            span,
        } => {
            let arguments = arguments
                .iter()
                .map(|argument| {
                    semantic_typed_type_from_type_ref(argument, resolved, semantic_types)
                })
                .collect();
            match external_type_canonical(resolved, *span) {
                Some(canonical) => TypedType::ExternalNamed {
                    name: name.clone(),
                    canonical,
                    arguments,
                },
                _ => TypedType::Named {
                    name: name.clone(),
                    arguments,
                },
            }
        }
        TypeRef::Hole { .. } => TypedType::Hole,
        TypeRef::Record { closed, fields, .. } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| crate::TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: semantic_typed_type_from_type_ref(
                        &field.type_ref,
                        resolved,
                        semantic_types,
                    ),
                })
                .collect(),
        },
        TypeRef::Tuple { elements, .. } => TypedType::Tuple {
            elements: elements
                .iter()
                .map(|element| semantic_typed_type_from_type_ref(element, resolved, semantic_types))
                .collect(),
        },
        TypeRef::Function {
            parameter, result, ..
        } => TypedType::Function {
            parameter: Box::new(semantic_typed_type_from_type_ref(
                parameter,
                resolved,
                semantic_types,
            )),
            result: Box::new(semantic_typed_type_from_type_ref(
                result,
                resolved,
                semantic_types,
            )),
        },
    }
}

fn external_type_canonical(resolved: &ResolvedModule, origin: ByteSpan) -> Option<String> {
    let target = resolved
        .references
        .iter()
        .find(|reference| {
            reference.namespace == SymbolNamespace::Type && reference.origin == origin
        })?
        .target?;
    let symbol = resolved.symbols.iter().find(|symbol| symbol.id == target)?;
    let canonical = symbol.canonical.as_ref()?;
    resolved
        .imports
        .iter()
        .any(|import| {
            import.symbol == target
                && matches!(
                    import.export.declaration_kind.as_deref(),
                    Some("opaque-type" | "opaque-struct")
                )
        })
        .then(|| canonical.clone())
}

fn standard_trait_method_callable(
    method: &crate::prelude::PreludeTraitMethod,
) -> TopLevelPureFunction {
    let trait_spec = crate::prelude::trait_by_name(method.trait_name)
        .expect("Prelude trait method owner must exist");
    let signature = crate::prelude::trait_method_signature(method);
    TopLevelPureFunction {
        symbol: method.canonical.to_owned(),
        trait_identity: Some(trait_spec.canonical.to_owned()),
        trait_method: Some(method.name.to_owned()),
        type_parameters: signature.type_parameters,
        constraints: vec![crate::TypedConstraint {
            name: trait_spec.name.to_owned(),
            arguments: vec![TypedType::Named {
                name: trait_spec.type_parameter.to_owned(),
                arguments: Vec::new(),
            }],
        }],
        constraint_identities: vec![Some(trait_spec.canonical.to_owned())],
        semantic_parameters: vec![SemanticTypeKey::Other; signature.parameters.len()],
        parameters: signature.parameters,
        result: signature.result,
        semantic_result: SemanticTypeKey::Other,
    }
}

fn standard_trait_operator_callable(
    operator: &seseragi_syntax::StandardTraitOperator,
) -> TopLevelPureFunction {
    let method = crate::prelude::trait_method(operator.trait_name, operator.method_name)
        .expect("standard trait operator method must exist in Prelude");
    let mut callable = standard_trait_method_callable(method);
    let method_parameters = std::mem::take(&mut callable.parameters);
    let method_semantic_parameters = std::mem::take(&mut callable.semantic_parameters);
    assert_eq!(
        method_parameters.len(),
        operator.method_operand_sources.len()
    );
    let mut source_parameters = [None, None];
    let mut source_semantic_parameters = [None, None];
    for (method_index, source_index) in operator.method_operand_sources.into_iter().enumerate() {
        source_parameters[source_index] = Some(method_parameters[method_index].clone());
        source_semantic_parameters[source_index] =
            Some(method_semantic_parameters[method_index].clone());
    }
    callable.symbol = operator.spelling.to_owned();
    callable.parameters = source_parameters
        .into_iter()
        .map(|parameter| parameter.expect("trait operator order must be a permutation"))
        .collect();
    callable.semantic_parameters = source_semantic_parameters
        .into_iter()
        .map(|parameter| parameter.expect("trait operator order must be a permutation"))
        .collect();
    callable
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
        type_parameters: vec![
            seseragi_syntax::TypeParameter::value("C"),
            seseragi_syntax::TypeParameter::value("A"),
            seseragi_syntax::TypeParameter::value("B"),
        ],
        constraints: vec![crate::TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![collection.clone(), element.clone()],
        }],
        constraint_identities: vec![None],
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

fn standard_join_callable() -> TopLevelPureFunction {
    let collection = named_type("C");
    let string = named_type("String");
    TopLevelPureFunction {
        symbol: "std/prelude::join".to_owned(),
        trait_identity: None,
        trait_method: None,
        type_parameters: vec![seseragi_syntax::TypeParameter::value("C")],
        constraints: vec![crate::TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![collection.clone(), string.clone()],
        }],
        constraint_identities: vec![None],
        parameters: vec![string.clone(), collection],
        semantic_parameters: vec![SemanticTypeKey::Other; 2],
        result: string,
        semantic_result: SemanticTypeKey::Other,
    }
}

fn standard_unfold_callable() -> TopLevelPureFunction {
    let state = named_type("S");
    let element = named_type("A");
    let maybe_step = named_type_with(
        "Maybe",
        vec![TypedType::Tuple {
            elements: vec![element.clone(), state.clone()],
        }],
    );
    let result = named_type_with("Iterator", vec![element.clone()]);
    let semantic_state = scheme_value("S");
    let semantic_element = scheme_value("A");
    let semantic_result = iterator_semantic(semantic_element);
    TopLevelPureFunction {
        symbol: "std/prelude::unfold".to_owned(),
        trait_identity: None,
        trait_method: None,
        type_parameters: vec![
            seseragi_syntax::TypeParameter::value("S"),
            seseragi_syntax::TypeParameter::value("A"),
        ],
        constraints: Vec::new(),
        constraint_identities: Vec::new(),
        parameters: vec![
            TypedType::Function {
                parameter: Box::new(state.clone()),
                result: Box::new(maybe_step),
            },
            state,
        ],
        semantic_parameters: vec![SemanticTypeKey::Other, semantic_state.key],
        semantic_result: semantic_result.key,
        result,
    }
}

fn standard_next_callable(
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
) -> TopLevelPureFunction {
    let element = named_type("A");
    let iterator = named_type_with("Iterator", vec![element.clone()]);
    let semantic_element = scheme_value("A");
    let semantic_iterator = iterator_semantic(semantic_element.clone());
    let semantic_payload = SemanticValueType {
        type_ref: TypedType::Tuple {
            elements: vec![element.clone(), iterator.clone()],
        },
        key: SemanticTypeKey::Tuple(vec![
            semantic_element.key.clone(),
            semantic_iterator.key.clone(),
        ]),
    };
    let result = named_type_with(
        "Maybe",
        vec![TypedType::Tuple {
            elements: vec![element, iterator.clone()],
        }],
    );
    let semantic_result = prelude_adt_semantic(resolved, semantic_types, &result, semantic_payload);
    TopLevelPureFunction {
        symbol: "std/prelude::next".to_owned(),
        trait_identity: None,
        trait_method: None,
        type_parameters: vec![seseragi_syntax::TypeParameter::value("A")],
        constraints: Vec::new(),
        constraint_identities: Vec::new(),
        parameters: vec![iterator.clone()],
        semantic_parameters: vec![semantic_iterator.key],
        semantic_result: semantic_result.key,
        result,
    }
}

fn named_type(name: &str) -> TypedType {
    named_type_with(name, Vec::new())
}

fn named_type_with(name: &str, arguments: Vec<TypedType>) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments,
    }
}

fn scheme_value(name: &str) -> SemanticValueType {
    SemanticValueType {
        type_ref: named_type(name),
        key: SemanticTypeKey::SchemeParameter(name.to_owned()),
    }
}

fn iterator_semantic(element: SemanticValueType) -> SemanticValueType {
    SemanticValueType {
        type_ref: named_type_with("Iterator", vec![element.type_ref.clone()]),
        key: SemanticTypeKey::ExternalNominal {
            canonical: "std/prelude::Iterator".to_owned(),
            arguments: vec![element],
        },
    }
}

fn prelude_adt_semantic(
    resolved: &ResolvedModule,
    semantic_types: &SemanticTypeCatalog,
    type_ref: &TypedType,
    argument: SemanticValueType,
) -> SemanticValueType {
    let SemanticTypeKey::Adt { owner, .. } = semantic_types.key_from_typed_type(resolved, type_ref)
    else {
        return SemanticValueType {
            type_ref: type_ref.clone(),
            key: SemanticTypeKey::Other,
        };
    };
    SemanticValueType {
        type_ref: type_ref.clone(),
        key: SemanticTypeKey::Adt {
            owner,
            arguments: vec![argument],
        },
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
            let mut constraints = vec![crate::TypedConstraint {
                name: name.clone(),
                arguments: type_parameters
                    .iter()
                    .map(|parameter| TypedType::Named {
                        name: parameter.name.clone(),
                        arguments: Vec::new(),
                    })
                    .collect(),
            }];
            constraints.extend(method.constraints.iter().map(|constraint| {
                crate::TypedConstraint {
                    name: constraint.name.clone(),
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(typed_type_from_type_ref)
                        .collect(),
                }
            }));
            let mut constraint_identities = vec![Some(trait_identity.clone())];
            constraint_identities.extend(
                method
                    .constraints
                    .iter()
                    .map(|constraint| constraint_identity(resolved, constraint.name_span)),
            );
            callables.insert(
                symbol.id,
                TopLevelPureFunction {
                    symbol: canonical,
                    trait_identity: Some(trait_identity.clone()),
                    trait_method: Some(method.name.clone()),
                    type_parameters: callable_type_parameters,
                    constraints,
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
                },
            );
        }
    }
}

fn constraint_identity(resolved: &ResolvedModule, origin: ByteSpan) -> Option<String> {
    let target = resolved
        .references
        .iter()
        .find(|reference| {
            reference.namespace == SymbolNamespace::Trait && reference.origin == origin
        })?
        .target?;
    resolved
        .symbols
        .iter()
        .find(|symbol| symbol.id == target)?
        .canonical
        .clone()
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
            SurfaceDecl::EffectFn { parameters, .. } => {
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
            SurfaceDecl::Impl { members, .. } => {
                for member in members {
                    let SurfaceImplMember::Method { method, .. } = member else {
                        continue;
                    };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_generic_constrained_and_higher_order_functions() {
        let resolved = crate::resolve_module(
            "artifact/functions/main.ssrg",
            "fn identity<A> value: A -> A = value\nfn constrained<A> value: A -> A\nwhere Eq<A> = value\nfn apply f: (Int -> Int) -> value: Int -> Int = f value\n",
        );

        let semantic_types = SemanticTypeCatalog::new(&resolved);
        let callables = collect_callables(&resolved, &semantic_types);
        assert_eq!(callables.len(), 3);
        assert!(callables
            .values()
            .any(|callable| callable.symbol == "artifact/functions::identity"
                && callable.type_parameters == ["A"]));
        assert!(callables
            .values()
            .any(|callable| callable.symbol == "artifact/functions::apply"
                && callable.parameters.len() == 2));
        assert!(callables.values().any(|callable| {
            callable.symbol == "artifact/functions::constrained"
                && matches!(callable.constraints.as_slice(), [constraint]
                    if constraint.name == "Eq")
        }));
    }
}
