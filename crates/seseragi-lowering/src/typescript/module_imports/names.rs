use crate::{CoreExpr, CoreModule, CoreStatement};
use std::collections::BTreeSet;

use super::super::names::safe_identifier;

pub(super) fn fresh_name(base: &str, used: &BTreeSet<String>) -> String {
    if !used.contains(base) {
        return base.to_owned();
    }
    (1usize..)
        .map(|suffix| format!("{base}_{suffix}"))
        .find(|candidate| !used.contains(candidate))
        .expect("source import suffix space is unbounded")
}

pub(super) fn local_value_names(module: &CoreModule) -> BTreeSet<String> {
    let mut names = module
        .adts
        .iter()
        .flat_map(|adt| adt.variants.iter().map(|variant| variant.name.clone()))
        .chain(module.bindings.iter().map(|binding| {
            binding
                .symbol
                .rsplit_once("::")
                .map_or(binding.symbol.as_str(), |(_, name)| name)
                .to_owned()
        }))
        .chain(module.functions.iter().map(|function| {
            function
                .symbol
                .rsplit_once("::")
                .map_or(function.symbol.as_str(), |(_, name)| name)
                .to_owned()
        }))
        .chain(
            module
                .instances
                .iter()
                .enumerate()
                .map(|(index, _)| format!("__ssrg$instance$Show${index}")),
        )
        .map(|name| safe_identifier(&name))
        .collect::<BTreeSet<_>>();
    for function in &module.functions {
        for parameter in &function.parameters {
            names.insert(safe_identifier(&parameter.id));
        }
        collect_local_expr_names(&function.body, &mut names);
    }
    names
}

fn collect_local_expr_names(expr: &CoreExpr, names: &mut BTreeSet<String>) {
    match expr {
        CoreExpr::Tuple { elements, .. }
        | CoreExpr::Array { elements, .. }
        | CoreExpr::EffectOperation {
            arguments: elements,
            ..
        }
        | CoreExpr::EffectInvoke {
            arguments: elements,
            ..
        } => {
            for element in elements {
                collect_local_expr_names(element, names);
            }
        }
        CoreExpr::Call { arguments, .. } => {
            for argument in arguments {
                collect_local_expr_names(argument, names);
            }
        }
        CoreExpr::Binary { left, right, .. } => {
            collect_local_expr_names(left, names);
            collect_local_expr_names(right, names);
        }
        CoreExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_local_expr_names(condition, names);
            collect_local_expr_names(then_branch, names);
            collect_local_expr_names(else_branch, names);
        }
        CoreExpr::Decision {
            scrutinee,
            branches,
            ..
        } => {
            collect_local_expr_names(scrutinee, names);
            for branch in branches {
                for binding in &branch.bindings {
                    names.insert(safe_identifier(&binding.name));
                }
                if let Some(guard) = &branch.guard {
                    collect_local_expr_names(guard, names);
                }
                collect_local_expr_names(&branch.value, names);
            }
        }
        CoreExpr::Sequence {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    CoreStatement::Effect { value } => collect_local_expr_names(value, names),
                    CoreStatement::PureLet { name, value, .. }
                    | CoreStatement::Bind { name, value, .. } => {
                        names.insert(safe_identifier(name));
                        collect_local_expr_names(value, names);
                    }
                }
            }
            collect_local_expr_names(result, names);
        }
        CoreExpr::Unit { .. }
        | CoreExpr::Int64 { .. }
        | CoreExpr::String { .. }
        | CoreExpr::Boolean { .. }
        | CoreExpr::Variable { .. } => {}
    }
}

pub(super) fn local_type_names(module: &CoreModule) -> BTreeSet<String> {
    module
        .adts
        .iter()
        .map(|adt| safe_identifier(&adt.name))
        .chain(
            module
                .functions
                .iter()
                .flat_map(|function| function.type_parameters.iter())
                .map(|parameter| safe_identifier(parameter)),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::fresh_name;
    use std::collections::BTreeSet;

    #[test]
    fn freshens_a_sanitized_source_import_collision() {
        let used = BTreeSet::from(["render_".to_owned(), "render__1".to_owned()]);

        assert_eq!(fresh_name("render_", &used), "render__2");
    }
}
