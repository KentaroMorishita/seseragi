use super::*;

pub(super) const SVG_TAGS: [&str; 9] = [
    "svg", "g", "rect", "path", "circle", "line", "polyline", "polygon", "text",
];

pub(super) fn web_svg_interface() -> ModuleInterface {
    let module = "std/web/svg";
    let svg = || named_with("Svg", vec![named("Action")]);
    let element_ref = || {
        external_type(
            "ElementRef",
            "std/web/html::ElementRef",
            "std/web/html",
            "ElementRef",
            Vec::new(),
        )
    };
    let binding_target = |value: InterfaceType| {
        external_type(
            "BindingTarget",
            "std/web/dom::BindingTarget",
            "std/web/dom",
            "BindingTarget",
            vec![named("Action"), value],
        )
    };
    let mut exports = vec![type_export(module, "Svg", 1, "opaque-type")];
    for name in SVG_TAGS {
        exports.push(function_export(
            module,
            name,
            ["Action"],
            Vec::new(),
            vec![svg_tag_props(name).expect("standard SVG tag props")],
            svg(),
        ));
    }
    for (name, value) in [
        ("viewBoxTarget", named("String")),
        ("transformTarget", named("String")),
        ("pathDataTarget", named("String")),
        ("pointsTarget", named("String")),
        ("fillTarget", named("String")),
        ("strokeTarget", named("String")),
        ("strokeWidthTarget", named("String")),
        ("pointerEventsTarget", named("String")),
        ("ariaLabelTarget", named("String")),
        ("xTarget", named("Float")),
        ("yTarget", named("Float")),
        ("x1Target", named("Float")),
        ("y1Target", named("Float")),
        ("x2Target", named("Float")),
        ("y2Target", named("Float")),
        ("cxTarget", named("Float")),
        ("cyTarget", named("Float")),
        ("radiusTarget", named("Float")),
        ("rxTarget", named("Float")),
        ("ryTarget", named("Float")),
        ("widthTarget", named("Float")),
        ("heightTarget", named("Float")),
        ("opacityTarget", named("Float")),
    ] {
        exports.push(function_export(
            module,
            name,
            ["Action"],
            Vec::new(),
            vec![element_ref()],
            binding_target(value),
        ));
    }
    exports.push(function_export(
        module,
        "toHtml",
        ["Action"],
        Vec::new(),
        vec![svg()],
        external_type(
            "Html",
            "std/web/html::Html",
            "std/web/html",
            "Html",
            vec![named("Action")],
        ),
    ));

    ModuleInterface {
        namespace_deprecations: Vec::new(),
        operator_deprecations: Vec::new(),
        schema: 1,
        module: module.to_owned(),
        source: "std/web/svg.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

/// The props record of one standard SVG tag function, or None for other names.
pub(super) fn svg_tag_props(tag: &str) -> Option<InterfaceType> {
    if !SVG_TAGS.contains(&tag) {
        return None;
    }
    let svg = || named_with("Svg", vec![named("Action")]);
    let html_type = |name: &str, arguments: Vec<InterfaceType>| {
        external_type(
            name,
            &format!("std/web/html::{name}"),
            "std/web/html",
            name,
            arguments,
        )
    };
    let event_action = || html_type("EventAction", vec![named("Action")]);
    let handler = |event: &str| function_type(vec![html_type(event, Vec::new())], event_action());
    let mut fields = vec![
        optional("elementRef", html_type("ElementRef", Vec::new())),
        optional("id", named("String")),
        optional("class", named("String")),
        optional("style", html_type("Style", Vec::new())),
        optional("role", named("String")),
        optional("ariaLabel", named("String")),
        optional("transform", named("String")),
        optional("fill", named("String")),
        optional("stroke", named("String")),
        optional("strokeWidth", named("String")),
        optional("strokeLinecap", named("String")),
        optional("strokeLinejoin", named("String")),
        optional("strokeDasharray", named("String")),
        optional("pointerEvents", named("String")),
        optional("onPointerDown", handler("PointerEvent")),
        optional("onPointerMove", handler("PointerEvent")),
        optional("onPointerUp", handler("PointerEvent")),
        optional("onPointerCancel", handler("PointerEvent")),
        optional("onWheel", handler("WheelEvent")),
    ];
    fields.extend(
        [
            "x", "y", "x1", "y1", "x2", "y2", "cx", "cy", "r", "rx", "ry", "width", "height",
            "opacity",
        ]
        .into_iter()
        .map(|name| optional(name, named("Float"))),
    );
    match tag {
        "svg" => {
            fields.push(optional("viewBox", named("String")));
            fields.push(optional("preserveAspectRatio", named("String")));
            fields.push(required("children", named_with("Array", vec![svg()])));
        }
        "g" => fields.push(required("children", named_with("Array", vec![svg()]))),
        "path" => fields.push(required("d", named("String"))),
        "polyline" | "polygon" => fields.push(required("points", named("String"))),
        "text" => {
            fields.push(optional("textAnchor", named("String")));
            fields.push(optional("dominantBaseline", named("String")));
            fields.push(optional("fontSize", named("String")));
            fields.push(optional("fontWeight", named("String")));
            fields.push(optional("letterSpacing", named("String")));
            fields.push(required("children", named("String")));
        }
        _ => {}
    }
    Some(record_vec(fields))
}
