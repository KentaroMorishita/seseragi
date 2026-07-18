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
        specifier: "std/web/html",
        interface: web_html_interface,
    },
    StandardModuleDefinition {
        specifier: "std/signal",
        interface: signal_interface,
    },
];

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

pub fn is_standard_module(specifier: &str) -> bool {
    STANDARD_MODULES
        .iter()
        .any(|module| module.specifier == specifier)
}

fn web_html_interface() -> ModuleInterface {
    let mut exports = vec![
        type_export("std/web/html", "Html", 1, "opaque-type"),
        trait_export("std/web/html", "IntoChildren", ["C", "Msg"]),
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
    exports.push(function_export(
        "std/web/html",
        "input",
        ["Msg"],
        Vec::new(),
        vec![input_props()],
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
        optional("disabled", named("Bool")),
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
        optional("value", named("String")),
        optional("checked", named("Bool")),
        optional("disabled", named("Bool")),
        optional("placeholder", named("String")),
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

fn named(name: &str) -> InterfaceType {
    named_with(name, Vec::new())
}

fn named_with(name: &str, arguments: Vec<InterfaceType>) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments,
    }
}

fn signal_type(name: &str, value: InterfaceType) -> InterfaceType {
    named_with(name, vec![value])
}

fn task(success: InterfaceType) -> InterfaceType {
    named_with("Effect", vec![record([]), named("Never"), success])
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
        assert!(standard_module_target("std/signal").is_some());
    }
}
