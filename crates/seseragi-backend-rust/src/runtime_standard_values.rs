fn native_standard_display_dictionary(provider: &'static str, children: Vec<V>, debug: bool) -> V {
    let method = if debug { "debug" } else { "show" };
    let mut captures = vec![V::text(provider)];
    captures.extend(children.iter().map(|child| child.field(method)));
    if provider == "std/validation" {
        captures[1] = native_standard_display_dictionary(
            "std/non-empty-list",
            vec![children[0].clone()],
            debug,
        )
        .field(method);
    }
    V::dict(vec![(
        method,
        V::fun(captures, |captures, value| {
            native_display(|| {
                let render = |index: usize, value: V| {
                    let V::Text(text) = captures[index + 1].apply(value) else {
                        panic!("Rust ABI: display method must return String")
                    };
                    text.get().clone()
                };
                let output = match native_text(&captures[0]) {
                    "std/either" | "std/validation" => {
                        let V::Adt(adt) = &value else {
                            panic!("Rust ABI: expected sum value")
                        };
                        let (tag, index) = match adt.get().0 {
                            "std/prelude::Left" => ("Left", 0),
                            "std/prelude::Right" => ("Right", 1),
                            "std/validation::Invalid" => ("Invalid", 0),
                            "std/validation::Valid" => ("Valid", 1),
                            _ => panic!("Rust ABI: expected canonical sum constructor"),
                        };
                        format!("{tag} {}", render(index, value.payload()))
                    }
                    "std/non-empty-list" => format!(
                        "`[{}]",
                        native_non_empty_values(&value)
                            .into_iter()
                            .map(|value| render(0, value))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    "std/map" => {
                        let V::Array(entries) =
                            native_map_operation("std/map::entries", vec![], vec![value])
                        else {
                            unreachable!()
                        };
                        format!(
                            "Map [{}]",
                            entries
                                .get()
                                .iter()
                                .map(|entry| {
                                    let key = render(0, entry.index(0));
                                    let value = render(1, entry.index(1));
                                    format!("({key}, {value})")
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                    "std/set" => {
                        let V::Array(values) =
                            native_set_operation("std/set::toArray", vec![], vec![value])
                        else {
                            unreachable!()
                        };
                        format!(
                            "Set [{}]",
                            values
                                .get()
                                .iter()
                                .map(|value| render(0, value.clone()))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                    _ => panic!("Rust ABI: unsupported standard display provider"),
                };
                V::text(&output)
            })
        }),
    )])
}

fn native_standard_append(provider: &str, children: &[V], left: V, right: V) -> V {
    match provider {
        "std/string" => V::text(&format!("{}{}", native_text(&left), native_text(&right))),
        "std/array" => {
            let (V::Array(left), V::Array(right)) = (left, right) else {
                panic!("Rust ABI: expected Arrays")
            };
            V::Array(Shared::new(
                left.get().iter().chain(right.get()).cloned().collect(),
            ))
        }
        "std/list" => left
            .list_values()
            .into_iter()
            .rev()
            .fold(right, |tail, head| V::list_cons(head, tail)),
        "std/non-empty-list" => {
            let mut values = native_non_empty_values(&left);
            values.extend(native_non_empty_values(&right));
            native_non_empty(values)
        }
        "std/maybe" => {
            if left.tagged("std/prelude::Nothing") {
                right
            } else if right.tagged("std/prelude::Nothing") {
                left
            } else {
                V::adt(
                    "std/prelude::Just",
                    children[0]
                        .field("append")
                        .apply(left.payload())
                        .apply(right.payload()),
                )
            }
        }
        "std/sum" | "std/product" => {
            let (tag, method) = if provider == "std/sum" {
                ("std/prelude::Sum", "add")
            } else {
                ("std/prelude::Product", "mul")
            };
            V::adt(
                tag,
                children
                    .last()
                    .unwrap()
                    .field(method)
                    .apply(left.payload())
                    .apply(right.payload()),
            )
        }
        _ => panic!("Rust ABI: unsupported standard append provider"),
    }
}

fn native_standard_append_dictionary(provider: &'static str, children: Vec<V>, monoid: bool) -> V {
    let mut captures = vec![V::text(provider)];
    captures.extend(children);
    let append = V::fun(captures.clone(), |captures, left| {
        let mut next = captures.to_vec();
        next.push(left);
        V::fun(next, |captures, right| {
            let end = captures.len() - 1;
            native_standard_append(
                native_text(&captures[0]),
                &captures[1..end],
                captures[end].clone(),
                right,
            )
        })
    });
    let mut methods = vec![("append", append)];
    if monoid {
        methods.push((
            "empty",
            V::fun(captures, |captures, _| match native_text(&captures[0]) {
                "std/string" => V::text(""),
                "std/array" => V::Array(Shared::new(vec![])),
                "std/list" => V::ListEmpty,
                "std/maybe" => V::adt("std/prelude::Nothing", V::Unit),
                "std/sum" => V::adt("std/prelude::Sum", captures[1].field("zero").apply(V::Unit)),
                "std/product" => V::adt(
                    "std/prelude::Product",
                    captures[1].field("one").apply(V::Unit),
                ),
                _ => panic!("Rust ABI: unsupported standard empty provider"),
            }),
        ));
    }
    V::dict(methods)
}
