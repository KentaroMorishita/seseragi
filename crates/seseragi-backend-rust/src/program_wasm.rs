//! A closed, pure public ABI over checked Core. It is deliberately distinct
//! from the TypeScript foreign ABI and from the compiler WASM adapter.
use crate::{emit, project, RustBackendError};
use serde::{Deserialize, Serialize};
use seseragi_lowering::{CoreModule, CoreType, SourceSpan};
use std::collections::BTreeSet;

pub const PROGRAM_WASM_ABI: &str = "seseragi-program-wasm-pure-v1";
pub const PROGRAM_WASM_EFFECT_ABI: &str = "seseragi-program-wasm-effect-v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ProgramCodec {
    Unit,
    Bool,
    Int,
    String,
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
    pub trap_policy: String,
    pub max_boundary_bytes: usize,
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
    let module = modules
        .iter()
        .find(|m| m.module == entry_module)
        .ok_or("program WASM: entry module is missing")?;
    if exports.is_empty() {
        return Err("program WASM: select at least one public function".into());
    }
    let mut seen = BTreeSet::new();
    let mut codecs = Vec::new();
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
        if !function.type_parameters.is_empty()
            || !function.constraints.is_empty()
            || function.parameters.is_empty()
        {
            return Err(error(
                "program WASM: an unconstrained monomorphic source function is required",
            ));
        }
        let ty = function
            .value_type
            .as_ref()
            .and_then(|ty| emit::canonical_boundary_type(module, ty))
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
                || emit::canonical_boundary_type(module, &parameter.type_ref).as_ref()
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
        let body_type = emit::checked_boundary_body_type(module, &function.body);
        let effect_result = |mut ty: &CoreType| {
            while let CoreType::Function { result, .. } = ty {
                ty = result;
            }
            matches!(ty, CoreType::Named { name, arguments } if name == "std/prelude::Effect" && arguments.len() == 3)
        };
        if body_type.as_ref() != Some(remainder)
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
            .ok_or_else(|| error("program WASM: unsupported public codec; expected closed Unit/Bool/Int/String, tuple, Maybe/Either, a returned function or providerless Effect"))?;
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
    // Reuse the ordinary linker and every native evidence/admission check.
    // This explicit library mode changes no native executable entry contract.
    let mut library = project::emit_library(modules, entry_module)?;
    if effect_abi {
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
        abi: if effect_abi {
            PROGRAM_WASM_EFFECT_ABI
        } else {
            PROGRAM_WASM_ABI
        }
        .into(),
        stability: "experimental".into(),
        module: entry_module.into(),
        modules: library.metadata.modules,
        exports: public,
        codecs,
        trap_policy: "retire-instance".into(),
        max_boundary_bytes: 16 * 1024 * 1024,
    };
    library.source.push_str(if effect_abi {
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
    let javascript = format!("// Generated checked program WASM ABI.\nconst contract = {quoted_schema};\nconst schema = JSON.parse(contract);\n{}", if effect_abi { include_str!("program_wasm_effect.js") } else { include_str!("program_wasm.js") });
    let declarations = declarations(&metadata);
    Ok(ProgramWasmArtifact {
        source: library.source,
        metadata,
        javascript,
        declarations,
    })
}

fn effect_codec(ty: &CoreType, codecs: &mut Vec<ProgramCodec>) -> Option<usize> {
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
            let terminal = |ty: &CoreType, callable, codecs: &mut Vec<ProgramCodec>| {
                if matches!(ty, CoreType::Named { name, arguments } if name == "std/prelude::Never" && arguments.is_empty())
                {
                    Some(None)
                } else {
                    codec(ty, callable, codecs).map(Some)
                }
            };
            ProgramCodec::Effect {
                failure: terminal(failure, false, codecs)?,
                success: terminal(success, true, codecs)?,
            }
        }
        _ => return None,
    };
    if let Some(index) = codecs.iter().position(|c| c == &item) {
        Some(index)
    } else {
        let index = codecs.len();
        codecs.push(item);
        Some(index)
    }
}

fn codec(ty: &CoreType, callable: bool, codecs: &mut Vec<ProgramCodec>) -> Option<usize> {
    let item = match ty {
        CoreType::Named { name, arguments } => match (name.as_str(), arguments.as_slice()) {
            ("std/prelude::Unit", []) => ProgramCodec::Unit,
            ("std/prelude::Bool", []) => ProgramCodec::Bool,
            ("std/prelude::Int", []) => ProgramCodec::Int,
            ("std/prelude::String", []) => ProgramCodec::String,
            ("std/prelude::Maybe", [value]) => ProgramCodec::Maybe {
                value: codec(value, false, codecs)?,
            },
            ("std/prelude::Either", [left, right]) => ProgramCodec::Either {
                left: codec(left, false, codecs)?,
                right: codec(right, false, codecs)?,
            },
            _ => return None,
        },
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
    if let Some(index) = codecs.iter().position(|c| c == &item) {
        Some(index)
    } else {
        let index = codecs.len();
        codecs.push(item);
        Some(index)
    }
}

fn rust_codec(codec: &ProgramCodec) -> String {
    match codec {
        ProgramCodec::Unit => "ProgramCode::Unit".into(),
        ProgramCodec::Bool => "ProgramCode::Bool".into(),
        ProgramCodec::Int => "ProgramCode::Int".into(),
        ProgramCodec::String => "ProgramCode::Text".into(),
        ProgramCodec::Tuple { elements } => format!("ProgramCode::Tuple(vec!{:?})", elements),
        ProgramCodec::Maybe { value } => format!("ProgramCode::Maybe({value})"),
        ProgramCodec::Either { left, right } => format!("ProgramCode::Either({left}, {right})"),
        ProgramCodec::Function { parameter, result } => {
            format!("ProgramCode::Function({parameter}, {result})")
        }
        ProgramCodec::Effect { failure, success } => {
            format!("ProgramCode::Effect({failure:?}, {success:?})")
        }
    }
}

fn declarations(metadata: &ProgramWasmMetadata) -> String {
    fn ty(metadata: &ProgramWasmMetadata, index: usize) -> String {
        match &metadata.codecs[index] {
            ProgramCodec::Unit => "undefined".into(),
            ProgramCodec::Bool => "boolean".into(),
            ProgramCodec::Int => "number".into(),
            ProgramCodec::String => "string".into(),
            ProgramCodec::Tuple { elements } => format!(
                "readonly [{}]",
                elements
                    .iter()
                    .map(|i| ty(metadata, *i))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ProgramCodec::Maybe { value } => format!(
                "Readonly<{{tag: 'Nothing'}}> | Readonly<{{tag: 'Just'; value: {}}}>",
                ty(metadata, *value)
            ),
            ProgramCodec::Either { left, right } => format!(
                "Readonly<{{tag: 'Left'; value: {}}}> | Readonly<{{tag: 'Right'; value: {}}}>",
                ty(metadata, *left),
                ty(metadata, *right)
            ),
            ProgramCodec::Function { parameter, result } => format!(
                "OwnedCallback<{}, {}>",
                ty(metadata, *parameter),
                ty(metadata, *result)
            ),
            ProgramCodec::Effect { failure, success } => format!(
                "OwnedEffect<{}, {}>",
                failure
                    .map(|id| ty(metadata, id))
                    .unwrap_or_else(|| "never".into()),
                success
                    .map(|id| ty(metadata, id))
                    .unwrap_or_else(|| "never".into())
            ),
        }
    }
    let mut result = String::from("export interface Ownership { readonly alive: boolean; readonly retired: boolean; readonly callbacks: number; readonly managedLiveAtClose: number | null; }\nexport interface OwnedCallback<A, B> { (argument: A): B; dispose(): void; readonly disposed: boolean; }\nexport class ProgramWasmDefect extends Error { readonly origin?: Readonly<{source: string; start: number; end: number}>; }\nexport class ProgramWasmClosed extends Error {}\nexport interface ProgramExports {\n");
    for export in &metadata.exports {
        result.push_str(&format!(
            "readonly {}: {};\n",
            serde_json::to_string(&export.name).unwrap(),
            ty(metadata, export.codec)
        ));
    }
    if metadata.abi == PROGRAM_WASM_EFFECT_ABI {
        result.push_str("}\nexport type ProgramOutcome<E, A> = Readonly<{tag: 'Success'; value: A}> | Readonly<{tag: 'Failure'; error: E}> | Readonly<{tag: 'Defect'; messages: readonly string[]}> | Readonly<{tag: 'Cancelled'}>;\nexport interface ProgramExecution<E, A> { readonly result: Promise<ProgramOutcome<E, A>>; cancel(): void; }\nexport interface OwnedEffect<E, A> { run(): ProgramExecution<E, A>; dispose(): void; readonly disposed: boolean; }\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): Promise<void>; ownership(): Ownership & Readonly<{plans: number; running: boolean; closing: boolean}>; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    } else {
        result.push_str("}\nexport interface ProgramInstance { readonly exports: ProgramExports; dispose(): void; ownership(): Ownership; }\nexport function loadProgram(bytes: BufferSource): Promise<ProgramInstance>;\n");
    }
    result
}
