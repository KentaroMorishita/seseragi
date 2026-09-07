use crate::{SymbolNamespace, TypedConstraint, TypedType};
use serde::Serialize;
use seseragi_syntax::TypeParameter;

mod surface;

pub use surface::{standard_prelude_surface, StandardModuleSurface};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeTrait {
    pub(crate) name: &'static str,
    pub(crate) canonical: &'static str,
    pub(crate) type_parameters: &'static [PreludeTypeParameter],
    pub(crate) supertrait: Option<&'static str>,
    pub(crate) deriving: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeTypeParameter {
    pub(crate) name: &'static str,
    pub(crate) arity: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreludeTraitMethod {
    pub(crate) trait_name: &'static str,
    pub(crate) name: &'static str,
    pub(crate) canonical: &'static str,
    pub(crate) operators: &'static [&'static str],
    kind: PreludeTraitMethodKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreludeTraitMethodSignature {
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) parameters: Vec<TypedType>,
    pub(crate) result: TypedType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) constraints: Vec<TypedConstraint>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreludeTraitMethodKind {
    Eq,
    Compare,
    Hash,
    Append,
    Empty,
    Show,
    Debug,
    Zero,
    One,
    Map,
    Pure,
    Apply,
    FlatMap,
    Iterate,
    Reduce,
    Traverse,
    Arithmetic,
    EncodeJson,
    DecodeJson,
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
    pub(crate) type_argument_indices: &'static [usize],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreludeSpecialInstance {
    pub type_name: &'static str,
    pub identity: &'static str,
    pub strict_equality_compatible: bool,
    pub trait_name: &'static str,
    pub arguments: &'static [&'static str],
    pub dispatch: PreludeSpecialInstanceDispatch,
    head: PreludeSpecialInstanceHead,
}

pub type StandardEqualityInstance = PreludeSpecialInstance;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreludeSpecialInstanceDispatch {
    Dictionary,
    OperatorAbi,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreludeSpecialInstanceHead {
    Value(&'static str),
    Homogeneous3(&'static str),
    ExternalArithmetic3 {
        type_name: &'static str,
        canonical: &'static str,
        int_exponent: bool,
    },
    Collection {
        constructor: &'static str,
        canonical: Option<&'static str>,
        int_element: bool,
        tuple_elements: bool,
    },
}

pub(crate) const TRAITS: &[PreludeTrait] = &[
    value_trait("Eq", "std/prelude::Eq", None, true),
    value_trait("Ord", "std/prelude::Ord", Some("Eq"), true),
    value_trait("Hash", "std/prelude::Hash", None, true),
    PreludeTrait {
        name: "Semigroup",
        canonical: "std/prelude::Semigroup",
        type_parameters: &[PreludeTypeParameter {
            name: "A",
            arity: 0,
        }],
        supertrait: None,
        deriving: false,
    },
    PreludeTrait {
        name: "Monoid",
        canonical: "std/prelude::Monoid",
        type_parameters: &[PreludeTypeParameter {
            name: "A",
            arity: 0,
        }],
        supertrait: Some("Semigroup"),
        deriving: false,
    },
    value_trait("Show", "std/prelude::Show", None, true),
    value_trait("Debug", "std/prelude::Debug", None, true),
    value_trait("Zero", "std/prelude::Zero", None, false),
    value_trait("One", "std/prelude::One", None, false),
    PreludeTrait {
        name: "Functor",
        canonical: "std/prelude::Functor",
        type_parameters: &[PreludeTypeParameter {
            name: "F",
            arity: 1,
        }],
        supertrait: None,
        deriving: false,
    },
    PreludeTrait {
        name: "Applicative",
        canonical: "std/prelude::Applicative",
        type_parameters: &[PreludeTypeParameter {
            name: "F",
            arity: 1,
        }],
        supertrait: Some("Functor"),
        deriving: false,
    },
    PreludeTrait {
        name: "Monad",
        canonical: "std/prelude::Monad",
        type_parameters: &[PreludeTypeParameter {
            name: "M",
            arity: 1,
        }],
        supertrait: Some("Applicative"),
        deriving: false,
    },
    value_trait("JsonEncode", "std/prelude::JsonEncode", None, true),
    value_trait("JsonDecode", "std/prelude::JsonDecode", None, true),
    PreludeTrait {
        name: "Iterable",
        canonical: "std/prelude::Iterable",
        type_parameters: &[
            PreludeTypeParameter {
                name: "C",
                arity: 0,
            },
            PreludeTypeParameter {
                name: "A",
                arity: 0,
            },
        ],
        supertrait: None,
        deriving: false,
    },
    PreludeTrait {
        name: "Reducible",
        canonical: "std/prelude::Reducible",
        type_parameters: &[
            PreludeTypeParameter {
                name: "C",
                arity: 0,
            },
            PreludeTypeParameter {
                name: "A",
                arity: 0,
            },
        ],
        supertrait: Some("Iterable"),
        deriving: false,
    },
    PreludeTrait {
        name: "Traversable",
        canonical: "std/prelude::Traversable",
        type_parameters: &[PreludeTypeParameter {
            name: "F",
            arity: 1,
        }],
        supertrait: Some("Functor"),
        deriving: false,
    },
    arithmetic_trait("Add", "std/prelude::Add"),
    arithmetic_trait("Sub", "std/prelude::Sub"),
    arithmetic_trait("Mul", "std/prelude::Mul"),
    arithmetic_trait("Div", "std/prelude::Div"),
    arithmetic_trait("Rem", "std/prelude::Rem"),
    arithmetic_trait("Pow", "std/prelude::Pow"),
];

const fn value_trait(
    name: &'static str,
    canonical: &'static str,
    supertrait: Option<&'static str>,
    deriving: bool,
) -> PreludeTrait {
    PreludeTrait {
        name,
        canonical,
        type_parameters: &[PreludeTypeParameter {
            name: "A",
            arity: 0,
        }],
        supertrait,
        deriving,
    }
}

const fn arithmetic_trait(name: &'static str, canonical: &'static str) -> PreludeTrait {
    PreludeTrait {
        name,
        canonical,
        type_parameters: &[
            PreludeTypeParameter {
                name: "L",
                arity: 0,
            },
            PreludeTypeParameter {
                name: "R",
                arity: 0,
            },
            PreludeTypeParameter {
                name: "O",
                arity: 0,
            },
        ],
        supertrait: None,
        deriving: false,
    }
}

pub(crate) const TRAIT_METHODS: &[PreludeTraitMethod] = &[
    method(
        "Eq",
        "eq",
        "std/prelude::Eq::eq",
        PreludeTraitMethodKind::Eq,
        &["==", "!="],
    ),
    method(
        "Ord",
        "compare",
        "std/prelude::Ord::compare",
        PreludeTraitMethodKind::Compare,
        &["<", "<=", ">", ">="],
    ),
    method(
        "Hash",
        "hash",
        "std/prelude::Hash::hash",
        PreludeTraitMethodKind::Hash,
        &[],
    ),
    PreludeTraitMethod {
        trait_name: "Semigroup",
        name: "append",
        canonical: "std/prelude::Semigroup::append",
        operators: &[],
        kind: PreludeTraitMethodKind::Append,
    },
    PreludeTraitMethod {
        trait_name: "Monoid",
        name: "empty",
        canonical: "std/prelude::Monoid::empty",
        operators: &[],
        kind: PreludeTraitMethodKind::Empty,
    },
    PreludeTraitMethod {
        trait_name: "Show",
        name: "show",
        canonical: "std/prelude::Show::show",
        operators: &[],
        kind: PreludeTraitMethodKind::Show,
    },
    PreludeTraitMethod {
        trait_name: "Debug",
        name: "debug",
        canonical: "std/prelude::Debug::debug",
        operators: &[],
        kind: PreludeTraitMethodKind::Debug,
    },
    method(
        "Zero",
        "zero",
        "std/prelude::Zero::zero",
        PreludeTraitMethodKind::Zero,
        &[],
    ),
    method(
        "One",
        "one",
        "std/prelude::One::one",
        PreludeTraitMethodKind::One,
        &[],
    ),
    PreludeTraitMethod {
        trait_name: "Functor",
        name: "map",
        canonical: "std/prelude::Functor::map",
        operators: &["<$>"],
        kind: PreludeTraitMethodKind::Map,
    },
    PreludeTraitMethod {
        trait_name: "Applicative",
        name: "pure",
        canonical: "std/prelude::Applicative::pure",
        operators: &[],
        kind: PreludeTraitMethodKind::Pure,
    },
    PreludeTraitMethod {
        trait_name: "Applicative",
        name: "apply",
        canonical: "std/prelude::Applicative::apply",
        operators: &["<*>"],
        kind: PreludeTraitMethodKind::Apply,
    },
    PreludeTraitMethod {
        trait_name: "Monad",
        name: "flatMap",
        canonical: "std/prelude::Monad::flatMap",
        operators: &[">>="],
        kind: PreludeTraitMethodKind::FlatMap,
    },
    PreludeTraitMethod {
        trait_name: "JsonEncode",
        name: "encodeJson",
        canonical: "std/prelude::JsonEncode::encodeJson",
        operators: &[],
        kind: PreludeTraitMethodKind::EncodeJson,
    },
    PreludeTraitMethod {
        trait_name: "JsonDecode",
        name: "decodeJson",
        canonical: "std/prelude::JsonDecode::decodeJson",
        operators: &[],
        kind: PreludeTraitMethodKind::DecodeJson,
    },
    method(
        "Iterable",
        "iterate",
        "std/prelude::Iterable::iterate",
        PreludeTraitMethodKind::Iterate,
        &[],
    ),
    method(
        "Reducible",
        "reduce",
        "std/prelude::Reducible::reduce",
        PreludeTraitMethodKind::Reduce,
        &[],
    ),
    method(
        "Traversable",
        "traverse",
        "std/prelude::Traversable::traverse",
        PreludeTraitMethodKind::Traverse,
        &[],
    ),
    method(
        "Add",
        "add",
        "std/prelude::Add::add",
        PreludeTraitMethodKind::Arithmetic,
        &["+"],
    ),
    method(
        "Sub",
        "sub",
        "std/prelude::Sub::sub",
        PreludeTraitMethodKind::Arithmetic,
        &["-"],
    ),
    method(
        "Mul",
        "mul",
        "std/prelude::Mul::mul",
        PreludeTraitMethodKind::Arithmetic,
        &["*"],
    ),
    method(
        "Div",
        "div",
        "std/prelude::Div::div",
        PreludeTraitMethodKind::Arithmetic,
        &["/"],
    ),
    method(
        "Rem",
        "rem",
        "std/prelude::Rem::rem",
        PreludeTraitMethodKind::Arithmetic,
        &["%"],
    ),
    method(
        "Pow",
        "pow",
        "std/prelude::Pow::pow",
        PreludeTraitMethodKind::Arithmetic,
        &["**"],
    ),
];

const fn method(
    trait_name: &'static str,
    name: &'static str,
    canonical: &'static str,
    kind: PreludeTraitMethodKind,
    operators: &'static [&'static str],
) -> PreludeTraitMethod {
    PreludeTraitMethod {
        trait_name,
        name,
        canonical,
        operators,
        kind,
    }
}

pub(crate) const STANDARD_INSTANCES: &[PreludeStandardInstance] = &[
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "MaybeT",
        type_canonical: Some("std/transformer/maybe::MaybeT"),
        type_arity: 2,
        identity: "std/transformer/maybe::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "MaybeT",
        type_canonical: Some("std/transformer/maybe::MaybeT"),
        type_arity: 2,
        identity: "std/transformer/maybe::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "MaybeT",
        type_canonical: Some("std/transformer/maybe::MaybeT"),
        type_arity: 2,
        identity: "std/transformer/maybe::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "EitherT",
        type_canonical: Some("std/transformer/either::EitherT"),
        type_arity: 3,
        identity: "std/transformer/either::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "EitherT",
        type_canonical: Some("std/transformer/either::EitherT"),
        type_arity: 3,
        identity: "std/transformer/either::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "EitherT",
        type_canonical: Some("std/transformer/either::EitherT"),
        type_arity: 3,
        identity: "std/transformer/either::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "ReaderT",
        type_canonical: Some("std/transformer/reader::ReaderT"),
        type_arity: 3,
        identity: "std/transformer/reader::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "ReaderT",
        type_canonical: Some("std/transformer/reader::ReaderT"),
        type_arity: 3,
        identity: "std/transformer/reader::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "ReaderT",
        type_canonical: Some("std/transformer/reader::ReaderT"),
        type_arity: 3,
        identity: "std/transformer/reader::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "StateT",
        type_canonical: Some("std/transformer/state::StateT"),
        type_arity: 3,
        identity: "std/transformer/state::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "StateT",
        type_canonical: Some("std/transformer/state::StateT"),
        type_arity: 3,
        identity: "std/transformer/state::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "StateT",
        type_canonical: Some("std/transformer/state::StateT"),
        type_arity: 3,
        identity: "std/transformer/state::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "WriterT",
        type_canonical: Some("std/transformer/writer::WriterT"),
        type_arity: 3,
        identity: "std/transformer/writer::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "WriterT",
        type_canonical: Some("std/transformer/writer::WriterT"),
        type_arity: 3,
        identity: "std/transformer/writer::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "WriterT",
        type_canonical: Some("std/transformer/writer::WriterT"),
        type_arity: 3,
        identity: "std/transformer/writer::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Monoid",
        type_name: "Product",
        type_canonical: None,
        type_arity: 1,
        identity: "std/product::Monoid",
    },
    PreludeStandardInstance {
        trait_name: "Semigroup",
        type_name: "Product",
        type_canonical: None,
        type_arity: 1,
        identity: "std/product::Semigroup",
    },
    PreludeStandardInstance {
        trait_name: "Monoid",
        type_name: "Sum",
        type_canonical: None,
        type_arity: 1,
        identity: "std/sum::Monoid",
    },
    PreludeStandardInstance {
        trait_name: "Semigroup",
        type_name: "Sum",
        type_canonical: None,
        type_arity: 1,
        identity: "std/sum::Semigroup",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "std/decimal::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "std/decimal::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "std/decimal::Hash",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "Show<std/decimal::Decimal>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "Debug<std/decimal::Decimal>",
    },
    PreludeStandardInstance {
        trait_name: "Zero",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "std/decimal::Zero",
    },
    PreludeStandardInstance {
        trait_name: "One",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "std/decimal::One",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "std/decimal::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Decimal",
        type_canonical: Some("std/decimal::Decimal"),
        type_arity: 0,
        identity: "std/decimal::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "DecimalParseError",
        type_canonical: Some("std/decimal::DecimalParseError"),
        type_arity: 0,
        identity: "Eq<std/decimal::DecimalParseError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DecimalParseError",
        type_canonical: Some("std/decimal::DecimalParseError"),
        type_arity: 0,
        identity: "Show<std/decimal::DecimalParseError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DecimalParseError",
        type_canonical: Some("std/decimal::DecimalParseError"),
        type_arity: 0,
        identity: "Debug<std/decimal::DecimalParseError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "DecimalContextError",
        type_canonical: Some("std/decimal::DecimalContextError"),
        type_arity: 0,
        identity: "Eq<std/decimal::DecimalContextError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DecimalContextError",
        type_canonical: Some("std/decimal::DecimalContextError"),
        type_arity: 0,
        identity: "Show<std/decimal::DecimalContextError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DecimalContextError",
        type_canonical: Some("std/decimal::DecimalContextError"),
        type_arity: 0,
        identity: "Debug<std/decimal::DecimalContextError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "DecimalArithmeticError",
        type_canonical: Some("std/decimal::DecimalArithmeticError"),
        type_arity: 0,
        identity: "Eq<std/decimal::DecimalArithmeticError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DecimalArithmeticError",
        type_canonical: Some("std/decimal::DecimalArithmeticError"),
        type_arity: 0,
        identity: "Show<std/decimal::DecimalArithmeticError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DecimalArithmeticError",
        type_canonical: Some("std/decimal::DecimalArithmeticError"),
        type_arity: 0,
        identity: "Debug<std/decimal::DecimalArithmeticError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "DecimalConversionError",
        type_canonical: Some("std/decimal::DecimalConversionError"),
        type_arity: 0,
        identity: "Eq<std/decimal::DecimalConversionError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DecimalConversionError",
        type_canonical: Some("std/decimal::DecimalConversionError"),
        type_arity: 0,
        identity: "Show<std/decimal::DecimalConversionError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DecimalConversionError",
        type_canonical: Some("std/decimal::DecimalConversionError"),
        type_arity: 0,
        identity: "Debug<std/decimal::DecimalConversionError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "BigInt",
        type_canonical: Some("std/big-int::BigInt"),
        type_arity: 0,
        identity: "std/big-int::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "BigInt",
        type_canonical: Some("std/big-int::BigInt"),
        type_arity: 0,
        identity: "std/big-int::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "BigInt",
        type_canonical: Some("std/big-int::BigInt"),
        type_arity: 0,
        identity: "std/big-int::Hash",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "BigInt",
        type_canonical: Some("std/big-int::BigInt"),
        type_arity: 0,
        identity: "Show<std/big-int::BigInt>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "BigInt",
        type_canonical: Some("std/big-int::BigInt"),
        type_arity: 0,
        identity: "Debug<std/big-int::BigInt>",
    },
    PreludeStandardInstance {
        trait_name: "Zero",
        type_name: "BigInt",
        type_canonical: Some("std/big-int::BigInt"),
        type_arity: 0,
        identity: "std/big-int::Zero",
    },
    PreludeStandardInstance {
        trait_name: "One",
        type_name: "BigInt",
        type_canonical: Some("std/big-int::BigInt"),
        type_arity: 0,
        identity: "std/big-int::One",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "BigIntParseError",
        type_canonical: Some("std/big-int::BigIntParseError"),
        type_arity: 0,
        identity: "Eq<std/big-int::BigIntParseError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "BigIntParseError",
        type_canonical: Some("std/big-int::BigIntParseError"),
        type_arity: 0,
        identity: "Show<std/big-int::BigIntParseError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "BigIntParseError",
        type_canonical: Some("std/big-int::BigIntParseError"),
        type_arity: 0,
        identity: "Debug<std/big-int::BigIntParseError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "BigIntDivisionError",
        type_canonical: Some("std/big-int::BigIntDivisionError"),
        type_arity: 0,
        identity: "Eq<std/big-int::BigIntDivisionError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "BigIntDivisionError",
        type_canonical: Some("std/big-int::BigIntDivisionError"),
        type_arity: 0,
        identity: "Show<std/big-int::BigIntDivisionError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "BigIntDivisionError",
        type_canonical: Some("std/big-int::BigIntDivisionError"),
        type_arity: 0,
        identity: "Debug<std/big-int::BigIntDivisionError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "BigIntPowerError",
        type_canonical: Some("std/big-int::BigIntPowerError"),
        type_arity: 0,
        identity: "Eq<std/big-int::BigIntPowerError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "BigIntPowerError",
        type_canonical: Some("std/big-int::BigIntPowerError"),
        type_arity: 0,
        identity: "Show<std/big-int::BigIntPowerError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "BigIntPowerError",
        type_canonical: Some("std/big-int::BigIntPowerError"),
        type_arity: 0,
        identity: "Debug<std/big-int::BigIntPowerError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "BigIntConversionError",
        type_canonical: Some("std/big-int::BigIntConversionError"),
        type_arity: 0,
        identity: "Eq<std/big-int::BigIntConversionError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "BigIntConversionError",
        type_canonical: Some("std/big-int::BigIntConversionError"),
        type_arity: 0,
        identity: "Show<std/big-int::BigIntConversionError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "BigIntConversionError",
        type_canonical: Some("std/big-int::BigIntConversionError"),
        type_arity: 0,
        identity: "Debug<std/big-int::BigIntConversionError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "TextSliceError",
        type_canonical: Some("std/text::TextSliceError"),
        type_arity: 0,
        identity: "Eq<std/text::TextSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "TextSliceError",
        type_canonical: Some("std/text::TextSliceError"),
        type_arity: 0,
        identity: "Show<std/text::TextSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "TextSliceError",
        type_canonical: Some("std/text::TextSliceError"),
        type_arity: 0,
        identity: "Debug<std/text::TextSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "GraphemeSliceError",
        type_canonical: Some("std/text/grapheme::GraphemeSliceError"),
        type_arity: 0,
        identity: "Eq<std/text/grapheme::GraphemeSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "GraphemeSliceError",
        type_canonical: Some("std/text/grapheme::GraphemeSliceError"),
        type_arity: 0,
        identity: "Show<std/text/grapheme::GraphemeSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "GraphemeSliceError",
        type_canonical: Some("std/text/grapheme::GraphemeSliceError"),
        type_arity: 0,
        identity: "Debug<std/text/grapheme::GraphemeSliceError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "NormalizationForm",
        type_canonical: Some("std/text/unicode::NormalizationForm"),
        type_arity: 0,
        identity: "Eq<std/text/unicode::NormalizationForm>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "NormalizationForm",
        type_canonical: Some("std/text/unicode::NormalizationForm"),
        type_arity: 0,
        identity: "Show<std/text/unicode::NormalizationForm>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "NormalizationForm",
        type_canonical: Some("std/text/unicode::NormalizationForm"),
        type_arity: 0,
        identity: "Debug<std/text/unicode::NormalizationForm>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "UnicodeGeneralCategory",
        type_canonical: Some("std/text/unicode::UnicodeGeneralCategory"),
        type_arity: 0,
        identity: "Eq<std/text/unicode::UnicodeGeneralCategory>",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "UnicodeGeneralCategory",
        type_canonical: Some("std/text/unicode::UnicodeGeneralCategory"),
        type_arity: 0,
        identity: "Ord<std/text/unicode::UnicodeGeneralCategory>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "UnicodeGeneralCategory",
        type_canonical: Some("std/text/unicode::UnicodeGeneralCategory"),
        type_arity: 0,
        identity: "Show<std/text/unicode::UnicodeGeneralCategory>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "UnicodeGeneralCategory",
        type_canonical: Some("std/text/unicode::UnicodeGeneralCategory"),
        type_arity: 0,
        identity: "Debug<std/text/unicode::UnicodeGeneralCategory>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "RegexCompileErrorKind",
        type_canonical: Some("std/regex::RegexCompileErrorKind"),
        type_arity: 0,
        identity: "Eq<std/regex::RegexCompileErrorKind>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RegexCompileErrorKind",
        type_canonical: Some("std/regex::RegexCompileErrorKind"),
        type_arity: 0,
        identity: "Show<std/regex::RegexCompileErrorKind>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RegexCompileErrorKind",
        type_canonical: Some("std/regex::RegexCompileErrorKind"),
        type_arity: 0,
        identity: "Debug<std/regex::RegexCompileErrorKind>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "RegexCompileError",
        type_canonical: Some("std/regex::RegexCompileError"),
        type_arity: 0,
        identity: "Eq<std/regex::RegexCompileError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RegexCompileError",
        type_canonical: Some("std/regex::RegexCompileError"),
        type_arity: 0,
        identity: "Show<std/regex::RegexCompileError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RegexCompileError",
        type_canonical: Some("std/regex::RegexCompileError"),
        type_arity: 0,
        identity: "Debug<std/regex::RegexCompileError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "RegexOptions",
        type_canonical: Some("std/regex::RegexOptions"),
        type_arity: 0,
        identity: "Eq<std/regex::RegexOptions>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RegexOptions",
        type_canonical: Some("std/regex::RegexOptions"),
        type_arity: 0,
        identity: "Show<std/regex::RegexOptions>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RegexOptions",
        type_canonical: Some("std/regex::RegexOptions"),
        type_arity: 0,
        identity: "Debug<std/regex::RegexOptions>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "RegexSpan",
        type_canonical: Some("std/regex::RegexSpan"),
        type_arity: 0,
        identity: "Eq<std/regex::RegexSpan>",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "RegexSpan",
        type_canonical: Some("std/regex::RegexSpan"),
        type_arity: 0,
        identity: "Ord<std/regex::RegexSpan>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RegexSpan",
        type_canonical: Some("std/regex::RegexSpan"),
        type_arity: 0,
        identity: "Show<std/regex::RegexSpan>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RegexSpan",
        type_canonical: Some("std/regex::RegexSpan"),
        type_arity: 0,
        identity: "Debug<std/regex::RegexSpan>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "RegexCapture",
        type_canonical: Some("std/regex::RegexCapture"),
        type_arity: 0,
        identity: "Eq<std/regex::RegexCapture>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RegexCapture",
        type_canonical: Some("std/regex::RegexCapture"),
        type_arity: 0,
        identity: "Show<std/regex::RegexCapture>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RegexCapture",
        type_canonical: Some("std/regex::RegexCapture"),
        type_arity: 0,
        identity: "Debug<std/regex::RegexCapture>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "RegexMatch",
        type_canonical: Some("std/regex::RegexMatch"),
        type_arity: 0,
        identity: "Eq<std/regex::RegexMatch>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RegexMatch",
        type_canonical: Some("std/regex::RegexMatch"),
        type_arity: 0,
        identity: "Show<std/regex::RegexMatch>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RegexMatch",
        type_canonical: Some("std/regex::RegexMatch"),
        type_arity: 0,
        identity: "Debug<std/regex::RegexMatch>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Validation",
        type_canonical: Some("std/validation::Validation"),
        type_arity: 2,
        identity: "std/validation::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Validation",
        type_canonical: Some("std/validation::Validation"),
        type_arity: 2,
        identity: "std/validation::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Validation",
        type_canonical: Some("std/validation::Validation"),
        type_arity: 2,
        identity: "std/validation::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "Validation",
        type_canonical: Some("std/validation::Validation"),
        type_arity: 2,
        identity: "std/validation::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "Validation",
        type_canonical: Some("std/validation::Validation"),
        type_arity: 2,
        identity: "std/validation::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "Int",
        type_canonical: None,
        type_arity: 0,
        identity: "std/int::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "Bool",
        type_canonical: None,
        type_arity: 0,
        identity: "std/bool::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "Char",
        type_canonical: None,
        type_arity: 0,
        identity: "std/char::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "std/string::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "Unit",
        type_canonical: None,
        type_arity: 0,
        identity: "std/unit::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "SizeError",
        type_canonical: Some("std/collection::SizeError"),
        type_arity: 0,
        identity: "std/collection::EqSizeError",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "SizeError",
        type_canonical: Some("std/collection::SizeError"),
        type_arity: 0,
        identity: "Show<std/collection::SizeError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "SizeError",
        type_canonical: Some("std/collection::SizeError"),
        type_arity: 0,
        identity: "Debug<std/collection::SizeError>",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Map",
        type_canonical: Some("std/map::Map"),
        type_arity: 2,
        identity: "std/map::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Map",
        type_canonical: Some("std/map::Map"),
        type_arity: 2,
        identity: "std/map::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Set",
        type_canonical: Some("std/set::Set"),
        type_arity: 1,
        identity: "std/set::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Set",
        type_canonical: Some("std/set::Set"),
        type_arity: 1,
        identity: "std/set::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Map",
        type_canonical: Some("std/map::Map"),
        type_arity: 2,
        identity: "std/map::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Map",
        type_canonical: Some("std/map::Map"),
        type_arity: 2,
        identity: "std/map::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Map",
        type_canonical: Some("std/map::Map"),
        type_arity: 2,
        identity: "std/map::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "Map",
        type_canonical: Some("std/map::Map"),
        type_arity: 2,
        identity: "std/map::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Set",
        type_canonical: Some("std/set::Set"),
        type_arity: 1,
        identity: "std/set::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Set",
        type_canonical: Some("std/set::Set"),
        type_arity: 1,
        identity: "std/set::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Set",
        type_canonical: Some("std/set::Set"),
        type_arity: 1,
        identity: "std/set::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "Int",
        type_canonical: None,
        type_arity: 0,
        identity: "std/int::Hash",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "Bool",
        type_canonical: None,
        type_arity: 0,
        identity: "std/bool::Hash",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "Char",
        type_canonical: None,
        type_arity: 0,
        identity: "std/char::Hash",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "std/string::Hash",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "Unit",
        type_canonical: None,
        type_arity: 0,
        identity: "std/unit::Hash",
    },
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
        type_name: "Js.Error",
        type_canonical: None,
        type_arity: 0,
        identity: "Show<std/prelude::Js.Error>",
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
        type_name: "ProcessSignal",
        type_canonical: Some("std/process::ProcessSignal"),
        type_arity: 0,
        identity: "Show<std/process::ProcessSignal>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ProcessSignal",
        type_canonical: Some("std/process::ProcessSignal"),
        type_arity: 0,
        identity: "Debug<std/process::ProcessSignal>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "ProcessError",
        type_canonical: Some("std/process::ProcessError"),
        type_arity: 0,
        identity: "Show<std/process::ProcessError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ProcessError",
        type_canonical: Some("std/process::ProcessError"),
        type_arity: 0,
        identity: "Debug<std/process::ProcessError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "ChildProcessConfigError",
        type_canonical: Some("std/child-process::ChildProcessConfigError"),
        type_arity: 0,
        identity: "Show<std/child-process::ChildProcessConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ChildProcessConfigError",
        type_canonical: Some("std/child-process::ChildProcessConfigError"),
        type_arity: 0,
        identity: "Debug<std/child-process::ChildProcessConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "ChildProcessError",
        type_canonical: Some("std/child-process::ChildProcessError"),
        type_arity: 0,
        identity: "Show<std/child-process::ChildProcessError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ChildProcessError",
        type_canonical: Some("std/child-process::ChildProcessError"),
        type_arity: 0,
        identity: "Debug<std/child-process::ChildProcessError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "ChildExitStatus",
        type_canonical: Some("std/child-process::ChildExitStatus"),
        type_arity: 0,
        identity: "Show<std/child-process::ChildExitStatus>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ChildExitStatus",
        type_canonical: Some("std/child-process::ChildExitStatus"),
        type_arity: 0,
        identity: "Debug<std/child-process::ChildExitStatus>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RandomRangeError",
        type_canonical: Some("std/random::RandomRangeError"),
        type_arity: 0,
        identity: "Show<std/random::RandomRangeError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RandomRangeError",
        type_canonical: Some("std/random::RandomRangeError"),
        type_arity: 0,
        identity: "Debug<std/random::RandomRangeError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "RandomConfigError",
        type_canonical: Some("std/random::RandomConfigError"),
        type_arity: 0,
        identity: "Show<std/random::RandomConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "RandomConfigError",
        type_canonical: Some("std/random::RandomConfigError"),
        type_arity: 0,
        identity: "Debug<std/random::RandomConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "EntropyConfigError",
        type_canonical: Some("std/entropy::EntropyConfigError"),
        type_arity: 0,
        identity: "Show<std/entropy::EntropyConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "EntropyConfigError",
        type_canonical: Some("std/entropy::EntropyConfigError"),
        type_arity: 0,
        identity: "Debug<std/entropy::EntropyConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "EntropyError",
        type_canonical: Some("std/entropy::EntropyError"),
        type_arity: 0,
        identity: "Show<std/entropy::EntropyError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "EntropyError",
        type_canonical: Some("std/entropy::EntropyError"),
        type_arity: 0,
        identity: "Debug<std/entropy::EntropyError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "StdinConfigError",
        type_canonical: Some("std/stdin::StdinConfigError"),
        type_arity: 0,
        identity: "Show<std/stdin::StdinConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "StdinConfigError",
        type_canonical: Some("std/stdin::StdinConfigError"),
        type_arity: 0,
        identity: "Debug<std/stdin::StdinConfigError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "LogError",
        type_canonical: Some("std/log::LogError"),
        type_arity: 0,
        identity: "Show<std/log::LogError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "LogError",
        type_canonical: Some("std/log::LogError"),
        type_arity: 0,
        identity: "Debug<std/log::LogError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "UrlBuildError",
        type_canonical: Some("std/web/navigation::UrlBuildError"),
        type_arity: 0,
        identity: "Show<std/web/navigation::UrlBuildError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "UrlBuildError",
        type_canonical: Some("std/web/navigation::UrlBuildError"),
        type_arity: 0,
        identity: "Debug<std/web/navigation::UrlBuildError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "NavigationError",
        type_canonical: Some("std/web/navigation::NavigationError"),
        type_arity: 0,
        identity: "Show<std/web/navigation::NavigationError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "NavigationError",
        type_canonical: Some("std/web/navigation::NavigationError"),
        type_arity: 0,
        identity: "Debug<std/web/navigation::NavigationError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "StorageArea",
        type_canonical: Some("std/web/storage::StorageArea"),
        type_arity: 0,
        identity: "Show<std/web/storage::StorageArea>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "StorageArea",
        type_canonical: Some("std/web/storage::StorageArea"),
        type_arity: 0,
        identity: "Debug<std/web/storage::StorageArea>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "StorageError",
        type_canonical: Some("std/web/storage::StorageError"),
        type_arity: 0,
        identity: "Show<std/web/storage::StorageError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "StorageError",
        type_canonical: Some("std/web/storage::StorageError"),
        type_arity: 0,
        identity: "Debug<std/web/storage::StorageError>",
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
        trait_name: "Eq",
        type_name: "HexDecodeError",
        type_canonical: Some("std/bytes/hex::HexDecodeError"),
        type_arity: 0,
        identity: "Eq<std/bytes/hex::HexDecodeError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "HexDecodeError",
        type_canonical: Some("std/bytes/hex::HexDecodeError"),
        type_arity: 0,
        identity: "Show<std/bytes/hex::HexDecodeError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "HexDecodeError",
        type_canonical: Some("std/bytes/hex::HexDecodeError"),
        type_arity: 0,
        identity: "Debug<std/bytes/hex::HexDecodeError>",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Base64DecodeError",
        type_canonical: Some("std/bytes/base64::Base64DecodeError"),
        type_arity: 0,
        identity: "Eq<std/bytes/base64::Base64DecodeError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "Base64DecodeError",
        type_canonical: Some("std/bytes/base64::Base64DecodeError"),
        type_arity: 0,
        identity: "Show<std/bytes/base64::Base64DecodeError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "Base64DecodeError",
        type_canonical: Some("std/bytes/base64::Base64DecodeError"),
        type_arity: 0,
        identity: "Debug<std/bytes/base64::Base64DecodeError>",
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
        trait_name: "Show",
        type_name: "ScheduleError",
        type_canonical: Some("std/effect::ScheduleError"),
        type_arity: 0,
        identity: "Show<std/effect::ScheduleError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ScheduleError",
        type_canonical: Some("std/effect::ScheduleError"),
        type_arity: 0,
        identity: "Debug<std/effect::ScheduleError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "ParallelismError",
        type_canonical: Some("std/effect::ParallelismError"),
        type_arity: 0,
        identity: "Show<std/effect::ParallelismError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "ParallelismError",
        type_canonical: Some("std/effect::ParallelismError"),
        type_arity: 0,
        identity: "Debug<std/effect::ParallelismError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "BufferCapacityError",
        type_canonical: Some("std/stream::BufferCapacityError"),
        type_arity: 0,
        identity: "Show<std/stream::BufferCapacityError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "BufferCapacityError",
        type_canonical: Some("std/stream::BufferCapacityError"),
        type_arity: 0,
        identity: "Debug<std/stream::BufferCapacityError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "QueueCreateError",
        type_canonical: Some("std/queue::QueueCreateError"),
        type_arity: 0,
        identity: "Show<std/queue::QueueCreateError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "QueueCreateError",
        type_canonical: Some("std/queue::QueueCreateError"),
        type_arity: 0,
        identity: "Debug<std/queue::QueueCreateError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "QueueClosed",
        type_canonical: Some("std/queue::QueueClosed"),
        type_arity: 0,
        identity: "Show<std/queue::QueueClosed>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "QueueClosed",
        type_canonical: Some("std/queue::QueueClosed"),
        type_arity: 0,
        identity: "Debug<std/queue::QueueClosed>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "SemaphoreCreateError",
        type_canonical: Some("std/semaphore::SemaphoreCreateError"),
        type_arity: 0,
        identity: "Show<std/semaphore::SemaphoreCreateError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "SemaphoreCreateError",
        type_canonical: Some("std/semaphore::SemaphoreCreateError"),
        type_arity: 0,
        identity: "Debug<std/semaphore::SemaphoreCreateError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DurationError",
        type_canonical: Some("std/time::DurationError"),
        type_arity: 0,
        identity: "Show<std/time::DurationError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DurationError",
        type_canonical: Some("std/time::DurationError"),
        type_arity: 0,
        identity: "Debug<std/time::DurationError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DateTimeError",
        type_canonical: Some("std/time::DateTimeError"),
        type_arity: 0,
        identity: "Show<std/time::DateTimeError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DateTimeError",
        type_canonical: Some("std/time::DateTimeError"),
        type_arity: 0,
        identity: "Debug<std/time::DateTimeError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "TimeZoneError",
        type_canonical: Some("std/time::TimeZoneError"),
        type_arity: 0,
        identity: "Show<std/time::TimeZoneError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "TimeZoneError",
        type_canonical: Some("std/time::TimeZoneError"),
        type_arity: 0,
        identity: "Debug<std/time::TimeZoneError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "PathError",
        type_canonical: Some("std/path::PathError"),
        type_arity: 0,
        identity: "Show<std/path::PathError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "PathError",
        type_canonical: Some("std/path::PathError"),
        type_arity: 0,
        identity: "Debug<std/path::PathError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "FileType",
        type_canonical: Some("std/fs::FileType"),
        type_arity: 0,
        identity: "Show<std/fs::FileType>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "FileType",
        type_canonical: Some("std/fs::FileType"),
        type_arity: 0,
        identity: "Debug<std/fs::FileType>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "FileSystemOperation",
        type_canonical: Some("std/fs::FileSystemOperation"),
        type_arity: 0,
        identity: "Show<std/fs::FileSystemOperation>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "FileSystemOperation",
        type_canonical: Some("std/fs::FileSystemOperation"),
        type_arity: 0,
        identity: "Debug<std/fs::FileSystemOperation>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "FileSystemErrorKind",
        type_canonical: Some("std/fs::FileSystemErrorKind"),
        type_arity: 0,
        identity: "Show<std/fs::FileSystemErrorKind>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "FileSystemErrorKind",
        type_canonical: Some("std/fs::FileSystemErrorKind"),
        type_arity: 0,
        identity: "Debug<std/fs::FileSystemErrorKind>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "FileSystemError",
        type_canonical: Some("std/fs::FileSystemError"),
        type_arity: 0,
        identity: "Show<std/fs::FileSystemError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "FileSystemError",
        type_canonical: Some("std/fs::FileSystemError"),
        type_arity: 0,
        identity: "Debug<std/fs::FileSystemError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "FileMetadata",
        type_canonical: Some("std/fs::FileMetadata"),
        type_arity: 0,
        identity: "Show<std/fs::FileMetadata>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "FileMetadata",
        type_canonical: Some("std/fs::FileMetadata"),
        type_arity: 0,
        identity: "Debug<std/fs::FileMetadata>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "DirectoryEntry",
        type_canonical: Some("std/fs::DirectoryEntry"),
        type_arity: 0,
        identity: "Show<std/fs::DirectoryEntry>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "DirectoryEntry",
        type_canonical: Some("std/fs::DirectoryEntry"),
        type_arity: 0,
        identity: "Debug<std/fs::DirectoryEntry>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "WriteMode",
        type_canonical: Some("std/fs::WriteMode"),
        type_arity: 0,
        identity: "Show<std/fs::WriteMode>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "WriteMode",
        type_canonical: Some("std/fs::WriteMode"),
        type_arity: 0,
        identity: "Debug<std/fs::WriteMode>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "FileTextError",
        type_canonical: Some("std/fs::FileTextError"),
        type_arity: 0,
        identity: "Show<std/fs::FileTextError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "FileTextError",
        type_canonical: Some("std/fs::FileTextError"),
        type_arity: 0,
        identity: "Debug<std/fs::FileTextError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "HttpBuildError",
        type_canonical: Some("std/http::HttpBuildError"),
        type_arity: 0,
        identity: "Show<std/http::HttpBuildError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "HttpBuildError",
        type_canonical: Some("std/http::HttpBuildError"),
        type_arity: 0,
        identity: "Debug<std/http::HttpBuildError>",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "HttpError",
        type_canonical: Some("std/http::HttpError"),
        type_arity: 0,
        identity: "Show<std/http::HttpError>",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "HttpError",
        type_canonical: Some("std/http::HttpError"),
        type_arity: 0,
        identity: "Debug<std/http::HttpError>",
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
        trait_name: "JsonEncode",
        type_name: "Bool",
        type_canonical: None,
        type_arity: 0,
        identity: "std/bool::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Bool",
        type_canonical: None,
        type_arity: 0,
        identity: "std/bool::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "std/string::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "String",
        type_canonical: None,
        type_arity: 0,
        identity: "std/string::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Int",
        type_canonical: None,
        type_arity: 0,
        identity: "std/int::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Int",
        type_canonical: None,
        type_arity: 0,
        identity: "std/int::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Unit",
        type_canonical: None,
        type_arity: 0,
        identity: "std/unit::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Unit",
        type_canonical: None,
        type_arity: 0,
        identity: "std/unit::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Json",
        type_canonical: Some("std/json::Json"),
        type_arity: 0,
        identity: "std/json::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Json",
        type_canonical: Some("std/json::Json"),
        type_arity: 0,
        identity: "std/json::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "Semigroup",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Semigroup",
    },
    PreludeStandardInstance {
        trait_name: "Monoid",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Monoid",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "JsonEncode",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::JsonEncode",
    },
    PreludeStandardInstance {
        trait_name: "JsonDecode",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::JsonDecode",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Maybe",
        type_canonical: None,
        type_arity: 1,
        identity: "std/maybe::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Either",
        type_canonical: None,
        type_arity: 2,
        identity: "std/either::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Eq",
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
        trait_name: "Traversable",
        type_name: "Array",
        type_canonical: None,
        type_arity: 1,
        identity: "std/array::Traversable",
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
        trait_name: "Traversable",
        type_name: "List",
        type_canonical: None,
        type_arity: 1,
        identity: "std/list::Traversable",
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
        type_name: "Stream",
        type_canonical: Some("std/stream::Stream"),
        type_arity: 3,
        identity: "std/stream::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "Stream",
        type_canonical: Some("std/stream::Stream"),
        type_arity: 3,
        identity: "std/stream::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "Stream",
        type_canonical: Some("std/stream::Stream"),
        type_arity: 3,
        identity: "std/stream::Monad",
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
    PreludeStandardInstance {
        trait_name: "Eq",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Eq",
    },
    PreludeStandardInstance {
        trait_name: "Ord",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Ord",
    },
    PreludeStandardInstance {
        trait_name: "Hash",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Hash",
    },
    PreludeStandardInstance {
        trait_name: "Show",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Show",
    },
    PreludeStandardInstance {
        trait_name: "Debug",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Debug",
    },
    PreludeStandardInstance {
        trait_name: "Semigroup",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Semigroup",
    },
    PreludeStandardInstance {
        trait_name: "Functor",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Functor",
    },
    PreludeStandardInstance {
        trait_name: "Applicative",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Applicative",
    },
    PreludeStandardInstance {
        trait_name: "Monad",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Monad",
    },
    PreludeStandardInstance {
        trait_name: "Traversable",
        type_name: "NonEmptyList",
        type_canonical: Some("std/non-empty-list::NonEmptyList"),
        type_arity: 1,
        identity: "std/non-empty-list::Traversable",
    },
];

pub(crate) const SPECIAL_STANDARD_INSTANCES: &[PreludeSpecialInstance] = &[
    special_value(
        "Eq",
        &["Int"],
        "std/int::Eq",
        "Int",
        true,
        PreludeSpecialInstanceDispatch::OperatorAbi,
    ),
    special_value(
        "Eq",
        &["Bool"],
        "std/bool::Eq",
        "Bool",
        true,
        PreludeSpecialInstanceDispatch::OperatorAbi,
    ),
    special_value(
        "Eq",
        &["String"],
        "std/string::Eq",
        "String",
        true,
        PreludeSpecialInstanceDispatch::OperatorAbi,
    ),
    special_value(
        "Eq",
        &["Char"],
        "std/char::Eq",
        "Char",
        true,
        PreludeSpecialInstanceDispatch::OperatorAbi,
    ),
    special_value(
        "Eq",
        &["Unit"],
        "std/unit::Eq",
        "Unit",
        true,
        PreludeSpecialInstanceDispatch::OperatorAbi,
    ),
    special_value(
        "Zero",
        &["Int"],
        "std/int::Zero",
        "Int",
        false,
        PreludeSpecialInstanceDispatch::Dictionary,
    ),
    special_value(
        "One",
        &["Int"],
        "std/int::One",
        "Int",
        false,
        PreludeSpecialInstanceDispatch::Dictionary,
    ),
    special_value(
        "Zero",
        &["Float"],
        "std/float::Zero",
        "Float",
        false,
        PreludeSpecialInstanceDispatch::Dictionary,
    ),
    special_value(
        "One",
        &["Float"],
        "std/float::One",
        "Float",
        false,
        PreludeSpecialInstanceDispatch::Dictionary,
    ),
    special_homogeneous(
        "Add",
        &["String", "String", "String"],
        "std/string::Add",
        "String",
    ),
    special_homogeneous("Add", &["Int", "Int", "Int"], "std/int::Add", "Int"),
    special_homogeneous("Sub", &["Int", "Int", "Int"], "std/int::Sub", "Int"),
    special_homogeneous("Mul", &["Int", "Int", "Int"], "std/int::Mul", "Int"),
    special_homogeneous("Div", &["Int", "Int", "Int"], "std/int::Div", "Int"),
    special_homogeneous("Rem", &["Int", "Int", "Int"], "std/int::Rem", "Int"),
    special_homogeneous("Pow", &["Int", "Int", "Int"], "std/int::Pow", "Int"),
    special_homogeneous(
        "Add",
        &["Float", "Float", "Float"],
        "std/float::Add",
        "Float",
    ),
    special_homogeneous(
        "Sub",
        &["Float", "Float", "Float"],
        "std/float::Sub",
        "Float",
    ),
    special_homogeneous(
        "Mul",
        &["Float", "Float", "Float"],
        "std/float::Mul",
        "Float",
    ),
    special_homogeneous(
        "Div",
        &["Float", "Float", "Float"],
        "std/float::Div",
        "Float",
    ),
    special_homogeneous(
        "Rem",
        &["Float", "Float", "Float"],
        "std/float::Rem",
        "Float",
    ),
    special_homogeneous(
        "Pow",
        &["Float", "Float", "Float"],
        "std/float::Pow",
        "Float",
    ),
    special_external_arithmetic(
        "Add",
        &["BigInt", "BigInt", "BigInt"],
        "std/big-int::Add",
        "BigInt",
        "std/big-int::BigInt",
        false,
    ),
    special_external_arithmetic(
        "Sub",
        &["BigInt", "BigInt", "BigInt"],
        "std/big-int::Sub",
        "BigInt",
        "std/big-int::BigInt",
        false,
    ),
    special_external_arithmetic(
        "Mul",
        &["BigInt", "BigInt", "BigInt"],
        "std/big-int::Mul",
        "BigInt",
        "std/big-int::BigInt",
        false,
    ),
    special_external_arithmetic(
        "Div",
        &["BigInt", "BigInt", "BigInt"],
        "std/big-int::Div",
        "BigInt",
        "std/big-int::BigInt",
        false,
    ),
    special_external_arithmetic(
        "Rem",
        &["BigInt", "BigInt", "BigInt"],
        "std/big-int::Rem",
        "BigInt",
        "std/big-int::BigInt",
        false,
    ),
    special_external_arithmetic(
        "Pow",
        &["BigInt", "Int", "BigInt"],
        "std/big-int::Pow",
        "BigInt",
        "std/big-int::BigInt",
        true,
    ),
    special_external_arithmetic(
        "Add",
        &["Decimal", "Decimal", "Decimal"],
        "std/decimal::Add",
        "Decimal",
        "std/decimal::Decimal",
        false,
    ),
    special_external_arithmetic(
        "Sub",
        &["Decimal", "Decimal", "Decimal"],
        "std/decimal::Sub",
        "Decimal",
        "std/decimal::Decimal",
        false,
    ),
    special_external_arithmetic(
        "Mul",
        &["Decimal", "Decimal", "Decimal"],
        "std/decimal::Mul",
        "Decimal",
        "std/decimal::Decimal",
        false,
    ),
    special_collection(
        "Iterable",
        &["Array<A>", "A"],
        "std/array::Iterable",
        "Array",
        false,
    ),
    special_collection(
        "Iterable",
        &["List<A>", "A"],
        "std/list::Iterable",
        "List",
        false,
    ),
    special_collection(
        "Iterable",
        &["Range<Int>", "Int"],
        "std/range::Iterable",
        "Range",
        true,
    ),
    special_collection(
        "Iterable",
        &["Iterator<A>", "A"],
        "std/iterator::Iterable",
        "Iterator",
        false,
    ),
    special_collection(
        "Reducible",
        &["Array<A>", "A"],
        "std/array::Reducible",
        "Array",
        false,
    ),
    special_collection(
        "Reducible",
        &["List<A>", "A"],
        "std/list::Reducible",
        "List",
        false,
    ),
    special_collection(
        "Reducible",
        &["Range<Int>", "Int"],
        "std/range::Reducible",
        "Range",
        true,
    ),
    special_external_collection(
        "Iterable",
        &["NonEmptyList<A>", "A"],
        "std/non-empty-list::Iterable",
        "NonEmptyList",
        "std/non-empty-list::NonEmptyList",
        false,
    ),
    special_external_collection(
        "Reducible",
        &["NonEmptyList<A>", "A"],
        "std/non-empty-list::Reducible",
        "NonEmptyList",
        "std/non-empty-list::NonEmptyList",
        false,
    ),
    special_external_collection(
        "Iterable",
        &["Map<K, V>", "(K, V)"],
        "std/map::Iterable",
        "Map",
        "std/map::Map",
        true,
    ),
    special_external_collection(
        "Reducible",
        &["Map<K, V>", "(K, V)"],
        "std/map::Reducible",
        "Map",
        "std/map::Map",
        true,
    ),
    special_external_collection(
        "Iterable",
        &["Set<A>", "A"],
        "std/set::Iterable",
        "Set",
        "std/set::Set",
        false,
    ),
    special_external_collection(
        "Reducible",
        &["Set<A>", "A"],
        "std/set::Reducible",
        "Set",
        "std/set::Set",
        false,
    ),
];

const fn special_value(
    trait_name: &'static str,
    arguments: &'static [&'static str],
    identity: &'static str,
    type_name: &'static str,
    strict_equality_compatible: bool,
    dispatch: PreludeSpecialInstanceDispatch,
) -> PreludeSpecialInstance {
    PreludeSpecialInstance {
        type_name,
        identity,
        strict_equality_compatible,
        trait_name,
        arguments,
        dispatch,
        head: PreludeSpecialInstanceHead::Value(type_name),
    }
}

const fn special_homogeneous(
    trait_name: &'static str,
    arguments: &'static [&'static str],
    identity: &'static str,
    type_name: &'static str,
) -> PreludeSpecialInstance {
    PreludeSpecialInstance {
        type_name,
        identity,
        strict_equality_compatible: false,
        trait_name,
        arguments,
        dispatch: PreludeSpecialInstanceDispatch::OperatorAbi,
        head: PreludeSpecialInstanceHead::Homogeneous3(type_name),
    }
}

const fn special_external_arithmetic(
    trait_name: &'static str,
    arguments: &'static [&'static str],
    identity: &'static str,
    type_name: &'static str,
    canonical: &'static str,
    int_exponent: bool,
) -> PreludeSpecialInstance {
    PreludeSpecialInstance {
        type_name,
        identity,
        strict_equality_compatible: false,
        trait_name,
        arguments,
        dispatch: PreludeSpecialInstanceDispatch::Dictionary,
        head: PreludeSpecialInstanceHead::ExternalArithmetic3 {
            type_name,
            canonical,
            int_exponent,
        },
    }
}

const fn special_collection(
    trait_name: &'static str,
    arguments: &'static [&'static str],
    identity: &'static str,
    type_name: &'static str,
    int_element: bool,
) -> PreludeSpecialInstance {
    PreludeSpecialInstance {
        type_name,
        identity,
        strict_equality_compatible: false,
        trait_name,
        arguments,
        dispatch: PreludeSpecialInstanceDispatch::Dictionary,
        head: PreludeSpecialInstanceHead::Collection {
            constructor: type_name,
            canonical: None,
            int_element,
            tuple_elements: false,
        },
    }
}

const fn special_external_collection(
    trait_name: &'static str,
    arguments: &'static [&'static str],
    identity: &'static str,
    type_name: &'static str,
    canonical: &'static str,
    tuple_elements: bool,
) -> PreludeSpecialInstance {
    PreludeSpecialInstance {
        type_name,
        identity,
        strict_equality_compatible: false,
        trait_name,
        arguments,
        dispatch: PreludeSpecialInstanceDispatch::Dictionary,
        head: PreludeSpecialInstanceHead::Collection {
            constructor: type_name,
            canonical: Some(canonical),
            int_element: false,
            tuple_elements,
        },
    }
}

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

const SUM_VARIANTS: &[PreludeVariant] = &[PreludeVariant {
    name: "Sum",
    canonical: "std/prelude::Sum",
    payload_parameter: Some(0),
}];

const PRODUCT_VARIANTS: &[PreludeVariant] = &[PreludeVariant {
    name: "Product",
    canonical: "std/prelude::Product",
    payload_parameter: Some(0),
}];

pub(crate) const SUM_TYPES: &[PreludeSumType] = &[
    PreludeSumType {
        name: "Product",
        canonical: "std/prelude::Product",
        type_parameters: &["A"],
        variants: PRODUCT_VARIANTS,
    },
    PreludeSumType {
        name: "Sum",
        canonical: "std/prelude::Sum",
        type_parameters: &["A"],
        variants: SUM_VARIANTS,
    },
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
                | "FileSystem"
                | "Js.Error"
                | "Js.Unknown"
                | "Js.NullOr"
                | "Js.Nullable"
                | "Js.UndefinedOr"
                | "Js.Promise"
                | "Js.Object"
                | "Js.Number"
                | "Js.String"
                | "Js.Null"
                | "Js.Undefined"
                | "Js.MutableArray"
                | "Js.Callback"
        ),
        SymbolNamespace::Value => {
            crate::effect_ops::known_effect_operation_by_surface(spelling).is_some()
                || PURE_FUNCTION_NAMES.contains(&spelling)
        }
        SymbolNamespace::Operator => {
            seseragi_syntax::standard_operator(spelling).is_some()
                || seseragi_syntax::standard_trait_operator(spelling).is_some()
        }
        SymbolNamespace::Trait => trait_by_name(spelling).is_some(),
        _ => false,
    }
}

pub fn type_constructor_arity(spelling: &str) -> Option<u32> {
    if let Some(sum_type) = sum_type_for_symbol(SymbolNamespace::Type, spelling) {
        return Some(sum_type.type_parameters.len() as u32);
    }
    match spelling {
        "Array" | "List" | "Range" | "Iterator" | "Task" | "Js.NullOr" | "Js.Nullable"
        | "Js.UndefinedOr" | "Js.Promise" | "Js.MutableArray" => Some(1),
        "Js.Callback" => Some(2),
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
            | "std/prelude::Js.Error"
            | "std/prelude::Js.Unknown"
            | "std/prelude::Js.NullOr"
            | "std/prelude::Js.Nullable"
            | "std/prelude::Js.UndefinedOr"
            | "std/prelude::Js.Promise"
            | "std/prelude::Js.Object"
            | "std/prelude::Js.Number"
            | "std/prelude::Js.String"
            | "std/prelude::Js.Null"
            | "std/prelude::Js.Undefined"
            | "std/prelude::Js.MutableArray"
            | "std/prelude::Js.Callback"
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
    let mut trait_parameters = trait_spec
        .type_parameters
        .iter()
        .map(|parameter| {
            if parameter.arity == 0 {
                TypeParameter::value(parameter.name)
            } else {
                TypeParameter::constructor(parameter.name, parameter.arity)
            }
        })
        .collect::<Vec<_>>();
    let constructor = trait_spec.type_parameters[0].name;
    let a = named("A");
    let b = named("B");
    let applied_a = applied(constructor, a.clone());
    let applied_b = applied(constructor, b.clone());
    let mut type_parameters = trait_parameters.clone();
    match method.kind {
        PreludeTraitMethodKind::Eq => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named("A"), named("A")],
            result: named("Bool"),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::Compare => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named("A"), named("A")],
            result: named("Ordering"),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::Hash => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named("A")],
            result: named("Int"),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::Append => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named(constructor), named(constructor)],
            result: named(constructor),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::Empty => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named("Unit")],
            result: named(constructor),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::Show | PreludeTraitMethodKind::Debug => {
            PreludeTraitMethodSignature {
                type_parameters: trait_parameters,
                parameters: vec![named(constructor)],
                result: named("String"),
                constraints: Vec::new(),
            }
        }
        PreludeTraitMethodKind::Zero | PreludeTraitMethodKind::One => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named("Unit")],
            result: named("A"),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::Map => {
            type_parameters.extend([TypeParameter::value("A"), TypeParameter::value("B")]);
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![function(a, b.clone()), applied_a],
                result: applied_b,
                constraints: Vec::new(),
            }
        }
        PreludeTraitMethodKind::Pure => {
            type_parameters.push(TypeParameter::value("A"));
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![a],
                result: applied_a,
                constraints: Vec::new(),
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
                constraints: Vec::new(),
            }
        }
        PreludeTraitMethodKind::FlatMap => {
            type_parameters.extend([TypeParameter::value("A"), TypeParameter::value("B")]);
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![function(a, applied_b.clone()), applied_a],
                result: applied_b,
                constraints: Vec::new(),
            }
        }
        PreludeTraitMethodKind::Iterate => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named("C")],
            result: applied("Iterator", named("A")),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::Reduce => {
            trait_parameters.push(TypeParameter::value("B"));
            PreludeTraitMethodSignature {
                type_parameters: trait_parameters,
                parameters: vec![
                    named("B"),
                    function(named("B"), function(named("A"), named("B"))),
                    named("C"),
                ],
                result: named("B"),
                constraints: Vec::new(),
            }
        }
        PreludeTraitMethodKind::Traverse => {
            type_parameters.extend([
                TypeParameter::constructor("G", 1),
                TypeParameter::value("A"),
                TypeParameter::value("B"),
            ]);
            PreludeTraitMethodSignature {
                type_parameters,
                parameters: vec![
                    function(named("A"), applied("G", named("B"))),
                    applied("F", named("A")),
                ],
                result: applied("G", applied("F", named("B"))),
                constraints: vec![TypedConstraint {
                    name: "Applicative".to_owned(),
                    arguments: vec![named("G")],
                }],
            }
        }
        PreludeTraitMethodKind::Arithmetic => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named("L"), named("R")],
            result: named("O"),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::EncodeJson => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![named(constructor)],
            result: external("std/json", "Json"),
            constraints: Vec::new(),
        },
        PreludeTraitMethodKind::DecodeJson => PreludeTraitMethodSignature {
            type_parameters: trait_parameters,
            parameters: vec![external("std/json", "Json")],
            result: TypedType::Named {
                name: "Either".to_owned(),
                arguments: vec![external("std/json", "DecodeError"), named(constructor)],
            },
            constraints: Vec::new(),
        },
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
                    matches!(trait_spec.type_parameters, [parameter]
                        if instance.type_arity.checked_sub(arguments) == Some(parameter.arity))
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
                arguments: constraint
                    .type_argument_indices
                    .iter()
                    .map(|index| arguments.get(*index).cloned())
                    .collect::<Option<Vec<_>>>()?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(constraints)
}

pub(crate) fn standard_instance_constraint_specs(
    identity: &str,
) -> &'static [PreludeStandardInstanceConstraint] {
    const EQ_ELEMENT: &[PreludeStandardInstanceConstraint] = &[PreludeStandardInstanceConstraint {
        trait_name: "Eq",
        type_argument_indices: &[0],
    }];
    const ORD_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "Ord",
            type_argument_indices: &[0],
        }];
    const HASH_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "Hash",
            type_argument_indices: &[0],
        }];
    const SHOW_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "Show",
            type_argument_indices: &[0],
        }];
    const DEBUG_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "Debug",
            type_argument_indices: &[0],
        }];
    const EQ_EITHER: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Eq",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Eq",
            type_argument_indices: &[1],
        },
    ];
    const SHOW_EITHER: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Show",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Show",
            type_argument_indices: &[1],
        },
    ];
    const DEBUG_EITHER: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Debug",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Debug",
            type_argument_indices: &[1],
        },
    ];
    const JSON_ENCODE_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "JsonEncode",
            type_argument_indices: &[0],
        }];
    const JSON_DECODE_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "JsonDecode",
            type_argument_indices: &[0],
        }];
    const JSON_ENCODE_EITHER: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "JsonEncode",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "JsonEncode",
            type_argument_indices: &[1],
        },
    ];
    const JSON_DECODE_EITHER: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "JsonDecode",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "JsonDecode",
            type_argument_indices: &[1],
        },
    ];
    const MAP_EQ: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Eq",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Hash",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Eq",
            type_argument_indices: &[1],
        },
    ];
    const SET_EQ: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Eq",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Hash",
            type_argument_indices: &[0],
        },
    ];
    const MAP_DECODE: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Eq",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Hash",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "JsonDecode",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "JsonDecode",
            type_argument_indices: &[1],
        },
    ];
    const SET_DECODE: &[PreludeStandardInstanceConstraint] = &[
        PreludeStandardInstanceConstraint {
            trait_name: "Eq",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "Hash",
            type_argument_indices: &[0],
        },
        PreludeStandardInstanceConstraint {
            trait_name: "JsonDecode",
            type_argument_indices: &[0],
        },
    ];
    const SEMIGROUP_ELEMENT: &[PreludeStandardInstanceConstraint] =
        &[PreludeStandardInstanceConstraint {
            trait_name: "Semigroup",
            type_argument_indices: &[0],
        }];
    match identity {
        "std/transformer/maybe::Functor"
        | "std/transformer/maybe::Applicative"
        | "std/transformer/maybe::Monad" => &[PreludeStandardInstanceConstraint {
            trait_name: "Monad",
            type_argument_indices: &[0],
        }],
        "std/transformer/either::Functor"
        | "std/transformer/either::Applicative"
        | "std/transformer/either::Monad" => &[PreludeStandardInstanceConstraint {
            trait_name: "Monad",
            type_argument_indices: &[1],
        }],
        "std/transformer/reader::Functor"
        | "std/transformer/reader::Applicative"
        | "std/transformer/reader::Monad" => &[PreludeStandardInstanceConstraint {
            trait_name: "Monad",
            type_argument_indices: &[1],
        }],
        "std/transformer/state::Functor"
        | "std/transformer/state::Applicative"
        | "std/transformer/state::Monad" => &[PreludeStandardInstanceConstraint {
            trait_name: "Monad",
            type_argument_indices: &[1],
        }],
        "std/transformer/writer::Functor"
        | "std/transformer/writer::Applicative"
        | "std/transformer/writer::Monad" => &[
            PreludeStandardInstanceConstraint {
                trait_name: "Monad",
                type_argument_indices: &[1],
            },
            PreludeStandardInstanceConstraint {
                trait_name: "Monoid",
                type_argument_indices: &[0],
            },
        ],
        "std/product::Semigroup" => &[PreludeStandardInstanceConstraint {
            trait_name: "Mul",
            type_argument_indices: &[0, 0, 0],
        }],
        "std/product::Monoid" => &[
            PreludeStandardInstanceConstraint {
                trait_name: "One",
                type_argument_indices: &[0],
            },
            PreludeStandardInstanceConstraint {
                trait_name: "Mul",
                type_argument_indices: &[0, 0, 0],
            },
        ],

        "std/sum::Semigroup" => &[PreludeStandardInstanceConstraint {
            trait_name: "Add",
            type_argument_indices: &[0, 0, 0],
        }],
        "std/sum::Monoid" => &[
            PreludeStandardInstanceConstraint {
                trait_name: "Zero",
                type_argument_indices: &[0],
            },
            PreludeStandardInstanceConstraint {
                trait_name: "Add",
                type_argument_indices: &[0, 0, 0],
            },
        ],

        "std/validation::Eq" => &[
            PreludeStandardInstanceConstraint {
                trait_name: "Eq",
                type_argument_indices: &[0],
            },
            PreludeStandardInstanceConstraint {
                trait_name: "Eq",
                type_argument_indices: &[1],
            },
        ],
        "std/validation::Show" => SHOW_EITHER,
        "std/validation::Debug" => DEBUG_EITHER,
        "std/maybe::Semigroup" | "std/maybe::Monoid" => SEMIGROUP_ELEMENT,
        "std/map::JsonEncode" => JSON_ENCODE_EITHER,
        "std/map::JsonDecode" => MAP_DECODE,
        "std/set::JsonEncode" => JSON_ENCODE_ELEMENT,
        "std/set::JsonDecode" => SET_DECODE,
        "std/map::Eq" => MAP_EQ,
        "std/set::Eq" => SET_EQ,
        "std/map::Show" => SHOW_EITHER,
        "std/map::Debug" => DEBUG_EITHER,
        "std/set::Show" => SHOW_ELEMENT,
        "std/set::Debug" => DEBUG_ELEMENT,
        "std/array::Eq" | "std/list::Eq" | "std/non-empty-list::Eq" | "std/maybe::Eq" => EQ_ELEMENT,
        "std/either::Eq" => EQ_EITHER,
        "std/non-empty-list::Ord" => ORD_ELEMENT,
        "std/non-empty-list::Hash" => HASH_ELEMENT,
        "std/array::Show"
        | "std/list::Show"
        | "std/maybe::Show"
        | "std/range::Show"
        | "std/non-empty-list::Show" => SHOW_ELEMENT,
        "std/array::Debug"
        | "std/list::Debug"
        | "std/maybe::Debug"
        | "std/range::Debug"
        | "std/non-empty-list::Debug" => DEBUG_ELEMENT,
        "std/either::Show" => SHOW_EITHER,
        "std/either::Debug" => DEBUG_EITHER,
        "std/web/dom::DomRuntimeError::Show" => SHOW_ELEMENT,
        "std/web/dom::DomRuntimeError::Debug" => DEBUG_ELEMENT,
        "std/array::JsonEncode" | "std/list::JsonEncode" | "std/maybe::JsonEncode" => {
            JSON_ENCODE_ELEMENT
        }
        "std/array::JsonDecode" | "std/list::JsonDecode" | "std/maybe::JsonDecode" => {
            JSON_DECODE_ELEMENT
        }
        "std/either::JsonEncode" => JSON_ENCODE_EITHER,
        "std/either::JsonDecode" => JSON_DECODE_EITHER,
        _ => &[],
    }
}

pub fn standard_equality_instance_by_identity(
    identity: &str,
) -> Option<&'static PreludeSpecialInstance> {
    SPECIAL_STANDARD_INSTANCES
        .iter()
        .find(|instance| instance.trait_name == "Eq" && instance.identity == identity)
}

pub fn special_standard_instance_by_identity(
    identity: &str,
) -> Option<&'static PreludeSpecialInstance> {
    SPECIAL_STANDARD_INSTANCES
        .iter()
        .find(|instance| instance.identity == identity)
}

/// Returns the canonical special-instance registry used by operator typing,
/// generic evidence selection, and backend dictionary materialization.
///
/// Consumers must not reconstruct this list from operator spellings or
/// runtime helpers: `dispatch` is only an implementation strategy for the
/// same language-level standard instance.
pub fn special_standard_instances() -> &'static [PreludeSpecialInstance] {
    SPECIAL_STANDARD_INSTANCES
}

#[cfg(test)]
pub(crate) fn special_standard_instance_constraints() -> Vec<TypedConstraint> {
    SPECIAL_STANDARD_INSTANCES
        .iter()
        .map(|instance| {
            let arguments = match instance.head {
                PreludeSpecialInstanceHead::Value(type_name) => vec![named(type_name)],
                PreludeSpecialInstanceHead::Homogeneous3(type_name) => {
                    vec![named(type_name), named(type_name), named(type_name)]
                }
                PreludeSpecialInstanceHead::ExternalArithmetic3 {
                    type_name,
                    canonical,
                    int_exponent,
                } => {
                    let value = TypedType::ExternalNamed {
                        name: type_name.to_owned(),
                        canonical: canonical.to_owned(),
                        arguments: Vec::new(),
                    };
                    vec![
                        value.clone(),
                        if int_exponent {
                            named("Int")
                        } else {
                            value.clone()
                        },
                        value,
                    ]
                }
                PreludeSpecialInstanceHead::Collection {
                    constructor,
                    canonical,
                    int_element,
                    tuple_elements,
                } => {
                    let element = if int_element {
                        named("Int")
                    } else {
                        named("String")
                    };
                    let collection = match canonical {
                        Some(canonical) => TypedType::ExternalNamed {
                            name: constructor.to_owned(),
                            canonical: canonical.to_owned(),
                            arguments: if tuple_elements {
                                vec![element.clone(), element.clone()]
                            } else {
                                vec![element.clone()]
                            },
                        },
                        None => applied(constructor, element.clone()),
                    };
                    let element = if tuple_elements {
                        TypedType::Tuple {
                            elements: vec![element.clone(), element],
                        }
                    } else {
                        element
                    };
                    vec![collection, element]
                }
            };
            TypedConstraint {
                name: instance.trait_name.to_owned(),
                arguments,
            }
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn registered_standard_instance_constraints() -> Vec<(TypedConstraint, &'static str)> {
    STANDARD_INSTANCES
        .iter()
        .map(|instance| {
            let parameter_arity = trait_by_name(instance.trait_name)
                .and_then(|trait_spec| trait_spec.type_parameters.first())
                .map(|parameter| parameter.arity)
                .expect("standard instance trait must have one type parameter");
            let supplied = instance
                .type_arity
                .checked_sub(parameter_arity)
                .expect("instance head must satisfy the trait constructor arity");
            let arguments = (0..supplied).map(|_| named("String")).collect();
            let type_ref = match instance.type_canonical {
                Some(canonical) => TypedType::ExternalNamed {
                    name: instance.type_name.to_owned(),
                    canonical: canonical.to_owned(),
                    arguments,
                },
                None => TypedType::Named {
                    name: instance.type_name.to_owned(),
                    arguments,
                },
            };
            (
                TypedConstraint {
                    name: instance.trait_name.to_owned(),
                    arguments: vec![type_ref],
                },
                instance.identity,
            )
        })
        .collect()
}

pub(crate) fn special_standard_instance(
    constraint: &TypedConstraint,
) -> Option<&'static PreludeSpecialInstance> {
    SPECIAL_STANDARD_INSTANCES.iter().find(|instance| {
        if instance.trait_name != constraint.name {
            return false;
        }
        match (instance.head, constraint.arguments.as_slice()) {
            (PreludeSpecialInstanceHead::Value(expected), [value]) => {
                prelude_named_type_is(value, expected)
            }
            (PreludeSpecialInstanceHead::Homogeneous3(expected), [left, right, output]) => {
                [left, right, output]
                    .iter()
                    .all(|value| prelude_named_type_is(value, expected))
            }
            (
                PreludeSpecialInstanceHead::ExternalArithmetic3 {
                    canonical,
                    int_exponent,
                    ..
                },
                [left, right, output],
            ) => {
                external_named_type_is(left, canonical)
                    && external_named_type_is(output, canonical)
                    && if int_exponent {
                        prelude_named_type_is(right, "Int")
                    } else {
                        external_named_type_is(right, canonical)
                    }
            }
            (
                PreludeSpecialInstanceHead::Collection {
                    constructor,
                    canonical,
                    int_element,
                    tuple_elements,
                },
                [collection, element],
            ) => collection_type_arguments(collection, constructor, canonical).is_some_and(
                |arguments| {
                    collection_element(arguments, tuple_elements).as_ref() == Some(element)
                        && (!int_element || prelude_named_type_is(element, "Int"))
                },
            ),
            _ => false,
        }
    })
}

pub(crate) fn special_homogeneous_instance_heads(trait_name: &str) -> Vec<[TypedType; 3]> {
    SPECIAL_STANDARD_INSTANCES
        .iter()
        .filter_map(|instance| match instance.head {
            PreludeSpecialInstanceHead::Homogeneous3(name) if instance.trait_name == trait_name => {
                let type_ref = TypedType::Named {
                    name: name.to_owned(),
                    arguments: Vec::new(),
                };
                Some([type_ref.clone(), type_ref.clone(), type_ref])
            }
            PreludeSpecialInstanceHead::ExternalArithmetic3 {
                type_name,
                canonical,
                int_exponent,
            } if instance.trait_name == trait_name => {
                let type_ref = TypedType::ExternalNamed {
                    name: type_name.to_owned(),
                    canonical: canonical.to_owned(),
                    arguments: Vec::new(),
                };
                Some([
                    type_ref.clone(),
                    if int_exponent {
                        named("Int")
                    } else {
                        type_ref.clone()
                    },
                    type_ref,
                ])
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn special_collection_constraint(
    trait_name: &str,
    collection: &TypedType,
) -> Option<TypedConstraint> {
    SPECIAL_STANDARD_INSTANCES.iter().find_map(|instance| {
        let PreludeSpecialInstanceHead::Collection {
            constructor,
            canonical,
            int_element,
            tuple_elements,
        } = instance.head
        else {
            return None;
        };
        if instance.trait_name != trait_name {
            return None;
        }
        let arguments = collection_type_arguments(collection, constructor, canonical)?;
        let element = collection_element(arguments, tuple_elements)?;
        (!int_element || prelude_named_type_is(&element, "Int")).then(|| TypedConstraint {
            name: trait_name.to_owned(),
            arguments: vec![collection.clone(), element.clone()],
        })
    })
}

fn collection_element(arguments: &[TypedType], tuple_elements: bool) -> Option<TypedType> {
    match (tuple_elements, arguments) {
        (false, [element]) => Some(element.clone()),
        (true, [key, value]) => Some(TypedType::Tuple {
            elements: vec![key.clone(), value.clone()],
        }),
        _ => None,
    }
}

fn collection_type_arguments<'a>(
    type_ref: &'a TypedType,
    constructor: &str,
    canonical: Option<&str>,
) -> Option<&'a [TypedType]> {
    match type_ref {
        TypedType::Named { name, arguments } if canonical.is_none() && name == constructor => {
            Some(arguments)
        }
        TypedType::ExternalNamed {
            canonical: actual,
            arguments,
            ..
        } if canonical
            .map(str::to_owned)
            .unwrap_or_else(|| format!("std/prelude::{constructor}"))
            == *actual =>
        {
            Some(arguments)
        }
        _ => None,
    }
}

fn prelude_named_type_is(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, arguments }
        if name == expected && arguments.is_empty())
}

fn external_named_type_is(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::ExternalNamed { canonical, arguments, .. }
        if canonical == expected && arguments.is_empty())
}

pub(crate) fn overlapping_standard_instance(
    trait_identity: &str,
    type_ref: &TypedType,
    canonical_type_ref: &str,
) -> Option<&'static PreludeStandardInstance> {
    STANDARD_INSTANCES.iter().find(|instance| {
        standard_instance_head(instance, type_ref).is_some_and(|arguments| {
            trait_by_name(instance.trait_name).is_some_and(|trait_spec| {
                matches!(trait_spec.type_parameters, [parameter]
                    if instance.type_arity.checked_sub(arguments) == Some(parameter.arity))
            }) || (arguments == instance.type_arity
                && matches!(last_type_argument(type_ref), Some(TypedType::Hole)))
        }) && trait_by_name(instance.trait_name)
            .is_some_and(|trait_spec| trait_spec.canonical == trait_identity)
            && standard_instance_canonical_head(instance, canonical_type_ref)
    })
}

pub(crate) fn structural_standard_instance_identity(
    trait_identity: &str,
    type_ref: &TypedType,
) -> Option<&'static str> {
    match (trait_identity, type_ref) {
        ("std/prelude::Eq", TypedType::Tuple { .. }) => Some("std/tuple::Eq"),
        ("std/prelude::Ord", TypedType::Tuple { .. }) => Some("std/tuple::Ord"),
        ("std/prelude::Hash", TypedType::Tuple { .. }) => Some("std/tuple::Hash"),
        ("std/prelude::Eq", TypedType::Record { closed: true, .. }) => Some("std/record::Eq"),
        ("std/prelude::Show", TypedType::Tuple { .. }) => Some("std/tuple::Show"),
        ("std/prelude::Debug", TypedType::Tuple { .. }) => Some("std/tuple::Debug"),
        ("std/prelude::Show", TypedType::Record { closed: true, .. }) => Some("std/record::Show"),
        ("std/prelude::Debug", TypedType::Record { closed: true, .. }) => Some("std/record::Debug"),
        ("std/prelude::JsonEncode", TypedType::Tuple { .. }) => Some("std/tuple::JsonEncode"),
        ("std/prelude::JsonDecode", TypedType::Tuple { .. }) => Some("std/tuple::JsonDecode"),
        ("std/prelude::JsonEncode", TypedType::Record { closed: true, .. }) => {
            Some("std/record::JsonEncode")
        }
        ("std/prelude::JsonDecode", TypedType::Record { closed: true, .. }) => {
            Some("std/record::JsonDecode")
        }
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
    let arguments = match type_ref {
        TypedType::Named { name, arguments }
            if instance.type_canonical.is_none()
                && instance.type_name == name
                && type_constructor_arity(name) == Some(instance.type_arity) =>
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
    let supplied = arguments
        .iter()
        .position(|argument| matches!(argument, TypedType::Hole))
        .unwrap_or(arguments.len());
    // Canonical instances fix a prefix of the constructor. A hole before a
    // fixed argument changes that constructor and cannot borrow this instance.
    arguments[supplied..]
        .iter()
        .all(|argument| matches!(argument, TypedType::Hole))
        .then_some(supplied as u32)
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

fn external(module: &str, name: &str) -> TypedType {
    TypedType::ExternalNamed {
        name: name.to_owned(),
        canonical: format!("{module}::{name}"),
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
        assert_eq!(type_constructor_arity("Js.Unknown"), Some(0));
        assert_eq!(type_constructor_arity("Js.Nullable"), Some(1));
        assert_eq!(type_constructor_arity("Js.Callback"), Some(2));
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
    fn keeps_standard_operator_traits_and_methods_in_the_canonical_registry() {
        for operator in seseragi_syntax::standard_operators() {
            let method = trait_method(operator.trait_name, operator.method_name)
                .expect("standard operator method must be registered in the Prelude");
            assert!(method.operators.contains(&operator.spelling));
        }
        for operator in seseragi_syntax::standard_trait_operators() {
            let method = trait_method(operator.trait_name, operator.method_name)
                .expect("standard trait operator method must be registered in the Prelude");
            assert!(method.operators.contains(&operator.spelling));
        }
    }

    #[test]
    fn describes_multi_parameter_collection_and_arithmetic_traits() {
        let iterable = trait_by_name("Iterable").unwrap();
        assert_eq!(iterable.type_parameters.len(), 2);
        assert_eq!(iterable.type_parameters[0].name, "C");
        assert_eq!(iterable.type_parameters[1].name, "A");

        let reduce = trait_method("Reducible", "reduce").unwrap();
        let reduce = trait_method_signature(reduce);
        assert_eq!(
            reduce.type_parameters.last(),
            Some(&TypeParameter::value("B"))
        );
        assert_eq!(reduce.parameters.len(), 3);
        assert_eq!(reduce.result, named("B"));

        let add = trait_by_name("Add").unwrap();
        assert_eq!(add.type_parameters.len(), 3);
        let add = trait_method_signature(trait_method("Add", "add").unwrap());
        assert_eq!(add.parameters, vec![named("L"), named("R")]);
        assert_eq!(add.result, named("O"));
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

    #[test]
    fn keeps_special_instance_heads_and_dispatch_in_the_registry() {
        let add_heads = special_homogeneous_instance_heads("Add");
        assert_eq!(add_heads.len(), 5);
        assert!(add_heads.contains(&[named("String"), named("String"), named("String")]));
        let big_int = external("std/big-int", "BigInt");
        assert!(add_heads.contains(&[big_int.clone(), big_int.clone(), big_int.clone()]));
        let decimal = external("std/decimal", "Decimal");
        assert!(add_heads.contains(&[decimal.clone(), decimal.clone(), decimal]));
        assert!(special_homogeneous_instance_heads("Pow").contains(&[
            big_int.clone(),
            named("Int"),
            big_int,
        ]));

        let array = applied("Array", named("String"));
        assert_eq!(
            special_collection_constraint("Iterable", &array),
            Some(TypedConstraint {
                name: "Iterable".to_owned(),
                arguments: vec![array, named("String")],
            })
        );
        assert_eq!(
            special_standard_instance_by_identity("std/int::Add").map(|instance| instance.dispatch),
            Some(PreludeSpecialInstanceDispatch::OperatorAbi)
        );
        assert_eq!(
            special_standard_instance_by_identity("std/int::Zero")
                .map(|instance| instance.dispatch),
            Some(PreludeSpecialInstanceDispatch::Dictionary)
        );
    }
}
