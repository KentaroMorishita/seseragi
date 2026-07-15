use crate::typescript::types::render_typescript_type;
use crate::{
    TypeScriptDerivedShowVariant, TypeScriptInstance, TypeScriptInstanceImplementation,
    TypeScriptInstanceMethod, TypeScriptShowDictionaryReference, TypeScriptTypeImport,
};

const SHOW_DICTIONARY_FEATURE: &str = "core.show.dictionary";

/// Emits one compiler-private exported dictionary per TypeScript IR instance.
///
/// Each dictionary occupies exactly one generated line so source-map accounting
/// can advance by `instances.len()` without inspecting rendered text.
pub(super) fn render_typescript_instances(
    output: &mut String,
    instances: &[TypeScriptInstance],
    type_imports: &[TypeScriptTypeImport],
) {
    if instances.is_empty() {
        return;
    }
    let show_type_local = type_imports
        .iter()
        .find(|import| import.feature == SHOW_DICTIONARY_FEATURE)
        .map(|import| import.local.as_str());

    for instance in instances {
        output.push_str(&render_instance(instance, show_type_local));
        output.push('\n');
    }
}

fn render_instance(instance: &TypeScriptInstance, show_type_local: Option<&str>) -> String {
    match &instance.implementation {
        TypeScriptInstanceImplementation::DerivedShow { adt_name, variants } => {
            let show_type_local = show_type_local
                .expect("TypeScript Show instances require the resolved Show type import");
            let head = render_typescript_type(
                instance
                    .arguments
                    .first()
                    .expect("DerivedShow instance must retain one head argument"),
            );
            let body = render_derived_show_body(adt_name, variants);
            format!(
                "export const {}: {show_type_local}<{head}> = {{ show: {body} }};",
                instance.dictionary_export
            )
        }
        TypeScriptInstanceImplementation::UserDefined { methods } => {
            render_user_defined_instance(instance, methods)
        }
    }
}

fn render_user_defined_instance(
    instance: &TypeScriptInstance,
    methods: &[TypeScriptInstanceMethod],
) -> String {
    let body = methods
        .iter()
        .map(|method| {
            format!(
                "{:?}: {}",
                method.name,
                super::render_function_body(
                    &method.type_parameters,
                    &method.parameters,
                    &method.body,
                    method.is_async,
                )
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let dictionary = format!("{{ {body} }}");
    if instance.type_parameters.is_empty() && instance.constraints.is_empty() {
        return format!(
            "export const {} = {dictionary} as const;",
            instance.dictionary_export
        );
    }

    let generics = super::render_arrow_type_parameters(&instance.type_parameters);
    let evidence = instance
        .constraints
        .iter()
        .enumerate()
        .map(|(index, _)| format!("_evidence{index}: unknown"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "export const {} = {generics}({evidence}) => ({dictionary}) as const;",
        instance.dictionary_export
    )
}

fn render_derived_show_body(adt_name: &str, variants: &[TypeScriptDerivedShowVariant]) -> String {
    if variants.is_empty() {
        return format!("(value: {adt_name}): string => value");
    }
    let cases = variants
        .iter()
        .map(render_derived_show_variant)
        .collect::<Vec<_>>()
        .join(" ");
    format!("(value: {adt_name}): string => {{ switch (value.tag) {{ {cases} }} }}")
}

fn render_derived_show_variant(variant: &TypeScriptDerivedShowVariant) -> String {
    let tag = format!("{:?}", variant.tag);
    let result = match &variant.payload {
        None => tag.clone(),
        Some(payload) => format!(
            "{tag} + \" \" + {}.show(value.value)",
            render_dictionary_reference(&payload.dictionary)
        ),
    };
    format!("case {tag}: return {result};")
}

fn render_dictionary_reference(reference: &TypeScriptShowDictionaryReference) -> &str {
    match reference {
        TypeScriptShowDictionaryReference::Runtime { local, .. } => local,
        TypeScriptShowDictionaryReference::Local {
            dictionary_export, ..
        } => dictionary_export,
        TypeScriptShowDictionaryReference::Imported { local, .. } => local,
    }
}

#[cfg(test)]
mod tests;
