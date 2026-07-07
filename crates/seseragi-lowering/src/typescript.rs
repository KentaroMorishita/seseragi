use crate::{CoreExpr, CoreModule, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_syntax::Visibility;

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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptExpr {
    Bigint {
        value: String,
    },
    String {
        value: String,
    },
    Call {
        callee: String,
        arguments: Vec<TypeScriptExpr>,
    },
}

pub fn lower_core_module_to_typescript_ir(module: CoreModule) -> TypeScriptModule {
    let mut runtime_requirements = Vec::new();
    let mut imports = Vec::new();
    let bindings = module
        .bindings
        .into_iter()
        .map(|binding| {
            if matches!(binding.value, CoreExpr::Int64 { .. }) {
                push_unique(&mut runtime_requirements, "core.int64");
            }
            TypeScriptBinding::Const {
                exported: binding.visibility == Visibility::Public,
                name: local_name(&binding.symbol),
                type_ref: TypeScriptType::Bigint,
                initializer: lower_core_expr_to_typescript(binding.value),
                origin: binding.origin,
            }
        })
        .collect();
    let functions = module
        .functions
        .into_iter()
        .map(|function| {
            push_unique(&mut runtime_requirements, "core.unit");
            if function_body_uses_console_println(&function.body) {
                push_unique(&mut runtime_requirements, "effect.console.println");
                imports.push(TypeScriptImport {
                    feature: "effect.console.println".to_owned(),
                    local: "_ssrg_console_println".to_owned(),
                });
            }
            TypeScriptFunction::ConstFunction {
                exported: function.visibility == Visibility::Public,
                name: local_name(&function.symbol),
                parameters: function
                    .parameters
                    .into_iter()
                    .map(|_| TypeScriptParameter {
                        name: "_unit".to_owned(),
                        type_name: "undefined".to_owned(),
                        implicit: true,
                    })
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
        CoreExpr::Unit { .. } => TypeScriptExpr::Call {
            callee: "undefined".to_owned(),
            arguments: Vec::new(),
        },
        CoreExpr::Int64 { value, .. } => TypeScriptExpr::Bigint { value },
        CoreExpr::String { value, .. } => TypeScriptExpr::String { value },
        CoreExpr::EffectOperation {
            operation,
            arguments,
            ..
        } => TypeScriptExpr::Call {
            callee: match operation.as_str() {
                "console.println" => "_ssrg_console_println".to_owned(),
                other => other.to_owned(),
            },
            arguments: arguments
                .into_iter()
                .map(lower_core_expr_to_typescript)
                .collect(),
        },
    }
}

fn function_body_uses_console_println(expr: &CoreExpr) -> bool {
    match expr {
        CoreExpr::EffectOperation { operation, .. } => operation == "console.println",
        _ => false,
    }
}

fn local_name(symbol: &str) -> String {
    symbol
        .rsplit_once("::")
        .map(|(_, name)| name.to_owned())
        .unwrap_or_else(|| symbol.to_owned())
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}
