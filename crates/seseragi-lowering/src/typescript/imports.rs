use std::collections::{BTreeMap, BTreeSet};

use super::{
    TypeScriptBinding, TypeScriptExpr, TypeScriptFunction, TypeScriptInstanceImplementation,
    TypeScriptModule, TypeScriptShowDictionaryReference, TypeScriptStatement,
};

pub(super) fn freshen_runtime_imports(module: &mut TypeScriptModule) {
    let mut used = declaration_names(module);
    let mut renames = BTreeMap::new();
    for import in &mut module.imports {
        let original = import.local.clone();
        let fresh = fresh_name(&original, &used);
        used.insert(fresh.clone());
        if fresh != original {
            renames.insert(original, fresh.clone());
            import.local = fresh;
        }
    }
    if renames.is_empty() {
        return;
    }
    for binding in &mut module.bindings {
        match binding {
            TypeScriptBinding::Const { initializer, .. } => rewrite_expr(initializer, &renames),
        }
    }
    for function in &mut module.functions {
        match function {
            TypeScriptFunction::ConstFunction { body, .. } => rewrite_expr(body, &renames),
        }
    }
    for instance in &mut module.instances {
        rewrite_instance_imports(&mut instance.implementation, &renames);
    }
}

fn declaration_names(module: &TypeScriptModule) -> BTreeSet<String> {
    module
        .adts
        .iter()
        .flat_map(|adt| {
            std::iter::once(adt.name.clone())
                .chain(adt.variants.iter().map(|variant| variant.name.clone()))
        })
        .chain(
            module
                .structs
                .iter()
                .flat_map(|structure| [structure.name.clone(), structure.brand.clone()]),
        )
        .chain(module.bindings.iter().map(|binding| match binding {
            TypeScriptBinding::Const { name, .. } => name.clone(),
        }))
        .chain(module.functions.iter().map(|function| match function {
            TypeScriptFunction::ConstFunction { name, .. } => name.clone(),
        }))
        .chain(
            module
                .instances
                .iter()
                .map(|instance| instance.dictionary_export.clone()),
        )
        .chain(module.source_imports.iter().flat_map(|import| {
            import
                .bindings
                .iter()
                .filter(|binding| !binding.type_only)
                .map(|binding| binding.local.clone())
        }))
        .collect()
}

fn rewrite_instance_imports(
    implementation: &mut TypeScriptInstanceImplementation,
    renames: &BTreeMap<String, String>,
) {
    match implementation {
        TypeScriptInstanceImplementation::DerivedShow { variants, .. } => {
            for payload in variants
                .iter_mut()
                .filter_map(|variant| variant.payload.as_mut())
            {
                if let TypeScriptShowDictionaryReference::Runtime { local, .. } =
                    &mut payload.dictionary
                {
                    if let Some(fresh) = renames.get(local) {
                        *local = fresh.clone();
                    }
                }
            }
        }
        TypeScriptInstanceImplementation::UserDefined { methods } => {
            for method in methods {
                rewrite_expr(&mut method.body, renames);
            }
        }
    }
}

fn fresh_name(base: &str, used: &BTreeSet<String>) -> String {
    if !used.contains(base) {
        return base.to_owned();
    }
    (1usize..)
        .map(|suffix| format!("{base}_{suffix}"))
        .find(|candidate| !used.contains(candidate))
        .expect("runtime import suffix space is unbounded")
}

fn rewrite_expr(expr: &mut TypeScriptExpr, renames: &BTreeMap<String, String>) {
    match expr {
        TypeScriptExpr::RuntimeReference { name }
        | TypeScriptExpr::CurriedRuntimeReference { name, .. } => {
            if let Some(fresh) = renames.get(name) {
                *name = fresh.clone();
            }
        }
        TypeScriptExpr::RuntimeCall { callee, arguments } => {
            if let Some(fresh) = renames.get(callee) {
                *callee = fresh.clone();
            }
            for argument in arguments {
                rewrite_expr(argument, renames);
            }
        }
        TypeScriptExpr::Call { arguments, .. }
        | TypeScriptExpr::TypeApplicationCall { arguments, .. } => {
            for argument in arguments {
                rewrite_expr(argument, renames);
            }
        }
        TypeScriptExpr::DictionaryCall {
            dictionary,
            arguments,
            ..
        } => {
            rewrite_expr(dictionary, renames);
            for argument in arguments {
                rewrite_expr(argument, renames);
            }
        }
        TypeScriptExpr::Tuple { elements } | TypeScriptExpr::Array { elements, .. } => {
            for element in elements {
                rewrite_expr(element, renames);
            }
        }
        TypeScriptExpr::FieldAccess { receiver, .. } => rewrite_expr(receiver, renames),
        TypeScriptExpr::OptionalFieldAccess {
            receiver,
            just_constructor,
            nothing_constructor,
            ..
        } => {
            rewrite_expr(receiver, renames);
            if let Some(fresh) = renames.get(just_constructor) {
                *just_constructor = fresh.clone();
            }
            if let Some(fresh) = renames.get(nothing_constructor) {
                *nothing_constructor = fresh.clone();
            }
        }
        TypeScriptExpr::Record { items, .. } => {
            for item in items {
                match item {
                    super::TypeScriptRecordValueItem::Field { value, .. }
                    | super::TypeScriptRecordValueItem::Spread { value } => {
                        rewrite_expr(value, renames)
                    }
                }
            }
        }
        TypeScriptExpr::Binary { left, right, .. } => {
            rewrite_expr(left, renames);
            rewrite_expr(right, renames);
        }
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            rewrite_expr(condition, renames);
            rewrite_expr(then_branch, renames);
            rewrite_expr(else_branch, renames);
        }
        TypeScriptExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            rewrite_expr(scrutinee, renames);
            for branch in branches {
                if let Some(guard) = &mut branch.guard {
                    rewrite_expr(guard, renames);
                }
                rewrite_expr(&mut branch.value, renames);
            }
        }
        TypeScriptExpr::Await { value } => rewrite_expr(value, renames),
        TypeScriptExpr::Lambda { body, .. } => rewrite_expr(body, renames),
        TypeScriptExpr::Sequence { statements, result } => {
            for statement in statements {
                match statement {
                    TypeScriptStatement::Effect { value } => rewrite_expr(value, renames),
                    TypeScriptStatement::PureLet { initializer, .. }
                    | TypeScriptStatement::Const { initializer, .. } => {
                        rewrite_expr(initializer, renames);
                    }
                }
            }
            rewrite_expr(result, renames);
        }
        TypeScriptExpr::MonadDo {
            dictionary,
            statements,
            result,
        } => {
            rewrite_expr(dictionary, renames);
            for statement in statements {
                match statement {
                    TypeScriptStatement::Effect { value } => rewrite_expr(value, renames),
                    TypeScriptStatement::PureLet { initializer, .. }
                    | TypeScriptStatement::Const { initializer, .. } => {
                        rewrite_expr(initializer, renames);
                    }
                }
            }
            rewrite_expr(result, renames);
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::String { .. }
        | TypeScriptExpr::Boolean { .. }
        | TypeScriptExpr::Identifier { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::fresh_name;
    use std::collections::BTreeSet;

    #[test]
    fn suffixes_used_runtime_local() {
        let used = BTreeSet::from(["_ssrg_int64_add".to_owned(), "_ssrg_int64_add_1".to_owned()]);

        assert_eq!(fresh_name("_ssrg_int64_add", &used), "_ssrg_int64_add_2");
    }
}
