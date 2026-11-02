use seseragi_driver::CompiledModule;
use seseragi_lowering::TypeScriptExpr;

#[path = "../../../test-support/source_locations.rs"]
mod shared;

pub fn canonical_typescript(compiled: &CompiledModule, source: &str) -> String {
    shared::canonical_typescript(
        compiled,
        source,
        &seseragi_project::standalone_runtime_source_path(&compiled.diagnostics.source),
    )
}

pub fn source_action(expr: &TypeScriptExpr, effect: bool) -> &TypeScriptExpr {
    let TypeScriptExpr::RuntimeCall { callee, arguments } = expr else {
        panic!("expected a located expression: {expr:?}");
    };
    assert_eq!(callee, "$ssrg$atSource");
    assert_eq!(arguments.len(), 3);
    assert!(
        matches!(&arguments[0], TypeScriptExpr::Identifier { name } if name.strip_prefix("$ssrg$source").is_some_and(|suffix| suffix.parse::<usize>().is_ok()))
    );
    assert_eq!(arguments[2], TypeScriptExpr::Boolean { value: effect });
    let TypeScriptExpr::Lambda { parameter, body } = &arguments[1] else {
        panic!("location action must be a thunk: {expr:?}");
    };
    assert!(parameter.is_empty());
    body
}
