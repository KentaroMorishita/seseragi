use crate::{
    effect_ops::{runtime_effect_operation_by_local_name, runtime_effect_operation_for_feature},
    TypeScriptBinding, TypeScriptExpr, TypeScriptFunction, TypeScriptModule, TypeScriptStatement,
};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMap {
    pub version: u32,
    pub file: String,
    pub source_root: String,
    pub sources: Vec<String>,
    pub sources_content: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
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
        if let Some(operation) = runtime_effect_operation_for_feature(&import.feature) {
            let rendered = format!("{} as {}", operation.export_name, import.local);
            if let Some((_, specifiers)) = import_groups
                .iter_mut()
                .find(|(module, _)| *module == operation.module)
            {
                specifiers.push(rendered);
            } else {
                import_groups.push((operation.module, vec![rendered]));
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
                let rendered_parameters = parameters
                    .iter()
                    .map(|parameter| format!("{}: {}", parameter.name, parameter.type_name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let async_prefix = if *is_async { "async " } else { "" };
                output.push_str(&format!(
                    "const {name} = {async_prefix}({rendered_parameters}) => {}\n",
                    render_typescript_expr(body)
                ));
            }
        }
    }
    output
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

fn source_map_for_module(module: &TypeScriptModule, source_text: &str) -> SourceMap {
    let names = module_names(module);
    SourceMap {
        version: 3,
        file: "main.ts".to_owned(),
        source_root: String::new(),
        sources: vec![format!("seseragi://{}", module.module)],
        sources_content: vec![source_text.to_owned()],
        mappings: if module.functions.is_empty() {
            "AAAA,aAAQA,iBAAc".to_owned()
        } else {
            ";;aAAcA,6BAGZC,sBAAQ".to_owned()
        },
        names,
    }
}

fn module_names(module: &TypeScriptModule) -> Vec<String> {
    let mut names = Vec::new();
    for binding in &module.bindings {
        match binding {
            TypeScriptBinding::Const { name, .. } => names.push(name.clone()),
        }
    }
    for function in &module.functions {
        match function {
            TypeScriptFunction::ConstFunction { name, body, .. } => {
                names.push(name.clone());
                collect_expr_names(body, &mut names);
            }
        }
    }
    names
}

fn collect_expr_names(expr: &TypeScriptExpr, names: &mut Vec<String>) {
    match expr {
        TypeScriptExpr::Call { callee, arguments } => {
            if let Some(operation) = runtime_effect_operation_by_local_name(callee) {
                names.push(operation.source_map_name.to_owned());
            } else {
                names.push(callee.clone());
            }
            for argument in arguments {
                collect_expr_names(argument, names);
            }
        }
        TypeScriptExpr::Await { value } => collect_expr_names(value, names),
        TypeScriptExpr::Binary { left, right, .. } => {
            collect_expr_names(left, names);
            collect_expr_names(right, names);
        }
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_expr_names(condition, names);
            collect_expr_names(then_branch, names);
            collect_expr_names(else_branch, names);
        }
        TypeScriptExpr::Sequence { statements, result } => {
            for statement in statements {
                collect_statement_names(statement, names);
            }
            collect_expr_names(result, names);
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::Boolean { .. }
        | TypeScriptExpr::Identifier { .. }
        | TypeScriptExpr::String { .. } => {}
    }
}

fn collect_statement_names(statement: &TypeScriptStatement, names: &mut Vec<String>) {
    match statement {
        TypeScriptStatement::Effect { value } => collect_expr_names(value, names),
        TypeScriptStatement::Const {
            name, initializer, ..
        } => {
            names.push(name.clone());
            collect_expr_names(initializer, names);
        }
    }
}
