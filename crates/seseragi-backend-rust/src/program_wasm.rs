//! A closed, pure public ABI over checked Core. It is deliberately distinct
//! from the TypeScript foreign ABI and from the compiler WASM adapter.
use crate::{emit, project, RustBackendError};
use serde::{Deserialize, Serialize};
use seseragi_lowering::{CoreModule, CoreType, SourceSpan};
use std::collections::BTreeSet;

pub const PROGRAM_WASM_ABI: &str = "seseragi-program-wasm-pure-v1";
pub const PROGRAM_WASM_EFFECT_ABI: &str = "seseragi-program-wasm-effect-v1";
pub const PROGRAM_WASM_HOST_EFFECT_ABI: &str = "seseragi-program-wasm-host-effect-v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ProgramCodec {
    Unit,
    Bool,
    Int,
    String,
    /// A JS number, including NaN, the infinities and negative zero.
    Float,
    /// A JS string holding exactly one Unicode scalar value.
    Char,
    Tuple {
        elements: Vec<usize>,
    },
    Maybe {
        value: usize,
    },
    Either {
        left: usize,
        right: usize,
    },
    Function {
        parameter: usize,
        result: usize,
    },
    Effect {
        failure: Option<usize>,
        success: Option<usize>,
    },
    HostEffect {
        parameter: usize,
        failure: Option<usize>,
        success: Option<usize>,
    },
    Array {
        element: usize,
    },
    List {
        element: usize,
    },
    /// A closed record or a struct, with fields in name order. A struct
    /// keeps its name so a recursive struct can be declared by name.
    Record {
        fields: Vec<ProgramField>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// A value of a function's type parameter. WASM only holds and returns
    /// it, so the loader passes the host value itself by token.
    Parameter {
        name: String,
    },
    /// A sum type. The wire tag is the variant's index.
    Adt {
        name: String,
        variants: Vec<ProgramVariant>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramField {
    pub name: String,
    pub codec: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramVariant {
    /// The constructor's source name, used as the boundary `tag`.
    pub tag: String,
    /// The constructor's checked identity in the native runtime.
    pub owner: String,
    pub payload: Option<usize>,
}

/// The prefix of a type parameter's name in a checked public type.
const TYPE_PARAMETER: &str = "\0type-parameter::";

fn max_sequence_items() -> usize {
    1 << 20
}

/// The codecs of one artifact. Data types declared in `modules` (ADTs and
/// structs) are found by name; each named type gets one codec, so a
/// recursive type refers to itself by index.
struct Codecs<'a> {
    items: Vec<ProgramCodec>,
    modules: Option<&'a [&'a CoreModule]>,
    named: Vec<(CoreType, usize)>,
}

impl<'a> Codecs<'a> {
    fn new(modules: &'a [&'a CoreModule]) -> Self {
        Self {
            items: Vec::new(),
            modules: Some(modules),
            named: Vec::new(),
        }
    }

    /// Only the original Unit, Bool, Int, String, tuple, Maybe and Either
    /// data codecs.
    fn original() -> Self {
        Self {
            items: Vec::new(),
            modules: None,
            named: Vec::new(),
        }
    }

    fn intern(&mut self, item: ProgramCodec) -> usize {
        if let Some(index) = self.items.iter().position(|c| c == &item) {
            index
        } else {
            self.items.push(item);
            self.items.len() - 1
        }
    }
}

impl std::ops::Deref for Codecs<'_> {
    type Target = Vec<ProgramCodec>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramWasmExport {
    pub name: String,
    pub symbol: String,
    pub source_arity: usize,
    pub value_type: CoreType,
    pub codec: usize,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramWasmMetadata {
    pub schema: u32,
    pub abi: String,
    pub stability: String,
    pub module: String,
    pub modules: Vec<String>,
    pub exports: Vec<ProgramWasmExport>,
    pub codecs: Vec<ProgramCodec>,
    /// The standard runtime areas the linked program uses, such as `text`
    /// or `list`. A standard API without a program WASM proof fails the
    /// build instead of appearing here.
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub trap_policy: String,
    pub max_boundary_bytes: usize,
    /// The most elements one Array or List may hold at the boundary, in
    /// either direction. The loader rejects a longer input before it enters
    /// WASM, so the instance stays usable.
    #[serde(default = "max_sequence_items")]
    pub max_sequence_items: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramWasmArtifact {
    pub source: String,
    pub metadata: ProgramWasmMetadata,
    pub javascript: String,
    pub declarations: String,
}

pub fn emit_program_wasm(
    modules: &[&CoreModule],
    entry_module: &str,
    exports: &[String],
) -> Result<ProgramWasmArtifact, RustBackendError> {
    emit_program_wasm_with_sources(modules, entry_module, exports, Default::default())
}

/// Like [`emit_program_wasm`], with the original files keyed by Core module
/// identity, so a Defect reports its source location.
pub fn emit_program_wasm_with_sources(
    modules: &[&CoreModule],
    entry_module: &str,
    exports: &[String],
    sources: std::collections::BTreeMap<String, crate::RustSourceFile>,
) -> Result<ProgramWasmArtifact, RustBackendError> {
    let module = modules
        .iter()
        .find(|m| m.module == entry_module)
        .ok_or("program WASM: entry module is missing")?;
    if exports.is_empty() {
        return Err("program WASM: select at least one public function".into());
    }
    let mut seen = BTreeSet::new();
    let mut codecs = Codecs::new(modules);
    let mut public = Vec::new();
    let mut effect_abi = false;
    for name in exports {
        if !seen.insert(name) {
            return Err("program WASM: duplicate export selection".into());
        }
        let function = module
            .functions
            .iter()
            .find(|f| f.symbol == format!("{entry_module}::{name}"))
            .ok_or_else(|| {
                if let Some(binding) = module
                    .bindings
                    .iter()
                    .find(|b| b.symbol == format!("{entry_module}::{name}"))
                {
                    project::at(
                        "program WASM: selected value is not a proven source function",
                        &binding.origin,
                        entry_module,
                    )
                } else {
                    RustBackendError::from("program WASM: selected function is missing")
                }
            })?;
        let error = |message: &str| project::at(message, &function.origin, entry_module);
        if function.visibility != seseragi_syntax::Visibility::Public {
            return Err(error("program WASM: selected function must be public"));
        }
        // Type parameters cross the boundary as host values that WASM only
        // holds and returns; a trait constraint would need an instance for
        // them, which the boundary cannot select yet.
        if !function.constraints.is_empty()
            || function.parameters.is_empty()
            || function
                .type_parameters
                .iter()
                .any(|parameter| parameter.arity != 0)
        {
            return Err(error(
                "program WASM: an unconstrained source function with value type parameters is required",
            ));
        }
        let substitutions = function
            .type_parameters
            .iter()
            .map(|parameter| {
                (
                    parameter.name.clone(),
                    CoreType::Named {
                        name: format!("{TYPE_PARAMETER}{}", parameter.name),
                        arguments: Vec::new(),
                    },
                )
            })
            .collect();
        let ty = function
            .value_type
            .as_ref()
            .and_then(|ty| emit::canonical_declared_type(module, ty, &substitutions))
            .ok_or_else(|| error("program WASM: checked public value type is unavailable"))?;
        // Source arity is declaration metadata. A returned function is never
        // silently folded into the declaration's parameter list.
        let mut remainder = &ty;
        for parameter in &function.parameters {
            let CoreType::Function {
                parameter: expected,
                result,
            } = remainder
            else {
                return Err(error(
                    "program WASM: source parameters exceed the checked function type",
                ));
            };
            if !matches!(parameter.kind.as_str(), "named" | "implicit")
                || emit::canonical_declared_type(module, &parameter.type_ref, &substitutions)
                    .as_ref()
                    != Some(expected.as_ref())
                || (parameter.kind == "implicit"
                    && !matches!(expected.as_ref(), CoreType::Named {name, arguments} if name == "std/prelude::Unit" && arguments.is_empty()))
            {
                return Err(error(
                    "program WASM: source parameter identity does not match its checked signature",
                ));
            }
            remainder = result;
        }
        let body_type = emit::checked_declared_body_type(module, &function.body, &substitutions);
        let effect_result = |mut ty: &CoreType| {
            while let CoreType::Function { result, .. } = ty {
                ty = result;
            }
            matches!(ty, CoreType::Named { name, arguments } if name == "std/prelude::Effect" && arguments.len() == 3)
        };
        if body_type.as_ref().map(records_in_name_order) != Some(records_in_name_order(remainder))
            && !(effect_result(remainder)
                && body_type.as_ref().is_some_and(|actual| {
                    crate::program_wasm_effect::boundary_accepts(remainder, actual)
                }))
        {
            return Err(error(
                "program WASM: body result identity disagrees with the checked public signature",
            ));
        }
        let codec = codec(&ty, true, &mut codecs)
            .or_else(|| effect_codec(&ty, &mut codecs))
            .or_else(|| host_codec(&ty, &mut codecs))
            .ok_or_else(|| error("program WASM: unsupported public codec; expected closed Unit/Bool/Int/String, tuple, Maybe/Either, a returned function or providerless Effect"))?;
        let mut terminal = codec;
        while let ProgramCodec::Function { result, .. } = &codecs[terminal] {
            terminal = *result;
        }
        let returns_effect = matches!(codecs[terminal], ProgramCodec::Effect { .. });
        if returns_effect && !function.type_parameters.is_empty() {
            return Err(error(
                "program WASM: generic Effect functions need the Effect ABI's own proof",
            ));
        }
        if function.is_effect && !returns_effect {
            return Err(error(
                "program WASM: checked Effect declaration contract is unavailable",
            ));
        }
        effect_abi |= returns_effect;
        public.push(ProgramWasmExport {
            name: name.clone(),
            symbol: function.symbol.clone(),
            source_arity: function.parameters.len(),
            value_type: ty,
            codec,
            origin: function.origin.clone(),
        });
    }
    let host_abi = codecs
        .iter()
        .any(|code| matches!(code, ProgramCodec::HostEffect { .. }));
    effect_abi |= host_abi;
    // Reuse the ordinary linker and every native evidence/admission check.
    // This explicit library mode changes no native executable entry contract.
    let options = crate::RustTargetOptions {
        sources,
        ..crate::RustTargetOptions::default()
    };
    let mut library = project::emit_library(modules, entry_module, &options)?;
    let tracks_private_refs = library.metadata.capabilities.iter().any(|c| c == "ref");
    if effect_abi {
        // Both resumable ABIs use the same checked data-only cell proof. A
        // private cell never transports a host lease or another callable.
        crate::program_wasm_effect::validate(modules)?;
    } else if library.metadata.capabilities.iter().any(|c| {
        matches!(
            c.as_str(),
            "effect-sync" | "ref" | "fiber" | "deferred" | "filesystem-async" | "console"
        )
    }) {
        let (owner, origin) = modules
            .iter()
            .find_map(|m| {
                m.functions
                    .iter()
                    .find(|f| f.is_effect)
                    .map(|f| (m.module.as_str(), &f.origin))
            })
            .unwrap_or((entry_module, &public[0].origin));
        return Err(project::at("program WASM: Effect, Ref and host operations require a separate unwind/cancellation ABI", origin, owner));
    }
    let metadata = ProgramWasmMetadata {
        schema: 1,
        abi: if host_abi {
            PROGRAM_WASM_HOST_EFFECT_ABI
        } else if effect_abi {
            PROGRAM_WASM_EFFECT_ABI
        } else {
            PROGRAM_WASM_ABI
        }
        .into(),
        stability: "experimental".into(),
        module: entry_module.into(),
        modules: library.metadata.modules,
        exports: public,
        codecs: codecs.items,
        capabilities: {
            let mut capabilities = library.metadata.capabilities.clone();
            capabilities.sort();
            capabilities.dedup();
            capabilities
        },
        trap_policy: "retire-instance".into(),
        max_boundary_bytes: 16 * 1024 * 1024,
        max_sequence_items: max_sequence_items(),
    };
    if effect_abi {
        library.source.push_str(&format!(
            "\nconst PROGRAM_TRACK_REFS: bool = {tracks_private_refs};\n"
        ));
    }
    library.source.push_str(&format!(
        "\nconst PROGRAM_MAX_ITEMS: usize = {};\n",
        metadata.max_sequence_items
    ));
    library.source.push_str(if host_abi {
        include_str!("runtime_program_wasm_host_effect.rs")
    } else if effect_abi {
        include_str!("runtime_program_wasm_effect.rs")
    } else {
        include_str!("runtime_program_wasm.rs")
    });
    let specs = metadata
        .codecs
        .iter()
        .map(rust_codec)
        .collect::<Vec<_>>()
        .join(",");
    library.source.push_str(&format!(
        "\nfn program_codecs() -> Vec<ProgramCode> {{ vec![{specs}] }}\n"
    ));
    library
        .source
        .push_str("fn program_export(index: u32) -> Option<(V, usize)> { match index {\n");
    for (index, export) in metadata.exports.iter().enumerate() {
        let symbol = export
            .symbol
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        library.source.push_str(&format!(
            "{index} => Some((v_{symbol}(vec![]), {})),\n",
            export.codec
        ));
    }
    library.source.push_str("_ => None } }\n");
    let schema = serde_json::to_string(&metadata).expect("serializable checked ABI");
    library.source.push_str(&format!("\nconst PROGRAM_CONTRACT: &str = {schema:?};\n#[no_mangle]\npub extern \"C\" fn ssrg_contract_ptr() -> usize {{ PROGRAM_CONTRACT.as_ptr() as usize }}\n#[no_mangle]\npub extern \"C\" fn ssrg_contract_len() -> usize {{ PROGRAM_CONTRACT.len() }}\n"));
    let quoted_schema = serde_json::to_string(&schema).expect("serializable checked ABI text");
    let javascript = format!("// Generated checked program WASM ABI.\nconst contract = {quoted_schema};\nconst schema = JSON.parse(contract);\n// The standard runtime areas this program uses.\nexport const capabilities = Object.freeze(schema.capabilities.slice());\n{}", if host_abi { include_str!("program_wasm_host_effect.js") } else if effect_abi { include_str!("program_wasm_effect.js") } else { include_str!("program_wasm.js") });
    let declarations = declarations(&metadata);
    Ok(ProgramWasmArtifact {
        source: library.source,
        metadata,
        javascript,
        declarations,
    })
}

fn host_codec(ty: &CoreType, codecs: &mut Codecs) -> Option<usize> {
    let CoreType::Function { parameter, result } = ty else {
        return effect_codec(ty, codecs).or_else(|| codec(ty, true, codecs));
    };
    let parameter =
        codec(parameter, false, codecs).or_else(|| host_parameter_codec(parameter, codecs))?;
    let result = host_codec(result, codecs)?;
    let item = ProgramCodec::Function { parameter, result };
    Some(codecs.intern(item))
}

fn host_parameter_codec(ty: &CoreType, codecs: &mut Codecs) -> Option<usize> {
    // The transport's AbortSignal is out of band. This checked Function is
    // exactly one Seseragi source argument followed by one cold Effect plan.
    let CoreType::Function { parameter, result } = ty else {
        return None;
    };
    let CoreType::Named { name, arguments } = result.as_ref() else {
        return None;
    };
    let [CoreType::Record {
        closed: true,
        fields,
    }, failure, success] = arguments.as_slice()
    else {
        return None;
    };
    if name != "std/prelude::Effect" || !fields.is_empty() {
        return None;
    }
    let parameter = codec(parameter, false, codecs)?;
    let mut terminal = |ty: &CoreType| {
        if matches!(ty, CoreType::Named { name, arguments } if name == "std/prelude::Never" && arguments.is_empty())
        {
            Some(None)
        } else {
            codec(ty, false, codecs).map(Some)
        }
    };
    let failure = terminal(failure)?;
    let success = terminal(success)?;
    let item = ProgramCodec::HostEffect {
        parameter,
        failure,
        success,
    };
    Some(codecs.intern(item))
}

fn effect_codec(ty: &CoreType, codecs: &mut Codecs) -> Option<usize> {
    let item = match ty {
        CoreType::Function { parameter, result } => ProgramCodec::Function {
            parameter: codec(parameter, false, codecs)?,
            result: effect_codec(result, codecs).or_else(|| codec(result, true, codecs))?,
        },
        CoreType::Named { name, arguments } if name == "std/prelude::Effect" => {
            let [requirements, failure, success] = arguments.as_slice() else {
                return None;
            };
            if !matches!(requirements, CoreType::Record { closed: true, fields } if fields.is_empty())
            {
                return None;
            }
            let terminal = |ty: &CoreType, callable, codecs: &mut Codecs| {
                if matches!(ty, CoreType::Named { name, arguments } if name == "std/prelude::Never" && arguments.is_empty())
                {
                    Some(None)
                } else {
                    codec(ty, callable, codecs)
                        // Success values may own a returned callback whose
                        // checked result is another closed Effect. Reuse the
                        // same recursive proof; inputs and failures still use
                        // their non-callable value-only codec boundary.
                        .or_else(|| callable.then(|| effect_codec(ty, codecs)).flatten())
                        .map(Some)
                }
            };
            ProgramCodec::Effect {
                failure: terminal(failure, false, codecs)?,
                success: terminal(success, true, codecs)?,
            }
        }
        _ => return None,
    };
    Some(codecs.intern(item))
}

pub(crate) fn has_closed_data_codec(ty: &CoreType) -> bool {
    // Internal cells reuse the exact existing data fragment. This creates no
    // Ref transport codec and admits neither callbacks nor Effect values.
    codec(ty, false, &mut Codecs::original()).is_some()
}

fn codec(ty: &CoreType, callable: bool, codecs: &mut Codecs) -> Option<usize> {
    let item = match ty {
        CoreType::Named { name, arguments } => match (name.as_str(), arguments.as_slice()) {
            ("std/prelude::Unit", []) => ProgramCodec::Unit,
            ("std/prelude::Bool", []) => ProgramCodec::Bool,
            ("std/prelude::Int", []) => ProgramCodec::Int,
            ("std/prelude::String", []) => ProgramCodec::String,
            ("std/prelude::Float", []) if codecs.modules.is_some() => ProgramCodec::Float,
            ("std/prelude::Char", []) if codecs.modules.is_some() => ProgramCodec::Char,
            ("std/prelude::Maybe", [value]) => ProgramCodec::Maybe {
                value: codec(value, false, codecs)?,
            },
            ("std/prelude::Either", [left, right]) => ProgramCodec::Either {
                left: codec(left, false, codecs)?,
                right: codec(right, false, codecs)?,
            },
            ("std/prelude::Array", [element]) if codecs.modules.is_some() => ProgramCodec::Array {
                element: codec(element, false, codecs)?,
            },
            ("std/prelude::List", [element]) if codecs.modules.is_some() => ProgramCodec::List {
                element: codec(element, false, codecs)?,
            },
            _ if codecs.modules.is_some()
                && name.starts_with(TYPE_PARAMETER)
                && arguments.is_empty() =>
            {
                ProgramCodec::Parameter {
                    name: name[TYPE_PARAMETER.len()..].to_owned(),
                }
            }
            _ => return named_codec(ty, codecs),
        },
        CoreType::Record {
            closed: true,
            fields,
        } if codecs.modules.is_some() => {
            let mut sorted = fields.iter().collect::<Vec<_>>();
            sorted.sort_by(|left, right| left.name.cmp(&right.name));
            if sorted.iter().any(|field| field.optional)
                || sorted.windows(2).any(|pair| pair[0].name == pair[1].name)
            {
                return None;
            }
            ProgramCodec::Record {
                name: None,
                fields: sorted
                    .into_iter()
                    .map(|field| {
                        Some(ProgramField {
                            name: field.name.clone(),
                            codec: codec(&field.type_ref, false, codecs)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
            }
        }
        CoreType::Tuple { elements } if (2..=32).contains(&elements.len()) => ProgramCodec::Tuple {
            elements: elements
                .iter()
                .map(|e| codec(e, false, codecs))
                .collect::<Option<Vec<_>>>()?,
        },
        CoreType::Function { parameter, result } if callable => ProgramCodec::Function {
            parameter: codec(parameter, false, codecs)?,
            result: codec(result, true, codecs)?,
        },
        _ => return None,
    };
    Some(codecs.intern(item))
}

/// `ty` with every record's fields in name order. Records are structural,
/// so a record literal and its declared type can list fields differently.
fn records_in_name_order(ty: &CoreType) -> CoreType {
    let recur = |ty: &CoreType| Box::new(records_in_name_order(ty));
    match ty {
        CoreType::Record { closed, fields } => {
            let mut fields = fields
                .iter()
                .map(|field| seseragi_lowering::CoreRecordField {
                    type_ref: records_in_name_order(&field.type_ref),
                    ..field.clone()
                })
                .collect::<Vec<_>>();
            fields.sort_by(|left, right| left.name.cmp(&right.name));
            CoreType::Record {
                closed: *closed,
                fields,
            }
        }
        CoreType::Named { name, arguments } => CoreType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(records_in_name_order).collect(),
        },
        CoreType::Tuple { elements } => CoreType::Tuple {
            elements: elements.iter().map(records_in_name_order).collect(),
        },
        CoreType::Function { parameter, result } => CoreType::Function {
            parameter: recur(parameter),
            result: recur(result),
        },
        other => other.clone(),
    }
}

fn mentions_type_parameter(ty: &CoreType) -> bool {
    match ty {
        CoreType::Named { name, arguments } => {
            name.starts_with(TYPE_PARAMETER) || arguments.iter().any(mentions_type_parameter)
        }
        CoreType::Tuple { elements } => elements.iter().any(mentions_type_parameter),
        CoreType::Function { parameter, result } => {
            mentions_type_parameter(parameter) || mentions_type_parameter(result)
        }
        CoreType::Record { fields, .. } => fields
            .iter()
            .any(|field| mentions_type_parameter(&field.type_ref)),
        _ => false,
    }
}

/// The codec of an ADT or struct declared in the linked modules. Opaque types
/// and newtypes keep their representation private, so they are not data
/// codecs. The codec is registered before its fields so that a recursive
/// type refers to its own index.
fn named_codec(ty: &CoreType, codecs: &mut Codecs) -> Option<usize> {
    let modules = codecs.modules?;
    let CoreType::Named { name, arguments } = ty else {
        return None;
    };
    if let Some((_, index)) = codecs.named.iter().find(|(named, _)| named == ty) {
        return Some(*index);
    }
    // A named type over a type parameter would need a generic declaration.
    if arguments.iter().any(mentions_type_parameter) {
        return None;
    }
    let (owner, adt, structure) = modules.iter().find_map(|module| {
        module
            .adts
            .iter()
            .find(|adt| &adt.symbol == name)
            .map(|adt| (*module, Some(adt), None))
            .or_else(|| {
                module
                    .structs
                    .iter()
                    .find(|structure| &structure.symbol == name)
                    .map(|structure| (*module, None, Some(structure)))
            })
    })?;
    let parameters = adt
        .map(|adt| &adt.type_parameters)
        .or_else(|| structure.map(|structure| &structure.type_parameters))?;
    if adt.is_some_and(|adt| adt.opaque || adt.newtype)
        || structure.is_some_and(|structure| structure.opaque)
        || parameters.len() != arguments.len()
    {
        return None;
    }
    let substitutions = parameters
        .iter()
        .zip(arguments)
        .map(|(parameter, argument)| (parameter.name.clone(), argument.clone()))
        .collect();
    let declared = |ty: &CoreType| emit::canonical_declared_type(owner, ty, &substitutions);
    let index = codecs.items.len();
    let named = codecs.named.len();
    codecs.items.push(ProgramCodec::Adt {
        name: "\0pending".into(),
        variants: Vec::new(),
    });
    codecs.named.push((ty.clone(), index));
    let item = if let Some(adt) = adt {
        adt.variants
            .iter()
            .map(|variant| {
                Some(ProgramVariant {
                    tag: variant.name.clone(),
                    owner: variant.symbol.clone(),
                    payload: match &variant.payload {
                        Some(payload) => Some(codec(&declared(payload)?, false, codecs)?),
                        None => None,
                    },
                })
            })
            .collect::<Option<Vec<_>>>()
            .map(|variants| ProgramCodec::Adt {
                name: adt.name.clone(),
                variants,
            })
    } else {
        let structure = structure.expect("a named codec is an ADT or a struct");
        let mut fields = structure.fields.iter().collect::<Vec<_>>();
        fields.sort_by(|left, right| left.name.cmp(&right.name));
        fields
            .into_iter()
            .map(|field| {
                Some(ProgramField {
                    name: field.name.clone(),
                    codec: codec(&declared(&field.type_ref)?, false, codecs)?,
                })
            })
            .collect::<Option<Vec<_>>>()
            .map(|fields| ProgramCodec::Record {
                fields,
                name: Some(structure.name.clone()),
            })
    };
    match item {
        Some(item) => {
            codecs.items[index] = item;
            Some(index)
        }
        None => {
            codecs.items.truncate(index);
            codecs.named.truncate(named);
            None
        }
    }
}

fn rust_codec(codec: &ProgramCodec) -> String {
    match codec {
        ProgramCodec::Unit => "ProgramCode::Unit".into(),
        ProgramCodec::Bool => "ProgramCode::Bool".into(),
        ProgramCodec::Int => "ProgramCode::Int".into(),
        ProgramCodec::String => "ProgramCode::Text".into(),
        ProgramCodec::Float => "ProgramCode::Float".into(),
        ProgramCodec::Char => "ProgramCode::Char".into(),
        ProgramCodec::Tuple { elements } => format!("ProgramCode::Tuple(vec!{:?})", elements),
        ProgramCodec::Maybe { value } => format!("ProgramCode::Maybe({value})"),
        ProgramCodec::Either { left, right } => format!("ProgramCode::Either({left}, {right})"),
        ProgramCodec::Function { parameter, result } => {
            format!("ProgramCode::Function({parameter}, {result})")
        }
        ProgramCodec::Effect { failure, success } => {
            format!("ProgramCode::Effect({failure:?}, {success:?})")
        }
        ProgramCodec::HostEffect {
            parameter,
            failure,
            success,
        } => format!("ProgramCode::HostEffect({parameter}, {failure:?}, {success:?})"),
        ProgramCodec::Array { element } => format!("ProgramCode::Array({element})"),
        ProgramCodec::Parameter { .. } => "ProgramCode::Parameter".into(),
        ProgramCodec::List { element } => format!("ProgramCode::List({element})"),
        ProgramCodec::Record { fields, .. } => format!(
            "ProgramCode::Record(vec![{}])",
            fields
                .iter()
                .map(|field| format!("({:?}, {})", field.name, field.codec))
                .collect::<Vec<_>>()
                .join(",")
        ),
        ProgramCodec::Adt { variants, .. } => format!(
            "ProgramCode::Adt(vec![{}])",
            variants
                .iter()
                .map(|variant| format!("({:?}, {:?})", variant.owner, variant.payload))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

/// The TS type names of ADT and struct codecs. These types can be
/// recursive, so each is declared once by name instead of being expanded
/// where it is used. A source name becomes the TS name only when it is a
/// plain identifier used by no other codec and does not shadow a TS global
/// or a name these declarations use; otherwise the codec gets
/// `ProgramData<index>`.
fn type_aliases(metadata: &ProgramWasmMetadata) -> std::collections::BTreeMap<usize, String> {
    const RESERVED: &[&str] = &[
        // TS globals the declarations refer to.
        "AbortSignal",
        "BufferSource",
        "Error",
        "Promise",
        "PromiseLike",
        "Readonly",
        "ReadonlyArray",
        // Names the declarations define.
        "Ownership",
        "OwnedCallback",
        "OwnedEffect",
        "HostResult",
        "HostEffect",
        "HostEffectContext",
    ];
    let names = metadata
        .codecs
        .iter()
        .enumerate()
        .filter_map(|(index, codec)| match codec {
            ProgramCodec::Adt { name, .. }
            | ProgramCodec::Record {
                name: Some(name), ..
            } => Some((index, name.as_str())),
            _ => None,
        })
        .collect::<Vec<_>>();
    names
        .iter()
        .map(|(index, name)| {
            let unique = names.iter().filter(|(_, other)| other == name).count() == 1;
            let identifier = name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            // A type parameter of an export is written by its name too.
            let parameter = metadata.codecs.iter().any(|codec| {
                matches!(codec, ProgramCodec::Parameter { name: parameter } if parameter == name)
            });
            let alias = if unique
                && identifier
                && !parameter
                && !RESERVED.contains(name)
                && !name.starts_with("Program")
            {
                (*name).to_owned()
            } else {
                format!("ProgramData{index}")
            };
            (*index, alias)
        })
        .collect()
}

fn declared_type(
    metadata: &ProgramWasmMetadata,
    aliases: &std::collections::BTreeMap<usize, String>,
    index: usize,
) -> String {
    let ty = |index| declared_type(metadata, aliases, index);
    match &metadata.codecs[index] {
        ProgramCodec::Unit => "undefined".into(),
        ProgramCodec::Bool => "boolean".into(),
        ProgramCodec::Int => "number".into(),
        ProgramCodec::String | ProgramCodec::Char => "string".into(),
        ProgramCodec::Float => "number".into(),
        ProgramCodec::Tuple { elements } => format!(
            "readonly [{}]",
            elements
                .iter()
                .map(|i| ty(*i))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ProgramCodec::Maybe { value } => format!(
            "Readonly<{{tag: 'Nothing'}}> | Readonly<{{tag: 'Just'; value: {}}}>",
            ty(*value)
        ),
        ProgramCodec::Either { left, right } => format!(
            "Readonly<{{tag: 'Left'; value: {}}}> | Readonly<{{tag: 'Right'; value: {}}}>",
            ty(*left),
            ty(*right)
        ),
        ProgramCodec::Function { parameter, result } => {
            format!("OwnedCallback<{}, {}>", ty(*parameter), ty(*result))
        }
        ProgramCodec::Effect { failure, success } => format!(
            "OwnedEffect<{}, {}>",
            failure.map(ty).unwrap_or_else(|| "never".into()),
            success.map(ty).unwrap_or_else(|| "never".into())
        ),
        ProgramCodec::HostEffect {
            parameter,
            failure,
            success,
        } => format!(
            "HostEffect<{}, {}, {}>",
            ty(*parameter),
            failure.map(ty).unwrap_or_else(|| "never".into()),
            success.map(ty).unwrap_or_else(|| "never".into()),
        ),
        ProgramCodec::Array { element } => format!("ReadonlyArray<{}>", ty(*element)),
        ProgramCodec::List { element } => format!("ProgramList<{}>", ty(*element)),
        ProgramCodec::Parameter { name } => name.clone(),
        ProgramCodec::Record { name: Some(_), .. } | ProgramCodec::Adt { .. } => {
            aliases[&index].clone()
        }
        ProgramCodec::Record { fields, .. } => record_type(metadata, aliases, fields),
    }
}

/// A callback that mentions type parameters. Each call binds the type
/// parameters that first appear in its argument, so TS can infer them; the
/// last call also binds any that appear only in its result.
fn generic_callback_type(
    metadata: &ProgramWasmMetadata,
    aliases: &std::collections::BTreeMap<usize, String>,
    index: usize,
    bound: &[String],
) -> String {
    let ProgramCodec::Function { parameter, result } = &metadata.codecs[index] else {
        return declared_type(metadata, aliases, index);
    };
    let callable = matches!(metadata.codecs[*result], ProgramCodec::Function { .. });
    let mut binding = type_parameters(metadata, *parameter);
    if !callable {
        binding.extend(type_parameters(metadata, *result));
    }
    let mut seen = Vec::new();
    binding.retain(|name| {
        !bound.contains(name) && {
            let first = !seen.contains(name);
            seen.push(name.clone());
            first
        }
    });
    let mut inner = bound.to_vec();
    inner.extend(binding.iter().cloned());
    let result = if callable
        && !type_parameters(metadata, *result)
            .iter()
            .all(|name| inner.contains(name))
    {
        generic_callback_type(metadata, aliases, *result, &inner)
    } else {
        declared_type(metadata, aliases, *result)
    };
    let generics = if binding.is_empty() {
        String::new()
    } else {
        format!("<{}>", binding.join(", "))
    };
    format!(
        "{{{generics}(argument: {}): {result}; dispose(): void; readonly disposed: boolean}}",
        declared_type(metadata, aliases, *parameter),
    )
}

/// The type parameters an export's codec mentions, in first-use order.
fn type_parameters(metadata: &ProgramWasmMetadata, root: usize) -> Vec<String> {
    let mut found = Vec::new();
    let mut visited = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(index) = pending.pop() {
        if !visited.insert(index) {
            continue;
        }
        match &metadata.codecs[index] {
            ProgramCodec::Parameter { name } => {
                if !found.contains(name) {
                    found.push(name.clone());
                }
            }
            ProgramCodec::Tuple { elements } => pending.extend(elements.iter().rev()),
            ProgramCodec::Maybe { value } => pending.push(*value),
            ProgramCodec::Either { left, right } => pending.extend([*right, *left]),
            ProgramCodec::Function { parameter, result } => pending.extend([*result, *parameter]),
            ProgramCodec::Array { element } | ProgramCodec::List { element } => {
                pending.push(*element)
            }
            ProgramCodec::Record { fields, name: None } => {
                pending.extend(fields.iter().rev().map(|field| field.codec))
            }
            _ => {}
        }
    }
    found
}

fn record_type(
    metadata: &ProgramWasmMetadata,
    aliases: &std::collections::BTreeMap<usize, String>,
    fields: &[ProgramField],
) -> String {
    format!(
        "Readonly<{{{}}}>",
        fields
            .iter()
            .map(|field| format!(
                "{}: {}",
                serde_json::to_string(&field.name).unwrap(),
                declared_type(metadata, aliases, field.codec)
            ))
            .collect::<Vec<_>>()
            .join("; ")
    )
}

fn declarations(metadata: &ProgramWasmMetadata) -> String {
    let aliases = type_aliases(metadata);
    let mut result = String::from("export declare const capabilities: readonly string[];\nexport interface Ownership { readonly alive: boolean; readonly retired: boolean; readonly callbacks: number; readonly managedLiveAtClose: number | null; }\nexport interface OwnedCallback<A, B> { (argument: A): B; dispose(): void; readonly disposed: boolean; }\nexport class ProgramWasmDefect extends Error { readonly origin?: Readonly<{source: string; start: number; end: number}>; readonly kind?: 'defect' | 'trap'; readonly location?: Readonly<{file: string; line: number; column: number}>; }\nexport class ProgramWasmClosed extends Error {}\nexport interface ProgramExports {\n");
    if metadata
        .codecs
        .iter()
        .any(|codec| matches!(codec, ProgramCodec::List { .. }))
    {
        result.insert_str(0, "export type ProgramList<A> = Readonly<{tag: 'Empty'}> | Readonly<{tag: 'Cons'; head: A; tail: ProgramList<A>}>;\n");
    }
    for (index, alias) in &aliases {
        let body = match &metadata.codecs[*index] {
            ProgramCodec::Adt { variants, .. } if variants.is_empty() => "never".to_owned(),
            ProgramCodec::Adt { variants, .. } => variants
                .iter()
                .map(|variant| {
                    let tag = serde_json::to_string(&variant.tag).unwrap();
                    match variant.payload {
                        Some(payload) => format!(
                            "Readonly<{{tag: {tag}; value: {}}}>",
                            declared_type(metadata, &aliases, payload)
                        ),
                        None => format!("Readonly<{{tag: {tag}}}>"),
                    }
                })
                .collect::<Vec<_>>()
                .join(" | "),
            ProgramCodec::Record { fields, .. } => record_type(metadata, &aliases, fields),
            _ => unreachable!("aliases name ADT and struct codecs"),
        };
        result.insert_str(0, &format!("export type {alias} = {body};\n"));
    }
    for export in &metadata.exports {
        let ty = if type_parameters(metadata, export.codec).is_empty() {
            declared_type(metadata, &aliases, export.codec)
        } else {
            generic_callback_type(metadata, &aliases, export.codec, &[])
        };
        result.push_str(&format!(
            "readonly {}: {ty};\n",
            serde_json::to_string(&export.name).unwrap(),
        ));
    }
    if metadata.abi == PROGRAM_WASM_HOST_EFFECT_ABI {
        result.push_str("}\nexport type HostResult<E, A> = Readonly<{tag: 'Success'; value: A}> | Readonly<{tag: 'Failure'; error: E}> | Readonly<{tag: 'Defect'; messages: readonly string[]}>;\nexport interface HostEffectContext { readonly signal: AbortSignal; }\nexport type HostEffect<A, E, B> = (argument: A, context: HostEffectContext) => HostResult<E, B> | PromiseLike<HostResult<E, B>>;\nexport type ProgramLocation = Readonly<{file: string; line: number; column: number}>;\nexport type ProgramOutcome<E, A> = Readonly<{tag: 'Success'; value: A}> | Readonly<{tag: 'Failure'; error: E; location?: ProgramLocation}> | Readonly<{tag: 'Defect'; messages: readonly string[]; locations?: readonly (ProgramLocation | null)[]}> | Readonly<{tag: 'Cancelled'}>;\nexport interface ProgramExecution<E, A> { readonly result: Promise<ProgramOutcome<E, A>>; cancel(): void; }\nexport interface OwnedEffect<E, A> { run(): ProgramExecution<E, A>; dispose(): void; readonly disposed: boolean; }\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): Promise<void>; ownership(): Ownership & Readonly<{plans: number; running: boolean; closing: boolean; hostCallbacks: number; waitingHost: boolean}>; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    } else if metadata.abi == PROGRAM_WASM_EFFECT_ABI {
        result.push_str("}\nexport type ProgramLocation = Readonly<{file: string; line: number; column: number}>;\nexport type ProgramOutcome<E, A> = Readonly<{tag: 'Success'; value: A}> | Readonly<{tag: 'Failure'; error: E; location?: ProgramLocation}> | Readonly<{tag: 'Defect'; messages: readonly string[]; locations?: readonly (ProgramLocation | null)[]}> | Readonly<{tag: 'Cancelled'}>;\nexport interface ProgramExecution<E, A> { readonly result: Promise<ProgramOutcome<E, A>>; cancel(): void; }\nexport interface OwnedEffect<E, A> { run(): ProgramExecution<E, A>; dispose(): void; readonly disposed: boolean; }\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): Promise<void>; ownership(): Ownership & Readonly<{plans: number; running: boolean; closing: boolean}>; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    } else {
        result.push_str("}\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): void; ownership(): Ownership; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    }
    result
}
