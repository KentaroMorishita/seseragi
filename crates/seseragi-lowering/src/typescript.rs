use crate::effect_ops::runtime_effect_operation;
use crate::{CoreExpr, CoreModule, CoreStatement, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_syntax::Visibility;

mod names;
mod runtime;

use names::safe_identifier;
use runtime::{
    collect_expr_runtime_imports, collect_expr_runtime_requirements,
    collect_type_runtime_requirement, lower_core_parameter_to_typescript, type_ref_from_core_expr,
    type_ref_from_core_type,
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
    Maybe { element: Box<TypeScriptType> },
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
    Await {
        value: Box<TypeScriptExpr>,
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
        type_ref: TypeScriptType,
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
        CoreExpr::Call {
            callee, arguments, ..
        } => TypeScriptExpr::Call {
            // Core symbols are module-qualified. Generated TypeScript keeps
            // declarations module-local, so calls use the same local-name
            // boundary as declarations and parameters.
            callee: local_name(&callee),
            arguments: arguments
                .into_iter()
                .map(lower_core_expr_to_typescript)
                .collect(),
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
        } => {
            // The target ABI, rather than Core IR, decides whether an operation
            // must be awaited.  That keeps the language-level effect operation
            // independent from the JavaScript runtime calling convention.
            let runtime_operation = runtime_effect_operation(&operation);
            let call = TypeScriptExpr::Call {
                callee: runtime_operation
                    .map(|operation| operation.local_name.to_owned())
                    .unwrap_or_else(|| safe_identifier(&operation)),
                arguments: arguments
                    .into_iter()
                    .map(lower_core_expr_to_typescript)
                    .collect(),
            };
            if runtime_operation.is_some_and(|operation| operation.await_result) {
                TypeScriptExpr::Await {
                    value: Box::new(call),
                }
            } else {
                call
            }
        }
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
            type_ref,
            value,
            origin,
        } => TypeScriptStatement::Const {
            name: safe_identifier(&name),
            type_ref: type_ref_from_core_type(&type_ref),
            initializer: lower_core_expr_to_typescript(value),
            origin,
        },
    }
}

fn typescript_expr_contains_await(expr: &TypeScriptExpr) -> bool {
    match expr {
        TypeScriptExpr::Await { .. } => true,
        TypeScriptExpr::Binary { left, right, .. } => {
            typescript_expr_contains_await(left) || typescript_expr_contains_await(right)
        }
        TypeScriptExpr::Call { arguments, .. } => {
            arguments.iter().any(typescript_expr_contains_await)
        }
        TypeScriptExpr::Sequence { statements, result } => {
            statements.iter().any(typescript_statement_contains_await)
                || typescript_expr_contains_await(result)
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::String { .. }
        | TypeScriptExpr::Boolean { .. }
        | TypeScriptExpr::Identifier { .. } => false,
    }
}

fn typescript_statement_contains_await(statement: &TypeScriptStatement) -> bool {
    match statement {
        TypeScriptStatement::Effect { value } => typescript_expr_contains_await(value),
        TypeScriptStatement::Const { initializer, .. } => {
            typescript_expr_contains_await(initializer)
        }
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

fn is_false(value: &bool) -> bool {
    !*value
}
