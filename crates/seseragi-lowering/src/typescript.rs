use crate::effect_ops::runtime_effect_operation;
use crate::{CoreExpr, CoreModule, CoreStatement, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_syntax::Visibility;

mod names;
mod runtime;

use names::safe_identifier;
use runtime::{
    collect_expr_runtime_imports, collect_expr_runtime_requirements,
    collect_type_runtime_requirement, lower_core_parameter_to_typescript, render_type_name,
    type_ref_from_core_expr,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    pub runtime_requirements: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<TypeScriptImport>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<TypeScriptBinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<TypeScriptFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptImport {
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
    Binary {
        operator: String,
        left: Box<TypeScriptExpr>,
        right: Box<TypeScriptExpr>,
    },
    Call {
        callee: String,
        arguments: Vec<TypeScriptExpr>,
    },
    Sequence {
        statements: Vec<TypeScriptStatement>,
        result: Box<TypeScriptExpr>,
    },
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
    Const {
        name: String,
        #[serde(rename = "type")]
        type_name: String,
        initializer: TypeScriptExpr,
        origin: SourceSpan,
    },
}

pub fn lower_core_module_to_typescript_ir(module: CoreModule) -> TypeScriptModule {
    let mut runtime_requirements = Vec::new();
    let mut imports = Vec::new();
    let bindings = module
        .bindings
        .into_iter()
        .map(|binding| {
            collect_expr_runtime_requirements(&binding.value, &mut runtime_requirements);
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
                collect_type_runtime_requirement(&parameter.type_name, &mut runtime_requirements);
            }
            collect_expr_runtime_requirements(&function.body, &mut runtime_requirements);
            collect_expr_runtime_imports(&function.body, &mut imports);
            TypeScriptFunction::ConstFunction {
                exported: function.visibility == Visibility::Public,
                name: local_name(&function.symbol),
                parameters: function
                    .parameters
                    .into_iter()
                    .map(lower_core_parameter_to_typescript)
                    .collect(),
                body: lower_core_expr_to_typescript(function.body),
                origin: function.origin,
            }
        })
        .collect();

    TypeScriptModule {
        schema: module.schema,
        stage: "typescript-ir".to_owned(),
        module: module.module,
        runtime_requirements,
        imports,
        bindings,
        functions,
    }
}

fn lower_core_expr_to_typescript(expr: CoreExpr) -> TypeScriptExpr {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptExpr::Undefined,
        CoreExpr::Int64 { value, .. } => TypeScriptExpr::Bigint { value },
        CoreExpr::String { value, .. } => TypeScriptExpr::String { value },
        CoreExpr::Boolean { value, .. } => TypeScriptExpr::Boolean { value },
        CoreExpr::Variable { name, .. } => TypeScriptExpr::Identifier {
            name: safe_identifier(&name),
        },
        CoreExpr::Binary {
            operator,
            left,
            right,
            ..
        } => TypeScriptExpr::Binary {
            operator,
            left: Box::new(lower_core_expr_to_typescript(*left)),
            right: Box::new(lower_core_expr_to_typescript(*right)),
        },
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => TypeScriptExpr::Call {
            callee: runtime_effect_operation(&operation)
                .map(|operation| operation.local_name.to_owned())
                .unwrap_or_else(|| safe_identifier(&operation)),
            arguments: arguments
                .into_iter()
                .map(lower_core_expr_to_typescript)
                .collect(),
        },
        CoreExpr::Sequence {
            statements, result, ..
        } => TypeScriptExpr::Sequence {
            statements: statements
                .into_iter()
                .map(lower_core_statement_to_typescript)
                .collect(),
            result: Box::new(lower_core_expr_to_typescript(*result)),
        },
    }
}

fn lower_core_statement_to_typescript(statement: CoreStatement) -> TypeScriptStatement {
    match statement {
        CoreStatement::Effect { value } => TypeScriptStatement::Effect {
            value: lower_core_expr_to_typescript(value),
        },
        CoreStatement::Bind {
            name,
            type_name,
            value,
            origin,
        } => TypeScriptStatement::Const {
            name: safe_identifier(&name),
            type_name: render_type_name(&type_name).to_owned(),
            initializer: lower_core_expr_to_typescript(value),
            origin,
        },
    }
}

fn local_name(symbol: &str) -> String {
    symbol
        .rsplit_once("::")
        .map(|(_, name)| safe_identifier(name))
        .unwrap_or_else(|| safe_identifier(symbol))
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
