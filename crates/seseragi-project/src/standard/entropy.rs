use super::*;

pub(super) fn entropy_interface() -> ModuleInterface {
    let module = "std/entropy";
    let mut exports = vec![
        type_export(module, "Entropy", 0, "opaque-type"),
        opaque_adt_type_export(module, "EntropyConfigError", []),
        constructor_export(
            module,
            "EntropyConfigError",
            "NonPositiveEntropySize",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "EntropyConfigError",
            "EntropySizeTooLarge",
            [],
            Some(named("Int")),
        ),
        opaque_adt_type_export(module, "EntropyError", []),
        constructor_export(module, "EntropyError", "EntropyUnavailable", [], None),
        constructor_export(module, "EntropyError", "EntropyReadFailure", [], None),
        type_export(module, "EntropySize", 0, "opaque-type"),
        function_export(
            module,
            "entropySize",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with(
                "Either",
                vec![named("EntropyConfigError"), named("EntropySize")],
            ),
        ),
        function_export(
            module,
            "secureBytes",
            [],
            Vec::new(),
            vec![named("EntropySize")],
            effect(
                record([required("entropy", named("Entropy"))]),
                named("EntropyError"),
                external_type(
                    "Bytes",
                    "std/bytes::Bytes",
                    "std/bytes",
                    "Bytes",
                    Vec::new(),
                ),
            ),
        ),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}
