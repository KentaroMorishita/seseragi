use crate::{
    TypedDecl, TypedEffect, TypedInstance, TypedModule, TypedModuleInterface, TypedParameter,
    TypedRecordField, TypedScheme, TypedType,
};
use seseragi_syntax::{
    InterfaceConstraint, InterfaceExport, InterfaceInstance, InterfaceRecordField, InterfaceScheme,
    InterfaceType, ModuleInterface, Visibility,
};

pub(crate) fn typed_interface_from_modules(
    shallow: ModuleInterface,
    typed: &TypedModule,
) -> TypedModuleInterface {
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
            .filter_map(typed_value_export)
            .collect::<Vec<_>>(),
    );

    let mut instances = shallow.instances;
    instances.extend(typed.instances.iter().map(interface_instance_from_typed));

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

fn interface_instance_from_typed(instance: &TypedInstance) -> InterfaceInstance {
    InterfaceInstance {
        trait_name: instance.trait_name.clone(),
        type_parameters: Vec::new(),
        head: InterfaceType::Apply {
            constructor: instance.trait_name.clone(),
            arguments: vec![interface_type_from_typed_type(&instance.head)],
        },
        constraints: instance
            .constraints
            .iter()
            .map(|constraint| InterfaceConstraint {
                name: constraint.name.clone(),
            })
            .collect(),
        origin: instance.origin,
    }
}

fn typed_value_export(declaration: &TypedDecl) -> Option<InterfaceExport> {
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
            scheme: interface_scheme_from_typed_scheme(scheme),
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
                    })
                    .collect(),
                type_ref: function_interface_type(
                    parameters,
                    &interface_type_from_typed_type(&scheme.type_ref),
                ),
            },
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
                type_ref: function_interface_type(parameters, &effect_interface_type(effect)),
            },
            representation: None,
        }),
        _ => None,
    }
}

fn interface_scheme_from_typed_scheme(scheme: &TypedScheme) -> InterfaceScheme {
    InterfaceScheme {
        type_parameters: scheme.type_parameters.clone(),
        constraints: scheme
            .constraints
            .iter()
            .map(|constraint| InterfaceConstraint {
                name: constraint.name.clone(),
            })
            .collect(),
        type_ref: interface_type_from_typed_type(&scheme.type_ref),
    }
}

fn function_interface_type(parameters: &[TypedParameter], result: &InterfaceType) -> InterfaceType {
    parameters
        .iter()
        .rev()
        .fold(result.clone(), |result, parameter| {
            InterfaceType::Function {
                parameter: Box::new(interface_type_from_typed_type(parameter_type(parameter))),
                result: Box::new(result),
            }
        })
}

fn effect_interface_type(effect: &TypedEffect) -> InterfaceType {
    InterfaceType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            interface_type_from_typed_type(&effect.environment),
            interface_type_from_typed_type(&effect.failure),
            interface_type_from_typed_type(&effect.success),
        ],
    }
}

fn interface_type_from_typed_type(type_ref: &TypedType) -> InterfaceType {
    match type_ref {
        TypedType::Named { name, arguments } => InterfaceType::Named {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(interface_type_from_typed_type)
                .collect(),
        },
        TypedType::Hole => InterfaceType::Hole,
        TypedType::Record { closed, fields } => InterfaceType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(interface_record_field_from_typed)
                .collect(),
        },
        TypedType::Tuple { elements } => InterfaceType::Tuple {
            elements: elements
                .iter()
                .map(interface_type_from_typed_type)
                .collect(),
        },
        TypedType::Function { parameter, result } => InterfaceType::Function {
            parameter: Box::new(interface_type_from_typed_type(parameter)),
            result: Box::new(interface_type_from_typed_type(result)),
        },
    }
}

fn interface_record_field_from_typed(field: &TypedRecordField) -> InterfaceRecordField {
    InterfaceRecordField {
        name: field.name.clone(),
        optional: field.optional,
        type_ref: interface_type_from_typed_type(&field.type_ref),
    }
}

fn parameter_type(parameter: &TypedParameter) -> &TypedType {
    match parameter {
        TypedParameter::ImplicitUnit { type_ref } | TypedParameter::Named { type_ref, .. } => {
            type_ref
        }
    }
}

fn local_name(symbol: &str) -> String {
    symbol
        .rsplit_once("::")
        .map(|(_, name)| name.to_owned())
        .unwrap_or_else(|| symbol.to_owned())
}
