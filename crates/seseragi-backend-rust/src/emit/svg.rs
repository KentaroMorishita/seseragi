//! The static part of `std/web/svg`: scene nodes rendered to HTML text when
//! the page is written. The binding targets need a browser DOM and stay
//! unsupported here.
use super::*;

const TAGS: [&str; 9] = [
    "svg", "g", "rect", "path", "circle", "line", "polyline", "polygon", "text",
];

pub(super) fn operation(name: &str) -> Option<&str> {
    let name = name.strip_prefix("std/web/svg::")?;
    (TAGS.contains(&name) || name == "toHtml").then_some(name)
}

fn scene_action(ty: &CoreType) -> Option<&CoreType> {
    match ty {
        CoreType::Named { name, arguments } if name == "std/web/svg::Svg" => {
            match arguments.as_slice() {
                [action] => Some(action),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Whether `parameter -> result` is the checked type of an SVG operation:
/// a closed record of props with the fields the scene requires.
pub(super) fn valid(operation: &str, parameter: &CoreType, result: &CoreType) -> bool {
    if operation == "toHtml" {
        return matches!(result, CoreType::Named { name, arguments }
            if name == "std/web/html::Html"
                && matches!(arguments.as_slice(), [action] if scene_action(parameter) == Some(action)));
    }
    if scene_action(result).is_none() {
        return false;
    }
    let CoreType::Record {
        closed: true,
        fields,
    } = parameter
    else {
        return false;
    };
    // The runtime checks each value when it is written, with the TS messages,
    // so a wider record passed on behaves as on TS.
    let mut names = BTreeSet::new();
    let fields_valid = fields.iter().all(|field| names.insert(field.name.as_str()));
    let required: &[&str] = match operation {
        "svg" | "g" | "text" => &["children"],
        "path" => &["d"],
        "polyline" | "polygon" => &["points"],
        _ => &[],
    };
    fields_valid
        && required.iter().all(|required| {
            fields
                .iter()
                .any(|field| field.name == *required && !field.optional)
        })
}
