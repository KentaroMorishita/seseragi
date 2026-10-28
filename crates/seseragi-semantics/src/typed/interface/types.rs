use crate::{ExternalTypeBinding, TypedParameter, TypedRecordField, TypedType};
use seseragi_syntax::{InterfaceRecordField, InterfaceType};

#[derive(Clone)]
pub(super) struct InterfaceTypes<'a> {
    bindings: &'a [ExternalTypeBinding],
    parameters: std::collections::BTreeSet<String>,
    local_nominals: std::collections::BTreeMap<String, (String, String)>,
    nominal_owners: std::collections::BTreeMap<String, (String, String)>,
}

impl<'a> InterfaceTypes<'a> {
    pub(super) fn new(bindings: &'a [ExternalTypeBinding]) -> Self {
        Self {
            bindings,
            parameters: Default::default(),
            local_nominals: Default::default(),
            nominal_owners: Default::default(),
        }
    }

    pub(super) fn with_local_nominals(mut self, module: &crate::TypedModule) -> Self {
        for declaration in &module.declarations {
            if let crate::TypedDecl::Struct { symbol, name, .. }
            | crate::TypedDecl::Adt { symbol, name, .. } = declaration
            {
                if crate::prelude::type_constructor_arity(name).is_some()
                    && symbol != &format!("std/prelude::{name}")
                    && !crate::prelude::SUM_TYPES
                        .iter()
                        .any(|sum| sum.name == *name && sum.canonical == *symbol)
                {
                    self.local_nominals
                        .insert(symbol.clone(), (module.module.clone(), name.clone()));
                }
            }
        }
        fn collect(
            members: &[crate::TypedForeignMember],
            module: &str,
            out: &mut std::collections::BTreeMap<String, (String, String)>,
        ) {
            for member in members {
                match member {
                    crate::TypedForeignMember::OpaqueType { symbol, name, .. }
                        if crate::prelude::type_constructor_arity(name).is_some() =>
                    {
                        out.insert(symbol.clone(), (module.into(), name.clone()));
                    }
                    crate::TypedForeignMember::Namespace { members, .. } => {
                        collect(members, module, out)
                    }
                    _ => {}
                }
            }
        }
        for foreign in &module.foreign_modules {
            collect(&foreign.members, &module.module, &mut self.local_nominals);
        }
        self
    }

    pub(super) fn with_parameters(&self, parameters: &[seseragi_syntax::TypeParameter]) -> Self {
        let mut scoped = self.clone();
        scoped
            .parameters
            .extend(parameters.iter().map(|parameter| parameter.name.clone()));
        scoped
    }

    /// Public declaration identity must travel with aliases and callable
    /// signatures, even when a facade exports only an alias for that type.
    /// The provider is the checked public declaration, never its display name
    /// reconstructed in the consumer or a private representation.
    pub(super) fn with_public_owners(mut self, module: &crate::TypedModule) -> Self {
        for declaration in &module.declarations {
            if let crate::TypedDecl::Struct {
                symbol,
                name,
                visibility: seseragi_syntax::Visibility::Public,
                ..
            }
            | crate::TypedDecl::Adt {
                symbol,
                name,
                visibility: seseragi_syntax::Visibility::Public,
                ..
            } = declaration
            {
                self.nominal_owners
                    .insert(name.clone(), (symbol.clone(), module.module.clone()));
            }
        }
        self
    }

    /// An instance can travel through a facade that renames its nominal head.
    /// Retain the declaration owner in the dictionary interface itself instead
    /// of requiring the consumer to import the provider's original spelling.
    pub(super) fn with_instance_owners(mut self, module: &crate::TypedModule) -> Self {
        for declaration in &module.declarations {
            if let crate::TypedDecl::Struct { symbol, name, .. }
            | crate::TypedDecl::Adt { symbol, name, .. } = declaration
            {
                self.nominal_owners
                    .insert(name.clone(), (symbol.clone(), module.module.clone()));
            }
        }
        self
    }

    pub(super) fn convert(&self, type_ref: &TypedType) -> InterfaceType {
        match type_ref {
            TypedType::Named { name, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.convert(argument))
                    .collect();
                if self.parameters.contains(name) {
                    return InterfaceType::Named {
                        name: name.clone(),
                        arguments,
                    };
                }
                let primitive_or_collection = matches!(
                    name.as_str(),
                    "Unit" | "Bool" | "Int" | "Float" | "Char" | "String" | "Array" | "List"
                );
                let collision = self.local_nominals.values().any(|(_, local)| local == name)
                    || self.bindings.iter().any(|binding| {
                        binding.spelling == *name
                            && binding.canonical != format!("std/prelude::{name}")
                    });
                if primitive_or_collection && collision && !self.parameters.contains(name) {
                    return InterfaceType::ExternalNamed {
                        name: name.clone(),
                        canonical: format!("std/prelude::{name}"),
                        provider_module: "std/prelude".into(),
                        provider_export: name.clone(),
                        arguments,
                    };
                }
                if !self.parameters.contains(name) && !primitive_or_collection {
                    if let Some((canonical, provider)) = self.nominal_owners.get(name) {
                        return InterfaceType::ExternalNamed {
                            name: name.clone(),
                            canonical: canonical.clone(),
                            provider_module: provider.clone(),
                            provider_export: name.clone(),
                            arguments,
                        };
                    }
                }
                match self.unique_provider_binding(name) {
                    Some(binding) => {
                        let provider = binding
                            .provider
                            .as_ref()
                            .expect("provider binding was filtered above");
                        InterfaceType::ExternalNamed {
                            name: name.clone(),
                            canonical: binding.canonical.clone(),
                            provider_module: provider.module.clone(),
                            provider_export: provider.export.clone(),
                            arguments,
                        }
                    }
                    None => InterfaceType::Named {
                        name: name.clone(),
                        arguments,
                    },
                }
            }
            TypedType::ExternalNamed {
                name,
                canonical,
                arguments,
            } => {
                // Anonymous parameters and closed aliases can introduce a
                // checked primitive without a source import. Its canonical
                // owner is its provenance; same-named local nominals still
                // follow the owner table below.
                if let Some(primitive) = canonical.strip_prefix("std/prelude::").filter(|name| {
                    arguments.is_empty()
                        && matches!(*name, "Unit" | "Bool" | "Int" | "Float" | "Char" | "String")
                }) {
                    return InterfaceType::ExternalNamed {
                        name: name.clone(),
                        canonical: canonical.clone(),
                        provider_module: "std/prelude".to_owned(),
                        provider_export: primitive.to_owned(),
                        arguments: Vec::new(),
                    };
                }
                if let Some((provider_module, provider_export)) = self.local_nominals.get(canonical)
                {
                    return InterfaceType::ExternalNamed {
                        name: name.clone(),
                        canonical: canonical.clone(),
                        provider_module: provider_module.clone(),
                        provider_export: provider_export.clone(),
                        arguments: arguments
                            .iter()
                            .map(|argument| self.convert(argument))
                            .collect(),
                    };
                }
                let provider = self
                    .bindings
                    .iter()
                    .filter(|binding| binding.canonical == *canonical)
                    .find_map(|binding| binding.provider.as_ref())
                    .expect("external typed nominal has provider provenance");
                InterfaceType::ExternalNamed {
                    name: name.clone(),
                    canonical: canonical.clone(),
                    provider_module: provider.module.clone(),
                    provider_export: provider.export.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| self.convert(argument))
                        .collect(),
                }
            }
            TypedType::Hole => InterfaceType::Hole,
            TypedType::Record { closed, fields } => InterfaceType::Record {
                closed: *closed,
                fields: fields
                    .iter()
                    .map(|field| self.record_field(field))
                    .collect(),
            },
            TypedType::Tuple { elements } => InterfaceType::Tuple {
                elements: elements
                    .iter()
                    .map(|element| self.convert(element))
                    .collect(),
            },
            TypedType::Function { parameter, result } => InterfaceType::Function {
                parameter: Box::new(self.convert(parameter)),
                result: Box::new(self.convert(result)),
            },
            TypedType::RequirementMerge { operands } => InterfaceType::RequirementMerge {
                operands: operands
                    .iter()
                    .map(|operand| self.convert(operand))
                    .collect(),
            },
        }
    }

    pub(super) fn parameter(&self, parameter: &TypedParameter) -> InterfaceType {
        match parameter {
            TypedParameter::ImplicitUnit { type_ref } | TypedParameter::Named { type_ref, .. } => {
                self.convert(type_ref)
            }
        }
    }

    fn record_field(&self, field: &TypedRecordField) -> InterfaceRecordField {
        InterfaceRecordField {
            name: field.name.clone(),
            optional: field.optional,
            type_ref: self.convert(&field.type_ref),
        }
    }

    fn unique_provider_binding(&self, spelling: &str) -> Option<&ExternalTypeBinding> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|binding| binding.spelling == spelling && binding.provider.is_some());
        let first = bindings.next()?;
        bindings
            .all(|binding| binding.canonical == first.canonical)
            .then_some(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExternalTypeProvider;

    #[test]
    fn closed_primitive_aliases_keep_canonical_provider_provenance() {
        let types = InterfaceTypes::new(&[]);
        for primitive in ["Unit", "Bool", "Int", "Float", "Char", "String"] {
            let canonical = format!("std/prelude::{primitive}");
            assert_eq!(
                types.convert(&TypedType::ExternalNamed {
                    name: "Renamed".to_owned(),
                    canonical: canonical.clone(),
                    arguments: vec![],
                }),
                InterfaceType::ExternalNamed {
                    name: "Renamed".to_owned(),
                    canonical,
                    provider_module: "std/prelude".to_owned(),
                    provider_export: primitive.to_owned(),
                    arguments: vec![],
                }
            );
        }
    }

    #[test]
    fn canonicalizes_a_named_type_from_its_unique_provider_binding() {
        let bindings = [ExternalTypeBinding {
            arity: None,
            spelling: "html.Html".to_owned(),
            canonical: "std/web/html::Html".to_owned(),
            provider: Some(ExternalTypeProvider {
                module: "std/web/html".to_owned(),
                export: "Html".to_owned(),
            }),
        }];
        let types = InterfaceTypes::new(&bindings);

        assert_eq!(
            types.convert(&TypedType::Named {
                name: "html.Html".to_owned(),
                arguments: vec![TypedType::Named {
                    name: "Action".to_owned(),
                    arguments: Vec::new(),
                }],
            }),
            InterfaceType::ExternalNamed {
                name: "html.Html".to_owned(),
                canonical: "std/web/html::Html".to_owned(),
                provider_module: "std/web/html".to_owned(),
                provider_export: "Html".to_owned(),
                arguments: vec![InterfaceType::Named {
                    name: "Action".to_owned(),
                    arguments: Vec::new(),
                }],
            }
        );
    }
}
