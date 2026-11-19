//! The static part of `std/web/svg`: scene nodes rendered to HTML text. The
//! binding targets need a browser DOM and stay unsupported here.
use super::*;

const TAGS: [&str; 9] = [
    "svg", "g", "rect", "path", "circle", "line", "polyline", "polygon", "text",
];

/// The attributes the TS renderer writes for every element, with their types.
const COMMON: [(&str, &str); 17] = [
    ("id", "std/prelude::String"),
    ("class", "std/prelude::String"),
    ("title", "std/prelude::String"),
    ("hidden", "std/prelude::Bool"),
    ("style", "std/web/html::Style"),
    ("role", "std/prelude::String"),
    ("tabIndex", "std/prelude::Int"),
    ("lang", "std/prelude::String"),
    ("dir", "std/prelude::String"),
    ("draggable", "std/prelude::Bool"),
    ("contentEditable", "std/prelude::Bool"),
    ("inert", "std/prelude::Bool"),
    ("ariaLabel", "std/prelude::String"),
    ("ariaBusy", "std/prelude::Bool"),
    ("ariaExpanded", "std/prelude::Bool"),
    ("ariaHidden", "std/prelude::Bool"),
    ("ariaSelected", "std/prelude::Bool"),
];

/// The SVG attributes the TS renderer writes, as text or as a number.
const TEXT: [&str; 13] = [
    "viewBox",
    "preserveAspectRatio",
    "transform",
    "d",
    "points",
    "fill",
    "stroke",
    "strokeWidth",
    "strokeLinecap",
    "strokeLinejoin",
    "textAnchor",
    "dominantBaseline",
    "pointerEvents",
];
const NUMBERS: [&str; 14] = [
    "x", "y", "x1", "y1", "x2", "y2", "cx", "cy", "r", "rx", "ry", "width", "height", "opacity",
];

pub(super) fn operation(name: &str) -> Option<&str> {
    let name = name.strip_prefix("std/web/svg::")?;
    (TAGS.contains(&name) || name == "toHtml").then_some(name)
}

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
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

/// Whether `parameter -> result` is the checked type of an SVG operation.
/// A field the TS renderer writes must have the type it renders; any other
/// field, including handlers and element refs that only a browser uses, is
/// ignored by both renderers.
pub(super) fn valid(operation: &str, parameter: &CoreType, result: &CoreType) -> bool {
    if operation == "toHtml" {
        return matches!(result, CoreType::Named { name, arguments }
            if name == "std/web/html::Html"
                && matches!(arguments.as_slice(), [action] if scene_action(parameter) == Some(action)));
    }
    let Some(action) = scene_action(result) else {
        return false;
    };
    let CoreType::Record {
        closed: true,
        fields,
    } = parameter
    else {
        return false;
    };
    let string = named("std/prelude::String", vec![]);
    let scene = named("std/web/svg::Svg", vec![action.clone()]);
    let mut names = BTreeSet::new();
    let fields_valid = fields.iter().all(|field| {
        if !names.insert(field.name.as_str()) {
            return false;
        }
        let name = field.name.as_str();
        if name == "children" {
            // The TS scene accepts text, one node or an Array of nodes.
            return field.type_ref == string
                || field.type_ref == scene
                || field.type_ref == named("std/prelude::Array", vec![scene.clone()]);
        }
        if name == "attributes" {
            return field.type_ref
                == named(
                    "std/prelude::Array",
                    vec![named("std/web/html::Attribute", vec![])],
                );
        }
        if let Some((_, ty)) = COMMON.iter().find(|(common, _)| *common == name) {
            return field.type_ref == named(ty, vec![]);
        }
        if TEXT.contains(&name) {
            return field.type_ref == string;
        }
        if NUMBERS.contains(&name) {
            return field.type_ref == named("std/prelude::Float", vec![]);
        }
        true
    });
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
