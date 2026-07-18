use crate::{TypedConstraint, TypedType};

const INTO_CHILDREN: &str = "std/web/html::trait(IntoChildren)";

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

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }
}
