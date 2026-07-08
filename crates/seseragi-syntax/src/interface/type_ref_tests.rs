use super::*;

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
fn parses_qualified_type_names_in_interface() {
    let interface = parse_module_interface(
        "artifact/qualified-type/main.ssrg",
        "pub let counts: maps.Map<String, Int> = value\n",
    );

    let json = serde_json::to_value(&interface).expect("interface serializes");
    assert_eq!(
        json["exports"][0]["scheme"]["type"],
        serde_json::json!({
            "kind": "named",
            "name": "maps.Map",
            "arguments": [
                { "kind": "named", "name": "String", "arguments": [] },
                { "kind": "named", "name": "Int", "arguments": [] }
            ],
        })
    );
}

#[test]
fn parses_record_type_references_in_interface() {
    let interface = parse_module_interface(
        "artifact/record-type/main.ssrg",
        "pub let env: { console: Console, clock?: Clock } = config\n",
    );

    let json = serde_json::to_value(&interface).expect("interface serializes");
    assert_eq!(json["exports"][0]["scheme"]["type"]["kind"], "record");
    assert_eq!(json["exports"][0]["scheme"]["type"]["closed"], true);
    assert_eq!(
        json["exports"][0]["scheme"]["type"]["fields"][0]["name"],
        "console"
    );
    assert_eq!(
        json["exports"][0]["scheme"]["type"]["fields"][1]["optional"],
        true
    );
    assert_eq!(
        json["exports"][0]["scheme"]["type"]["fields"][1]["type"]["name"],
        "Clock"
    );
}

#[test]
fn parses_tuple_type_references_in_interface() {
    let interface = parse_module_interface(
        "artifact/tuple-type/main.ssrg",
        "pub let pair: (String, Int) = value\n",
    );

    let json = serde_json::to_value(&interface).expect("interface serializes");
    assert_eq!(
        json["exports"][0]["scheme"]["type"],
        serde_json::json!({
            "kind": "tuple",
            "elements": [
                { "kind": "named", "name": "String", "arguments": [] },
                { "kind": "named", "name": "Int", "arguments": [] },
            ],
        })
    );
}

#[test]
fn parses_function_type_references_in_interface() {
    let interface = parse_module_interface(
        "artifact/function-type/main.ssrg",
        "pub let mapper: (String -> Int) = value\n",
    );

    let json = serde_json::to_value(&interface).expect("interface serializes");
    assert_eq!(
        json["exports"][0]["scheme"]["type"],
        serde_json::json!({
            "kind": "function",
            "parameter": { "kind": "named", "name": "String", "arguments": [] },
            "result": { "kind": "named", "name": "Int", "arguments": [] },
        })
    );
}

#[test]
fn parses_type_holes_in_partial_instance_heads() {
    let interface = parse_module_interface(
        "artifact/partial-instance/main.ssrg",
        "instance<E> Functor<Either<E, _>> {\n}\n",
    );

    let json = serde_json::to_value(&interface).expect("interface serializes");
    assert_eq!(
        json["instances"][0]["head"],
        serde_json::json!({
            "kind": "apply",
            "constructor": "Functor",
            "arguments": [
                {
                    "kind": "named",
                    "name": "Either",
                    "arguments": [
                        { "kind": "named", "name": "E", "arguments": [] },
                        { "kind": "hole" }
                    ]
                }
            ],
        })
    );
}
