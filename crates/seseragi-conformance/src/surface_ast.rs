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
        "tuple" => validate_expression_array(expression, "elements", path, 2),
        "array" | "list" => validate_expression_array(expression, "elements", path, 0),
        "arrayComprehension" | "listComprehension" => validate_comprehension(expression, path),
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
        "match" => validate_match(expression, path),
        "do" => validate_do(expression, path),
        other => Err(format!(
            "SurfaceAst {path} has unknown expression kind {other}"
        )),
    }
}

fn validate_comprehension(expression: &Value, path: &str) -> Result<(), String> {
    validate_child(expression, "element", path)?;
    let clauses = expression
        .get("clauses")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("SurfaceAst {path}.clauses must be an array"))?;
    if clauses.is_empty() {
        return Err(format!(
            "SurfaceAst {path}.clauses must contain at least one clause"
        ));
    }
    for (index, clause) in clauses.iter().enumerate() {
        let clause_path = format!("{path}.clauses[{index}]");
        require_span(clause, &clause_path)?;
        match clause.get("kind").and_then(Value::as_str) {
            Some("generator") => {
                let pattern = clause
                    .get("pattern")
                    .ok_or_else(|| format!("SurfaceAst {clause_path}.pattern is required"))?;
                validate_pattern(pattern, &format!("{clause_path}.pattern"))?;
                validate_child(clause, "source", &clause_path)?;
            }
            Some("guard") => validate_child(clause, "condition", &clause_path)?,
            Some(other) => {
                return Err(format!(
                    "SurfaceAst {clause_path} has unknown comprehension clause kind {other}"
                ));
            }
            None => {
                return Err(format!("SurfaceAst {clause_path}.kind must be a string"));
            }
        }
    }
    Ok(())
}

fn validate_match(expression: &Value, path: &str) -> Result<(), String> {
    validate_child(expression, "scrutinee", path)?;

    let arms = expression
        .get("arms")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("SurfaceAst {path}.arms must be an array"))?;

    for (index, arm) in arms.iter().enumerate() {
        let arm_path = format!("{path}.arms[{index}]");
        require_span(arm, &arm_path)?;

        let pattern = arm
            .get("pattern")
            .ok_or_else(|| format!("SurfaceAst {arm_path}.pattern is required"))?;
        validate_pattern(pattern, &format!("{arm_path}.pattern"))?;

        if let Some(guard) = arm.get("guard") {
            validate_expression(guard, &format!("{arm_path}.guard"))?;
        }

        validate_child(arm, "body", &arm_path)?;
    }

    Ok(())
}

fn validate_do(expression: &Value, path: &str) -> Result<(), String> {
    let items = expression
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("SurfaceAst {path}.items must be an array"))?;
    for (index, item) in items.iter().enumerate() {
        let item_path = format!("{path}.items[{index}]");
        require_span(item, &item_path)?;
        let kind = item
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("SurfaceAst {item_path}.kind must be a string"))?;
        match kind {
            "bind" | "let" => {
                let pattern = item
                    .get("pattern")
                    .ok_or_else(|| format!("SurfaceAst {item_path}.pattern is required"))?;
                validate_pattern(pattern, &format!("{item_path}.pattern"))?;
            }
            "expression" => {}
            other => {
                return Err(format!(
                    "SurfaceAst {item_path} has unknown do item kind {other}"
                ));
            }
        }
        validate_child(item, "value", &item_path)?;
    }
    let result = expression
        .get("result")
        .ok_or_else(|| format!("valid SurfaceAst {path} must preserve its final do result"))?;
    validate_expression(result, &format!("{path}.result"))
}

fn validate_expression_array(
    parent: &Value,
    field: &str,
    path: &str,
    minimum: usize,
) -> Result<(), String> {
    let elements = parent
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("SurfaceAst {path}.{field} must be an array"))?;
    if elements.len() < minimum {
        return Err(format!(
            "SurfaceAst {path}.{field} must contain at least {minimum} elements"
        ));
    }
    for (index, element) in elements.iter().enumerate() {
        validate_expression(element, &format!("{path}.{field}[{index}]"))?;
    }
    Ok(())
}

fn validate_pattern(pattern: &Value, path: &str) -> Result<(), String> {
    let kind = pattern
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("SurfaceAst {path}.kind must be a string"))?;
    if kind == "error" {
        return Err(format!(
            "valid SurfaceAst {path} must not contain an error pattern"
        ));
    }
    require_span(pattern, path)?;
    match kind {
        "integer" | "string" | "boolean" | "name" | "wildcard" => Ok(()),
        "constructor" => match pattern.get("argument") {
            Some(argument) => validate_pattern(argument, &format!("{path}.argument")),
            None => Ok(()),
        },
        "tuple" => {
            let elements = pattern
                .get("elements")
                .and_then(Value::as_array)
                .ok_or_else(|| format!("SurfaceAst {path}.elements must be an array"))?;
            if elements.len() < 2 {
                return Err(format!(
                    "SurfaceAst {path}.elements must contain at least two elements"
                ));
            }
            for (index, element) in elements.iter().enumerate() {
                validate_pattern(element, &format!("{path}.elements[{index}]"))?;
            }
            Ok(())
        }
        other => Err(format!(
            "SurfaceAst {path} has unknown pattern kind {other}"
        )),
    }
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

    #[test]
    fn accepts_array_values_at_any_expression_depth() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "body": {
                    "kind": "application",
                    "function": { "kind": "name", "span": { "start": 0, "end": 6 } },
                    "argument": {
                        "kind": "array",
                        "elements": [
                            { "kind": "integer", "span": { "start": 8, "end": 9 } },
                            {
                                "kind": "array",
                                "elements": [],
                                "span": { "start": 11, "end": 13 }
                            }
                        ],
                        "span": { "start": 7, "end": 14 }
                    },
                    "span": { "start": 0, "end": 14 }
                }
            }]
        });

        assert!(validate_surface_ast(&module).is_ok());
    }

    #[test]
    fn accepts_array_comprehension_with_generator_and_guard() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "body": {
                    "kind": "arrayComprehension",
                    "element": { "kind": "name", "span": { "start": 1, "end": 2 } },
                    "clauses": [
                        {
                            "kind": "generator",
                            "pattern": { "kind": "name", "span": { "start": 5, "end": 6 } },
                            "source": { "kind": "name", "span": { "start": 10, "end": 16 } },
                            "span": { "start": 5, "end": 16 }
                        },
                        {
                            "kind": "guard",
                            "condition": { "kind": "boolean", "span": { "start": 18, "end": 22 } },
                            "span": { "start": 18, "end": 22 }
                        }
                    ],
                    "span": { "start": 0, "end": 23 }
                }
            }]
        });

        assert!(validate_surface_ast(&module).is_ok());
    }

    #[test]
    fn accepts_tuple_values_and_nested_tuple_patterns() {
        let module = json!({
            "declarations": [{
                "kind": "effectFn",
                "body": {
                    "kind": "do",
                    "items": [{
                        "kind": "bind",
                        "pattern": {
                            "kind": "tuple",
                            "elements": [
                                { "kind": "name", "span": { "start": 1, "end": 2 } },
                                {
                                    "kind": "tuple",
                                    "elements": [
                                        { "kind": "wildcard", "span": { "start": 4, "end": 5 } },
                                        { "kind": "name", "span": { "start": 6, "end": 7 } }
                                    ],
                                    "span": { "start": 3, "end": 8 }
                                }
                            ],
                            "span": { "start": 0, "end": 9 }
                        },
                        "value": { "kind": "name", "span": { "start": 13, "end": 17 } },
                        "span": { "start": 0, "end": 17 }
                    }],
                    "result": {
                        "kind": "tuple",
                        "elements": [
                            { "kind": "integer", "span": { "start": 19, "end": 20 } },
                            { "kind": "boolean", "span": { "start": 22, "end": 26 } }
                        ],
                        "span": { "start": 18, "end": 27 }
                    },
                    "span": { "start": 0, "end": 28 }
                }
            }]
        });

        assert!(validate_surface_ast(&module).is_ok());
    }

    #[test]
    fn accepts_match_arms_with_tuple_patterns_and_guards() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "body": {
                    "kind": "match",
                    "scrutinee": {
                        "kind": "name",
                        "span": { "start": 6, "end": 11 }
                    },
                    "arms": [
                        {
                            "pattern": {
                                "kind": "tuple",
                                "elements": [
                                    {
                                        "kind": "constructor",
                                        "argument": {
                                            "kind": "name",
                                            "span": { "start": 20, "end": 25 }
                                        },
                                        "span": { "start": 15, "end": 25 }
                                    },
                                    { "kind": "wildcard", "span": { "start": 27, "end": 28 } }
                                ],
                                "span": { "start": 14, "end": 29 }
                            },
                            "guard": {
                                "kind": "boolean",
                                "span": { "start": 35, "end": 39 }
                            },
                            "body": {
                                "kind": "name",
                                "span": { "start": 43, "end": 48 }
                            },
                            "span": { "start": 14, "end": 48 }
                        }
                    ],
                    "span": { "start": 0, "end": 50 }
                }
            }]
        });

        assert!(validate_surface_ast(&module).is_ok());
    }

    #[test]
    fn accepts_literal_patterns() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "body": {
                    "kind": "match",
                    "scrutinee": { "kind": "name", "span": { "start": 0, "end": 5 } },
                    "arms": [
                        {
                            "pattern": { "kind": "string", "raw": "\"rock\"", "span": { "start": 8, "end": 14 } },
                            "body": { "kind": "boolean", "value": true, "span": { "start": 18, "end": 22 } },
                            "span": { "start": 8, "end": 22 }
                        },
                        {
                            "pattern": { "kind": "integer", "raw": "1", "span": { "start": 24, "end": 25 } },
                            "body": { "kind": "boolean", "value": false, "span": { "start": 29, "end": 34 } },
                            "span": { "start": 24, "end": 34 }
                        },
                        {
                            "pattern": { "kind": "boolean", "value": true, "span": { "start": 36, "end": 40 } },
                            "body": { "kind": "boolean", "value": true, "span": { "start": 44, "end": 48 } },
                            "span": { "start": 36, "end": 48 }
                        }
                    ],
                    "span": { "start": 0, "end": 49 }
                },
                "span": { "start": 0, "end": 49 }
            }]
        });

        assert!(validate_surface_ast(&module).is_ok());
    }

    #[test]
    fn rejects_match_arm_without_a_body() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "body": {
                    "kind": "match",
                    "scrutinee": {
                        "kind": "name",
                        "span": { "start": 6, "end": 11 }
                    },
                    "arms": [{
                        "pattern": {
                            "kind": "wildcard",
                            "span": { "start": 14, "end": 15 }
                        },
                        "span": { "start": 14, "end": 15 }
                    }],
                    "span": { "start": 0, "end": 17 }
                }
            }]
        });

        let error = validate_surface_ast(&module).unwrap_err();
        assert!(error.contains("arms[0].body is required"));
    }

    #[test]
    fn accepts_an_empty_match_for_later_exhaustiveness_diagnostics() {
        let module = json!({
            "declarations": [{
                "kind": "fn",
                "body": {
                    "kind": "match",
                    "scrutinee": {
                        "kind": "name",
                        "span": { "start": 6, "end": 11 }
                    },
                    "arms": [],
                    "span": { "start": 0, "end": 14 }
                }
            }]
        });

        assert!(validate_surface_ast(&module).is_ok());
    }
}
