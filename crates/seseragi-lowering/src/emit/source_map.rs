use crate::{
    effect_ops::runtime_effect_operation_for_feature, int_ops::runtime_int_operation_for_feature,
    SourceSpan, TypeScriptBinding, TypeScriptExpr, TypeScriptFunction, TypeScriptModule,
    TypeScriptStatement,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Mapping {
    generated_line: usize,
    original_line: usize,
    original_column: usize,
    name_index: usize,
}

pub(super) fn source_map_for_module(module: &TypeScriptModule, source_text: &str) -> SourceMap {
    let helper_names = runtime_helper_names(module);
    let (names, mappings) = module_names_and_mappings(module, source_text, &helper_names);
    SourceMap {
        version: 3,
        file: "main.ts".to_owned(),
        source_root: String::new(),
        sources: vec![format!("seseragi://{}", module.module)],
        sources_content: vec![source_text.to_owned()],
        names,
        mappings: encode_mappings(&mappings),
    }
}

fn module_names_and_mappings(
    module: &TypeScriptModule,
    source_text: &str,
    helper_names: &BTreeMap<String, String>,
) -> (Vec<String>, Vec<Mapping>) {
    let mut names = Vec::new();
    let mut mappings = Vec::new();
    let mut generated_line = generated_declaration_start_line(module);

    for adt in &module.adts {
        push_declaration_mapping(
            &mut names,
            &mut mappings,
            generated_line,
            &adt.name,
            &adt.origin,
            source_text,
        );
        let type_lines = if adt.variants.is_empty() {
            1
        } else {
            1 + adt.variants.len()
        };
        for (index, variant) in adt.variants.iter().enumerate() {
            push_declaration_mapping(
                &mut names,
                &mut mappings,
                generated_line + type_lines + index,
                &variant.name,
                &variant.origin,
                source_text,
            );
        }
        generated_line += type_lines + adt.variants.len();
    }

    for binding in &module.bindings {
        match binding {
            TypeScriptBinding::Const {
                name,
                initializer,
                origin,
                ..
            } => {
                push_declaration_mapping(
                    &mut names,
                    &mut mappings,
                    generated_line,
                    name,
                    origin,
                    source_text,
                );
                collect_expr_names(initializer, helper_names, &mut names);
                generated_line += 1;
            }
        }
    }
    for function in &module.functions {
        match function {
            TypeScriptFunction::ConstFunction {
                name, body, origin, ..
            } => {
                push_declaration_mapping(
                    &mut names,
                    &mut mappings,
                    generated_line,
                    name,
                    origin,
                    source_text,
                );
                collect_expr_names(body, helper_names, &mut names);
                generated_line += 1;
            }
        }
    }
    (names, mappings)
}

fn push_declaration_mapping(
    names: &mut Vec<String>,
    mappings: &mut Vec<Mapping>,
    generated_line: usize,
    name: &str,
    origin: &SourceSpan,
    source_text: &str,
) {
    let name_index = names.len();
    names.push(name.to_owned());
    let (original_line, original_column) = source_position(source_text, origin.start);
    mappings.push(Mapping {
        generated_line,
        original_line,
        original_column,
        name_index,
    });
}

fn generated_declaration_start_line(module: &TypeScriptModule) -> usize {
    let import_lines = module
        .imports
        .iter()
        .filter_map(|import| runtime_module_for_feature(&import.feature))
        .collect::<BTreeSet<_>>()
        .len();
    import_lines
        + usize::from(
            !module.imports.is_empty()
                && (!module.adts.is_empty()
                    || !module.bindings.is_empty()
                    || !module.functions.is_empty()),
        )
}

fn runtime_helper_names(module: &TypeScriptModule) -> BTreeMap<String, String> {
    module
        .imports
        .iter()
        .filter_map(|import| {
            runtime_source_name_for_feature(&import.feature)
                .map(|name| (import.local.clone(), name.to_owned()))
        })
        .collect()
}

fn runtime_module_for_feature(feature: &str) -> Option<&'static str> {
    runtime_effect_operation_for_feature(feature)
        .map(|operation| operation.module)
        .or_else(|| runtime_int_operation_for_feature(feature).map(|operation| operation.module))
}

fn runtime_source_name_for_feature(feature: &str) -> Option<&'static str> {
    runtime_effect_operation_for_feature(feature)
        .map(|operation| operation.source_map_name)
        .or_else(|| {
            runtime_int_operation_for_feature(feature).map(|operation| operation.source_map_name)
        })
}

fn collect_expr_names(
    expr: &TypeScriptExpr,
    helper_names: &BTreeMap<String, String>,
    names: &mut Vec<String>,
) {
    match expr {
        TypeScriptExpr::Call { callee, arguments }
        | TypeScriptExpr::RuntimeCall { callee, arguments } => {
            names.push(
                helper_names
                    .get(callee)
                    .cloned()
                    .unwrap_or_else(|| callee.clone()),
            );
            for argument in arguments {
                collect_expr_names(argument, helper_names, names);
            }
        }
        TypeScriptExpr::Await { value } => collect_expr_names(value, helper_names, names),
        TypeScriptExpr::Tuple { elements } => {
            for element in elements {
                collect_expr_names(element, helper_names, names);
            }
        }
        TypeScriptExpr::Binary { left, right, .. } => {
            collect_expr_names(left, helper_names, names);
            collect_expr_names(right, helper_names, names);
        }
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_expr_names(condition, helper_names, names);
            collect_expr_names(then_branch, helper_names, names);
            collect_expr_names(else_branch, helper_names, names);
        }
        TypeScriptExpr::Sequence { statements, result } => {
            for statement in statements {
                collect_statement_names(statement, helper_names, names);
            }
            collect_expr_names(result, helper_names, names);
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::Boolean { .. }
        | TypeScriptExpr::Identifier { .. }
        | TypeScriptExpr::String { .. } => {}
    }
}

fn collect_statement_names(
    statement: &TypeScriptStatement,
    helper_names: &BTreeMap<String, String>,
    names: &mut Vec<String>,
) {
    match statement {
        TypeScriptStatement::Effect { value } => collect_expr_names(value, helper_names, names),
        TypeScriptStatement::Const {
            name, initializer, ..
        } => {
            names.push(name.clone());
            collect_expr_names(initializer, helper_names, names);
        }
    }
}

fn source_position(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut offset = byte_offset.min(source.len());
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let line_start = source[..offset]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let line = source[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let column = source[line_start..offset].encode_utf16().count();
    (line, column)
}

fn encode_mappings(mappings: &[Mapping]) -> String {
    let mut output = String::new();
    let mut generated_line = 0usize;
    let mut previous_source = 0i64;
    let mut previous_original_line = 0i64;
    let mut previous_original_column = 0i64;
    let mut previous_name = 0i64;

    for mapping in mappings {
        while generated_line < mapping.generated_line {
            output.push(';');
            generated_line += 1;
        }
        output.push_str(&encode_vlq(0));
        output.push_str(&encode_vlq(-previous_source));
        previous_source = 0;
        output.push_str(&encode_vlq(
            mapping.original_line as i64 - previous_original_line,
        ));
        previous_original_line = mapping.original_line as i64;
        output.push_str(&encode_vlq(
            mapping.original_column as i64 - previous_original_column,
        ));
        previous_original_column = mapping.original_column as i64;
        output.push_str(&encode_vlq(mapping.name_index as i64 - previous_name));
        previous_name = mapping.name_index as i64;
    }
    output
}

fn encode_vlq(value: i64) -> String {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = if value < 0 {
        ((-value) as u64) << 1 | 1
    } else {
        (value as u64) << 1
    };
    let mut output = String::new();
    loop {
        let mut digit = (value & 31) as usize;
        value >>= 5;
        if value != 0 {
            digit |= 32;
        }
        output.push(BASE64[digit] as char);
        if value == 0 {
            return output;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_signed_source_map_vlq_values() {
        assert_eq!(encode_vlq(0), "A");
        assert_eq!(encode_vlq(1), "C");
        assert_eq!(encode_vlq(-1), "D");
        assert_eq!(encode_vlq(16), "gB");
        assert_eq!(encode_vlq(-16), "hB");
    }

    #[test]
    fn maps_declarations_from_generated_and_original_lines() {
        let mappings = encode_mappings(&[
            Mapping {
                generated_line: 2,
                original_line: 1,
                original_column: 3,
                name_index: 0,
            },
            Mapping {
                generated_line: 3,
                original_line: 4,
                original_column: 0,
                name_index: 2,
            },
        ]);

        assert_eq!(mappings, ";;AACGA;AAGHE");
    }

    #[test]
    fn counts_original_columns_as_utf16_code_units() {
        assert_eq!(source_position("// 🌊\npub let", 8), (1, 0));
        assert_eq!(source_position("// 🌊pub let", 7), (0, 5));
    }
}
