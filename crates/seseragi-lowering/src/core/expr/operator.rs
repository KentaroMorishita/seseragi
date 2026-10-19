use super::{CoreExpr, CoreParameter, CoreStatement};
use crate::{CoreType, SourceSpan};
use seseragi_syntax::standard_trait_operator;

/// Keep source operand evaluation independent of a trait method's ABI order.
/// This uses ordinary Core lets/calls/lambdas so every backend sees the same
/// saved operands and the same application stages, including unused partials.
pub(super) fn lower_trait_operator(call: CoreExpr, argument_types: Vec<CoreType>) -> CoreExpr {
    let CoreExpr::Call {
        callee,
        arguments,
        evidence,
        deferred_evidence_parameters,
        deferred_evidence_type_constructor_parameters,
        evidence_argument_index,
        trait_dispatch: Some(dispatch),
        type_ref,
        origin,
        ..
    } = &call
    else {
        return call;
    };
    let Some(operator) = standard_trait_operator(callee) else {
        return call;
    };
    if dispatch.method != operator.method_name
        || evidence.is_empty()
        || evidence[0]
            .constraint
            .trait_identity
            .as_deref()
            .is_some_and(|identity| identity != dispatch.trait_identity)
        || !deferred_evidence_parameters.is_empty()
        || !deferred_evidence_type_constructor_parameters.is_empty()
        || dispatch.method_contract.as_ref().is_some_and(|contract| {
            *evidence_argument_index
                != (arguments.len() > contract.parameters.len())
                    .then_some(contract.parameters.len())
        })
        || (dispatch.method_contract.is_none() && evidence.len() != 1)
    {
        // Do not invent a method contract or dictionary-introduction stage.
        return call;
    }

    let mut source_type = type_ref.clone();
    for parameter in argument_types.iter().rev() {
        source_type = function(parameter.clone(), source_type);
    }
    let CoreType::Function {
        parameter: left,
        result,
    } = source_type
    else {
        return call;
    };
    let CoreType::Function {
        parameter: right,
        result,
    } = *result
    else {
        return call;
    };
    let source_types = [*left, *right];
    let [first, second] = operator.method_operand_sources;
    let method_type = function(
        source_types[first].clone(),
        function(source_types[second].clone(), *result),
    );
    let method = format!("__ssrg$trait${}$method", origin.start);
    let selected = CoreExpr::Call {
        callee: operator.method_name.to_owned(),
        arguments: vec![],
        callee_signature: None,
        evidence: evidence.clone(),
        deferred_evidence_parameters: vec![],
        deferred_evidence_type_constructor_parameters: vec![],
        evidence_argument_index: None,
        trait_dispatch: Some(dispatch.clone()),
        type_ref: method_type.clone(),
        origin: origin.clone(),
    };
    let statements = vec![binding(&method, &method_type, selected, origin)];
    let state = OperatorStages {
        origin,
        source_types: &source_types,
        method_sources: operator.method_operand_sources,
        stage: method,
        stage_type: method_type,
        applied: 0,
        operands: [None, None],
    };
    let result = state.source_stage(0, arguments, type_ref);
    CoreExpr::Sequence {
        statements,
        result: Box::new(result),
        origin: origin.clone(),
    }
}

struct OperatorStages<'a> {
    origin: &'a SourceSpan,
    source_types: &'a [CoreType; 2],
    method_sources: [usize; 2],
    stage: String,
    stage_type: CoreType,
    applied: usize,
    operands: [Option<CoreExpr>; 2],
}

impl OperatorStages<'_> {
    fn source_stage(
        mut self,
        index: usize,
        supplied: &[CoreExpr],
        result_type: &CoreType,
    ) -> CoreExpr {
        if index == 2 {
            return if supplied.len() > 2 {
                apply(
                    self.stage,
                    supplied[2..].to_vec(),
                    result_type.clone(),
                    self.origin,
                )
            } else {
                variable(self.stage, self.stage_type, self.origin)
            };
        }
        let source = format!("__ssrg$trait${}$source{index}", self.origin.start);
        let mut statements = Vec::new();
        if let Some(value) = supplied.get(index) {
            statements.push(binding(
                &source,
                &self.source_types[index],
                value.clone(),
                self.origin,
            ));
        }
        self.operands[index] = Some(variable(
            source.clone(),
            self.source_types[index].clone(),
            self.origin,
        ));
        while self.applied < 2 {
            let Some(operand) = self.operands[self.method_sources[self.applied]].take() else {
                break;
            };
            let CoreType::Function { result, .. } = self.stage_type else {
                unreachable!("checked operator method has two curried source stages")
            };
            self.stage_type = *result;
            let stage = format!("__ssrg$trait${}$stage{}", self.origin.start, self.applied);
            statements.push(binding(
                &stage,
                &self.stage_type,
                apply(
                    self.stage,
                    vec![operand],
                    self.stage_type.clone(),
                    self.origin,
                ),
                self.origin,
            ));
            self.stage = stage;
            self.applied += 1;
        }
        let origin = self.origin.clone();
        let parameter_type = self.source_types[index].clone();
        // For a missing operand, result_type already contains its remaining
        // source arrows. Subsequent recursion consumes those arrows once.
        let next_result_type = if index >= supplied.len() {
            match result_type {
                CoreType::Function { result, .. } => result.as_ref(),
                _ => unreachable!("checked operator partial retains its source arrow"),
            }
        } else {
            result_type
        };
        let result = self.source_stage(index + 1, supplied, next_result_type);
        let body = CoreExpr::Sequence {
            statements,
            result: Box::new(result),
            origin: origin.clone(),
        };
        if index >= supplied.len() {
            CoreExpr::Lambda {
                parameter: CoreParameter {
                    id: source,
                    kind: "named".to_owned(),
                    type_ref: parameter_type,
                },
                body: Box::new(body),
                type_ref: result_type.clone(),
                origin,
            }
        } else {
            body
        }
    }
}

fn function(parameter: CoreType, result: CoreType) -> CoreType {
    CoreType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
}
fn variable(name: String, type_ref: CoreType, origin: &SourceSpan) -> CoreExpr {
    CoreExpr::Variable {
        name,
        callable_signature: None,
        evidence: vec![],
        type_ref,
        origin: origin.clone(),
    }
}
fn binding(name: &str, type_ref: &CoreType, value: CoreExpr, origin: &SourceSpan) -> CoreStatement {
    CoreStatement::PureLet {
        type_parameters: vec![],
        name: name.to_owned(),
        type_ref: type_ref.clone(),
        value,
        origin: origin.clone(),
    }
}
fn apply(
    callee: String,
    arguments: Vec<CoreExpr>,
    type_ref: CoreType,
    origin: &SourceSpan,
) -> CoreExpr {
    CoreExpr::Call {
        callee,
        arguments,
        callee_signature: None,
        evidence: vec![],
        deferred_evidence_parameters: vec![],
        deferred_evidence_type_constructor_parameters: vec![],
        evidence_argument_index: None,
        trait_dispatch: None,
        type_ref,
        origin: origin.clone(),
    }
}
