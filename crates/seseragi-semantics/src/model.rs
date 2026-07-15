use serde::{Deserialize, Serialize};
use seseragi_syntax::{
    ByteSpan, InterfaceDependency, InterfaceExport, InterfaceInstance, InterfaceOperator,
    Visibility,
};

mod resolved;
mod typed_interface;

pub use resolved::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedModule {
    pub schema: u32,
    pub stage: String,
    pub source: String,
    pub module: String,
    /// External nominal types referenced by this module and needed by later
    /// target-specific import selection. This is a module import set, not an
    /// occurrence-level replacement for resolved type symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_type_bindings: Vec<ExternalTypeBinding>,
    /// Source-module dependency edges whose identities were fixed by project
    /// linking. Edges remain present even when they contain no runtime value
    /// binding, while individual imports retain canonical symbol identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_dependencies: Vec<TypedModuleDependency>,
    /// Trait instances selected by semantic analysis. Later stages consume
    /// this evidence instead of rediscovering instances from ADT shape.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<TypedInstance>,
    pub declarations: Vec<TypedDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTypeBinding {
    pub spelling: String,
    pub canonical: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ExternalTypeProvider>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTypeProvider {
    pub module: String,
    pub export: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTraitBinding {
    pub spelling: String,
    pub canonical: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedModuleDependency {
    pub specifier: String,
    pub module: String,
    pub origin: ByteSpan,
    pub imports: Vec<TypedModuleImport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedModuleImport {
    pub namespace: String,
    pub imported: String,
    pub local: String,
    pub canonical: String,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedModuleInterface {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    pub source: String,
    pub dependencies: Vec<InterfaceDependency>,
    pub exports: Vec<InterfaceExport>,
    pub operators: Vec<InterfaceOperator>,
    pub instances: Vec<InterfaceInstance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedDecl {
    Adt {
        symbol: String,
        name: String,
        visibility: Visibility,
        opaque: bool,
        type_parameters: Vec<String>,
        variants: Vec<TypedAdtVariant>,
        origin: ByteSpan,
    },
    Let {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        scheme: TypedScheme,
        value: TypedExpr,
    },
    Fn {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        scheme: TypedScheme,
        parameters: Vec<TypedParameter>,
        body: TypedExpr,
    },
    EffectFn {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        #[serde(default, skip_serializing_if = "is_false")]
        inferred_contract: bool,
        parameters: Vec<TypedParameter>,
        effect: TypedEffect,
        body: TypedExpr,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedAdtVariant {
    pub symbol: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<TypedType>,
    pub scheme: TypedScheme,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedScheme {
    pub type_parameters: Vec<String>,
    pub constraints: Vec<TypedConstraint>,
    #[serde(rename = "type")]
    pub type_ref: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedConstraint {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<TypedType>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedCallEvidence {
    pub constraint: TypedConstraint,
    pub evidence: TypedInstanceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedTraitDispatch {
    pub trait_identity: String,
    pub method: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedInstance {
    pub identity: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<String>,
    /// Ordered arguments of the trait application in the instance head.
    /// Keeping all arguments prevents multi-parameter traits from collapsing
    /// into the first nominal type used by current runtime dictionaries.
    pub arguments: Vec<TypedType>,
    /// Canonical primary nominal type used by specialized runtime consumers.
    /// General multi-parameter instances are identified by `identity` and
    /// `arguments` and do not have to invent a single primary type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_identity: Option<String>,
    pub constraints: Vec<TypedConstraint>,
    pub origin: ByteSpan,
    pub implementation: TypedInstanceImplementation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedInstanceImplementation {
    DerivedShow {
        adt_symbol: String,
        payload_evidence: Vec<TypedShowPayloadEvidence>,
    },
    UserDefined {
        methods: Vec<TypedInstanceMethod>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedInstanceMethod {
    pub name: String,
    pub scheme: TypedScheme,
    pub parameters: Vec<TypedParameter>,
    pub body: TypedExpr,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedShowPayloadEvidence {
    pub variant_symbol: String,
    pub type_identity: String,
    pub evidence: TypedInstanceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedInstanceEvidence {
    Local {
        identity: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_arguments: Vec<TypedType>,
    },
    Imported {
        identity: String,
        provider_module: String,
    },
    Standard {
        identity: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedType {
    Named {
        name: String,
        arguments: Vec<TypedType>,
    },
    ExternalNamed {
        name: String,
        canonical: String,
        arguments: Vec<TypedType>,
    },
    Hole,
    Record {
        closed: bool,
        fields: Vec<TypedRecordField>,
    },
    Tuple {
        elements: Vec<TypedType>,
    },
    Function {
        parameter: Box<TypedType>,
        result: Box<TypedType>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedRecordField {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_ref: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedParameter {
    ImplicitUnit {
        #[serde(rename = "type")]
        type_ref: TypedType,
    },
    Named {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedEffect {
    pub environment: TypedType,
    pub failure: TypedType,
    pub success: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedExpr {
    Unit {
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Integer {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    String {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Boolean {
        value: bool,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Variable {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<TypedCallEvidence>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Call {
        callee: String,
        arguments: Vec<TypedExpr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<TypedCallEvidence>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trait_dispatch: Option<TypedTraitDispatch>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Tuple {
        elements: Vec<TypedExpr>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Array {
        elements: Vec<TypedExpr>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Binary {
        operator: String,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<TypedCallEvidence>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    If {
        condition: Box<TypedExpr>,
        then_branch: Box<TypedExpr>,
        else_branch: Box<TypedExpr>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Match {
        scrutinee: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
        #[serde(default, skip_serializing_if = "is_false")]
        exhaustive: bool,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    EffectCall {
        operation: String,
        effect: TypedEffect,
        arguments: Vec<TypedExpr>,
        origin: ByteSpan,
    },
    EffectInvoke {
        callee: String,
        effect: TypedEffect,
        arguments: Vec<TypedExpr>,
        origin: ByteSpan,
    },
    DoBlock {
        statements: Vec<TypedDoStatement>,
        result: Box<TypedExpr>,
        origin: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedMatchArm {
    pub pattern: TypedPattern,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<TypedExpr>,
    pub body: TypedExpr,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedPattern {
    Integer {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    String {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Boolean {
        value: bool,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Wildcard {
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Binding {
        symbol: SymbolId,
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Constructor {
        symbol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        argument: Option<Box<TypedPattern>>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Tuple {
        elements: Vec<TypedPattern>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Invalid {
        origin: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedDoStatement {
    Effect {
        value: TypedExpr,
    },
    PureLet {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        value: TypedExpr,
        origin: ByteSpan,
    },
    Bind {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        value: TypedExpr,
        origin: ByteSpan,
    },
}

pub(crate) fn unit_type() -> TypedType {
    TypedType::Named {
        name: "Unit".to_owned(),
        arguments: Vec::new(),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}
