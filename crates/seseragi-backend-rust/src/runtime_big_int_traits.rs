fn native_big_int_equal(left: V, right: V) -> V {
    let (V::BigInt(left), V::BigInt(right)) = (left, right) else {
        panic!("Rust ABI: expected BigInt Eq arguments")
    };
    V::Bool(left.get().equal(right.get()))
}

fn native_big_int_eq_dictionary() -> V {
    V::dict(vec![(
        "eq",
        V::fun(vec![], |_, left| {
            V::fun(vec![left], |captures, right| {
                native_big_int_equal(captures[0].clone(), right)
            })
        }),
    )])
}

fn native_big_int_order(left: V, right: V) -> V {
    let (V::BigInt(left), V::BigInt(right)) = (left, right) else {
        panic!("Rust ABI: expected BigInt Ord arguments")
    };
    let tag = match left.get().compare(right.get()) {
        std::cmp::Ordering::Less => "std/prelude::Less",
        std::cmp::Ordering::Equal => "std/prelude::Equal",
        std::cmp::Ordering::Greater => "std/prelude::Greater",
    };
    V::adt(tag, V::Unit)
}

fn native_big_int_ord_dictionary() -> V {
    V::dict(vec![
        ("eq", native_big_int_eq_dictionary().field("eq")),
        (
            "compare",
            V::fun(vec![], |_, left| {
                V::fun(vec![left], |captures, right| {
                    native_big_int_order(captures[0].clone(), right)
                })
            }),
        ),
    ])
}

fn native_big_int_hash_dictionary() -> V {
    V::dict(vec![(
        "hash",
        V::fun(vec![], |_, value| {
            let V::BigInt(value) = value else {
                panic!("Rust ABI: expected BigInt Hash argument")
            };
            V::Int(value.get().hash())
        }),
    )])
}

fn native_big_int_display_dictionary(debug: bool) -> V {
    V::dict(vec![(
        if debug { "debug" } else { "show" },
        V::fun(vec![], |_, value| {
            native_display(|| native_big_int_operation("std/big-int::format", value))
        }),
    )])
}

#[cfg(test)]
mod big_int_dictionary_tests {
    use super::*;

    #[test]
    fn binary_hash_and_order_do_not_construct_a_decimal_representation() {
        for (value, expected) in [
            (0, 292_984_781),
            (1, 276_207_162),
            (-1, -327_934_217),
            (42, 662_092_399),
            (4_294_967_295, -309_762_400),
            (4_294_967_296, 1_236_334_116),
        ] {
            let value = NativeBigInt::from_i64(value);
            assert!(value.decimal.get().is_none());
            assert_eq!(value.hash(), expected);
            assert!(value.decimal.get().is_none());
        }
        let huge = NativeBigInt::from_digits(false, b"ffffffffffffffffffffffffffffffff", 16);
        let negative = NativeBigInt::from_digits(true, b"100000000000000000000000000000000", 16);
        assert_eq!(huge.hash(), 1_213_781_163);
        assert_eq!(negative.hash(), -2_078_174_921);
        assert_eq!(negative.compare(&huge), std::cmp::Ordering::Less);
        assert!(huge.equal(&huge));
        assert!(huge.decimal.get().is_none() && negative.decimal.get().is_none());
        let zero = NativeBigInt::from_limbs(true, vec![0, 0, 0]);
        assert!(!zero.negative && zero.equal(&NativeBigInt::from_i64(0)));
        assert_eq!(zero.hash(), 292_984_781);
    }

    #[test]
    fn public_hash_collision_keeps_distinct_values_and_inherited_eq() {
        clear_dictionaries();
        {
            let left = native_big_int_operation("std/big-int::fromInt", V::Int(343_591_635));
            let right = native_big_int_operation("std/big-int::fromInt", V::Int(4_294_967_296));
            let hasher = native_big_int_hash_dictionary().field("hash");
            assert!(matches!(hasher.apply(left.clone()), V::Int(1_236_334_116)));
            assert!(matches!(hasher.apply(right.clone()), V::Int(1_236_334_116)));
            let order = native_big_int_ord_dictionary();
            assert!(matches!(
                order.field("eq").apply(left.clone()).apply(right.clone()),
                V::Bool(false)
            ));
            assert!(order
                .field("compare")
                .apply(left.clone())
                .apply(right.clone())
                .tagged("std/prelude::Less"));
            assert_eq!(
                native_big_int_display_dictionary(false)
                    .field("show")
                    .apply(left.clone())
                    .render(),
                "343591635"
            );
            assert_eq!(
                native_big_int_display_dictionary(true)
                    .field("debug")
                    .apply(right)
                    .render(),
                "4294967296"
            );
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn partial_order_captures_binary_values_and_collects_reference_cycles() {
        enable_cycle_collection();
        let left = native_big_int_parse("-340282366920938463463374607431768211456").payload();
        let cell = V::reference(left.clone());
        let compare = native_big_int_ord_dictionary()
            .field("compare")
            .apply(left.clone());
        assert!(compare
            .apply(native_big_int_parse("0").payload())
            .tagged("std/prelude::Less"));
        let capture = V::fun(vec![cell.clone(), compare.clone()], |captures, _| {
            captures[1].apply(captures[0].read_reference())
        });
        cell.write_reference(V::Tuple(Shared::new(vec![
            cell.clone(),
            left.clone(),
            capture.clone(),
        ])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((left, cell, compare, capture));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
