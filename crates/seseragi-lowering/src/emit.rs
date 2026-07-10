use crate::{
    effect_ops::runtime_effect_operation_for_feature, int_ops::runtime_int_operation_for_feature,
    TypeScriptBinding, TypeScriptExpr, TypeScriptFunction, TypeScriptModule, TypeScriptStatement,
};
use serde::{Deserialize, Serialize};

mod source_map;

use source_map::source_map_for_module;
pub use source_map::SourceMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedBundle {
    pub metadata: GeneratedModule,
    pub typescript: String,
    pub source_map: SourceMap,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedModule {
    pub schema: u32,
    pub module: String,
    pub target: String,
    pub runtime: GeneratedRuntime,
    pub exports: Vec<String>,
    pub outputs: GeneratedOutputs,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedRuntime {
    pub identity: String,
    pub abi_major: u32,
    pub requirements: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedOutputs {
    pub typescript: String,
    pub source_map: String,
}

pub fn emit_typescript_module(module: TypeScriptModule, source_text: &str) -> GeneratedBundle {
    let typescript = render_typescript(&module);
    let source_map = source_map_for_module(&module, source_text);
    let exports = module_exports(&module);
    let metadata = GeneratedModule {
        schema: module.schema,
        module: module.module,
        target: "typescript-es2022".to_owned(),
        runtime: GeneratedRuntime {
            identity: "@seseragi/runtime".to_owned(),
            abi_major: 1,
            requirements: module.runtime_requirements,
        },
        exports,
        outputs: GeneratedOutputs {
            typescript: "main.ts".to_owned(),
            source_map: "main.ts.map".to_owned(),
        },
    };

    GeneratedBundle {
        metadata,
        typescript,
        source_map,
    }
}

fn module_exports(module: &TypeScriptModule) -> Vec<String> {
    let mut exports = Vec::new();
    for binding in &module.bindings {
        match binding {
            TypeScriptBinding::Const { exported, name, .. } if *exported => {
                exports.push(name.clone());
            }
            _ => {}
        }
    }
    for function in &module.functions {
        match function {
            TypeScriptFunction::ConstFunction { exported, name, .. } if *exported => {
                exports.push(name.clone());
            }
            _ => {}
        }
    }
    exports
}

fn render_typescript(module: &TypeScriptModule) -> String {
    let mut output = String::new();
    let mut import_groups: Vec<(&str, Vec<String>)> = Vec::new();
    for import in &module.imports {
        let helper = runtime_effect_operation_for_feature(&import.feature)
            .map(|operation| (operation.module, operation.export_name))
            .or_else(|| {
                runtime_int_operation_for_feature(&import.feature)
                    .map(|operation| (operation.module, operation.export_name))
            });
        if let Some((runtime_module, export_name)) = helper {
            let rendered = format!("{export_name} as {}", import.local);
            if let Some((_, specifiers)) = import_groups
                .iter_mut()
                .find(|(module, _)| *module == runtime_module)
            {
                specifiers.push(rendered);
            } else {
                import_groups.push((runtime_module, vec![rendered]));
            }
        }
    }
    for (module, specifiers) in import_groups {
        output.push_str(&format!(
            "import {{ {} }} from \"{module}\"\n",
            specifiers.join(", ")
        ));
    }
    if !module.imports.is_empty() && !module.functions.is_empty() {
        output.push('\n');
    }
    for binding in &module.bindings {
        match binding {
            TypeScriptBinding::Const {
                exported,
                name,
                type_ref,
                initializer,
                ..
            } => {
                if *exported {
                    output.push_str("export ");
                }
                output.push_str(&format!(
                    "const {name}: {} = {};\n",
                    render_typescript_type(type_ref),
                    render_typescript_expr(initializer)
                ));
            }
        }
    }
    for function in &module.functions {
        match function {
            TypeScriptFunction::ConstFunction {
                exported,
                is_async,
                name,
                parameters,
                body,
                ..
            } => {
                if *exported {
                    output.push_str("export ");
                }
                let rendered_body = render_function_body(parameters, body, *is_async);
                output.push_str(&format!("const {name} = {rendered_body}\n",));
            }
        }
    }
    output
}

fn render_function_body(
    parameters: &[crate::TypeScriptParameter],
    body: &TypeScriptExpr,
    is_async: bool,
) -> String {
    let rendered_body = render_typescript_expr(body);
    let Some((last, leading)) = parameters.split_last() else {
        return rendered_body;
    };
    let async_prefix = if is_async { "async " } else { "" };
    let final_arrow = format!(
        "{async_prefix}({}: {}) => {rendered_body}",
        last.name, last.type_name
    );
    leading.iter().rev().fold(final_arrow, |result, parameter| {
        format!("({}: {}) => {result}", parameter.name, parameter.type_name)
    })
}

fn render_typescript_type(type_ref: &crate::TypeScriptType) -> &'static str {
    match type_ref {
        crate::TypeScriptType::Bigint => "bigint",
        crate::TypeScriptType::Boolean => "boolean",
        crate::TypeScriptType::String => "string",
        crate::TypeScriptType::Undefined => "undefined",
        crate::TypeScriptType::Unknown => "unknown",
        crate::TypeScriptType::Maybe { element } => match element.as_ref() {
            crate::TypeScriptType::Bigint => "bigint | undefined",
            crate::TypeScriptType::Boolean => "boolean | undefined",
            crate::TypeScriptType::String => "string | undefined",
            crate::TypeScriptType::Undefined => "undefined",
            crate::TypeScriptType::Unknown | crate::TypeScriptType::Maybe { .. } => {
                "unknown | undefined"
            }
        },
    }
}

fn render_typescript_expr(expr: &TypeScriptExpr) -> String {
    match expr {
        TypeScriptExpr::Undefined => "undefined".to_owned(),
        TypeScriptExpr::Bigint { value } => format!("{value}n"),
        TypeScriptExpr::String { value } => format!("{value:?}"),
        TypeScriptExpr::Boolean { value } => value.to_string(),
        TypeScriptExpr::Identifier { name } => name.clone(),
        TypeScriptExpr::Binary {
            operator,
            left,
            right,
        } => format!(
            "{} {operator} {}",
            render_typescript_expr(left),
            render_typescript_expr(right)
        ),
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => format!(
            "{} ? {} : {}",
            render_typescript_expr(condition),
            render_typescript_expr(then_branch),
            render_typescript_expr(else_branch)
        ),
        TypeScriptExpr::Call { callee, arguments } => {
            if arguments.is_empty() {
                return format!("{callee}()");
            }
            arguments.iter().fold(callee.clone(), |call, argument| {
                format!("{call}({})", render_typescript_expr(argument))
            })
        }
        TypeScriptExpr::RuntimeCall { callee, arguments } => {
            let rendered_arguments = arguments
                .iter()
                .map(render_typescript_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{callee}({rendered_arguments})")
        }
        TypeScriptExpr::Await { value } => format!("await {}", render_typescript_expr(value)),
        TypeScriptExpr::Sequence { statements, result } => {
            render_effect_sequence(statements, result)
        }
    }
}

fn render_effect_sequence(statements: &[TypeScriptStatement], result: &TypeScriptExpr) -> String {
    let Some((statement, rest)) = statements.split_first() else {
        return render_typescript_expr(result);
    };
    let continuation = render_effect_sequence(rest, result);
    match statement {
        TypeScriptStatement::Effect { value } => format!(
            "_ssrg_effect_flatMap({}, () => {continuation})",
            render_typescript_expr(value)
        ),
        TypeScriptStatement::Const {
            name,
            type_ref,
            initializer,
            ..
        } => format!(
            "_ssrg_effect_flatMap({}, ({name}: {}) => {continuation})",
            render_typescript_expr(initializer),
            render_typescript_type(type_ref)
        ),
    }
}
