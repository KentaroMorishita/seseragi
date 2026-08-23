use super::{
    expression_origin, expression_type, render_document_compact, AnalysisCallable,
    AnalysisCompletionContext, AnalysisCompletionField,
};
use crate::{
    TypeDocument, TypedBlockStatement, TypedComprehensionClause, TypedDecl, TypedDoStatement,
    TypedExpr, TypedInstanceImplementation, TypedModule, TypedMonadDoStatement,
    TypedRecordValueItem, TypedTemplatePart, TypedType,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect_completion_contexts(
    typed: &TypedModule,
    callables: &BTreeMap<String, AnalysisCallable>,
) -> Vec<AnalysisCompletionContext> {
    let mut collector = CompletionCollector {
        callables,
        contexts: Vec::new(),
    };
    for declaration in &typed.declarations {
        match declaration {
            TypedDecl::Let { scheme, value, .. } => {
                collector.visit(value, Some(document(&scheme.type_ref)));
            }
            TypedDecl::Fn { scheme, body, .. } => {
                collector.visit(body, Some(document(&scheme.type_ref)));
            }
            TypedDecl::EffectFn { body, .. } => collector.visit(body, None),
            TypedDecl::Alias { .. } | TypedDecl::Adt { .. } | TypedDecl::Struct { .. } => {}
        }
    }
    for instance in &typed.instances {
        let TypedInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            continue;
        };
        for method in methods {
            collector.visit(&method.body, Some(document(&method.scheme.type_ref)));
        }
    }
    collector.contexts.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then_with(|| left.range.end.cmp(&right.range.end))
            .then_with(|| left.type_name.cmp(&right.type_name))
    });
    collector.contexts.dedup_by(|left, right| {
        left.range == right.range && left.type_document == right.type_document
    });
    collector.contexts
}

struct CompletionCollector<'analysis> {
    callables: &'analysis BTreeMap<String, AnalysisCallable>,
    contexts: Vec<AnalysisCompletionContext>,
}

impl CompletionCollector<'_> {
    fn visit(&mut self, expression: &TypedExpr, expected: Option<TypeDocument>) {
        if let Some(expected) = expected.as_ref() {
            self.push_context(expression, expected.clone());
        }

        match expression {
            TypedExpr::Template { parts, .. } => {
                for part in parts {
                    if let TypedTemplatePart::Interpolation { value, .. } = part {
                        self.visit(value, None);
                    }
                }
            }
            TypedExpr::FieldAccess { receiver, .. }
            | TypedExpr::OptionalFieldAccess { receiver, .. } => self.visit(receiver, None),
            TypedExpr::Call {
                callee, arguments, ..
            }
            | TypedExpr::EffectInvoke {
                callee, arguments, ..
            } => self.visit_call(callee, arguments, expected.as_ref()),
            TypedExpr::EffectCall {
                operation,
                arguments,
                ..
            } => self.visit_call(operation, arguments, expected.as_ref()),
            TypedExpr::Lambda { body, .. } => self.visit(
                body,
                expected.as_ref().and_then(function_result_after_first),
            ),
            TypedExpr::Tuple { elements, .. } => {
                let expected_elements = match expected.as_ref() {
                    Some(TypeDocument::Tuple { elements }) => elements.as_slice(),
                    _ => &[],
                };
                for (index, element) in elements.iter().enumerate() {
                    self.visit(element, expected_elements.get(index).cloned());
                }
            }
            TypedExpr::Array { elements, .. } | TypedExpr::List { elements, .. } => {
                let expected_element = expected.as_ref().and_then(collection_element);
                for element in elements {
                    self.visit(element, expected_element.clone());
                }
            }
            TypedExpr::Record { items, .. } => {
                let expected_fields = expected_record_fields(expected.as_ref());
                for item in items {
                    match item {
                        TypedRecordValueItem::Field { name, value, .. } => {
                            let field_expected = expected_fields
                                .iter()
                                .find(|field| field.name == *name)
                                .map(|field| field.type_ref.clone());
                            self.visit(value, field_expected);
                        }
                        TypedRecordValueItem::Spread { value, .. } => self.visit(value, None),
                    }
                }
            }
            TypedExpr::ArrayComprehension {
                element, clauses, ..
            }
            | TypedExpr::ListComprehension {
                element, clauses, ..
            } => {
                self.visit(element, expected.as_ref().and_then(collection_element));
                for clause in clauses {
                    match clause {
                        TypedComprehensionClause::Generator { source, .. } => {
                            self.visit(source, None)
                        }
                        TypedComprehensionClause::Guard { condition, .. } => {
                            self.visit(condition, Some(named("Bool")))
                        }
                    }
                }
            }
            TypedExpr::Binary { left, right, .. } => {
                self.visit(left, None);
                self.visit(right, None);
            }
            TypedExpr::Unary { operand, .. } => self.visit(operand, None),
            TypedExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit(condition, Some(named("Bool")));
                self.visit(then_branch, expected.clone());
                self.visit(else_branch, expected);
            }
            TypedExpr::Match {
                scrutinee, arms, ..
            } => {
                self.visit(scrutinee, None);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.visit(guard, Some(named("Bool")));
                    }
                    self.visit(&arm.body, expected.clone());
                }
            }
            TypedExpr::Block {
                statements, result, ..
            } => {
                for statement in statements {
                    match statement {
                        TypedBlockStatement::Let { value, .. } => {
                            self.visit(value, expression_type(value).map(|value| document(&value)))
                        }
                        TypedBlockStatement::Function { body, .. } => {
                            self.visit(body, expression_type(body).map(|value| document(&value)))
                        }
                    }
                }
                self.visit(result, expected);
            }
            TypedExpr::DoBlock {
                statements, result, ..
            } => {
                for statement in statements {
                    match statement {
                        TypedDoStatement::Effect { value }
                        | TypedDoStatement::PureLet { value, .. }
                        | TypedDoStatement::Bind { value, .. } => self.visit(value, None),
                    }
                }
                self.visit(result, None);
            }
            TypedExpr::MonadDo {
                statements, result, ..
            } => {
                for statement in statements {
                    match statement {
                        TypedMonadDoStatement::Expression { value }
                        | TypedMonadDoStatement::PureLet { value, .. }
                        | TypedMonadDoStatement::Bind { value, .. } => self.visit(value, None),
                    }
                }
                self.visit(result, expected);
            }
            TypedExpr::Unit { .. }
            | TypedExpr::Integer { .. }
            | TypedExpr::Float { .. }
            | TypedExpr::String { .. }
            | TypedExpr::Boolean { .. }
            | TypedExpr::Variable { .. } => {}
        }
    }

    fn visit_call(
        &mut self,
        callee: &str,
        arguments: &[TypedExpr],
        expected_result: Option<&TypeDocument>,
    ) {
        let expected_parameters = self
            .callables
            .get(callee)
            .map(|callable| instantiate_parameters(callable, arguments, expected_result))
            .unwrap_or_default();
        for (index, argument) in arguments.iter().enumerate() {
            self.visit(argument, expected_parameters.get(index).cloned());
        }
    }

    fn push_context(&mut self, expression: &TypedExpr, expected: TypeDocument) {
        if !matches!(expected, TypeDocument::Record { .. }) {
            return;
        }
        let range = expression_origin(expression);
        if range.end <= range.start {
            return;
        }
        let actual_fields = match expression_type(expression) {
            Some(TypedType::Record { fields, .. }) => fields
                .into_iter()
                .map(|field| field.name)
                .collect::<BTreeSet<_>>(),
            _ => BTreeSet::new(),
        };
        let excluded_ranges = match expression {
            TypedExpr::Record { items, .. } => items
                .iter()
                .map(|item| expression_origin(item.value()))
                .filter(|range| range.end > range.start)
                .collect(),
            _ => Vec::new(),
        };
        let record_fields = expected_record_fields(Some(&expected))
            .iter()
            .filter(|field| !actual_fields.contains(&field.name))
            .map(|field| AnalysisCompletionField {
                name: field.name.clone(),
                optional: field.optional,
                type_name: render_document_compact(&field.type_ref),
                type_document: field.type_ref.clone(),
            })
            .collect();
        self.contexts.push(AnalysisCompletionContext {
            range,
            type_name: render_document_compact(&expected),
            type_document: expected,
            record_fields,
            excluded_ranges,
        });
    }
}

fn instantiate_parameters(
    callable: &AnalysisCallable,
    arguments: &[TypedExpr],
    expected_result: Option<&TypeDocument>,
) -> Vec<TypeDocument> {
    let parameter_names = callable
        .type_document
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    let parameter_documents = callable
        .parameters
        .iter()
        .map(|parameter| parameter.type_document.clone())
        .collect::<Vec<_>>();
    let result_document = callable_result_document(callable);
    let mut substitutions = BTreeMap::new();
    for (parameter, argument) in parameter_documents.iter().zip(arguments) {
        let Some(actual) = expression_type(argument) else {
            continue;
        };
        infer_parameters(
            parameter,
            &document(&actual),
            &parameter_names,
            &mut substitutions,
        );
    }
    if let Some(expected_result) = expected_result {
        let remaining = if arguments.len() < parameter_documents.len() {
            TypeDocument::Function {
                parameters: parameter_documents[arguments.len()..].to_vec(),
                result: Box::new(result_document.clone()),
            }
        } else {
            result_document
        };
        infer_parameters(
            &remaining,
            expected_result,
            &parameter_names,
            &mut substitutions,
        );
    }
    parameter_documents
        .iter()
        .map(|parameter| substitute_parameters(parameter, &substitutions))
        .collect()
}

fn callable_result_document(callable: &AnalysisCallable) -> TypeDocument {
    match &callable.type_document.type_ref {
        TypeDocument::Function { result, .. } => result.as_ref().clone(),
        other => other.clone(),
    }
}

fn infer_parameters(
    parameter: &TypeDocument,
    actual: &TypeDocument,
    parameter_names: &BTreeSet<&str>,
    substitutions: &mut BTreeMap<String, TypeDocument>,
) {
    match (parameter, actual) {
        (
            TypeDocument::Variable {
                name,
                arity: 0,
                arguments,
            },
            actual,
        ) if arguments.is_empty()
            && parameter_names.contains(name.as_str())
            && !matches!(actual, TypeDocument::Unknown) =>
        {
            substitutions
                .entry(name.clone())
                .or_insert_with(|| actual.clone());
        }
        (
            TypeDocument::Named {
                name: parameter_name,
                canonical: parameter_canonical,
                arguments: parameter_arguments,
            },
            TypeDocument::Named {
                name: actual_name,
                canonical: actual_canonical,
                arguments: actual_arguments,
            },
        ) if same_named_type(
            parameter_name,
            parameter_canonical.as_deref(),
            actual_name,
            actual_canonical.as_deref(),
        ) =>
        {
            for (parameter, actual) in parameter_arguments.iter().zip(actual_arguments) {
                infer_parameters(parameter, actual, parameter_names, substitutions);
            }
        }
        (
            TypeDocument::Function {
                parameters: parameter_parameters,
                result: parameter_result,
            },
            TypeDocument::Function {
                parameters: actual_parameters,
                result: actual_result,
            },
        ) => {
            for (parameter, actual) in parameter_parameters.iter().zip(actual_parameters) {
                infer_parameters(parameter, actual, parameter_names, substitutions);
            }
            infer_parameters(
                parameter_result,
                actual_result,
                parameter_names,
                substitutions,
            );
        }
        (
            TypeDocument::Tuple {
                elements: parameter_elements,
            },
            TypeDocument::Tuple {
                elements: actual_elements,
            },
        ) => {
            for (parameter, actual) in parameter_elements.iter().zip(actual_elements) {
                infer_parameters(parameter, actual, parameter_names, substitutions);
            }
        }
        (
            TypeDocument::Record {
                fields: parameter_fields,
                ..
            },
            TypeDocument::Record {
                fields: actual_fields,
                ..
            },
        ) => {
            for parameter in parameter_fields {
                let Some(actual) = actual_fields
                    .iter()
                    .find(|actual| actual.name == parameter.name)
                else {
                    continue;
                };
                infer_parameters(
                    &parameter.type_ref,
                    &actual.type_ref,
                    parameter_names,
                    substitutions,
                );
            }
        }
        _ => {}
    }
}

fn substitute_parameters(
    document: &TypeDocument,
    substitutions: &BTreeMap<String, TypeDocument>,
) -> TypeDocument {
    match document {
        TypeDocument::Variable {
            name,
            arity: 0,
            arguments,
        } if arguments.is_empty() => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| document.clone()),
        TypeDocument::Named {
            name,
            canonical,
            arguments,
        } => TypeDocument::Named {
            name: name.clone(),
            canonical: canonical.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute_parameters(argument, substitutions))
                .collect(),
        },
        TypeDocument::Variable {
            name,
            arity,
            arguments,
        } => TypeDocument::Variable {
            name: name.clone(),
            arity: *arity,
            arguments: arguments
                .iter()
                .map(|argument| substitute_parameters(argument, substitutions))
                .collect(),
        },
        TypeDocument::Function { parameters, result } => TypeDocument::Function {
            parameters: parameters
                .iter()
                .map(|parameter| substitute_parameters(parameter, substitutions))
                .collect(),
            result: Box::new(substitute_parameters(result, substitutions)),
        },
        TypeDocument::Tuple { elements } => TypeDocument::Tuple {
            elements: elements
                .iter()
                .map(|element| substitute_parameters(element, substitutions))
                .collect(),
        },
        TypeDocument::Record { closed, fields } => TypeDocument::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| crate::TypeDocumentField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: substitute_parameters(&field.type_ref, substitutions),
                })
                .collect(),
        },
        TypeDocument::RequirementMerge { operands } => TypeDocument::RequirementMerge {
            operands: operands
                .iter()
                .map(|operand| substitute_parameters(operand, substitutions))
                .collect(),
        },
        TypeDocument::TypeConstructor { .. } | TypeDocument::Unknown => document.clone(),
    }
}

fn same_named_type(
    left_name: &str,
    left_canonical: Option<&str>,
    right_name: &str,
    right_canonical: Option<&str>,
) -> bool {
    match (left_canonical, right_canonical) {
        (Some(left), Some(right)) => left == right,
        _ => left_name == right_name,
    }
}

fn function_result_after_first(expected: &TypeDocument) -> Option<TypeDocument> {
    let TypeDocument::Function { parameters, result } = expected else {
        return None;
    };
    let (_, remaining) = parameters.split_first()?;
    let result = if remaining.is_empty() {
        result.as_ref().clone()
    } else {
        TypeDocument::Function {
            parameters: remaining.to_vec(),
            result: result.clone(),
        }
    };
    Some(result)
}

fn collection_element(expected: &TypeDocument) -> Option<TypeDocument> {
    match expected {
        TypeDocument::Named {
            name, arguments, ..
        } if matches!(name.as_str(), "Array" | "List") => arguments.first().cloned(),
        _ => None,
    }
}

fn expected_record_fields(expected: Option<&TypeDocument>) -> &[crate::TypeDocumentField] {
    match expected {
        Some(TypeDocument::Record { fields, .. }) => fields,
        _ => &[],
    }
}

fn document(type_ref: &TypedType) -> TypeDocument {
    TypeDocument::from_typed_type(type_ref)
}

fn named(name: &str) -> TypeDocument {
    TypeDocument::Named {
        name: name.to_owned(),
        canonical: None,
        arguments: Vec::new(),
    }
}
