//! Admission for a resumable, providerless Effect target. WASM panics abort;
//! native catch_unwind cannot establish a finalizer guarantee. Validate every
//! supplied checked declaration, including unused bodies and returned callbacks.
use crate::{emit, project, RustBackendError};
use seseragi_lowering::{
    core_expr_origin, CoreCallEvidence, CoreCallSignature, CoreExpr, CoreInstanceEvidence,
    CoreModule, CoreParameter, CoreStatement, CoreType, SourceSpan,
};
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, RustBackendError>;
type Scope = BTreeMap<String, CoreType>;

fn primitive(name: &str) -> CoreType {
    CoreType::Named {
        name: format!("std/prelude::{name}"),
        arguments: Vec::new(),
    }
}
fn is_never(ty: &CoreType) -> bool {
    ty == &primitive("Never")
}
fn empty_requirements(ty: &CoreType) -> bool {
    matches!(ty, CoreType::Record { closed: true, fields } if fields.is_empty())
}
fn effect(failure: CoreType, success: CoreType) -> CoreType {
    CoreType::Named {
        name: "std/prelude::Effect".into(),
        arguments: vec![
            CoreType::Record {
                closed: true,
                fields: Vec::new(),
            },
            failure,
            success,
        ],
    }
}
fn effect_parts(ty: &CoreType) -> Option<(&CoreType, &CoreType)> {
    let CoreType::Named { name, arguments } = ty else {
        return None;
    };
    let [requirements, failure, success] = arguments.as_slice() else {
        return None;
    };
    (name == "std/prelude::Effect" && empty_requirements(requirements))
        .then_some((failure, success))
}
fn closed_type(ty: &CoreType) -> bool {
    match ty {
        CoreType::Named { name, arguments } => match (name.as_str(), arguments.as_slice()) {
            (
                "std/prelude::Unit"
                | "std/prelude::Never"
                | "std/prelude::Bool"
                | "std/prelude::Int"
                | "std/prelude::String",
                [],
            ) => true,
            ("std/prelude::Maybe", [value]) => closed_type(value),
            ("std/prelude::Either", [left, right]) => closed_type(left) && closed_type(right),
            ("std/ref::Ref", [value]) => crate::program_wasm::has_closed_data_codec(value),
            ("std/prelude::Effect", [requirements, failure, success]) => {
                empty_requirements(requirements) && closed_type(failure) && closed_type(success)
            }
            _ => false,
        },
        CoreType::Tuple { elements } => {
            (2..=32).contains(&elements.len()) && elements.iter().all(closed_type)
        }
        CoreType::Function { parameter, result } => closed_type(parameter) && closed_type(result),
        _ => false,
    }
}

/// Consume canonical closed identities, never display spellings or Holes.
/// The pure public ABI keeps its separate exact-equality check.
pub(crate) fn boundary_accepts(expected: &CoreType, actual: &CoreType) -> bool {
    if !closed_type(expected) || !closed_type(actual) {
        return false;
    }
    if expected == actual || is_never(actual) {
        return true;
    }
    if let (Some((ee, ea)), Some((ae, aa))) = (effect_parts(expected), effect_parts(actual)) {
        return boundary_accepts(ee, ae) && boundary_accepts(ea, aa);
    }
    match (expected, actual) {
        (
            CoreType::Function {
                parameter: ep,
                result: er,
            },
            CoreType::Function {
                parameter: ap,
                result: ar,
            },
        ) => ep == ap && boundary_accepts(er, ar),
        _ => false,
    }
}
fn join(left: &CoreType, right: &CoreType) -> Option<CoreType> {
    if boundary_accepts(left, right) {
        Some(left.clone())
    } else if boundary_accepts(right, left) {
        Some(right.clone())
    } else {
        None
    }
}
fn canonical(module: &CoreModule, ty: &CoreType) -> Option<CoreType> {
    emit::canonical_boundary_type(module, ty).filter(closed_type)
}

// Core bindings have no separate declaration type. Treat their checked value
// annotations as a provisional contract and prove the entire value below.
// A Sequence's last expression omits failures of preceding executed Effects.
fn binding_type(module: &CoreModule, value: &CoreExpr) -> Option<CoreType> {
    if let CoreExpr::Sequence {
        statements, result, ..
    } = value
    {
        let result = binding_type(module, result)?;
        let mut errors = primitive("Never");
        let mut executed = false;
        for statement in statements {
            if let CoreStatement::Bind { value, .. } | CoreStatement::Effect { value } = statement {
                let source = binding_type(module, value)?;
                let (failure, _) = effect_parts(&source)?;
                errors = join(&errors, failure)?;
                executed = true;
            }
        }
        if executed {
            let (failure, success) = effect_parts(&result)?;
            return Some(effect(join(&errors, failure)?, success.clone()));
        }
        Some(result)
    } else {
        canonical(module, &emit::checked_boundary_body_type(module, value)?)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    Succeed,
    Fail,
    Die,
    Defer,
    FlatMap,
    Recover,
    MapError,
    Scoped,
    AcquireRelease,
    FromEither,
    RefMake,
    RefGet,
    RefSet,
}
impl Operation {
    fn named(name: &str, normalized: bool) -> Option<Self> {
        Some(match name {
            "std/effect::succeed" => Self::Succeed,
            "std/effect::fail" => Self::Fail,
            "std/effect::die" => Self::Die,
            "std/effect::defer" => Self::Defer,
            "std/effect::flatMap" => Self::FlatMap,
            "std/effect::recover" => Self::Recover,
            "std/effect::mapError" => Self::MapError,
            "std/effect::scoped" => Self::Scoped,
            "std/effect::acquireRelease" => Self::AcquireRelease,
            "std/effect::fromEither" => Self::FromEither,
            "std/ref::make" => Self::RefMake,
            "std/ref::get" => Self::RefGet,
            "std/ref::set" => Self::RefSet,
            "effect.succeed" if normalized => Self::Succeed,
            "effect.fail" if normalized => Self::Fail,
            "effect.die" if normalized => Self::Die,
            "effect.mapError" if normalized => Self::MapError,
            "effect.fromEither" if normalized => Self::FromEither,
            _ => return None,
        })
    }
    fn arity(self) -> usize {
        match self {
            Self::FlatMap
            | Self::Recover
            | Self::MapError
            | Self::AcquireRelease
            | Self::RefSet => 2,
            _ => 1,
        }
    }
}
struct Global {
    ty: CoreType,
    source_arity: Option<usize>,
}
struct Admission<'a> {
    module: &'a CoreModule,
    globals: &'a BTreeMap<String, Global>,
    edges: &'a mut BTreeMap<String, BTreeSet<String>>,
    current: &'a str,
}
impl Admission<'_> {
    fn error(&self, message: &str, origin: &SourceSpan) -> RustBackendError {
        project::at(
            &format!("program WASM: target-safe Effect ABI: {message}"),
            origin,
            &self.module.module,
        )
    }
    fn ty(&self, ty: &CoreType, origin: &SourceSpan) -> Result<CoreType> {
        canonical(self.module, ty)
            .ok_or_else(|| self.error("a closed providerless canonical type is required", origin))
    }
    fn accepts(&self, expected: &CoreType, actual: &CoreType, origin: &SourceSpan) -> Result<()> {
        if boundary_accepts(expected, actual) {
            Ok(())
        } else {
            Err(self.error(
                "checked value and declared type identities disagree",
                origin,
            ))
        }
    }
    fn evidence(&self, evidence: &[CoreCallEvidence], origin: &SourceSpan) -> Result<()> {
        if evidence.is_empty() {
            Ok(())
        } else {
            Err(self.error(
                "dictionary calls need a fallible evaluation contract",
                origin,
            ))
        }
    }
    fn parameter(&self, parameter: &CoreParameter, origin: &SourceSpan) -> Result<CoreType> {
        let ty = self.ty(&parameter.type_ref, origin)?;
        if !matches!(parameter.kind.as_str(), "named" | "implicit")
            || (parameter.kind == "implicit" && ty != primitive("Unit"))
        {
            return Err(self.error("unproven source parameter kind or identity", origin));
        }
        Ok(ty)
    }
    fn source(
        &mut self,
        name: &str,
        scope: &Scope,
        deferred: bool,
        origin: &SourceSpan,
    ) -> Result<(CoreType, Option<usize>)> {
        if let Some(ty) = scope.get(name) {
            return Ok((ty.clone(), None));
        }
        let local = format!("{}::{name}", self.module.module);
        let symbol = if self.globals.contains_key(name) {
            name
        } else if !name.contains("::") && self.module.bindings.iter().any(|b| b.symbol == local) {
            &local
        } else {
            return Err(self.error("unsupported operation or unproven callable owner", origin));
        };
        if !deferred {
            self.edges
                .entry(self.current.into())
                .or_default()
                .insert(symbol.into());
        }
        let declaration = &self.globals[symbol];
        Ok((declaration.ty.clone(), declaration.source_arity))
    }
    fn effect_annotation(
        &self,
        requirements: &CoreType,
        failure: &CoreType,
        success: &CoreType,
        origin: &SourceSpan,
    ) -> Result<CoreType> {
        if !emit::canonical_boundary_type(self.module, requirements)
            .as_ref()
            .is_some_and(empty_requirements)
        {
            return Err(self.error("providers require another host ABI", origin));
        }
        Ok(effect(self.ty(failure, origin)?, self.ty(success, origin)?))
    }
    fn effect<'a>(
        &self,
        ty: &'a CoreType,
        origin: &SourceSpan,
    ) -> Result<(&'a CoreType, &'a CoreType)> {
        effect_parts(ty)
            .ok_or_else(|| self.error("a checked providerless Effect is required", origin))
    }
    fn function<'a>(
        &self,
        ty: &'a CoreType,
        origin: &SourceSpan,
    ) -> Result<(&'a CoreType, &'a CoreType)> {
        let CoreType::Function { parameter, result } = ty else {
            return Err(self.error("a checked function stage is required", origin));
        };
        Ok((parameter, result))
    }
    fn signature(
        &self,
        signature: &CoreCallSignature,
        callee: &CoreType,
        source_arity: Option<usize>,
        arguments: &[CoreType],
        origin: &SourceSpan,
    ) -> Result<()> {
        if !signature.type_parameters.is_empty()
            || source_arity.is_some_and(|arity| arity != signature.parameters.len())
        {
            return Err(self.error("unproven generic callable or source arity", origin));
        }
        let mut remainder = callee;
        for expected in &signature.parameters {
            let (parameter, result) = self.function(remainder, origin)?;
            if self.ty(expected, origin)? != *parameter {
                return Err(self.error("callee signature changes a source parameter", origin));
            }
            remainder = result;
        }
        if self.ty(&signature.result, origin)? != *remainder {
            return Err(self.error("callee signature changes its returned value", origin));
        }
        self.argument_annotations(signature, arguments, origin)
    }
    fn argument_annotations(
        &self,
        signature: &CoreCallSignature,
        arguments: &[CoreType],
        origin: &SourceSpan,
    ) -> Result<()> {
        if !signature.argument_types.is_empty() {
            if signature.argument_types.len() != arguments.len() {
                return Err(self.error("checked argument annotation arity mismatch", origin));
            }
            for (expected, actual) in signature.argument_types.iter().zip(arguments) {
                self.accepts(&self.ty(expected, origin)?, actual, origin)?;
            }
        }
        Ok(())
    }
    fn apply(
        &self,
        callee: &CoreType,
        arguments: &[CoreType],
        origin: &SourceSpan,
    ) -> Result<CoreType> {
        let mut result = callee;
        for argument in arguments {
            let (parameter, next) = self.function(result, origin)?;
            self.accepts(parameter, argument, origin)?;
            result = next;
        }
        Ok(result.clone())
    }
    fn operation(
        &mut self,
        operation: Operation,
        arguments: &[CoreExpr],
        scope: &Scope,
        deferred: bool,
        origin: &SourceSpan,
    ) -> Result<(CoreType, Vec<CoreType>)> {
        if arguments.len() != operation.arity() {
            return Err(self.error(
                "standard Effects require complete source application",
                origin,
            ));
        }
        let mut types = Vec::new();
        for argument in arguments {
            // Only a direct checked thunk proves a checkpoint before its body.
            let thunk =
                operation == Operation::Defer && matches!(argument, CoreExpr::Lambda { .. });
            types.push(self.expr(argument, scope, deferred || thunk)?);
        }
        let first = &types[0];
        let result = match operation {
            Operation::Succeed => effect(primitive("Never"), first.clone()),
            Operation::Fail => effect(first.clone(), primitive("Never")),
            Operation::RefMake => {
                if !crate::program_wasm::has_closed_data_codec(first) {
                    return Err(self.error("private Ref requires a closed data payload", origin));
                }
                effect(
                    primitive("Never"),
                    CoreType::Named {
                        name: "std/ref::Ref".into(),
                        arguments: vec![first.clone()],
                    },
                )
            }
            Operation::RefGet | Operation::RefSet => {
                let cell = if operation == Operation::RefGet {
                    first
                } else {
                    &types[1]
                };
                let CoreType::Named { name, arguments } = cell else {
                    return Err(self.error("private Ref requires its canonical cell owner", origin));
                };
                let [value] = arguments.as_slice() else {
                    return Err(self.error("private Ref requires one checked payload type", origin));
                };
                if name != "std/ref::Ref" || !crate::program_wasm::has_closed_data_codec(value) {
                    return Err(self.error("private Ref requires its canonical data cell", origin));
                }
                if operation == Operation::RefGet {
                    effect(primitive("Never"), value.clone())
                } else {
                    self.accepts(value, first, origin)?;
                    effect(primitive("Never"), primitive("Unit"))
                }
            }
            Operation::Die => {
                if first != &primitive("String") {
                    return Err(self.error("die requires the canonical String identity", origin));
                }
                effect(primitive("Never"), primitive("Never"))
            }
            Operation::Defer => {
                let (parameter, result) = self.function(first, origin)?;
                if parameter != &primitive("Unit") {
                    return Err(self.error("defer requires a canonical Unit thunk", origin));
                }
                self.effect(result, origin)?;
                result.clone()
            }
            Operation::Scoped => {
                self.effect(first, origin)?;
                first.clone()
            }
            Operation::FromEither => {
                let CoreType::Named { name, arguments } = first else {
                    return Err(self.error("fromEither requires its canonical ADT", origin));
                };
                let [failure, success] = arguments.as_slice() else {
                    return Err(self.error("fromEither requires two checked payloads", origin));
                };
                if name != "std/prelude::Either" {
                    return Err(self.error("fromEither requires its canonical owner", origin));
                }
                effect(failure.clone(), success.clone())
            }
            Operation::FlatMap => {
                let (failure, success) = self.effect(first, origin)?;
                let (parameter, result) = self.function(&types[1], origin)?;
                self.accepts(parameter, success, origin)?;
                let (next_failure, next_success) = self.effect(result, origin)?;
                effect(
                    join(failure, next_failure).ok_or_else(|| {
                        self.error("incompatible Effect failure channels", origin)
                    })?,
                    next_success.clone(),
                )
            }
            Operation::Recover | Operation::MapError => {
                let (parameter, result) = self.function(first, origin)?;
                let (failure, success) = self.effect(&types[1], origin)?;
                self.accepts(parameter, failure, origin)?;
                if operation == Operation::MapError {
                    effect(result.clone(), success.clone())
                } else {
                    let (next_failure, next_success) = self.effect(result, origin)?;
                    effect(
                        next_failure.clone(),
                        join(success, next_success).ok_or_else(|| {
                            self.error("incompatible recovered success channels", origin)
                        })?,
                    )
                }
            }
            Operation::AcquireRelease => {
                let (_, success) = self.effect(first, origin)?;
                let (parameter, result) = self.function(&types[1], origin)?;
                self.accepts(parameter, success, origin)?;
                let (failure, released) = self.effect(result, origin)?;
                if !is_never(failure) || !boundary_accepts(&primitive("Unit"), released) {
                    return Err(self.error("release must be an infallible Unit Effect", origin));
                }
                first.clone()
            }
        };
        Ok((result, types))
    }
    fn constructor(&self, name: &str, ty: &CoreType, origin: &SourceSpan) -> Result<()> {
        let valid = match (name, ty) {
            ("std/prelude::Nothing", CoreType::Named { name, arguments }) => {
                name == "std/prelude::Maybe" && arguments.len() == 1
            }
            (
                "std/prelude::Just" | "std/prelude::Left" | "std/prelude::Right",
                CoreType::Function { parameter, result },
            ) => match result.as_ref() {
                CoreType::Named {
                    name: owner,
                    arguments,
                } => match (name, owner.as_str(), arguments.as_slice()) {
                    ("std/prelude::Just", "std/prelude::Maybe", [value]) => {
                        parameter.as_ref() == value
                    }
                    ("std/prelude::Left", "std/prelude::Either", [left, _]) => {
                        parameter.as_ref() == left
                    }
                    ("std/prelude::Right", "std/prelude::Either", [_, right]) => {
                        parameter.as_ref() == right
                    }
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(self.error("unproven canonical constructor contract", origin))
        }
    }
    fn constructor_call(
        &mut self,
        name: &str,
        arguments: &[CoreExpr],
        output: &CoreType,
        scope: &Scope,
        deferred: bool,
        origin: &SourceSpan,
    ) -> Result<()> {
        if name == "std/prelude::Nothing" {
            self.constructor(name, output, origin)?;
            if !arguments.is_empty() {
                return Err(self.error("Nothing has no source arguments", origin));
            }
            return Ok(());
        }
        if arguments.len() != 1 {
            return Err(self.error("constructor source arity mismatch", origin));
        }
        let argument = self.expr(&arguments[0], scope, deferred)?;
        let function = CoreType::Function {
            parameter: Box::new(argument),
            result: Box::new(output.clone()),
        };
        self.constructor(name, &function, origin)
    }
    fn expr(&mut self, expr: &CoreExpr, scope: &Scope, deferred: bool) -> Result<CoreType> {
        let origin = core_expr_origin(expr);
        match expr {
            CoreExpr::Unit { .. } => Ok(primitive("Unit")),
            CoreExpr::Integer { value, .. } => {
                if !value
                    .parse::<i64>()
                    .ok()
                    .is_some_and(|n| (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&n))
                {
                    return Err(self.error("Int literal is outside the language range", origin));
                }
                Ok(primitive("Int"))
            }
            CoreExpr::String { .. } => Ok(primitive("String")),
            CoreExpr::Boolean { .. } => Ok(primitive("Bool")),
            CoreExpr::Variable {
                name,
                evidence,
                type_ref,
                ..
            } => {
                self.evidence(evidence, origin)?;
                let annotation = self.ty(type_ref, origin)?;
                if !scope.contains_key(name) && Operation::named(name, false).is_some() {
                    return Err(self.error(
                        "first-class standard Effects require another callable proof",
                        origin,
                    ));
                }
                if !scope.contains_key(name) && is_constructor(name) {
                    self.constructor(name, &annotation, origin)?;
                } else {
                    let (actual, _) = self.source(name, scope, deferred, origin)?;
                    self.accepts(&annotation, &actual, origin)?;
                }
                Ok(annotation)
            }
            CoreExpr::Call {
                callee,
                arguments,
                callee_signature,
                evidence,
                type_ref,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } => {
                self.evidence(evidence, origin)?;
                if trait_dispatch.is_some()
                    || !deferred_evidence_parameters.is_empty()
                    || !deferred_evidence_type_constructor_parameters.is_empty()
                    || evidence_argument_index.is_some()
                {
                    return Err(self.error("unproven callable evidence or staging", origin));
                }
                let annotation = self.ty(type_ref, origin)?;
                if !scope.contains_key(callee) && is_constructor(callee) {
                    self.constructor_call(callee, arguments, &annotation, scope, deferred, origin)?;
                    return Ok(annotation);
                }
                let actual = if let Some(operation) = (!scope.contains_key(callee))
                    .then(|| Operation::named(callee, false))
                    .flatten()
                {
                    let (actual, argument_types) =
                        self.operation(operation, arguments, scope, deferred, origin)?;
                    if let Some(signature) = callee_signature {
                        if signature.parameters.len() != operation.arity() {
                            return Err(self
                                .error("standard Effect signature source arity mismatch", origin));
                        }
                        // The canonical intrinsic and checked runtime inputs,
                        // not generic display spellings, prove this operation.
                        self.argument_annotations(signature, &argument_types, origin)?;
                    }
                    actual
                } else {
                    let (callee_type, source_arity) =
                        self.source(callee, scope, deferred, origin)?;
                    let argument_types = arguments
                        .iter()
                        .map(|a| self.expr(a, scope, deferred))
                        .collect::<Result<Vec<_>>>()?;
                    if let Some(signature) = callee_signature {
                        self.signature(
                            signature,
                            &callee_type,
                            source_arity,
                            &argument_types,
                            origin,
                        )?;
                    }
                    self.apply(&callee_type, &argument_types, origin)?
                };
                self.accepts(&annotation, &actual, origin)?;
                Ok(annotation)
            }
            CoreExpr::EffectOperation {
                operation,
                arguments,
                requirements,
                failure,
                success,
                ..
            } => {
                let annotation = self.effect_annotation(requirements, failure, success, origin)?;
                let operation = Operation::named(operation, true).ok_or_else(|| {
                    self.error(
                        "this host, Ref or concurrent operation is unsupported",
                        origin,
                    )
                })?;
                let (actual, _) = self.operation(operation, arguments, scope, deferred, origin)?;
                self.accepts(&annotation, &actual, origin)?;
                Ok(annotation)
            }
            CoreExpr::EffectInvoke {
                callee,
                arguments,
                evidence,
                requirements,
                failure,
                success,
                ..
            } => {
                self.evidence(evidence, origin)?;
                let annotation = self.effect_annotation(requirements, failure, success, origin)?;
                let actual = if let Some(operation) = (!scope.contains_key(callee))
                    .then(|| Operation::named(callee, false))
                    .flatten()
                {
                    self.operation(operation, arguments, scope, deferred, origin)?
                        .0
                } else {
                    let (callee_type, _) = self.source(callee, scope, deferred, origin)?;
                    let argument_types = arguments
                        .iter()
                        .map(|a| self.expr(a, scope, deferred))
                        .collect::<Result<Vec<_>>>()?;
                    self.apply(&callee_type, &argument_types, origin)?
                };
                self.effect(&actual, origin)?;
                self.accepts(&annotation, &actual, origin)?;
                Ok(annotation)
            }
            CoreExpr::Lambda {
                parameter,
                body,
                type_ref,
                ..
            } => {
                let annotation = self.ty(type_ref, origin)?;
                let (expected, result) = self.function(&annotation, origin)?;
                let parameter_type = self.parameter(parameter, origin)?;
                if expected != &parameter_type {
                    return Err(self.error(
                        "lambda parameter changes its checked function input",
                        origin,
                    ));
                }
                let mut inner = scope.clone();
                inner.insert(parameter.id.clone(), parameter_type);
                let actual = self.expr(body, &inner, deferred)?;
                self.accepts(result, &actual, origin)?;
                Ok(annotation)
            }
            CoreExpr::Tuple {
                elements, type_ref, ..
            } => {
                let annotation = self.ty(type_ref, origin)?;
                let CoreType::Tuple { elements: expected } = &annotation else {
                    return Err(self.error("tuple has a non-tuple annotation", origin));
                };
                if expected.len() != elements.len() {
                    return Err(self.error("tuple annotation arity mismatch", origin));
                }
                for (expected, value) in expected.iter().zip(elements) {
                    let actual = self.expr(value, scope, deferred)?;
                    self.accepts(expected, &actual, origin)?;
                }
                Ok(annotation)
            }
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                type_ref,
                ..
            } => {
                let annotation = self.ty(type_ref, origin)?;
                if self.expr(condition, scope, deferred)? != primitive("Bool") {
                    return Err(self.error("if requires a canonical Bool condition", origin));
                }
                for branch in [then_branch, else_branch] {
                    let actual = self.expr(branch, scope, deferred)?;
                    self.accepts(&annotation, &actual, origin)?;
                }
                Ok(annotation)
            }
            CoreExpr::Binary {
                operator,
                left,
                right,
                evidence,
                type_ref,
                ..
            } => {
                let annotation = self.ty(type_ref, origin)?;
                if operator != "+" || annotation != primitive("String") {
                    return Err(self.error(
                        "arithmetic requires fallible evaluation before cleanup can be guaranteed",
                        origin,
                    ));
                }
                let [selected] = evidence.as_slice() else {
                    return Err(
                        self.error("String addition needs selected canonical evidence", origin)
                    );
                };
                if selected.constraint.name != "Add"
                    || selected
                        .constraint
                        .trait_identity
                        .as_deref()
                        .is_some_and(|id| id != "std/prelude::Add")
                    || selected.constraint.arguments.len() != 3
                    || !matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, type_arguments, evidence_arguments } if identity == "std/string::Add" && type_arguments.is_empty() && evidence_arguments.is_empty())
                {
                    return Err(
                        self.error("unproven String addition evidence owner or frame", origin)
                    );
                }
                for argument in &selected.constraint.arguments {
                    if self.ty(argument, origin)? != primitive("String") {
                        return Err(self.error(
                            "String addition evidence changes an operand identity",
                            origin,
                        ));
                    }
                }
                for value in [left, right] {
                    if self.expr(value, scope, deferred)? != primitive("String") {
                        return Err(self.error(
                            "String addition requires two canonical String operands",
                            origin,
                        ));
                    }
                }
                Ok(annotation)
            }
            CoreExpr::Sequence {
                statements, result, ..
            } => {
                let mut inner = scope.clone();
                let mut failure = primitive("Never");
                let mut executed = false;
                for statement in statements {
                    match statement {
                        CoreStatement::PureLet {
                            name,
                            type_parameters,
                            type_ref,
                            value,
                            origin,
                        } => {
                            if !type_parameters.is_empty() {
                                return Err(
                                    self.error("polymorphic local values are unsupported", origin)
                                );
                            }
                            let annotation = self.ty(type_ref, origin)?;
                            let actual = self.expr(value, &inner, deferred)?;
                            self.accepts(&annotation, &actual, origin)?;
                            inner.insert(name.clone(), annotation);
                        }
                        CoreStatement::Bind {
                            name,
                            type_ref,
                            value,
                            origin,
                        } => {
                            let annotation = self.ty(type_ref, origin)?;
                            let actual = self.expr(value, &inner, deferred)?;
                            let (errors, success) = self.effect(&actual, origin)?;
                            self.accepts(&annotation, success, origin)?;
                            failure = join(&failure, errors).ok_or_else(|| {
                                self.error("incompatible executed failure channels", origin)
                            })?;
                            executed = true;
                            inner.insert(name.clone(), annotation);
                        }
                        CoreStatement::Effect { value } => {
                            let actual = self.expr(value, &inner, deferred)?;
                            let (errors, _) = self.effect(&actual, core_expr_origin(value))?;
                            failure = join(&failure, errors).ok_or_else(|| {
                                self.error("incompatible executed failure channels", origin)
                            })?;
                            executed = true;
                        }
                        CoreStatement::LocalFunction { .. } => {
                            return Err(self.error(
                                "local function declarations need a separate termination proof",
                                origin,
                            ))
                        }
                    }
                }
                let actual = self.expr(result, &inner, deferred)?;
                if executed {
                    let (errors, success) = self.effect(&actual, origin)?;
                    Ok(effect(
                        join(&failure, errors).ok_or_else(|| {
                            self.error("incompatible terminal failure channels", origin)
                        })?,
                        success.clone(),
                    ))
                } else {
                    Ok(actual)
                }
            }
            _ => Err(self.error(
                "this expression has no non-panicking target contract",
                origin,
            )),
        }
    }
}
fn is_constructor(name: &str) -> bool {
    matches!(
        name,
        "std/prelude::Nothing" | "std/prelude::Just" | "std/prelude::Left" | "std/prelude::Right"
    )
}

pub(crate) fn validate(modules: &[&CoreModule]) -> Result<()> {
    let mut globals = BTreeMap::new();
    let mut origins = BTreeMap::new();
    for module in modules {
        let at = |message: &str, origin: &SourceSpan| {
            project::at(
                &format!("program WASM: target-safe Effect ABI: {message}"),
                origin,
                &module.module,
            )
        };
        if let Some(instance) = module.instances.first() {
            return Err(at(
                "user dictionaries require fallible evaluation",
                &instance.origin,
            ));
        }
        for function in &module.functions {
            if !function.type_parameters.is_empty() || !function.constraints.is_empty() {
                return Err(at(
                    "generic source functions are unsupported",
                    &function.origin,
                ));
            }
            let ty = function
                .value_type
                .as_ref()
                .and_then(|ty| canonical(module, ty))
                .ok_or_else(|| {
                    at(
                        "checked closed function type is unavailable",
                        &function.origin,
                    )
                })?;
            globals.insert(
                function.symbol.clone(),
                Global {
                    ty,
                    source_arity: Some(function.parameters.len()),
                },
            );
            origins.insert(function.symbol.clone(), (&module.module, &function.origin));
        }
        for binding in &module.bindings {
            if !binding.type_parameters.is_empty() {
                return Err(at("polymorphic globals are unsupported", &binding.origin));
            }
            let ty = binding_type(module, &binding.value).ok_or_else(|| {
                at(
                    "checked closed binding type is unavailable",
                    &binding.origin,
                )
            })?;
            globals.insert(
                binding.symbol.clone(),
                Global {
                    ty,
                    source_arity: None,
                },
            );
            origins.insert(binding.symbol.clone(), (&module.module, &binding.origin));
        }
    }
    let mut edges = BTreeMap::new();
    for module in modules {
        for function in &module.functions {
            let mut admission = Admission {
                module,
                globals: &globals,
                edges: &mut edges,
                current: &function.symbol,
            };
            let mut remainder = &globals[&function.symbol].ty;
            let mut scope = Scope::new();
            for parameter in &function.parameters {
                let (expected, result) = admission.function(remainder, &function.origin)?;
                let actual = admission.parameter(parameter, &function.origin)?;
                if expected != &actual || scope.insert(parameter.id.clone(), actual).is_some() {
                    return Err(admission.error(
                        "source parameter identity or arity mismatch",
                        &function.origin,
                    ));
                }
                remainder = result;
            }
            if function.is_effect && effect_parts(remainder).is_none() {
                return Err(admission.error(
                    "Effect declaration lost its checked result",
                    &function.origin,
                ));
            }
            let actual = admission.expr(&function.body, &scope, false)?;
            admission.accepts(remainder, &actual, &function.origin)?;
        }
        for binding in &module.bindings {
            let mut admission = Admission {
                module,
                globals: &globals,
                edges: &mut edges,
                current: &binding.symbol,
            };
            let actual = admission.expr(&binding.value, &Scope::new(), false)?;
            admission.accepts(&globals[&binding.symbol].ty, &actual, &binding.origin)?;
        }
    }
    fn visit(
        name: &str,
        edges: &BTreeMap<String, BTreeSet<String>>,
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> bool {
        if done.contains(name) {
            return true;
        }
        if !active.insert(name.into()) {
            return false;
        }
        if edges
            .get(name)
            .is_some_and(|next| next.iter().any(|name| !visit(name, edges, active, done)))
        {
            return false;
        }
        active.remove(name);
        done.insert(name.into());
        true
    }
    let mut done = BTreeSet::new();
    for name in origins.keys() {
        if !visit(name, &edges, &mut BTreeSet::new(), &mut done) {
            let (module, origin) = origins[name];
            return Err(project::at("program WASM: target-safe Effect ABI: recursion must cross an explicit defer checkpoint", origin, module));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_canonical_bottom_and_covariant_effect_results_widen() {
        let string = primitive("String");
        let never = primitive("Never");
        assert!(boundary_accepts(
            &effect(string.clone(), string.clone()),
            &effect(never.clone(), string.clone())
        ));
        assert!(boundary_accepts(
            &effect(string.clone(), string.clone()),
            &effect(never.clone(), never.clone())
        ));
        assert!(!boundary_accepts(
            &effect(never.clone(), string.clone()),
            &effect(string.clone(), string.clone())
        ));
        assert!(!boundary_accepts(&string, &primitive("Bool")));
        assert!(!boundary_accepts(&CoreType::Hole, &CoreType::Hole));
        assert!(!boundary_accepts(
            &string,
            &CoreType::Named {
                name: "other::Never".into(),
                arguments: Vec::new()
            }
        ));
        let function = |parameter, result| CoreType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        };
        assert!(boundary_accepts(
            &function(string.clone(), effect(string.clone(), string.clone())),
            &function(string.clone(), effect(never.clone(), string.clone()))
        ));
        assert!(!boundary_accepts(
            &function(never, string.clone()),
            &function(string.clone(), string)
        ));
    }
}
