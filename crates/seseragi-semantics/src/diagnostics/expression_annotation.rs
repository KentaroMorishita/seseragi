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
        let Some(head) = type_notation_head(written) else {
            continue;
        };
        // `(Flag:Flag)` conses onto a constructor that shares its type's
        // name; that is a value, so the type mismatch stays as reported.
        if !names_a_type(resolved, head) || (head == written && names_a_value(resolved, head)) {
            continue;
        }
        diagnostic.message_key = "expression.type-annotation".to_owned();
        diagnostic.type_difference = None;
        for related in &mut diagnostic.related {
            related.message = format!("`{written}` is a type, not a List to cons onto");
        }
    }
}

/// The head name of `written` when all of it is type notation, such as
/// `Maybe<Int>` or `Js.Promise<Array<A>>`; a value such as `Point { x: 2 }`
/// or `Just 1` is not.
fn type_notation_head(written: &str) -> Option<&str> {
    let is_name = |character: char| character.is_alphanumeric() || character == '_';
    let head_end = written
        .find(|character: char| !(is_name(character) || character == '.'))
        .unwrap_or(written.len());
    let (head, rest) = written.split_at(head_end);
    let arguments = match rest {
        "" => "",
        rest => rest.strip_prefix('<')?.strip_suffix('>')?,
    };
    arguments
        .chars()
        .all(|character| {
            is_name(character) || matches!(character, '.' | ',' | ' ' | '<' | '>' | '(' | ')')
        })
        .then_some(head)
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

fn names_a_value(resolved: &ResolvedModule, name: &str) -> bool {
    SUM_TYPES
        .iter()
        .any(|sum_type| sum_type.variants.iter().any(|variant| variant.name == name))
        || resolved
            .symbols
            .iter()
            .any(|symbol| symbol.namespace == SymbolNamespace::Value && symbol.spelling == name)
}

#[cfg(test)]
mod tests {
    use super::type_notation_head;

    #[test]
    fn reads_only_type_notation_as_a_type() {
        assert_eq!(type_notation_head("Maybe<Int>"), Some("Maybe"));
        assert_eq!(
            type_notation_head("Js.Promise<Array<A>>"),
            Some("Js.Promise")
        );
        assert_eq!(type_notation_head("Int"), Some("Int"));
        assert_eq!(type_notation_head("Point { x: 2 }"), None);
        assert_eq!(type_notation_head("Just 1"), None);
        assert_eq!(type_notation_head("Pair<Int>(1)"), None);
    }
}
