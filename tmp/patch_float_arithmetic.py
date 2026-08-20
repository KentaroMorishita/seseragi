from pathlib import Path

path = Path("crates/seseragi-semantics/src/typed/call_evidence.rs")
text = path.read_text()

old_heads = '''fn standard_binary_heads(trait_name: &str) -> Vec<[TypedType; 3]> {
    let mut heads = Vec::new();
    if matches!(trait_name, "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow") {
        let int = named_type("Int");
        heads.push([int.clone(), int.clone(), int]);
    }
    if trait_name == "Add" {
        let string = named_type("String");
        heads.push([string.clone(), string.clone(), string]);
    }
    heads
}
'''
new_heads = '''fn standard_binary_heads(trait_name: &str) -> Vec<[TypedType; 3]> {
    let mut heads = Vec::new();
    if matches!(trait_name, "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow") {
        let int = named_type("Int");
        heads.push([int.clone(), int.clone(), int]);
        let float = named_type("Float");
        heads.push([float.clone(), float.clone(), float]);
    }
    if trait_name == "Add" {
        let string = named_type("String");
        heads.push([string.clone(), string.clone(), string]);
    }
    heads
}
'''

old_output = '''pub(crate) fn standard_binary_output(
    trait_name: &str,
    left: &TypedType,
    right: &TypedType,
) -> Option<TypedType> {
    if trait_name == "Add" && named_type_is(left, "String") && named_type_is(right, "String") {
        return Some(left.clone());
    }
    matches!(trait_name, "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow")
        .then(|| named_type_is(left, "Int") && named_type_is(right, "Int"))
        .filter(|matches| *matches)
        .map(|_| left.clone())
}
'''
new_output = '''pub(crate) fn standard_binary_output(
    trait_name: &str,
    left: &TypedType,
    right: &TypedType,
) -> Option<TypedType> {
    if trait_name == "Add" && named_type_is(left, "String") && named_type_is(right, "String") {
        return Some(left.clone());
    }
    let supported_numeric_pair =
        (named_type_is(left, "Int") && named_type_is(right, "Int"))
            || (named_type_is(left, "Float") && named_type_is(right, "Float"));
    matches!(trait_name, "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow")
        .then_some(supported_numeric_pair)
        .filter(|matches| *matches)
        .map(|_| left.clone())
}
'''

old_identity = '''    let all_int = [left, right, output]
        .iter()
        .all(|type_ref| matches!(type_ref, TypedType::Named { name, arguments } if name == "Int" && arguments.is_empty()));
    if !all_int {
        return None;
    }
    match constraint.name.as_str() {
        "Add" => Some("std/int::Add"),
        "Sub" => Some("std/int::Sub"),
        "Mul" => Some("std/int::Mul"),
        "Div" => Some("std/int::Div"),
        "Rem" => Some("std/int::Rem"),
        "Pow" => Some("std/int::Pow"),
        _ => None,
    }
'''
new_identity = '''    let all_int = [left, right, output]
        .iter()
        .all(|type_ref| matches!(type_ref, TypedType::Named { name, arguments } if name == "Int" && arguments.is_empty()));
    if all_int {
        return match constraint.name.as_str() {
            "Add" => Some("std/int::Add"),
            "Sub" => Some("std/int::Sub"),
            "Mul" => Some("std/int::Mul"),
            "Div" => Some("std/int::Div"),
            "Rem" => Some("std/int::Rem"),
            "Pow" => Some("std/int::Pow"),
            _ => None,
        };
    }
    let all_float = [left, right, output]
        .iter()
        .all(|type_ref| matches!(type_ref, TypedType::Named { name, arguments } if name == "Float" && arguments.is_empty()));
    if all_float {
        return match constraint.name.as_str() {
            "Add" => Some("std/float::Add"),
            "Sub" => Some("std/float::Sub"),
            "Mul" => Some("std/float::Mul"),
            "Div" => Some("std/float::Div"),
            "Rem" => Some("std/float::Rem"),
            "Pow" => Some("std/float::Pow"),
            _ => None,
        };
    }
    None
'''

old_test = '''    #[test]
    fn selects_standard_string_add_evidence() {
'''
new_test = '''    #[test]
    fn selects_standard_float_add_evidence() {
        let evidence = select_call_evidence(&[TypedConstraint {
            name: "Add".to_owned(),
            arguments: vec![named("Float"), named("Float"), named("Float")],
        }])
        .expect("standard Float Add evidence");
        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                constraint: TypedConstraint { name, arguments },
                evidence: TypedInstanceEvidence::Standard { identity, .. },
            }] if name == "Add" && arguments.len() == 3 && identity == "std/float::Add"
        ));
    }

    #[test]
    fn selects_standard_string_add_evidence() {
'''

for old, new, label in [
    (old_heads, new_heads, "standard_binary_heads"),
    (old_output, new_output, "standard_binary_output"),
    (old_identity, new_identity, "arithmetic_instance_identity"),
    (old_test, new_test, "float evidence test"),
]:
    if old not in text:
        raise SystemExit(f"missing expected block: {label}")
    text = text.replace(old, new, 1)

path.write_text(text)
