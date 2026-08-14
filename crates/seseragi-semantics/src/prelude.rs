use crate::{SymbolNamespace, TypedConstraint, TypedType};
use serde::Serialize;
use seseragi_syntax::TypeParameter;

mod surface;

pub use surface::{standard_prelude_surface, StandardModuleSurface};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeTrait {
    pub(crate) name: &'static str,
    pub(crate) canonical: &'static str,
    pub(crate) type_parameter: &'static str,
    pub(crate) type_parameter_arity: u32,
    pub(crate) supertrait: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeTraitMethod {
    pub(crate) trait_name: &'static str,
    pub(crate) name: &'static str,
    pub(crate) canonical: &'static str,
    kind: PreludeTraitMethodKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreludeTraitMethodSignature {
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) parameters: Vec<TypedType>,
    pub(crate) result: TypedType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreludeTraitMethodKind {
    Append,
    Empty,
    Show,
    Debug,
    Map,
    Pure,
    Apply,
    FlatMap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeStandardInstance {
    pub(crate) trait_name: &'static str,
    pub(crate) type_name: &'static str,
    pub(crate) type_canonical: Option<&'static str>,
    pub(crate) type_arity: u32,
    pub(crate) identity: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeStandardInstanceConstraint {
    pub(crate) trait_name: &'static str,
    pub(crate) type_argument_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardEqualityInstance {
    pub type_name: &'static str,
    pub identity: &'static str,
    pub strict_equality_compatible: bool,
}

pub(crate) const TRAITS: &[PreludeTrait] = &[
    PreludeTrait {
        name: "Semigroup",
        canonical: "std/prelude::Semigroup",
        type_parameter: "A",
        type_parameter_arity: 0,
        supertrait: None,
    },
    PreludeTrait {
        name: "Monoid",
        canonical: "std/prelude::Monoid",
        type_parameter: "A",
        type_parameter_arity: 0,
        supertrait: Some("Semigroup"),
    },
    PreludeTrait {
        name: "Show",
        canonical: "std/prelude::Show",
        type_parameter: "A",
        type_parameter_arity: 0,
        supertrait: None,
    },
    PreludeTrait {
        name: "Debug",
        canonical: "std/prelude::Debug",
        type_parameter: "A",
        type_parameter_arity: 0,
        supertrait: None,
    },
    PreludeTrait {
        name: "Functor",
        canonical: "std/prelude::Functor",
        type_parameter: "F",
        type_parameter_arity: 1,
        supertrait: None,
    },
    PreludeTrait {
        name: "Applicative",
        canonical: "std/prelude::Applicative",
        type_parameter: "F",
        type_parameter_arity: 1,
        supertrait: Some("Functor"),
    },
    PreludeTrait {
        name: "Monad",
        canonical: "std/prelude::Monad",
        type_parameter: "M",
        type_parameter_arity: 1,
        supertrait: Some("Applicative"),
    },
];

pub(crate) const TRAIT_METHODS: &[PreludeTraitMethod] = &[
    PreludeTraitMethod {
        trait_name: "Semigroup",
        name: "append",
        canonical: "std/prelude::Semigroup::append",
        kind: PreludeTraitMethodKind::Append,
    },
    PreludeTraitMethod {
        trait_name: "Monoid",
        name: "empty",
        canonical: "std/prelude::Monoid::empty",
        kind: PreludeTraitMethodKind::Empty,
    },
    PreludeTraitMethod {
        trait_name: "Show",
        name: "show",
        canonical: "std/prelude::Show::show",
        kind: PreludeTraitMethodKind::Show,
    },
    PreludeTraitMethod {
        trait_name: "Debug",
        name: "debug",
        canonical: "std/prelude::Debug::debug",
        kind: PreludeTraitMethodKind::Debug,
    },
    PreludeTraitMethod {
        trait_name: "Functor",
        name: "map",
        canonical: "std/prelude::Functor::map",
        kind: PreludeTraitMethodKind::Map,
    },
    PreludeTraitMethod {
        trait_name: "Applicative",
        name: "pure",
        canonical: "std/prelude::Applicative::pure",
        kind: PreludeTraitMethodKind::Pure,
    },
    PreludeTraitMethod {
        trait_name: "Applicative",
        name: "apply",
        canonical: "std/prelude::Applicative::apply",
        kind: PreludeTraitMethodKind::Apply,
    },
    PreludeTraitMethod {
        trait_name: "Monad",
        name: "flatMap",
        canonical: "std/prelude::Monad::flatMap",
        kind: PreludeTraitMethodKind::FlatMap,
    },
];

pub(crate) const STANDARD_INSTANCES: &[PreludeStandardInstance] = &[
    PreludeStandardInstance {
        trait_name: "Semigroup",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "std/string::Semigroup",
    },
    PreludeStandardInstance {
        trait_name: "Monoid",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "std/string::Monoid",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Int",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::Int>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Int",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::Int>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Float",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::Float>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Float",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::Float>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Never",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::Never>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Never",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::Never>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::String>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Bool",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::Bool>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Unit",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::Unit>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Char",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::Char>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "ConsoleError",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::ConsoleError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "StdinError",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::StdinError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ConsoleError",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::ConsoleError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "StdinError",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::StdinError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DomError",
        type_canonical: Some("std/web/dom::DomError"),
        type_arity: 0,
        identity: "Show<std/web/dom::DomError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DomError",
        type_canonical: Some("std/web/dom::DomError"),
        type_arity: 0,
        identity: "Debug<std/web/dom::DomError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DomRuntimeError",
        type_canonical: Some("std/web/dom::DomRuntimeError"),
        type_arity: 1,
        identity: "std/web/dom::DomRuntimeError::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DomRuntimeError",
        type_canonical: Some("std/web/dom::DomRuntimeError"),
        type_arity: 1,
        identity: "std/web/dom::DomRuntimeError::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "HtmlBuildError",
        type_canonical: Some("std/web/html::HtmlBuildError"),
        type_arity: 0,
        identity: "Show<std/web/html::HtmlBuildError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "HtmlBuildError",
        type_canonical: Some("std/web/html::HtmlBuildError"),
        type_arity: 0,
        identity: "Debug<std/web/html::HtmlBuildError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "ByteError",
        type_canonical: Some("std/bytes::ByteError"),
        type_arity: 0,
        identity: "Show<std/bytes::ByteError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ByteError",
        type_canonical: Some("std/bytes::ByteError"),
        type_arity: 0,
        identity: "Debug<std/bytes::ByteError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "BytesSliceError",
        type_canonical: Some("std/bytes::BytesSliceError"),
        type_arity: 0,
        identity: "Show<std/bytes::BytesSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "BytesSliceError",
        type_canonical: Some("std/bytes::BytesSliceError"),
        type_arity: 0,
        identity: "Debug<std/bytes::BytesSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Utf8DecodeError",
        type_canonical: Some("std/text::Utf8DecodeError"),
        type_arity: 0,
        identity: "Show<std/text::Utf8DecodeError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Utf8DecodeError",
        type_canonical: Some("std/text::Utf8DecodeError"),
        type_arity: 0,
        identity: "Debug<std/text::Utf8DecodeError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::String>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Bool",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::Bool>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Unit",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::Unit>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Char",
        type_canonical: None,
        type_arity: 0,
        identity: "Debug<std/prelude::Char>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Range",
        type_canonical: None,
        type_arity: 1,
        identity: "std/range::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Range",
        type_canonical: None,
        type_arity: 1,
        identity: "std/range::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Semigroup",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Semigroup",
    },
    PreludeStandardInstance {
        trait_name: "Monoid",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Monoid",
    },
    PreludeStandardInstance {
        trait_name: "Semigroup",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Semigroup",
    },
    PreludeStandardInstance {
        trait_name: "Monoid",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Monoid",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "Effect",
        type_canonical: None,
        type_arity: 3,
        identity: "std/effect::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "Effect",
        type_canonical: None,
        type_arity: 3,
        identity: "std/effect::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "Effect",
        type_canonical: None,
        type_arity: 3,
        identity: "std/effect::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "Signal",
        type_canonical: Some("std/signal::Signal"),
        type_arity: 1,
        identity: "std/signal::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "Signal",
        type_canonical: Some("std/signal::Signal"),
        type_arity: 1,
        identity: "std/signal::Applicative",
    },
];

const STANDARD_EQUALITY_INSTANCES: &[StandardEqualityInstance] = &[
    StandardEqualityInstance {
        type_name: "Int",
        identity: "std/int::Eq",
        strict_equality_compatible: true,
    },
    StandardEqualityInstance {
        type_name: "Bool",
        identity: "std/bool::Eq",
        strict_equality_compatible: true,
    },
    StandardEqualityInstance {
        type_name: "String",
        identity: "std/string::Eq",
        strict_equality_compatible: true,
    },
];

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

const ORDERING_VARIANTS: &[PreludeVariant] = &[
    PreludeVariant {
        name: "Less",
        canonical: "std/prelude::Less",
        payload_parameter: None,
    },
    PreludeVariant {
        name: "Equal",
        canonical: "std/prelude::Equal",
        payload_parameter: None,
    },
    PreludeVariant {
        name: "Greater",
        canonical: "std/prelude::Greater",
        payload_parameter: None,
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
    PreludeSumType {
        name: "Ordering",
        canonical: "std/prelude::Ordering",
        type_parameters: &[],
        variants: ORDERING_VARIANTS,
    },
];

pub(crate) const PURE_FUNCTION_NAMES: &[&str] = &[
    "reduce", "join", "sum", "product", "combine", "any", "all", "forEach", "unfold", "next",
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
                | "Char"
                | "String"
                | "Array"
                | "List"
                | "Range"
                | "Iterator"
                | "Effect"
                | "Task"
                | "Console"
                | "ConsoleError"
                | "Stdin"
                | "StdinError"
        ),
        SymbolNamespace::Value => {
            crate::effect_ops::known_effect_operation_by_surface(spelling).is_some()
                || PURE_FUNCTION_NAMES.contains(&spelling)
        }
        SymbolNamespace::Operator => {
            seseragi_syntax::standard_operator(spelling).is_some()
                || seseragi_syntax::standard_trait_operator(spelling).is_some()
                || matches!(spelling, "<" | "<=" | ">" | ">=")
        }
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
        "Array" | "List" | "Range" | "Iterator" | "Task" => Some(1),
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
            | "std/prelude::Effect"
            | "std/prelude::Iterator"
            | "std/prelude::List"
    )
}

pub(crate) fn trait_by_name(name: &str) -> Option<&'static PreludeTrait> {
    TRAITS.iter().find(|trait_spec| trait_spec.name == name)
}

pub(crate) fn trait_by_canonical(canonical: &str) -> Option<&'static PreludeTrait> {
    TRAITS
        .iter()
        .find(|trait_spec| trait_spec.canonical == canonical)
}

pub(crate) fn trait_methods_named(name: &str) -> Vec<&'static PreludeTraitMethod> {
    TRAIT_METHODS
        .iter()
        .filter(move |method| method.name == name)
        .collect()
}

pub(crate) fn trait_method(
    trait_name: &str,
    method_name: &str,
) -> Option<&'static PreludeTraitMethod> {
    TRAIT_METHODS
        .iter()
        .find(|method| method.trait_name == trait_name && method.name == method_name)
}

pub(crate) fn trait_method_by_canonical(canonical: &str) -> Option<&'static PreludeTraitMethod> {
    TRAIT_METHODS
        .iter()
        .find(|method| method.canonical == canonical)
}

pub(crate) fn trait_method_signature(method: &PreludeTraitMethod) -> PreludeTraitMethodSignature {
    let trait_spec = trait_by_name(method.trait_name).expect("Prelude method trait must exist");
    let constructor = trait_spec.type_parameter;
    let a = named("A");
    let b = named("B");
    let applied_a = applied(constructor, a.clone());
    let applied_b = applied(constructor, b.clone());
    let mut type_parameters = vec![TypeParameter::constructor(constructor, 1)];
    match method.kind {
        PreludeTraitMethodKind::Append => PreludeTraitMethodSignature {
            type_parameters: vec![TypeParameter::value(constructor)],
            parameters: vec![named(constructor), named(constructor)],
            result: named(constructor),
        },
        PreludeTraitMethodKind::Empty => PreludeTraitMethodSignature {
            type_parameters: vec![TypeParameter::value(constructor)],
            parameters: vec![named("Unit")],
            result: named(constructor),
        },
        PreludeTraitMethodKind::Show | PreludeTraitMethodKind::Debug => {
            PreludeTraitMethodSignature {
                type_parameters: vec![TypeParameter::value(constructor)],
                parameters: vec![named(constructor)],
                result: named("String"),
            }
        }
        PreludeTraitMethodKind::Map => {
            type_parameters.extend([TypeParameter::value("A"), TypeParameter::value("B")]);
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![function(a, b.clone()), applied_a],
                result: applied_b,
            }
        }
        PreludeTraitMethodKind::Pure => {
            type_parameters.push(TypeParameter::value("A"));
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![a],
                result: applied_a,
            }
        }
        PreludeTraitMethodKind::Apply => {
            type_parameters.extend([TypeParameter::value("A"), TypeParameter::value("B")]);
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![
                    applied(constructor, function(a.clone(), b.clone())),
                    applied_a,
                ],
                result: applied_b,
            }
        }
        PreludeTraitMethodKind::FlatMap => {
            type_parameters.extend([TypeParameter::value("A"), TypeParameter::value("B")]);
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![function(a, applied_b.clone()), applied_a],
                result: applied_b,
            }
        }
    }
}

pub(crate) fn standard_instance(
    trait_name: &str,
    type_ref: &TypedType,
) -> Option<&'static PreludeStandardInstance> {
    STANDARD_INSTANCES.iter().find(|instance| {
        instance.trait_name == trait_name
            && standard_instance_head(instance, type_ref).is_some_and(|arguments| {
                trait_by_name(trait_name).is_some_and(|trait_spec| {
                    instance.type_arity.checked_sub(arguments)
                        == Some(trait_spec.type_parameter_arity)
                })
            })
    })
}

pub(crate) fn standard_instance_by_identity(
    identity: &str,
) -> Option<&'static PreludeStandardInstance> {
    STANDARD_INSTANCES
        .iter()
        .find(|instance| instance.identity == identity)
}

pub(crate) fn standard_instance_constraints(
    instance: &PreludeStandardInstance,
    type_ref: &TypedType,
) -> Option<Vec<TypedConstraint>> {
    let arguments = match type_ref {
        TypedType::Named { name, arguments }
            if instance.type_canonical.is_none() && name == instance.type_name =>
        {
            arguments
        }
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if instance.type_canonical == Some(canonical.as_str()) => arguments,
        _ => return None,
    };
    let constraints = standard_instance_constraint_specs(instance.identity)
        .iter()
        .map(|constraint| {
            Some(TypedConstraint {
                name: constraint.trait_name.to_owned(),
                arguments: vec![arguments.get(constraint.type_argument_index)?.clone()],
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(constraints)
}

pub(crate) fn standard_instance_constraint_specs(
    identity: &str,
) -> &'static [PreludeStandardInstanceConstraint] {
    const SHOW_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "Show",
            type_argument_index: 0,
        }];
    const DEBUG_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "Debug",
            type_argument_index: 0,
        }];
    const SHOW_EITHER: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Show",
            type_argument_index: 0,
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Show",
            type_argument_index: 1,
        },
    ];
    const DEBUG_EITHER: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Debug",
            type_argument_index: 0,
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Debug",
            type_argument_index: 1,
        },
    ];
    match identity {
        "std/array::Show" | "std/list::Show" | "std/maybe::Show" | "std/range::Show" => {
            SHOW_ELEMENT
        }
        "std/array::Debug" | "std/list::Debug" | "std/maybe::Debug" | "std/range::Debug" => {
            DEBUG_ELEMENT
        }
        "std/either::Show" => SHOW_EITHER,
        "std/either::Debug" => DEBUG_EITHER,
        "std/web/dom::DomRuntimeError::Show" => SHOW_ELEMENT,
        "std/web/dom::DomRuntimeError::Debug" => DEBUG_ELEMENT,
        _ => &[],
    }
}

pub(crate) fn standard_equality_instance(
    type_ref: &TypedType,
) -> Option<&'static StandardEqualityInstance> {
    let TypedType::Named { name, arguments } = type_ref else {
        return None;
    };
    arguments.is_empty().then_some(())?;
    STANDARD_EQUALITY_INSTANCES
        .iter()
        .find(|instance| instance.type_name == name)
}

pub fn standard_equality_instance_by_identity(
    identity: &str,
) -> Option<&'static StandardEqualityInstance> {
    STANDARD_EQUALITY_INSTANCES
        .iter()
        .find(|instance| instance.identity == identity)
}

pub(crate) fn overlapping_standard_instance(
    trait_identity: &str,
    type_ref: &TypedType,
    canonical_type_ref: &str,
) -> Option<&'static PreludeStandardInstance> {
    STANDARD_INSTANCES.iter().find(|instance| {
        standard_instance_head(instance, type_ref).is_some_and(|arguments| {
            trait_by_name(instance.trait_name).is_some_and(|trait_spec| {
                instance.type_arity.checked_sub(arguments) == Some(trait_spec.type_parameter_arity)
            }) || (arguments == instance.type_arity
                && matches!(last_type_argument(type_ref), Some(TypedType::Hole)))
        }) && trait_by_name(instance.trait_name)
            .is_some_and(|trait_spec| trait_spec.canonical == trait_identity)
            && standard_instance_canonical_head(instance, canonical_type_ref)
    })
}

pub(crate) fn structural_display_instance_identity(
    trait_identity: &str,
    type_ref: &TypedType,
) -> Option<&'static str> {
    match (trait_identity, type_ref) {
        ("std/prelude::Show", TypedType::Tuple { .. }) => Some("std/tuple::Show"),
        ("std/prelude::Debug", TypedType::Tuple { .. }) => Some("std/tuple::Debug"),
        ("std/prelude::Show", TypedType::Record { closed: true, .. }) => Some("std/record::Show"),
        ("std/prelude::Debug", TypedType::Record { closed: true, .. }) => Some("std/record::Debug"),
        _ => None,
    }
}

fn standard_instance_canonical_head(
    instance: &PreludeStandardInstance,
    canonical_type_ref: &str,
) -> bool {
    let expected = instance
        .type_canonical
        .map(str::to_owned)
        .unwrap_or_else(|| format!("std/prelude::{}", instance.type_name));
    canonical_type_ref == expected
        || canonical_type_ref
            .strip_prefix(&expected)
            .is_some_and(|arguments| arguments.starts_with('<'))
}

fn standard_instance_head(instance: &PreludeStandardInstance, type_ref: &TypedType) -> Option<u32> {
    match type_ref {
        TypedType::Named { name, arguments }
            if instance.type_canonical.is_none() && instance.type_name == name =>
        {
            (type_constructor_arity(name) == Some(instance.type_arity))
                .then_some(arguments.len() as u32)
        }
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if instance.type_canonical == Some(canonical.as_str()) => Some(arguments.len() as u32),
        _ => None,
    }
}

fn last_type_argument(type_ref: &TypedType) -> Option<&TypedType> {
    match type_ref {
        TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
            arguments.last()
        }
        _ => None,
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn applied(constructor: &str, argument: TypedType) -> TypedType {
    TypedType::Named {
        name: constructor.to_owned(),
        arguments: vec![argument],
    }
}

fn function(parameter: TypedType, result: TypedType) -> TypedType {
    TypedType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
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
        assert_eq!(type_constructor_arity("Char"), Some(0));
        assert_eq!(type_constructor_arity("Maybe"), Some(1));
        assert_eq!(type_constructor_arity("Either"), Some(2));
        assert_eq!(type_constructor_arity("Effect"), Some(3));
        assert_eq!(type_constructor_arity("Iterator"), Some(1));
    }

    #[test]
    fn describes_the_standard_monad_hierarchy_and_methods() {
        assert_eq!(
            trait_by_name("Monad").unwrap().supertrait,
            Some("Applicative")
        );
        assert_eq!(
            trait_by_name("Applicative").unwrap().supertrait,
            Some("Functor")
        );
        let flat_map = trait_method_by_canonical("std/prelude::Monad::flatMap").unwrap();
        let signature = trait_method_signature(flat_map);
        assert_eq!(
            signature.type_parameters[0],
            TypeParameter::constructor("M", 1)
        );
        assert_eq!(signature.parameters.len(), 2);
        assert_eq!(signature.result, applied("M", named("B")));

        assert_eq!(
            trait_by_name("Monoid").unwrap().supertrait,
            Some("Semigroup")
        );
        let empty = trait_method_by_canonical("std/prelude::Monoid::empty").unwrap();
        let empty_signature = trait_method_signature(empty);
        assert_eq!(
            empty_signature.type_parameters,
            vec![TypeParameter::value("A")]
        );
        assert_eq!(empty_signature.parameters, vec![named("Unit")]);
        assert_eq!(empty_signature.result, named("A"));

        let show = trait_method_by_canonical("std/prelude::Show::show").unwrap();
        let show_signature = trait_method_signature(show);
        assert_eq!(
            show_signature.type_parameters,
            vec![TypeParameter::value("A")]
        );
        assert_eq!(show_signature.parameters, vec![named("A")]);
        assert_eq!(show_signature.result, named("String"));

        let debug = trait_method_by_canonical("std/prelude::Debug::debug").unwrap();
        assert_eq!(trait_method_signature(debug), show_signature);
    }

    #[test]
    fn selects_registered_instances_by_remaining_constructor_arity() {
        let maybe = named("Maybe");
        let either_error = applied("Either", named("String"));
        let saturated_either = TypedType::Named {
            name: "Either".to_owned(),
            arguments: vec![named("String"), named("Int")],
        };

        assert_eq!(
            standard_instance("Monad", &maybe).map(|instance| instance.identity),
            Some("std/maybe::Monad")
        );
        assert_eq!(
            standard_instance("Applicative", &either_error).map(|instance| instance.identity),
            Some("std/either::Applicative")
        );
        assert_eq!(
            standard_instance("Functor", &named("Array")).map(|instance| instance.identity),
            Some("std/array::Functor")
        );
        assert_eq!(
            standard_instance("Monad", &named("List")).map(|instance| instance.identity),
            Some("std/list::Monad")
        );
        assert_eq!(
            standard_instance("Show", &named("Bool")).map(|instance| instance.identity),
            Some("Show<std/prelude::Bool>")
        );
        assert_eq!(
            standard_instance("Debug", &named("Char")).map(|instance| instance.identity),
            Some("Debug<std/prelude::Char>")
        );
        assert_eq!(
            standard_instance(
                "Applicative",
                &TypedType::Named {
                    name: "Effect".to_owned(),
                    arguments: vec![named("ConsoleEnvironment"), named("ConsoleError")],
                }
            )
            .map(|instance| instance.identity),
            Some("std/effect::Applicative")
        );
        assert!(standard_instance("Monad", &saturated_either).is_none());
        assert_eq!(
            standard_instance("Monoid", &named("String")).map(|instance| instance.identity),
            Some("std/string::Monoid")
        );
        assert_eq!(
            standard_instance("Semigroup", &applied("Array", named("Int")))
                .map(|instance| instance.identity),
            Some("std/array::Semigroup")
        );
        assert_eq!(
            standard_instance("Monoid", &applied("List", named("String")))
                .map(|instance| instance.identity),
            Some("std/list::Monoid")
        );

        let signal = TypedType::ExternalNamed {
            name: "Signal".to_owned(),
            canonical: "std/signal::Signal".to_owned(),
            arguments: Vec::new(),
        };
        assert_eq!(
            standard_instance("Functor", &signal).map(|instance| instance.identity),
            Some("std/signal::Functor")
        );
        assert_eq!(
            standard_instance("Applicative", &signal).map(|instance| instance.identity),
            Some("std/signal::Applicative")
        );
        assert!(standard_instance("Monad", &signal).is_none());
    }

    #[test]
    fn exposes_the_complete_primitive_show_and_debug_matrix() {
        for type_name in ["Int", "Float", "Bool", "Char", "String", "Unit", "Never"] {
            let type_ref = named(type_name);
            for trait_name in ["Show", "Debug"] {
                let instance = standard_instance(trait_name, &type_ref)
                    .unwrap_or_else(|| panic!("{trait_name}<{type_name}> must be registered"));
                assert_eq!(instance.type_name, type_name);
                assert_eq!(instance.type_arity, 0);
            }
        }
    }

    #[test]
    fn exposes_standard_error_show_and_debug_with_payload_evidence() {
        for (name, canonical) in [
            ("DomError", "std/web/dom::DomError"),
            ("HtmlBuildError", "std/web/html::HtmlBuildError"),
            ("ByteError", "std/bytes::ByteError"),
            ("BytesSliceError", "std/bytes::BytesSliceError"),
            ("Utf8DecodeError", "std/text::Utf8DecodeError"),
        ] {
            let type_ref = TypedType::ExternalNamed {
                name: name.to_owned(),
                canonical: canonical.to_owned(),
                arguments: Vec::new(),
            };
            for trait_name in ["Show", "Debug"] {
                assert!(
                    standard_instance(trait_name, &type_ref).is_some(),
                    "{trait_name}<{canonical}> must be registered"
                );
            }
        }

        let runtime_error = TypedType::ExternalNamed {
            name: "DomRuntimeError".to_owned(),
            canonical: "std/web/dom::DomRuntimeError".to_owned(),
            arguments: vec![named("String")],
        };
        for trait_name in ["Show", "Debug"] {
            let instance = standard_instance(trait_name, &runtime_error)
                .unwrap_or_else(|| panic!("{trait_name}<DomRuntimeError<String>> is missing"));
            let constraints = standard_instance_constraints(instance, &runtime_error)
                .expect("generic standard error must expose its payload constraint");
            assert_eq!(
                constraints,
                vec![TypedConstraint {
                    name: trait_name.to_owned(),
                    arguments: vec![named("String")],
                }]
            );
        }

        for unsupported in [
            TypedType::Named {
                name: "Effect".to_owned(),
                arguments: vec![named("Unit"), named("Never"), named("Unit")],
            },
            TypedType::Function {
                parameter: Box::new(named("Int")),
                result: Box::new(named("Int")),
            },
            TypedType::ExternalNamed {
                name: "Signal".to_owned(),
                canonical: "std/signal::Signal".to_owned(),
                arguments: vec![named("Int")],
            },
        ] {
            assert!(standard_instance("Show", &unsupported).is_none());
            assert!(standard_instance("Debug", &unsupported).is_none());
        }
    }

    #[test]
    fn detects_user_heads_that_overlap_registered_standard_instances() {
        let maybe = named("Maybe");
        let either_string = TypedType::Named {
            name: "Either".to_owned(),
            arguments: vec![named("String"), TypedType::Hole],
        };

        assert_eq!(
            overlapping_standard_instance("std/prelude::Functor", &maybe, "std/prelude::Maybe")
                .map(|instance| instance.identity),
            Some("std/maybe::Functor")
        );
        assert_eq!(
            overlapping_standard_instance(
                "std/prelude::Monad",
                &either_string,
                "std/prelude::Either<std/prelude::String,_>"
            )
            .map(|instance| instance.identity),
            Some("std/either::Monad")
        );
        assert!(
            overlapping_standard_instance("fixture::Functor", &maybe, "std/prelude::Maybe")
                .is_none()
        );
        assert!(
            overlapping_standard_instance("std/prelude::Functor", &maybe, "fixture::Maybe")
                .is_none()
        );
        assert!(overlapping_standard_instance(
            "std/prelude::Functor",
            &TypedType::Named {
                name: "Either".to_owned(),
                arguments: vec![named("String"), named("Int")],
            },
            "std/prelude::Either<std/prelude::String,std/prelude::Int>"
        )
        .is_none());
    }
}
