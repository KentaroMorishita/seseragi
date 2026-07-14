use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::InterfaceType;

pub(super) fn reject_shadowed_effect_constructor(
    interface: &TypedModuleInterface,
    entry_type: &InterfaceType,
    entry_export: &str,
) -> Result<(), String> {
    let mut result = entry_type;
    while let InterfaceType::Function {
        result: next_result,
        ..
    } = result
    {
        result = next_result;
    }
    let InterfaceType::Named { name, .. } = result else {
        return Ok(());
    };
    if name != "Effect" {
        return Ok(());
    }
    reject_user_shadow(interface, name, "result type")
        .map_err(|error| format!("execution Effect entry {entry_export} {error}"))
}

pub(super) fn reject_shadowed_environment_types(
    interface: &TypedModuleInterface,
    environment: &InterfaceType,
    entry_export: &str,
) -> Result<(), String> {
    let InterfaceType::Record { fields, .. } = environment else {
        return Ok(());
    };
    for field in fields {
        let Some(name) = named_type(&field.type_ref) else {
            continue;
        };
        if !matches!(name, "Console" | "Stdin") {
            continue;
        }
        reject_user_shadow(
            interface,
            name,
            &format!("environment field {}", field.name),
        )
        .map_err(|error| format!("execution Effect entry {entry_export} {error}"))?;
    }
    Ok(())
}

pub(super) fn reject_shadowed_standard_failure(
    interface: &TypedModuleInterface,
    failure: &InterfaceType,
) -> Result<(), String> {
    let Some(name) = named_type(failure) else {
        return Ok(());
    };
    if !matches!(name, "Never" | "String" | "ConsoleError" | "StdinError") {
        return Ok(());
    }
    reject_user_shadow(interface, name, "failure type")
}

pub(super) fn validate_effect_success(
    interface: &TypedModuleInterface,
    success: &InterfaceType,
    entry_export: &str,
) -> Result<(), String> {
    if !matches!(
        success,
        InterfaceType::Named { name, arguments }
            if name == "Unit" && arguments.is_empty()
    ) {
        return Err(format!(
            "execution Effect entry {entry_export} success type A must be standard Unit"
        ));
    }
    reject_user_shadow(interface, "Unit", "success type")
        .map_err(|error| format!("execution Effect entry {entry_export} {error}"))
}

pub(in crate::execution_case) fn reject_user_shadow(
    interface: &TypedModuleInterface,
    spelling: &str,
    context: &str,
) -> Result<(), String> {
    let standard = standard_identity(spelling).expect("callers filter standard type spellings");
    if let Some(symbol) = interface
        .exports
        .iter()
        .filter(|export| export.namespace == "type" && export.name == spelling)
        .map(|export| export.symbol.as_str())
        .chain(interface.dependencies.iter().flat_map(|dependency| {
            dependency
                .imports
                .iter()
                .filter(|import| import.namespace == "type")
                .filter(move |import| {
                    import.local_name.as_deref().unwrap_or(&import.name) == spelling
                })
                .map(|import| import.symbol.as_str())
        }))
        .find(|symbol| *symbol != standard)
    {
        return Err(format!(
            "{context} {spelling} resolves to user type {symbol}, not standard runtime type {standard}"
        ));
    }
    Ok(())
}

fn named_type(type_ref: &InterfaceType) -> Option<&str> {
    match type_ref {
        InterfaceType::Named { name, arguments } if arguments.is_empty() => Some(name),
        _ => None,
    }
}

fn standard_identity(spelling: &str) -> Option<&'static str> {
    match spelling {
        "Unit" => Some("std/prelude::Unit"),
        "Never" => Some("std/prelude::Never"),
        "String" => Some("std/prelude::String"),
        "Effect" => Some("std/prelude::Effect"),
        "Console" => Some("std/prelude::Console"),
        "ConsoleError" => Some("std/prelude::ConsoleError"),
        "Stdin" => Some("std/prelude::Stdin"),
        "StdinError" => Some("std/prelude::StdinError"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        reject_shadowed_effect_constructor, reject_shadowed_environment_types,
        validate_effect_success,
    };
    use seseragi_semantics::TypedModuleInterface;
    use seseragi_syntax::{
        ByteSpan, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceRecordField,
        InterfaceScheme, InterfaceType, Visibility,
    };

    fn named(name: &str) -> InterfaceType {
        InterfaceType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    fn interface() -> TypedModuleInterface {
        TypedModuleInterface {
            schema: 1,
            stage: "typed-interface".to_owned(),
            module: "artifact/main".to_owned(),
            source: "main.ssrg".to_owned(),
            dependencies: Vec::new(),
            exports: Vec::new(),
            operators: Vec::new(),
            instances: Vec::new(),
        }
    }

    fn environment(name: &str) -> InterfaceType {
        InterfaceType::Record {
            closed: true,
            fields: vec![InterfaceRecordField {
                name: name.to_ascii_lowercase(),
                optional: false,
                type_ref: named(name),
            }],
        }
    }

    fn effect_entry() -> InterfaceType {
        InterfaceType::Function {
            parameter: Box::new(named("Unit")),
            result: Box::new(InterfaceType::Named {
                name: "Effect".to_owned(),
                arguments: vec![
                    InterfaceType::Record {
                        closed: true,
                        fields: Vec::new(),
                    },
                    named("Never"),
                    named("Unit"),
                ],
            }),
        }
    }

    fn type_export(name: &str) -> InterfaceExport {
        InterfaceExport {
            symbol: format!("artifact/main::{name}"),
            namespace: "type".to_owned(),
            name: name.to_owned(),
            constructor_of: None,
            visibility: Visibility::Public,
            declaration_kind: Some("type".to_owned()),
            declaration: ByteSpan { start: 0, end: 8 },
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.to_owned(),
                    arity: 0,
                },
            },
            methods: Vec::new(),
            representation: None,
        }
    }

    #[test]
    fn rejects_local_host_service_type_shadows() {
        for name in ["Console", "Stdin"] {
            let mut typed = interface();
            typed.exports.push(type_export(name));

            let error =
                reject_shadowed_environment_types(&typed, &environment(name), "main").unwrap_err();

            assert!(error.contains("resolves to user type artifact/main"));
            assert!(error.contains("not standard runtime type"));
        }
    }

    #[test]
    fn rejects_imported_host_service_type_shadows_and_aliases() {
        for (name, imported, local_name) in [
            ("Console", "Console", None),
            ("Stdin", "CustomInput", Some("Stdin")),
        ] {
            let mut typed = interface();
            typed.dependencies.push(InterfaceDependency {
                specifier: "./domain".to_owned(),
                module: "artifact/domain".to_owned(),
                origin: ByteSpan { start: 0, end: 20 },
                imports: vec![InterfaceImport {
                    namespace: "type".to_owned(),
                    name: imported.to_owned(),
                    symbol: format!("artifact/domain::{imported}"),
                    local_name: local_name.map(str::to_owned),
                }],
            });

            let error =
                reject_shadowed_environment_types(&typed, &environment(name), "main").unwrap_err();

            assert!(error.contains("resolves to user type artifact/domain"));
            assert!(error.contains("not standard runtime type"));
        }
    }

    #[test]
    fn rejects_local_and_imported_effect_constructor_shadows() {
        let mut local = interface();
        local.exports.push(type_export("Effect"));
        let local_error =
            reject_shadowed_effect_constructor(&local, &effect_entry(), "main").unwrap_err();
        assert!(local_error.contains("resolves to user type artifact/main::Effect"));

        let mut imported = interface();
        imported.dependencies.push(InterfaceDependency {
            specifier: "./domain".to_owned(),
            module: "artifact/domain".to_owned(),
            origin: ByteSpan { start: 0, end: 20 },
            imports: vec![InterfaceImport {
                namespace: "type".to_owned(),
                name: "CustomEffect".to_owned(),
                symbol: "artifact/domain::CustomEffect".to_owned(),
                local_name: Some("Effect".to_owned()),
            }],
        });
        let imported_error =
            reject_shadowed_effect_constructor(&imported, &effect_entry(), "main").unwrap_err();
        assert!(imported_error.contains("resolves to user type artifact/domain::CustomEffect"));
    }

    #[test]
    fn accepts_only_unshadowed_standard_unit_as_effect_success() {
        let typed = interface();
        assert_eq!(
            validate_effect_success(&typed, &named("Unit"), "main"),
            Ok(())
        );

        let non_unit = validate_effect_success(&typed, &named("Never"), "main").unwrap_err();
        assert!(non_unit.contains("success type A must be standard Unit"));

        let mut shadowed = interface();
        shadowed.exports.push(type_export("Unit"));
        let shadowed_error =
            validate_effect_success(&shadowed, &named("Unit"), "main").unwrap_err();
        assert!(shadowed_error.contains("resolves to user type artifact/main::Unit"));
    }
}
