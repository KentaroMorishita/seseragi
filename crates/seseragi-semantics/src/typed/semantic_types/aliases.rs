use crate::{
    ExternalTypeBinding, ResolvedModule, SymbolId, SymbolKind, SymbolNamespace, TypedType,
};
use seseragi_syntax::{InterfaceType, SurfaceDecl, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
enum AliasTarget {
    Local(TypeRef),
    Imported {
        type_ref: InterfaceType,
        bindings: Vec<ExternalTypeBinding>,
    },
}

#[derive(Clone, Debug)]
struct AliasDefinition {
    parameter_symbols: Vec<SymbolId>,
    parameter_names: Vec<String>,
    target: AliasTarget,
}

#[derive(Clone, Debug, Default)]
pub(super) struct AliasCatalog {
    definitions: BTreeMap<SymbolId, AliasDefinition>,
}

impl AliasCatalog {
    pub(super) fn new(resolved: &ResolvedModule) -> Self {
        let mut definitions = BTreeMap::new();
        for declaration in &resolved.declarations {
            let SurfaceDecl::Alias {
                name_span,
                type_parameters,
                target,
                span,
                ..
            } = declaration
            else {
                continue;
            };
            let Some(owner) = resolved
                .symbols
                .iter()
                .find(|symbol| symbol.kind == SymbolKind::Type && symbol.origin == *name_span)
            else {
                continue;
            };
            let parameter_symbols = resolved
                .scopes
                .iter()
                .find(|scope| scope.kind == crate::ScopeKind::Declaration && scope.origin == *span)
                .map(|scope| {
                    resolved
                        .symbols
                        .iter()
                        .filter(|symbol| {
                            symbol.kind == SymbolKind::TypeParameter && symbol.scope == scope.id
                        })
                        .map(|symbol| symbol.id)
                        .take(type_parameters.len())
                        .collect()
                })
                .unwrap_or_default();
            definitions.insert(
                owner.id,
                AliasDefinition {
                    parameter_symbols,
                    parameter_names: type_parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    target: AliasTarget::Local(target.clone()),
                },
            );
        }
        for import in resolved.imports.iter().filter(|import| {
            import.export.namespace == "type"
                && import.export.declaration_kind.as_deref() == Some("alias")
        }) {
            let Some(type_ref) = import.export.representation.clone() else {
                continue;
            };
            definitions
                .entry(import.symbol)
                .or_insert_with(|| AliasDefinition {
                    parameter_symbols: Vec::new(),
                    parameter_names: import
                        .export
                        .scheme
                        .type_parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    target: AliasTarget::Imported {
                        type_ref,
                        bindings: import.scheme_type_bindings.clone().unwrap_or_default(),
                    },
                });
        }
        Self { definitions }
    }

    pub(super) fn expands(&self, resolved: &ResolvedModule, type_ref: &TypeRef) -> bool {
        let TypeRef::Named { span, .. } = type_ref else {
            return false;
        };
        self.target(resolved, *span).is_some_and(|target| {
            self.definitions.contains_key(&target)
                || canonical(resolved, target).as_deref() == Some("std/prelude::Task")
        })
    }

    pub(super) fn expand(&self, resolved: &ResolvedModule, type_ref: &TypeRef) -> TypedType {
        self.expand_type_ref(resolved, type_ref, &BTreeMap::new(), &mut BTreeSet::new())
    }

    fn expand_type_ref(
        &self,
        resolved: &ResolvedModule,
        type_ref: &TypeRef,
        substitutions: &BTreeMap<SymbolId, TypedType>,
        stack: &mut BTreeSet<SymbolId>,
    ) -> TypedType {
        match type_ref {
            TypeRef::Named {
                name,
                arguments,
                span,
            } => {
                let target = self.target(resolved, *span);
                if arguments.is_empty() {
                    if let Some(substitution) = target.and_then(|target| substitutions.get(&target))
                    {
                        return substitution.clone();
                    }
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expand_type_ref(resolved, argument, substitutions, stack))
                    .collect::<Vec<_>>();
                if target
                    .and_then(|target| canonical(resolved, target))
                    .as_deref()
                    == Some("std/prelude::Task")
                {
                    return task_type(arguments);
                }
                if let Some(target) = target {
                    if let Some(definition) = self.definitions.get(&target) {
                        return self.instantiate(resolved, target, definition, arguments, stack);
                    }
                }
                match target.and_then(|target| external_canonical(resolved, target)) {
                    Some(canonical) => TypedType::ExternalNamed {
                        name: name.clone(),
                        canonical,
                        arguments,
                    },
                    None => TypedType::Named {
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
                        type_ref: self.expand_type_ref(
                            resolved,
                            &field.type_ref,
                            substitutions,
                            stack,
                        ),
                    })
                    .collect(),
            },
            TypeRef::Tuple { elements, .. } => TypedType::Tuple {
                elements: elements
                    .iter()
                    .map(|element| self.expand_type_ref(resolved, element, substitutions, stack))
                    .collect(),
            },
            TypeRef::Function {
                parameter, result, ..
            } => TypedType::Function {
                parameter: Box::new(self.expand_type_ref(
                    resolved,
                    parameter,
                    substitutions,
                    stack,
                )),
                result: Box::new(self.expand_type_ref(resolved, result, substitutions, stack)),
            },
        }
    }

    fn instantiate(
        &self,
        resolved: &ResolvedModule,
        owner: SymbolId,
        definition: &AliasDefinition,
        arguments: Vec<TypedType>,
        stack: &mut BTreeSet<SymbolId>,
    ) -> TypedType {
        if arguments.len() != definition.parameter_names.len() || !stack.insert(owner) {
            return TypedType::Hole;
        }
        let expanded = match &definition.target {
            AliasTarget::Local(target) => {
                let substitutions = definition
                    .parameter_symbols
                    .iter()
                    .copied()
                    .zip(arguments)
                    .collect::<BTreeMap<_, _>>();
                self.expand_type_ref(resolved, target, &substitutions, stack)
            }
            AliasTarget::Imported { type_ref, bindings } => {
                let substitutions = definition
                    .parameter_names
                    .iter()
                    .cloned()
                    .zip(arguments)
                    .collect::<BTreeMap<_, _>>();
                expand_interface_type(type_ref, &substitutions, bindings)
            }
        };
        stack.remove(&owner);
        expanded
    }

    fn target(
        &self,
        resolved: &ResolvedModule,
        origin: seseragi_syntax::ByteSpan,
    ) -> Option<SymbolId> {
        resolved
            .references
            .iter()
            .find(|reference| {
                reference.namespace == SymbolNamespace::Type && reference.origin == origin
            })
            .and_then(|reference| reference.target)
    }
}

fn expand_interface_type(
    type_ref: &InterfaceType,
    substitutions: &BTreeMap<String, TypedType>,
    bindings: &[ExternalTypeBinding],
) -> TypedType {
    match type_ref {
        InterfaceType::Named { name, arguments } => {
            if arguments.is_empty() {
                if let Some(substitution) = substitutions.get(name) {
                    return substitution.clone();
                }
            }
            let arguments = arguments
                .iter()
                .map(|argument| expand_interface_type(argument, substitutions, bindings))
                .collect();
            match bindings.iter().find(|binding| binding.spelling == *name) {
                Some(binding) => TypedType::ExternalNamed {
                    name: name.clone(),
                    canonical: binding.canonical.clone(),
                    arguments,
                },
                None => TypedType::Named {
                    name: name.clone(),
                    arguments,
                },
            }
        }
        InterfaceType::ExternalNamed {
            name,
            canonical,
            arguments,
            ..
        } => TypedType::ExternalNamed {
            name: name.clone(),
            canonical: canonical.clone(),
            arguments: arguments
                .iter()
                .map(|argument| expand_interface_type(argument, substitutions, bindings))
                .collect(),
        },
        InterfaceType::Record { closed, fields } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| crate::TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: expand_interface_type(&field.type_ref, substitutions, bindings),
                })
                .collect(),
        },
        InterfaceType::Tuple { elements } => TypedType::Tuple {
            elements: elements
                .iter()
                .map(|element| expand_interface_type(element, substitutions, bindings))
                .collect(),
        },
        InterfaceType::Function { parameter, result } => TypedType::Function {
            parameter: Box::new(expand_interface_type(parameter, substitutions, bindings)),
            result: Box::new(expand_interface_type(result, substitutions, bindings)),
        },
        InterfaceType::Apply {
            constructor,
            arguments,
        } => TypedType::Named {
            name: constructor.clone(),
            arguments: arguments
                .iter()
                .map(|argument| expand_interface_type(argument, substitutions, bindings))
                .collect(),
        },
        InterfaceType::Hole | InterfaceType::TypeConstructor { .. } => TypedType::Hole,
    }
}

fn task_type(mut arguments: Vec<TypedType>) -> TypedType {
    if arguments.len() != 1 {
        return TypedType::Hole;
    }
    TypedType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            TypedType::Record {
                closed: true,
                fields: Vec::new(),
            },
            TypedType::Named {
                name: "Never".to_owned(),
                arguments: Vec::new(),
            },
            arguments.remove(0),
        ],
    }
}

fn canonical(resolved: &ResolvedModule, symbol: SymbolId) -> Option<String> {
    resolved
        .symbols
        .iter()
        .find(|candidate| candidate.id == symbol)
        .and_then(|candidate| candidate.canonical.clone())
}

fn external_canonical(resolved: &ResolvedModule, symbol: SymbolId) -> Option<String> {
    let canonical = canonical(resolved, symbol)?;
    resolved
        .imports
        .iter()
        .any(|import| {
            import.symbol == symbol
                && matches!(
                    import.export.declaration_kind.as_deref(),
                    Some("opaque-type" | "opaque-struct")
                )
        })
        .then_some(canonical)
}
