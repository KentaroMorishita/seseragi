fn native_traversal_combine(applicative: &V, left: V, right: V) -> V {
    let combine = V::fun(vec![], |_, left| {
        V::fun(vec![left], |captures, right| {
            V::adt(
                "\0traversal::Branch",
                V::Tuple(Shared::new(vec![captures[0].clone(), right])),
            )
        })
    });
    applicative
        .field("apply")
        .apply(applicative.field("map").apply(combine).apply(left))
        .apply(right)
}

// Mirror runtime/ts/src/traversable.ts, including callbacks after an empty
// target result. Balanced persistent trees also bound deferred target stacks.
fn native_traverse_sequence(source: &V, callback: &V, applicative: &V, list: bool) -> V {
    let mut levels: Vec<Option<V>> = Vec::new();
    for value in native_sequence_iter(source, list) {
        let leaf = V::fun(vec![], |_, value| V::adt("\0traversal::Leaf", value));
        let mut wrapped = applicative
            .field("map")
            .apply(leaf)
            .apply(callback.apply(value.clone()));
        let mut level = 0;
        loop {
            if level == levels.len() {
                levels.push(None);
            }
            match levels[level].take() {
                Some(left) => {
                    wrapped = native_traversal_combine(applicative, left, wrapped);
                    level += 1;
                }
                None => {
                    levels[level] = Some(wrapped);
                    break;
                }
            }
        }
    }
    let mut accumulated = None;
    for wrapped in levels.into_iter().rev().flatten() {
        accumulated = Some(match accumulated {
            None => wrapped,
            Some(left) => native_traversal_combine(applicative, left, wrapped),
        });
    }
    let Some(accumulated) = accumulated else {
        return applicative
            .field("pure")
            .apply(native_sequence_collect(vec![], list));
    };
    applicative
        .field("map")
        .apply(V::fun(vec![V::Bool(list)], |captures, tree| {
            let mut result = Vec::new();
            let mut pending = vec![tree];
            while let Some(node) = pending.pop() {
                if node.tagged("\0traversal::Leaf") {
                    result.push(node.payload());
                } else {
                    assert!(
                        node.tagged("\0traversal::Branch"),
                        "Rust ABI: traversal tree"
                    );
                    let V::Tuple(children) = node.payload() else {
                        panic!("Rust ABI: traversal branch")
                    };
                    let [left, right] = children.get().as_slice() else {
                        panic!("Rust ABI: traversal branch arity")
                    };
                    pending.push(right.clone());
                    pending.push(left.clone());
                }
            }
            native_sequence_collect(result, captures[0].boolean())
        }))
        .apply(accumulated)
}

fn native_sequence_traversable_dictionary(list: bool) -> V {
    cached(
        if list {
            "\0native:list-traversable"
        } else {
            "\0native:array-traversable"
        },
        || {
            V::dict(vec![
                (
                    "map",
                    V::fun(vec![V::Bool(list)], |captures, callback| {
                        V::fun(vec![captures[0].clone(), callback], |captures, source| {
                            let list = captures[0].boolean();
                            native_sequence_collect(
                                native_sequence_iter(&source, list)
                                    .map(|value| captures[1].apply(value.clone()))
                                    .collect(),
                                list,
                            )
                        })
                    }),
                ),
                (
                    "traverse",
                    V::fun(vec![V::Bool(list)], |captures, callback| {
                        V::fun(vec![captures[0].clone(), callback], |captures, source| {
                            V::fun(
                                vec![captures[0].clone(), captures[1].clone(), source],
                                |captures, target| {
                                    native_traverse_sequence(
                                        &captures[2],
                                        &captures[1],
                                        &target,
                                        captures[0].boolean(),
                                    )
                                },
                            )
                        })
                    }),
                ),
            ])
        },
    )
}
