// Set has its own native tag while sharing the persistent Map storage and
// tracing implementation. The internal Unit values are never exposed.
fn native_set_into_map(value: V) -> V {
    let V::Set(storage) = value else {
        panic!("Rust ABI: expected Set")
    };
    V::Map(storage)
}
fn native_set_from_map(value: V) -> V {
    let V::Map(storage) = value else {
        panic!("Rust ABI: expected internal Set Map")
    };
    V::Set(storage)
}
fn native_set_operation(name: &str, dictionaries: Vec<V>, mut arguments: Vec<V>) -> V {
    if name == "std/set::empty" {
        return native_set_from_map(native_map_operation(
            "std/map::empty",
            dictionaries,
            arguments,
        ));
    }
    if name == "std/set::singleton" {
        assert_eq!(arguments.len(), 1, "Rust ABI: Set singleton arity");
        arguments.push(V::Unit);
        return native_set_from_map(native_map_operation(
            "std/map::singleton",
            dictionaries,
            arguments,
        ));
    }
    let values = native_set_into_map(arguments.pop().expect("Rust ABI: Set source arity"));
    let (operation, returns_set) = match name {
        "std/set::contains" => ("std/map::containsKey", false),
        "std/set::insert" => {
            arguments.push(V::Unit);
            ("std/map::insert", true)
        }
        "std/set::remove" => ("std/map::remove", true),
        "std/set::toArray" => ("std/map::keys", false),
        "std/set::size" => ("std/map::size", false),
        "std/set::isEmpty" => ("std/map::isEmpty", false),
        _ => panic!("Rust ABI: unsupported Set operation"),
    };
    arguments.push(values);
    let result = native_map_operation(operation, dictionaries, arguments);
    if returns_set {
        native_set_from_map(result)
    } else {
        result
    }
}

#[cfg(test)]
mod set_tests {
    use super::*;

    thread_local! { static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; }
    fn events() -> Vec<String> {
        EVENTS.with(|e| std::mem::take(&mut *e.borrow_mut()))
    }
    fn dictionaries() -> Vec<V> {
        vec![
            V::dict(vec![(
                "eq",
                V::fun(vec![], |_, left| {
                    V::fun(vec![left], |c, right| {
                        EVENTS.with(|e| {
                            e.borrow_mut()
                                .push(format!("e{}:{}", c[0].render(), right.render()))
                        });
                        V::Bool(c[0].render() == right.render())
                    })
                }),
            )]),
            V::dict(vec![(
                "hash",
                V::fun(vec![], |_, key| {
                    let V::Int(key) = key else { panic!() };
                    EVENTS.with(|e| e.borrow_mut().push(format!("h{key}")));
                    V::Int(key % 2)
                }),
            )]),
        ]
    }

    #[test]
    fn set_preserves_snapshots_order_and_single_selected_hash_then_collision_eq() {
        assert_eq!(std::mem::size_of::<V>(), 16);
        let d = dictionaries();
        events();
        let empty = native_set_operation("std/set::empty", vec![], vec![V::Unit]);
        let one = native_set_operation("std/set::singleton", d.clone(), vec![V::Int(1)]);
        assert_eq!(events(), ["h1"]);
        let two = native_set_operation("std/set::insert", d.clone(), vec![V::Int(3), one.clone()]);
        assert_eq!(events(), ["h3", "e1:3"]);
        assert_eq!(
            native_set_operation("std/set::contains", d.clone(), vec![V::Int(3), two.clone()])
                .render(),
            "True"
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        let duplicate =
            native_set_operation("std/set::insert", d.clone(), vec![V::Int(1), two.clone()]);
        assert_eq!(events(), ["h1", "e1:1"]);
        let removed = native_set_operation(
            "std/set::remove",
            d.clone(),
            vec![V::Int(3), duplicate.clone()],
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        assert_eq!(
            native_set_operation("std/set::toArray", vec![], vec![two.clone()]).render(),
            "[1, 3]"
        );
        assert_eq!(
            native_set_operation("std/set::toArray", vec![], vec![removed.clone()]).render(),
            "[1]"
        );
        assert_eq!(
            native_set_operation("std/set::size", vec![], vec![two.clone()]).render(),
            "2"
        );
        assert_eq!(
            native_set_operation("std/set::isEmpty", vec![], vec![empty.clone()]).render(),
            "True"
        );
        assert!(events().is_empty());
        drop((d, empty, one, two, duplicate, removed));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn set_keys_trace_saved_callback_reference_cycles() {
        enable_cycle_collection();
        let reference = V::reference(V::Unit);
        let callback = V::fun(vec![reference.clone()], |_, value| value);
        let key = V::adt("proof::Key", callback.clone());
        let d = vec![
            V::dict(vec![(
                "eq",
                V::fun(vec![], |_, _| V::fun(vec![], |_, _| V::Bool(true))),
            )]),
            V::dict(vec![("hash", V::fun(vec![], |_, _| V::Int(0)))]),
        ];
        let values = native_set_operation("std/set::singleton", d.clone(), vec![key.clone()]);
        reference.write_reference(values.clone());
        let selected = native_set_operation("std/set::toArray", vec![], vec![values.clone()])
            .index(0)
            .payload();
        drop((values, key, callback));
        assert_eq!(selected.apply(V::Int(42)).render(), "42");
        drop((reference, selected, d));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn removed_set_keys_are_not_retained_by_the_result() {
        clear_dictionaries();
        let d = vec![
            native_operator_dictionary("std/string::Eq"),
            native_hash_dictionary("std/string::Hash"),
        ];
        let key = V::text("discarded-雪🙂");
        let V::Text(node) = &key else { panic!() };
        let weak = Rc::downgrade(node.0.as_ref().unwrap());
        let values = native_set_operation("std/set::singleton", d.clone(), vec![key.clone()]);
        let removed = native_set_operation(
            "std/set::remove",
            d.clone(),
            vec![V::text("discarded-雪🙂"), values.clone()],
        );
        drop((key, values));
        assert!(weak.upgrade().is_none());
        assert_eq!(
            native_set_operation("std/set::size", vec![], vec![removed.clone()]).render(),
            "0"
        );
        drop((d, removed));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
