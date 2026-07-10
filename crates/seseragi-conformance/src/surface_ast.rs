use serde_json::Value;

pub(crate) fn validate_surface_ast(module: &Value) -> Result<(), String> {
    let declarations = module
        .pointer("/declarations")
        .and_then(Value::as_array)
        .ok_or_else(|| "SurfaceAst declarations must be an array".to_owned())?;

    for (index, declaration) in declarations.iter().enumerate() {
        let kind = declaration
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("SurfaceAst declarations[{index}] kind must be a string"))?;
        if matches!(kind, "let" | "fn" | "effectFn") {
            let body = declaration.get("body").ok_or_else(|| {
                format!("valid SurfaceAst {kind} declarations must preserve their body")
            })?;
            validate_expression(body, &format!("declarations[{index}].body"))?;
        }
    }
    Ok(())
}

fn validate_expression(expression: &Value, path: &str) -> Result<(), String> {
    let kind = expression
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("SurfaceAst {path}.kind must be a string"))?;
    if kind == "error" {
        return Err(format!(
            "valid SurfaceAst {path} must not contain an error expression"
        ));
    }
    require_span(expression, path)?;

    match kind {
        "unit" | "integer" | "string" | "boolean" | "name" => Ok(()),
        "grouped" => validate_child(expression, "value", path),
        "application" => {
            validate_child(expression, "function", path)?;
            validate_child(expression, "argument", path)
        }
        "binary" => {
            validate_child(expression, "left", path)?;
            validate_child(expression, "right", path)
        }
        "if" => {
            validate_child(expression, "condition", path)?;
            validate_child(expression, "thenBranch", path)?;
            validate_child(expression, "elseBranch", path)
        }
        "do" => validate_do(expression, path),
        other => Err(format!(
            "SurfaceAst {path} has unknown expression kind {other}"
        )),
    }
}

fn validate_do(expression: &Value, path: &str) -> Result<(), String> {
    let items = expression
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("SurfaceAst {path}.items must be an array"))?;
    for (index, item) in items.iter().enumerate() {
        require_span(item, &format!("{path}.items[{index}]"))?;
        validate_child(item, "value", &format!("{path}.items[{index}]"))?;
    }
    let result = expression
        .get("result")
        .ok_or_else(|| format!("valid SurfaceAst {path} must preserve its final do result"))?;
    validate_expression(result, &format!("{path}.result"))
}

fn validate_child(parent: &Value, field: &str, path: &str) -> Result<(), String> {
    let child = parent
        .get(field)
        .ok_or_else(|| format!("SurfaceAst {path}.{field} is required"))?;
    validate_expression(child, &format!("{path}.{field}"))
}

fn require_span(value: &Value, path: &str) -> Result<(), String> {
    let start = value.pointer("/span/start").and_then(Value::as_u64);
    let end = value.pointer("/span/end").and_then(Value::as_u64);
    match (start, end) {
        (Some(start), Some(end)) if start <= end => Ok(()),
        _ => Err(format!(
            "SurfaceAst {path}.span must be an ordered byte range"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_valid_declaration_without_a_body() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "span": { "start": 0, "end": 1 }
            }]
        });

        assert!(validate_surface_ast(&module)
            .unwrap_err()
            .contains("must preserve their body"));
    }

    #[test]
    fn accepts_left_nested_application_body() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "body": {
                    "kind": "application",
                    "function": {
                        "kind": "application",
                        "function": { "kind": "name", "span": { "start": 0, "end": 3 } },
                        "argument": { "kind": "name", "span": { "start": 4, "end": 5 } },
                        "span": { "start": 0, "end": 5 }
                    },
                    "argument": { "kind": "integer", "span": { "start": 6, "end": 7 } },
                    "span": { "start": 0, "end": 7 }
                }
            }]
        });

        assert!(validate_surface_ast(&module).is_ok());
    }
}
