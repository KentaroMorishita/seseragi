use crate::{
    collection_ops::runtime_collection_operation_for_feature,
    effect_ops::runtime_effect_operation_for_feature, int_ops::runtime_int_operation_for_feature,
    iterator_ops::runtime_iterator_operation_for_feature,
    list_ops::runtime_list_operation_for_feature,
    prelude_ops::runtime_prelude_dictionary_for_feature,
    range_ops::runtime_range_operation_for_feature, runtime_types::runtime_type_import_for_feature,
    show_ops::runtime_show_dictionary_for_feature, sum_ops::runtime_sum_constructor_for_feature,
    web_html_ops::runtime_web_html_operation_for_feature, TypeScriptModule,
};

pub(super) fn render_import_lines(module: &TypeScriptModule) -> Vec<String> {
    let mut lines = render_source_imports(module);
    lines.extend(render_runtime_imports(module));
    lines
}

fn render_source_imports(module: &TypeScriptModule) -> Vec<String> {
    let mut lines = Vec::new();
    for import in &module.source_imports {
        if !import.bindings.is_empty() {
            let bindings = import
                .bindings
                .iter()
                .map(|binding| {
                    let kind = if binding.type_only { "type " } else { "" };
                    if binding.imported == binding.local {
                        format!("{kind}{}", binding.imported)
                    } else {
                        format!("{kind}{} as {}", binding.imported, binding.local)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!(
                "import {{ {bindings} }} from {:?}",
                import.specifier
            ));
        }
        if import.runtime_edge && !import.bindings.iter().any(|binding| !binding.type_only) {
            lines.push(format!("import {:?}", import.specifier));
        }
    }
    lines
}

fn render_runtime_imports(module: &TypeScriptModule) -> Vec<String> {
    let mut import_groups: Vec<(&str, Vec<String>)> = Vec::new();
    for import in &module.imports {
        let helper = runtime_effect_operation_for_feature(&import.feature)
            .map(|operation| (operation.module, operation.export_name))
            .or_else(|| {
                runtime_int_operation_for_feature(&import.feature)
                    .map(|operation| (operation.module, operation.export_name))
            })
            .or_else(|| {
                runtime_collection_operation_for_feature(&import.feature)
                    .map(|operation| (operation.module, operation.export_name))
            })
            .or_else(|| {
                runtime_iterator_operation_for_feature(&import.feature)
                    .map(|operation| (operation.module, operation.export_name))
            })
            .or_else(|| {
                runtime_list_operation_for_feature(&import.feature)
                    .map(|operation| (operation.module, operation.export_name))
            })
            .or_else(|| {
                runtime_range_operation_for_feature(&import.feature)
                    .map(|operation| (operation.module, operation.export_name))
            })
            .or_else(|| {
                runtime_sum_constructor_for_feature(&import.feature)
                    .map(|constructor| (constructor.module, constructor.export_name))
            })
            .or_else(|| {
                runtime_show_dictionary_for_feature(&import.feature)
                    .map(|dictionary| (dictionary.module, dictionary.export_name))
            })
            .or_else(|| {
                runtime_prelude_dictionary_for_feature(&import.feature)
                    .map(|dictionary| (dictionary.module, dictionary.export_name))
            })
            .or_else(|| {
                runtime_web_html_operation_for_feature(&import.feature)
                    .map(|operation| (operation.module, operation.export_name))
            });
        if let Some((runtime_module, export_name)) = helper {
            push_grouped_import(
                &mut import_groups,
                runtime_module,
                format!("{export_name} as {}", import.local),
            );
        }
    }
    for import in &module.type_imports {
        if let Some(type_import) = runtime_type_import_for_feature(&import.feature) {
            push_grouped_import(
                &mut import_groups,
                type_import.module,
                format!("type {} as {}", type_import.export_name, import.local),
            );
        }
    }
    import_groups
        .into_iter()
        .map(|(module, specifiers)| {
            format!("import {{ {} }} from \"{module}\"", specifiers.join(", "))
        })
        .collect()
}

fn push_grouped_import<'a>(
    groups: &mut Vec<(&'a str, Vec<String>)>,
    module: &'a str,
    specifier: String,
) {
    if let Some((_, specifiers)) = groups.iter_mut().find(|(existing, _)| *existing == module) {
        specifiers.push(specifier);
    } else {
        groups.push((module, vec![specifier]));
    }
}
