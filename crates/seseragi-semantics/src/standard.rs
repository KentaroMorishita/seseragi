use crate::{TypedConstraint, TypedType};

const INTO_CHILDREN: &str = "std/web/html::trait(IntoChildren)";
const SIGNAL: &str = "std/signal::Signal";
const MUTABLE_SIGNAL: &str = "std/signal::MutableSignal";
const SIGNAL_READ: &str = "std/signal::read";
const SIGNAL_SET: &str = "std/signal::set";

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
    if trait_identity != Some(INTO_CHILDREN) || constraint.name != "IntoChildren" {
        return None;
    }
    let [children, message] = constraint.arguments.as_slice() else {
        return None;
    };

    if named_leaf(children, "Unit") {
        return Some("std/web/html::IntoChildren<Unit>");
    }
    if named_leaf(children, "String") {
        return Some("std/web/html::IntoChildren<String>");
    }
    if html_message(children).is_some_and(|html_message| html_message == message) {
        return Some("std/web/html::IntoChildren<Html>");
    }
    if collection_html_message(children, "Array")
        .is_some_and(|html_message| html_message == message)
    {
        return Some("std/web/html::IntoChildren<Array>");
    }
    if collection_html_message(children, "List").is_some_and(|html_message| html_message == message)
    {
        return Some("std/web/html::IntoChildren<List>");
    }
    None
}

fn named_leaf(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, arguments } if name == expected && arguments.is_empty())
}

fn html_message(type_ref: &TypedType) -> Option<&TypedType> {
    match type_ref {
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if canonical == "std/web/html::Html" => arguments.first(),
        _ => None,
    }
}

fn collection_html_message<'a>(type_ref: &'a TypedType, collection: &str) -> Option<&'a TypedType> {
    let child = match type_ref {
        TypedType::Named { name, arguments } if name == collection => arguments.first(),
        TypedType::ExternalNamed {
            name, arguments, ..
        } if name == collection => arguments.first(),
        _ => None,
    }?;
    html_message(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_only_the_declared_html_children_shapes() {
        let string = TypedConstraint {
            name: "IntoChildren".to_owned(),
            arguments: vec![named("String"), named("Msg")],
        };
        assert_eq!(
            standard_module_instance(Some(INTO_CHILDREN), &string),
            Some("std/web/html::IntoChildren<String>")
        );

        let invalid = TypedConstraint {
            name: "IntoChildren".to_owned(),
            arguments: vec![named("Int"), named("Msg")],
        };
        assert_eq!(
            standard_module_instance(Some(INTO_CHILDREN), &invalid),
            None
        );
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
