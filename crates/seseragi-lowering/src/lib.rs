use serde::{Deserialize, Serialize};
use seseragi_semantics::{TypedDecl, TypedEffect, TypedExpr, TypedModule, TypedType};
use seseragi_syntax::{ByteSpan, Visibility};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<CoreBinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<CoreFunction>,
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
    pub parameters: Vec<CoreParameter>,
    pub body: CoreExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreParameter {
    pub id: String,
    pub kind: String,
    #[serde(rename = "type")]
    pub type_name: String,
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
    Int64 {
        value: String,
        origin: SourceSpan,
    },
    String {
        value: String,
        origin: SourceSpan,
    },
    EffectOperation {
        operation: String,
        requirements: Vec<String>,
        failure: String,
        success: String,
        arguments: Vec<CoreExpr>,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    pub source: String,
    pub start: usize,
    pub end: usize,
}

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

pub fn lower_typed_module(module: TypedModule) -> CoreModule {
    let mut bindings = Vec::new();
    let mut functions = Vec::new();

    for declaration in module.declarations {
        match declaration {
            TypedDecl::Let {
                symbol,
                visibility,
                origin,
                value,
                ..
            } => bindings.push(CoreBinding {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                value: lower_expr(&module.source, value),
            }),
            TypedDecl::EffectFn {
                symbol,
                visibility,
                origin,
                parameters,
                effect,
                body,
            } => functions.push(CoreFunction {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                parameters: parameters
                    .into_iter()
                    .map(|_| CoreParameter {
                        id: "unit".to_owned(),
                        kind: "implicit".to_owned(),
                        type_name: "Unit".to_owned(),
                    })
                    .collect(),
                body: lower_effect_body(&module.source, effect, body),
            }),
        }
    }

    CoreModule {
        schema: module.schema,
        stage: "core-ir".to_owned(),
        module: module.module,
        bindings,
        functions,
    }
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

fn lower_effect_body(source: &str, effect: TypedEffect, body: TypedExpr) -> CoreExpr {
    match body {
        TypedExpr::EffectCall {
            operation,
            arguments,
            origin,
        } => CoreExpr::EffectOperation {
            operation: lower_effect_operation(&operation),
            requirements: effect_requirements(&effect),
            failure: type_name(&effect.failure),
            success: type_name(&effect.success),
            arguments: arguments
                .into_iter()
                .map(|argument| lower_expr(source, argument))
                .collect(),
            origin: source_span(source, origin),
        },
        expr => lower_expr(source, expr),
    }
}

fn lower_expr(source: &str, expr: TypedExpr) -> CoreExpr {
    match expr {
        TypedExpr::Unit { origin, .. } => CoreExpr::Unit {
            origin: source_span(source, origin),
        },
        TypedExpr::Integer { value, origin, .. } => CoreExpr::Int64 {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::String { value, origin, .. } => CoreExpr::String {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::EffectCall {
            operation,
            arguments,
            origin,
        } => CoreExpr::EffectOperation {
            operation: lower_effect_operation(&operation),
            requirements: Vec::new(),
            failure: "Never".to_owned(),
            success: "Unit".to_owned(),
            arguments: arguments
                .into_iter()
                .map(|argument| lower_expr(source, argument))
                .collect(),
            origin: source_span(source, origin),
        },
        TypedExpr::DoBlock { result, .. } => lower_expr(source, *result),
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

fn lower_effect_operation(operation: &str) -> String {
    match operation {
        "std/prelude::println" => "console.println".to_owned(),
        other => other.to_owned(),
    }
}

fn effect_requirements(effect: &TypedEffect) -> Vec<String> {
    match &effect.environment {
        TypedType::Record { fields, .. } => fields.iter().map(|field| field.name.clone()).collect(),
        _ => Vec::new(),
    }
}

fn type_name(type_ref: &TypedType) -> String {
    match type_ref {
        TypedType::Named { name, .. } => name.clone(),
        TypedType::Record { .. } => "Record".to_owned(),
    }
}

fn source_span(source: &str, span: ByteSpan) -> SourceSpan {
    SourceSpan {
        source: source.to_owned(),
        start: span.start,
        end: span.end,
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

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_semantics::type_module;

    #[test]
    fn lowers_public_let_to_core_binding() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);

        assert_eq!(core.stage, "core-ir");
        assert_eq!(core.module, "artifact/basic");
        assert_eq!(core.bindings.len(), 1);
        assert!(matches!(core.bindings[0].value, CoreExpr::Int64 { .. }));
        assert!(core.functions.is_empty());
    }

    #[test]
    fn lowers_console_println_effect_operation() {
        let typed = type_module(
            "artifact/effect-main/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n",
        );
        let core = lower_typed_module(typed);

        assert!(core.bindings.is_empty());
        assert_eq!(core.functions.len(), 1);
        assert_eq!(core.functions[0].parameters[0].id, "unit");
        assert!(matches!(
            core.functions[0].body,
            CoreExpr::EffectOperation { .. }
        ));
    }

    #[test]
    fn lowers_core_binding_to_typescript_const() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.stage, "typescript-ir");
        assert_eq!(typescript.runtime_requirements, vec!["core.int64"]);
        assert_eq!(typescript.bindings.len(), 1);
        assert!(typescript.functions.is_empty());
    }

    #[test]
    fn lowers_core_effect_to_typescript_imported_call() {
        let typed = type_module(
            "artifact/effect-main/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n",
        );
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.unit", "effect.console.println"]
        );
        assert_eq!(typescript.imports[0].local, "_ssrg_console_println");
        assert_eq!(typescript.functions.len(), 1);
    }
}
