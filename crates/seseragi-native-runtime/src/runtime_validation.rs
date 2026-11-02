pub fn native_validation_operation(name: &str, arguments: Vec<V>) -> V {
    let [value]: [V; 1] = arguments
        .try_into()
        .unwrap_or_else(|_| panic!("Rust ABI: Validation source arity"));
    match name {
        "std/validation::Valid" | "std/validation::valid" => V::adt("std/validation::Valid", value),
        "std/validation::Invalid" | "std/validation::invalidMany" => {
            V::adt("std/validation::Invalid", value)
        }
        "std/validation::invalid" => {
            V::adt("std/validation::Invalid", native_non_empty(vec![value]))
        }
        "std/validation::fromEither" => {
            if value.tagged("std/prelude::Left") {
                V::adt(
                    "std/validation::Invalid",
                    native_non_empty(vec![value.payload()]),
                )
            } else {
                V::adt("std/validation::Valid", value.payload())
            }
        }
        "std/validation::toEither" => V::adt(
            if value.tagged("std/validation::Invalid") {
                "std/prelude::Left"
            } else {
                "std/prelude::Right"
            },
            value.payload(),
        ),
        _ => panic!("Rust ABI: unsupported Validation operation"),
    }
}
pub fn native_validation_eq(children: Vec<V>) -> V {
    V::dict(vec![(
        "eq",
        V::fun(children, |children, left| {
            V::fun(
                vec![children[0].clone(), children[1].clone(), left],
                |captures, right| {
                    let left = &captures[2];
                    let valid = left.tagged("std/validation::Valid");
                    if valid != right.tagged("std/validation::Valid") {
                        return V::Bool(false);
                    }
                    if valid {
                        return captures[1]
                            .field("eq")
                            .apply(left.payload())
                            .apply(right.payload());
                    }
                    let mut left = native_non_empty_values(&left.payload()).into_iter();
                    let mut right = native_non_empty_values(&right.payload()).into_iter();
                    loop {
                        match (left.next(), right.next()) {
                            (None, None) => return V::Bool(true),
                            (Some(left), Some(right)) => {
                                if !captures[0].field("eq").apply(left).apply(right).boolean() {
                                    return V::Bool(false);
                                }
                            }
                            _ => return V::Bool(false),
                        }
                    }
                },
            )
        }),
    )])
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    fn operation(name: &str, value: V) -> V {
        native_validation_operation(name, vec![value])
    }

    fn errors(values: &[i64]) -> V {
        operation(
            "std/validation::invalidMany",
            native_non_empty(values.iter().copied().map(V::Int).collect()),
        )
    }

    fn numbers(value: &V) -> Vec<i64> {
        native_non_empty_values(value)
            .iter()
            .map(|value| match value {
                V::Int(value) => *value,
                _ => panic!("expected Int"),
            })
            .collect()
    }

    fn cleanup() {
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn constructors_and_either_conversions_preserve_function_payloads_and_nested_error_lists() {
        enable_cycle_collection();
        let root = V::reference(V::Unit);
        root.write_reference(root.clone());
        let function = V::fun(vec![root.clone()], |_, value| value);
        for name in ["std/validation::Valid", "std/validation::valid"] {
            let value = operation(name, function.clone());
            assert!(value.tagged("std/validation::Valid"));
            assert_eq!(value.payload().node_id(), function.node_id());
            let right = operation("std/validation::toEither", value);
            assert!(right.tagged("std/prelude::Right"));
            assert_eq!(right.payload().node_id(), function.node_id());
            let restored = operation("std/validation::fromEither", right);
            assert_eq!(restored.payload().apply(V::Int(42)).render(), "42");
        }
        let list = native_non_empty(vec![function.clone(), function.clone()]);
        for name in ["std/validation::Invalid", "std/validation::invalidMany"] {
            let invalid = operation(name, list.clone());
            assert!(invalid.tagged("std/validation::Invalid"));
            assert_eq!(invalid.payload().node_id(), list.node_id());
            let left = operation("std/validation::toEither", invalid);
            assert!(left.tagged("std/prelude::Left"));
            assert_eq!(left.payload().node_id(), list.node_id());
            let nested = operation("std/validation::fromEither", left);
            let outer = native_non_empty_values(&nested.payload());
            assert_eq!(outer.len(), 1);
            assert_eq!(outer[0].node_id(), list.node_id());
        }
        let invalid = operation("std/validation::invalid", function.clone());
        let singleton = native_non_empty_values(&invalid.payload());
        assert_eq!(singleton.len(), 1);
        assert_eq!(singleton[0].node_id(), function.node_id());
        drop((root, function, list, invalid, singleton));
        cleanup();
    }

    #[test]
    fn applicative_keeps_branch_identity_error_order_and_inputs_without_invoking_invalid_callbacks()
    {
        enable_cycle_collection();
        let dictionary = native_monad_dictionary(6, "Applicative");
        let unused = V::fun(vec![], |_, _| panic!("invalid callback was invoked"));
        let first = errors(&[1, 2, 1]);
        let last = errors(&[3, 4]);
        let valid = dictionary.field("pure").apply(V::Int(42));
        let mapped = dictionary
            .field("map")
            .apply(unused.clone())
            .apply(first.clone());
        assert_eq!(mapped.node_id(), first.node_id());
        let apply = dictionary.field("apply");
        let left = apply.apply(first.clone()).apply(valid.clone());
        assert_eq!(left.node_id(), first.node_id());
        let right = apply
            .apply(dictionary.field("pure").apply(unused))
            .apply(last.clone());
        assert_eq!(right.node_id(), last.node_id());
        let combined = apply.apply(first.clone()).apply(last.clone());
        assert!(combined.tagged("std/validation::Invalid"));
        assert_eq!(numbers(&combined.payload()), [1, 2, 1, 3, 4]);
        assert_eq!(numbers(&first.payload()), [1, 2, 1]);
        assert_eq!(numbers(&last.payload()), [3, 4]);
        let identity = dictionary
            .field("pure")
            .apply(V::fun(vec![], |_, value| value));
        let success = apply.apply(identity).apply(valid.clone());
        assert!(success.tagged("std/validation::Valid"));
        assert_eq!(success.payload().render(), "42");
        drop((
            dictionary, first, last, valid, mapped, apply, left, right, combined, success,
        ));
        cleanup();
    }

    #[test]
    fn equality_uses_the_selected_payload_dictionaries_in_order_and_short_circuits() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let error_eq = V::dict(vec![(
            "eq",
            V::fun(vec![calls.clone()], |captures, left| {
                V::fun(vec![captures[0].clone(), left], |captures, right| {
                    let V::Int(count) = captures[0].read_reference() else {
                        panic!()
                    };
                    captures[0].write_reference(V::Int(count + 1));
                    let (V::Int(left), V::Int(right)) = (&captures[1], right) else {
                        panic!()
                    };
                    assert_ne!(*left, 99, "unselected error was compared");
                    V::Bool(left % 10 == right % 10)
                })
            }),
        )]);
        let value_eq = V::dict(vec![(
            "eq",
            V::fun(vec![], |_, _| panic!("unselected value dictionary")),
        )]);
        let equality = native_validation_eq(vec![error_eq, value_eq]).field("eq");
        assert!(equality
            .apply(errors(&[11, 12]))
            .apply(errors(&[1, 2]))
            .boolean());
        assert_eq!(calls.read_reference().render(), "2");
        assert!(!equality
            .apply(errors(&[1, 99]))
            .apply(errors(&[2, 99]))
            .boolean());
        assert_eq!(calls.read_reference().render(), "3");
        assert!(!equality
            .apply(errors(&[1, 2]))
            .apply(errors(&[1]))
            .boolean());
        assert_eq!(calls.read_reference().render(), "4");
        let valid = operation("std/validation::valid", V::Int(1));
        assert!(!equality.apply(valid.clone()).apply(errors(&[99])).boolean());
        assert!(!equality.apply(errors(&[99])).apply(valid.clone()).boolean());
        assert_eq!(calls.read_reference().render(), "4");
        drop((calls, equality, valid));
        cleanup();
    }
}
