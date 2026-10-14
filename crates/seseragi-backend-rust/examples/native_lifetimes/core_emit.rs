//! Restricted checked-CoreIr research emitter. Never selected by the CLI.
use seseragi_lowering::{
    CoreExpr, CoreModule, CoreParameter, CoreStatement, CoreTemplatePart, CoreType,
};
use std::collections::BTreeSet;
type Scope = BTreeSet<String>;
type Result<T> = std::result::Result<T, String>;

fn id(name: &str) -> String {
    format!(
        "v_{}",
        name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    )
}
fn capture(scope: &Scope) -> (String, String) {
    let values = scope
        .iter()
        .map(|name| format!("{}.clone()", id(name)))
        .collect::<Vec<_>>()
        .join(",");
    let setup = scope
        .iter()
        .enumerate()
        .map(|(i, name)| format!("let {} = c[{i}].clone();", id(name)))
        .collect::<String>();
    (values, setup)
}
fn primitive(ty: &CoreType, name: &str) -> bool {
    matches!(ty, CoreType::Named { name: n, arguments } if n == name && arguments.is_empty())
}

pub fn emit(module: &CoreModule) -> Result<String> {
    if !module.instances.is_empty()
        || !module.adts.is_empty()
        || !module.bindings.is_empty()
        || !module.structs.is_empty()
        || !module.foreign_modules.is_empty()
    {
        return Err(
            "research emitter: declarations outside the synchronous Effect / Ref slice".into(),
        );
    }
    let main = module
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::main"))
        .ok_or("missing main")?;
    if !main.is_effect
        || main.parameters.len() != 1
        || !primitive(&main.parameters[0].type_ref, "Unit")
        || !main.constraints.is_empty()
    {
        return Err("research entry must be an unconstrained Unit Effect function".into());
    }
    let mut output = String::from("#![allow(dead_code, unused_variables)]\n");
    for (name, source) in [
        ("heap", include_str!("heap.rs")),
        ("scope", include_str!("scope.rs")),
        ("effect", include_str!("effect.rs")),
    ] {
        // The native executable depends only on std. Serialization belongs to
        // the research report runner, not its generated runtime.
        let source = source
            .replace(", serde::Serialize", "")
            .lines()
            .filter(|line| !line.starts_with("#[serde("))
            .collect::<Vec<_>>()
            .join("\n");
        output.push_str(&format!("mod {name} {{\n{source}\n}}\n"));
    }
    for function in &module.functions {
        if !function.constraints.is_empty() {
            return Err("research constraints unsupported".into());
        }
        let body = function_expr(&function.parameters, &function.body, &Scope::new())?;
        output.push_str(&format!(
            "fn {}(h: &heap::Heap) -> Result<heap::Handle, scope::Exit> {{ Ok({body}) }}\n",
            id(&function.symbol)
        ));
    }
    output.push_str(&format!(r#"
fn main() {{
    let h = heap::Heap::default();
    let plan = {}(&h).and_then(|f| f.call(&h, h.unit())).expect("checked entry");
    let mut context = effect::Context::default();
    context.candidate_collection = std::env::var("SESERAGI_RESEARCH_GC").as_deref() == Ok("candidates");
    let result = context.root(&h, &plan);
    for line in &context.output {{ println!("{{line}}"); }}
    let exit = match result {{ Ok(value) => scope::Exit::Success(value.show()), Err(exit) => exit }};
    drop(plan);
    let collected = if context.candidate_collection {{ h.collect_candidates() }} else {{ h.collect() }}.expect("root released");
    eprintln!("research exit={{exit:?}} live={{}} registry={{}} checkpoints={{}} reclaimed={{}}", h.live(), h.registry(), context.checkpoints, collected.reclaimed + context.reclaimed);
    assert_eq!(h.live(), 0, "all managed roots must be released");
    if !matches!(exit, scope::Exit::Success(_)) {{ std::process::exit(1); }}
}}
"#, id(&main.symbol)));
    Ok(output)
}

fn function_expr(params: &[CoreParameter], body: &CoreExpr, scope: &Scope) -> Result<String> {
    let Some((parameter, rest)) = params.split_first() else {
        return expr(body, scope);
    };
    let (captures, setup) = capture(scope);
    let mut nested = scope.clone();
    nested.insert(parameter.id.clone());
    let body = function_expr(rest, body, &nested)?;
    Ok(format!(
        "h.callable(vec![{captures}], |h,c,input| {{ {setup} let {} = input; Ok({body}) }})",
        id(&parameter.id)
    ))
}

fn operation(name: &str, args: &[CoreExpr], scope: &Scope) -> Result<String> {
    let arity = match name {
        "std/effect::succeed"
        | "effect.succeed"
        | "std/effect::fail"
        | "effect.fail"
        | "std/ref::make"
        | "std/ref::get"
        | "std/effect::scoped"
        | "std/prelude::println"
        | "console.println" => 1,
        "std/ref::set" | "std/effect::acquireRelease" | "std/effect::recover" => 2,
        _ => return Err(format!("research operation unsupported: {name}")),
    };
    if args.len() != arity {
        return Err(format!("research operation arity: {name}"));
    }
    let args = args
        .iter()
        .map(|e| expr(e, scope))
        .collect::<Result<Vec<_>>>()?
        .join(",");
    Ok(format!("effect::operation(h, {name:?}, vec![{args}])?"))
}

fn expr(value: &CoreExpr, scope: &Scope) -> Result<String> {
    match value {
        CoreExpr::Unit { .. } => Ok("h.unit()".into()),
        CoreExpr::Integer { value, .. } => Ok(format!("h.int({value})")),
        CoreExpr::Boolean { value, .. } => Ok(format!("h.boolean({value})")),
        CoreExpr::String { value, .. } => Ok(format!("h.text({value:?})")),
        CoreExpr::Variable { name, evidence, .. } if evidence.is_empty() => {
            if scope.contains(name) { Ok(format!("{}.clone()", id(name))) } else { Ok(format!("{}(h)?", id(name))) }
        }
        CoreExpr::Lambda { parameter, body, .. } => function_expr(std::slice::from_ref(parameter), body, scope),
        CoreExpr::Call { callee, arguments, evidence, deferred_evidence_parameters, deferred_evidence_type_constructor_parameters, trait_dispatch, .. }
            if evidence.is_empty() && deferred_evidence_parameters.is_empty() && deferred_evidence_type_constructor_parameters.is_empty() && trait_dispatch.is_none() => call(callee, arguments, scope),
        CoreExpr::EffectInvoke { callee, arguments, evidence, .. } if evidence.is_empty() => call(callee, arguments, scope),
        CoreExpr::EffectOperation { operation: name, arguments, .. } => operation(name, arguments, scope),
        CoreExpr::Binary { operator, left, right, evidence, .. }
            if ["+", "-", "*", "==", "<"].contains(&operator.as_str()) && evidence.iter().all(|e| matches!(&e.evidence, seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } if ["std/int::Add", "std/int::Sub", "std/int::Mul", "std/int::Eq", "std/int::Ord"].contains(&identity.as_str()))) => {
                Ok(format!("effect::binary(h, {operator:?}, {}, {})?", expr(left, scope)?, expr(right, scope)?))
            }
        CoreExpr::If { condition, then_branch, else_branch, .. } => Ok(format!("if {}.boolean() {{ {} }} else {{ {} }}", expr(condition, scope)?, expr(then_branch, scope)?, expr(else_branch, scope)?)),
        CoreExpr::Template { parts, .. } => {
            let mut text = String::from("{ let mut text = String::new();");
            for part in parts {
                match part {
                    CoreTemplatePart::Text { value, .. } => text.push_str(&format!("text.push_str({value:?});")),
                    CoreTemplatePart::Interpolation { value, evidence, .. } => {
                        if !evidence.as_ref().is_some_and(|e| matches!(&e.evidence, seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } if ["Show<std/prelude::Bool>", "Show<std/prelude::Int>", "Show<std/prelude::String>", "Show<std/prelude::Unit>"].contains(&identity.as_str()))) {
                            return Err("research interpolation requires checked primitive Show".into());
                        }
                        text.push_str(&format!("text.push_str(&{}.show());", expr(value, scope)?));
                    }
                }
            }
            text.push_str("h.text(&text) }"); Ok(text)
        }
        CoreExpr::Sequence { statements, result, .. } => sequence(statements, result, scope),
        _ => Err(format!("research CoreIr unsupported: {value:?}")),
    }
}
fn call(callee: &str, args: &[CoreExpr], scope: &Scope) -> Result<String> {
    if callee.starts_with("std/") {
        return operation(callee, args, scope);
    }
    let mut body = if scope.contains(callee) {
        format!("{}.clone()", id(callee))
    } else {
        format!("{}(h)?", id(callee))
    };
    for arg in args {
        body = format!("{body}.call(h, {})?", expr(arg, scope)?);
    }
    Ok(body)
}

// Match CoreIr's staged sequence: pure prefixes and the next Effect value are
// constructed now; the continuation is constructed only after that Effect runs.
// This is observable through defects and release callback timing.
fn sequence(statements: &[CoreStatement], result: &CoreExpr, scope: &Scope) -> Result<String> {
    let Some((statement, rest)) = statements.split_first() else {
        return expr(result, scope);
    };
    match statement {
        CoreStatement::PureLet {
            name,
            value,
            type_parameters,
            ..
        } if type_parameters.is_empty() => {
            let value = expr(value, scope)?;
            let mut nested = scope.clone();
            nested.insert(name.clone());
            Ok(format!(
                "{{ let {} = {value}; {} }}",
                id(name),
                sequence(rest, result, &nested)?
            ))
        }
        CoreStatement::Bind { name, value, .. } => {
            effect_step(Some(name), value, rest, result, scope)
        }
        CoreStatement::Effect { value } => effect_step(None, value, rest, result, scope),
        _ => Err("research statement unsupported".into()),
    }
}
fn effect_step(
    name: Option<&str>,
    value: &CoreExpr,
    rest: &[CoreStatement],
    result: &CoreExpr,
    scope: &Scope,
) -> Result<String> {
    let value = expr(value, scope)?;
    let (captures, setup) = capture(scope);
    let separator = if captures.is_empty() { "" } else { "," };
    let index = scope.len();
    let mut nested = scope.clone();
    let run = if let Some(name) = name {
        nested.insert(name.into());
        format!("let {} = c[{index}].execute(h,ctx)?;", id(name))
    } else {
        format!("drop(c[{index}].execute(h,ctx)?);")
    };
    let continuation = sequence(rest, result, &nested)?;
    Ok(format!("{{ let input_effect = {value}; h.effect(vec![{captures}{separator}input_effect], |h,c,ctx| {{ {setup} {run} {continuation}.execute(h,ctx) }}) }}"))
}
