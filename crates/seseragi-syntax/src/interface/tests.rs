use super::*;
use crate::surface::ByteSpan;

#[test]
fn parses_basic_public_let_interface() {
    let interface =
        parse_module_interface("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
    let json = serde_json::to_value(&interface).expect("interface serializes");

    assert_eq!(json["schema"], 1);
    assert_eq!(json["module"], "artifact/basic");
    assert_eq!(json["source"], "main.ssrg");
    assert_eq!(json["dependencies"], serde_json::json!([]));
    assert_eq!(json["exports"][0]["symbol"], "artifact/basic::answer");
    assert_eq!(json["exports"][0]["namespace"], "value");
    assert_eq!(json["exports"][0]["name"], "answer");
    assert_eq!(json["exports"][0]["visibility"], "public");
    assert_eq!(
        json["exports"][0]["declaration"],
        serde_json::json!({
            "start": 0,
            "end": 24,
        })
    );
    assert_eq!(
        json["exports"][0]["scheme"],
        serde_json::json!({
            "typeParameters": [],
            "constraints": [],
            "type": {
                "kind": "named",
                "name": "Int",
                "arguments": [],
            },
        })
    );
    assert_eq!(json["operators"], serde_json::json!([]));
    assert_eq!(json["instances"], serde_json::json!([]));
}

#[test]
fn omits_private_lets_from_interface() {
    let interface = parse_module_interface(
        "artifact/multiple-lets/main.ssrg",
        "let first = 1\npub let second: Int = 2\n",
    );

    assert_eq!(interface.exports.len(), 1);
    assert_eq!(
        interface.exports[0].symbol,
        "artifact/multiple-lets::second"
    );
    assert_eq!(interface.exports[0].name, "second");
    assert_eq!(
        interface.exports[0].declaration,
        ByteSpan { start: 14, end: 37 }
    );
}

#[test]
fn omits_exports_when_module_has_parse_errors() {
    let interface = parse_module_interface("artifact/recovery/main.ssrg", "pub let answer: Int =");

    assert_eq!(interface.module, "artifact/recovery");
    assert_eq!(interface.source, "main.ssrg");
    assert!(interface.exports.is_empty());
}

#[test]
fn parses_public_effect_fn_interface() {
    let interface = parse_module_interface(
        "artifact/effect-do/main.ssrg",
        "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {}\n",
    );
    let json = serde_json::to_value(&interface).expect("interface serializes");

    assert_eq!(json["exports"][0]["symbol"], "artifact/effect-do::main");
    assert_eq!(
        json["exports"][0]["scheme"]["type"],
        serde_json::json!({
            "kind": "named",
            "name": "Unit",
            "arguments": [],
        })
    );
}

#[test]
fn parses_public_pure_fn_interface() {
    let interface = parse_module_interface(
        "artifact/pure-function/main.ssrg",
        "pub fn member<A> target: A -> values: List<A> -> Bool\nwhere Eq<A> =\n  contains target values\n",
    );
    let function_export = interface
        .exports
        .iter()
        .find(|export| export.name == "member")
        .expect("function export exists");

    assert_eq!(
        function_export.declaration_kind,
        Some("function".to_owned())
    );
    assert_eq!(function_export.namespace, "value");
    assert_eq!(function_export.scheme.type_parameters, vec!["A".to_owned()]);
    assert_eq!(function_export.scheme.constraints[0].name, "Eq");
}

#[test]
fn parses_nested_type_arguments_in_interface() {
    let interface = parse_module_interface(
        "artifact/nested-types/main.ssrg",
        "pub let values: Array<Maybe<Int>> = []\n",
    );
    let json = serde_json::to_value(&interface).expect("interface serializes");

    assert_eq!(
        json["exports"][0]["scheme"]["type"],
        serde_json::json!({
            "kind": "named",
            "name": "Array",
            "arguments": [
                {
                    "kind": "named",
                    "name": "Maybe",
                    "arguments": [
                        {
                            "kind": "named",
                            "name": "Int",
                            "arguments": [],
                        },
                    ],
                },
            ],
        })
    );
}

#[test]
fn parses_operator_type_parameters_in_interface() {
    let interface = parse_module_interface(
        "artifact/generic-operator/main.ssrg",
        "pub operator<A> infixr 5 <+>\n  left: A -> right: A -> A\nwhere Semigroup<A> =\n  left\n",
    );
    let operator_export = interface
        .exports
        .iter()
        .find(|export| export.namespace == "operator")
        .expect("operator export exists");

    assert_eq!(operator_export.name, "<+>");
    assert_eq!(operator_export.scheme.type_parameters, vec!["A".to_owned()]);
    assert_eq!(operator_export.scheme.constraints[0].name, "Semigroup");
    assert_eq!(interface.operators[0].fixity, "infixr");
    assert_eq!(interface.operators[0].precedence, 5);
}

#[test]
fn parses_instance_type_parameters_and_constraints_in_interface() {
    let interface = parse_module_interface(
        "artifact/constrained-instance/main.ssrg",
        "instance<A> Show<Box<A>>\nwhere Show<A> {\n  fn show value: Box<A> -> String = \"Box\"\n}\n",
    );

    assert_eq!(interface.instances.len(), 1);
    assert_eq!(interface.instances[0].type_parameters, vec!["A".to_owned()]);
    assert_eq!(interface.instances[0].constraints[0].name, "Show");
}

#[test]
fn parses_trait_declarations_in_interface() {
    let interface = parse_module_interface(
        "artifact/basic-trait/main.ssrg",
        "pub trait Ord<A>\nwhere Eq<A> {\n  fn compare x: A -> y: A -> Ordering\n}\n",
    );
    let trait_export = interface
        .exports
        .iter()
        .find(|export| export.namespace == "trait")
        .expect("trait export exists");

    assert_eq!(trait_export.symbol, "artifact/basic-trait::trait(Ord)");
    assert_eq!(trait_export.scheme.type_parameters, vec!["A".to_owned()]);
    assert_eq!(trait_export.scheme.constraints[0].name, "Eq");
}

#[test]
fn parses_alias_declarations_in_interface() {
    let interface = parse_module_interface(
        "artifact/type-alias/main.ssrg",
        "pub alias Boxed<A> = Box<A>\n",
    );
    let alias_export = interface
        .exports
        .iter()
        .find(|export| export.name == "Boxed")
        .expect("alias export exists");

    assert_eq!(alias_export.declaration_kind, Some("alias".to_owned()));
    assert_eq!(alias_export.scheme.type_parameters, vec!["A".to_owned()]);
}

#[test]
fn parses_generic_newtype_interface() {
    let interface = parse_module_interface(
        "artifact/generic-newtype/main.ssrg",
        "pub newtype Box<A> = A\n",
    );

    assert_eq!(interface.exports.len(), 1);
    let newtype_export = &interface.exports[0];
    assert_eq!(newtype_export.name, "Box");
    assert_eq!(newtype_export.scheme.type_parameters, vec!["A".to_owned()]);
    assert_eq!(
        newtype_export.scheme.type_ref,
        InterfaceType::TypeConstructor {
            name: "Box".to_owned(),
            arity: 1,
        }
    );
    assert_eq!(
        newtype_export.representation,
        Some(InterfaceType::Named {
            name: "A".to_owned(),
            arguments: Vec::new(),
        })
    );
}

#[test]
fn hides_opaque_newtype_representation_in_interface() {
    let interface = parse_module_interface(
        "artifact/opaque-newtype/main.ssrg",
        "pub opaque newtype UserId = Int\n",
    );

    assert_eq!(interface.exports.len(), 1);
    let newtype_export = &interface.exports[0];
    assert_eq!(newtype_export.name, "UserId");
    assert_eq!(newtype_export.declaration_kind, Some("newtype".to_owned()));
    assert_eq!(newtype_export.representation, None);
}

#[test]
fn exports_opaque_type_names_in_interface() {
    let interface = parse_module_interface(
        "artifact/opaque-type/main.ssrg",
        "pub opaque type Token<A> =\n  | Token A\n",
    );

    assert_eq!(interface.exports.len(), 1);
    let type_export = &interface.exports[0];
    assert_eq!(type_export.name, "Token");
    assert_eq!(type_export.declaration_kind, Some("opaque".to_owned()));
    assert_eq!(type_export.scheme.type_parameters, vec!["A".to_owned()]);
    assert_eq!(
        type_export.scheme.type_ref,
        InterfaceType::TypeConstructor {
            name: "Token".to_owned(),
            arity: 1,
        }
    );
    assert_eq!(type_export.representation, None);
}

#[test]
fn exports_opaque_struct_names_in_interface() {
    let interface = parse_module_interface(
        "artifact/opaque-struct/main.ssrg",
        "pub opaque struct UserId {\n  value: Int,\n}\n",
    );

    assert_eq!(interface.exports.len(), 1);
    let struct_export = &interface.exports[0];
    assert_eq!(struct_export.name, "UserId");
    assert_eq!(struct_export.declaration_kind, Some("opaque".to_owned()));
    assert_eq!(
        struct_export.scheme.type_ref,
        InterfaceType::TypeConstructor {
            name: "UserId".to_owned(),
            arity: 0,
        }
    );
    assert_eq!(struct_export.representation, None);
}

#[test]
fn exports_public_type_names_in_interface() {
    let interface = parse_module_interface(
        "artifact/public-type/main.ssrg",
        "pub type Maybe<A> =\n  | Nothing\n  | Just A\n",
    );

    assert_eq!(interface.exports.len(), 1);
    let type_export = &interface.exports[0];
    assert_eq!(type_export.name, "Maybe");
    assert_eq!(type_export.declaration_kind, Some("type".to_owned()));
    assert_eq!(type_export.scheme.type_parameters, vec!["A".to_owned()]);
    assert_eq!(
        type_export.scheme.type_ref,
        InterfaceType::TypeConstructor {
            name: "Maybe".to_owned(),
            arity: 1,
        }
    );
}

#[test]
fn exports_public_struct_names_in_interface() {
    let interface = parse_module_interface(
        "artifact/public-struct/main.ssrg",
        "pub struct User {\n  name: String,\n}\n",
    );

    assert_eq!(interface.exports.len(), 1);
    let struct_export = &interface.exports[0];
    assert_eq!(struct_export.name, "User");
    assert_eq!(struct_export.declaration_kind, Some("struct".to_owned()));
    assert_eq!(
        struct_export.scheme.type_ref,
        InterfaceType::TypeConstructor {
            name: "User".to_owned(),
            arity: 0,
        }
    );
}

#[test]
fn parses_aliased_import_in_interface() {
    let interface = parse_module_interface(
        "artifact/import-alias/main.ssrg",
        "import { parse as parseJson } from \"json\"\n\npub let answer: Int = 42\n",
    );

    assert_eq!(interface.dependencies.len(), 1);
    assert_eq!(interface.dependencies[0].imports.len(), 1);
    assert_eq!(interface.dependencies[0].imports[0].namespace, "value");
    assert_eq!(interface.dependencies[0].imports[0].name, "parse");
    assert_eq!(
        interface.dependencies[0].imports[0].local_name,
        Some("parseJson".to_owned())
    );
}

#[test]
fn parses_namespace_import_in_interface() {
    let interface = parse_module_interface(
        "artifact/namespace-import/main.ssrg",
        "import * as text from \"std/text\"\n\npub let answer: Int = 42\n",
    );

    assert_eq!(interface.dependencies.len(), 1);
    assert_eq!(interface.dependencies[0].imports.len(), 1);
    assert_eq!(interface.dependencies[0].imports[0].namespace, "namespace");
    assert_eq!(interface.dependencies[0].imports[0].name, "*");
    assert_eq!(
        interface.dependencies[0].imports[0].local_name,
        Some("text".to_owned())
    );
}

#[test]
fn parses_rich_module_interface_fixture() {
    let interface = parse_module_interface(
        "artifact/rich/main.ssrg",
        include_str!("../../../../examples/spec/artifacts/interface-schema-1/rich/main.ssrg"),
    );
    let actual = serde_json::to_value(&interface).expect("interface serializes");
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../examples/spec/artifacts/interface-schema-1/rich/interface.json"
    ))
    .expect("expected interface fixture parses");

    assert_eq!(actual, expected);
}
