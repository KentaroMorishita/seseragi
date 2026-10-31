fn native_validation_operation(name: &str, arguments: Vec<V>) -> V {
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
fn native_validation_eq(children: Vec<V>) -> V {
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
