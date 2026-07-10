use super::*;

#[test]
fn keeps_private_declarations_and_resolves_parameter_and_module_references() {
    let resolved = resolve_module(
        "artifact/resolved-body/main.ssrg",
        "let answer: Int = 42\npub fn choose value: Int -> Int = if True then value else answer\n",
    );

    assert_eq!(resolved.schema, 2);
    assert_eq!(resolved.stage, "resolved-ast");
    assert_eq!(resolved.declarations.len(), 2);
    let answer = symbol(&resolved, SymbolKind::Let, "answer");
    let value = symbol(&resolved, SymbolKind::Parameter, "value");
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Value
            && reference.spelling == "answer"
            && reference.target == Some(answer)
    }));
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Value
            && reference.spelling == "value"
            && reference.target == Some(value)
    }));
    assert!(resolved.issues.is_empty());
}

#[test]
fn separates_adt_type_and_constructor_symbols_by_namespace() {
    let resolved = resolve_module(
        "artifact/adt-resolution/main.ssrg",
        "type Hand = | Rock | Paper\nfn opening unit: Unit -> Hand = Rock\n",
    );

    let hand = symbol(&resolved, SymbolKind::Type, "Hand");
    let rock = symbol(&resolved, SymbolKind::Constructor, "Rock");
    assert_ne!(hand, rock);
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Type
            && reference.spelling == "Hand"
            && reference.target == Some(hand)
    }));
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Value
            && reference.spelling == "Rock"
            && reference.target == Some(rock)
    }));
}

#[test]
fn records_unresolved_value_names_in_the_resolver() {
    let resolved = resolve_module(
        "artifact/unresolved/main.ssrg",
        "fn use unit: Unit -> Int = missing\n",
    );

    assert_eq!(resolved.issues.len(), 1);
    assert_eq!(resolved.issues[0].code, "SES-N0001");
    assert_eq!(resolved.issues[0].message_key, "name.unresolved");
    assert!(resolved
        .references
        .iter()
        .any(|reference| { reference.spelling == "missing" && reference.target.is_none() }));
}

#[test]
fn resolves_do_bindings_in_a_child_scope() {
    let resolved = resolve_module(
        "artifact/do-scope/main.ssrg",
        "effect fn copy =\n  do {\n    line <- readLine ()\n    succeed line\n  }\n",
    );

    let line = symbol(&resolved, SymbolKind::PatternBinding, "line");
    let do_scope = resolved
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::DoBlock)
        .expect("do scope exists");
    assert!(do_scope.parent.is_some());
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Value
            && reference.spelling == "line"
            && reference.target == Some(line)
    }));
    assert!(resolved.issues.is_empty());
}

fn symbol(resolved: &ResolvedModule, kind: SymbolKind, spelling: &str) -> SymbolId {
    resolved
        .symbols
        .iter()
        .find(|symbol| symbol.kind == kind && symbol.spelling == spelling)
        .map(|symbol| symbol.id)
        .unwrap_or_else(|| panic!("missing {kind:?} symbol {spelling}"))
}
