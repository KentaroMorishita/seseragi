//! A closed, pure public ABI over checked Core. It is deliberately distinct
//! from the TypeScript foreign ABI and from the compiler WASM adapter.
use crate::{emit, project, RustBackendError};
use serde::{Deserialize, Serialize};
use seseragi_lowering::{CoreModule, CoreType, SourceSpan};
use std::collections::BTreeSet;

pub const PROGRAM_WASM_ABI: &str = "seseragi-program-wasm-pure-v1";
pub const PROGRAM_WASM_EFFECT_ABI: &str = "seseragi-program-wasm-effect-v3";
pub const PROGRAM_WASM_HOST_EFFECT_ABI: &str = "seseragi-program-wasm-host-effect-v3";

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
    /// A JS `bigint`, as on the TS target. It crosses as decimal text.
    BigInt,
    /// The TS target's frozen `{coefficient, scale}` of `bigint`s, in the
    /// canonical form both targets keep. It crosses as two decimal texts.
    Decimal,
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
        /// The environment fields a run must be given, such as `console`.
        /// Only the host Effect ABI carries provider requests.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        services: Vec<String>,
    },
    /// A host function `A -> B -> ... -> Effect<{}, E, R>`, called once
    /// all its source arguments are applied and run by the loader.
    HostEffect {
        parameters: Vec<usize>,
        failure: Option<usize>,
        success: Option<usize>,
    },
    /// A synchronous host function `A -> B -> ... -> R` over closed values.
    /// WASM calls it through an import once all its arguments are applied.
    HostFunction {
        parameters: Vec<usize>,
        result: usize,
    },
    Array {
        element: usize,
    },
    List {
        element: usize,
    },
    /// A closed record or a struct, with fields in name order. A struct
    /// keeps its name so a recursive struct can be declared by name. An
    /// opaque struct crosses with its fields, as on the TS target, but its
    /// declared type hides them behind a brand.
    Record {
        fields: Vec<ProgramField>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        opaque: bool,
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
    /// A newtype passed as its payload alone, as the TS target's release
    /// profile passes it. WASM keeps the wrapper of `owner`.
    Newtype {
        name: String,
        owner: String,
        value: usize,
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

/// How many times one named type may be nested in its own codec with larger
/// or unrelated type arguments before the expansion counts as unbounded.
const MAX_NESTED_TYPE_EXPANSION: usize = 16;

/// The prefix of a type parameter's name in a checked public type.
pub(crate) const TYPE_PARAMETER: &str = "\0type-parameter::";

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
    /// The named types whose codecs are being built, outermost first.
    building: Vec<CoreType>,
    /// The TS target's release profile erases newtype wrappers.
    erase_newtypes: bool,
}

impl<'a> Codecs<'a> {
    fn new(modules: &'a [&'a CoreModule], erase_newtypes: bool) -> Self {
        Self {
            items: Vec::new(),
            modules: Some(modules),
            named: Vec::new(),
            building: Vec::new(),
            erase_newtypes,
        }
    }

    /// Only the original Unit, Bool, Int, String, tuple, Maybe and Either
    /// data codecs.
    fn original() -> Self {
        Self {
            items: Vec::new(),
            modules: None,
            named: Vec::new(),
            building: Vec::new(),
            erase_newtypes: false,
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
    emit_program_wasm_with_options(
        modules,
        entry_module,
        exports,
        sources,
        ProgramWasmOptions::default(),
    )
}

/// Choices that follow the TS target the program is paired with.
#[derive(Clone, Debug, Default)]
pub struct ProgramWasmOptions {
    /// Pass a newtype as its payload alone, as the TS target's release
    /// profile does. The development profile keeps `{tag, value}`.
    pub erase_newtypes: bool,
    /// Exports of generic functions at fixed type arguments.
    pub instantiations: Vec<ProgramWasmInstantiation>,
}

/// An export of a generic function at fixed type arguments, such as
/// `--export label<Int>`. The caller adds a value binding of the explicit
/// application (`let hidden = label<Int>`) to the entry module before
/// checking, so the type checker selects the trait instances as it does for
/// any other call. WASM then exports that binding.
#[derive(Clone, Debug)]
pub struct ProgramWasmInstantiation {
    /// The name the host sees.
    pub name: String,
    /// The entry module binding that holds the application.
    pub binding: String,
    /// The generic source function, which gives the source arity.
    pub function: String,
}

/// Like [`emit_program_wasm_with_sources`], for a chosen TS target profile.
pub fn emit_program_wasm_with_options(
    modules: &[&CoreModule],
    entry_module: &str,
    exports: &[String],
    sources: std::collections::BTreeMap<String, crate::RustSourceFile>,
    options: ProgramWasmOptions,
) -> Result<ProgramWasmArtifact, RustBackendError> {
    let module = modules
        .iter()
        .find(|m| m.module == entry_module)
        .ok_or("program WASM: entry module is missing")?;
    if exports.is_empty() {
        return Err("program WASM: select at least one public function".into());
    }
    let mut seen = BTreeSet::new();
    let mut codecs = Codecs::new(modules, options.erase_newtypes);
    let mut public = Vec::new();
    let mut effect_abi = false;
    for name in exports {
        if !seen.insert(name) {
            return Err("program WASM: duplicate export selection".into());
        }
        if let Some(instance) = options.instantiations.iter().find(|i| &i.name == name) {
            let export = instantiated_export(module, entry_module, instance, &mut codecs)?;
            let mut terminal = export.codec;
            while let ProgramCodec::Function { result, .. } = &codecs[terminal] {
                terminal = *result;
            }
            effect_abi |= matches!(codecs[terminal], ProgramCodec::Effect { .. });
            public.push(export);
            continue;
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
            .ok_or_else(|| error("program WASM: unsupported public codec; expected a type built from Unit, Bool, Int, Float, Char, String, tuples, Array, List, closed records, structs, ADTs and newtypes with a finite expansion, type parameters, functions or an Effect that needs no provider other than Console"))?;
        let mut terminal = codec;
        while let ProgramCodec::Function { result, .. } = &codecs[terminal] {
            terminal = *result;
        }
        let returns_effect = matches!(codecs[terminal], ProgramCodec::Effect { .. });
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
    // A provider request suspends a run as a host Effect does.
    let host_abi = codecs.iter().any(|code| match code {
        ProgramCodec::HostEffect { .. } | ProgramCodec::HostFunction { .. } => true,
        ProgramCodec::Effect { services, .. } => !services.is_empty(),
        _ => false,
    });
    effect_abi |= host_abi;
    // Reuse the ordinary linker and every native evidence/admission check.
    // This explicit library mode changes no native executable entry contract.
    let options = crate::RustTargetOptions {
        sources,
        ..crate::RustTargetOptions::default()
    };
    let mut library = project::emit_library(modules, entry_module, &options)?;
    // Cells, Fibers and Deferreds can form cycles, so tracing starts before
    // any of them exists.
    let tracks_private_refs = library
        .metadata
        .capabilities
        .iter()
        .any(|c| matches!(c.as_str(), "ref" | "fiber" | "deferred"));
    if effect_abi {
        // Both resumable ABIs use the same checked data-only cell proof. A
        // private cell never transports a host lease or another callable.
        // Only the providerless ABI holds a type parameter's host value; a
        // host Effect run has no proof for passing one through a callback.
        if host_abi {
            if let Some(function) = public.iter().find_map(|export| {
                module
                    .functions
                    .iter()
                    .find(|f| f.symbol == export.symbol && !f.type_parameters.is_empty())
            }) {
                return Err(project::at(
                    "program WASM: generic functions need the providerless Effect ABI",
                    &function.origin,
                    entry_module,
                ));
            }
        }
        // Fibers share one host-request slot only in the providerless ABI.
        crate::program_wasm_effect::validate(modules, !host_abi, host_abi)?;
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
        // An instantiated export is a checked value binding.
        let value = match library.globals.get(&export.symbol) {
            Some(slot) => format!("get_global({slot})"),
            None => format!("v_{symbol}(vec![])"),
        };
        library
            .source
            .push_str(&format!("{index} => Some(({value}, {})),\n", export.codec));
    }
    library.source.push_str("_ => None } }\n");
    let schema = serde_json::to_string(&metadata).expect("serializable checked ABI");
    library.source.push_str(&format!("\nconst PROGRAM_CONTRACT: &str = {schema:?};\n#[no_mangle]\npub extern \"C\" fn ssrg_contract_ptr() -> usize {{ PROGRAM_CONTRACT.as_ptr() as usize }}\n#[no_mangle]\npub extern \"C\" fn ssrg_contract_len() -> usize {{ PROGRAM_CONTRACT.len() }}\n"));
    // The loader holds the contract without source spans and reads them from
    // the module, so a rebuilt module whose declarations only moved keeps the
    // same loader and `seseragi dev` can swap it in place.
    let mut identity = serde_json::to_value(&metadata).expect("serializable checked ABI");
    for export in identity["exports"].as_array_mut().into_iter().flatten() {
        export.as_object_mut().map(|fields| fields.remove("origin"));
    }
    let identity = serde_json::to_string(&identity).expect("serializable checked ABI");
    let quoted_schema = serde_json::to_string(&identity).expect("serializable checked ABI text");
    let javascript = format!("// Generated checked program WASM ABI.\nconst contract = {quoted_schema};\nconst schema = JSON.parse(contract);\n// The standard runtime areas this program uses.\nexport const capabilities = Object.freeze(schema.capabilities.slice());\n{CONTRACT_ORIGINS}{}", if host_abi { include_str!("program_wasm_host_effect.js") } else if effect_abi { include_str!("program_wasm_effect.js") } else { include_str!("program_wasm.js") });
    let declarations = declarations(&metadata);
    Ok(ProgramWasmArtifact {
        source: library.source,
        metadata,
        javascript,
        declarations,
    })
}

/// Checks the module's embedded contract against the loader's and returns
/// each export's source span from it.
const CONTRACT_ORIGINS: &str = r#"function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`
  if (value !== null && typeof value === "object")
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(",")}}`
  return JSON.stringify(value)
}
const identity = canonical(schema)
function contractOrigins(text) {
  let embedded
  try {
    embedded = JSON.parse(text)
  } catch {
    return undefined
  }
  if (!Array.isArray(embedded?.exports)) return undefined
  const origins = []
  for (const item of embedded.exports) {
    if (item === null || typeof item !== "object") return undefined
    origins.push(Object.freeze(item.origin))
    delete item.origin
  }
  return canonical(embedded) === identity ? Object.freeze(origins) : undefined
}
"#;

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
    // The transport's AbortSignal is out of band. A host function takes its
    // Seseragi source arguments one by one and then gives either a cold
    // Effect plan or, for a synchronous host function, a closed value.
    let mut parameters = Vec::new();
    let mut result = ty;
    while let CoreType::Function {
        parameter,
        result: next,
    } = result
    {
        parameters.push(parameter.as_ref());
        result = next.as_ref();
    }
    if parameters.is_empty() {
        return None;
    }
    let parameters = parameters
        .into_iter()
        .map(|parameter| codec(parameter, false, codecs))
        .collect::<Option<Vec<_>>>()?;
    let CoreType::Named { name, arguments } = result else {
        let result = codec(result, false, codecs)?;
        return Some(codecs.intern(ProgramCodec::HostFunction { parameters, result }));
    };
    if name != "std/prelude::Effect" {
        let result = codec(result, false, codecs)?;
        return Some(codecs.intern(ProgramCodec::HostFunction { parameters, result }));
    }
    let [CoreType::Record {
        closed: true,
        fields,
    }, failure, success] = arguments.as_slice()
    else {
        return None;
    };
    if !fields.is_empty() {
        return None;
    }
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
        parameters,
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
            let services = provider_services(requirements)?;
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
                services,
            }
        }
        _ => return None,
    };
    Some(codecs.intern(item))
}

/// The providers program WASM can bridge, by environment field and service.
pub(crate) const PROVIDERS: &[(&str, &str)] = &[("console", "std/prelude::Console")];

/// The environment fields of a requirement record that every bridged
/// provider covers, in name order; `None` for any other requirement.
pub(crate) fn provider_services(requirements: &CoreType) -> Option<Vec<String>> {
    let CoreType::Record {
        closed: true,
        fields,
    } = requirements
    else {
        return None;
    };
    let mut services = fields
        .iter()
        .map(|field| {
            let CoreType::Named { name, arguments } = &field.type_ref else {
                return None;
            };
            (!field.optional
                && arguments.is_empty()
                && PROVIDERS.contains(&(field.name.as_str(), name.as_str())))
            .then(|| field.name.clone())
        })
        .collect::<Option<Vec<_>>>()?;
    services.sort();
    services.dedup();
    (services.len() == fields.len()).then_some(services)
}

fn provider_type(service: &str) -> &'static str {
    match service {
        "console" => "ProgramConsole",
        _ => unreachable!("only bridged providers have services"),
    }
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
            ("std/big-int::BigInt", []) if codecs.modules.is_some() => ProgramCodec::BigInt,
            ("std/decimal::Decimal", []) if codecs.modules.is_some() => ProgramCodec::Decimal,
            ("std/prelude::Maybe", [value]) => ProgramCodec::Maybe {
                value: codec(value, false, codecs)?,
            },
            // The Console provider's failure, as `{kind, message}` on both
            // targets.
            ("std/prelude::ConsoleError", []) if codecs.modules.is_some() => {
                let text = codecs.intern(ProgramCodec::String);
                ProgramCodec::Record {
                    name: None,
                    opaque: false,
                    fields: ["kind", "message"]
                        .into_iter()
                        .map(|name| ProgramField {
                            name: name.into(),
                            codec: text,
                        })
                        .collect(),
                }
            }
            ("std/prelude::Either", [left, right]) => ProgramCodec::Either {
                left: codec(left, false, codecs)?,
                right: codec(right, false, codecs)?,
            },
            // A Fiber's exit crosses in the TS target's tagged shape. A side
            // of type Never has no values, so its variant never crosses.
            ("std/effect::FiberExit", [failure, success]) if codecs.modules.is_some() => {
                let never = |ty: &CoreType| {
                    matches!(ty, CoreType::Named { name, arguments }
                        if name == "std/prelude::Never" && arguments.is_empty())
                };
                let mut variants = Vec::new();
                for (tag, payload) in [("FiberSucceeded", success), ("FiberFailed", failure)] {
                    if !never(payload) {
                        variants.push(ProgramVariant {
                            tag: tag.to_owned(),
                            owner: format!("std/effect::{tag}"),
                            payload: Some(codec(payload, false, codecs)?),
                        });
                    }
                }
                variants.push(ProgramVariant {
                    tag: "FiberCancelled".to_owned(),
                    owner: "std/effect::FiberCancelled".to_owned(),
                    payload: None,
                });
                ProgramCodec::Adt {
                    name: "FiberExit".to_owned(),
                    variants,
                }
            }
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
                opaque: false,
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

/// Whether `part` occurs in `whole` below its root.
fn contains_strictly(whole: &CoreType, part: &CoreType) -> bool {
    let children: Vec<&CoreType> = match whole {
        CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
            arguments.iter().collect()
        }
        CoreType::Tuple { elements } => elements.iter().collect(),
        CoreType::Record { fields, .. } => fields.iter().map(|field| &field.type_ref).collect(),
        CoreType::Function { parameter, result } => vec![parameter, result],
        _ => Vec::new(),
    };
    children
        .into_iter()
        .any(|child| child == part || contains_strictly(child, part))
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

/// The codec of an ADT, newtype or struct declared in the linked modules.
/// They cross in the TS target's shape: a newtype is its single tagged
/// variant, and an opaque type keeps its representation, with only its
/// declared type hidden. The codec is registered before its fields so that
/// a recursive type refers to its own index.
fn named_codec(ty: &CoreType, codecs: &mut Codecs) -> Option<usize> {
    let modules = codecs.modules?;
    let CoreType::Named { name, arguments } = ty else {
        return None;
    };
    if let Some((_, index)) = codecs.named.iter().find(|(named, _)| named == ty) {
        return Some(*index);
    }
    // A type that refers to itself with other type arguments, such as
    // `Nested<A> = | Next (Nested<Array<A>>)`, has no finite set of codecs.
    // Each new argument nests the type once more, so stop at a fixed depth.
    // A type written inside an outer use of the same name, such as each
    // level of `Box<Box<Box<Int>>>`, is smaller than that outer type, and a
    // chain of smaller types ends, so only the other expansions count.
    if codecs
        .building
        .iter()
        .filter(|building| {
            matches!(building, CoreType::Named { name: outer, .. } if outer == name)
                && !contains_strictly(building, ty)
        })
        .count()
        >= MAX_NESTED_TYPE_EXPANSION
    {
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
    if parameters.len() != arguments.len() {
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
    codecs.building.push(ty.clone());
    codecs.items.push(ProgramCodec::Adt {
        name: "\0pending".into(),
        variants: Vec::new(),
    });
    codecs.named.push((ty.clone(), index));
    let item = if let Some(adt) = adt.filter(|adt| adt.newtype && codecs.erase_newtypes) {
        match adt.variants.as_slice() {
            [variant] => variant
                .payload
                .as_ref()
                .and_then(|payload| codec(&declared(payload)?, false, codecs))
                .map(|value| ProgramCodec::Newtype {
                    name: adt.name.clone(),
                    owner: variant.symbol.clone(),
                    value,
                }),
            _ => None,
        }
    } else if let Some(adt) = adt {
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
                opaque: structure.opaque,
            })
    };
    codecs.building.pop();
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

fn instantiated_export(
    module: &CoreModule,
    entry_module: &str,
    instance: &ProgramWasmInstantiation,
    codecs: &mut Codecs,
) -> Result<ProgramWasmExport, RustBackendError> {
    let function = module
        .functions
        .iter()
        .find(|f| f.symbol == format!("{entry_module}::{}", instance.function))
        .ok_or("program WASM: the instantiated function is missing from the entry module")?;
    let error = |message: &str| project::at(message, &function.origin, entry_module);
    if function.visibility != seseragi_syntax::Visibility::Public {
        return Err(error("program WASM: selected function must be public"));
    }
    if function.type_parameters.is_empty() {
        return Err(error(
            "program WASM: type arguments select an instance of a generic function only",
        ));
    }
    let binding = module
        .bindings
        .iter()
        .find(|b| b.symbol == format!("{entry_module}::{}", instance.binding))
        .ok_or_else(|| error("program WASM: the checked instantiation is missing"))?;
    let ty = emit::checked_boundary_body_type(module, &binding.value)
        .ok_or_else(|| error("program WASM: checked instantiated type is unavailable"))?;
    // Source arity is the generic declaration's; the type arguments fix the
    // types and the instances, never the number of source stages.
    let mut remainder = &ty;
    for _ in &function.parameters {
        let CoreType::Function { result, .. } = remainder else {
            return Err(error(
                "program WASM: source parameters exceed the checked function type",
            ));
        };
        remainder = result;
    }
    let codec = codec(&ty, true, codecs)
        .or_else(|| effect_codec(&ty, codecs))
        .ok_or_else(|| error("program WASM: unsupported public codec for these type arguments"))?;
    Ok(ProgramWasmExport {
        name: instance.name.clone(),
        symbol: binding.symbol.clone(),
        source_arity: function.parameters.len(),
        value_type: ty,
        codec,
        origin: function.origin.clone(),
    })
}

fn rust_codec(codec: &ProgramCodec) -> String {
    match codec {
        ProgramCodec::Unit => "ProgramCode::Unit".into(),
        ProgramCodec::Bool => "ProgramCode::Bool".into(),
        ProgramCodec::Int => "ProgramCode::Int".into(),
        ProgramCodec::String => "ProgramCode::Text".into(),
        ProgramCodec::Float => "ProgramCode::Float".into(),
        ProgramCodec::Char => "ProgramCode::Char".into(),
        ProgramCodec::BigInt => "ProgramCode::BigInt".into(),
        ProgramCodec::Decimal => "ProgramCode::Decimal".into(),
        ProgramCodec::Tuple { elements } => format!("ProgramCode::Tuple(vec!{:?})", elements),
        ProgramCodec::Maybe { value } => format!("ProgramCode::Maybe({value})"),
        ProgramCodec::Either { left, right } => format!("ProgramCode::Either({left}, {right})"),
        ProgramCodec::Function { parameter, result } => {
            format!("ProgramCode::Function({parameter}, {result})")
        }
        ProgramCodec::Effect {
            failure, success, ..
        } => {
            format!("ProgramCode::Effect({failure:?}, {success:?})")
        }
        ProgramCodec::HostEffect {
            parameters,
            failure,
            success,
        } => format!("ProgramCode::HostEffect(vec!{parameters:?}, {failure:?}, {success:?})"),
        ProgramCodec::HostFunction { parameters, result } => {
            format!("ProgramCode::HostFunction(vec!{parameters:?}, {result})")
        }
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
        ProgramCodec::Newtype { owner, value, .. } => {
            format!("ProgramCode::Newtype({owner:?}, {value})")
        }
    }
}

/// Names a generated declaration must not redefine: the TS globals the
/// declarations refer to and the names the declarations define.
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
    "ProviderResult",
    "ProviderOperation",
    "ProviderEffect",
];

/// The TS name of a type parameter. A name the declarations already use, or
/// one starting with `Program` like generated names, gets a `ProgramType`
/// prefix; ADT and struct names never take a type parameter's name.
fn type_parameter_name(name: &str) -> String {
    if RESERVED.contains(&name) || name.starts_with("Program") {
        format!("ProgramType{name}")
    } else {
        name.to_owned()
    }
}

/// The TS type names of ADT and struct codecs. These types can be
/// recursive, so each is declared once by name instead of being expanded
/// where it is used. A source name becomes the TS name only when it is a
/// plain identifier used by no other codec and does not shadow a TS global
/// or a name these declarations use; otherwise the codec gets
/// `ProgramData<index>`.
fn type_aliases(metadata: &ProgramWasmMetadata) -> std::collections::BTreeMap<usize, String> {
    let names = metadata
        .codecs
        .iter()
        .enumerate()
        .filter_map(|(index, codec)| match codec {
            ProgramCodec::Adt { name, .. }
            | ProgramCodec::Newtype { name, .. }
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
        ProgramCodec::BigInt => "ProgramBigInt".into(),
        ProgramCodec::Decimal => "ProgramDecimal".into(),
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
        ProgramCodec::Effect {
            failure,
            success,
            services,
        } if services.is_empty() => format!(
            "OwnedEffect<{}, {}>",
            failure.map(ty).unwrap_or_else(|| "never".into()),
            success.map(ty).unwrap_or_else(|| "never".into())
        ),
        ProgramCodec::Effect {
            failure,
            success,
            services,
        } => format!(
            "ProviderEffect<{}, {}, {{ {} }}>",
            failure.map(ty).unwrap_or_else(|| "never".into()),
            success.map(ty).unwrap_or_else(|| "never".into()),
            services
                .iter()
                .map(|service| format!("readonly {service}: {}", provider_type(service)))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        ProgramCodec::HostEffect {
            parameters,
            failure,
            success,
        } => match parameters.as_slice() {
            [parameter] => format!(
                "HostEffect<{}, {}, {}>",
                ty(*parameter),
                failure.map(ty).unwrap_or_else(|| "never".into()),
                success.map(ty).unwrap_or_else(|| "never".into()),
            ),
            // The source arguments come first, then the transport context.
            _ => format!(
                "({}, context: HostEffectContext) => HostResult<{}, {}> | PromiseLike<HostResult<{}, {}>>",
                parameters
                    .iter()
                    .enumerate()
                    .map(|(index, parameter)| format!("argument{index}: {}", ty(*parameter)))
                    .collect::<Vec<_>>()
                    .join(", "),
                failure.map(ty).unwrap_or_else(|| "never".into()),
                success.map(ty).unwrap_or_else(|| "never".into()),
                failure.map(ty).unwrap_or_else(|| "never".into()),
                success.map(ty).unwrap_or_else(|| "never".into()),
            ),
        },
        ProgramCodec::HostFunction { parameters, result } => format!(
            "({}) => {}",
            parameters
                .iter()
                .enumerate()
                .map(|(index, parameter)| format!("argument{index}: {}", ty(*parameter)))
                .collect::<Vec<_>>()
                .join(", "),
            ty(*result)
        ),
        ProgramCodec::Array { element } => format!("ReadonlyArray<{}>", ty(*element)),
        ProgramCodec::List { element } => format!("ProgramList<{}>", ty(*element)),
        ProgramCodec::Parameter { name } => type_parameter_name(name),
        ProgramCodec::Record { name: Some(_), .. }
        | ProgramCodec::Adt { .. }
        | ProgramCodec::Newtype { .. } => {
            // A declaration over type parameters is applied to the same
            // parameters, which the enclosing callback binds.
            generic_alias(metadata, &aliases[&index], index)
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
                let name = type_parameter_name(name);
                if !found.contains(&name) {
                    found.push(name);
                }
            }
            ProgramCodec::Tuple { elements } => pending.extend(elements.iter().rev()),
            ProgramCodec::Maybe { value } => pending.push(*value),
            ProgramCodec::Either { left, right } => pending.extend([*right, *left]),
            ProgramCodec::Function { parameter, result } => pending.extend([*result, *parameter]),
            ProgramCodec::Array { element } | ProgramCodec::List { element } => {
                pending.push(*element)
            }
            ProgramCodec::Record { fields, .. } => {
                pending.extend(fields.iter().rev().map(|field| field.codec))
            }
            ProgramCodec::Adt { variants, .. } => {
                pending.extend(variants.iter().rev().filter_map(|variant| variant.payload))
            }
            ProgramCodec::Effect {
                failure, success, ..
            } => pending.extend([*success, *failure].into_iter().flatten()),
            ProgramCodec::HostEffect {
                parameters,
                failure,
                success,
            } => {
                pending.extend([*success, *failure].into_iter().flatten());
                pending.extend(parameters.iter().rev());
            }
            ProgramCodec::HostFunction { parameters, result } => {
                pending.push(*result);
                pending.extend(parameters.iter().rev());
            }
            ProgramCodec::Newtype { value, .. } => pending.push(*value),
            _ => {}
        }
    }
    found
}

/// An ADT or struct alias, applied to the type parameters its codec
/// mentions in first-use order, such as `Box<A>`.
fn generic_alias(metadata: &ProgramWasmMetadata, alias: &str, index: usize) -> String {
    let parameters = type_parameters(metadata, index);
    if parameters.is_empty() {
        alias.to_owned()
    } else {
        format!("{alias}<{}>", parameters.join(", "))
    }
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
    // The TS target brands both types, so the host passes back only values
    // that a program returned, as with the TS target's own functions.
    if metadata
        .codecs
        .iter()
        .any(|codec| matches!(codec, ProgramCodec::BigInt))
    {
        result.insert_str(0, "declare const ProgramBigIntBrand: unique symbol;\nexport type ProgramBigInt = bigint & { readonly [ProgramBigIntBrand]: true };\n");
    }
    if metadata
        .codecs
        .iter()
        .any(|codec| matches!(codec, ProgramCodec::Decimal))
    {
        result.insert_str(0, "declare const ProgramDecimalBrand: unique symbol;\nexport type ProgramDecimal = Readonly<{coefficient: bigint; scale: bigint; readonly [ProgramDecimalBrand]: true}>;\n");
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
            ProgramCodec::Newtype { value, .. } => declared_type(metadata, &aliases, *value),
            ProgramCodec::Record { opaque: true, .. } => {
                // The TS target brands an opaque struct the same way, so the
                // host can hold and return it but cannot read or build it.
                let brand = format!("ProgramBrand{index}");
                result.insert_str(0, &format!("declare const {brand}: unique symbol;\n"));
                format!("{{ readonly [{brand}]: true }}")
            }
            ProgramCodec::Record { fields, .. } => record_type(metadata, &aliases, fields),
            _ => unreachable!("aliases name ADT and struct codecs"),
        };
        result.insert_str(
            0,
            &format!(
                "export type {} = {body};\n",
                generic_alias(metadata, alias, *index)
            ),
        );
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
        result.push_str("}\nexport type HostResult<E, A> = Readonly<{tag: 'Success'; value: A}> | Readonly<{tag: 'Failure'; error: E}> | Readonly<{tag: 'Defect'; messages: readonly string[]}>;\nexport interface HostEffectContext { readonly signal: AbortSignal; }\nexport type HostEffect<A, E, B> = (argument: A, context: HostEffectContext) => HostResult<E, B> | PromiseLike<HostResult<E, B>>;\nexport type ProgramLocation = Readonly<{file: string; line: number; column: number}>;\nexport type ProgramOutcome<E, A> = Readonly<{tag: 'Success'; value: A}> | Readonly<{tag: 'Failure'; error: E; location?: ProgramLocation}> | Readonly<{tag: 'Defect'; messages: readonly string[]; locations?: readonly (ProgramLocation | null)[]}> | Readonly<{tag: 'Cancelled'}>;\nexport interface ProgramExecution<E, A> { readonly result: Promise<ProgramOutcome<E, A>>; cancel(): void; }\nexport interface OwnedEffect<E, A> { run(): ProgramExecution<E, A>; dispose(): void; readonly disposed: boolean; }\nexport type ProviderResult<E, A> = Readonly<{kind: 'success'; value: A}> | Readonly<{kind: 'failure'; error: E}>;\nexport type ProviderOperation<E> = ProviderResult<E, unknown> | PromiseLike<ProviderResult<E, unknown>>;\nexport type ProgramConsoleError = Readonly<{kind: string; message: string}>;\nexport interface ProgramConsole { print(text: string): ProviderOperation<ProgramConsoleError>; println(text: string): ProviderOperation<ProgramConsoleError>; error(text: string): ProviderOperation<ProgramConsoleError>; errorLine(text: string): ProviderOperation<ProgramConsoleError>; flush(): ProviderOperation<ProgramConsoleError>; }\nexport interface ProviderEffect<E, A, R> { run(environment: R): ProgramExecution<E, A>; dispose(): void; readonly disposed: boolean; }\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): Promise<void>; ownership(): Ownership & Readonly<{plans: number; running: boolean; closing: boolean; hostCallbacks: number; waitingHost: boolean}>; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    } else if metadata.abi == PROGRAM_WASM_EFFECT_ABI {
        result.push_str("}\nexport type ProgramLocation = Readonly<{file: string; line: number; column: number}>;\nexport type ProgramOutcome<E, A> = Readonly<{tag: 'Success'; value: A}> | Readonly<{tag: 'Failure'; error: E; location?: ProgramLocation}> | Readonly<{tag: 'Defect'; messages: readonly string[]; locations?: readonly (ProgramLocation | null)[]}> | Readonly<{tag: 'Cancelled'}>;\nexport interface ProgramExecution<E, A> { readonly result: Promise<ProgramOutcome<E, A>>; cancel(): void; }\nexport interface OwnedEffect<E, A> { run(): ProgramExecution<E, A>; dispose(): void; readonly disposed: boolean; }\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): Promise<void>; ownership(): Ownership & Readonly<{plans: number; hostValues: number; running: boolean; closing: boolean}>; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    } else {
        result.push_str("}\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): void; ownership(): Ownership & Readonly<{hostValues: number}>; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    }
    result
}
