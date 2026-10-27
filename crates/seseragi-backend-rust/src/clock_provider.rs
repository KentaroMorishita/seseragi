//! A finite native target binding, not an implementation of every Clock API.
use crate::RustBackendError;
use serde::{Deserialize, Serialize};
use seseragi_lowering::SourceSpan;
use seseragi_provider::{
    ContractVersion, LogicalType, OperationKind, Portability, ProviderContract,
};
use sha2::{Digest, Sha256};

pub const NATIVE_CLOCK_SLEEP_PROVIDER: &str = "seseragi/runtime-native#clock-sleep";
const CONTRACT: &str =
    include_str!("../../../examples/spec/artifacts/provider-contract-schema-1/clock/contract.json");
pub(crate) const SLEEP: &str = "std/clock::Clock#sleep";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeClockProvider {
    BuiltinSleep,
}
impl NativeClockProvider {
    pub fn from_identity(identity: &str) -> Result<Self, RustBackendError> {
        if identity == NATIVE_CLOCK_SLEEP_PROVIDER {
            Ok(Self::BuiltinSleep)
        } else {
            Err(format!("Rust backend: unsupported native Clock provider {identity}; explicitly select {NATIVE_CLOCK_SLEEP_PROVIDER}; JavaScript providers are not substituted").into())
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeProviderSelectionOrigin {
    #[default]
    ExplicitApi,
    CommandLine,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RustTargetOptions {
    pub clock_provider: Option<NativeClockProvider>,
    pub clock_selection_origin: NativeProviderSelectionOrigin,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeProviderReceipt {
    pub target: String,
    pub backend: String,
    pub provider: String,
    pub selection: NativeProviderSelectionOrigin,
    pub service: String,
    pub contract_version: ContractVersion,
    pub contract_digest: String,
    pub supported_operations: Vec<String>,
    pub traces: Vec<SourceSpan>,
}

pub(crate) fn contract() -> Result<ProviderContract, RustBackendError> {
    let contract = ProviderContract::from_json(CONTRACT)
        .map_err(|error| format!("Rust backend: invalid Clock contract: {error}"))?;
    let operation = contract
        .operation(SLEEP)
        .ok_or("Rust backend: Clock contract is missing canonical sleep")?;
    if contract.identity != "std/clock::Clock"
        || contract.requirement.field != "clock"
        || contract.requirement.type_identity != "std/clock::Clock"
        || contract.version != (ContractVersion { major: 1, minor: 0 })
        || operation.kind != OperationKind::OneShot
        || operation.portability != Portability::Portable
        || operation.input
            != (LogicalType::Named {
                identity: "std/time::Duration".into(),
            })
        || operation.success != LogicalType::Unit
        || operation.failure != LogicalType::Never
    {
        return Err("Rust backend: native sleep disagrees with canonical Clock contract".into());
    }
    Ok(contract)
}

pub(crate) fn receipt(
    options: &RustTargetOptions,
    traces: Vec<SourceSpan>,
) -> Result<NativeProviderReceipt, RustBackendError> {
    options.clock_provider.ok_or_else(|| RustBackendError::from(
        format!("Rust backend: Clock sleep requires explicit provider selection; use --provider std/clock::Clock={NATIVE_CLOCK_SLEEP_PROVIDER}")
    ))?;
    let contract = contract()?;
    Ok(NativeProviderReceipt {
        target: "native".into(),
        backend: "rust-native-sleep-v1".into(),
        provider: NATIVE_CLOCK_SLEEP_PROVIDER.into(),
        selection: options.clock_selection_origin,
        service: contract.identity,
        contract_version: contract.version,
        contract_digest: format!("{:x}", Sha256::digest(CONTRACT.as_bytes())),
        supported_operations: vec![SLEEP.into()],
        traces,
    })
}
