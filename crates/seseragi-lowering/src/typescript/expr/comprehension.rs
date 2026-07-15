use crate::collection_ops::runtime_iterable_operation;
use crate::{CoreComprehensionClause, CoreExpr, CorePattern};
use std::collections::BTreeMap;

use super::lower_core_expr_to_typescript;
use crate::typescript::names::safe_identifier;
use crate::typescript::TypeScriptExpr;

pub(super) fn lower_array_comprehension(
    element: CoreExpr,
    clauses: Vec<CoreComprehensionClause>,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
    depth: usize,
) -> TypeScriptExpr {
    let mut clauses = clauses.into_iter();
    let CoreComprehensionClause::Generator {
        pattern,
        source,
        evidence,
        ..
    } = clauses
        .next()
        .expect("typed comprehension requires a generator")
    else {
        panic!("typed comprehension must begin with a generator");
    };
    let parameter = match pattern {
        CorePattern::Binding { name, .. } => safe_identifier(&name),
        CorePattern::Wildcard { .. } => format!("$ssrg_item{depth}"),
        _ => panic!("unsupported comprehension pattern reached TypeScript lowering"),
    };
    let remaining = clauses.collect::<Vec<_>>();
    let next_generator = remaining
        .iter()
        .position(|clause| matches!(clause, CoreComprehensionClause::Generator { .. }));
    let (guards, nested) = next_generator
        .map(|index| remaining.split_at(index))
        .unwrap_or((&remaining, &[]));
    let predicate = guards
        .iter()
        .filter_map(|clause| match clause {
            CoreComprehensionClause::Guard { condition, .. } => Some(
                lower_core_expr_to_typescript(condition.clone(), imported_values, imported_types),
            ),
            CoreComprehensionClause::Generator { .. } => None,
        })
        .reduce(|left, right| TypeScriptExpr::Binary {
            operator: "&&".to_owned(),
            left: Box::new(left),
            right: Box::new(right),
        })
        .unwrap_or(TypeScriptExpr::Boolean { value: true });
    let flatten = !nested.is_empty();
    let transform = if flatten {
        lower_array_comprehension(
            element,
            nested.to_vec(),
            imported_values,
            imported_types,
            depth + 1,
        )
    } else {
        lower_core_expr_to_typescript(element, imported_values, imported_types)
    };
    let operation = runtime_iterable_operation(&evidence, flatten)
        .expect("typed comprehension requires materialized Iterable evidence");
    TypeScriptExpr::RuntimeCall {
        callee: operation.local_name.to_owned(),
        arguments: vec![
            lower_core_expr_to_typescript(source, imported_values, imported_types),
            TypeScriptExpr::Lambda {
                parameter: parameter.clone(),
                body: Box::new(predicate),
            },
            TypeScriptExpr::Lambda {
                parameter,
                body: Box::new(transform),
            },
        ],
    }
}
