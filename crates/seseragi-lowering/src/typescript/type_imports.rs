use crate::runtime_types::{runtime_type_import, runtime_type_imports};
use crate::{CoreModule, CoreType};
use seseragi_semantics::ExternalTypeBinding;

use super::names::safe_identifier;
use super::{push_unique, TypeScriptTypeImport};

mod expr;

pub(super) fn collect_module_type_imports(
    module: &CoreModule,
    requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptTypeImport>,
) {
    let type_bindings = runtime_type_bindings(module);
    for adt in &module.adts {
        for variant in &adt.variants {
            if let Some(payload) = &variant.payload {
                collect_type_imports(payload, &type_bindings, requirements, imports);
            }
        }
    }
    for structure in &module.structs {
        for field in &structure.fields {
            collect_type_imports(&field.type_ref, &type_bindings, requirements, imports);
        }
    }
    for binding in &module.bindings {
        expr::collect_expr_type_imports(&binding.value, &type_bindings, requirements, imports);
    }
    for function in &module.functions {
        for parameter in &function.parameters {
            collect_type_imports(&parameter.type_ref, &type_bindings, requirements, imports);
        }
        expr::collect_expr_type_imports(&function.body, &type_bindings, requirements, imports);
    }
    for instance in &module.instances {
        for argument in &instance.arguments {
            collect_type_imports(argument, &type_bindings, requirements, imports);
        }
        for constraint in &instance.constraints {
            for argument in &constraint.arguments {
                collect_type_imports(argument, &type_bindings, requirements, imports);
            }
        }
        if let crate::CoreInstanceImplementation::UserDefined { methods } = &instance.implementation
        {
            for method in methods {
                for parameter in &method.parameters {
                    collect_type_imports(
                        &parameter.type_ref,
                        &type_bindings,
                        requirements,
                        imports,
                    );
                }
                expr::collect_expr_type_imports(
                    &method.body,
                    &type_bindings,
                    requirements,
                    imports,
                );
            }
        }
    }
}

fn runtime_type_bindings(module: &CoreModule) -> Vec<ExternalTypeBinding> {
    let mut bindings = module.external_type_bindings.clone();
    for type_import in runtime_type_imports() {
        let shadowed_by_local = module
            .adts
            .iter()
            .any(|adt| adt.name == type_import.export_name)
            || module
                .structs
                .iter()
                .any(|structure| structure.name == type_import.export_name);
        let already_bound = bindings
            .iter()
            .any(|binding| binding.spelling == type_import.export_name);
        if shadowed_by_local || already_bound {
            continue;
        }
        bindings.push(ExternalTypeBinding {
            spelling: type_import.export_name.to_owned(),
            canonical: type_import.canonical.to_owned(),
            provider: None,
        });
    }
    bindings
}

fn collect_type_imports(
    type_ref: &CoreType,
    bindings: &[ExternalTypeBinding],
    requirements: &mut Vec<String>,
    imports: &mut Vec<TypeScriptTypeImport>,
) {
    match type_ref {
        CoreType::Named { name, arguments } => {
            if let Some(type_import) = unambiguous_runtime_type(name, bindings) {
                push_unique(requirements, type_import.runtime_feature);
                push_type_import_unique(
                    imports,
                    TypeScriptTypeImport {
                        feature: type_import.runtime_feature.to_owned(),
                        local: safe_identifier(type_import.export_name),
                    },
                );
            }
            for argument in arguments {
                collect_type_imports(argument, bindings, requirements, imports);
            }
        }
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => {
            if let Some(type_import) = runtime_type_import(canonical) {
                push_unique(requirements, type_import.runtime_feature);
                push_type_import_unique(
                    imports,
                    TypeScriptTypeImport {
                        feature: type_import.runtime_feature.to_owned(),
                        local: safe_identifier(type_import.export_name),
                    },
                );
            }
            for argument in arguments {
                collect_type_imports(argument, bindings, requirements, imports);
            }
        }
        CoreType::Record { fields, .. } => {
            for field in fields {
                collect_type_imports(&field.type_ref, bindings, requirements, imports);
            }
        }
        CoreType::Tuple { elements } => {
            for element in elements {
                collect_type_imports(element, bindings, requirements, imports);
            }
        }
        CoreType::Function { parameter, result } => {
            collect_type_imports(parameter, bindings, requirements, imports);
            collect_type_imports(result, bindings, requirements, imports);
        }
        CoreType::Hole => {}
    }
}

fn push_type_import_unique(imports: &mut Vec<TypeScriptTypeImport>, import: TypeScriptTypeImport) {
    if !imports
        .iter()
        .any(|existing| existing.feature == import.feature && existing.local == import.local)
    {
        imports.push(import);
    }
}

fn unambiguous_runtime_type(
    spelling: &str,
    bindings: &[ExternalTypeBinding],
) -> Option<crate::runtime_types::RuntimeTypeImport> {
    let mut canonical = None;
    for binding in bindings
        .iter()
        .filter(|binding| binding.spelling == spelling)
    {
        match canonical {
            None => canonical = Some(binding.canonical.as_str()),
            Some(existing) if existing == binding.canonical => {}
            Some(_) => return None,
        }
    }
    canonical.and_then(runtime_type_import)
}

#[cfg(test)]
mod tests;
