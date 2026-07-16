use crate::SymbolNamespace;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeSumType {
    pub(crate) name: &'static str,
    pub(crate) canonical: &'static str,
    pub(crate) type_parameters: &'static [&'static str],
    pub(crate) variants: &'static [PreludeVariant],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeVariant {
    pub(crate) name: &'static str,
    pub(crate) canonical: &'static str,
    pub(crate) payload_parameter: Option<usize>,
}

const MAYBE_VARIANTS: &[PreludeVariant] = &[
    PreludeVariant {
        name: "Nothing",
        canonical: "std/prelude::Nothing",
        payload_parameter: None,
    },
    PreludeVariant {
        name: "Just",
        canonical: "std/prelude::Just",
        payload_parameter: Some(0),
    },
];

const EITHER_VARIANTS: &[PreludeVariant] = &[
    PreludeVariant {
        name: "Left",
        canonical: "std/prelude::Left",
        payload_parameter: Some(0),
    },
    PreludeVariant {
        name: "Right",
        canonical: "std/prelude::Right",
        payload_parameter: Some(1),
    },
];

pub(crate) const SUM_TYPES: &[PreludeSumType] = &[
    PreludeSumType {
        name: "Maybe",
        canonical: "std/prelude::Maybe",
        type_parameters: &["A"],
        variants: MAYBE_VARIANTS,
    },
    PreludeSumType {
        name: "Either",
        canonical: "std/prelude::Either",
        type_parameters: &["E", "A"],
        variants: EITHER_VARIANTS,
    },
];

pub(crate) fn sum_type_for_symbol(
    namespace: SymbolNamespace,
    spelling: &str,
) -> Option<&'static PreludeSumType> {
    SUM_TYPES.iter().find(|sum_type| match namespace {
        SymbolNamespace::Type => sum_type.name == spelling,
        SymbolNamespace::Value => sum_type
            .variants
            .iter()
            .any(|variant| variant.name == spelling),
        _ => false,
    })
}

pub(crate) fn is_standalone_symbol(namespace: SymbolNamespace, spelling: &str) -> bool {
    match namespace {
        SymbolNamespace::Type => matches!(
            spelling,
            "Unit"
                | "Never"
                | "Bool"
                | "Int"
                | "Float"
                | "String"
                | "Array"
                | "List"
                | "Range"
                | "Iterator"
                | "Effect"
                | "Console"
                | "ConsoleError"
                | "Stdin"
                | "StdinError"
        ),
        SymbolNamespace::Value => matches!(
            spelling,
            "print"
                | "println"
                | "readLine"
                | "succeed"
                | "fail"
                | "mapError"
                | "fromEither"
                | "reduce"
                | "unfold"
                | "next"
        ),
        SymbolNamespace::Operator => matches!(
            spelling,
            "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">="
        ),
        SymbolNamespace::Trait => matches!(
            spelling,
            "Eq" | "Ord"
                | "Hash"
                | "Show"
                | "Debug"
                | "Zero"
                | "One"
                | "Semigroup"
                | "Monoid"
                | "JsonEncode"
                | "JsonDecode"
                | "Functor"
                | "Applicative"
                | "Monad"
                | "Iterable"
                | "Reducible"
                | "Traversable"
                | "Add"
                | "Sub"
                | "Mul"
                | "Div"
                | "Rem"
                | "Pow"
        ),
        _ => false,
    }
}

pub(crate) fn type_constructor_arity(spelling: &str) -> Option<u32> {
    if let Some(sum_type) = sum_type_for_symbol(SymbolNamespace::Type, spelling) {
        return Some(sum_type.type_parameters.len() as u32);
    }
    match spelling {
        "Array" | "List" | "Range" | "Iterator" => Some(1),
        "Effect" => Some(3),
        name if is_standalone_symbol(SymbolNamespace::Type, name) => Some(0),
        _ => None,
    }
}

pub(crate) fn is_external_nominal_type(canonical: &str) -> bool {
    matches!(
        canonical,
        "std/prelude::Console"
            | "std/prelude::ConsoleError"
            | "std/prelude::Stdin"
            | "std/prelude::StdinError"
            | "std/prelude::Iterator"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_sum_types_from_their_type_or_constructor_names() {
        assert_eq!(
            sum_type_for_symbol(SymbolNamespace::Type, "Maybe").map(|sum_type| sum_type.name),
            Some("Maybe")
        );
        assert_eq!(
            sum_type_for_symbol(SymbolNamespace::Value, "Right").map(|sum_type| sum_type.name),
            Some("Either")
        );
        assert!(sum_type_for_symbol(SymbolNamespace::Value, "println").is_none());
    }

    #[test]
    fn records_prelude_type_constructor_arities() {
        assert_eq!(type_constructor_arity("Int"), Some(0));
        assert_eq!(type_constructor_arity("Maybe"), Some(1));
        assert_eq!(type_constructor_arity("Either"), Some(2));
        assert_eq!(type_constructor_arity("Effect"), Some(3));
        assert_eq!(type_constructor_arity("Iterator"), Some(1));
    }
}
