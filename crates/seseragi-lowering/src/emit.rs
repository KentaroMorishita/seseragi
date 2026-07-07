use crate::{TypeScriptBinding, TypeScriptExpr, TypeScriptFunction, TypeScriptModule};
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
    let metadata = GeneratedModule {
        schema: module.schema,
        module: module.module,
        target: "typescript-es2022".to_owned(),
        runtime: GeneratedRuntime {
            identity: "@seseragi/runtime".to_owned(),
            abi_major: 1,
            requirements: module.runtime_requirements,
        },
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

fn render_typescript(module: &TypeScriptModule) -> String {
    let mut output = String::new();
    for import in &module.imports {
        if import.feature == "effect.console.println" {
            output.push_str(&format!(
                "import {{ println as {} }} from \"@seseragi/runtime/console\"\n",
                import.local
            ));
        }
    }
    if !module.imports.is_empty() && !module.functions.is_empty() {
        output.push('\n');
    }
    for binding in &module.bindings {
        match binding {
            TypeScriptBinding::Const {
                exported,
                name,
                initializer,
                ..
            } => {
                if *exported {
                    output.push_str("export ");
                }
                output.push_str(&format!(
                    "const {name}: bigint = {};\n",
                    render_typescript_expr(initializer)
                ));
            }
        }
    }
    for function in &module.functions {
        match function {
            TypeScriptFunction::ConstFunction {
                exported,
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
                output.push_str(&format!(
                    "const {name} = ({rendered_parameters}) => {}\n",
                    render_typescript_expr(body)
                ));
            }
        }
    }
    output
}

fn render_typescript_expr(expr: &TypeScriptExpr) -> String {
    match expr {
        TypeScriptExpr::Undefined => "undefined".to_owned(),
        TypeScriptExpr::Bigint { value } => format!("{value}n"),
        TypeScriptExpr::String { value } => format!("{value:?}"),
        TypeScriptExpr::Call { callee, arguments } => {
            let rendered_arguments = arguments
                .iter()
                .map(render_typescript_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{callee}({rendered_arguments})")
        }
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
            if callee == "_ssrg_console_println" {
                names.push("println".to_owned());
            }
            for argument in arguments {
                collect_expr_names(argument, names);
            }
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::String { .. } => {}
    }
}
