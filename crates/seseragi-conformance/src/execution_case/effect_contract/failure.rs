use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::{InterfaceInstance, InterfaceType};

use super::model::{DictionaryImport, EffectEntryContract, FailureRenderer};

const RUNTIME_SHOW_MODULE: &str = "@seseragi/runtime/show";

pub(crate) fn resolve_effect_entry_contract(
    interface: &TypedModuleInterface,
    failure: &InterfaceType,
    generated_module: &serde_json::Value,
) -> Result<EffectEntryContract, String> {
    if is_never(failure) {
        return Ok(EffectEntryContract {
            failure_renderer: FailureRenderer::Never,
        });
    }

    // A public local type owns its spelling. Resolve it before considering the
    // standard prelude names so `type ConsoleError ...` cannot silently pick
    // the host ConsoleError dictionary.
    if let Some(type_identity) = local_type_identity(interface, failure) {
        return resolve_local_show_dictionary(interface, failure, generated_module, type_identity);
    }

    if let Some(dictionary) = standard_show_dictionary(failure) {
        return Ok(EffectEntryContract {
            failure_renderer: FailureRenderer::Show { dictionary },
        });
    }

    Err(format!(
        "execution Effect failure {} does not resolve to a concrete local or standard Show type",
        canonical_type_spelling(failure)
    ))
}

fn resolve_local_show_dictionary(
    interface: &TypedModuleInterface,
    failure: &InterfaceType,
    generated_module: &serde_json::Value,
    type_identity: String,
) -> Result<EffectEntryContract, String> {
    let selected = selected_show_instances(interface, failure).collect::<Vec<_>>();
    match selected.len() {
        1 => {}
        0 => {
            return Err(format!(
                "execution Effect failure {} requires a selected Show instance",
                canonical_type_spelling(failure)
            ));
        }
        count => {
            return Err(format!(
                "execution Effect failure {} has {count} selected Show instances; expected exactly one",
                canonical_type_spelling(failure)
            ));
        }
    }

    let generated_instances = generated_module
        .get("instances")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "compiled generated-module.json instances is missing".to_owned())?;
    let matching = generated_instances
        .iter()
        .filter(|instance| {
            instance.get("trait").and_then(serde_json::Value::as_str) == Some("Show")
                && instance
                    .get("typeIdentity")
                    .and_then(serde_json::Value::as_str)
                    == Some(type_identity.as_str())
        })
        .collect::<Vec<_>>();
    let metadata = match matching.as_slice() {
        [metadata] => *metadata,
        [] => {
            return Err(format!(
                "compiled generated-module.json is missing the Show dictionary for {type_identity}"
            ));
        }
        _ => {
            return Err(format!(
                "compiled generated-module.json has duplicate Show dictionaries for {type_identity}"
            ));
        }
    };
    if !metadata
        .get("head")
        .is_some_and(serde_json::Value::is_object)
    {
        return Err(format!(
            "compiled generated-module.json Show dictionary for {type_identity} has no type head"
        ));
    }
    let dictionary_export = metadata
        .get("dictionaryExport")
        .and_then(serde_json::Value::as_str)
        .filter(|name| valid_typescript_identifier(name))
        .ok_or_else(|| {
            format!(
                "compiled generated-module.json Show dictionary for {type_identity} has an invalid dictionaryExport"
            )
        })?;

    Ok(EffectEntryContract {
        failure_renderer: FailureRenderer::Show {
            dictionary: DictionaryImport {
                module: "./main.ts".to_owned(),
                export: dictionary_export.to_owned(),
            },
        },
    })
}

fn standard_show_dictionary(failure: &InterfaceType) -> Option<DictionaryImport> {
    let InterfaceType::Named { name, arguments } = failure else {
        return None;
    };
    if !arguments.is_empty() {
        return None;
    }
    let export = match name.as_str() {
        "String" => "stringShow",
        "ConsoleError" => "consoleErrorShow",
        "StdinError" => "stdinErrorShow",
        _ => return None,
    };
    Some(DictionaryImport {
        module: RUNTIME_SHOW_MODULE.to_owned(),
        export: export.to_owned(),
    })
}

fn selected_show_instances<'a>(
    interface: &'a TypedModuleInterface,
    failure: &'a InterfaceType,
) -> impl Iterator<Item = &'a InterfaceInstance> {
    interface.instances.iter().filter(move |instance| {
        instance.trait_name == "Show"
            && matches!(
                &instance.head,
                InterfaceType::Apply {
                    constructor,
                    arguments,
                } if constructor == "Show"
                    && arguments.len() == 1
                    && &arguments[0] == failure
            )
    })
}

fn local_type_identity(
    interface: &TypedModuleInterface,
    failure: &InterfaceType,
) -> Option<String> {
    let InterfaceType::Named { name, arguments } = failure else {
        return None;
    };
    if !arguments.is_empty() {
        return None;
    }
    interface
        .exports
        .iter()
        .find(|export| export.namespace == "type" && export.name == *name)
        .map(|export| export.symbol.clone())
}

fn is_never(type_ref: &InterfaceType) -> bool {
    matches!(
        type_ref,
        InterfaceType::Named { name, arguments }
            if name == "Never" && arguments.is_empty()
    )
}

fn canonical_type_spelling(type_ref: &InterfaceType) -> String {
    match type_ref {
        InterfaceType::Named { name, arguments }
        | InterfaceType::Apply {
            constructor: name,
            arguments,
        } => {
            if arguments.is_empty() {
                name.clone()
            } else {
                format!(
                    "{name}<{}>",
                    arguments
                        .iter()
                        .map(canonical_type_spelling)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        InterfaceType::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(canonical_type_spelling)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        InterfaceType::Function { .. } => "<function>".to_owned(),
        InterfaceType::Record { .. } => "<record>".to_owned(),
        InterfaceType::TypeConstructor { name, .. } => name.clone(),
        InterfaceType::Hole => "_".to_owned(),
    }
}

fn valid_typescript_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && characters.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
}

#[cfg(test)]
mod tests;
