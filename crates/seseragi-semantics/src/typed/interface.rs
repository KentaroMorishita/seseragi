use crate::{
    ResolvedDependencyInstance, TypedDecl, TypedEffect, TypedInstance, TypedModule,
    TypedModuleInterface, TypedParameter, TypedScheme,
};
use seseragi_syntax::{
    InterfaceConstraint, InterfaceExport, InterfaceInstance, InterfaceScheme, InterfaceType,
    ModuleInterface, Visibility,
};

mod types;

use types::InterfaceTypes;

pub(crate) fn typed_interface_from_modules(
    shallow: ModuleInterface,
    typed: &TypedModule,
    dependency_instances: &[ResolvedDependencyInstance],
) -> TypedModuleInterface {
    let types = InterfaceTypes::new(&typed.external_type_bindings);
    let mut exports = shallow
        .exports
        .into_iter()
        .filter(|export| {
            export.namespace != "value" || export.declaration_kind.as_deref() == Some("constructor")
        })
        .collect::<Vec<_>>();
    for declaration in &typed.declarations {
        let TypedDecl::Alias {
            symbol,
            visibility,
            target,
            ..
        } = declaration
        else {
            continue;
        };
        if *visibility != Visibility::Public {
            continue;
        }
        if let Some(export) = exports.iter_mut().find(|export| {
            export.namespace == "type"
                && export.declaration_kind.as_deref() == Some("alias")
                && export.symbol == *symbol
        }) {
            export.representation = Some(types.convert(target));
        }
    }
    exports.extend(
        typed
            .declarations
            .iter()
            .flat_map(|declaration| typed_value_exports(declaration, &typed.module, &types))
            .collect::<Vec<_>>(),
    );

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
