//! Runtime locators and ABI are supplied by adapters, never inferred from types.
use crate::{
    ArtifactIdentity, CallingConvention, Evaluation, FailureBoundary, SemanticGraph, SemanticType,
    TypeId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const LINKAGE_SCHEMA: u32 = 1;
pub const LINKAGE_KIND: &str = "seseragi-interop-runtime-linkage";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeTarget {
    Browser,
    Node,
    Bun,
}
impl RuntimeTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Node => "node",
            Self::Bun => "bun",
        }
    }
    fn condition(self, name: &str) -> bool {
        name == "import"
            || name == "default"
            || match self {
                Self::Browser => name == "browser",
                Self::Node => name == "node",
                Self::Bun => name == "bun" || name == "node",
            }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeArtifactIdentity {
    /// Exact identity from the host's loader, e.g. npm:example@1.0.0/dist/api.js.
    pub exact_identity: String,
    /// sha256:<64 lowercase hex>, or sha256:builtin for a node: identity only.
    pub content_digest: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeDelivery {
    Synchronous,
    Promise,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum ReceiverAbi {
    None,
    FirstArgument,
    ExportPath(Vec<String>),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallbackLifetime {
    DuringCall,
    Retained,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CallbackAbi {
    pub parameter_index: u32,
    pub delivery: RuntimeDelivery,
    pub lifetime: CallbackLifetime,
    pub failure: FailureBoundary,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum CancellationAbi {
    Unsupported,
    AbortSignalParameter(u32),
    HostContract(String),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeExportAbi {
    /// Semantic ownership and callable/value meaning are separate from the
    /// runtime export path (e.g. a class owns a constructor/member signature).
    pub semantic_owner: String,
    pub semantic_type: TypeId,
    pub specifier: String,
    /// Binding-level identity from its declaration adapter, separate from an
    /// individual target's selected runtime file identity.
    pub declaration_module_identity: String,
    pub export_path: Vec<String>,
    pub convention: CallingConvention,
    pub receiver: ReceiverAbi,
    pub evaluation: Evaluation,
    pub delivery: RuntimeDelivery,
    pub failure: FailureBoundary,
    pub cancellation: CancellationAbi,
    pub callbacks: Vec<CallbackAbi>,
    pub targets: BTreeMap<RuntimeTarget, RuntimeArtifactIdentity>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeLinkageContract {
    pub schema: u32,
    pub kind: String,
    pub declaration: ArtifactIdentity,
    /// Digest of semantics including declaration provenance, with the
    /// runtime_contract_digest field cleared. This avoids a digest cycle.
    pub semantic_digest: String,
    /// Keys are stable binding ABI identities, independent of runtime export
    /// spelling. Each entry names its semantic owner and specific type node.
    pub exports: BTreeMap<String, RuntimeExportAbi>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeLinkageDiagnostic {
    pub code: String,
    pub symbol: Option<String>,
    pub target: Option<RuntimeTarget>,
    pub reason: String,
}
impl RuntimeLinkageDiagnostic {
    pub fn new(code: &str, target: Option<RuntimeTarget>, reason: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            symbol: None,
            target,
            reason: reason.into(),
        }
    }
    fn at(mut self, symbol: &str) -> Self {
        self.symbol = Some(symbol.to_owned());
        self
    }
}
impl std::fmt::Display for RuntimeLinkageDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.reason)
    }
}
impl std::error::Error for RuntimeLinkageDiagnostic {}

pub fn semantic_linkage_digest(graph: &SemanticGraph) -> Result<String, RuntimeLinkageDiagnostic> {
    let mut semantics = graph.clone();
    if let Some(identity) = &mut semantics.identity {
        identity.runtime_contract_digest = None;
    }
    semantics
        .digest()
        .map_err(|error| RuntimeLinkageDiagnostic::new("SES-F0201", None, error.to_string()))
}
fn digest_text(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn artifact_valid(artifact: &RuntimeArtifactIdentity) -> bool {
    !artifact.exact_identity.trim().is_empty()
        && (artifact
            .content_digest
            .strip_prefix("sha256:")
            .is_some_and(digest_text)
            || artifact.exact_identity.starts_with("node:")
                && artifact.content_digest == "sha256:builtin")
}
fn path_valid(path: &[String]) -> bool {
    !path.is_empty() && path.iter().all(|item| !item.trim().is_empty())
}
impl RuntimeLinkageContract {
    pub fn new(graph: &SemanticGraph) -> Result<Self, RuntimeLinkageDiagnostic> {
        let mut declaration = graph.identity.clone().ok_or_else(|| {
            RuntimeLinkageDiagnostic::new(
                "SES-F0201",
                None,
                "semantic graph has no declaration artifact identity",
            )
        })?;
        declaration.runtime_contract_digest = None;
        Ok(Self {
            schema: LINKAGE_SCHEMA,
            kind: LINKAGE_KIND.to_owned(),
            declaration,
            semantic_digest: semantic_linkage_digest(graph)?,
            exports: BTreeMap::new(),
        })
    }
    pub fn digest(&self) -> Result<String, RuntimeLinkageDiagnostic> {
        serde_json::to_vec(self)
            .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
            .map_err(|error| RuntimeLinkageDiagnostic::new("SES-F0201", None, error.to_string()))
    }
    /// Declaration semantics, a contract, and the host's freshly resolved
    /// runtime artifacts must all agree before emitting or executing a binding.
    pub fn validate_for(
        &self,
        graph: &SemanticGraph,
        target: RuntimeTarget,
        artifacts: &BTreeMap<String, RuntimeArtifactIdentity>,
    ) -> Result<(), Vec<RuntimeLinkageDiagnostic>> {
        let mut errors = Vec::new();
        let mut declaration = graph.identity.clone();
        if let Some(identity) = &mut declaration {
            identity.runtime_contract_digest = None;
        }
        if self.schema != LINKAGE_SCHEMA
            || self.kind != LINKAGE_KIND
            || declaration.as_ref() != Some(&self.declaration)
            || semantic_linkage_digest(graph).ok().as_ref() != Some(&self.semantic_digest)
        {
            errors.push(RuntimeLinkageDiagnostic::new(
                "SES-F0201",
                Some(target),
                "stale or incompatible runtime linkage declaration identity",
            ));
        }
        if let Some(expected) = graph
            .identity
            .as_ref()
            .and_then(|id| id.runtime_contract_digest.as_ref())
        {
            if self.digest().ok().as_ref() != Some(expected) {
                errors.push(RuntimeLinkageDiagnostic::new(
                    "SES-F0201",
                    Some(target),
                    "runtime linkage contract digest does not match semantic artifact",
                ));
            }
        }
        for (symbol, export) in &self.exports {
            let error =
                |code, reason| RuntimeLinkageDiagnostic::new(code, Some(target), reason).at(symbol);
            if !graph.symbols.contains_key(&export.semantic_owner)
                || !graph.types.contains_key(&export.semantic_type)
            {
                errors.push(error(
                    "SES-F0201",
                    "runtime linkage refers to missing semantic owner or type",
                ));
            }
            if graph.private_symbols.contains(&export.semantic_owner) {
                errors.push(error(
                    "SES-F0202",
                    "runtime ABI cannot expose a private semantic declaration",
                ));
            }
            let owner_type = graph
                .symbols
                .get(&export.semantic_owner)
                .map(|owner| owner.type_ref);
            if !owner_type.is_some_and(|owner| {
                owned_role(graph, owner, export.semantic_type, export.convention)
            }) {
                errors.push(error(
                    "SES-F0202",
                    "runtime signature is not an owned callable/value surface for this convention",
                ));
            }
            if export.specifier.trim().is_empty()
                || export.declaration_module_identity.trim().is_empty()
                || !path_valid(&export.export_path)
            {
                errors.push(error(
                    "SES-F0202",
                    "runtime locator or export identity is unresolved",
                ));
            }
            if export.evaluation == Evaluation::Unresolved
                || export.failure == FailureBoundary::Unresolved
                || export
                    .callbacks
                    .iter()
                    .any(|cb| cb.failure == FailureBoundary::Unresolved)
            {
                errors.push(error(
                    "SES-F0202",
                    "runtime evaluation or failure ABI is unresolved",
                ));
            }
            let method_receiver = matches!(
                export.receiver,
                ReceiverAbi::FirstArgument | ReceiverAbi::ExportPath(_)
            );
            if (export.convention == CallingConvention::Method) != method_receiver
                || matches!(&export.receiver, ReceiverAbi::ExportPath(path) if !path_valid(path))
            {
                errors.push(error(
                    "SES-F0202",
                    "method receiver ABI is missing or attached to a non-method export",
                ));
            }
            if export.delivery == RuntimeDelivery::Synchronous
                && matches!(
                    export.failure,
                    FailureBoundary::Rejection | FailureBoundary::ThrowOrRejection
                )
            {
                errors.push(error(
                    "SES-F0202",
                    "synchronous export cannot declare Promise rejection ABI",
                ));
            }
            if export.convention == CallingConvention::Constructor
                && export.delivery != RuntimeDelivery::Synchronous
            {
                errors.push(error(
                    "SES-F0202",
                    "constructor ABI must return synchronously",
                ));
            }
            let signatures = callable_types(graph, export.semantic_type);
            if export.convention != CallingConvention::Value && signatures.is_empty() {
                errors.push(error(
                    "SES-F0202",
                    "callable runtime convention has no resolved semantic signature",
                ));
            }
            let index_valid = |index: u32| {
                !signatures.is_empty()
                    && signatures.iter().all(|signature| match signature {
                        SemanticType::Callable { parameters, .. } => {
                            (index as usize) < parameters.len()
                        }
                        _ => false,
                    })
            };
            if export.callbacks.iter().any(|callback| {
                !index_valid(callback.parameter_index)
                    || callback.delivery == RuntimeDelivery::Synchronous
                        && matches!(
                            callback.failure,
                            FailureBoundary::Rejection | FailureBoundary::ThrowOrRejection
                        )
            }) || matches!(export.cancellation, CancellationAbi::AbortSignalParameter(index) if !index_valid(index))
            {
                errors.push(error(
                    "SES-F0202",
                    "callback or cancellation parameter ABI does not match callable signatures",
                ));
            }
            if export.convention != CallingConvention::Value
                && signatures.iter().any(|signature| match signature {
                    SemanticType::Callable {
                        result, receiver, ..
                    } => {
                        delivery_for(graph, *result) != Some(export.delivery)
                            || export.convention == CallingConvention::Method && receiver.is_none()
                            || export.convention != CallingConvention::Method && receiver.is_some()
                    }
                    _ => true,
                })
            {
                errors.push(error("SES-F0202", "runtime receiver or delivery ABI differs from the resolved declaration signature"));
            }
            for callback in &export.callbacks {
                let compatible = signatures.iter().all(|signature| {
                    let SemanticType::Callable { parameters, .. } = signature else {
                        return false;
                    };
                    let Some(parameter) = parameters.get(callback.parameter_index as usize) else {
                        return false;
                    };
                    let callback_types = callable_types(graph, parameter.type_ref);
                    !callback_types.is_empty()
                        && callback_types.iter().all(|signature| match signature {
                            SemanticType::Callable {
                                result, receiver, ..
                            } => {
                                receiver.is_none()
                                    && delivery_for(graph, *result) == Some(callback.delivery)
                            }
                            _ => false,
                        })
                });
                if !compatible {
                    errors.push(error("SES-F0202", "callback ABI does not match a resolved callable parameter and result delivery"));
                }
            }
            let mut callback_indices = BTreeSet::new();
            if export
                .callbacks
                .iter()
                .any(|cb| !callback_indices.insert(cb.parameter_index))
                || matches!(&export.cancellation, CancellationAbi::HostContract(identity) if identity.trim().is_empty())
            {
                errors.push(error(
                    "SES-F0202",
                    "callback or cancellation ABI identity is invalid",
                ));
            }
            for (available, artifact) in &export.targets {
                if !artifact_valid(artifact)
                    || *available == RuntimeTarget::Browser
                        && artifact.exact_identity.starts_with("node:")
                {
                    errors.push(error(
                        "SES-F0202",
                        "runtime artifact identity or target availability is invalid",
                    ));
                }
            }
            match export.targets.get(&target) {
                None => errors.push(error(
                    "SES-F0203",
                    "runtime export is unavailable for selected target",
                )),
                Some(expected) if artifacts.get(symbol) != Some(expected) => errors.push(error(
                    "SES-F0201",
                    "runtime artifact exact identity or content digest is stale",
                )),
                Some(_) => {}
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// An ordered JSON object is required: conditional exports use package property
/// order, not a fixed preference and not lexicographically sorted map keys.
#[derive(Clone, Debug)]
enum ExportValue {
    String(String),
    Object(Vec<(String, ExportValue)>),
    Array(Vec<ExportValue>),
    Null,
    Other,
}
impl<'de> Deserialize<'de> for ExportValue {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ExportValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a JSON export value")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(ExportValue::String(v.to_owned()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(ExportValue::String(v))
            }
            fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<Self::Value, E> {
                Ok(ExportValue::Other)
            }
            fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<Self::Value, E> {
                Ok(ExportValue::Other)
            }
            fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<Self::Value, E> {
                Ok(ExportValue::Other)
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Ok(ExportValue::Other)
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(ExportValue::Null)
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> Result<Self::Value, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = a.next_element()? {
                    items.push(item);
                }
                Ok(ExportValue::Array(items))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut a: A,
            ) -> Result<Self::Value, A::Error> {
                let mut items = Vec::new();
                let mut keys = BTreeSet::new();
                while let Some((key, value)) = a.next_entry::<String, ExportValue>()? {
                    if !keys.insert(key.clone()) {
                        return Err(serde::de::Error::custom("duplicate package export key"));
                    }
                    items.push((key, value));
                }
                Ok(ExportValue::Object(items))
            }
        }
        d.deserialize_any(Visitor)
    }
}
fn field<'a>(value: &'a ExportValue, name: &str) -> Option<&'a ExportValue> {
    match value {
        ExportValue::Object(items) => items
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value),
        _ => None,
    }
}
fn text(value: &ExportValue) -> Option<&str> {
    match value {
        ExportValue::String(text) => Some(text),
        _ => None,
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedPackageExport {
    pub name: String,
    pub version: String,
    pub path: String,
}
fn valid_package_target(path: &str) -> bool {
    path.starts_with("./")
        && path.len() > 2
        && !path.contains(['\\', '?', '#', '*', '%'])
        && path[2..]
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".." && part != "node_modules")
}
fn select_condition<'a>(
    value: &'a ExportValue,
    target: RuntimeTarget,
) -> Result<Option<&'a str>, &'static str> {
    match value {
        ExportValue::String(path) => Ok(Some(path)),
        ExportValue::Null => Err("package export is explicitly unavailable"),
        ExportValue::Object(items) => {
            if items
                .iter()
                .any(|(key, _)| key.starts_with('.') || key.parse::<u32>().is_ok())
            {
                return Err("invalid conditional export object");
            }
            for (condition, value) in items {
                if target.condition(condition) {
                    if let Some(path) = select_condition(value, target)? {
                        return Ok(Some(path));
                    }
                }
            }
            Ok(None)
        }
        ExportValue::Array(items) => {
            for value in items {
                match select_condition(value, target) {
                    Ok(Some(path)) if valid_package_target(path) => return Ok(Some(path)),
                    Ok(None) | Ok(Some(_)) => continue,
                    Err(error) if !matches!(value, ExportValue::Null) => {
                        let _ = error;
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            }
            Ok(None)
        }
        ExportValue::Other => Err("unsupported package export value"),
    }
}
/// Host-neutral package export selection. Filesystem containment, loading and
/// content hashing remain host responsibilities. Unsupported browser mappings
/// fail explicitly; this API never manufactures host polyfills.
pub fn select_package_export(
    source: &str,
    package_name: &str,
    subpath: Option<&str>,
    target: RuntimeTarget,
) -> Result<SelectedPackageExport, RuntimeLinkageDiagnostic> {
    let error = |reason| RuntimeLinkageDiagnostic::new("SES-F0203", Some(target), reason);
    let package: ExportValue = serde_json::from_str(source)
        .map_err(|e| error(format!("invalid package manifest: {e}")))?;
    let name = field(&package, "name")
        .and_then(text)
        .unwrap_or(package_name);
    if name != package_name {
        return Err(error(
            "runtime package name differs from declared package identity".to_owned(),
        ));
    }
    let version = field(&package, "version")
        .and_then(text)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| error("runtime package has no exact version".to_owned()))?;
    let key = subpath.map_or(".".to_owned(), |p| format!("./{p}"));
    let has_exports = field(&package, "exports").is_some();
    let path = if let Some(exports) = field(&package, "exports") {
        let selected = match exports {
            ExportValue::Object(items) if items.iter().any(|(key, _)| key.starts_with('.')) => {
                if items.iter().any(|(key, _)| !key.starts_with('.')) {
                    return Err(error(
                        "package exports mix subpaths and conditions".to_owned(),
                    ));
                }
                items
                    .iter()
                    .find(|(name, _)| name == &key)
                    .map(|(_, value)| value)
            }
            _ => (key == ".").then_some(exports),
        }
        .ok_or_else(|| error(format!("package does not export {key}")))?;
        select_condition(selected, target)
            .map_err(|e| error(e.to_owned()))?
            .ok_or_else(|| error("package has no export for selected runtime target".to_owned()))?
            .to_owned()
    } else if let Some(path) = subpath {
        format!("./{path}")
    } else {
        let legacy = if target == RuntimeTarget::Browser {
            field(&package, "browser")
        } else {
            None
        };
        if matches!(legacy, Some(ExportValue::Other | ExportValue::Null)) {
            return Err(error(
                "package explicitly disables browser runtime".to_owned(),
            ));
        }
        if matches!(legacy, Some(ExportValue::Object(_))) {
            return Err(error(
                "browser replacement map requires an explicit target export".to_owned(),
            ));
        }
        legacy
            .and_then(text)
            .or_else(|| field(&package, "module").and_then(text))
            .or_else(|| field(&package, "main").and_then(text))
            .unwrap_or("./index.js")
            .to_owned()
    };
    if has_exports && !path.starts_with("./") {
        return Err(error("package export target must start with ./".to_owned()));
    }
    let path = if path.starts_with("./") {
        path
    } else {
        format!("./{path}")
    };
    if !valid_package_target(&path) {
        return Err(error(
            "runtime export path is not a contained package-relative file".to_owned(),
        ));
    }
    Ok(SelectedPackageExport {
        name: name.to_owned(),
        version: version.to_owned(),
        path,
    })
}

// Deliberately follow only callable wrappers, never parameters/results or the
// graph's general reference closure: those edges do not establish ownership.
fn callable_closure(graph: &SemanticGraph, root: TypeId) -> BTreeSet<TypeId> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        match graph.types.get(&id) {
            Some(SemanticType::Generic { body, .. }) => pending.push(*body),
            Some(SemanticType::Overload { signatures }) => pending.extend(signatures),
            Some(SemanticType::Reference { symbol }) => {
                if let Some(symbol) = graph.symbols.get(symbol) {
                    pending.push(symbol.type_ref);
                }
            }
            _ => {}
        }
    }
    seen
}
fn callable_types(graph: &SemanticGraph, root: TypeId) -> Vec<&SemanticType> {
    callable_closure(graph, root)
        .into_iter()
        .filter_map(|id| graph.types.get(&id))
        .filter(|node| matches!(node, SemanticType::Callable { .. }))
        .collect()
}
fn owner_roots(graph: &SemanticGraph, root: TypeId) -> Vec<TypeId> {
    callable_closure(graph, root)
        .into_iter()
        .filter(|id| {
            !matches!(
                graph.types.get(id),
                Some(
                    SemanticType::Generic { .. }
                        | SemanticType::Reference { .. }
                        | SemanticType::Overload { .. }
                )
            )
        })
        .collect()
}
fn owned_role(
    graph: &SemanticGraph,
    owner: TypeId,
    selected: TypeId,
    convention: CallingConvention,
) -> bool {
    if matches!(
        convention,
        CallingConvention::Function | CallingConvention::Value
    ) && callable_closure(graph, owner).contains(&selected)
    {
        if convention == CallingConvention::Value || !callable_types(graph, selected).is_empty() {
            return true;
        }
    }
    for root in owner_roots(graph, owner) {
        let Some(SemanticType::Nominal {
            instance,
            constructors,
            constructible,
            static_members,
            ..
        }) = graph.types.get(&root)
        else {
            continue;
        };
        match convention {
            CallingConvention::Constructor if *constructible => {
                if constructors
                    .iter()
                    .any(|signature| callable_closure(graph, *signature).contains(&selected))
                {
                    return true;
                }
            }
            CallingConvention::Method => {
                let instance_members = owner_roots(graph, *instance)
                    .into_iter()
                    .filter_map(|id| match graph.types.get(&id) {
                        Some(SemanticType::Shape { fields, .. }) => Some(fields.values()),
                        _ => None,
                    })
                    .flatten();
                if instance_members
                    .chain(static_members.values())
                    .any(|field| {
                        callable_closure(graph, field.type_ref).contains(&selected)
                            && !callable_types(graph, selected).is_empty()
                    })
                {
                    return true;
                }
            }
            CallingConvention::Function => {
                if static_members.values().any(|field| {
                    callable_closure(graph, field.type_ref).contains(&selected)
                        && !callable_types(graph, selected).is_empty()
                }) {
                    return true;
                }
            }
            CallingConvention::Value => {
                if static_members
                    .values()
                    .any(|field| field.type_ref == selected)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
fn delivery_for(graph: &SemanticGraph, root: TypeId) -> Option<RuntimeDelivery> {
    let mut pending = vec![root];
    let mut seen = BTreeSet::new();
    let mut deliveries = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        match graph.types.get(&id)? {
            SemanticType::Async { .. } => {
                deliveries.insert(true);
            }
            SemanticType::Reference { symbol } => {
                pending.push(graph.symbols.get(symbol)?.type_ref);
            }
            SemanticType::Union { members } => pending.extend(members),
            SemanticType::Primitive {
                domain:
                    crate::PrimitiveDomain::Unknown
                    | crate::PrimitiveDomain::Dynamic
                    | crate::PrimitiveDomain::Object,
            }
            | SemanticType::Unresolved { .. }
            | SemanticType::Parameter { .. }
            | SemanticType::Projection { .. }
            | SemanticType::Apply { .. } => return None,
            _ => {
                deliveries.insert(false);
            }
        }
    }
    match deliveries.len() {
        1 => Some(if deliveries.contains(&true) {
            RuntimeDelivery::Promise
        } else {
            RuntimeDelivery::Synchronous
        }),
        _ => None,
    }
}
