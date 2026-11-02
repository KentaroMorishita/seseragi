use super::*;

pub(super) fn web_storage_interface() -> ModuleInterface {
    let module = "std/web/storage";
    let area = named("StorageArea");
    let storage_effect = |success: InterfaceType| {
        effect(
            record([required("storage", named("Storage"))]),
            named("StorageError"),
            success,
        )
    };
    standard_interface(
        module,
        vec![
            type_export(module, "Storage", 0, "opaque-type"),
            opaque_adt_type_export(module, "StorageArea", []),
            constructor_export(module, "StorageArea", "Local", [], None),
            constructor_export(module, "StorageArea", "Session", [], None),
            opaque_adt_type_export(module, "StorageError", []),
            constructor_export(
                module,
                "StorageError",
                "StorageQuotaExceeded",
                [],
                Some(record([
                    required("area", area.clone()),
                    required("key", named("String")),
                    required("message", named("String")),
                ])),
            ),
            constructor_export(
                module,
                "StorageError",
                "StorageSecurityFailure",
                [],
                Some(record([
                    required("area", area.clone()),
                    required("message", named("String")),
                ])),
            ),
            constructor_export(
                module,
                "StorageError",
                "StorageUnavailable",
                [],
                Some(record([
                    required("area", area.clone()),
                    required("message", named("String")),
                ])),
            ),
            effect_function_export(
                module,
                "get",
                [],
                Vec::new(),
                vec![area.clone(), named("String")],
                storage_effect(named_with("Maybe", vec![named("String")])),
            ),
            effect_function_export(
                module,
                "set",
                [],
                Vec::new(),
                vec![area.clone(), named("String"), named("String")],
                storage_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "remove",
                [],
                Vec::new(),
                vec![area.clone(), named("String")],
                storage_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "clear",
                [],
                Vec::new(),
                vec![area.clone()],
                storage_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "keys",
                [],
                Vec::new(),
                vec![area],
                storage_effect(named_with("Array", vec![named("String")])),
            ),
            function_export(
                module,
                "errorMessage",
                [],
                Vec::new(),
                vec![named("StorageError")],
                named("String"),
            ),
        ],
    )
}
