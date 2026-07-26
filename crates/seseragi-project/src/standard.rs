use crate::ModuleLinkTarget;
use seseragi_syntax::{
    ByteSpan, InterfaceConstraint, InterfaceExport, InterfaceRecordField, InterfaceScheme,
    InterfaceType, ModuleInterface, TypeParameter, Visibility,
};

const ORIGIN: ByteSpan = ByteSpan { start: 0, end: 0 };

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardHtmlTagKind {
    Element,
    VoidElement,
    Link,
    Anchor,
    Image,
    Source,
    Video,
    Audio,
    Button,
    Form,
    Label,
    Input,
    Textarea,
    Select,
    Option,
    TableCell,
    OpenElement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardHtmlTag {
    pub name: &'static str,
    pub kind: StandardHtmlTagKind,
    pub void_element: bool,
}

macro_rules! html_tag {
    ($name:literal, $kind:ident) => {
        StandardHtmlTag {
            name: $name,
            kind: StandardHtmlTagKind::$kind,
            void_element: false,
        }
    };
    ($name:literal, $kind:ident, void) => {
        StandardHtmlTag {
            name: $name,
            kind: StandardHtmlTagKind::$kind,
            void_element: true,
        }
    };
}

pub const STANDARD_HTML_TAGS: &[StandardHtmlTag] = &[
    html_tag!("html", Element),
    html_tag!("head", Element),
    html_tag!("body", Element),
    html_tag!("title", Element),
    html_tag!("meta", VoidElement, void),
    html_tag!("link", Link, void),
    html_tag!("header", Element),
    html_tag!("footer", Element),
    html_tag!("nav", Element),
    html_tag!("article", Element),
    html_tag!("aside", Element),
    html_tag!("div", Element),
    html_tag!("span", Element),
    html_tag!("p", Element),
    html_tag!("main", Element),
    html_tag!("section", Element),
    html_tag!("h1", Element),
    html_tag!("h2", Element),
    html_tag!("h3", Element),
    html_tag!("h4", Element),
    html_tag!("h5", Element),
    html_tag!("h6", Element),
    html_tag!("strong", Element),
    html_tag!("em", Element),
    html_tag!("small", Element),
    html_tag!("code", Element),
    html_tag!("pre", Element),
    html_tag!("blockquote", Element),
    html_tag!("ul", Element),
    html_tag!("ol", Element),
    html_tag!("li", Element),
    html_tag!("br", VoidElement, void),
    html_tag!("hr", VoidElement, void),
    html_tag!("a", Anchor),
    html_tag!("img", Image, void),
    html_tag!("picture", Element),
    html_tag!("source", Source, void),
    html_tag!("video", Video),
    html_tag!("audio", Audio),
    html_tag!("button", Button),
    html_tag!("form", Form),
    html_tag!("label", Label),
    html_tag!("input", Input, void),
    html_tag!("textarea", Textarea),
    html_tag!("select", Select),
    html_tag!("option", Option),
    html_tag!("fieldset", Element),
    html_tag!("legend", Element),
    html_tag!("table", Element),
    html_tag!("thead", Element),
    html_tag!("tbody", Element),
    html_tag!("tfoot", Element),
    html_tag!("tr", Element),
    html_tag!("th", TableCell),
    html_tag!("td", TableCell),
    html_tag!("caption", Element),
    html_tag!("details", OpenElement),
    html_tag!("summary", Element),
    html_tag!("dialog", OpenElement),
];

pub fn standard_html_tag(name: &str) -> Option<StandardHtmlTag> {
    STANDARD_HTML_TAGS
        .iter()
        .copied()
        .find(|tag| tag.name == name)
}

pub fn standard_html_tag_props(name: &str) -> Option<(StandardHtmlTag, Vec<InterfaceRecordField>)> {
    let tag = standard_html_tag(name)?;
    let InterfaceType::Record { fields, .. } = props_for_html_tag(tag) else {
        unreachable!("standard HTML tag props must be a record")
    };
    Some((tag, fields))
}

pub fn is_standard_void_html_tag(name: &str) -> bool {
    standard_html_tag(name).is_some_and(|tag| tag.void_element)
}

struct StandardModuleDefinition {
    specifier: &'static str,
    interface: fn() -> ModuleInterface,
}

const STANDARD_MODULES: &[StandardModuleDefinition] = &[
    StandardModuleDefinition {
        specifier: "std/array",
        interface: array_interface,
    },
    StandardModuleDefinition {
        specifier: "std/list",
        interface: list_interface,
    },
    StandardModuleDefinition {
        specifier: "std/web/html",
        interface: web_html_interface,
    },
    StandardModuleDefinition {
        specifier: "std/web/dom",
        interface: web_dom_interface,
    },
    StandardModuleDefinition {
        specifier: "std/signal",
        interface: signal_interface,
    },
];

fn array_interface() -> ModuleInterface {
    collection_interface("std/array", "Array", "toList", "List")
}

fn list_interface() -> ModuleInterface {
    collection_interface("std/list", "List", "toArray", "Array")
}

fn collection_interface(
    module: &str,
    collection: &str,
    conversion: &str,
    conversion_target: &str,
) -> ModuleInterface {
    let values = named_with(collection, vec![named("A")]);
    let mapped_values = named_with(collection, vec![named("B")]);
    let maybe_value = named_with("Maybe", vec![named("A")]);
    let exports = vec![
        function_export(
            module,
            conversion,
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named_with(conversion_target, vec![named("A")]),
        ),
        function_export(
            module,
            "filter",
            ["A"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("Bool")),
                values.clone(),
            ],
            values.clone(),
        ),
        function_export(
            module,
            "filterMap",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named_with("Maybe", vec![named("B")])),
                values.clone(),
            ],
            mapped_values.clone(),
        ),
        function_export(
            module,
            "flatMap",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], mapped_values.clone()),
                values.clone(),
            ],
            mapped_values,
        ),
        function_export(
            module,
            "find",
            ["A"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("Bool")),
                values.clone(),
            ],
            maybe_value.clone(),
        ),
        function_export(
            module,
            "take",
            ["A"],
            Vec::new(),
            vec![named("Int"), values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "drop",
            ["A"],
            Vec::new(),
            vec![named("Int"), values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "append",
            ["A"],
            Vec::new(),
            vec![values.clone(), values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "concat",
            ["A"],
            Vec::new(),
            vec![named_with(collection, vec![values.clone()])],
            values.clone(),
        ),
        function_export(
            module,
            "reverse",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "length",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named("Int"),
        ),
        function_export(
            module,
            "isEmpty",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named("Bool"),
        ),
        function_export(
            module,
            "get",
            ["A"],
            Vec::new(),
            vec![named("Int"), values.clone()],
            maybe_value.clone(),
        ),
        function_export(
            module,
            "head",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            maybe_value,
        ),
        function_export(
            module,
            "tail",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named_with("Maybe", vec![values]),
        ),
    ];
    ModuleInterface {
        schema: 1,
        module: module.to_owned(),
        source: format!("{module}.ssrg"),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

/// Returns the compiler-owned public interface for a standard module.
///
/// Standard modules participate in ordinary project linking. Their
/// implementation is selected later by the target runtime ABI, so they never
/// become filesystem graph nodes or generated source-module imports.
pub fn standard_module_target(specifier: &str) -> Option<ModuleLinkTarget> {
    STANDARD_MODULES
        .iter()
        .find(|module| module.specifier == specifier)
        .map(|module| ModuleLinkTarget::external((module.interface)()))
}

/// Returns every compiler-owned standard module interface.
///
/// Tooling consumes the same interface registry as the linker so Reference,
/// hover, and future completion surfaces cannot drift from compilation.
pub fn standard_module_interfaces() -> Vec<ModuleInterface> {
    STANDARD_MODULES
        .iter()
        .map(|module| (module.interface)())
        .collect()
}

fn web_dom_interface() -> ModuleInterface {
    let exports = vec![
        type_export("std/web/dom", "Dom", 0, "opaque-type"),
        type_export("std/web/dom", "DomOptions", 0, "opaque-type"),
        type_export("std/web/dom", "DomTarget", 0, "opaque-type"),
        type_export("std/web/dom", "DomError", 0, "opaque-type"),
        type_export("std/web/dom", "DomRuntimeError", 1, "opaque-type"),
        function_export(
            "std/web/dom",
            "defaultOptions",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("DomOptions"),
        ),
        function_export(
            "std/web/dom",
            "query",
            [],
            Vec::new(),
            vec![named("String")],
            effect(
                record([required("dom", named("Dom"))]),
                named("DomError"),
                named("DomTarget"),
            ),
        ),
        function_export(
            "std/web/dom",
            "run",
            ["Action"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(record([]), named("Never"), named("Unit")),
                ),
                external_type(
                    "Signal",
                    "std/signal::Signal",
                    "std/signal",
                    "Signal",
                    vec![external_type(
                        "Html",
                        "std/web/html::Html",
                        "std/web/html",
                        "Html",
                        vec![named("Action")],
                    )],
                ),
            ],
            effect(
                record([required("dom", named("Dom"))]),
                named_with("DomRuntimeError", vec![named("Never")]),
                named("Unit"),
            ),
        ),
        function_export(
            "std/web/dom",
            "app",
            ["State", "Action"],
            Vec::new(),
            vec![record([
                required("target", named("String")),
                required("initial", named("State")),
                required(
                    "update",
                    function_type(vec![named("Action"), named("State")], named("State")),
                ),
                required(
                    "view",
                    function_type(
                        vec![named("State")],
                        external_type(
                            "Html",
                            "std/web/html::Html",
                            "std/web/html",
                            "Html",
                            vec![named("Action")],
                        ),
                    ),
                ),
            ])],
            effect(
                record([required("dom", named("Dom"))]),
                named("String"),
                named("Unit"),
            ),
        ),
    ];
    ModuleInterface {
        schema: 1,
        module: "std/web/dom".to_owned(),
        source: "std/web/dom.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

pub fn is_standard_module(specifier: &str) -> bool {
    STANDARD_MODULES
        .iter()
        .any(|module| module.specifier == specifier)
}

fn web_html_interface() -> ModuleInterface {
    let mut exports = vec![
        type_export("std/web/html", "Html", 1, "opaque-type"),
        type_export("std/web/html", "Style", 0, "opaque-type"),
        type_export("std/web/html", "Tag", 0, "opaque-type"),
        type_export("std/web/html", "Attribute", 0, "opaque-type"),
        record_type_export(
            "std/web/html",
            "InputEvent",
            [required("value", named("String"))],
        ),
        record_type_export(
            "std/web/html",
            "ChangeEvent",
            [
                required("value", named("String")),
                required("checked", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "KeyboardEvent",
            [
                required("key", named("String")),
                required("code", named("String")),
                required("repeat", named("Bool")),
                required("altKey", named("Bool")),
                required("controlKey", named("Bool")),
                required("metaKey", named("Bool")),
                required("shiftKey", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "MouseEvent",
            [
                required("button", named("Int")),
                required("clientX", named("Float")),
                required("clientY", named("Float")),
                required("altKey", named("Bool")),
                required("controlKey", named("Bool")),
                required("metaKey", named("Bool")),
                required("shiftKey", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "PointerEvent",
            [
                required("pointerId", named("Int")),
                required("pointerType", named("String")),
                required("isPrimary", named("Bool")),
                required("button", named("Int")),
                required("clientX", named("Float")),
                required("clientY", named("Float")),
                required("pressure", named("Float")),
                required("altKey", named("Bool")),
                required("controlKey", named("Bool")),
                required("metaKey", named("Bool")),
                required("shiftKey", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "ScrollEvent",
            [
                required("scrollLeft", named("Float")),
                required("scrollTop", named("Float")),
            ],
        ),
        opaque_adt_type_export("std/web/html", "EventAction", ["Action"]),
        constructor_export(
            "std/web/html",
            "EventAction",
            "IgnoreEvent",
            ["Action"],
            None,
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "Dispatch",
            ["Action"],
            Some(named("Action")),
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "DispatchPreventDefault",
            ["Action"],
            Some(named("Action")),
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "DispatchStopPropagation",
            ["Action"],
            Some(named("Action")),
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "DispatchPreventDefaultAndStop",
            ["Action"],
            Some(named("Action")),
        ),
        adt_type_export("std/web/html", "HtmlBuildError", []),
        constructor_export(
            "std/web/html",
            "HtmlBuildError",
            "InvalidTagName",
            [],
            Some(named("String")),
        ),
        constructor_export(
            "std/web/html",
            "HtmlBuildError",
            "InvalidAttributeName",
            [],
            Some(named("String")),
        ),
        constructor_export(
            "std/web/html",
            "HtmlBuildError",
            "ReservedAttributeName",
            [],
            Some(named("String")),
        ),
        trait_export("std/web/html", "IntoChildren", ["C", "Action"]),
        trait_export("std/web/html", "StyleRecord", ["R"]),
        function_export(
            "std/web/html",
            "style",
            ["R"],
            vec![InterfaceConstraint {
                name: "StyleRecord".to_owned(),
                trait_identity: Some("std/web/html::trait(StyleRecord)".to_owned()),
                arguments: vec![named("R")],
            }],
            vec![named("R")],
            named("Style"),
        ),
        function_export(
            "std/web/html",
            "customTag",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![named("HtmlBuildError"), named("Tag")]),
        ),
        function_export(
            "std/web/html",
            "attribute",
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            named_with("Either", vec![named("HtmlBuildError"), named("Attribute")]),
        ),
        function_export(
            "std/web/html",
            "text",
            ["Action"],
            Vec::new(),
            vec![named("String")],
            html(named("Action")),
        ),
        constrained_html_function("fragment", fragment_parameter()),
    ];
    exports.extend(html_props_aliases());
    exports.extend(STANDARD_HTML_TAGS.iter().copied().map(html_tag_export));
    exports.push(function_export(
        "std/web/html",
        "custom",
        ["Action", "C"],
        vec![InterfaceConstraint {
            name: "IntoChildren".to_owned(),
            trait_identity: Some("std/web/html::trait(IntoChildren)".to_owned()),
            arguments: vec![named("C"), named("Action")],
        }],
        vec![named("Tag"), element_props()],
        html(named("Action")),
    ));
    for renderer in ["renderToString", "renderDocument"] {
        exports.push(function_export(
            "std/web/html",
            renderer,
            ["Action"],
            Vec::new(),
            vec![html(named("Action"))],
            named("String"),
        ));
    }

    ModuleInterface {
        schema: 1,
        module: "std/web/html".to_owned(),
        source: "std/web/html.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn html_props_aliases() -> Vec<InterfaceExport> {
    vec![
        alias_type_export(
            "std/web/html",
            "ElementProps",
            ["Action", "C"],
            element_props(),
        ),
        alias_type_export(
            "std/web/html",
            "ButtonProps",
            ["Action", "C"],
            button_props(),
        ),
        alias_type_export("std/web/html", "FormProps", ["Action", "C"], form_props()),
        alias_type_export("std/web/html", "LabelProps", ["Action", "C"], label_props()),
        alias_type_export("std/web/html", "InputProps", ["Action"], input_props()),
        alias_type_export(
            "std/web/html",
            "TextareaProps",
            ["Action"],
            textarea_props(),
        ),
        alias_type_export(
            "std/web/html",
            "AnchorProps",
            ["Action", "C"],
            anchor_props(),
        ),
        alias_type_export("std/web/html", "ImageProps", ["Action"], image_props()),
    ]
}

fn html_tag_export(tag: StandardHtmlTag) -> InterfaceExport {
    let props = props_for_html_tag(tag);
    if tag.void_element || tag.kind == StandardHtmlTagKind::Textarea {
        function_export(
            "std/web/html",
            tag.name,
            ["Action"],
            Vec::new(),
            vec![props],
            html(named("Action")),
        )
    } else {
        constrained_html_function(tag.name, props)
    }
}

fn signal_interface() -> ModuleInterface {
    let exports = vec![
        type_export("std/signal", "Signal", 1, "opaque-type"),
        type_export("std/signal", "MutableSignal", 1, "opaque-type"),
        type_export("std/signal", "SignalChange", 0, "opaque-type"),
        type_export("std/signal", "Subscription", 0, "opaque-type"),
        signal_function(
            "make",
            ["A"],
            vec![named("A")],
            task(signal_type("MutableSignal", named("A"))),
        ),
        signal_function(
            "read",
            ["A"],
            vec![signal_type("Signal", named("A"))],
            task(named("A")),
        ),
        signal_function(
            "set",
            ["A"],
            vec![named("A"), signal_type("MutableSignal", named("A"))],
            task(named("Unit")),
        ),
        signal_function(
            "update",
            ["A"],
            vec![
                function_type(vec![named("A")], named("A")),
                signal_type("MutableSignal", named("A")),
            ],
            task(named("Unit")),
        ),
        signal_function(
            "planSet",
            ["A"],
            vec![named("A"), signal_type("MutableSignal", named("A"))],
            named("SignalChange"),
        ),
        signal_function(
            "planUpdate",
            ["A"],
            vec![
                function_type(vec![named("A")], named("A")),
                signal_type("MutableSignal", named("A")),
            ],
            named("SignalChange"),
        ),
        signal_function(
            "transaction",
            [],
            vec![named_with("Array", vec![named("SignalChange")])],
            task(named("Unit")),
        ),
        signal_function(
            "map",
            ["A", "B"],
            vec![
                function_type(vec![named("A")], named("B")),
                signal_type("Signal", named("A")),
            ],
            signal_type("Signal", named("B")),
        ),
        signal_function(
            "combine",
            ["A", "B", "C"],
            vec![
                function_type(vec![named("A"), named("B")], named("C")),
                signal_type("Signal", named("A")),
                signal_type("Signal", named("B")),
            ],
            signal_type("Signal", named("C")),
        ),
        signal_function(
            "constant",
            ["A"],
            vec![named("A")],
            signal_type("Signal", named("A")),
        ),
        signal_function(
            "switchMap",
            ["A", "B"],
            vec![
                function_type(vec![named("A")], signal_type("Signal", named("B"))),
                signal_type("Signal", named("A")),
            ],
            signal_type("Signal", named("B")),
        ),
        signal_function(
            "subscribe",
            ["R", "A"],
            vec![
                function_type(
                    vec![named("A")],
                    effect(named("R"), named("Never"), named("Unit")),
                ),
                signal_type("Signal", named("A")),
            ],
            effect(named("R"), named("Never"), named("Subscription")),
        ),
        signal_function(
            "unsubscribe",
            [],
            vec![named("Subscription")],
            task(named("Unit")),
        ),
    ];
    ModuleInterface {
        schema: 1,
        module: "std/signal".to_owned(),
        source: "std/signal.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn signal_function<const N: usize>(
    name: &str,
    parameters: [&str; N],
    arguments: Vec<InterfaceType>,
    result: InterfaceType,
) -> InterfaceExport {
    function_export(
        "std/signal",
        name,
        parameters,
        Vec::new(),
        arguments,
        result,
    )
}

fn constrained_html_function(name: &str, parameter: InterfaceType) -> InterfaceExport {
    function_export(
        "std/web/html",
        name,
        ["Action", "C"],
        vec![InterfaceConstraint {
            name: "IntoChildren".to_owned(),
            trait_identity: Some("std/web/html::trait(IntoChildren)".to_owned()),
            arguments: vec![named("C"), named("Action")],
        }],
        vec![parameter],
        html(named("Action")),
    )
}

fn fragment_parameter() -> InterfaceType {
    named("C")
}

fn props_for_html_tag(tag: StandardHtmlTag) -> InterfaceType {
    use StandardHtmlTagKind as Kind;
    match tag.kind {
        Kind::Element => element_props(),
        Kind::VoidElement => void_element_props(),
        Kind::Link => link_props(),
        Kind::Anchor => anchor_props(),
        Kind::Image => image_props(),
        Kind::Source => source_props(),
        Kind::Video => video_props(),
        Kind::Audio => audio_props(),
        Kind::Button => button_props(),
        Kind::Form => form_props(),
        Kind::Label => label_props(),
        Kind::Input => input_props(),
        Kind::Textarea => textarea_props(),
        Kind::Select => select_props(),
        Kind::Option => option_props(),
        Kind::TableCell => table_cell_props(),
        Kind::OpenElement => open_element_props(),
    }
}

fn element_props() -> InterfaceType {
    with_children(common_html_props())
}

fn void_element_props() -> InterfaceType {
    record_vec(common_html_props())
}

fn link_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            required("rel", named("String")),
            required("href", named("String")),
        ],
    ))
}

fn anchor_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            required("href", named("String")),
            optional("target", named("String")),
            optional("rel", named("String")),
            optional("download", named("Bool")),
        ],
    ))
}

fn image_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            required("src", named("String")),
            required("alt", named("String")),
            optional("width", named("Int")),
            optional("height", named("Int")),
            optional("loading", named("String")),
        ],
    ))
}

fn source_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            required("src", named("String")),
            optional("media", named("String")),
            optional("mimeType", named("String")),
        ],
    ))
}

fn video_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("src", named("String")),
            optional("width", named("Int")),
            optional("height", named("Int")),
        ],
    ))
}

fn audio_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [optional("src", named("String"))],
    ))
}

fn button_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("disabled", named("Bool")),
            optional("buttonType", named("String")),
            optional("name", named("String")),
            optional("value", named("String")),
            optional("autoFocus", named("Bool")),
        ],
    ))
}

fn form_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("onSubmit", named("Action")),
            optional("name", named("String")),
            optional("autoComplete", named("String")),
        ],
    ))
}

fn label_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [optional("htmlFor", named("String"))],
    ))
}

fn input_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            optional("value", named("String")),
            optional("checked", named("Bool")),
            optional("name", named("String")),
            optional("disabled", named("Bool")),
            optional("required", named("Bool")),
            optional("readOnly", named("Bool")),
            optional("multiple", named("Bool")),
            optional("placeholder", named("String")),
            optional("autoComplete", named("String")),
            optional("autoFocus", named("Bool")),
            optional("min", named("String")),
            optional("max", named("String")),
            optional("step", named("String")),
            optional("pattern", named("String")),
            optional("inputType", named("String")),
            optional(
                "onInput",
                function_type(vec![html_event_type("InputEvent")], named("Action")),
            ),
            optional(
                "onChange",
                function_type(vec![html_event_type("ChangeEvent")], named("Action")),
            ),
        ],
    ))
}

fn textarea_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            optional("value", named("String")),
            optional("name", named("String")),
            optional("disabled", named("Bool")),
            optional("required", named("Bool")),
            optional("readOnly", named("Bool")),
            optional("placeholder", named("String")),
            optional("autoComplete", named("String")),
            optional("autoFocus", named("Bool")),
            optional("rows", named("Int")),
            optional("cols", named("Int")),
            optional(
                "onInput",
                function_type(vec![html_event_type("InputEvent")], named("Action")),
            ),
            optional(
                "onChange",
                function_type(vec![html_event_type("ChangeEvent")], named("Action")),
            ),
        ],
    ))
}

fn select_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("name", named("String")),
            optional("value", named("String")),
            optional("disabled", named("Bool")),
            optional("required", named("Bool")),
            optional("multiple", named("Bool")),
            optional("autoFocus", named("Bool")),
            optional(
                "onChange",
                function_type(vec![html_event_type("ChangeEvent")], named("Action")),
            ),
        ],
    ))
}

fn option_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("value", named("String")),
            optional("selected", named("Bool")),
            optional("disabled", named("Bool")),
        ],
    ))
}

fn table_cell_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("colSpan", named("Int")),
            optional("rowSpan", named("Int")),
        ],
    ))
}

fn open_element_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [optional("open", named("Bool"))],
    ))
}

fn common_html_props() -> Vec<InterfaceRecordField> {
    vec![
        optional("id", named("String")),
        optional("className", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("attributes", named_with("Array", vec![named("Attribute")])),
        optional("role", named("String")),
        optional("tabIndex", named("Int")),
        optional("lang", named("String")),
        optional("dir", named("String")),
        optional("draggable", named("Bool")),
        optional("contentEditable", named("Bool")),
        optional("onClick", named("Action")),
        optional("preventClickDefault", named("Bool")),
        optional("stopClickPropagation", named("Bool")),
        optional("onFocus", named("Action")),
        optional("onBlur", named("Action")),
        optional(
            "onKeyDown",
            function_type(vec![html_event_type("KeyboardEvent")], event_action_type()),
        ),
        optional(
            "onKeyUp",
            function_type(vec![html_event_type("KeyboardEvent")], event_action_type()),
        ),
        optional(
            "onMouseDown",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onMouseUp",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onPointerDown",
            function_type(vec![html_event_type("PointerEvent")], event_action_type()),
        ),
        optional(
            "onPointerUp",
            function_type(vec![html_event_type("PointerEvent")], event_action_type()),
        ),
        optional(
            "onDoubleClick",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onContextMenu",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onScroll",
            function_type(vec![html_event_type("ScrollEvent")], event_action_type()),
        ),
    ]
}

fn event_action_type() -> InterfaceType {
    external_type(
        "EventAction",
        "std/web/html::EventAction",
        "std/web/html",
        "EventAction",
        vec![named("Action")],
    )
}

fn with_fields<const N: usize>(
    mut fields: Vec<InterfaceRecordField>,
    additions: [InterfaceRecordField; N],
) -> Vec<InterfaceRecordField> {
    fields.extend(additions);
    fields
}

fn with_children(mut fields: Vec<InterfaceRecordField>) -> InterfaceType {
    fields.push(required("children", named("C")));
    record_vec(fields)
}

fn record_vec(fields: Vec<InterfaceRecordField>) -> InterfaceType {
    InterfaceType::Record {
        closed: true,
        fields,
    }
}

fn type_export(module: &str, name: &str, arity: u32, declaration_kind: &str) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "type".to_owned(),
        name: name.to_owned(),
        constructor_of: None,
        visibility: Visibility::Public,
        declaration_kind: Some(declaration_kind.to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: (0..arity)
                .map(|index| TypeParameter::value(format!("T{index}")))
                .collect(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity,
            },
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn alias_type_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
    representation: InterfaceType,
) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "type".to_owned(),
        name: name.to_owned(),
        constructor_of: None,
        visibility: Visibility::Public,
        declaration_kind: Some("alias".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity: N as u32,
            },
        },
        methods: Vec::new(),
        representation: Some(representation),
    }
}

fn adt_type_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
) -> InterfaceExport {
    let mut export = type_export(module, name, N as u32, "type");
    export.scheme.type_parameters = parameters.into_iter().map(TypeParameter::value).collect();
    export
}

fn opaque_adt_type_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
) -> InterfaceExport {
    let mut export = type_export(module, name, N as u32, "opaque-type");
    export.scheme.type_parameters = parameters.into_iter().map(TypeParameter::value).collect();
    export
}

fn constructor_export<const N: usize>(
    module: &str,
    owner: &str,
    name: &str,
    parameters: [&str; N],
    payload: Option<InterfaceType>,
) -> InterfaceExport {
    let arguments = parameters
        .iter()
        .map(|parameter| named(parameter))
        .collect::<Vec<_>>();
    let result = external_type(
        owner,
        &format!("{module}::{owner}"),
        module,
        owner,
        arguments,
    );
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "value".to_owned(),
        name: name.to_owned(),
        constructor_of: Some(format!("{module}::{owner}")),
        visibility: Visibility::Public,
        declaration_kind: Some("constructor".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints: Vec::new(),
            type_ref: payload.map_or(result.clone(), |payload| InterfaceType::Function {
                parameter: Box::new(payload),
                result: Box::new(result),
            }),
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn record_type_export<const N: usize>(
    module: &str,
    name: &str,
    fields: [InterfaceRecordField; N],
) -> InterfaceExport {
    let mut export = type_export(module, name, 0, "opaque-struct");
    export.representation = Some(record(fields));
    export
}

fn trait_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::trait({name})"),
        namespace: "trait".to_owned(),
        name: name.to_owned(),
        constructor_of: None,
        visibility: Visibility::Public,
        declaration_kind: Some("trait".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity: N as u32,
            },
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn function_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
    constraints: Vec<InterfaceConstraint>,
    arguments: Vec<InterfaceType>,
    result: InterfaceType,
) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "value".to_owned(),
        name: name.to_owned(),
        constructor_of: None,
        visibility: Visibility::Public,
        declaration_kind: Some("function".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints,
            type_ref: function_type(arguments, result),
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn html(action: InterfaceType) -> InterfaceType {
    InterfaceType::Named {
        name: "Html".to_owned(),
        arguments: vec![action],
    }
}

fn html_event_type(name: &str) -> InterfaceType {
    external_type(
        name,
        &format!("std/web/html::{name}"),
        "std/web/html",
        name,
        Vec::new(),
    )
}

fn named(name: &str) -> InterfaceType {
    named_with(name, Vec::new())
}

fn named_with(name: &str, arguments: Vec<InterfaceType>) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments,
    }
}

fn external_type(
    name: &str,
    canonical: &str,
    provider_module: &str,
    provider_export: &str,
    arguments: Vec<InterfaceType>,
) -> InterfaceType {
    InterfaceType::ExternalNamed {
        name: name.to_owned(),
        canonical: canonical.to_owned(),
        provider_module: provider_module.to_owned(),
        provider_export: provider_export.to_owned(),
        arguments,
    }
}

fn signal_type(name: &str, value: InterfaceType) -> InterfaceType {
    named_with(name, vec![value])
}

fn task(success: InterfaceType) -> InterfaceType {
    effect(record([]), named("Never"), success)
}

fn effect(
    environment: InterfaceType,
    failure: InterfaceType,
    success: InterfaceType,
) -> InterfaceType {
    named_with("Effect", vec![environment, failure, success])
}

fn function_type(parameters: Vec<InterfaceType>, result: InterfaceType) -> InterfaceType {
    parameters
        .into_iter()
        .rev()
        .fold(result, |result, parameter| InterfaceType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        })
}

fn record<const N: usize>(fields: [InterfaceRecordField; N]) -> InterfaceType {
    InterfaceType::Record {
        closed: true,
        fields: fields.into_iter().collect(),
    }
}

fn required(name: &str, type_ref: InterfaceType) -> InterfaceRecordField {
    InterfaceRecordField {
        name: name.to_owned(),
        optional: false,
        type_ref,
    }
}

fn optional(name: &str, type_ref: InterfaceType) -> InterfaceRecordField {
    InterfaceRecordField {
        name: name.to_owned(),
        optional: true,
        type_ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_compiler_owned_standard_modules_as_external_link_targets() {
        let target = standard_module_target("std/web/html").unwrap();

        assert_eq!(target.interface().module, "std/web/html");
        assert!(target
            .interface()
            .exports
            .iter()
            .any(|export| { export.namespace == "type" && export.name == "Html" }));
        assert!(target
            .interface()
            .exports
            .iter()
            .any(|export| { export.namespace == "value" && export.name == "renderToString" }));
        assert!(standard_module_target("std/web/missing").is_none());
        for (module, conversion) in [("std/array", "toList"), ("std/list", "toArray")] {
            let target = standard_module_target(module).unwrap();
            for name in [
                "filter",
                "filterMap",
                "flatMap",
                "find",
                "take",
                "drop",
                "append",
                "concat",
                "reverse",
                conversion,
                "length",
                "isEmpty",
                "get",
                "head",
                "tail",
            ] {
                assert!(target
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.namespace == "value" && export.name == name));
            }
        }
        assert!(standard_module_target("std/signal").is_some());
        let dom = standard_module_target("std/web/dom").unwrap();
        assert!(dom
            .interface()
            .exports
            .iter()
            .any(|export| export.namespace == "value" && export.name == "run"));
        assert!(dom
            .interface()
            .exports
            .iter()
            .any(|export| export.namespace == "value" && export.name == "app"));
    }

    #[test]
    fn exposes_typed_form_events_from_the_shared_html_interface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for name in ["form", "label", "input", "textarea"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value" && export.name == name));
        }
        let input_event = interface
            .exports
            .iter()
            .find(|export| export.namespace == "type" && export.name == "InputEvent")
            .unwrap();
        assert_eq!(
            input_event.declaration_kind.as_deref(),
            Some("opaque-struct")
        );
        let Some(InterfaceType::Record { fields, .. }) = &input_event.representation else {
            panic!("InputEvent must expose its immutable snapshot fields");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "value");

        let input = interface
            .exports
            .iter()
            .find(|export| export.namespace == "value" && export.name == "input")
            .unwrap();
        let InterfaceType::Function { parameter, .. } = &input.scheme.type_ref else {
            panic!("input must be callable");
        };
        let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
            panic!("input must accept a props record");
        };
        assert!(fields.iter().any(|field| field.name == "onInput"));
        assert!(fields.iter().any(|field| field.name == "onChange"));
        assert!(fields.iter().any(|field| field.name == "required"));
        assert!(fields.iter().any(|field| field.name == "inputType"));
    }

    #[test]
    fn exposes_focus_and_keyboard_events_from_the_shared_html_interface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        let keyboard_event = interface
            .exports
            .iter()
            .find(|export| export.namespace == "type" && export.name == "KeyboardEvent")
            .unwrap();
        assert_eq!(
            keyboard_event.declaration_kind.as_deref(),
            Some("opaque-struct")
        );
        let Some(InterfaceType::Record { fields, .. }) = &keyboard_event.representation else {
            panic!("KeyboardEvent must expose only immutable snapshot fields");
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            [
                "key",
                "code",
                "repeat",
                "altKey",
                "controlKey",
                "metaKey",
                "shiftKey",
            ]
        );

        let button = interface
            .exports
            .iter()
            .find(|export| export.namespace == "value" && export.name == "button")
            .unwrap();
        let InterfaceType::Function { parameter, .. } = &button.scheme.type_ref else {
            panic!("button must be callable");
        };
        let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
            panic!("button must accept a props record");
        };
        for name in ["onFocus", "onBlur", "onKeyDown", "onKeyUp"] {
            assert!(fields.iter().any(|field| field.name == name));
        }
    }

    #[test]
    fn exposes_pointer_scroll_and_event_control_surface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for name in ["EventAction", "MouseEvent", "PointerEvent", "ScrollEvent"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "type" && export.name == name));
        }
        for name in [
            "IgnoreEvent",
            "Dispatch",
            "DispatchPreventDefault",
            "DispatchStopPropagation",
            "DispatchPreventDefaultAndStop",
        ] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value"
                    && export.declaration_kind.as_deref() == Some("constructor")
                    && export.name == name));
        }

        let div = interface
            .exports
            .iter()
            .find(|export| export.namespace == "value" && export.name == "div")
            .unwrap();
        let InterfaceType::Function { parameter, .. } = &div.scheme.type_ref else {
            panic!("div must be callable");
        };
        let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
            panic!("div must accept a props record");
        };
        for name in [
            "preventClickDefault",
            "stopClickPropagation",
            "onMouseDown",
            "onMouseUp",
            "onPointerDown",
            "onPointerUp",
            "onDoubleClick",
            "onContextMenu",
            "onScroll",
        ] {
            assert!(fields.iter().any(|field| field.name == name));
        }
    }

    #[test]
    fn exposes_standard_html_tags_with_their_children_contract() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for tag in STANDARD_HTML_TAGS {
            let name = tag.name;
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            assert_eq!(
                fields.iter().any(|field| field.name == "children"),
                !tag.void_element && tag.kind != StandardHtmlTagKind::Textarea,
                "std/web/html::{name} has the wrong children contract"
            );
        }
    }

    #[test]
    fn exposes_props_aliases_and_derives_diagnostic_metadata_from_the_same_records() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for name in [
            "ElementProps",
            "ButtonProps",
            "FormProps",
            "LabelProps",
            "InputProps",
            "TextareaProps",
            "AnchorProps",
            "ImageProps",
        ] {
            let alias = interface
                .exports
                .iter()
                .find(|export| export.namespace == "type" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            assert_eq!(alias.declaration_kind.as_deref(), Some("alias"));
            assert!(matches!(
                alias.representation,
                Some(InterfaceType::Record { .. })
            ));
        }

        let (image, fields) = standard_html_tag_props("img").unwrap();
        assert!(image.void_element);
        assert!(fields
            .iter()
            .any(|field| field.name == "src" && !field.optional));
        assert!(fields
            .iter()
            .any(|field| field.name == "alt" && !field.optional));
        assert!(!fields.iter().any(|field| field.name == "children"));
        assert!(standard_html_tag_props("custom").is_none());
    }

    #[test]
    fn exposes_tag_specific_link_and_media_props() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for name in ["a", "img", "picture", "source", "video", "audio", "link"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value" && export.name == name));
        }

        let props = |name: &str| {
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap();
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>()
        };

        assert!(props("a").contains(&"href"));
        assert!(!props("article").contains(&"href"));
        assert!(props("img").contains(&"alt"));
        assert!(!props("img").contains(&"children"));
        assert!(!props("a").contains(&"alt"));
        assert!(!props("source").contains(&"children"));
        assert!(is_standard_void_html_tag("img"));
        assert!(is_standard_void_html_tag("source"));
        assert!(!is_standard_void_html_tag("picture"));
    }

    #[test]
    fn exposes_form_table_and_interactive_tag_props() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        let props = |name: &str| {
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>()
        };

        for name in [
            "select", "option", "fieldset", "legend", "table", "thead", "tbody", "tfoot", "tr",
            "th", "td", "caption", "details", "summary", "dialog",
        ] {
            assert!(props(name).contains(&"children"));
        }
        for name in ["input", "textarea"] {
            assert!(props(name).contains(&"readOnly"));
            assert!(props(name).contains(&"autoFocus"));
        }
        assert!(props("input").contains(&"multiple"));
        assert!(props("input").contains(&"pattern"));
        assert!(props("textarea").contains(&"rows"));
        assert!(props("textarea").contains(&"cols"));
        assert!(props("select").contains(&"onChange"));
        assert!(props("option").contains(&"selected"));
        assert!(props("th").contains(&"colSpan"));
        assert!(props("td").contains(&"rowSpan"));
        assert!(props("details").contains(&"open"));
        assert!(props("dialog").contains(&"open"));
        assert!(!props("table").contains(&"colSpan"));
    }

    #[test]
    fn exposes_global_and_validated_custom_html_surface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        let props = |name: &str| {
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>()
        };

        for tag in ["div", "img", "input", "table", "dialog"] {
            let fields = props(tag);
            for field in [
                "attributes",
                "role",
                "tabIndex",
                "lang",
                "dir",
                "draggable",
                "contentEditable",
            ] {
                assert!(fields.contains(&field), "std/web/html::{tag} lacks {field}");
            }
        }
        for name in ["Tag", "Attribute", "HtmlBuildError"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "type" && export.name == name));
        }
        for name in [
            "customTag",
            "attribute",
            "custom",
            "InvalidTagName",
            "InvalidAttributeName",
            "ReservedAttributeName",
        ] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value" && export.name == name));
        }
    }
}
