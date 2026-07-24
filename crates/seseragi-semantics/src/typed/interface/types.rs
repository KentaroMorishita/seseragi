use crate::{ExternalTypeBinding, TypedParameter, TypedRecordField, TypedType};
use seseragi_syntax::{InterfaceRecordField, InterfaceType};

pub(super) struct InterfaceTypes<'a> {
    bindings: &'a [ExternalTypeBinding],
}

impl<'a> InterfaceTypes<'a> {
    pub(super) fn new(bindings: &'a [ExternalTypeBinding]) -> Self {
        Self { bindings }
    }

    pub(super) fn convert(&self, type_ref: &TypedType) -> InterfaceType {
        match type_ref {
            TypedType::Named { name, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.convert(argument))
                    .collect();
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
                let provider = self
                    .bindings
                    .iter()
                    .find(|binding| binding.canonical == *canonical)
                    .and_then(|binding| binding.provider.as_ref())
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
    fn canonicalizes_a_named_type_from_its_unique_provider_binding() {
        let bindings = [ExternalTypeBinding {
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
