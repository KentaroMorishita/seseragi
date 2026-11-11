//! `(value: Type)` annotates an expression in other languages. In Seseragi
//! `:` there is the List cons operator, so the type is read as a value. A
//! cons whose right operand is a type name written right after `value:` and
//! closed by `)` is reported as the annotation it was meant to be.
use crate::{
    prelude::{STANDALONE_TYPE_NAMES, SUM_TYPES},
    ResolvedModule, SymbolNamespace,
};
use seseragi_syntax::Diagnostic;

pub(super) fn guide_expression_annotations(
    diagnostics: &mut [Diagnostic],
    resolved: &ResolvedModule,
    source: &str,
) {
    for diagnostic in diagnostics {
        if diagnostic.message_key != "call.argument-type-mismatch"
            || !diagnostic
                .expected_actual_types()
                .0
                .is_some_and(|expected| expected.starts_with("List<"))
        {
            continue;
        }
        let (start, end) = (diagnostic.primary.start, diagnostic.primary.end);
        let (Some(before), Some(written), Some(after)) = (
            source.get(..start),
            source.get(start..end),
            source.get(end..),
        ) else {
            continue;
        };
        let Some(before_colon) = before.trim_end().strip_suffix(':') else {
            continue;
        };
        let attached = before_colon
            .chars()
            .next_back()
            .is_some_and(|character| character.is_alphanumeric() || character == ')');
        if !attached || !after.trim_start().starts_with(')') {
            continue;
        }
        let head = written
            .split(|character: char| {
                !(character.is_alphanumeric() || character == '.' || character == '_')
            })
            .next()
            .unwrap_or_default();
        if !names_a_type(resolved, head) {
            continue;
        }
        diagnostic.message_key = "expression.type-annotation".to_owned();
        diagnostic.type_difference = None;
        for related in &mut diagnostic.related {
            related.message = format!("`{written}` is a type, not a List to cons onto");
        }
    }
}

fn names_a_type(resolved: &ResolvedModule, name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
        && (STANDALONE_TYPE_NAMES.contains(&name)
            || SUM_TYPES.iter().any(|sum_type| sum_type.name == name)
            || resolved
                .symbols
                .iter()
                .any(|symbol| symbol.namespace == SymbolNamespace::Type && symbol.spelling == name))
}
