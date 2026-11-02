use seseragi_driver::CompiledModule;

/// Source edits move coordinates without changing the program. Validate every
/// emitted coordinate before comparing code with stable location names.
pub fn canonical_typescript(
    compiled: &CompiledModule,
    source: &str,
    runtime_source_path: &str,
) -> String {
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
        assert_eq!(origin.source, runtime_source_path);
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
