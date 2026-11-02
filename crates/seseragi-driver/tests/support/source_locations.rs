use seseragi_driver::CompiledModule;
use seseragi_lowering::TypeScriptExpr;

/// Source edits move coordinates without changing the program. Validate every
/// emitted coordinate before comparing code with stable location names.
pub fn canonical_typescript(compiled: &CompiledModule, source: &str) -> String {
    let mut result = compiled.generated.typescript.clone();
    let mut origins = compiled
        .typescript_ir
        .runtime_sources
        .iter()
        .collect::<Vec<_>>();
    origins.sort_by_key(|origin| origin.start);
    assert_eq!(result.matches("const $ssrg$source").count(), origins.len());
    for (index, origin) in origins.iter().enumerate() {
        assert!(origin.start < origin.end && origin.end <= source.len());
        assert!(source.is_char_boundary(origin.start) && source.is_char_boundary(origin.end));
        assert_eq!(
            origin.source,
            seseragi_project::standalone_runtime_source_path(&compiled.diagnostics.source)
        );
        let prefix = &source[..origin.start];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
        let uri = format!("seseragi://{}", compiled.typescript_ir.module);
        let declaration = format!(
            "const $ssrg$source{} = {{ file: {:?}, line: {line}, column: {column}, uri: {uri:?}, start: {}, end: {} }};",
            origin.start, origin.source, origin.start, origin.end
        );
        assert_eq!(
            result.matches(&declaration).count(),
            1,
            "{declaration}\n{result}"
        );
        result = result.replace(
            &declaration,
            &format!(
                "const $ssrg$canonicalSource{index} = {{ file: {:?}, uri: {uri:?} }};",
                origin.source
            ),
        );
    }
    // Longest identifiers first: source12 must not replace part of source123.
    let mut replacements = origins
        .iter()
        .enumerate()
        .map(|(index, origin)| {
            (
                format!("$ssrg$source{}", origin.start),
                format!("$ssrg$canonicalSource{index}"),
            )
        })
        .collect::<Vec<_>>();
    replacements.sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));
    for (from, to) in replacements {
        assert!(result.contains(&from), "unused source location {from}");
        result = result.replace(&from, &to);
    }
    result
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
