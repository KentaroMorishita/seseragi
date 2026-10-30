struct NativeDecimalContext {
    digits: i64,
    rounding: NativeRoundingMode,
}

impl TraceValue for NativeDecimalContext {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}

fn native_decimal_rounding_value(mode: NativeRoundingMode) -> V {
    let tag = match mode {
        NativeRoundingMode::HalfEven => "std/number::HalfEven",
        NativeRoundingMode::HalfUp => "std/number::HalfUp",
        NativeRoundingMode::TowardZero => "std/number::TowardZero",
        NativeRoundingMode::AwayFromZero => "std/number::AwayFromZero",
        NativeRoundingMode::Floor => "std/number::Floor",
        NativeRoundingMode::Ceiling => "std/number::Ceiling",
    };
    V::adt(tag, V::Unit)
}

fn native_decimal_standard_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/decimal::parse" | "std/decimal::fromInt", [value]) => {
            native_decimal_operation(name, value.clone())
        }
        ("std/decimal::context", [V::Int(digits), mode]) => {
            let digits = checked_int_value(*digits);
            let rounding = native_rounding_mode(mode);
            if digits <= 0 {
                native_left(V::adt(
                    "std/decimal::NonPositiveDecimalPrecision",
                    V::Int(digits),
                ))
            } else {
                native_right(V::DecimalContext(Shared::new(NativeDecimalContext {
                    digits,
                    rounding,
                })))
            }
        }
        ("std/decimal::precision", [V::DecimalContext(value)]) => V::Int(value.get().digits),
        ("std/decimal::rounding", [V::DecimalContext(value)]) => {
            native_decimal_rounding_value(value.get().rounding)
        }
        ("std/decimal::NonPositiveDecimalPrecision", [V::Int(digits)]) => V::adt(
            "std/decimal::NonPositiveDecimalPrecision",
            checked_int(*digits),
        ),
        _ => panic!("Rust ABI: unsupported Decimal standard operation or type"),
    }
}

#[cfg(test)]
mod decimal_context_tests {
    use super::*;

    #[test]
    fn context_preserves_all_modes_and_exact_precision_extremes() {
        for mode in [
            NativeRoundingMode::HalfEven,
            NativeRoundingMode::HalfUp,
            NativeRoundingMode::TowardZero,
            NativeRoundingMode::AwayFromZero,
            NativeRoundingMode::Floor,
            NativeRoundingMode::Ceiling,
        ] {
            for digits in [1, 34, 9_007_199_254_740_991] {
                let original = native_decimal_rounding_value(mode);
                let context = native_decimal_standard_operation(
                    "std/decimal::context",
                    vec![V::Int(digits), original.clone()],
                );
                assert!(context.tagged("std/prelude::Right"));
                let value = context.payload();
                assert!(
                    matches!(native_decimal_standard_operation("std/decimal::precision", vec![value.clone()]), V::Int(actual) if actual == digits)
                );
                let restored =
                    native_decimal_standard_operation("std/decimal::rounding", vec![value]);
                let V::Adt(tag) = original else { panic!() };
                assert!(restored.tagged(tag.get().0));
            }
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn nonpositive_precision_is_a_typed_error_with_its_exact_payload() {
        for digits in [-9_007_199_254_740_991, -1, 0] {
            let result = native_decimal_standard_operation(
                "std/decimal::context",
                vec![V::Int(digits), V::adt("std/number::HalfEven", V::Unit)],
            );
            assert!(result.tagged("std/prelude::Left"));
            assert!(result
                .payload()
                .tagged("std/decimal::NonPositiveDecimalPrecision"));
            assert!(matches!(result.payload().payload(), V::Int(actual) if actual == digits));
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn contexts_remain_alive_in_managed_partial_captures_and_collect_reference_cycles() {
        enable_cycle_collection();
        let value = native_decimal_standard_operation(
            "std/decimal::context",
            vec![V::Int(34), V::adt("std/number::HalfEven", V::Unit)],
        )
        .payload();
        let cell = V::reference(value.clone());
        let getter = V::fun(vec![cell.clone(), value.clone()], |captures, _| {
            native_decimal_standard_operation("std/decimal::precision", vec![captures[1].clone()])
        });
        assert!(matches!(getter.apply(V::Unit), V::Int(34)));
        cell.write_reference(V::Tuple(Shared::new(vec![cell.clone(), getter.clone()])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((value, cell, getter));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
