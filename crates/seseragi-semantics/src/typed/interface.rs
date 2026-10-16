use crate::{
    ResolvedDependencyInstance, TypedDecl, TypedEffect, TypedInstance, TypedModule,
    TypedModuleInterface, TypedParameter, TypedScheme,
};
use seseragi_syntax::{
    InterfaceConstraint, InterfaceExport, InterfaceInstance, InterfaceRecordField, InterfaceScheme,
    InterfaceType, ModuleInterface, Visibility,
};

mod types;

use types::InterfaceTypes;

/// Project the canonical Prelude signature through the same representation as
/// imported trait contracts. Trait parameters are substituted by the instance;
/// only the remaining method parameters are independently quantified.
pub(super) fn prelude_method_interface(
    method: &crate::prelude::PreludeTraitMethod,
    origin: seseragi_syntax::ByteSpan,
) -> seseragi_syntax::InterfaceMethod {
    let signature = crate::prelude::trait_method_signature(method);
    let trait_spec = crate::prelude::trait_by_name(method.trait_name)
        .expect("Prelude method has a registered trait");
    // Registry signatures have canonical external nominals even when the user
    // does not import them. Their provider cannot depend on local aliases.
    fn collect_providers(
        type_ref: &crate::TypedType,
        bindings: &mut Vec<crate::ExternalTypeBinding>,
    ) {
        use crate::TypedType;
        match type_ref {
            TypedType::ExternalNamed {
                name,
                canonical,
                arguments,
            } => {
                let (module, export) = canonical
                    .rsplit_once("::")
                    .expect("registered external nominal has a canonical owner");
                bindings.push(crate::ExternalTypeBinding {
                    arity: None,
                    spelling: name.clone(),
                    canonical: canonical.clone(),
                    provider: Some(crate::ExternalTypeProvider {
                        module: module.into(),
                        export: export.into(),
                    }),
                });
                for argument in arguments {
                    collect_providers(argument, bindings);
                }
            }
            TypedType::Named { arguments, .. }
            | TypedType::Tuple {
                elements: arguments,
            }
            | TypedType::RequirementMerge {
                operands: arguments,
            } => {
                for argument in arguments {
                    collect_providers(argument, bindings);
                }
            }
            TypedType::Function { parameter, result } => {
                collect_providers(parameter, bindings);
                collect_providers(result, bindings);
            }
            TypedType::Record { fields, .. } => {
                for field in fields {
                    collect_providers(&field.type_ref, bindings);
                }
            }
            TypedType::Hole => {}
        }
    }
    let type_ref =
        signature
            .parameters
            .into_iter()
            .rev()
            .fold(signature.result, |result, parameter| {
                crate::TypedType::Function {
                    parameter: Box::new(parameter),
                    result: Box::new(result),
                }
            });
    let mut bindings = Vec::new();
    collect_providers(&type_ref, &mut bindings);
    for argument in signature
        .constraints
        .iter()
        .flat_map(|constraint| &constraint.arguments)
    {
        collect_providers(argument, &mut bindings);
    }
    let mut scheme = interface_scheme_from_typed_scheme(
        &TypedScheme {
            type_parameters: signature.type_parameters,
            constraint_identities: signature
                .constraints
                .iter()
                .map(|constraint| {
                    crate::prelude::trait_by_name(&constraint.name)
                        .map(|trait_spec| trait_spec.canonical.to_owned())
                })
                .collect(),
            constraints: signature.constraints,
            type_ref,
        },
        &InterfaceTypes::new(&bindings),
    );
    scheme
        .type_parameters
        .drain(..trait_spec.type_parameters.len());
    seseragi_syntax::InterfaceMethod {
        name: method.name.to_owned(),
        scheme,
        origin,
    }
}

#[cfg(test)]
mod prelude_contract_tests {
    #[test]
    fn projects_every_registered_method_without_user_imports() {
        for method in crate::prelude::TRAIT_METHODS {
            let signature = crate::prelude::trait_method_signature(method);
            let projected = super::prelude_method_interface(
                method,
                seseragi_syntax::ByteSpan { start: 0, end: 0 },
            );
            let trait_parameters = crate::prelude::trait_by_name(method.trait_name)
                .unwrap()
                .type_parameters
                .len();
            assert_eq!(projected.name, method.name);
            assert_eq!(
                projected.scheme.type_parameters,
                signature.type_parameters[trait_parameters..]
            );
        }
    }
}

pub(crate) fn typed_interface_from_modules(
    shallow: ModuleInterface,
    typed: &TypedModule,
    dependency_instances: &[ResolvedDependencyInstance],
) -> TypedModuleInterface {
    let types = InterfaceTypes::new(&typed.external_type_bindings).with_local_nominals(typed);
    let local_symbol_prefix = format!("{}::", shallow.module);
    let mut exports = shallow
        .exports
        .into_iter()
        .filter(|export| {
            export.namespace != "value"
                || export.declaration_kind.as_deref() == Some("constructor")
                || !export.symbol.starts_with(&local_symbol_prefix)
        })
        .collect::<Vec<_>>();
    for declaration in &typed.declarations {
        let parameters = match declaration {
            TypedDecl::Adt {
                type_parameters, ..
            }
            | TypedDecl::Struct {
                type_parameters, ..
            }
            | TypedDecl::Alias {
                type_parameters, ..
            } => type_parameters.as_slice(),
            _ => &[],
        };
        let types = types.with_parameters(parameters);
        match declaration {
            TypedDecl::Alias {
                symbol,
                visibility: Visibility::Public,
                target,
                ..
            } => {
                if let Some(export) = exports.iter_mut().find(|export| {
                    export.namespace == "type"
                        && export.declaration_kind.as_deref() == Some("alias")
                        && export.symbol == *symbol
                }) {
                    export.representation = Some(types.convert(target));
                }
            }
            TypedDecl::Struct {
                symbol,
                visibility: Visibility::Public,
                opaque: false,
                fields,
                ..
            } => {
                if let Some(export) = exports.iter_mut().find(|export| {
                    export.namespace == "type"
                        && export.declaration_kind.as_deref() == Some("struct")
                        && export.symbol == *symbol
                }) {
                    export.representation = Some(InterfaceType::Record {
                        closed: true,
                        fields: fields
                            .iter()
                            .map(|field| InterfaceRecordField {
                                name: field.name.clone(),
                                optional: false,
                                type_ref: types.convert(&field.type_ref),
                            })
                            .collect(),
                    });
                }
            }
            _ => {}
        }
    }
    exports.extend(
        typed
            .declarations
            .iter()
            .flat_map(|declaration| typed_value_exports(declaration, &typed.module, &types))
            .collect::<Vec<_>>(),
    );

    fn mark(members: &[crate::TypedForeignMember], exports: &mut [InterfaceExport]) {
        for member in members {
            match member {
                crate::TypedForeignMember::Function {
                    type_parameters,
                    symbol,
                    ..
                } if !type_parameters.is_empty() => {
                    for export in exports.iter_mut().filter(|export| export.symbol == *symbol) {
                        export.declaration_kind = Some("generic-foreign-function".into());
                    }
                }
                crate::TypedForeignMember::Namespace { members, .. } => mark(members, exports),
                _ => {}
            }
        }
    }
    for foreign in &typed.foreign_modules {
        mark(&foreign.members, &mut exports);
    }

    // A final typed interface replaces shallow instance heads with canonical
    // evidence. Retaining both would expose one source declaration twice.
    let mut instances = Vec::new();
    instances.extend(
        typed
            .instances
            .iter()
            .map(|instance| interface_instance_from_typed(instance, &typed.module, &types)),
    );
    instances.extend(
        dependency_instances
            .iter()
            .map(interface_instance_from_dependency),
    );

    TypedModuleInterface {
        schema: shallow.schema,
        stage: "typed-interface".to_owned(),
        module: shallow.module,
        source: shallow.source,
        dependencies: shallow.dependencies,
        exports,
        operators: shallow.operators,
        instances,
    }
}

fn interface_instance_from_typed(
    instance: &TypedInstance,
    module: &str,
    types: &InterfaceTypes<'_>,
) -> InterfaceInstance {
    let types = types.with_parameters(&instance.type_parameters);
    InterfaceInstance {
        identity: Some(instance.identity.clone()),
        provider_module: Some(module.to_owned()),
        trait_identity: Some(instance.trait_identity.clone()),
        argument_identities: instance.argument_identities.clone(),
        type_identity: instance.type_identity.clone(),
        trait_name: instance.trait_name.clone(),
        type_parameters: instance.type_parameters.clone(),
        head: InterfaceType::Apply {
            constructor: instance.trait_name.clone(),
            arguments: instance
                .arguments
                .iter()
                .map(|argument| types.convert(argument))
                .collect(),
        },
        constraints: instance
            .constraints
            .iter()
            .zip(&instance.constraint_identities)
            .map(|(constraint, trait_identity)| InterfaceConstraint {
                name: constraint.name.clone(),
                trait_identity: trait_identity.clone(),
                arguments: constraint
                    .arguments
                    .iter()
                    .map(|argument| types.convert(argument))
                    .collect(),
            })
            .collect(),
        origin: instance.origin,
    }
}

fn interface_instance_from_dependency(instance: &ResolvedDependencyInstance) -> InterfaceInstance {
    InterfaceInstance {
        identity: Some(instance.identity.clone()),
        provider_module: Some(instance.provider_module.clone()),
        trait_identity: Some(instance.trait_identity.clone()),
        argument_identities: instance.argument_identities.clone(),
        type_identity: instance.type_identity.clone(),
        trait_name: instance.trait_name.clone(),
        type_parameters: instance.type_parameters.clone(),
        head: instance.head.clone(),
        constraints: instance.constraints.clone(),
        origin: instance.origin,
    }
}

fn typed_value_exports(
    declaration: &TypedDecl,
    module: &str,
    types: &InterfaceTypes<'_>,
) -> Vec<InterfaceExport> {
    match declaration {
        TypedDecl::Let {
            bindings,
            visibility,
            scheme,
            ..
        } if *visibility == Visibility::Public => bindings
            .iter()
            .map(|binding| InterfaceExport {
                symbol: binding.symbol.clone(),
                namespace: "value".to_owned(),
                name: binding.name.clone(),
                constructor_of: None,
                call_arity: None,
                visibility: *visibility,
                declaration_kind: None,
                declaration: binding.origin,
                scheme: interface_scheme_from_typed_scheme(
                    &TypedScheme {
                        type_parameters: scheme.type_parameters.clone(),
                        constraints: scheme.constraints.clone(),
                        constraint_identities: scheme.constraint_identities.clone(),
                        type_ref: binding.type_ref.clone(),
                    },
                    types,
                ),
                methods: Vec::new(),
                representation: None,
            })
            .collect(),
        declaration => typed_value_export(declaration, module, types)
            .into_iter()
            .collect(),
    }
}

fn typed_value_export(
    declaration: &TypedDecl,
    module: &str,
    types: &InterfaceTypes<'_>,
) -> Option<InterfaceExport> {
    let parameters = match declaration {
        TypedDecl::Fn { scheme, .. } => scheme.type_parameters.as_slice(),
        TypedDecl::EffectFn {
            type_parameters, ..
        } => type_parameters.as_slice(),
        _ => &[],
    };
    let scoped_types = types.with_parameters(parameters);
    let types = &scoped_types;
    match declaration {
        TypedDecl::Alias { .. } | TypedDecl::Adt { .. } | TypedDecl::Let { .. } => None,
        TypedDecl::Fn {
            symbol,
            visibility,
            origin,
            scheme,
            parameters,
            ..
        } if *visibility == Visibility::Public
            && !is_inherent_method_symbol(module, symbol)
            && !is_operator_symbol(module, symbol) =>
        {
            Some(InterfaceExport {
                symbol: symbol.clone(),
                namespace: "value".to_owned(),
                name: local_name(symbol),
                constructor_of: None,
                call_arity: matches!(scheme.type_ref, crate::TypedType::Function { .. })
                    .then_some(parameters.len()),
                visibility: *visibility,
                declaration_kind: Some("function".to_owned()),
                declaration: *origin,
                scheme: InterfaceScheme {
                    type_parameters: scheme.type_parameters.clone(),
                    constraints: scheme
                        .constraints
                        .iter()
                        .enumerate()
                        .map(|(index, constraint)| InterfaceConstraint {
                            name: constraint.name.clone(),
                            trait_identity: scheme
                                .constraint_identities
                                .get(index)
                                .cloned()
                                .flatten(),
                            arguments: constraint
                                .arguments
                                .iter()
                                .map(|argument| types.convert(argument))
                                .collect(),
                        })
                        .collect(),
                    type_ref: function_interface_type(
                        parameters,
                        &types.convert(&scheme.type_ref),
                        types,
                    ),
                },
                methods: Vec::new(),
                representation: None,
            })
        }
        TypedDecl::EffectFn {
            symbol,
            visibility,
            origin,
            type_parameters,
            constraints,
            constraint_identities,
            parameters,
            effect,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: symbol.clone(),
            namespace: "value".to_owned(),
            name: local_name(symbol),
            constructor_of: None,
            call_arity: None,
            visibility: *visibility,
            declaration_kind: Some("effect-function".to_owned()),
            declaration: *origin,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: constraints
                    .iter()
                    .enumerate()
                    .map(|(index, constraint)| InterfaceConstraint {
                        name: constraint.name.clone(),
                        trait_identity: constraint_identities.get(index).cloned().flatten(),
                        arguments: constraint
                            .arguments
                            .iter()
                            .map(|argument| types.convert(argument))
                            .collect(),
                    })
                    .collect(),
                type_ref: function_interface_type(
                    parameters,
                    &effect_interface_type(effect, types),
                    types,
                ),
            },
            methods: Vec::new(),
            representation: None,
        }),
        _ => None,
    }
}

fn is_inherent_method_symbol(module: &str, symbol: &str) -> bool {
    symbol
        .strip_prefix(module)
        .and_then(|relative| relative.strip_prefix("::"))
        .is_some_and(|relative| relative.contains("::"))
}

fn is_operator_symbol(module: &str, symbol: &str) -> bool {
    symbol
        .strip_prefix(module)
        .and_then(|relative| relative.strip_prefix("::"))
        .is_some_and(|relative| relative.starts_with("operator(") && relative.ends_with(')'))
}

fn interface_scheme_from_typed_scheme(
    scheme: &TypedScheme,
    types: &InterfaceTypes<'_>,
) -> InterfaceScheme {
    let types = &types.with_parameters(&scheme.type_parameters);
    InterfaceScheme {
        type_parameters: scheme.type_parameters.clone(),
        constraints: scheme
            .constraints
            .iter()
            .enumerate()
            .map(|(index, constraint)| InterfaceConstraint {
                name: constraint.name.clone(),
                trait_identity: scheme.constraint_identities.get(index).cloned().flatten(),
                arguments: constraint
                    .arguments
                    .iter()
                    .map(|argument| types.convert(argument))
                    .collect(),
            })
            .collect(),
        type_ref: types.convert(&scheme.type_ref),
    }
}

fn function_interface_type(
    parameters: &[TypedParameter],
    result: &InterfaceType,
    types: &InterfaceTypes<'_>,
) -> InterfaceType {
    parameters
        .iter()
        .rev()
        .fold(result.clone(), |result, parameter| {
            InterfaceType::Function {
                parameter: Box::new(types.parameter(parameter)),
                result: Box::new(result),
            }
        })
}

fn effect_interface_type(effect: &TypedEffect, types: &InterfaceTypes<'_>) -> InterfaceType {
    InterfaceType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            types.convert(&effect.environment),
            types.convert(&effect.failure),
            types.convert(&effect.success),
        ],
    }
}

fn local_name(symbol: &str) -> String {
    symbol
        .rsplit_once("::")
        .map(|(_, name)| name.to_owned())
        .unwrap_or_else(|| symbol.to_owned())
}
