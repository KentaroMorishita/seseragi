use super::*;

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
    let event_action = || {
        external_type(
            "EventAction",
            "std/web/html::EventAction",
            "std/web/html",
            "EventAction",
            vec![named("Action")],
        )
    };
    let event = |name: &str| {
        external_type(
            name,
            &format!("std/web/html::{name}"),
            "std/web/html",
            name,
            Vec::new(),
        )
    };
    let common_props = || {
        vec![
            optional("elementRef", element_ref()),
            optional("id", named("String")),
            optional("class", named("String")),
            optional("role", named("String")),
            optional("ariaLabel", named("String")),
            optional("transform", named("String")),
            optional("fill", named("String")),
            optional("stroke", named("String")),
            optional("strokeWidth", named("String")),
            optional("strokeLinecap", named("String")),
            optional("strokeLinejoin", named("String")),
            optional("pointerEvents", named("String")),
            optional(
                "onPointerDown",
                function_type(vec![event("PointerEvent")], event_action()),
            ),
            optional(
                "onPointerMove",
                function_type(vec![event("PointerEvent")], event_action()),
            ),
            optional(
                "onPointerUp",
                function_type(vec![event("PointerEvent")], event_action()),
            ),
            optional(
                "onPointerCancel",
                function_type(vec![event("PointerEvent")], event_action()),
            ),
            optional(
                "onWheel",
                function_type(vec![event("WheelEvent")], event_action()),
            ),
        ]
    };
    let numeric_props = || {
        [
            "x", "y", "x1", "y1", "x2", "y2", "cx", "cy", "r", "rx", "ry", "width", "height",
            "opacity",
        ]
        .into_iter()
        .map(|name| optional(name, named("Float")))
        .collect::<Vec<_>>()
    };
    let container_props = |root: bool| {
        let mut fields = common_props();
        fields.extend(numeric_props());
        if root {
            fields.push(optional("viewBox", named("String")));
            fields.push(optional("preserveAspectRatio", named("String")));
        }
        fields.push(required("children", named_with("Array", vec![svg()])));
        record_vec(fields)
    };
    let leaf_props = |kind: &str| {
        let mut fields = common_props();
        fields.extend(numeric_props());
        if kind == "path" {
            fields.push(required("d", named("String")));
        }
        if kind == "polyline" || kind == "polygon" {
            fields.push(required("points", named("String")));
        }
        record_vec(fields)
    };
    let mut text_props = common_props();
    text_props.extend(numeric_props());
    text_props.push(optional("textAnchor", named("String")));
    text_props.push(optional("dominantBaseline", named("String")));
    text_props.push(required("children", named("String")));

    let mut exports = vec![type_export(module, "Svg", 1, "opaque-type")];
    for (name, props) in [
        ("svg", container_props(true)),
        ("g", container_props(false)),
        ("rect", leaf_props("rect")),
        ("path", leaf_props("path")),
        ("circle", leaf_props("circle")),
        ("line", leaf_props("line")),
        ("polyline", leaf_props("polyline")),
        ("polygon", leaf_props("polygon")),
        ("text", record_vec(text_props)),
    ] {
        exports.push(function_export(
            module,
            name,
            ["Action"],
            Vec::new(),
            vec![props],
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
