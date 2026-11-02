use super::*;

pub(super) fn validation_interface() -> ModuleInterface {
    let module = "std/validation";
    let validation = external_type(
        "Validation",
        "std/validation::Validation",
        module,
        "Validation",
        vec![named("E"), named("A")],
    );
    let errors = external_type(
        "NonEmptyList",
        "std/non-empty-list::NonEmptyList",
        "std/non-empty-list",
        "NonEmptyList",
        vec![named("E")],
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "Validation", ["E", "A"]),
        constructor_export(module, "Validation", "Valid", ["E", "A"], Some(named("A"))),
        constructor_export(
            module,
            "Validation",
            "Invalid",
            ["E", "A"],
            Some(errors.clone()),
        ),
    ];
    for (name, argument, result) in [
        ("valid", named("A"), validation.clone()),
        ("invalid", named("E"), validation.clone()),
        ("invalidMany", errors.clone(), validation.clone()),
        (
            "fromEither",
            named_with("Either", vec![named("E"), named("A")]),
            validation.clone(),
        ),
        (
            "toEither",
            validation,
            named_with("Either", vec![errors, named("A")]),
        ),
    ] {
        exports.push(function_export(
            module,
            name,
            ["E", "A"],
            vec![],
            vec![argument],
            result,
        ));
    }
    standard_interface(module, exports)
}
