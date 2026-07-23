use crate::ModuleLinkTarget;
use seseragi_syntax::{
    ByteSpan, InterfaceConstraint, InterfaceExport, InterfaceRecordField, InterfaceScheme,
    InterfaceType, ModuleInterface, TypeParameter, Visibility,
};

const ORIGIN: ByteSpan = ByteSpan { start: 0, end: 0 };

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
    collection_interface("std/array", "Array")
}

fn list_interface() -> ModuleInterface {
    collection_interface("std/list", "List")
}

fn collection_interface(module: &str, collection: &str) -> ModuleInterface {
    let values = named_with(collection, vec![named("A")]);
    let mapped_values = named_with(collection, vec![named("B")]);
    let maybe_value = named_with("Maybe", vec![named("A")]);
    let exports = vec![
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
            ["Msg"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Msg")],
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
                        vec![named("Msg")],
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
            ["State", "Msg"],
            Vec::new(),
            vec![record([
                required("target", named("String")),
                required("initial", named("State")),
                required(
                    "update",
                    function_type(vec![named("Msg"), named("State")], named("State")),
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
                            vec![named("Msg")],
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
        trait_export("std/web/html", "IntoChildren", ["C", "Msg"]),
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
            "text",
            ["Msg"],
            Vec::new(),
            vec![named("String")],
            html(named("Msg")),
        ),
        constrained_html_function("fragment", fragment_parameter()),
    ];
    for tag in ["div", "span", "p", "main", "section", "h1", "h2"] {
        exports.push(constrained_html_function(tag, element_props()));
    }
    exports.push(constrained_html_function("button", button_props()));
    exports.push(constrained_html_function("form", form_props()));
    exports.push(constrained_html_function("label", label_props()));
    exports.push(function_export(
        "std/web/html",
        "input",
        ["Msg"],
        Vec::new(),
        vec![input_props()],
        html(named("Msg")),
    ));
    exports.push(function_export(
        "std/web/html",
        "textarea",
        ["Msg"],
        Vec::new(),
        vec![textarea_props()],
        html(named("Msg")),
    ));
    for renderer in ["renderToString", "renderDocument"] {
        exports.push(function_export(
            "std/web/html",
            renderer,
            ["Msg"],
            Vec::new(),
            vec![html(named("Msg"))],
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
        ["Msg", "C"],
        vec![InterfaceConstraint {
            name: "IntoChildren".to_owned(),
            trait_identity: Some("std/web/html::trait(IntoChildren)".to_owned()),
            arguments: vec![named("C"), named("Msg")],
        }],
        vec![parameter],
        html(named("Msg")),
    )
}

fn fragment_parameter() -> InterfaceType {
    named("C")
}

fn element_props() -> InterfaceType {
    record([
        optional("id", named("String")),
        optional("className", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("onClick", named("Msg")),
        required("children", named("C")),
    ])
}

fn button_props() -> InterfaceType {
    record([
        optional("id", named("String")),
        optional("className", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("disabled", named("Bool")),
        optional("buttonType", named("String")),
        optional("onClick", named("Msg")),
        required("children", named("C")),
    ])
}

fn form_props() -> InterfaceType {
    record([
        optional("id", named("String")),
        optional("className", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("onClick", named("Msg")),
        optional("onSubmit", named("Msg")),
        required("children", named("C")),
    ])
}

fn label_props() -> InterfaceType {
    record([
        optional("id", named("String")),
        optional("className", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("htmlFor", named("String")),
        optional("onClick", named("Msg")),
        required("children", named("C")),
    ])
}

fn input_props() -> InterfaceType {
    record([
        optional("id", named("String")),
        optional("className", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("value", named("String")),
        optional("checked", named("Bool")),
        optional("name", named("String")),
        optional("disabled", named("Bool")),
        optional("required", named("Bool")),
        optional("placeholder", named("String")),
        optional("inputType", named("String")),
        optional(
            "onInput",
            function_type(vec![html_event_type("InputEvent")], named("Msg")),
        ),
        optional(
            "onChange",
            function_type(vec![html_event_type("ChangeEvent")], named("Msg")),
        ),
    ])
}

fn textarea_props() -> InterfaceType {
    record([
        optional("id", named("String")),
        optional("className", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("value", named("String")),
        optional("name", named("String")),
        optional("disabled", named("Bool")),
        optional("required", named("Bool")),
        optional("placeholder", named("String")),
        optional(
            "onInput",
            function_type(vec![html_event_type("InputEvent")], named("Msg")),
        ),
        optional(
            "onChange",
            function_type(vec![html_event_type("ChangeEvent")], named("Msg")),
        ),
    ])
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

fn html(message: InterfaceType) -> InterfaceType {
    InterfaceType::Named {
        name: "Html".to_owned(),
        arguments: vec![message],
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
        for module in ["std/array", "std/list"] {
            let target = standard_module_target(module).unwrap();
            for name in [
                "filter",
                "filterMap",
                "flatMap",
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
}
