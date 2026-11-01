fn native_derived_structural_dictionary(
    operation: &'static str,
    children: Vec<V>,
    fields: Option<Vec<(&str, bool)>>,
) -> V {
    let record = fields.is_some();
    let metadata = V::Array(Shared::new(
        fields
            .unwrap_or_default()
            .into_iter()
            .map(|(name, optional)| V::Tuple(Shared::new(vec![V::text(name), V::Bool(optional)])))
            .collect(),
    ));
    let mut methods = Vec::new();
    for method in if operation == "compare" {
        vec!["eq", "compare"]
    } else {
        vec![operation]
    } {
        let mut captures = vec![V::text(method), V::Bool(record), metadata.clone()];
        captures.extend(children.iter().map(|child| child.field(method)));
        let unary = matches!(method, "show" | "debug" | "hash");
        let function = V::fun(captures, native_derived_structural_stage);
        // Unary and binary methods use the same metadata, with the left value
        // retained only by the second source stage of a binary method.
        let function = if unary {
            V::fun(vec![function], |capture, value| {
                capture[0].apply(value).apply(V::Unit)
            })
        } else {
            function
        };
        methods.push((method, function));
    }
    V::dict(methods)
}

fn native_derived_structural_stage(captures: &[V], left: V) -> V {
    let mut captures = captures.to_vec();
    captures.push(left);
    V::fun(captures, |captures, right| {
        let V::Text(method) = &captures[0] else {
            unreachable!()
        };
        let method = method.get().as_str();
        let record = captures[1].boolean();
        let V::Array(metadata) = &captures[2] else {
            unreachable!()
        };
        let left = captures.last().unwrap();
        let children = &captures[3..captures.len() - 1];
        let run = || {
            let mut state = 0x811c9dc5u32;
            let mut text = String::from(if record { "{ " } else { "(" });
            if record && children.is_empty() {
                text = "{".into();
            }
            for (index, child) in children.iter().enumerate() {
                let (a, b) = if record {
                    let V::Tuple(field) = &metadata.get()[index] else {
                        unreachable!()
                    };
                    let [V::Text(name), V::Bool(optional)] = field.get().as_slice() else {
                        unreachable!()
                    };
                    let a = left.record_fields().get(name.get());
                    let b = if method == "eq" {
                        right.record_fields().get(name.get())
                    } else {
                        None
                    };
                    if method == "eq" {
                        if a.is_some() != b.is_some() || (!optional && a.is_none()) {
                            return V::Bool(false);
                        }
                        if a.is_none() {
                            continue;
                        }
                    } else {
                        if index != 0 {
                            text.push_str(", ");
                        }
                        text.push_str(name.get());
                        text.push_str(if *optional { "?: " } else { ": " });
                        if *optional {
                            text.push_str(if a.is_some() { "Just " } else { "Nothing" });
                            if a.is_none() {
                                continue;
                            }
                        }
                    }
                    (a.unwrap().clone(), b.cloned().unwrap_or(V::Unit))
                } else {
                    let V::Tuple(a) = left else {
                        panic!("Rust ABI: tuple")
                    };
                    assert_eq!(a.get().len(), children.len(), "Rust ABI: tuple arity");
                    let b = if matches!(method, "eq" | "compare") {
                        let V::Tuple(b) = &right else {
                            panic!("Rust ABI: tuple")
                        };
                        assert_eq!(b.get().len(), children.len(), "Rust ABI: tuple arity");
                        b.get()[index].clone()
                    } else {
                        V::Unit
                    };
                    if index != 0 {
                        text.push_str(", ");
                    }
                    (a.get()[index].clone(), b)
                };
                let result = child.apply(a);
                match method {
                    "eq" => {
                        if !result.apply(b).boolean() {
                            return V::Bool(false);
                        }
                    }
                    "compare" => {
                        let order = result.apply(b);
                        if !order.tagged("std/prelude::Equal") {
                            return order;
                        }
                    }
                    "hash" => {
                        let V::Int(hash) = result else {
                            panic!("Rust ABI: Hash result")
                        };
                        let bits = hash as u64;
                        state = (state ^ (bits ^ (bits >> 32)) as u32).wrapping_mul(16777619);
                    }
                    _ => {
                        let V::Text(child) = result else {
                            panic!("Rust ABI: display result")
                        };
                        text.push_str(child.get());
                    }
                }
            }
            match method {
                "eq" => V::Bool(true),
                "compare" => V::adt("std/prelude::Equal", V::Unit),
                "hash" => V::Int(state as i32 as i64),
                _ => {
                    text.push_str(if record && !children.is_empty() {
                        " }"
                    } else if record {
                        "}"
                    } else {
                        ")"
                    });
                    V::text(&text)
                }
            }
        };
        if matches!(method, "show" | "debug") {
            native_display(run)
        } else {
            run()
        }
    })
}
