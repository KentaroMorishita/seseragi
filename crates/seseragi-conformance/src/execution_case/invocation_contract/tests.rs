use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::{
    ByteSpan, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceScheme,
    InterfaceType, Visibility,
};

use super::validate_final_interface_invocation;
use crate::execution::{Invocation, InvocationArgument};

fn named(name: &str) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn function(parameter: InterfaceType, result: InterfaceType) -> InterfaceType {
    InterfaceType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
}

fn interface(entry_type: InterfaceType) -> TypedModuleInterface {
    TypedModuleInterface {
        schema: 1,
        stage: "typed-interface".to_owned(),
        module: "fixture/main".to_owned(),
        source: "src/main.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports: vec![InterfaceExport {
            symbol: "fixture/main::main".to_owned(),
            namespace: "value".to_owned(),
            name: "main".to_owned(),
            constructor_of: None,
            visibility: Visibility::Public,
            declaration_kind: Some("function".to_owned()),
            declaration: ByteSpan { start: 0, end: 8 },
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: entry_type,
            },
            representation: None,
        }],
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn type_export(name: &str, arity: u32) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("fixture/main::{name}"),
        namespace: "type".to_owned(),
        name: name.to_owned(),
        constructor_of: None,
        visibility: Visibility::Public,
        declaration_kind: Some("type".to_owned()),
        declaration: ByteSpan { start: 9, end: 17 },
        scheme: InterfaceScheme {
            type_parameters: Vec::new(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity,
            },
        },
        representation: None,
    }
}

#[test]
fn accepts_supported_arguments_across_a_curried_signature() {
    let typed = interface(function(
        named("String"),
        function(named("Unit"), named("String")),
    ));
    let invocation = Invocation::PureJson {
        arguments: vec![
            InvocationArgument::String("rock".to_owned()),
            InvocationArgument::Unit,
        ],
    };

    validate_final_interface_invocation(&typed, "main", &invocation).unwrap();
}

#[test]
fn rejects_non_final_interfaces_and_missing_value_exports() {
    let mut shallow = interface(function(named("Unit"), named("String")));
    shallow.stage = "module-interface".to_owned();
    let invocation = Invocation::PureJson {
        arguments: vec![InvocationArgument::Unit],
    };

    assert!(
        validate_final_interface_invocation(&shallow, "main", &invocation)
            .unwrap_err()
            .contains("final schema 1 typed interface")
    );
    assert!(validate_final_interface_invocation(
        &interface(function(named("Unit"), named("String"))),
        "missing",
        &invocation,
    )
    .unwrap_err()
    .contains("missing from typed interface"));
}

#[test]
fn rejects_argument_type_and_curried_arity_mismatches() {
    let two_parameters = interface(function(
        named("String"),
        function(named("Unit"), named("String")),
    ));
    let wrong_type = Invocation::PureJson {
        arguments: vec![InvocationArgument::Unit, InvocationArgument::Unit],
    };
    let error =
        validate_final_interface_invocation(&two_parameters, "main", &wrong_type).unwrap_err();
    assert!(error.contains("argument 1 expects String, got Unit"));

    let too_few = Invocation::PureJson {
        arguments: vec![InvocationArgument::String("rock".to_owned())],
    };
    let error = validate_final_interface_invocation(&two_parameters, "main", &too_few).unwrap_err();
    assert!(error.contains("expects 2 arguments, got 1"));

    let one_parameter = interface(function(named("Unit"), named("String")));
    let too_many = Invocation::PureJson {
        arguments: vec![InvocationArgument::Unit, InvocationArgument::Unit],
    };
    let error = validate_final_interface_invocation(&one_parameter, "main", &too_many).unwrap_err();
    assert!(error.contains("expects 1 arguments, got 2"));
}

#[test]
fn rejects_invocation_modes_that_disagree_with_the_typed_tail() {
    let effect_result = InterfaceType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            InterfaceType::Record {
                closed: true,
                fields: Vec::new(),
            },
            named("Never"),
            named("Unit"),
        ],
    };
    let effect_entry = interface(function(named("Unit"), effect_result));
    let pure = Invocation::PureJson {
        arguments: vec![InvocationArgument::Unit],
    };
    let error = validate_final_interface_invocation(&effect_entry, "main", &pure).unwrap_err();
    assert!(error.contains("cannot use pure invocation"));

    let pure_entry = interface(function(named("Unit"), named("String")));
    let effect = Invocation::Effect {
        arguments: vec![InvocationArgument::Unit],
    };
    let error = validate_final_interface_invocation(&pure_entry, "main", &effect).unwrap_err();
    assert!(error.contains("must return Effect<R, E, A>"));
}

#[test]
fn does_not_treat_shadowed_standard_spellings_as_runner_types() {
    let mut local_string = interface(function(named("String"), named("String")));
    local_string.exports.push(type_export("String", 0));
    let string = Invocation::PureJson {
        arguments: vec![InvocationArgument::String("value".to_owned())],
    };
    let error = validate_final_interface_invocation(&local_string, "main", &string).unwrap_err();
    assert!(error.contains("requires standard String"));
    assert!(error.contains("resolves to user type fixture/main::String"));

    let mut imported_unit = interface(function(named("Unit"), named("String")));
    imported_unit.dependencies.push(InterfaceDependency {
        specifier: "./domain".to_owned(),
        module: "fixture/domain".to_owned(),
        origin: ByteSpan { start: 0, end: 8 },
        imports: vec![InterfaceImport {
            namespace: "type".to_owned(),
            name: "ForeignUnit".to_owned(),
            symbol: "fixture/domain::ForeignUnit".to_owned(),
            local_name: Some("Unit".to_owned()),
        }],
    });
    let unit = Invocation::PureJson {
        arguments: vec![InvocationArgument::Unit],
    };
    let error = validate_final_interface_invocation(&imported_unit, "main", &unit).unwrap_err();
    assert!(error.contains("requires standard Unit"));

    let mut local_effect = interface(function(
        named("Unit"),
        InterfaceType::Named {
            name: "Effect".to_owned(),
            arguments: vec![named("Unit"), named("Unit"), named("Unit")],
        },
    ));
    local_effect.exports.push(type_export("Effect", 3));
    let effect = Invocation::Effect {
        arguments: vec![InvocationArgument::Unit],
    };
    let error = validate_final_interface_invocation(&local_effect, "main", &effect).unwrap_err();
    assert!(error.contains("must return Effect<R, E, A>"));
}
