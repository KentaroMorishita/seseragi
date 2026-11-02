use crate::{TypedConstraint, TypedType};
use std::{collections::BTreeSet, sync::OnceLock};

const INTO_CHILDREN: &str = "std/web/html::trait(IntoChildren)";
const STYLE_RECORD: &str = "std/web/html::trait(StyleRecord)";
const SIGNAL: &str = "std/signal::Signal";
const MUTABLE_SIGNAL: &str = "std/signal::MutableSignal";
const SIGNAL_READ: &str = "std/signal::read";
const SIGNAL_SET: &str = "std/signal::set";

struct StandardModuleTrait {
    identity: &'static str,
    name: &'static str,
    heads: fn() -> Vec<(StandardInstanceHead, &'static str)>,
    structural_instance: Option<StandardStructuralInstance>,
}

struct StandardStructuralInstance {
    identity: &'static str,
    matches: fn(&TypedConstraint) -> bool,
}

const STANDARD_MODULE_TRAITS: &[StandardModuleTrait] = &[
    StandardModuleTrait {
        identity: INTO_CHILDREN,
        name: "IntoChildren",
        heads: into_children_instance_heads,
        structural_instance: None,
    },
    StandardModuleTrait {
        identity: STYLE_RECORD,
        name: "StyleRecord",
        heads: Vec::new,
        structural_instance: Some(StandardStructuralInstance {
            identity: "std/web/html::StyleRecord<Record>",
            matches: is_style_record_instance,
        }),
    },
];

pub(crate) struct StandardInstanceHead {
    pub(crate) parameters: Vec<seseragi_syntax::TypeParameter>,
    pub(crate) arguments: Vec<TypedType>,
}

pub(crate) fn standard_module_instance_heads(
    trait_identity: Option<&str>,
    name: &str,
) -> Vec<(StandardInstanceHead, &'static str)> {
    standard_module_trait(trait_identity, name)
        .map(|trait_spec| (trait_spec.heads)())
        .unwrap_or_default()
}

fn standard_module_trait(
    identity: Option<&str>,
    name: &str,
) -> Option<&'static StandardModuleTrait> {
    STANDARD_MODULE_TRAITS
        .iter()
        .find(|trait_spec| Some(trait_spec.identity) == identity && trait_spec.name == name)
}

fn into_children_instance_heads() -> Vec<(StandardInstanceHead, &'static str)> {
    let action = named("Action");
    let html = TypedType::ExternalNamed {
        name: "Html".to_owned(),
        canonical: "std/web/html::Html".to_owned(),
        arguments: vec![action.clone()],
    };
    [
        (named("Unit"), "std/web/html::IntoChildren<Unit>"),
        (named("String"), "std/web/html::IntoChildren<String>"),
        (html.clone(), "std/web/html::IntoChildren<Html>"),
        (
            TypedType::Named {
                name: "Array".to_owned(),
                arguments: vec![html.clone()],
            },
            "std/web/html::IntoChildren<Array>",
        ),
        (
            TypedType::Named {
                name: "List".to_owned(),
                arguments: vec![html],
            },
            "std/web/html::IntoChildren<List>",
        ),
    ]
    .into_iter()
    .map(|(children, identity)| {
        (
            StandardInstanceHead {
                parameters: vec![seseragi_syntax::TypeParameter::value("Action".to_owned())],
                arguments: vec![children, action.clone()],
            },
            identity,
        )
    })
    .collect()
}

/// Returns whether a registered standard instance has no dictionary methods
/// or superclass dictionaries. Instance arguments are checked by its consumer.
pub fn standard_instance_has_empty_dictionary(identity: &str) -> bool {
    static EMPTY_MODULE_INSTANCES: OnceLock<BTreeSet<&'static str>> = OnceLock::new();
    if EMPTY_MODULE_INSTANCES
        .get_or_init(|| {
            let mut instances = BTreeSet::new();
            for trait_spec in STANDARD_MODULE_TRAITS {
                let Some((module, _)) = trait_spec.identity.split_once("::") else {
                    continue;
                };
                let Some(target) = seseragi_project::standard_module_target(module) else {
                    continue;
                };
                if target.interface().exports.iter().any(|export| {
                    export.symbol == trait_spec.identity
                        && module_trait_has_empty_dictionary(export)
                }) {
                    instances.extend(
                        (trait_spec.heads)()
                            .into_iter()
                            .map(|(_, identity)| identity),
                    );
                    instances.extend(
                        trait_spec
                            .structural_instance
                            .iter()
                            .map(|instance| instance.identity),
                    );
                }
            }
            instances
        })
        .contains(identity)
    {
        return true;
    }
    let trait_name = crate::prelude::standard_instance_by_identity(identity)
        .map(|instance| instance.trait_name)
        .or_else(|| {
            crate::prelude::special_standard_instance_by_identity(identity)
                .map(|instance| instance.trait_name)
        });
    trait_name
        .and_then(crate::prelude::trait_by_name)
        .is_some_and(|trait_spec| {
            trait_spec.supertrait.is_none()
                && !crate::prelude::TRAIT_METHODS
                    .iter()
                    .any(|method| method.trait_name == trait_spec.name)
        })
}

fn module_trait_has_empty_dictionary(export: &seseragi_syntax::InterfaceExport) -> bool {
    export.namespace == "trait"
        && export.declaration_kind.as_deref() == Some("trait")
        && export.methods.is_empty()
        && export.scheme.constraints.is_empty()
}

pub(crate) struct StandardSignalCall {
    pub(crate) canonical: &'static str,
    pub(crate) result: TypedType,
}

pub(crate) fn standard_signal_read_call(source: &TypedType) -> Option<StandardSignalCall> {
    let value = signal_value_type(source, false)?.clone();
    Some(StandardSignalCall {
        canonical: SIGNAL_READ,
        result: signal_task(value),
    })
}

pub(crate) fn standard_signal_set_call(
    target: &TypedType,
) -> Option<(TypedType, StandardSignalCall)> {
    let value = signal_value_type(target, true)?.clone();
    Some((
        value,
        StandardSignalCall {
            canonical: SIGNAL_SET,
            result: signal_task(named("Unit")),
        },
    ))
}

pub(crate) fn standard_signal_read_recovery_call() -> StandardSignalCall {
    StandardSignalCall {
        canonical: SIGNAL_READ,
        result: signal_task(TypedType::Hole),
    }
}

pub(crate) fn standard_signal_set_recovery_call() -> StandardSignalCall {
    StandardSignalCall {
        canonical: SIGNAL_SET,
        result: signal_task(named("Unit")),
    }
}

pub(crate) fn standard_signal_expected(mutable: bool) -> TypedType {
    TypedType::ExternalNamed {
        name: if mutable { "MutableSignal" } else { "Signal" }.to_owned(),
        canonical: if mutable { MUTABLE_SIGNAL } else { SIGNAL }.to_owned(),
        arguments: vec![TypedType::Hole],
    }
}

fn signal_value_type(type_ref: &TypedType, mutable_only: bool) -> Option<&TypedType> {
    let TypedType::ExternalNamed {
        canonical,
        arguments,
        ..
    } = type_ref
    else {
        return None;
    };
    let [value] = arguments.as_slice() else {
        return None;
    };
    ((!mutable_only && matches!(canonical.as_str(), SIGNAL | MUTABLE_SIGNAL))
        || (mutable_only && canonical == MUTABLE_SIGNAL))
        .then_some(value)
}

fn signal_task(success: TypedType) -> TypedType {
    TypedType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            TypedType::Record {
                closed: true,
                fields: Vec::new(),
            },
            named("Never"),
            success,
        ],
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

pub(crate) fn standard_type_coercion(expected: &TypedType, actual: &TypedType) -> bool {
    standard_type_coercion_arguments(expected, actual)
        .is_some_and(|(expected, actual)| expected == actual)
}

pub(crate) fn standard_type_coercion_arguments<'a>(
    expected: &'a TypedType,
    actual: &'a TypedType,
) -> Option<(&'a [TypedType], &'a [TypedType])> {
    match (expected, actual) {
        (
            TypedType::ExternalNamed {
                canonical: expected_canonical,
                arguments: expected_arguments,
                ..
            },
            TypedType::ExternalNamed {
                canonical: actual_canonical,
                arguments: actual_arguments,
                ..
            },
        ) if expected_canonical == SIGNAL
            && actual_canonical == MUTABLE_SIGNAL
            && expected_arguments.len() == actual_arguments.len() =>
        {
            Some((expected_arguments, actual_arguments))
        }
        _ => None,
    }
}

pub(crate) fn standard_module_instance(
    trait_identity: Option<&str>,
    constraint: &TypedConstraint,
) -> Option<&'static str> {
    let trait_spec = standard_module_trait(trait_identity, &constraint.name)?;
    (trait_spec.heads)()
        .into_iter()
        .find_map(|(head, identity)| {
            if head.arguments.len() != constraint.arguments.len() {
                return None;
            }
            let mut substitutions = std::collections::BTreeMap::new();
            head.arguments
                .iter()
                .zip(&constraint.arguments)
                .all(|(template, actual)| {
                    instance_argument_matches(
                        template,
                        actual,
                        &head.parameters,
                        &mut substitutions,
                    )
                })
                .then_some(identity)
        })
        .or_else(|| {
            trait_spec
                .structural_instance
                .as_ref()
                .filter(|instance| (instance.matches)(constraint))
                .map(|instance| instance.identity)
        })
}

fn is_style_record_instance(constraint: &TypedConstraint) -> bool {
    let [declarations] = constraint.arguments.as_slice() else {
        return false;
    };
    let TypedType::Record { fields, .. } = declarations else {
        return false;
    };
    fields.iter().all(|field| {
        if field.name == "variables" {
            return style_variables_record(&field.type_ref);
        }
        named_leaf(&field.type_ref, "String")
    })
}

fn style_variables_record(type_ref: &TypedType) -> bool {
    let TypedType::Record { fields, .. } = type_ref else {
        return false;
    };
    fields
        .iter()
        .all(|field| named_leaf(&field.type_ref, "String"))
}

fn named_leaf(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, arguments } if name == expected && arguments.is_empty())
}

fn instance_argument_matches(
    expected: &TypedType,
    actual: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
    substitutions: &mut std::collections::BTreeMap<String, TypedType>,
) -> bool {
    if let TypedType::Named { name, arguments } = expected {
        if arguments.is_empty() && parameters.iter().any(|parameter| parameter.name == *name) {
            return match substitutions.entry(name.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(actual.clone());
                    true
                }
                std::collections::btree_map::Entry::Occupied(entry) => entry.get() == actual,
            };
        }
    }
    match (expected, actual) {
        (
            TypedType::Named { name, arguments },
            TypedType::Named {
                name: actual_name,
                arguments: actual_arguments,
            },
        ) => {
            name == actual_name
                && arguments.len() == actual_arguments.len()
                && arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(expected, actual)| {
                        instance_argument_matches(expected, actual, parameters, substitutions)
                    })
        }
        (
            TypedType::Named { name, arguments },
            TypedType::ExternalNamed {
                name: actual_name,
                arguments: actual_arguments,
                ..
            },
        ) if !arguments.is_empty() && name == actual_name => {
            arguments.len() == actual_arguments.len()
                && arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(expected, actual)| {
                        instance_argument_matches(expected, actual, parameters, substitutions)
                    })
        }
        (
            TypedType::ExternalNamed {
                canonical,
                arguments,
                ..
            },
            TypedType::ExternalNamed {
                canonical: actual_canonical,
                arguments: actual_arguments,
                ..
            },
        ) => {
            canonical == actual_canonical
                && arguments.len() == actual_arguments.len()
                && arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(expected, actual)| {
                        instance_argument_matches(expected, actual, parameters, substitutions)
                    })
        }
        _ => expected == actual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methodless_dictionaries_follow_the_instance_selection_registry() {
        for trait_spec in STANDARD_MODULE_TRAITS {
            let (module, _) = trait_spec.identity.split_once("::").unwrap();
            let target = seseragi_project::standard_module_target(module).unwrap();
            let export = target
                .interface()
                .exports
                .iter()
                .find(|export| export.symbol == trait_spec.identity)
                .unwrap();
            let empty = module_trait_has_empty_dictionary(export);
            for (head, identity) in (trait_spec.heads)() {
                let constraint = TypedConstraint {
                    name: trait_spec.name.to_owned(),
                    arguments: head.arguments,
                };
                assert_eq!(
                    standard_module_instance(Some(trait_spec.identity), &constraint),
                    Some(identity)
                );
                assert_eq!(standard_instance_has_empty_dictionary(identity), empty);
            }
            if let Some(instance) = &trait_spec.structural_instance {
                assert_eq!(
                    standard_instance_has_empty_dictionary(instance.identity),
                    empty
                );
            }
        }
    }

    #[test]
    fn empty_dictionaries_require_no_methods_or_superclass_dictionaries() {
        let target = seseragi_project::standard_module_target("std/web/html").unwrap();
        let mut marker = target
            .interface()
            .exports
            .iter()
            .find(|export| export.symbol == STYLE_RECORD)
            .unwrap()
            .clone();
        marker.name = "Marker".to_owned();
        marker.symbol = "std/example::trait(Marker)".to_owned();
        assert!(module_trait_has_empty_dictionary(&marker));

        marker.methods.push(seseragi_syntax::InterfaceMethod {
            name: "inspect".to_owned(),
            deprecation: None,
            scheme: marker.scheme.clone(),
            origin: marker.declaration,
        });
        assert!(!module_trait_has_empty_dictionary(&marker));

        marker.methods.clear();
        marker
            .scheme
            .constraints
            .push(seseragi_syntax::InterfaceConstraint {
                name: "Parent".to_owned(),
                trait_identity: Some("std/example::trait(Parent)".to_owned()),
                arguments: Vec::new(),
            });
        assert!(!module_trait_has_empty_dictionary(&marker));

        marker.scheme.constraints.clear();
        marker.namespace = "value".to_owned();
        assert!(!module_trait_has_empty_dictionary(&marker));
    }

    #[test]
    fn unknown_and_method_bearing_standard_instances_are_not_empty_dictionaries() {
        for identity in [
            "std/example::Marker<Unit>",
            "std/web/html::IntoChildren<Int>",
            "std/int::Add",
            "std/array::Functor",
            "std/string::Monoid",
        ] {
            assert!(
                !standard_instance_has_empty_dictionary(identity),
                "{identity}"
            );
        }
    }

    #[test]
    fn selects_only_the_declared_html_children_shapes() {
        let string = TypedConstraint {
            name: "IntoChildren".to_owned(),
            arguments: vec![named("String"), named("Action")],
        };
        assert_eq!(
            standard_module_instance(Some(INTO_CHILDREN), &string),
            Some("std/web/html::IntoChildren<String>")
        );

        let invalid = TypedConstraint {
            name: "IntoChildren".to_owned(),
            arguments: vec![named("Int"), named("Action")],
        };
        assert_eq!(
            standard_module_instance(Some(INTO_CHILDREN), &invalid),
            None
        );
    }

    #[test]
    fn selects_only_string_valued_style_records() {
        let style = TypedConstraint {
            name: "StyleRecord".to_owned(),
            arguments: vec![TypedType::Record {
                closed: true,
                fields: vec![
                    crate::TypedRecordField {
                        name: "backgroundColor".to_owned(),
                        optional: false,
                        type_ref: named("String"),
                    },
                    crate::TypedRecordField {
                        name: "variables".to_owned(),
                        optional: false,
                        type_ref: TypedType::Record {
                            closed: true,
                            fields: vec![crate::TypedRecordField {
                                name: "cardShadow".to_owned(),
                                optional: false,
                                type_ref: named("String"),
                            }],
                        },
                    },
                ],
            }],
        };
        assert_eq!(
            standard_module_instance(Some(STYLE_RECORD), &style),
            Some("std/web/html::StyleRecord<Record>")
        );

        let invalid = TypedConstraint {
            name: "StyleRecord".to_owned(),
            arguments: vec![TypedType::Record {
                closed: true,
                fields: vec![crate::TypedRecordField {
                    name: "padding".to_owned(),
                    optional: false,
                    type_ref: named("Int"),
                }],
            }],
        };
        assert!(standard_module_instance(Some(STYLE_RECORD), &invalid).is_none());
    }

    #[test]
    fn lowers_mutable_signal_to_read_only_signal_only() {
        let expected = external("Signal", SIGNAL, vec![named("Int")]);
        let actual = external("MutableSignal", MUTABLE_SIGNAL, vec![named("Int")]);

        assert!(standard_type_coercion(&expected, &actual));
        assert!(!standard_type_coercion(&actual, &expected));
        assert!(!standard_type_coercion(
            &expected,
            &external("MutableSignal", MUTABLE_SIGNAL, vec![named("String")])
        ));
    }

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    fn external(name: &str, canonical: &str, arguments: Vec<TypedType>) -> TypedType {
        TypedType::ExternalNamed {
            name: name.to_owned(),
            canonical: canonical.to_owned(),
            arguments,
        }
    }
}
