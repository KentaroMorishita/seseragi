use crate::typed::pure_issues::MatchIssue;
use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
use crate::{SymbolId, TypedPattern, TypedType};
use seseragi_syntax::SurfacePattern;
use std::collections::BTreeMap;

use super::super::PureExpressionContext;

mod literal;
mod record;
mod struct_pattern;

pub(super) use literal::LiteralPattern;
use literal::{type_boolean_pattern, type_integer_pattern, type_string_pattern};
use record::type_record_pattern;
use struct_pattern::type_struct_pattern;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CoveragePattern {
    Any,
    Literal(LiteralPattern),
    Constructor {
        constructor: SymbolId,
        argument: Option<Box<CoveragePattern>>,
    },
    Tuple(Vec<CoveragePattern>),
    Array {
        elements: Vec<CoveragePattern>,
        has_rest: bool,
    },
    List {
        elements: Vec<CoveragePattern>,
        has_rest: bool,
    },
    Record(Vec<(String, CoveragePattern)>),
    Invalid,
}

pub(in crate::typed::surface_expr) struct PatternAnalysis {
    pub(in crate::typed::surface_expr) typed: TypedPattern,
    pub(super) coverage: CoveragePattern,
    pub(in crate::typed::surface_expr) locals: BTreeMap<SymbolId, SemanticValueType>,
    pub(in crate::typed::surface_expr) issues: Vec<MatchIssue>,
    pub(super) invalid: bool,
}

impl PatternAnalysis {
    pub(in crate::typed::surface_expr) fn is_irrefutable(&self) -> bool {
        !self.invalid && coverage_is_irrefutable(&self.coverage)
    }
}

fn coverage_is_irrefutable(pattern: &CoveragePattern) -> bool {
    match pattern {
        CoveragePattern::Any => true,
        CoveragePattern::Tuple(elements) => elements.iter().all(coverage_is_irrefutable),
        CoveragePattern::Array { elements, has_rest }
        | CoveragePattern::List { elements, has_rest } => *has_rest && elements.is_empty(),
        CoveragePattern::Record(fields) => fields
            .iter()
            .all(|(_, pattern)| coverage_is_irrefutable(pattern)),
        CoveragePattern::Literal(_)
        | CoveragePattern::Constructor { .. }
        | CoveragePattern::Invalid => false,
    }
}

pub(in crate::typed::surface_expr) fn type_pattern(
    pattern: &SurfacePattern,
    expected: &SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> PatternAnalysis {
    if expected.key == SemanticTypeKey::Invalid {
        return suppressed_invalid(pattern_span(pattern));
    }
    match pattern {
        SurfacePattern::Integer { raw, span } => type_integer_pattern(raw, *span, expected),
        SurfacePattern::String { raw, span } => type_string_pattern(raw, *span, expected),
        SurfacePattern::Boolean { value, span } => type_boolean_pattern(*value, *span, expected),
        SurfacePattern::Wildcard { span } => PatternAnalysis {
            typed: TypedPattern::Wildcard {
                type_ref: expected.type_ref.clone(),
                origin: *span,
            },
            coverage: CoveragePattern::Any,
            locals: BTreeMap::new(),
            issues: Vec::new(),
            invalid: false,
        },
        SurfacePattern::Name {
            name,
            name_span,
            span,
        } => {
            let Some(symbol) = context.binding_symbol(*name_span) else {
                return invalid(*span, "pattern binding has no resolved symbol");
            };
            let mut locals = BTreeMap::new();
            locals.insert(symbol, expected.clone());
            PatternAnalysis {
                typed: TypedPattern::Binding {
                    symbol,
                    name: name.clone(),
                    type_ref: expected.type_ref.clone(),
                    origin: *span,
                },
                coverage: CoveragePattern::Any,
                locals,
                issues: Vec::new(),
                invalid: false,
            }
        }
        SurfacePattern::Constructor {
            name_span,
            argument,
            span,
            ..
        } => type_constructor_pattern(*name_span, argument.as_deref(), *span, expected, context),
        SurfacePattern::Tuple { elements, span } => {
            type_tuple_pattern(elements, *span, expected, context)
        }
        SurfacePattern::Array {
            elements,
            rest,
            span,
        } => type_collection_pattern(
            CollectionPatternKind::Array,
            elements,
            rest.as_deref(),
            *span,
            expected,
            context,
        ),
        SurfacePattern::List {
            elements,
            rest,
            span,
        } => type_collection_pattern(
            CollectionPatternKind::List,
            elements,
            rest.as_deref(),
            *span,
            expected,
            context,
        ),
        SurfacePattern::Record { fields, span } => {
            type_record_pattern(fields, *span, expected, context)
        }
        SurfacePattern::Struct {
            name,
            name_span,
            fields,
            span,
        } => type_struct_pattern(name, *name_span, fields, *span, expected, context),
        SurfacePattern::Error { span } => PatternAnalysis {
            typed: TypedPattern::Invalid { origin: *span },
            coverage: CoveragePattern::Invalid,
            locals: BTreeMap::new(),
            issues: Vec::new(),
            invalid: true,
        },
    }
}

#[derive(Clone, Copy)]
enum CollectionPatternKind {
    Array,
    List,
}

impl CollectionPatternKind {
    fn name(self) -> &'static str {
        match self {
            Self::Array => "Array",
            Self::List => "List",
        }
    }
}

fn type_collection_pattern(
    kind: CollectionPatternKind,
    elements: &[SurfacePattern],
    rest: Option<&SurfacePattern>,
    span: seseragi_syntax::ByteSpan,
    expected: &SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> PatternAnalysis {
    let expected_element = match &expected.type_ref {
        TypedType::Named { name, arguments }
        | TypedType::ExternalNamed {
            name, arguments, ..
        } if name == kind.name() && arguments.len() == 1 => {
            context.semantic_value_from_typed_type(&arguments[0])
        }
        _ => {
            return invalid(
                span,
                format!(
                    "{} pattern requires a {} scrutinee",
                    kind.name(),
                    kind.name()
                ),
            );
        }
    };
    let mut children = elements
        .iter()
        .map(|pattern| type_pattern(pattern, &expected_element, context))
        .collect::<Vec<_>>();
    let rest_index = rest.map(|pattern| {
        children.push(type_pattern(pattern, expected, context));
        children.len() - 1
    });
    let mut locals = BTreeMap::new();
    let mut issues = Vec::new();
    let invalid = children.iter().any(|child| child.invalid);
    for child in &children {
        locals.extend(child.locals.clone());
        issues.extend(child.issues.clone());
    }
    let typed_elements = children[..elements.len()]
        .iter()
        .map(|child| child.typed.clone())
        .collect::<Vec<_>>();
    let typed_rest = rest_index.map(|index| Box::new(children[index].typed.clone()));
    let coverage_elements = children[..elements.len()]
        .iter()
        .map(|child| child.coverage.clone())
        .collect::<Vec<_>>();
    let has_rest = rest.is_some();
    let typed = match kind {
        CollectionPatternKind::Array => TypedPattern::Array {
            elements: typed_elements,
            rest: typed_rest,
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        CollectionPatternKind::List => TypedPattern::List {
            elements: typed_elements,
            rest: typed_rest,
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
    };
    let coverage = match kind {
        CollectionPatternKind::Array => CoveragePattern::Array {
            elements: coverage_elements,
            has_rest,
        },
        CollectionPatternKind::List => CoveragePattern::List {
            elements: coverage_elements,
            has_rest,
        },
    };
    PatternAnalysis {
        typed,
        coverage,
        locals,
        issues,
        invalid,
    }
}

fn type_constructor_pattern(
    name_span: seseragi_syntax::ByteSpan,
    argument: Option<&SurfacePattern>,
    span: seseragi_syntax::ByteSpan,
    expected: &SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> PatternAnalysis {
    let Some(target) = context.target(name_span) else {
        return PatternAnalysis {
            typed: TypedPattern::Invalid { origin: span },
            coverage: CoveragePattern::Invalid,
            locals: BTreeMap::new(),
            issues: Vec::new(),
            invalid: true,
        };
    };
    let Some((owner, variant)) = context.semantic_types().constructor(target) else {
        return invalid(span, "constructor does not belong to a closed local ADT");
    };
    let SemanticTypeKey::Adt {
        owner: expected_owner,
        arguments,
    } = &expected.key
    else {
        return invalid(
            span,
            format!(
                "constructor {} does not belong to the matched type",
                variant.spelling
            ),
        );
    };
    if *expected_owner != owner {
        return invalid(
            span,
            format!(
                "constructor {} does not belong to the matched type",
                variant.spelling
            ),
        );
    }
    let canonical = variant.canonical.clone();
    let payload = variant.payload.as_ref().map(|payload| {
        context
            .semantic_types()
            .instantiate_payload(owner, arguments, payload)
    });
    match (payload, argument) {
        (None, None) => PatternAnalysis {
            typed: TypedPattern::Constructor {
                symbol: canonical,
                argument: None,
                type_ref: expected.type_ref.clone(),
                origin: span,
            },
            coverage: CoveragePattern::Constructor {
                constructor: target,
                argument: None,
            },
            locals: BTreeMap::new(),
            issues: Vec::new(),
            invalid: false,
        },
        (Some(payload), Some(argument)) => {
            let nested = type_pattern(argument, &payload, context);
            PatternAnalysis {
                typed: TypedPattern::Constructor {
                    symbol: canonical,
                    argument: Some(Box::new(nested.typed)),
                    type_ref: expected.type_ref.clone(),
                    origin: span,
                },
                coverage: CoveragePattern::Constructor {
                    constructor: target,
                    argument: Some(Box::new(nested.coverage)),
                },
                locals: nested.locals,
                issues: nested.issues,
                invalid: nested.invalid,
            }
        }
        (None, Some(_)) => invalid(span, "nullary constructor pattern cannot have an argument"),
        (Some(_), None) => invalid(span, "payload constructor pattern requires an argument"),
    }
}

fn type_tuple_pattern(
    elements: &[SurfacePattern],
    span: seseragi_syntax::ByteSpan,
    expected: &SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> PatternAnalysis {
    let (TypedType::Tuple { elements: types }, SemanticTypeKey::Tuple(keys)) =
        (&expected.type_ref, &expected.key)
    else {
        return invalid(span, "tuple pattern requires a tuple scrutinee");
    };
    if elements.len() != types.len() || elements.len() != keys.len() {
        return invalid(
            span,
            format!(
                "tuple pattern has {} elements, matched tuple has {}",
                elements.len(),
                types.len()
            ),
        );
    }
    let children = elements
        .iter()
        .zip(types)
        .zip(keys)
        .map(|((pattern, type_ref), key)| {
            type_pattern(
                pattern,
                &SemanticValueType {
                    type_ref: type_ref.clone(),
                    key: key.clone(),
                },
                context,
            )
        })
        .collect::<Vec<_>>();
    let mut locals = BTreeMap::new();
    let mut issues = Vec::new();
    let invalid = children.iter().any(|child| child.invalid);
    for child in &children {
        locals.extend(child.locals.clone());
        issues.extend(child.issues.clone());
    }
    PatternAnalysis {
        typed: TypedPattern::Tuple {
            elements: children.iter().map(|child| child.typed.clone()).collect(),
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        coverage: CoveragePattern::Tuple(
            children.into_iter().map(|child| child.coverage).collect(),
        ),
        locals,
        issues,
        invalid,
    }
}

pub(super) fn invalid(
    span: seseragi_syntax::ByteSpan,
    message: impl Into<String>,
) -> PatternAnalysis {
    PatternAnalysis {
        typed: TypedPattern::Invalid { origin: span },
        coverage: CoveragePattern::Invalid,
        locals: BTreeMap::new(),
        issues: vec![MatchIssue::PatternMismatch {
            pattern: span,
            message: message.into(),
        }],
        invalid: true,
    }
}

fn suppressed_invalid(span: seseragi_syntax::ByteSpan) -> PatternAnalysis {
    PatternAnalysis {
        typed: TypedPattern::Invalid { origin: span },
        coverage: CoveragePattern::Invalid,
        locals: BTreeMap::new(),
        issues: Vec::new(),
        invalid: true,
    }
}

fn pattern_span(pattern: &SurfacePattern) -> seseragi_syntax::ByteSpan {
    match pattern {
        SurfacePattern::Integer { span, .. }
        | SurfacePattern::String { span, .. }
        | SurfacePattern::Boolean { span, .. }
        | SurfacePattern::Wildcard { span }
        | SurfacePattern::Name { span, .. }
        | SurfacePattern::Constructor { span, .. }
        | SurfacePattern::Tuple { span, .. }
        | SurfacePattern::Array { span, .. }
        | SurfacePattern::List { span, .. }
        | SurfacePattern::Record { span, .. }
        | SurfacePattern::Struct { span, .. }
        | SurfacePattern::Error { span } => *span,
    }
}
