use crate::{source_span, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::{ExternalTypeBinding, TypedDecl, TypedForeignMember, TypedModule};
use seseragi_syntax::{ByteSpan, ForeignCallKind, ForeignCallMode, TypeParameter, Visibility};
use std::collections::BTreeSet;

mod adt;
mod decision;
mod expr;
mod instances;
mod types;

use adt::{lower_adt, AdtDeclInput};
pub use adt::{CoreAdt, CoreAdtVariant};
pub use decision::{
    CoreDecisionBinding, CoreDecisionBranch, CoreDecisionProjection, CoreDecisionTest,
};
use expr::{lower_effect_body, lower_expr, lower_parameter, lower_top_level_pattern_binding};
use instances::lower_instances;
pub use instances::{
    CoreInstance, CoreInstanceConstraint, CoreInstanceEvidence, CoreInstanceImplementation,
    CoreInstanceMethod, CoreShowPayloadEvidence,
};
pub use types::{CoreRecordField, CoreType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub foreign_modules: Vec<CoreForeignModule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_type_bindings: Vec<ExternalTypeBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_dependencies: Vec<CoreModuleDependency>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adts: Vec<CoreAdt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<CoreAlias>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structs: Vec<CoreStruct>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<CoreInstance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<CoreBinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<CoreFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreForeignModule {
    pub visibility: Visibility,
    pub language: String,
    pub specifier: String,
    pub pure_load: bool,
    pub members: Vec<CoreForeignMember>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreForeignMember {
    Function {
        mode: ForeignCallMode,
        call_kind: ForeignCallKind,
        symbol: String,
        name: String,
        host_name: String,
        parameters: Vec<CoreParameter>,
        return_type: CoreType,
        origin: SourceSpan,
    },
    Value {
        symbol: String,
        name: String,
        host_name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    OpaqueType {
        symbol: String,
        name: String,
        origin: SourceSpan,
    },
    Namespace {
        symbol: String,
        name: String,
        host_name: String,
        members: Vec<CoreForeignMember>,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStruct {
    pub symbol: String,
    pub name: String,
    pub visibility: Visibility,
    pub opaque: bool,
    pub type_parameters: Vec<TypeParameter>,
    pub fields: Vec<CoreStructField>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreAlias {
    pub symbol: String,
    pub name: String,
    pub visibility: Visibility,
    pub type_parameters: Vec<TypeParameter>,
    pub target: CoreType,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStructField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: CoreType,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModuleDependency {
    pub specifier: String,
    pub module: String,
    pub origin: SourceSpan,
    pub imports: Vec<CoreModuleImport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModuleImport {
    pub namespace: String,
    pub imported: String,
    pub local: String,
    pub canonical: String,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreBinding {
    pub symbol: String,
    pub visibility: Visibility,
    pub origin: SourceSpan,
    pub value: CoreExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreFunction {
    pub symbol: String,
    pub visibility: Visibility,
    pub origin: SourceSpan,
    /// Whether this declaration returns a cold Effect thunk rather than a
    /// pure value. This emitter-only marker is omitted from serialized Core
    /// IR so the artifact schema remains unchanged.
    #[serde(skip)]
    pub is_effect: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<TypeParameter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<CoreInstanceConstraint>,
    pub parameters: Vec<CoreParameter>,
    pub body: CoreExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreParameter {
    pub id: String,
    pub kind: String,
    #[serde(rename = "type")]
    pub type_ref: CoreType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreCallEvidence {
    pub constraint: CoreInstanceConstraint,
    pub evidence: CoreInstanceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreTraitDispatch {
    pub trait_identity: String,
    pub method: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreRecordValueItem {
    Field {
        name: String,
        value: CoreExpr,
        origin: SourceSpan,
    },
    Spread {
        value: CoreExpr,
        origin: SourceSpan,
    },
}

impl CoreRecordValueItem {
    pub fn value(&self) -> &CoreExpr {
        match self {
            Self::Field { value, .. } | Self::Spread { value, .. } => value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreTemplatePart {
    Text {
        value: String,
        origin: SourceSpan,
    },
    Interpolation {
        value: CoreExpr,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence: Option<CoreCallEvidence>,
        trait_identity: String,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreComprehensionClause {
    Generator {
        pattern: CorePattern,
        source: CoreExpr,
        evidence: CoreCallEvidence,
        origin: SourceSpan,
    },
    Guard {
        condition: CoreExpr,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CorePattern {
    Integer {
        value: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    String {
        value: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Boolean {
        value: bool,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Binding {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Wildcard {
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Constructor {
        symbol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        argument: Option<Box<CorePattern>>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Tuple {
        elements: Vec<CorePattern>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Array {
        elements: Vec<CorePattern>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rest: Option<Box<CorePattern>>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    List {
        elements: Vec<CorePattern>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rest: Option<Box<CorePattern>>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Record {
        fields: Vec<CoreRecordPatternField>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Invalid {
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreRecordPatternField {
    pub name: String,
    pub pattern: CorePattern,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreExpr {
    Unit {
        origin: SourceSpan,
    },
    Integer {
        value: String,
        origin: SourceSpan,
    },
    Float64 {
        value: String,
        origin: SourceSpan,
    },
    String {
        value: String,
        origin: SourceSpan,
    },
    Template {
        parts: Vec<CoreTemplatePart>,
        origin: SourceSpan,
    },
    Boolean {
        value: bool,
        origin: SourceSpan,
    },
    Variable {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<CoreCallEvidence>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Call {
        callee: String,
        arguments: Vec<CoreExpr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<CoreCallEvidence>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deferred_evidence_parameters: Vec<CoreType>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deferred_evidence_type_constructor_parameters: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trait_dispatch: Option<CoreTraitDispatch>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Lambda {
        parameter: CoreParameter,
        body: Box<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Tuple {
        elements: Vec<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    FieldAccess {
        receiver: Box<CoreExpr>,
        field: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    OptionalFieldAccess {
        receiver: Box<CoreExpr>,
        field: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Record {
        items: Vec<CoreRecordValueItem>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Array {
        elements: Vec<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    List {
        elements: Vec<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    ArrayComprehension {
        element: Box<CoreExpr>,
        clauses: Vec<CoreComprehensionClause>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    ListComprehension {
        element: Box<CoreExpr>,
        clauses: Vec<CoreComprehensionClause>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Binary {
        operator: String,
        left: Box<CoreExpr>,
        right: Box<CoreExpr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<CoreCallEvidence>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Unary {
        operator: String,
        operand: Box<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    If {
        condition: Box<CoreExpr>,
        then_branch: Box<CoreExpr>,
        else_branch: Box<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Decision {
        scrutinee: Box<CoreExpr>,
        scrutinee_type: CoreType,
        branches: Vec<CoreDecisionBranch>,
        exhaustive: bool,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    EffectOperation {
        operation: String,
        requirements: CoreType,
        failure: CoreType,
        success: CoreType,
        arguments: Vec<CoreExpr>,
        origin: SourceSpan,
    },
    EffectInvoke {
        callee: String,
        requirements: CoreType,
        failure: CoreType,
        success: CoreType,
        arguments: Vec<CoreExpr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<CoreCallEvidence>,
        origin: SourceSpan,
    },
    Sequence {
        statements: Vec<CoreStatement>,
        result: Box<CoreExpr>,
        origin: SourceSpan,
    },
    MonadDo {
        statements: Vec<CoreMonadDoStatement>,
        result: Box<CoreExpr>,
        evidence: CoreCallEvidence,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreStatement {
    Effect {
        value: CoreExpr,
    },
    PureLet {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        value: CoreExpr,
        origin: SourceSpan,
    },
    Bind {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        value: CoreExpr,
        origin: SourceSpan,
    },
    LocalFunction {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<CoreInstanceConstraint>,
        parameters: Vec<CoreParameter>,
        body: CoreExpr,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreMonadDoStatement {
    Expression {
        value: CoreExpr,
    },
    PureLet {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        value: CoreExpr,
        origin: SourceSpan,
    },
    Bind {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        value: CoreExpr,
        origin: SourceSpan,
    },
}

pub fn lower_typed_module(module: TypedModule) -> CoreModule {
    let mut foreign_origins = Vec::new();
    let mut foreign_symbols = BTreeSet::new();
    for member in module
        .foreign_modules
        .iter()
        .flat_map(|foreign| foreign.members.iter())
    {
        collect_foreign_declaration_identity(member, &mut foreign_symbols, &mut foreign_origins);
    }
    let foreign_modules = module
        .foreign_modules
        .into_iter()
        .map(|foreign| CoreForeignModule {
            visibility: foreign.visibility,
            language: foreign.language,
            specifier: foreign.specifier,
            pure_load: foreign.pure_load,
            members: foreign
                .members
                .into_iter()
                .map(|member| lower_foreign_member(&module.source, member))
                .collect(),
            origin: source_span(&module.source, foreign.origin),
        })
        .collect();
    let module_dependencies = module
        .module_dependencies
        .into_iter()
        .map(|dependency| CoreModuleDependency {
            specifier: dependency.specifier,
            module: dependency.module,
            origin: source_span(&module.source, dependency.origin),
            imports: dependency
                .imports
                .into_iter()
                .map(|import| CoreModuleImport {
                    namespace: import.namespace,
                    imported: import.imported,
                    local: import.local,
                    canonical: import.canonical,
                    origin: source_span(&module.source, import.origin),
                })
                .collect(),
        })
        .collect();
    let instances = lower_instances(&module.source, module.instances);
    let mut adts = Vec::new();
    let mut aliases = Vec::new();
    let mut structs = Vec::new();
    let mut bindings = Vec::new();
    let mut functions = Vec::new();

    for declaration in module.declarations {
        let (declaration_symbol, declaration_origin) = match &declaration {
            TypedDecl::Alias { symbol, origin, .. }
            | TypedDecl::Adt { symbol, origin, .. }
            | TypedDecl::Struct { symbol, origin, .. }
            | TypedDecl::Fn { symbol, origin, .. }
            | TypedDecl::EffectFn { symbol, origin, .. } => (Some(symbol), *origin),
            TypedDecl::Let {
                bindings, origin, ..
            } => (bindings.first().map(|binding| &binding.symbol), *origin),
        };
        if declaration_symbol.is_some_and(|symbol| foreign_symbols.contains(symbol))
            || foreign_origins.contains(&declaration_origin)
        {
            continue;
        }
        match declaration {
            TypedDecl::Alias {
                symbol,
                name,
                visibility,
                type_parameters,
                target,
                origin,
            } => aliases.push(CoreAlias {
                symbol,
                name,
                visibility,
                type_parameters,
                target: types::lower_typed_type(target),
                origin: source_span(&module.source, origin),
            }),
            TypedDecl::Adt {
                symbol,
                name,
                visibility,
                opaque,
                type_parameters,
                variants,
                origin,
            } => adts.push(lower_adt(
                &module.source,
                AdtDeclInput {
                    symbol,
                    name,
                    visibility,
                    opaque,
                    type_parameters,
                    variants,
                    origin,
                },
            )),
            TypedDecl::Struct {
                symbol,
                name,
                visibility,
                opaque,
                type_parameters,
                fields,
                origin,
            } => structs.push(CoreStruct {
                symbol,
                name,
                visibility,
                opaque,
                type_parameters,
                fields: fields
                    .into_iter()
                    .map(|field| CoreStructField {
                        name: field.name,
                        type_ref: types::lower_typed_type(field.type_ref),
                        origin: source_span(&module.source, field.origin),
                    })
                    .collect(),
                origin: source_span(&module.source, origin),
            }),
            TypedDecl::Let {
                bindings: pattern_bindings,
                pattern,
                visibility,
                origin,
                value,
                ..
            } => bindings.extend(lower_top_level_pattern_binding(
                &module.source,
                &module.module,
                pattern_bindings,
                pattern,
                value,
                visibility,
                origin,
            )),
            TypedDecl::Fn {
                symbol,
                visibility,
                origin,
                scheme,
                parameters,
                body,
            } => {
                let constraint_identities = scheme.constraint_identities;
                functions.push(CoreFunction {
                    symbol,
                    visibility,
                    origin: source_span(&module.source, origin),
                    is_effect: false,
                    type_parameters: scheme.type_parameters,
                    constraints: scheme
                        .constraints
                        .into_iter()
                        .enumerate()
                        .map(|(index, constraint)| {
                            instances::lower_constraint_with_identity(
                                constraint,
                                constraint_identities.get(index).cloned().flatten(),
                            )
                        })
                        .collect(),
                    parameters: parameters
                        .into_iter()
                        .map(|parameter| lower_parameter(&parameter))
                        .collect(),
                    body: lower_expr(&module.source, body),
                })
            }
            TypedDecl::EffectFn {
                symbol,
                visibility,
                origin,
                inferred_contract: _,
                type_parameters,
                constraints,
                constraint_identities,
                parameters,
                effect: _,
                body,
            } => functions.push(CoreFunction {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                is_effect: true,
                type_parameters,
                constraints: constraints
                    .into_iter()
                    .enumerate()
                    .map(|(index, constraint)| {
                        instances::lower_constraint_with_identity(
                            constraint,
                            constraint_identities.get(index).cloned().flatten(),
                        )
                    })
                    .collect(),
                parameters: parameters
                    .into_iter()
                    .map(|parameter| lower_parameter(&parameter))
                    .collect(),
                body: lower_effect_body(&module.source, body),
            }),
        }
    }

    CoreModule {
        schema: module.schema,
        stage: "core-ir".to_owned(),
        module: module.module,
        foreign_modules,
        external_type_bindings: module.external_type_bindings,
        module_dependencies,
        adts,
        aliases,
        structs,
        instances,
        bindings,
        functions,
    }
}

fn collect_foreign_declaration_identity(
    member: &TypedForeignMember,
    symbols: &mut BTreeSet<String>,
    origins: &mut Vec<ByteSpan>,
) {
    let (symbol, origin, members) = match member {
        TypedForeignMember::Function { symbol, origin, .. }
        | TypedForeignMember::OpaqueType { symbol, origin, .. }
        | TypedForeignMember::Value { symbol, origin, .. } => (symbol, origin, None),
        TypedForeignMember::Namespace {
            symbol,
            origin,
            members,
            ..
        } => (symbol, origin, Some(members.as_slice())),
    };
    symbols.insert(symbol.clone());
    origins.push(*origin);
    for child in members.into_iter().flatten() {
        collect_foreign_declaration_identity(child, symbols, origins);
    }
}

fn lower_foreign_member(source: &str, member: TypedForeignMember) -> CoreForeignMember {
    match member {
        TypedForeignMember::Function {
            mode,
            call_kind,
            symbol,
            name,
            host_name,
            parameters,
            return_type,
            origin,
        } => CoreForeignMember::Function {
            mode,
            call_kind,
            symbol,
            name,
            host_name,
            parameters: parameters.iter().map(lower_parameter).collect(),
            return_type: types::lower_typed_type(return_type),
            origin: source_span(source, origin),
        },
        TypedForeignMember::OpaqueType {
            symbol,
            name,
            origin,
        } => CoreForeignMember::OpaqueType {
            symbol,
            name,
            origin: source_span(source, origin),
        },
        TypedForeignMember::Value {
            symbol,
            name,
            host_name,
            type_ref,
            origin,
        } => CoreForeignMember::Value {
            symbol,
            name,
            host_name,
            type_ref: types::lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedForeignMember::Namespace {
            symbol,
            name,
            host_name,
            members,
            origin,
        } => CoreForeignMember::Namespace {
            symbol,
            name,
            host_name,
            members: members
                .into_iter()
                .map(|member| lower_foreign_member(source, member))
                .collect(),
            origin: source_span(source, origin),
        },
    }
}
