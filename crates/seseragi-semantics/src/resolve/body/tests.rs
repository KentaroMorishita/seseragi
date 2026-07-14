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
fn resolves_instance_method_signatures_and_bodies_in_child_scopes() {
    let resolved = resolve_module(
        "artifact/instance-method/main.ssrg",
        "newtype Score = Int\ninstance Show<Score> {\n  fn show value: Score -> Score = value\n}\n",
    );

    let value = symbol(&resolved, SymbolKind::Parameter, "value");
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Value
            && reference.spelling == "value"
            && reference.target == Some(value)
    }));
    assert_eq!(
        resolved
            .references
            .iter()
            .filter(|reference| {
                reference.namespace == SymbolNamespace::Type && reference.spelling == "Score"
            })
            .count(),
        3
    );
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Trait
            && reference.spelling == "Show"
            && reference.target.is_some_and(|target| {
                resolved.symbols.iter().any(|symbol| {
                    symbol.id == target && symbol.canonical.as_deref() == Some("std/prelude::Show")
                })
            })
    }));
    assert!(resolved.issues.is_empty());
}

#[test]
fn resolves_local_trait_instance_heads_and_constraint_names_by_symbol() {
    let resolved = resolve_module(
        "artifact/local-trait-instance/main.ssrg",
        "trait Render<A> { fn render value: A -> String }\n\
         newtype Score = Int\n\
         instance Render<Score> where Eq<Score> { fn render value: Score -> String = \"score\" }\n",
    );

    let render = symbol(&resolved, SymbolKind::Trait, "Render");
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Trait
            && reference.spelling == "Render"
            && reference.target == Some(render)
    }));
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Trait
            && reference.spelling == "Eq"
            && reference.target.is_some_and(|target| {
                resolved.symbols.iter().any(|symbol| {
                    symbol.id == target && symbol.canonical.as_deref() == Some("std/prelude::Eq")
                })
            })
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
fn lazily_materializes_complete_standard_sum_type_families() {
    let resolved = resolve_module(
        "artifact/prelude-maybe/main.ssrg",
        "fn wrap value: String -> Maybe<String> = Just value\n",
    );

    for (kind, spelling, canonical) in [
        (SymbolKind::Type, "Maybe", "std/prelude::Maybe"),
        (SymbolKind::Constructor, "Nothing", "std/prelude::Nothing"),
        (SymbolKind::Constructor, "Just", "std/prelude::Just"),
    ] {
        assert!(resolved.symbols.iter().any(|symbol| {
            symbol.kind == kind
                && symbol.spelling == spelling
                && symbol.canonical.as_deref() == Some(canonical)
        }));
    }
    assert!(resolved.symbols.iter().any(|symbol| {
        symbol.kind == SymbolKind::TypeParameter
            && symbol.spelling == "A"
            && symbol.canonical.as_deref() == Some("std/prelude::Maybe::A")
    }));
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "Just"
            && reference.target.is_some_and(|target| {
                resolved.symbols.iter().any(|symbol| {
                    symbol.id == target && symbol.canonical.as_deref() == Some("std/prelude::Just")
                })
            })
    }));
    assert!(resolved.issues.is_empty());
}

#[test]
fn local_adt_family_shadows_the_standard_sum_type_without_materializing_it() {
    let resolved = resolve_module(
        "artifact/local-maybe/main.ssrg",
        "type Maybe<A> = | Empty | Filled A\nfn wrap value: String -> Maybe<String> = Filled value\n",
    );

    assert!(resolved.symbols.iter().all(|symbol| {
        !symbol
            .canonical
            .as_deref()
            .is_some_and(|canonical| canonical.starts_with("std/prelude::Maybe"))
    }));
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "Maybe"
            && reference.target.is_some_and(|target| {
                resolved.symbols.iter().any(|symbol| {
                    symbol.id == target
                        && symbol.canonical.as_deref() == Some("artifact/local-maybe::Maybe")
                })
            })
    }));
    assert!(resolved.issues.is_empty());
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
fn records_unresolved_adt_payload_types_in_the_resolver() {
    let resolved = resolve_module(
        "artifact/unresolved-adt-payload/main.ssrg",
        "type Label = | Present Strng\n",
    );

    assert_eq!(resolved.issues.len(), 1);
    assert_eq!(resolved.issues[0].code, "SES-N0001");
    assert_eq!(resolved.issues[0].message_key, "name.unresolved");
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Type
            && reference.spelling == "Strng"
            && reference.target.is_none()
    }));
}

#[test]
fn rejects_duplicate_constructors_without_creating_ambiguous_symbols() {
    let resolved = resolve_module(
        "artifact/duplicate-constructors/main.ssrg",
        "type First = | Shared\ntype Second = | Shared\nfn use unit: Unit -> First = Shared\n",
    );

    assert_duplicate_definition(&resolved);
    let constructors = resolved
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.namespace == SymbolNamespace::Value
                && symbol.kind == SymbolKind::Constructor
                && symbol.spelling == "Shared"
        })
        .collect::<Vec<_>>();
    assert_eq!(constructors.len(), 1);
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Value
            && reference.spelling == "Shared"
            && reference.target == Some(constructors[0].id)
    }));
}

#[test]
fn rejects_duplicate_type_declarations_without_creating_ambiguous_symbols() {
    let resolved = resolve_module(
        "artifact/duplicate-types/main.ssrg",
        "type Hand = | Rock\ntype Hand = | Paper\nfn use unit: Unit -> Hand = Rock\n",
    );

    assert_duplicate_definition(&resolved);
    let types = resolved
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.namespace == SymbolNamespace::Type
                && symbol.kind == SymbolKind::Type
                && symbol.spelling == "Hand"
        })
        .collect::<Vec<_>>();
    assert_eq!(types.len(), 1);
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Type
            && reference.spelling == "Hand"
            && reference.target == Some(types[0].id)
    }));
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

#[test]
fn resolves_match_patterns_and_bodies_in_sibling_arm_scopes() {
    let resolved = resolve_module(
        "artifact/match-scope/main.ssrg",
        "type Label = | Missing | Present String\nfn render value: Label -> String = match value { Present item -> item; Missing -> \"missing\" }\n",
    );

    let arm_scopes = resolved
        .scopes
        .iter()
        .filter(|scope| scope.kind == ScopeKind::MatchArm)
        .collect::<Vec<_>>();
    assert_eq!(arm_scopes.len(), 2);
    assert_eq!(arm_scopes[0].parent, arm_scopes[1].parent);
    let binding = resolved
        .symbols
        .iter()
        .find(|symbol| symbol.kind == SymbolKind::PatternBinding)
        .expect("payload binding");
    assert_eq!(binding.spelling, "item");
    assert_eq!(binding.scope, arm_scopes[0].id);
    let body_reference = resolved
        .references
        .iter()
        .find(|reference| reference.spelling == "item")
        .expect("arm body reference");
    assert_eq!(body_reference.target, Some(binding.id));
    assert!(resolved.issues.is_empty());
}

#[test]
fn exposes_pattern_bindings_to_their_guard_but_not_sibling_arms() {
    let resolved = resolve_module(
        "artifact/match-scope-isolation/main.ssrg",
        "type Label = | Missing | Present String\nfn render value: Label -> String = match value { Present item when item -> item; Missing -> item }\n",
    );

    let item_references = resolved
        .references
        .iter()
        .filter(|reference| reference.spelling == "item")
        .collect::<Vec<_>>();
    assert_eq!(item_references.len(), 3);
    assert!(item_references[0].target.is_some());
    assert_eq!(item_references[1].target, item_references[0].target);
    assert_eq!(item_references[2].target, None);

    for constructor in ["Present", "Missing"] {
        assert!(resolved
            .references
            .iter()
            .any(|reference| reference.spelling == constructor && reference.target.is_some()));
    }
    assert_eq!(resolved.issues.len(), 1);
    assert_eq!(resolved.issues[0].code, "SES-N0001");
}

#[test]
fn literal_patterns_do_not_create_bindings_or_name_references() {
    let resolved = resolve_module(
        "artifact/literal-patterns/main.ssrg",
        "fn accepts value: (Int, String, Bool) -> Bool = match value { (42, \"go\", True) -> True; (0, \"stop\", False) -> False; _ -> False }\n",
    );

    assert!(resolved
        .symbols
        .iter()
        .all(|symbol| symbol.kind != SymbolKind::PatternBinding));
    assert!(resolved.references.iter().all(|reference| {
        !matches!(
            reference.spelling.as_str(),
            "42" | "0" | "\"go\"" | "\"stop\"" | "True" | "False"
        )
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

fn assert_duplicate_definition(resolved: &ResolvedModule) {
    let duplicates = resolved
        .issues
        .iter()
        .filter(|issue| issue.code == "SES-N0002")
        .collect::<Vec<_>>();
    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].message_key, "name.duplicate-definition");
}
