use crate::typescript::types::render_typescript_type;
use crate::{
    TypeScriptDerivedShowField, TypeScriptDerivedShowVariant, TypeScriptInstance,
    TypeScriptInstanceImplementation, TypeScriptInstanceMethod, TypeScriptShowDictionaryReference,
    TypeScriptTypeImport,
};

const SHOW_DICTIONARY_FEATURE: &str = "core.show.dictionary";
const DEBUG_DICTIONARY_FEATURE: &str = "core.debug.dictionary";

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
    for instance in instances {
        let dictionary_feature = match instance.trait_name.as_str() {
            "Show" => SHOW_DICTIONARY_FEATURE,
            "Debug" => DEBUG_DICTIONARY_FEATURE,
            _ => "",
        };
        let display_type_local = type_imports
            .iter()
            .find(|import| import.feature == dictionary_feature)
            .map(|import| import.local.as_str());
        output.push_str(&render_instance(instance, display_type_local));
        output.push('\n');
    }
}

fn render_instance(instance: &TypeScriptInstance, display_type_local: Option<&str>) -> String {
    match &instance.implementation {
        TypeScriptInstanceImplementation::DerivedShow { adt_name, variants } => {
            let _ = adt_name;
            render_derived_display_instance(instance, display_type_local, |head, method| {
                render_derived_adt_body(head, method, variants)
            })
        }
        TypeScriptInstanceImplementation::DerivedStructShow {
            struct_name,
            fields,
        } => {
            let _ = struct_name;
            render_derived_display_instance(instance, display_type_local, |head, method| {
                render_derived_struct_body(head, method, fields)
            })
        }
        TypeScriptInstanceImplementation::DerivedJson {
            variants,
            transparent_newtype,
            ..
        } => render_derived_json_adt_instance(instance, variants, *transparent_newtype),
        TypeScriptInstanceImplementation::DerivedStructJson { fields, .. } => {
            render_derived_json_struct_instance(instance, fields)
        }
        TypeScriptInstanceImplementation::DerivedStructural {
            variants,
            transparent_newtype,
            ..
        } => render_structural_instance(instance, Some((variants, *transparent_newtype)), &[]),
        TypeScriptInstanceImplementation::DerivedStructStructural { fields, .. } => {
            render_structural_instance(instance, None, fields)
        }
        TypeScriptInstanceImplementation::UserDefined { methods } => {
            render_user_defined_instance(instance, methods)
        }
    }
}

fn render_structural_instance(
    instance: &TypeScriptInstance,
    adt: Option<(&[TypeScriptDerivedShowVariant], bool)>,
    fields: &[TypeScriptDerivedShowField],
) -> String {
    let head = render_instance_head(instance);
    let method = match instance.trait_name.as_str() {
        "Eq" => "eq",
        "Ord" => "compare",
        "Hash" => "hash",
        _ => unreachable!(),
    };
    let result = match method {
        "eq" => "boolean",
        "compare" => "{ readonly tag: \"Less\" | \"Equal\" | \"Greater\" }",
        _ => "number",
    };
    let body = if let Some((variants, transparent)) = adt {
        let mut body = String::new();
        if method == "eq" {
            body.push_str("if (left.tag !== right.tag) return false; ");
        }
        if method == "compare" && !transparent {
            let tags = render_string_array(variants.iter().map(|variant| variant.tag.as_str()));
            body.push_str(&format!("if (left.tag !== right.tag) return {{ tag: {tags}.indexOf(left.tag) < {tags}.indexOf(right.tag) ? \"Less\" : \"Greater\" }}; "));
        }
        body.push_str("switch (left.tag) { ");
        for (index, variant) in variants.iter().enumerate() {
            body.push_str(&format!("case {:?}: {{ ", variant.tag));
            if method != "hash" {
                body.push_str(&format!(
                    "if (right.tag !== {:?}) throw new Error(\"invalid derived comparison\"); ",
                    variant.tag
                ));
            }
            let members = variant
                .payload
                .as_ref()
                .map(|payload| {
                    vec![(
                        "left.value".to_owned(),
                        "right.value".to_owned(),
                        &payload.dictionary,
                    )]
                })
                .unwrap_or_default();
            body.push_str(&structural_body(
                method,
                &members,
                if transparent { None } else { Some(index) },
            ));
            body.push_str(" } ");
        }
        body.push_str("} throw new Error(\"invalid derived value\");");
        body
    } else {
        let members = fields
            .iter()
            .map(|field| {
                (
                    format!("left[{:?}]", field.name),
                    format!("right[{:?}]", field.name),
                    &field.dictionary,
                )
            })
            .collect::<Vec<_>>();
        structural_body(method, &members, Some(0))
    };
    let function = if method == "hash" {
        format!("(left: {head}): {result} => {{ {body} }}")
    } else {
        format!("(left: {head}) => (right: {head}): {result} => {{ {body} }}")
    };
    let inherited = if instance.supertrait_count > 0 {
        format!("...{}, ", crate::typescript::evidence_parameter_name(0))
    } else {
        String::new()
    };
    let dictionary = format!("{{ {inherited}{method}: {function} }}");
    let annotation = render_structural_constraint_type(&instance.trait_name, &head);
    if instance.type_parameters.is_empty() && instance.constraints.is_empty() {
        format!(
            "export const {}: {annotation} = {dictionary};",
            instance.dictionary_export
        )
    } else {
        let generics = super::render_arrow_type_parameters(&instance.type_parameters);
        let evidence = instance
            .constraints
            .iter()
            .enumerate()
            .map(|(index, _)| format!("__ssrg$input${index}: {}", super::ERASED_EVIDENCE_TYPE))
            .collect::<Vec<_>>()
            .join(", ");
        let bindings = instance
            .constraints
            .iter()
            .enumerate()
            .map(|(index, constraint)| {
                let dictionary_type = render_structural_constraint_type(
                    &constraint.name,
                    &render_typescript_type(&constraint.arguments[0]),
                );
                format!(
                    "const {} = __ssrg$input${index} as {dictionary_type};",
                    crate::typescript::evidence_parameter_name(index)
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "export const {} = {generics}({evidence}): {annotation} => {{ {bindings} return {dictionary}; }};",
            instance.dictionary_export
        )
    }
}

fn render_structural_constraint_type(trait_name: &str, head: &str) -> String {
    let eq = format!("eq: (left: {head}) => (right: {head}) => boolean");
    match trait_name {
        "Eq" => format!("{{ {eq} }}"),
        "Ord" => format!("{{ {eq}; compare: (left: {head}) => (right: {head}) => {{ readonly tag: \"Less\" | \"Equal\" | \"Greater\" }} }}"),
        "Hash" => format!("{{ hash: (value: {head}) => number }}"),
        _ => unreachable!("structural deriving only uses standard structural constraints"),
    }
}

fn structural_body(
    method: &str,
    members: &[(String, String, &TypeScriptShowDictionaryReference)],
    seed: Option<usize>,
) -> String {
    let calls = members
        .iter()
        .map(|(left, right, dictionary)| {
            let dictionary = render_dictionary_reference(dictionary);
            if method == "hash" {
                format!("{dictionary}.hash({left})")
            } else {
                format!("{dictionary}.{method}({left})({right})")
            }
        })
        .collect::<Vec<_>>();
    match method {
        "eq" => format!("return {};", if calls.is_empty() { "true".to_owned() } else { calls.join(" && ") }),
        "compare" => format!("{} return {{ tag: \"Equal\" }};", calls.iter().enumerate().map(|(index, call)| format!("const order{index} = {call}; if (order{index}.tag !== \"Equal\") return order{index};")).collect::<Vec<_>>().join(" ")),
        "hash" if seed.is_none() => format!("return {};", calls[0]),
        "hash" => format!("let state = {}; {} return state | 0;", seed.unwrap(), calls.iter().map(|call| format!("state = Math.imul(state ^ ({call}), 16777619);")).collect::<Vec<_>>().join(" ")),
        _ => unreachable!(),
    }
}

fn render_derived_json_struct_instance(
    instance: &TypeScriptInstance,
    fields: &[TypeScriptDerivedShowField],
) -> String {
    let head = render_instance_head(instance);
    let names = render_string_array(fields.iter().map(|field| field.name.as_str()));
    let dictionaries = render_dictionary_thunk_array(fields.iter().map(|field| &field.dictionary));
    let direction = json_direction(instance);
    let helper = format!("_ssrg_json_derivedstruct_{direction}");
    render_derived_json_factory(
        instance,
        &head,
        format!("{helper}<{head}>({names}, {dictionaries})"),
    )
}

fn render_derived_json_adt_instance(
    instance: &TypeScriptInstance,
    variants: &[TypeScriptDerivedShowVariant],
    transparent_newtype: bool,
) -> String {
    let head = render_instance_head(instance);
    let direction = json_direction(instance);
    let body = if transparent_newtype {
        let variant = variants
            .first()
            .expect("derived newtype JSON codec must retain its constructor");
        let payload = variant
            .payload
            .as_ref()
            .expect("derived newtype JSON codec must retain its representation");
        let thunk = render_dictionary_thunk(&payload.dictionary);
        let helper = format!("_ssrg_json_derivednewtype_{direction}");
        if direction == "decode" {
            format!("{helper}<{head}>({:?}, {thunk})", variant.tag)
        } else {
            format!("{helper}<{head}>({thunk})")
        }
    } else {
        let cases = variants
            .iter()
            .map(|variant| {
                let dictionary = variant.payload.as_ref().map_or_else(
                    || "undefined".to_owned(),
                    |payload| render_dictionary_thunk(&payload.dictionary),
                );
                format!("[{:?}, {dictionary}]", variant.tag)
            })
            .collect::<Vec<_>>()
            .join(", ");
        let helper = format!("_ssrg_json_derivedadt_{direction}");
        format!("{helper}<{head}>([{cases}])")
    };
    render_derived_json_factory(instance, &head, body)
}

fn render_derived_json_factory(
    instance: &TypeScriptInstance,
    head: &str,
    dictionary: String,
) -> String {
    let dictionary_type = match instance.trait_name.as_str() {
        "JsonEncode" => "_ssrg_json_JsonEncode",
        "JsonDecode" => "_ssrg_json_JsonDecode",
        trait_name => panic!("unsupported derived JSON trait {trait_name}"),
    };
    if instance.type_parameters.is_empty() && instance.constraints.is_empty() {
        return format!(
            "export const {}: {dictionary_type}<{head}> = {dictionary};",
            instance.dictionary_export
        );
    }
    let generics = super::render_arrow_type_parameters(&instance.type_parameters);
    let evidence = instance
        .constraints
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                "{}: {}",
                crate::typescript::evidence_parameter_name(index),
                super::ERASED_EVIDENCE_TYPE,
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "export const {} = {generics}({evidence}): {dictionary_type}<{head}> => ({dictionary});",
        instance.dictionary_export
    )
}

fn render_instance_head(instance: &TypeScriptInstance) -> String {
    render_typescript_type(
        instance
            .arguments
            .first()
            .expect("derived instance must retain one head argument"),
    )
}

fn json_direction(instance: &TypeScriptInstance) -> &'static str {
    match instance.trait_name.as_str() {
        "JsonEncode" => "encode",
        "JsonDecode" => "decode",
        trait_name => panic!("unsupported derived JSON trait {trait_name}"),
    }
}

fn render_string_array<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    format!(
        "[{}]",
        values
            .into_iter()
            .map(|value| format!("{value:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_dictionary_thunk_array<'a>(
    dictionaries: impl IntoIterator<Item = &'a TypeScriptShowDictionaryReference>,
) -> String {
    format!(
        "[{}]",
        dictionaries
            .into_iter()
            .map(render_dictionary_thunk)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_dictionary_thunk(reference: &TypeScriptShowDictionaryReference) -> String {
    format!("() => ({})", render_dictionary_reference(reference))
}

fn render_derived_display_instance(
    instance: &TypeScriptInstance,
    display_type_local: Option<&str>,
    render_body: impl FnOnce(&str, &str) -> String,
) -> String {
    let display_type_local = display_type_local
        .expect("TypeScript derived display instances require their dictionary type import");
    let head = render_typescript_type(
        instance
            .arguments
            .first()
            .expect("derived display instance must retain one head argument"),
    );
    let method = match instance.trait_name.as_str() {
        "Show" => "show",
        "Debug" => "debug",
        trait_name => panic!("unsupported derived display trait {trait_name}"),
    };
    let body = render_body(&head, method);
    let bounded = match instance.trait_name.as_str() {
        "Show" => "_ssrg_show_boundedShow",
        "Debug" => "_ssrg_debug_boundedDebug",
        trait_name => panic!("unsupported derived display trait {trait_name}"),
    };
    let dictionary = format!("{bounded}({body})");
    if instance.type_parameters.is_empty() && instance.constraints.is_empty() {
        return format!(
            "export const {}: {display_type_local}<{head}> = {dictionary};",
            instance.dictionary_export
        );
    }
    let generics = super::render_arrow_type_parameters(&instance.type_parameters);
    let evidence = instance
        .constraints
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                "{}: {}",
                crate::typescript::evidence_parameter_name(index),
                super::ERASED_EVIDENCE_TYPE,
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "export const {} = {generics}({evidence}): {display_type_local}<{head}> => ({dictionary});",
        instance.dictionary_export
    )
}

fn render_user_defined_instance(
    instance: &TypeScriptInstance,
    methods: &[TypeScriptInstanceMethod],
) -> String {
    let inherited = (0..instance.supertrait_count)
        .map(|index| format!("...{}", crate::typescript::evidence_parameter_name(index)));
    let body = inherited
        .chain(methods.iter().map(|method| {
            let parameters = super::evidence_parameters(
                &method.parameters,
                instance.constraints.len(),
                method.constraints.len(),
            );
            format!(
                "{:?}: {}",
                method.name,
                super::render_function_body(
                    &method.type_parameters,
                    &parameters,
                    &method.body,
                    method.is_async,
                    false,
                    None,
                )
            )
        }))
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
        .map(|(index, _)| {
            format!(
                "{}: {}",
                crate::typescript::evidence_parameter_name(index),
                super::ERASED_EVIDENCE_TYPE,
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "export const {} = {generics}({evidence}) => ({dictionary}) as const;",
        instance.dictionary_export
    )
}

fn render_derived_adt_body(
    head: &str,
    method: &str,
    variants: &[TypeScriptDerivedShowVariant],
) -> String {
    if variants.is_empty() {
        return format!("(value: {head}): string => value");
    }
    let cases = variants
        .iter()
        .map(|variant| render_derived_adt_variant(variant, method))
        .collect::<Vec<_>>()
        .join(" ");
    format!("(value: {head}): string => {{ switch (value.tag) {{ {cases} }} }}")
}

fn render_derived_adt_variant(variant: &TypeScriptDerivedShowVariant, method: &str) -> String {
    let tag = format!("{:?}", variant.tag);
    let result = match &variant.payload {
        None => tag.clone(),
        Some(payload) => format!(
            "{tag} + \" \" + {}.{method}(value.value)",
            render_dictionary_reference(&payload.dictionary),
        ),
    };
    format!("case {tag}: return {result};")
}

fn render_derived_struct_body(
    head: &str,
    method: &str,
    fields: &[TypeScriptDerivedShowField],
) -> String {
    let rendered_fields = fields
        .iter()
        .map(|field| {
            format!(
                "{:?} + {}.{method}(value[{:?}])",
                format!("{}: ", field.name),
                render_dictionary_reference(&field.dictionary),
                field.name,
            )
        })
        .collect::<Vec<_>>()
        .join(" + \", \" + ");
    let type_name = head.split('<').next().unwrap_or(head);
    let body = if rendered_fields.is_empty() {
        format!("{:?}", format!("{type_name} {{}}"))
    } else {
        format!(
            "{:?} + {rendered_fields} + {:?}",
            format!("{type_name} {{ "),
            " }"
        )
    };
    format!("(value: {head}): string => {body}")
}

fn render_dictionary_reference(reference: &TypeScriptShowDictionaryReference) -> String {
    match reference {
        TypeScriptShowDictionaryReference::Runtime { local, .. } => local.clone(),
        TypeScriptShowDictionaryReference::Local {
            dictionary_export, ..
        } => dictionary_export.clone(),
        TypeScriptShowDictionaryReference::Imported { local, .. } => local.clone(),
        TypeScriptShowDictionaryReference::Expression { expression } => {
            format!("({})", super::render_typescript_expr(expression))
        }
    }
}

#[cfg(test)]
mod tests;
