use super::*;

pub(super) fn number_interface() -> ModuleInterface {
    let module = "std/number";
    let mut exports = vec![opaque_adt_type_export(module, "RoundingMode", [])];
    for name in [
        "HalfEven",
        "HalfUp",
        "TowardZero",
        "AwayFromZero",
        "Floor",
        "Ceiling",
    ] {
        exports.push(constructor_export(module, "RoundingMode", name, [], None));
    }
    standard_interface(module, exports)
}
