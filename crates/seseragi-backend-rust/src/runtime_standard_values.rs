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

fn native_standard_composite_dictionary(identity: &'static str, children: Vec<V>) -> V {
    let method = if identity.ends_with("::Hash") {
        "hash"
    } else if identity.ends_with("::Ord") {
        "compare"
    } else {
        "eq"
    };
    let mut captures = vec![V::text(identity)];
    captures.extend(children);
    let function = if method == "hash" {
        V::fun(captures.clone(), |captures, value| {
            let mut state = 0x811c9dc5u32;
            for value in native_non_empty_values(&value) {
                let V::Int(hash) = captures[1].field("hash").apply(value) else {
                    panic!("Rust ABI: Hash result")
                };
                let bits = hash as u64;
                state = (state ^ (bits ^ (bits >> 32)) as u32).wrapping_mul(0x01000193);
            }
            V::Int(state as i32 as i64)
        })
    } else {
        V::fun(captures.clone(), |captures, left| {
            let mut captures = captures.to_vec();
            captures.push(left);
            V::fun(captures, |captures, right| {
                let left = captures.last().unwrap();
                let identity = native_text(&captures[0]);
                let child = &captures[1];
                let eq = |a: V, b: V| child.field("eq").apply(a).apply(b).boolean();
                match identity {
                    "std/array::Eq" => {
                        let (V::Array(a), V::Array(b)) = (left, &right) else {
                            panic!("Rust ABI: Array Eq")
                        };
                        V::Bool(
                            a.get().len() == b.get().len()
                                && a.get()
                                    .iter()
                                    .zip(b.get())
                                    .all(|(a, b)| eq(a.clone(), b.clone())),
                        )
                    }
                    "std/list::Eq" | "std/non-empty-list::Eq" => {
                        let (mut a, mut b) = if identity == "std/list::Eq" {
                            (left.clone(), right)
                        } else {
                            if !eq(left.field("head"), right.field("head")) {
                                return V::Bool(false);
                            }
                            (left.field("tail"), right.field("tail"))
                        };
                        loop {
                            match (a.list_cell(), b.list_cell()) {
                                (None, None) => return V::Bool(true),
                                (Some(x), Some(y)) => {
                                    if !eq(x.head.clone(), y.head.clone()) {
                                        return V::Bool(false);
                                    }
                                    let next = (x.tail.clone(), y.tail.clone());
                                    (a, b) = next;
                                }
                                _ => return V::Bool(false),
                            }
                        }
                    }
                    "std/maybe::Eq" | "std/either::Eq" => {
                        let tag = if identity == "std/maybe::Eq" {
                            "std/prelude::Nothing"
                        } else {
                            "std/prelude::Left"
                        };
                        if left.tagged(tag) != right.tagged(tag) {
                            return V::Bool(false);
                        }
                        if identity == "std/maybe::Eq" && left.tagged(tag) {
                            return V::Bool(true);
                        }
                        let index = if identity == "std/either::Eq" && !left.tagged(tag) {
                            2
                        } else {
                            1
                        };
                        captures[index]
                            .field("eq")
                            .apply(left.payload())
                            .apply(right.payload())
                    }
                    "std/map::Eq" | "std/set::Eq" => {
                        let (a, b) = if identity == "std/set::Eq" {
                            (
                                native_set_into_map(left.clone()),
                                native_set_into_map(right),
                            )
                        } else {
                            (left.clone(), right)
                        };
                        if native_map_index_size(&native_map_value(&a).order)
                            != native_map_index_size(&native_map_value(&b).order)
                        {
                            return V::Bool(false);
                        }
                        let source = native_map_value(&a);
                        let mut id = source.first;
                        while let Some(current) = id {
                            let entry = native_map_entry(source, current);
                            let found = native_map_operation(
                                "std/map::get",
                                vec![captures[1].clone(), captures[2].clone()],
                                vec![entry.key.clone(), b.clone()],
                            );
                            if found.tagged("std/prelude::Nothing") {
                                return V::Bool(false);
                            }
                            if identity == "std/map::Eq"
                                && !captures[3]
                                    .field("eq")
                                    .apply(entry.value.clone())
                                    .apply(found.payload())
                                    .boolean()
                            {
                                return V::Bool(false);
                            }
                            id = entry.next;
                        }
                        V::Bool(true)
                    }
                    "std/non-empty-list::Ord" => {
                        let (a, b) = (
                            native_non_empty_values(left),
                            native_non_empty_values(&right),
                        );
                        for (a, b) in a.iter().zip(&b) {
                            let order = child.field("compare").apply(a.clone()).apply(b.clone());
                            if !order.tagged("std/prelude::Equal") {
                                return order;
                            }
                        }
                        V::adt(
                            if a.len() < b.len() {
                                "std/prelude::Less"
                            } else if a.len() > b.len() {
                                "std/prelude::Greater"
                            } else {
                                "std/prelude::Equal"
                            },
                            V::Unit,
                        )
                    }
                    _ => panic!("Rust ABI: composite dictionary provider"),
                }
            })
        })
    };
    let mut methods = vec![(method, function)];
    if method == "compare" {
        methods.push((
            "eq",
            native_standard_composite_dictionary("std/non-empty-list::Eq", captures[1..].to_vec())
                .field("eq"),
        ));
    }
    V::dict(methods)
}

fn native_standard_ord_dictionary(primitive: &'static str) -> V {
    let identity = match primitive {
        "Bool" => "std/bool::Ord",
        "String" => "std/string::Ord",
        "Unit" => "std/unit::Ord",
        _ => unreachable!(),
    };
    cached(identity, || {
        let eq = native_operator_dictionary(match primitive {
            "Bool" => "std/bool::Eq",
            "String" => "std/string::Eq",
            "Unit" => "std/unit::Eq",
            _ => unreachable!(),
        })
        .field("eq");
        V::dict(vec![
            ("eq", eq),
            (
                "compare",
                V::fun(vec![], |_, left| {
                    V::fun(vec![left], |captures, right| {
                        let order = match (&captures[0], &right) {
                            (V::Bool(a), V::Bool(b)) => a.cmp(b),
                            (V::Text(a), V::Text(b)) => a.get().cmp(b.get()),
                            (V::Unit, V::Unit) => std::cmp::Ordering::Equal,
                            _ => panic!("Rust ABI: primitive Ord values"),
                        };
                        V::adt(
                            match order {
                                std::cmp::Ordering::Less => "std/prelude::Less",
                                std::cmp::Ordering::Equal => "std/prelude::Equal",
                                std::cmp::Ordering::Greater => "std/prelude::Greater",
                            },
                            V::Unit,
                        )
                    })
                }),
            ),
        ])
    })
}
