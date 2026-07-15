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
    exports.extend(
        typed
            .declarations
            .iter()
            .filter_map(|declaration| typed_value_export(declaration, &types))
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
            .map(|constraint| InterfaceConstraint {
                name: constraint.name.clone(),
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
        type_identity: Some(instance.type_identity.clone()),
        trait_name: instance.trait_name.clone(),
        type_parameters: instance.type_parameters.clone(),
        head: instance.head.clone(),
        constraints: instance.constraints.clone(),
        origin: instance.origin,
    }
}

fn typed_value_export(
    declaration: &TypedDecl,
    types: &InterfaceTypes<'_>,
) -> Option<InterfaceExport> {
    match declaration {
        TypedDecl::Adt { .. } => None,
        TypedDecl::Let {
            symbol,
            visibility,
            origin,
            scheme,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: symbol.clone(),
            namespace: "value".to_owned(),
            name: local_name(symbol),
            constructor_of: None,
            visibility: *visibility,
            declaration_kind: None,
            declaration: *origin,
            scheme: interface_scheme_from_typed_scheme(scheme, types),
            methods: Vec::new(),
            representation: None,
        }),
        TypedDecl::Fn {
            symbol,
            visibility,
            origin,
            scheme,
            parameters,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
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
                    .map(|constraint| InterfaceConstraint {
                        name: constraint.name.clone(),
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
        }),
        TypedDecl::EffectFn {
            symbol,
            visibility,
            origin,
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
                type_parameters: Vec::new(),
                constraints: Vec::new(),
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

fn interface_scheme_from_typed_scheme(
    scheme: &TypedScheme,
    types: &InterfaceTypes<'_>,
) -> InterfaceScheme {
    InterfaceScheme {
        type_parameters: scheme.type_parameters.clone(),
        constraints: scheme
            .constraints
            .iter()
            .map(|constraint| InterfaceConstraint {
                name: constraint.name.clone(),
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
