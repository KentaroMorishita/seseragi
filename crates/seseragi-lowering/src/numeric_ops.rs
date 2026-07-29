#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeNumericOperation {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

macro_rules! int_operation {
    ($name:literal, $feature:literal) => {
        RuntimeNumericOperation {
            canonical: concat!("std/int::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_int_", $name),
            module: "@seseragi/runtime/int",
            export_name: $name,
            source_map_name: $name,
        }
    };
}

macro_rules! float_operation {
    ($name:literal, $feature:literal) => {
        RuntimeNumericOperation {
            canonical: concat!("std/float::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_float_", $name),
            module: "@seseragi/runtime/float",
            export_name: $name,
            source_map_name: $name,
        }
    };
}

macro_rules! number_operation {
    ($name:literal, $feature:literal) => {
        RuntimeNumericOperation {
            canonical: concat!("std/number::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_number_", $name),
            module: "@seseragi/runtime/number",
            export_name: $name,
            source_map_name: $name,
        }
    };
}

const OPERATIONS: &[RuntimeNumericOperation] = &[
    number_operation!("HalfEven", "core.number.rounding.half-even"),
    number_operation!("HalfUp", "core.number.rounding.half-up"),
    number_operation!("TowardZero", "core.number.rounding.toward-zero"),
    number_operation!("AwayFromZero", "core.number.rounding.away-from-zero"),
    number_operation!("Floor", "core.number.rounding.floor"),
    number_operation!("Ceiling", "core.number.rounding.ceiling"),
    int_operation!("EmptyInt", "core.int.parse-error.empty"),
    int_operation!("InvalidIntRadix", "core.int.parse-error.invalid-radix"),
    int_operation!("InvalidIntDigit", "core.int.parse-error.invalid-digit"),
    int_operation!("IntOutsideRange", "core.int.parse-error.outside-range"),
    int_operation!(
        "IntDivisionByZero",
        "core.int.division-error.division-by-zero"
    ),
    int_operation!(
        "NegativeIntExponent",
        "core.int.power-error.negative-exponent"
    ),
    int_operation!("IntPowerOverflow", "core.int.power-error.overflow"),
    int_operation!("minValue", "core.int.api.min-value"),
    int_operation!("maxValue", "core.int.api.max-value"),
    int_operation!("parse", "core.int.api.parse"),
    int_operation!("parseRadix", "core.int.api.parse-radix"),
    int_operation!("format", "core.int.api.format"),
    int_operation!("formatRadix", "core.int.api.format-radix"),
    int_operation!("checkedAdd", "core.int.api.checked-add"),
    int_operation!("checkedSubtract", "core.int.api.checked-subtract"),
    int_operation!("checkedMultiply", "core.int.api.checked-multiply"),
    int_operation!("checkedDivide", "core.int.api.checked-divide"),
    int_operation!("checkedRemainder", "core.int.api.checked-remainder"),
    int_operation!("checkedPower", "core.int.api.checked-power"),
    int_operation!("saturatingAdd", "core.int.api.saturating-add"),
    int_operation!("saturatingSubtract", "core.int.api.saturating-subtract"),
    int_operation!("saturatingMultiply", "core.int.api.saturating-multiply"),
    int_operation!("saturatingPower", "core.int.api.saturating-power"),
    int_operation!("abs", "core.int.api.abs"),
    int_operation!("minimum", "core.int.api.minimum"),
    int_operation!("maximum", "core.int.api.maximum"),
    int_operation!("clamp", "core.int.api.clamp"),
    int_operation!("sign", "core.int.api.sign"),
    float_operation!("EmptyFloat", "core.float64.parse-error.empty"),
    float_operation!("InvalidFloat", "core.float64.parse-error.invalid"),
    float_operation!("FloatParseOverflow", "core.float64.parse-error.overflow"),
    float_operation!("FloatNotFinite", "core.float64.conversion-error.not-finite"),
    float_operation!(
        "FloatOutsideIntRange",
        "core.float64.conversion-error.outside-int-range"
    ),
    float_operation!("nan", "core.float64.api.nan"),
    float_operation!("positiveInfinity", "core.float64.api.positive-infinity"),
    float_operation!("negativeInfinity", "core.float64.api.negative-infinity"),
    float_operation!("parse", "core.float64.api.parse"),
    float_operation!("format", "core.float64.api.format"),
    float_operation!("fromInt", "core.float64.api.from-int"),
    float_operation!("toInt", "core.float64.api.to-int"),
    float_operation!("isNaN", "core.float64.api.is-nan"),
    float_operation!("isFinite", "core.float64.api.is-finite"),
    float_operation!("isInfinite", "core.float64.api.is-infinite"),
    float_operation!("isNegativeZero", "core.float64.api.is-negative-zero"),
    float_operation!("ieeeEq", "core.float64.api.ieee-eq"),
    float_operation!("totalCompare", "core.float64.api.total-compare"),
    float_operation!("minimumNumber", "core.float64.api.minimum-number"),
    float_operation!("maximumNumber", "core.float64.api.maximum-number"),
    float_operation!("clampNumber", "core.float64.api.clamp-number"),
    float_operation!("abs", "core.float64.api.abs"),
    float_operation!("sign", "core.float64.api.sign"),
    float_operation!("power", "core.float64.api.power"),
    float_operation!("roundIntegral", "core.float64.api.round-integral"),
];

pub(crate) fn runtime_numeric_operation(canonical: &str) -> Option<RuntimeNumericOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
}

pub(crate) fn runtime_numeric_operation_for_feature(
    feature: &str,
) -> Option<RuntimeNumericOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_safe_integer_float_and_rounding_operations() {
        for (canonical, feature) in [
            ("std/int::checkedAdd", "core.int.api.checked-add"),
            ("std/int::abs", "core.int.api.abs"),
            ("std/float::fromInt", "core.float64.api.from-int"),
            (
                "std/float::roundIntegral",
                "core.float64.api.round-integral",
            ),
            ("std/number::HalfEven", "core.number.rounding.half-even"),
        ] {
            let operation = runtime_numeric_operation(canonical).unwrap();
            assert_eq!(operation.runtime_feature, feature);
            assert_eq!(
                runtime_numeric_operation_for_feature(feature),
                Some(operation)
            );
        }
        assert!(runtime_numeric_operation("std/int::wrappingAdd").is_none());
        assert!(runtime_numeric_operation("std/float::fromIntExact").is_none());
    }
}
