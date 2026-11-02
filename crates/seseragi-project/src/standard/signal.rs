use super::*;

pub(super) fn signal_interface() -> ModuleInterface {
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
        function_export(
            "std/signal",
            "distinct",
            ["A"],
            vec![InterfaceConstraint {
                name: "Eq".to_owned(),
                trait_identity: Some("std/prelude::Eq".to_owned()),
                arguments: vec![named("A")],
            }],
            vec![signal_type("Signal", named("A"))],
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
        namespace_deprecations: Vec::new(),
        operator_deprecations: Vec::new(),
        schema: 1,
        module: "std/signal".to_owned(),
        source: "std/signal.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}
