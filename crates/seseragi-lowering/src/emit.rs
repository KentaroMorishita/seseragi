use crate::typescript::types::render_typescript_type;
use crate::{
    TypeScriptAdt, TypeScriptAdtVariant, TypeScriptBinding, TypeScriptExpr, TypeScriptFunction,
    TypeScriptModule, TypeScriptStatement,
};
use serde::{Deserialize, Serialize};

mod decision;
mod imports;
mod instances;
mod metadata;
mod source_map;

use imports::render_import_lines;
use instances::render_typescript_instances;
use metadata::generated_module_for;
pub use metadata::{GeneratedInstance, GeneratedModule, GeneratedOutputs, GeneratedRuntime};
use source_map::source_map_for_module;
pub use source_map::SourceMap;

/// Locations chosen by the project for a generated TypeScript module and its
/// source map.
///
/// The lowering crate preserves these paths in generated metadata and uses the
/// TypeScript path as the `file` field of the source map. It does not interpret
/// or normalize paths: project output planning owns that policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedOutputPaths {
    pub typescript: String,
    pub source_map: String,
}

impl GeneratedOutputPaths {
    pub fn new(typescript: impl Into<String>, source_map: impl Into<String>) -> Self {
        Self {
            typescript: typescript.into(),
            source_map: source_map.into(),
        }
    }
}

impl Default for GeneratedOutputPaths {
    fn default() -> Self {
        Self::new("main.ts", "main.ts.map")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedBundle {
    pub metadata: GeneratedModule,
    pub typescript: String,
    pub source_map: SourceMap,
}

pub fn emit_typescript_module(module: TypeScriptModule, source_text: &str) -> GeneratedBundle {
    emit_typescript_module_with_output_paths(module, source_text, GeneratedOutputPaths::default())
}

/// Emits a module using caller-selected generated artifact paths.
///
/// This keeps TypeScript path selection at the project boundary while making
/// the selected paths observable through both generated module metadata and
/// the source map's `file` field.
pub fn emit_typescript_module_with_output_paths(
    module: TypeScriptModule,
    source_text: &str,
    output_paths: GeneratedOutputPaths,
) -> GeneratedBundle {
    let typescript = render_typescript(&module);
    let source_map = source_map_for_module(&module, source_text, &output_paths.typescript);
    let metadata = generated_module_for(module, output_paths);

    GeneratedBundle {
        metadata,
        typescript,
        source_map,
    }
}

fn render_typescript(module: &TypeScriptModule) -> String {
    let mut output = String::new();
    let import_lines = render_import_lines(module);
    for line in &import_lines {
        output.push_str(line);
        output.push('\n');
    }
    if !import_lines.is_empty()
        && (!module.adts.is_empty()
            || !module.instances.is_empty()
            || !module.bindings.is_empty()
            || !module.functions.is_empty())
    {
        output.push('\n');
    }
    for adt in &module.adts {
        render_adt(&mut output, adt);
    }
    render_typescript_instances(&mut output, &module.instances, &module.type_imports);
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
                type_parameters,
                parameters,
                body,
                ..
            } => {
                if *exported {
                    output.push_str("export ");
                }
                let rendered_body =
                    render_function_body(type_parameters, parameters, body, *is_async);
                output.push_str(&format!("const {name} = {rendered_body}\n",));
            }
        }
    }
    output
}

fn render_adt(output: &mut String, adt: &TypeScriptAdt) {
    if adt.exported {
        output.push_str("export ");
    }
    let type_parameters = render_type_parameters(&adt.type_parameters);
    if adt.variants.is_empty() {
        output.push_str(&format!("type {}{type_parameters} = never;\n", adt.name));
    } else {
        output.push_str(&format!("type {}{type_parameters} =\n", adt.name));
        for (index, variant) in adt.variants.iter().enumerate() {
            let suffix = if index + 1 == adt.variants.len() {
                ";"
            } else {
                ""
            };
            output.push_str(&format!(
                "  | {}{suffix}\n",
                render_adt_variant_type(variant)
            ));
        }
    }

    let result_type = format!(
        "{}{}",
        adt.name,
        render_type_arguments(&adt.type_parameters)
    );
    for variant in &adt.variants {
        if variant.exported {
            output.push_str("export ");
        }
        let tag = format!("{:?}", variant.tag);
        match &variant.payload {
            None if adt.type_parameters.is_empty() => output.push_str(&format!(
                "const {}: {result_type} = {{ tag: {tag} }} as const;\n",
                variant.name
            )),
            None => output.push_str(&format!(
                "const {} = {{ tag: {tag} }} as const;\n",
                variant.name
            )),
            Some(payload) => {
                let generic_parameters = render_type_parameters(&adt.type_parameters);
                output.push_str(&format!(
                    "const {} = {generic_parameters}(value: {}): {result_type} => ({{ tag: {tag}, value }} as const);\n",
                    variant.name,
                    render_typescript_type(payload),
                ));
            }
        }
    }
}

fn render_adt_variant_type(variant: &TypeScriptAdtVariant) -> String {
    let tag = format!("{:?}", variant.tag);
    match &variant.payload {
        Some(payload) => format!(
            "{{ readonly tag: {tag}; readonly value: {} }}",
            render_typescript_type(payload)
        ),
        None => format!("{{ readonly tag: {tag} }}"),
    }
}

fn render_type_parameters(parameters: &[String]) -> String {
    if parameters.is_empty() {
        String::new()
    } else {
        format!("<{}>", parameters.join(", "))
    }
}

fn render_type_arguments(parameters: &[String]) -> String {
    render_type_parameters(parameters)
}

fn render_function_body(
    type_parameters: &[String],
    parameters: &[crate::TypeScriptParameter],
    body: &TypeScriptExpr,
    is_async: bool,
) -> String {
    let rendered_body = render_typescript_expr(body);
    let Some((last, leading)) = parameters.split_last() else {
        return rendered_body;
    };
    let async_prefix = if is_async { "async " } else { "" };
    let generic_prefix = render_arrow_type_parameters(type_parameters);
    let final_generic_prefix = if leading.is_empty() {
        generic_prefix.as_str()
    } else {
        ""
    };
    let final_arrow = format!(
        "{async_prefix}{final_generic_prefix}({}: {}) => {rendered_body}",
        last.name, last.type_name
    );
    let rendered = leading.iter().rev().fold(final_arrow, |result, parameter| {
        format!("({}: {}) => {result}", parameter.name, parameter.type_name)
    });
    if leading.is_empty() || generic_prefix.is_empty() {
        rendered
    } else {
        format!("{generic_prefix}{rendered}")
    }
}

fn render_arrow_type_parameters(parameters: &[String]) -> String {
    match parameters {
        [] => String::new(),
        [parameter] => format!("<{parameter},>"),
        _ => format!("<{},>", parameters.join(", ")),
    }
}

fn render_typescript_expr(expr: &TypeScriptExpr) -> String {
    match expr {
        TypeScriptExpr::Undefined => "undefined".to_owned(),
        TypeScriptExpr::Bigint { value } => format!("{value}n"),
        TypeScriptExpr::String { value } => format!("{value:?}"),
        TypeScriptExpr::Boolean { value } => value.to_string(),
        TypeScriptExpr::Identifier { name } => name.clone(),
        TypeScriptExpr::RuntimeReference { name } => name.clone(),
        TypeScriptExpr::CurriedRuntimeReference { name, arity } => {
            let parameters = (0..*arity)
                .map(|index| format!("_argument{index}"))
                .collect::<Vec<_>>();
            let call = format!("{name}({})", parameters.join(", "));
            parameters
                .into_iter()
                .rev()
                .fold(call, |body, parameter| format!("({parameter}) => {body}"))
        }
        TypeScriptExpr::Tuple { elements } => format!(
            "[{}] as const",
            elements
                .iter()
                .map(render_typescript_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeScriptExpr::Array {
            elements,
            element_type,
        } => {
            let literal = format!(
                "[{}]",
                elements
                    .iter()
                    .map(render_typescript_expr)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if elements.is_empty() {
                format!(
                    "{literal} as ReadonlyArray<{}>",
                    render_typescript_type(element_type)
                )
            } else {
                literal
            }
        }
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
        TypeScriptExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
        } => decision::render_decision(scrutinee, scrutinee_type, branches, type_ref),
        TypeScriptExpr::Call { callee, arguments } => {
            if arguments.is_empty() {
                return format!("{callee}()");
            }
            arguments.iter().fold(callee.clone(), |call, argument| {
                format!("{call}({})", render_typescript_expr(argument))
            })
        }
        TypeScriptExpr::TypeApplicationCall {
            callee,
            type_arguments,
            arguments,
        } => {
            let rendered_callee = if type_arguments.is_empty() {
                callee.clone()
            } else {
                let rendered_types = type_arguments
                    .iter()
                    .map(render_typescript_type)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{callee}<{rendered_types}>")
            };
            let rendered_arguments = arguments
                .iter()
                .map(render_typescript_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{rendered_callee}({rendered_arguments})")
        }
        TypeScriptExpr::DictionaryCall {
            dictionary,
            method,
            arguments,
        } => arguments.iter().fold(
            format!("{}[{method:?}]", render_typescript_expr(dictionary)),
            |call, argument| format!("{call}({})", render_typescript_expr(argument)),
        ),
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
        TypeScriptStatement::PureLet {
            name,
            type_ref,
            initializer,
            ..
        } => format!(
            "(() => {{ const {name}: {} = {}; return {continuation}; }})()",
            render_typescript_type(type_ref),
            render_typescript_expr(initializer)
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
