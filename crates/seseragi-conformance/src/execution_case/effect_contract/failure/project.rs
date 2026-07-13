use seseragi_lowering::GeneratedModule;
use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::{InterfaceType, Visibility};

use super::{resolve_effect_entry_contract, valid_typescript_identifier};
use crate::execution_case::effect_contract::model::{
    DictionaryImport, EffectEntryContract, FailureRenderer, ProjectFailureRendererCatalog,
};

pub(in crate::execution_case::effect_contract) fn resolve(
    interface: &TypedModuleInterface,
    failure: &InterfaceType,
    generated_module: &GeneratedModule,
    entry_module_specifier: &str,
    catalog: &ProjectFailureRendererCatalog<'_>,
) -> Result<EffectEntryContract, String> {
    if matches!(failure, InterfaceType::ExternalNamed { .. }) {
        return resolve_external_show_dictionary(interface, failure, catalog);
    }
    resolve_effect_entry_contract(interface, failure, generated_module, entry_module_specifier)
}

fn resolve_external_show_dictionary(
    interface: &TypedModuleInterface,
    failure: &InterfaceType,
    catalog: &ProjectFailureRendererCatalog<'_>,
) -> Result<EffectEntryContract, String> {
    let InterfaceType::ExternalNamed {
        canonical,
        provider_module: failure_provider,
        provider_export,
        arguments,
        ..
    } = failure
    else {
        unreachable!("the project resolver only calls this for external failures");
    };
    if !arguments.is_empty() {
        return Err(format!(
            "project execution Effect failure {canonical} is generic; imported generic failure dictionaries are not supported"
        ));
    }

    let selected = interface
        .instances
        .iter()
        .filter(|instance| {
            instance.trait_name == "Show"
                && instance.type_identity.as_deref() == Some(canonical.as_str())
        })
        .collect::<Vec<_>>();
    let evidence = match selected.as_slice() {
        [evidence] => *evidence,
        [] => {
            if interface.instances.iter().any(|instance| {
                instance.trait_name == "Show"
                    && instance.provider_module.as_deref() == Some(failure_provider.as_str())
                    && instance.type_identity.is_none()
            }) {
                return Err(format!(
                    "project execution selected Show evidence for {canonical} is missing typeIdentity"
                ));
            }
            return Err(format!(
                "project execution Effect failure {canonical} requires exactly one selected Show evidence; found none"
            ));
        }
        evidence => {
            return Err(format!(
                "project execution Effect failure {canonical} requires exactly one selected Show evidence; found {}",
                evidence.len()
            ));
        }
    };
    let provider_module = evidence.provider_module.as_deref().ok_or_else(|| {
        format!(
            "project execution selected Show evidence for {canonical} is missing providerModule"
        )
    })?;
    let identity = evidence
        .identity
        .as_deref()
        .filter(|identity| !identity.is_empty())
        .ok_or_else(|| {
            format!(
                "project execution selected Show evidence for {canonical} is missing its final concrete identity"
            )
        })?;
    if !evidence.type_parameters.is_empty() {
        return Err(format!(
            "project execution selected Show evidence for {canonical} is generic; execution requires concrete evidence"
        ));
    }
    if !evidence.constraints.is_empty() {
        return Err(format!(
            "project execution selected Show evidence for {canonical} is conditional; execution requires constraint-free evidence"
        ));
    }
    let type_identity = evidence
        .type_identity
        .as_deref()
        .expect("selected evidence matched its structured type identity");
    if type_identity != canonical {
        return Err(format!(
            "project execution selected Show evidence typeIdentity mismatch: expected {canonical}, got {type_identity}"
        ));
    }
    if provider_module != failure_provider {
        return Err(format!(
            "project execution selected Show evidence providerModule mismatch for {canonical}: expected {failure_provider}, got {provider_module}"
        ));
    }

    validate_failure_type_owner(catalog, provider_module, provider_export, canonical)?;

    let provider = catalog.generated_module(provider_module).ok_or_else(|| {
        format!(
            "project execution Show provider module {provider_module} has no compiled generated metadata"
        )
    })?;
    if provider.module != provider_module {
        return Err(format!(
            "project execution Show provider metadata module mismatch: expected {provider_module}, got {}",
            provider.module
        ));
    }
    let matching = provider
        .instances
        .iter()
        .filter(|instance| {
            instance.trait_name == "Show" && instance.type_identity == canonical.as_str()
        })
        .collect::<Vec<_>>();
    let metadata = match matching.as_slice() {
        [metadata] => *metadata,
        [] => {
            return Err(format!(
                "project execution Show provider {provider_module} is missing dictionary metadata for {canonical}"
            ));
        }
        _ => {
            return Err(format!(
                "project execution Show provider {provider_module} has ambiguous dictionary metadata for {canonical}"
            ));
        }
    };
    if metadata.identity != identity {
        return Err(format!(
            "project execution Show provider {provider_module} dictionary identity mismatch for {canonical}: expected {identity}, got {}",
            metadata.identity
        ));
    }
    if !valid_typescript_identifier(&metadata.dictionary_export) {
        return Err(format!(
            "project execution Show provider {provider_module} dictionary for {canonical} has an invalid dictionaryExport"
        ));
    }

    let module = catalog
        .wrapper_module_specifier(provider_module)
        .ok_or_else(|| {
            format!(
                "project execution Show provider module {provider_module} has no staged TypeScript output specifier"
            )
        })?;

    Ok(EffectEntryContract {
        failure_renderer: FailureRenderer::Show {
            dictionary: DictionaryImport {
                module: module.to_owned(),
                export: metadata.dictionary_export.clone(),
            },
        },
    })
}

fn validate_failure_type_owner(
    catalog: &ProjectFailureRendererCatalog<'_>,
    provider_module: &str,
    provider_export: &str,
    canonical: &str,
) -> Result<(), String> {
    let interface = catalog.typed_interface(provider_module).ok_or_else(|| {
        format!(
            "project execution Show provider module {provider_module} has no final typed interface"
        )
    })?;
    if interface.schema != 1 || interface.stage != "typed-interface" {
        return Err(format!(
            "project execution Show provider module {provider_module} does not expose a schema-1 final typed interface"
        ));
    }
    if interface.module != provider_module {
        return Err(format!(
            "project execution Show provider typed interface module mismatch: expected {provider_module}, got {}",
            interface.module
        ));
    }

    let matching = interface
        .exports
        .iter()
        .filter(|export| {
            export.namespace == "type"
                && export.visibility == Visibility::Public
                && export.symbol == canonical
                && export.name == provider_export
                && export.declaration_kind.as_deref() == Some("type")
                && export.scheme.type_parameters.is_empty()
                && export.scheme.constraints.is_empty()
                && matches!(
                    &export.scheme.type_ref,
                    InterfaceType::TypeConstructor { name, arity }
                        if name == provider_export && *arity == 0
                )
        })
        .collect::<Vec<_>>();
    match matching.len() {
        1 => Ok(()),
        0 => Err(format!(
            "project execution Show provider {provider_module} does not publicly own monomorphic type export {provider_export} with canonical identity {canonical}"
        )),
        count => Err(format!(
            "project execution Show provider {provider_module} has {count} matching public monomorphic type exports for {canonical}"
        )),
    }
}
