//! Adapter-supplied runtime evidence for emitted raw foreign bindings.
//! Declaration interpretation never manufactures runtime artifact digests.
use crate::{ConversionResult, ConvertError, EntryConfig};
use serde::Serialize;
pub use seseragi_interop::RuntimeLinkageEvidence;
use seseragi_interop::{
    CallbackAbi, CallbackLifetime, CallingConvention, CancellationAbi, Evaluation, FailureBoundary,
    ReceiverAbi, RuntimeArtifactIdentity, RuntimeDelivery, RuntimeExportAbi,
    RuntimeLinkageContract, RuntimeTarget, SemanticType,
};
use std::collections::{BTreeMap, BTreeSet};

/// Add truthful target artifact evidence after conversion. Missing targets do
/// not invalidate declaration conversion: execution checks availability later.
/// This preserves the lazy failure boundary for unused/declaration-only hosts.
pub fn attach_runtime_linkage(
    converted: &mut ConversionResult,
    entry: &EntryConfig,
    artifacts: &BTreeMap<RuntimeTarget, RuntimeArtifactIdentity>,
) -> Result<(), ConvertError> {
    // There is no runtime evidence to persist when adapters only have declarations.
    if artifacts.is_empty() {
        return Ok(());
    }
    let (Some(generated), Some(graph)) = (&mut converted.generated, &mut converted.semantic) else {
        return Ok(());
    };
    let mut metadata: serde_json::Value = serde_json::from_str(&generated.metadata)
        .map_err(|error| ConvertError::new(error.to_string()))?;
    let host_identity = metadata["hostModule"]["exactIdentity"]
        .as_str()
        .ok_or_else(|| ConvertError::new("binding metadata has no exact host identity"))?
        .to_owned();
    let mut contract =
        RuntimeLinkageContract::new(graph).map_err(|error| ConvertError::new(error.to_string()))?;
    for generated_symbol in metadata["symbols"].as_array().into_iter().flatten() {
        // Opaque types and namespaces have no executable callable ABI. Their
        // nominal constructor/member meanings remain available in the graph.
        if generated_symbol["kind"] != "function" {
            continue;
        }
        let owner = generated_symbol["publicPath"].as_str().unwrap_or_default();
        let Some(symbol) = graph.symbols.get(owner) else {
            return Err(ConvertError::new("emitted function has no semantic owner"));
        };
        let policy = entry.symbols.get(owner);
        let mut selected = symbol.type_ref;
        let mut seen = BTreeSet::new();
        while seen.insert(selected) {
            match graph.types.get(&selected) {
                Some(SemanticType::Generic { body, .. }) => selected = *body,
                Some(SemanticType::Reference { symbol }) => {
                    let Some(symbol) = graph.symbols.get(symbol) else {
                        break;
                    };
                    selected = symbol.type_ref;
                }
                Some(SemanticType::Overload { signatures }) => {
                    let local = generated_symbol["localName"].as_str().unwrap_or_default();
                    let index = policy
                        .and_then(|policy| policy.overloads.get(local))
                        .map_or(0, |selection| selection.signature);
                    let Some(signature) = signatures.get(index) else {
                        break;
                    };
                    selected = *signature;
                }
                _ => break,
            }
        }
        let Some(SemanticType::Callable {
            parameters,
            receiver,
            ..
        }) = graph.types.get(&selected)
        else {
            return Err(ConvertError::new(
                "emitted function has no resolved callable signature",
            ));
        };
        let declared_delivery = seseragi_interop::runtime_export_delivery(graph, owner, selected);
        let compiler_codec_wrapper =
            generated_symbol["signature"]
                .as_str()
                .is_some_and(|signature| {
                    let local = generated_symbol["localName"].as_str().unwrap_or_default();
                    signature.starts_with(&format!("task fn {local}<"))
                        || signature.starts_with(&format!("pure fn {local}<"))
                });
        let delivery = match declared_delivery {
            Some(RuntimeDelivery::CodecSelected) if !compiler_codec_wrapper => {
                RuntimeDelivery::Synchronous
            }
            Some(delivery) => delivery,
            None => RuntimeDelivery::Synchronous,
        };
        if delivery == RuntimeDelivery::CodecSelected {
            converted.diagnostics.push(crate::ConversionDiagnostic {
                code: "SES-F0202".into(), severity: crate::DiagnosticSeverity::Warning,
                message: "conditional generic delivery requires a compiler-selected checked primitive ForeignCodec at every concrete call".into(),
                file: entry.declaration.clone(), start: 0, end: 0, symbol: Some(owner.to_owned()),
            });
        }
        let pure = generated_symbol["rule"] == "pure-function";
        let mut callbacks = Vec::new();
        for (index, parameter) in parameters.iter().enumerate() {
            if let Some(policy) = policy.and_then(|policy| policy.callbacks.get(&parameter.name)) {
                let delivery = match policy.invocation {
                    crate::config::CallbackInvocation::Sync => RuntimeDelivery::Synchronous,
                    crate::config::CallbackInvocation::Promise => RuntimeDelivery::Promise,
                };
                callbacks.push(CallbackAbi {
                    parameter_index: index as u32,
                    delivery,
                    lifetime: match policy.lifetime {
                        crate::config::CallbackLifetime::DuringCall => CallbackLifetime::DuringCall,
                        // Retention cannot be mistaken for a during-call lifetime;
                        // the raw converter's release policy remains explicit.
                        crate::config::CallbackLifetime::Retained => CallbackLifetime::Retained,
                        crate::config::CallbackLifetime::UntilSettled => {
                            CallbackLifetime::UntilSettled
                        }
                    },
                    failure: if delivery == RuntimeDelivery::Promise {
                        FailureBoundary::ThrowOrRejection
                    } else {
                        FailureBoundary::Throw
                    },
                });
            }
        }
        let local = generated_symbol["localName"].as_str().unwrap_or_default();
        // Overload aliases must have distinct binding identities even though
        // their host export and semantic owner are the same.
        let key = format!(
            "{}#{local}",
            generated_symbol["identity"].as_str().unwrap_or(owner)
        );
        contract.exports.insert(
            key,
            RuntimeExportAbi {
                semantic_owner: owner.to_owned(),
                semantic_type: selected,
                specifier: entry.specifier.clone(),
                declaration_module_identity: host_identity.clone(),
                export_path: owner.split('.').map(str::to_owned).collect(),
                convention: CallingConvention::Function,
                receiver: if receiver.is_some() {
                    ReceiverAbi::FirstArgument
                } else {
                    ReceiverAbi::None
                },
                evaluation: if pure {
                    Evaluation::PureApproved
                } else {
                    Evaluation::Effectful
                },
                delivery,
                failure: if pure {
                    FailureBoundary::CannotFailApproved
                } else if delivery == RuntimeDelivery::Promise {
                    FailureBoundary::ThrowOrRejection
                } else {
                    FailureBoundary::Throw
                },
                cancellation: CancellationAbi::Unsupported,
                callbacks,
                targets: artifacts.clone(),
            },
        );
    }
    // Validate known target evidence now. Unsupported/missing host targets are
    // deliberately deferred; declaration conversion is still useful without JS.
    for (target, artifact) in artifacts {
        let fresh = contract
            .exports
            .keys()
            .map(|key| (key.clone(), artifact.clone()))
            .collect();
        if let Err(errors) = contract.validate_for(graph, *target, &fresh) {
            converted
                .diagnostics
                .extend(errors.into_iter().map(|error| crate::ConversionDiagnostic {
                    code: error.code,
                    severity: crate::DiagnosticSeverity::Warning,
                    message: error.reason,
                    file: entry.declaration.clone(),
                    start: 0,
                    end: 0,
                    symbol: error.symbol,
                }));
        }
    }
    graph
        .identity
        .as_mut()
        .ok_or_else(|| ConvertError::new("semantic graph has no artifact identity"))?
        .runtime_contract_digest = Some(
        contract
            .digest()
            .map_err(|error| ConvertError::new(error.to_string()))?,
    );
    metadata["runtimeLinkage"] = serde_json::to_value(RuntimeLinkageEvidence {
        graph: graph.clone(),
        contract: contract.clone(),
    })
    .map_err(|error| ConvertError::new(error.to_string()))?;
    generated.metadata = pretty(&metadata)?;
    let mut report: serde_json::Value = serde_json::from_str(&generated.report)
        .map_err(|error| ConvertError::new(error.to_string()))?;
    report["warnings"] = serde_json::to_value(
        converted
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == crate::DiagnosticSeverity::Warning)
            .collect::<Vec<_>>(),
    )
    .map_err(|error| ConvertError::new(error.to_string()))?;
    generated.report = pretty(&report)?;
    if let Some(report) = converted.compatibility.as_mut() {
        let proofs = artifacts
            .iter()
            .map(|(target, artifact)| {
                (
                    *target,
                    contract
                        .exports
                        .keys()
                        .map(|key| (key.clone(), artifact.clone()))
                        .collect(),
                )
            })
            .collect();
        report.apply_runtime_linkage(graph, &contract, &proofs);
    }
    crate::refresh_compatibility_identity(converted).map_err(ConvertError::new)?;
    Ok(())
}
fn pretty(value: &impl Serialize) -> Result<String, ConvertError> {
    serde_json::to_string_pretty(value)
        .map(|value| format!("{value}\n"))
        .map_err(|error| ConvertError::new(error.to_string()))
}
