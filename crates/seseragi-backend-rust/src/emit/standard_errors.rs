use super::*;
use seseragi_syntax::{InterfaceExport, InterfaceType};
use std::sync::OnceLock;

pub(super) const ERROR_TYPES: &[&str] = &[
    "std/big-int::BigIntConversionError",
    "std/big-int::BigIntDivisionError",
    "std/big-int::BigIntParseError",
    "std/big-int::BigIntPowerError",
    "std/bytes/base64::Base64DecodeError",
    "std/bytes/hex::HexDecodeError",
    "std/bytes::ByteError",
    "std/bytes::BytesSliceError",
    "std/child-process::ChildProcessConfigError",
    "std/child-process::ChildProcessError",
    "std/collection::SizeError",
    "std/decimal::DecimalArithmeticError",
    "std/decimal::DecimalContextError",
    "std/decimal::DecimalConversionError",
    "std/decimal::DecimalParseError",
    "std/effect::ParallelismError",
    "std/effect::ScheduleError",
    "std/entropy::EntropyConfigError",
    "std/entropy::EntropyError",
    "std/fs::FileSystemError",
    "std/fs::FileSystemErrorKind",
    "std/fs::FileTextError",
    "std/http::HttpBuildError",
    "std/http::HttpError",
    "std/int::IntParseError",
    "std/log::LogError",
    "std/path::PathError",
    "std/prelude::ConsoleError",
    "std/prelude::Js.Error",
    "std/prelude::StdinError",
    "std/process::ProcessError",
    "std/queue::QueueClosed",
    "std/queue::QueueCreateError",
    "std/random::RandomConfigError",
    "std/random::RandomRangeError",
    "std/regex::RegexCompileError",
    "std/regex::RegexCompileErrorKind",
    "std/semaphore::SemaphoreCreateError",
    "std/stdin::StdinConfigError",
    "std/stream::BufferCapacityError",
    "std/text/grapheme::GraphemeSliceError",
    "std/text::TextSliceError",
    "std/text::Utf8DecodeError",
    "std/time::DateTimeError",
    "std/time::DurationError",
    "std/time::TimeZoneError",
    "std/web/dom::DomError",
    "std/web/html::HtmlBuildError",
    "std/web/navigation::NavigationError",
    "std/web/navigation::UrlBuildError",
    "std/web/storage::StorageError",
];
const RELATED_TYPES: &[&str] = &[
    "std/process::ProcessSignal",
    "std/child-process::ChildOutputChannel",
    "std/child-process::Executable",
    "std/web/storage::StorageArea",
];
pub(super) fn constructor_slots(name: &str) -> Option<Vec<u32>> {
    if name == "std/web/dom::DomRuntimeError" {
        return Some(vec![0]);
    }
    (ERROR_TYPES.contains(&name) || RELATED_TYPES.contains(&name)).then(Vec::new)
}
fn eq_type(name: &str) -> bool {
    matches!(
        name,
        "std/decimal::DecimalParseError"
            | "std/decimal::DecimalContextError"
            | "std/decimal::DecimalArithmeticError"
            | "std/decimal::DecimalConversionError"
            | "std/big-int::BigIntParseError"
            | "std/big-int::BigIntDivisionError"
            | "std/big-int::BigIntPowerError"
            | "std/big-int::BigIntConversionError"
            | "std/text::TextSliceError"
            | "std/text/grapheme::GraphemeSliceError"
            | "std/regex::RegexCompileError"
            | "std/regex::RegexCompileErrorKind"
            | "std/bytes/hex::HexDecodeError"
            | "std/bytes/base64::Base64DecodeError"
            | "std/collection::SizeError"
    )
}
fn contract(identity: &str) -> Option<(&'static str, &'static str)> {
    for &name in ERROR_TYPES {
        for class in ["Show", "Debug", "Eq"] {
            if class == "Debug"
                && matches!(name, "std/int::IntParseError" | "std/prelude::Js.Error")
            {
                continue;
            }
            if class == "Eq" && !eq_type(name) {
                continue;
            }
            let expected = if class == "Eq" && name == "std/collection::SizeError" {
                "std/collection::EqSizeError".to_owned()
            } else {
                format!("{class}<{name}>")
            };
            if identity == expected {
                return Some((class, name));
            }
        }
    }
    match identity {
        "std/web/dom::DomRuntimeError::Show" => Some(("Show", "std/web/dom::DomRuntimeError")),
        "std/web/dom::DomRuntimeError::Debug" => Some(("Debug", "std/web/dom::DomRuntimeError")),
        _ => None,
    }
}
pub(super) fn interface_type(
    ty: &InterfaceType,
    module: &str,
    parameters: &BTreeSet<String>,
) -> Option<CoreType> {
    let recur = |ty| interface_type(ty, module, parameters);
    Some(match ty {
        InterfaceType::Named { name, arguments } => CoreType::Named {
            name: if parameters.contains(name) || name.contains("::") {
                name.clone()
            } else if matches!(
                name.as_str(),
                "Unit"
                    | "Int"
                    | "Bool"
                    | "String"
                    | "Char"
                    | "Float"
                    | "Array"
                    | "List"
                    | "Maybe"
                    | "Either"
                    | "StdinError"
                    | "ConsoleError"
            ) {
                format!("std/prelude::{name}")
            } else {
                format!("{module}::{name}")
            },
            arguments: arguments.iter().map(recur).collect::<Option<_>>()?,
        },
        InterfaceType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => CoreType::Named {
            name: canonical.clone(),
            arguments: arguments.iter().map(recur).collect::<Option<_>>()?,
        },
        InterfaceType::Function { parameter, result } => CoreType::Function {
            parameter: Box::new(recur(parameter)?),
            result: Box::new(recur(result)?),
        },
        InterfaceType::Record { closed, fields } => CoreType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|f| {
                    Some(seseragi_lowering::CoreRecordField {
                        name: f.name.clone(),
                        optional: f.optional,
                        type_ref: recur(&f.type_ref)?,
                    })
                })
                .collect::<Option<_>>()?,
        },
        InterfaceType::Tuple { elements } => CoreType::Tuple {
            elements: elements.iter().map(recur).collect::<Option<_>>()?,
        },
        _ => return None,
    })
}
fn constructors() -> &'static BTreeMap<String, (InterfaceExport, CoreType)> {
    static CONSTRUCTORS: OnceLock<BTreeMap<String, (InterfaceExport, CoreType)>> = OnceLock::new();
    CONSTRUCTORS.get_or_init(|| {
        let mut result = BTreeMap::new();
        for module in seseragi_project::standard_module_interfaces() {
            for export in module.exports {
                if !export
                    .constructor_of
                    .as_deref()
                    .is_some_and(|name| constructor_slots(name).is_some())
                {
                    continue;
                }
                let parameters = export
                    .scheme
                    .type_parameters
                    .iter()
                    .map(|p| p.name.clone())
                    .collect();
                if let Some(ty) =
                    interface_type(&export.scheme.type_ref, &module.module, &parameters)
                {
                    result.insert(export.symbol.clone(), (export, ty));
                }
            }
        }
        result
    })
}
impl Backend<'_> {
    pub(super) fn validate_standard_error_evidence(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<bool> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(false);
        };
        let Some((class, name)) = contract(identity) else {
            return Ok(false);
        };
        if self.program_wasm_library {
            return Err(
                "program WASM: standard error dictionaries require a target-specific ABI proof"
                    .into(),
            );
        }
        let generic = name == "std/web/dom::DomRuntimeError";
        let expected = CoreType::Named {
            name: name.into(),
            arguments: type_arguments.clone(),
        };
        if type_arguments.len() != usize::from(generic)
            || evidence_arguments.len() != usize::from(generic)
            || selected.constraint.name != class
            || selected.constraint.trait_identity.as_deref()
                != Some(format!("std/prelude::{class}").as_str())
            || selected.constraint.arguments.len() != 1
            || self.declared_method_type_kind(&selected.constraint.arguments[0], scope) != Some(0)
            || self.local_canonical_type(&selected.constraint.arguments[0], scope)?
                != self.local_canonical_type(&expected, scope)?
        {
            return Err(
                "Rust backend: standard error dictionary identity, type or children mismatch"
                    .into(),
            );
        }
        if generic {
            let child = &evidence_arguments[0];
            if !lists::standard_trait(child, class, &type_arguments[0]) {
                return Err("Rust backend: standard error child mismatch".into());
            }
            self.validate_display_child(child, class, scope)?;
        }
        Ok(true)
    }
    pub(super) fn standard_error_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !self.validate_standard_error_evidence(selected, scope)? {
            return Ok(None);
        }
        let CoreInstanceEvidence::Standard {
            identity,
            evidence_arguments,
            ..
        } = &selected.evidence
        else {
            unreachable!()
        };
        let (class, name) = contract(identity).unwrap();
        let children = self.evidence(evidence_arguments, scope)?;
        Ok(Some(format!(
            "native_standard_error_dictionary({}, {}, {children})",
            quoted(name),
            quoted(match class {
                "Show" => "show",
                "Debug" => "debug",
                _ => "eq",
            })
        )))
    }
    pub(super) fn standard_error_constructor(
        &mut self,
        name: &str,
        arguments: &[CoreExpr],
        evidence: &[CoreCallEvidence],
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((export, signature)) = constructors().get(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Ok(None);
        }
        if !evidence.is_empty()
            || arguments.len() > usize::from(matches!(signature, CoreType::Function { .. }))
        {
            return Err(
                "Rust backend: standard error constructor arity or evidence mismatch".into(),
            );
        }
        let variables = export
            .scheme
            .type_parameters
            .iter()
            .map(|p| (p.name.clone(), p.arity))
            .collect();
        let mut bindings = BTreeMap::new();
        let mut remaining = signature;
        for argument in arguments {
            let CoreType::Function { parameter, result } = remaining else {
                unreachable!()
            };
            let ty = local::method_argument_type(argument)
                .ok_or("Rust backend: missing error constructor payload type")?;
            self.match_local_type(parameter, &ty, &variables, &mut bindings, scope)?;
            remaining = result;
        }
        self.match_local_type(remaining, actual, &variables, &mut bindings, scope)?;
        let value = if matches!(signature, CoreType::Function { .. }) {
            format!("V::fun(vec![], |_, value| V::adt({}, value))", quoted(name))
        } else {
            format!("V::adt({}, V::Unit)", quoted(name))
        };
        Ok(Some(if arguments.is_empty() {
            value
        } else {
            format!("({value}).apply({})", self.expr(&arguments[0], scope)?)
        }))
    }
}
