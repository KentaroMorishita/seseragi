use crate::{CoreModule, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_syntax::Visibility;

mod adt;
mod decision;
mod expr;
mod imports;
mod names;
mod runtime;
mod type_imports;
pub(crate) mod types;

use adt::lower_core_adt_to_typescript;
use expr::{lower_core_expr_to_typescript, typescript_expr_contains_await};
use imports::freshen_runtime_imports;
use names::local_name;
use runtime::{
    collect_expr_runtime_imports, collect_expr_runtime_requirements,
    collect_type_runtime_requirement,
};
use type_imports::collect_module_type_imports;
use types::{lower_core_parameter_to_typescript, type_ref_from_core_expr};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    pub runtime_requirements: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<TypeScriptImport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_imports: Vec<TypeScriptTypeImport>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adts: Vec<TypeScriptAdt>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<TypeScriptBinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<TypeScriptFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptAdt {
    pub exported: bool,
    pub name: String,
    pub type_parameters: Vec<String>,
    pub variants: Vec<TypeScriptAdtVariant>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptAdtVariant {
    pub exported: bool,
    pub name: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<TypeScriptType>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptImport {
    pub feature: String,
    pub local: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptTypeImport {
    pub feature: String,
    pub local: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptBinding {
    Const {
        exported: bool,
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
        initializer: TypeScriptExpr,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptFunction {
    ConstFunction {
        exported: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        is_async: bool,
        name: String,
        parameters: Vec<TypeScriptParameter>,
        body: TypeScriptExpr,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub implicit: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptType {
    Bigint,
    Boolean,
    String,
    Undefined,
    Unknown,
    Reference {
        name: String,
        arguments: Vec<TypeScriptType>,
    },
    Maybe {
        element: Box<TypeScriptType>,
    },
    Either {
        error: Box<TypeScriptType>,
        value: Box<TypeScriptType>,
    },
    Tuple {
        elements: Vec<TypeScriptType>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptExpr {
    Undefined,
    Bigint {
        value: String,
    },
    String {
        value: String,
    },
    Boolean {
        value: bool,
    },
    Identifier {
        name: String,
    },
    RuntimeReference {
        name: String,
    },
    Tuple {
        elements: Vec<TypeScriptExpr>,
    },
    Binary {
        operator: String,
        left: Box<TypeScriptExpr>,
        right: Box<TypeScriptExpr>,
    },
    Conditional {
        condition: Box<TypeScriptExpr>,
        then_branch: Box<TypeScriptExpr>,
        else_branch: Box<TypeScriptExpr>,
    },
    Decision {
        scrutinee: Box<TypeScriptExpr>,
        scrutinee_type: TypeScriptType,
        branches: Vec<TypeScriptDecisionBranch>,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
    },
    Call {
        callee: String,
        arguments: Vec<TypeScriptExpr>,
    },
    RuntimeCall {
        callee: String,
        arguments: Vec<TypeScriptExpr>,
    },
    Await {
        value: Box<TypeScriptExpr>,
    },
    Sequence {
        statements: Vec<TypeScriptStatement>,
        result: Box<TypeScriptExpr>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptDecisionBranch {
    pub tests: Vec<TypeScriptDecisionTest>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<TypeScriptDecisionBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<TypeScriptExpr>,
    pub value: TypeScriptExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptDecisionBinding {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeScriptType,
    pub path: Vec<TypeScriptDecisionProjection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptDecisionTest {
    BigintEquals {
        path: Vec<TypeScriptDecisionProjection>,
        value: String,
    },
    StringEquals {
        path: Vec<TypeScriptDecisionProjection>,
        value: String,
    },
    BooleanEquals {
        path: Vec<TypeScriptDecisionProjection>,
        value: bool,
    },
    TagEquals {
        path: Vec<TypeScriptDecisionProjection>,
        tag: String,
    },
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptDecisionProjection {
    TupleElement { index: usize },
    AdtPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptStatement {
    Effect {
        value: TypeScriptExpr,
    },
    PureLet {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
        initializer: TypeScriptExpr,
        origin: SourceSpan,
    },
    Const {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
        initializer: TypeScriptExpr,
        origin: SourceSpan,
    },
}

pub fn lower_core_module_to_typescript_ir(module: CoreModule) -> TypeScriptModule {
    let mut runtime_requirements = Vec::new();
    let mut imports = Vec::new();
    let mut type_imports = Vec::new();
    collect_module_type_imports(&module, &mut runtime_requirements, &mut type_imports);
    let adts = module
        .adts
        .into_iter()
        .map(|adt| lower_core_adt_to_typescript(adt, &mut runtime_requirements))
        .collect();
    let bindings = module
        .bindings
        .into_iter()
        .map(|binding| {
            collect_expr_runtime_requirements(&binding.value, &mut runtime_requirements);
            collect_expr_runtime_imports(&binding.value, &mut imports);
            TypeScriptBinding::Const {
                exported: binding.visibility == Visibility::Public,
                name: local_name(&binding.symbol),
                type_ref: type_ref_from_core_expr(&binding.value),
                initializer: lower_core_expr_to_typescript(binding.value),
                origin: binding.origin,
            }
        })
        .collect();
    let functions = module
        .functions
        .into_iter()
        .map(|function| {
            for parameter in &function.parameters {
                collect_type_runtime_requirement(&parameter.type_ref, &mut runtime_requirements);
            }
            collect_expr_runtime_requirements(&function.body, &mut runtime_requirements);
            collect_expr_runtime_imports(&function.body, &mut imports);
            let body = lower_core_expr_to_typescript(function.body);
            TypeScriptFunction::ConstFunction {
                exported: function.visibility == Visibility::Public,
                is_async: typescript_expr_contains_await(&body),
                name: local_name(&function.symbol),
                parameters: function
                    .parameters
                    .into_iter()
                    .map(lower_core_parameter_to_typescript)
                    .collect(),
                body,
                origin: function.origin,
            }
        })
        .collect();

    let mut typescript = TypeScriptModule {
        schema: module.schema,
        stage: "typescript-ir".to_owned(),
        module: module.module,
        runtime_requirements,
        imports,
        type_imports,
        adts,
        bindings,
        functions,
    };
    freshen_runtime_imports(&mut typescript);
    typescript
}

pub(super) fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

pub(super) fn push_import_unique(imports: &mut Vec<TypeScriptImport>, import: TypeScriptImport) {
    if !imports
        .iter()
        .any(|existing| existing.feature == import.feature && existing.local == import.local)
    {
        imports.push(import);
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}
