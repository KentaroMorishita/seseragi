use crate::{
    SymbolId, SymbolKind, SymbolNamespace, TypedConstraint, TypedExpr, TypedParameter,
    TypedTemplatePart, TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceExpr, SurfaceRecordItem, SurfaceTemplatePart};
use std::collections::BTreeMap;

use super::functions::{application_result_type, TopLevelPureFunction};
use super::pure_issues::{
    ArrayIssue, ConditionalIssue, MatchIssue, MonadDoIssue, PureCallIssue, RangeIssue, RecordIssue,
};
use super::semantic_types::{SemanticTypeCatalog, SemanticTypeKey, SemanticValueType};
use super::TypedResolution;

pub(crate) mod application;
mod array;
mod binary;
mod comprehension;
mod conditional;
mod lambda;
mod match_expression;
mod monad_do;
mod record;
mod struct_value;
mod tuple;

pub(crate) struct PureExpressionContext<'a> {
    parameters: BTreeMap<SymbolId, SemanticValueType>,
    evidence_parameters: Vec<super::call_evidence::ScopedCallEvidence>,
    resolution: &'a TypedResolution<'a>,
    expected: Option<SemanticValueType>,
}

impl<'a> PureExpressionContext<'a> {
    pub(crate) fn new(parameters: &[TypedParameter], resolution: &'a TypedResolution<'a>) -> Self {
        Self {
            parameters: resolution.parameter_types(parameters),
            evidence_parameters: Vec::new(),
            resolution,
            expected: None,
        }
    }

    pub(crate) fn with_expected(&self, expected: Option<SemanticValueType>) -> Self {
        Self {
            parameters: self.parameters.clone(),
            evidence_parameters: self.evidence_parameters.clone(),
            resolution: self.resolution,
            expected,
        }
    }

    pub(super) fn without_expected(&self) -> Self {
        self.with_expected(None)
    }

    pub(super) fn expected(&self) -> Option<&SemanticValueType> {
        self.expected.as_ref()
    }

    pub(super) fn target(&self, origin: ByteSpan) -> Option<SymbolId> {
        self.resolution.target(origin, SymbolNamespace::Value)
    }

    pub(super) fn type_target(&self, origin: ByteSpan) -> Option<SymbolId> {
        self.resolution.target(origin, SymbolNamespace::Type)
    }

    pub(super) fn callable_candidates(&self, origin: ByteSpan) -> Vec<TopLevelPureFunction> {
        self.resolution
            .candidates(origin, SymbolNamespace::Value)
            .iter()
            .filter_map(|candidate| self.callable(*candidate).cloned())
            .collect()
    }

    pub(super) fn operator_target(&self, origin: ByteSpan) -> Option<SymbolId> {
        self.resolution.target(origin, SymbolNamespace::Operator)
    }

    pub(super) fn binding_symbol(&self, origin: ByteSpan) -> Option<SymbolId> {
        self.resolution
            .declaration_symbol(origin, SymbolKind::PatternBinding)
            .map(|symbol| symbol.id)
    }

    pub(super) fn lambda_parameter_symbol(&self, origin: ByteSpan) -> Option<SymbolId> {
        self.resolution
            .declaration_symbol(origin, SymbolKind::Parameter)
            .map(|symbol| symbol.id)
    }

    pub(super) fn callable(&self, target: SymbolId) -> Option<&TopLevelPureFunction> {
        self.resolution.callable(target)
    }

    pub(super) fn inherent_method(
        &self,
        receiver: &SemanticTypeKey,
        name: &str,
    ) -> Option<TopLevelPureFunction> {
        self.resolution.inherent_method(receiver, name).cloned()
    }

    pub(super) fn callable_value(&self, target: SymbolId) -> Option<TopLevelPureFunction> {
        if let Some(callable) = self.callable(target) {
            return Some(callable.clone());
        }
        let value = self.parameters.get(&target)?;
        let symbol = self.resolution.symbol(target)?;
        let mut type_ref = &value.type_ref;
        let mut parameters = Vec::new();
        while let TypedType::Function { parameter, result } = type_ref {
            parameters.push(parameter.as_ref().clone());
            type_ref = result;
        }
        if parameters.is_empty() {
            return None;
        }
        Some(TopLevelPureFunction {
            symbol: symbol.spelling.clone(),
            trait_identity: None,
            trait_method: None,
            type_parameters: Vec::new(),
            constraints: Vec::new(),
            constraint_identities: Vec::new(),
            semantic_parameters: parameters
                .iter()
                .map(|parameter| self.semantic_value_from_typed_type(parameter).key)
                .collect(),
            parameters,
            result: type_ref.clone(),
            semantic_result: self.semantic_value_from_typed_type(type_ref).key,
        })
    }

    pub(super) fn semantic_types(&self) -> &SemanticTypeCatalog {
        self.resolution.semantic_types()
    }

    pub(super) fn semantic_value_from_typed_type(&self, type_ref: &TypedType) -> SemanticValueType {
        self.resolution.semantic_value_from_typed_type(type_ref)
    }

    pub(super) fn hydrate_semantic_value(&self, value: SemanticValueType) -> SemanticValueType {
        if value.key != SemanticTypeKey::Other {
            return value;
        }
        let hydrated = self.semantic_value_from_typed_type(&value.type_ref);
        if hydrated.key != SemanticTypeKey::Other {
            hydrated
        } else {
            value
        }
    }

    pub(super) fn semantic_value_from_type_ref(
        &self,
        type_ref: &seseragi_syntax::TypeRef,
    ) -> SemanticValueType {
        self.resolution.semantic_value_from_type_ref(type_ref)
    }

    pub(super) fn select_call_evidence(
        &self,
        constraints: &[crate::TypedConstraint],
        constraint_identities: &[Option<String>],
    ) -> Result<Vec<crate::TypedCallEvidence>, crate::TypedConstraint> {
        super::call_evidence::select_function_call_evidence(
            constraints,
            constraint_identities,
            self.resolution,
            &self.evidence_parameters,
        )
    }

    pub(super) fn select_iterable_evidence(
        &self,
        collection: TypedType,
    ) -> Result<(TypedType, crate::TypedCallEvidence), crate::TypedConstraint> {
        let trait_identity = self.trait_identity("Iterable");
        super::call_evidence::select_iterable_evidence(
            collection,
            trait_identity.as_deref(),
            self.resolution,
            &self.evidence_parameters,
        )
    }

    pub(super) fn select_show_evidence(
        &self,
        value: TypedType,
    ) -> Result<(String, crate::TypedCallEvidence), TypedConstraint> {
        let constraint = TypedConstraint {
            name: "Show".to_owned(),
            arguments: vec![value],
        };
        let trait_identity = self
            .trait_identity("Show")
            .unwrap_or_else(|| "std/prelude::Show".to_owned());
        let selected = super::call_evidence::select_function_call_evidence(
            std::slice::from_ref(&constraint),
            &[Some(trait_identity.clone())],
            self.resolution,
            &self.evidence_parameters,
        )
        .map_err(|_| constraint.clone())?
        .into_iter()
        .next()
        .ok_or(constraint)?;
        Ok((trait_identity, selected))
    }

    pub(super) fn select_binary_operator_evidence(
        &self,
        trait_name: &str,
        left: TypedType,
        right: TypedType,
    ) -> Result<(TypedType, crate::TypedCallEvidence), TypedConstraint> {
        let trait_identity = self.trait_identity(trait_name);
        super::call_evidence::select_binary_operator_evidence(
            trait_name,
            left,
            right,
            trait_identity.as_deref(),
            self.resolution,
            &self.evidence_parameters,
        )
    }

    pub(super) fn select_binary_equality_evidence(
        &self,
        left: TypedType,
        right: TypedType,
    ) -> Result<crate::TypedCallEvidence, TypedConstraint> {
        let trait_identity = self.trait_identity("Eq");
        super::call_evidence::select_binary_equality_evidence(
            left,
            right,
            trait_identity.as_deref(),
            self.resolution,
            &self.evidence_parameters,
        )
    }

    pub(super) fn select_binary_operator_reference_evidence(
        &self,
        trait_name: &str,
        left: TypedType,
        right: TypedType,
        output: TypedType,
    ) -> Result<([TypedType; 3], crate::TypedCallEvidence), TypedConstraint> {
        let trait_identity = self.trait_identity(trait_name);
        super::call_evidence::select_binary_operator_reference_evidence(
            trait_name,
            left,
            right,
            output,
            trait_identity.as_deref(),
            self.resolution,
            &self.evidence_parameters,
        )
    }

    pub(super) fn trait_identity(&self, name: &str) -> Option<String> {
        let mut identities = self
            .resolution
            .resolved()
            .symbols
            .iter()
            .filter(|symbol| symbol.namespace == SymbolNamespace::Trait && symbol.spelling == name)
            .filter_map(|symbol| symbol.canonical.clone());
        let Some(identity) = identities.next() else {
            return crate::prelude::is_standalone_symbol(SymbolNamespace::Trait, name)
                .then(|| format!("std/prelude::{name}"));
        };
        identities
            .all(|candidate| candidate == identity)
            .then_some(identity)
    }

    pub(super) fn with_locals(&self, locals: BTreeMap<SymbolId, SemanticValueType>) -> Self {
        let mut parameters = self.parameters.clone();
        parameters.extend(locals);
        Self {
            parameters,
            evidence_parameters: self.evidence_parameters.clone(),
            resolution: self.resolution,
            expected: self.expected.clone(),
        }
    }

    pub(crate) fn with_evidence_parameters(
        mut self,
        evidence_parameters: Vec<super::call_evidence::ScopedCallEvidence>,
    ) -> Self {
        self.evidence_parameters = evidence_parameters;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SurfaceExpressionAnalysis {
    pub(crate) value: TypedExpr,
    pub(crate) conditional_issue: Option<ConditionalIssue>,
    pub(crate) array_issue: Option<ArrayIssue>,
    pub(crate) record_issue: Option<RecordIssue>,
    pub(crate) range_issue: Option<RangeIssue>,
    pub(crate) pure_call_issue: Option<PureCallIssue>,
    pub(crate) monad_do_issue: Option<MonadDoIssue>,
    pub(crate) match_issues: Vec<MatchIssue>,
    pub(crate) semantic_type: SemanticTypeKey,
}

impl SurfaceExpressionAnalysis {
    pub(super) fn valid(value: TypedExpr) -> Self {
        Self {
            value,
            conditional_issue: None,
            array_issue: None,
            record_issue: None,
            range_issue: None,
            pure_call_issue: None,
            monad_do_issue: None,
            match_issues: Vec::new(),
            semantic_type: SemanticTypeKey::Other,
        }
    }

    pub(crate) fn valid_with_semantic_type(
        value: TypedExpr,
        semantic_type: SemanticTypeKey,
    ) -> Self {
        Self {
            value,
            conditional_issue: None,
            array_issue: None,
            record_issue: None,
            range_issue: None,
            pure_call_issue: None,
            monad_do_issue: None,
            match_issues: Vec::new(),
            semantic_type,
        }
    }

    pub(super) fn merge_issues_from(&mut self, child: Self) {
        self.conditional_issue = self.conditional_issue.take().or(child.conditional_issue);
        self.array_issue = self.array_issue.take().or(child.array_issue);
        self.record_issue = self.record_issue.take().or(child.record_issue);
        self.range_issue = self.range_issue.take().or(child.range_issue);
        self.pure_call_issue = self.pure_call_issue.take().or(child.pure_call_issue);
        self.monad_do_issue = self.monad_do_issue.take().or(child.monad_do_issue);
        self.match_issues.extend(child.match_issues);
    }
}

pub(crate) fn analyze_resolved_expression(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_surface_expression(expression, context)
}

pub(crate) fn surface_expression_type_hint(expression: &SurfaceExpr) -> Option<TypedType> {
    match expression {
        SurfaceExpr::Unit { .. } => Some(named_type("Unit")),
        SurfaceExpr::Integer { .. } => Some(named_type("Int")),
        SurfaceExpr::String { .. } => Some(named_type("String")),
        SurfaceExpr::Template { .. } => Some(named_type("String")),
        SurfaceExpr::Boolean { .. } => Some(named_type("Bool")),
        SurfaceExpr::Tuple { elements, .. } => Some(TypedType::Tuple {
            elements: elements
                .iter()
                .map(surface_expression_type_hint)
                .collect::<Option<Vec<_>>>()?,
        }),
        SurfaceExpr::Record { items, .. } => {
            let mut fields = BTreeMap::new();
            for item in items {
                match item {
                    SurfaceRecordItem::Field { name, value, .. } => {
                        fields.insert(
                            name.clone(),
                            crate::TypedRecordField {
                                name: name.clone(),
                                optional: false,
                                type_ref: surface_expression_type_hint(value)?,
                            },
                        );
                    }
                    SurfaceRecordItem::Spread { value, .. } => {
                        let TypedType::Record {
                            fields: spread_fields,
                            ..
                        } = surface_expression_type_hint(value)?
                        else {
                            return None;
                        };
                        for field in spread_fields {
                            fields.insert(field.name.clone(), field);
                        }
                    }
                }
            }
            Some(TypedType::Record {
                closed: true,
                fields: fields.into_values().collect(),
            })
        }
        SurfaceExpr::Struct {
            name,
            type_arguments: Some(type_arguments),
            ..
        } => Some(TypedType::Named {
            name: name.clone(),
            arguments: type_arguments
                .iter()
                .map(super::type_ref::typed_type_from_type_ref)
                .collect(),
        }),
        SurfaceExpr::Struct { .. } => None,
        SurfaceExpr::Array { elements, .. } => {
            let element = elements.first().and_then(surface_expression_type_hint)?;
            elements
                .iter()
                .skip(1)
                .all(|value| surface_expression_type_hint(value).as_ref() == Some(&element))
                .then_some(TypedType::Named {
                    name: "Array".to_owned(),
                    arguments: vec![element],
                })
        }
        SurfaceExpr::List { elements, .. } => {
            let element = elements.first().and_then(surface_expression_type_hint)?;
            elements
                .iter()
                .skip(1)
                .all(|value| surface_expression_type_hint(value).as_ref() == Some(&element))
                .then_some(TypedType::Named {
                    name: "List".to_owned(),
                    arguments: vec![element],
                })
        }
        SurfaceExpr::ArrayComprehension { element, .. } => Some(TypedType::Named {
            name: "Array".to_owned(),
            arguments: vec![surface_expression_type_hint(element)?],
        }),
        SurfaceExpr::ListComprehension { element, .. } => Some(TypedType::Named {
            name: "List".to_owned(),
            arguments: vec![surface_expression_type_hint(element)?],
        }),
        SurfaceExpr::Grouped { value, .. } => surface_expression_type_hint(value),
        SurfaceExpr::Lambda {
            parameter, body, ..
        } => Some(TypedType::Function {
            parameter: Box::new(super::type_ref::typed_type_from_type_ref(
                parameter.type_ref.as_ref()?,
            )),
            result: Box::new(surface_expression_type_hint(body)?),
        }),
        SurfaceExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then_type = surface_expression_type_hint(then_branch)?;
            (surface_expression_type_hint(else_branch)? == then_type).then_some(then_type)
        }
        SurfaceExpr::Match { arms, .. } => arms
            .first()
            .and_then(|arm| surface_expression_type_hint(&arm.body)),
        SurfaceExpr::Name { .. }
        | SurfaceExpr::Member { .. }
        | SurfaceExpr::Application { .. }
        | SurfaceExpr::Binary { .. }
        | SurfaceExpr::InfixChain { .. }
        | SurfaceExpr::Do { .. }
        | SurfaceExpr::Error { .. } => None,
    }
}

pub(super) fn type_surface_expression(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    match expression {
        SurfaceExpr::Unit { span } => SurfaceExpressionAnalysis::valid(TypedExpr::Unit {
            type_ref: named_type("Unit"),
            origin: *span,
        }),
        SurfaceExpr::Integer { raw, span } => {
            SurfaceExpressionAnalysis::valid(TypedExpr::Integer {
                value: raw.clone(),
                type_ref: named_type("Int"),
                origin: *span,
            })
        }
        SurfaceExpr::String { raw, span } => SurfaceExpressionAnalysis::valid(TypedExpr::String {
            value: unquote_string(raw),
            type_ref: named_type("String"),
            origin: *span,
        }),
        SurfaceExpr::Template { parts, span } => type_template(parts, *span, context),
        SurfaceExpr::Boolean { value, span } => {
            SurfaceExpressionAnalysis::valid(TypedExpr::Boolean {
                value: *value,
                type_ref: named_type("Bool"),
                origin: *span,
            })
        }
        SurfaceExpr::Name { name, span } => type_name(name, *span, context),
        SurfaceExpr::Member {
            receiver,
            field,
            field_span,
            span,
        } => record::type_member(receiver, field, *field_span, *span, context),
        SurfaceExpr::Grouped { value, .. } => type_surface_expression(value, context),
        SurfaceExpr::Lambda {
            parameter,
            body,
            span,
        } => lambda::type_lambda(parameter, body, *span, context),
        SurfaceExpr::Application { .. } => application::type_application(expression, context),
        SurfaceExpr::Tuple { elements, span } => tuple::type_tuple(elements, *span, context),
        SurfaceExpr::Array { elements, span } => array::type_array(elements, *span, context),
        SurfaceExpr::List { elements, span } => array::type_list(elements, *span, context),
        SurfaceExpr::Record { items, span } => record::type_record(items, *span, context),
        SurfaceExpr::Struct {
            name,
            name_span,
            type_arguments,
            items,
            span,
        } => struct_value::type_struct(
            name,
            *name_span,
            type_arguments.as_deref(),
            items,
            *span,
            context,
        ),
        SurfaceExpr::ArrayComprehension {
            element,
            clauses,
            span,
        } => comprehension::type_array_comprehension(element, clauses, *span, context),
        SurfaceExpr::ListComprehension {
            element,
            clauses,
            span,
        } => comprehension::type_list_comprehension(element, clauses, *span, context),
        SurfaceExpr::Binary {
            operator,
            operator_span,
            left,
            right,
            span,
        } => binary::type_binary(operator, *operator_span, left, right, *span, context),
        SurfaceExpr::InfixChain { span, .. } => {
            // The resolver normalizes every valid chain. Retaining a typed
            // recovery hole here prevents malformed source from reaching a
            // backend as a raw JavaScript operator.
            SurfaceExpressionAnalysis::valid_with_semantic_type(
                TypedExpr::Variable {
                    name: String::new(),
                    evidence: Vec::new(),
                    type_ref: TypedType::Hole,
                    origin: *span,
                },
                SemanticTypeKey::Invalid,
            )
        }
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => conditional::type_if(condition, then_branch, else_branch, *span, context),
        SurfaceExpr::Match {
            scrutinee,
            arms,
            span,
        } => match_expression::type_match(scrutinee, arms, *span, context),
        SurfaceExpr::Do {
            items,
            result,
            span,
        } => monad_do::type_monad_do(items, result.as_deref(), *span, context),
        SurfaceExpr::Error { span } => SurfaceExpressionAnalysis::valid_with_semantic_type(
            TypedExpr::Variable {
                name: String::new(),
                evidence: Vec::new(),
                type_ref: TypedType::Hole,
                origin: *span,
            },
            SemanticTypeKey::Invalid,
        ),
    }
}

fn type_template(
    parts: &[SurfaceTemplatePart],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let mut typed_parts = Vec::with_capacity(parts.len());
    let mut children = Vec::new();
    let mut missing_instance = None;
    for part in parts {
        match part {
            SurfaceTemplatePart::Text { value, span } => {
                typed_parts.push(TypedTemplatePart::Text {
                    value: value.clone(),
                    origin: *span,
                });
            }
            SurfaceTemplatePart::Interpolation { value, span } => {
                let analysis = type_surface_expression(value, &context.without_expected());
                let type_ref = super::type_ref::inferred_type_from_expr(&analysis.value);
                let selected = context.select_show_evidence(type_ref.clone());
                let (trait_identity, evidence) = match selected {
                    Ok(selected) => (selected.0, Some(selected.1)),
                    Err(constraint) => {
                        missing_instance.get_or_insert(PureCallIssue::MissingInstance {
                            callee: value.span(),
                            constraint,
                        });
                        ("std/prelude::Show".to_owned(), None)
                    }
                };
                typed_parts.push(TypedTemplatePart::Interpolation {
                    value: Box::new(analysis.value.clone()),
                    evidence,
                    trait_identity,
                    origin: *span,
                });
                children.push(analysis);
            }
        }
    }
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Template {
            parts: typed_parts,
            type_ref: named_type("String"),
            origin: span,
        },
        if missing_instance.is_some() {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.pure_call_issue = missing_instance;
    for child in children {
        result.merge_issues_from(child);
    }
    result
}

fn type_name(
    name: &str,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    if is_arithmetic_operator_reference(name) {
        return type_arithmetic_operator_reference(name, span, context);
    }
    let Some(target) = context.target(span) else {
        return SurfaceExpressionAnalysis::valid_with_semantic_type(
            TypedExpr::Variable {
                name: name.to_owned(),
                evidence: Vec::new(),
                type_ref: TypedType::Hole,
                origin: span,
            },
            SemanticTypeKey::Invalid,
        );
    };
    if let Some(value_type) = context.parameters.get(&target) {
        return SurfaceExpressionAnalysis::valid_with_semantic_type(
            TypedExpr::Variable {
                name: name.to_owned(),
                evidence: Vec::new(),
                type_ref: value_type.type_ref.clone(),
                origin: span,
            },
            value_type.key.clone(),
        );
    }
    if let Some(type_ref) = context.resolution.top_level_value_type(target) {
        let resolved_name = context
            .resolution
            .symbol(target)
            .map(|symbol| symbol.spelling.clone())
            .unwrap_or_else(|| name.to_owned());
        return SurfaceExpressionAnalysis::valid_with_semantic_type(
            TypedExpr::Variable {
                name: resolved_name,
                evidence: Vec::new(),
                type_ref: type_ref.clone(),
                origin: span,
            },
            context.resolution.semantic_value_key(target),
        );
    }
    if let Some(function) = context.callable(target) {
        let application = if function.parameters.is_empty() {
            Some(super::functions::instantiated_application(
                function,
                context.expected(),
                0,
                &[],
            ))
        } else {
            None
        };
        let semantic_type = application
            .as_ref()
            .map(|application| application.result.key.clone())
            .unwrap_or(SemanticTypeKey::Other);
        let type_ref = application
            .map(|application| application.result.type_ref)
            .unwrap_or_else(|| application_result_type(function, 0));
        return SurfaceExpressionAnalysis::valid_with_semantic_type(
            TypedExpr::Variable {
                name: function.symbol.clone(),
                evidence: Vec::new(),
                type_ref,
                origin: span,
            },
            semantic_type,
        );
    }

    SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Variable {
            name: name.to_owned(),
            evidence: Vec::new(),
            type_ref: TypedType::Hole,
            origin: span,
        },
        SemanticTypeKey::Invalid,
    )
}

fn type_arithmetic_operator_reference(
    name: &str,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let valid = context.operator_target(span).is_some();
    let (left, right, output) = context
        .expected()
        .and_then(|expected| curried_binary_parts(&expected.type_ref))
        .unwrap_or_else(|| (named_type("Int"), named_type("Int"), named_type("Int")));
    let mut missing_instance = None;
    let (type_ref, evidence) = if valid {
        match context.select_binary_operator_reference_evidence(
            arithmetic_trait_name(name),
            left,
            right,
            output,
        ) {
            Ok(([left, right, output], evidence)) => (
                curried_binary_type_with_parameters(left, right, output),
                vec![evidence],
            ),
            Err(constraint) => {
                missing_instance = Some(super::pure_issues::PureCallIssue::MissingInstance {
                    callee: span,
                    constraint,
                });
                (TypedType::Hole, Vec::new())
            }
        }
    } else {
        (TypedType::Hole, Vec::new())
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Variable {
            name: name.to_owned(),
            evidence,
            type_ref,
            origin: span,
        },
        if valid && missing_instance.is_none() {
            SemanticTypeKey::Other
        } else {
            SemanticTypeKey::Invalid
        },
    );
    result.pure_call_issue = missing_instance;
    result
}

fn curried_binary_type_with_parameters(
    left: TypedType,
    right: TypedType,
    result: TypedType,
) -> TypedType {
    TypedType::Function {
        parameter: Box::new(left),
        result: Box::new(TypedType::Function {
            parameter: Box::new(right),
            result: Box::new(result),
        }),
    }
}

fn curried_binary_parts(type_ref: &TypedType) -> Option<(TypedType, TypedType, TypedType)> {
    let TypedType::Function {
        parameter: left,
        result,
    } = type_ref
    else {
        return None;
    };
    let TypedType::Function {
        parameter: right,
        result: output,
    } = result.as_ref()
    else {
        return None;
    };
    Some((
        left.as_ref().clone(),
        right.as_ref().clone(),
        output.as_ref().clone(),
    ))
}

fn arithmetic_trait_name(operator: &str) -> &'static str {
    match operator {
        "+" => "Add",
        "-" => "Sub",
        "*" => "Mul",
        "/" => "Div",
        "%" => "Rem",
        "**" => "Pow",
        _ => unreachable!("arithmetic operator reference must have a standard trait"),
    }
}

fn is_arithmetic_operator_reference(name: &str) -> bool {
    matches!(name, "+" | "-" | "*" | "/" | "%" | "**")
}

pub(super) fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

pub(super) fn named_type_is(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, .. } if name == expected)
}

fn unquote_string(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}
