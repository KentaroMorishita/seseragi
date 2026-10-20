use super::linked_program;
use crate::{analyze_linked_module, resolve_linked_module, SymbolNamespace};

#[test]
fn imported_trait_contracts_include_implicit_unit_and_preserve_returned_functions() {
    for (declaration, implementation, valid) in [
        (
            "fn pick<A> left: F<A> -> right: F<A> -> Bool where Show<A>",
            "fn pick<B> -> (Box<B> -> Box<B> -> Bool) where Show<B> = \\left -> \\right -> False",
            false,
        ),
        (
            "fn pick<A> left: F<A> -> right: F<A> -> Bool where Show<A>",
            "fn pick<B> left: Box<B> -> (Box<B> -> Bool) where Show<B> = \\right -> False",
            true,
        ),
        (
            "fn pick<A> -> (F<A> -> Bool) where Show<A>",
            "fn pick<B> unit: Unit -> (Box<B> -> Bool) where Show<B> = \\value -> False",
            true,
        ),
        (
            "fn pick<A> unit: Unit -> (F<A> -> Bool) where Show<A>",
            "fn pick<B> -> (Box<B> -> Bool) where Show<B> = \\value -> False",
            true,
        ),
    ] {
        let domain = format!("pub trait Pick<F<_>> {{ {declaration} }}\n");
        let source = format!("// 雫\nimport {{ Pick as Choose }} from \"./domain\"\ntype Box<A> = | Box A\ninstance Choose<Box> {{ {implementation} }}\n");
        let linked = linked_program(&source, [("./domain", "fixture/game::domain", &domain)]);
        let result = analyze_linked_module(
            seseragi_syntax::parse_diagnostics("main-λ.ssrg", &source),
            linked,
            &source,
        );
        if valid {
            result.unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        } else {
            let diagnostics = result.unwrap_err();
            let mismatch = diagnostics
                .diagnostics
                .iter()
                .find(|diagnostic| {
                    diagnostic.message_key == "trait.instance-method-signature-mismatch"
                })
                .unwrap_or_else(|| panic!("{source}\n{diagnostics:#?}"));
            assert_eq!(diagnostics.source, "main-λ.ssrg");
            assert_eq!(
                &source[mismatch.primary.start as usize..mismatch.primary.end as usize],
                implementation
            );
        }
    }
}

#[test]
fn rejects_dropped_and_missing_methods_of_an_import_aliased_trait() {
    let domain = "pub trait Show<A> { fn describe value: A -> String }\n";
    for (method, key) in [
        ("", "trait.instance-method-missing"),
        (
            "fn describe value = \"label\"",
            "parser.function-annotations-required",
        ),
        (
            "fn describe value: Label -> Int = 42",
            "trait.instance-method-signature-mismatch",
        ),
    ] {
        let source = format!("// 雫\nimport {{ Show as Render }} from \"./domain\"\ntype Label = | Label\ninstance Render<Label> {{ {method} }}\n");
        let linked = linked_program(&source, [("./domain", "fixture/game::domain", domain)]);
        let diagnostics = analyze_linked_module(
            seseragi_syntax::parse_diagnostics("main-λ.ssrg", &source),
            linked,
            &source,
        )
        .unwrap_err();
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message_key == key),
            "{source}\n{diagnostics:#?}"
        );
        assert_eq!(diagnostics.source, "main-λ.ssrg");
    }
}

#[test]
fn standard_method_contracts_expand_imported_aliases_by_owner() {
    for (target, body, valid) in [("String", "\"label\"", true), ("Int", "42", false)] {
        let domain = format!("pub alias Text = {target}\n");
        let source = format!("import {{ Text as Result }} from \"./domain\"\ntype Label = | Label\ninstance Show<Label> {{ fn show value: Label -> Result = {body} }}\n");
        let linked = linked_program(&source, [("./domain", "fixture/game::domain", &domain)]);
        let result = analyze_linked_module(
            seseragi_syntax::parse_diagnostics("main.ssrg", &source),
            linked,
            &source,
        );
        if valid {
            result.unwrap();
        } else {
            assert!(result
                .unwrap_err()
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message_key
                    == "trait.instance-method-signature-mismatch"));
        }
    }
}

#[test]
fn accepts_an_instance_that_matches_an_imported_trait_contract() {
    let domain_source = "pub trait Render<A> {\n  fn render value: A -> String\n}\n";
    let main_source = "import { Render } from \"./domain\"\n\nnewtype Score = Int\n\ninstance Render<Score> {\n  fn render value: Score -> String = \"score\"\n}\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    let imported = resolved
        .imports
        .iter()
        .find(|import| import.export.namespace == "trait")
        .unwrap();
    assert_eq!(imported.export.methods.len(), 1);
    assert!(resolved.references.iter().any(|reference| {
        reference.namespace == SymbolNamespace::Trait
            && reference.spelling == "Render"
            && reference.target == Some(imported.symbol)
    }));

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
}

#[test]
fn reports_an_import_aliased_local_and_dependency_instance_as_ambiguous() {
    let domain_source = "pub type Badge = | Active\n\
                         pub trait Render<A> { fn render value: A -> String }\n\
                         instance Render<Badge> { fn render value: Badge -> String = \"provider\" }\n";
    let main_source = "import { Badge, Render as R } from \"./domain\"\n\n\
                       instance R<Badge> { fn render value: Badge -> String = \"consumer\" }\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0202"
            && diagnostic.message_key == "trait.instance-ambiguous"
            && diagnostic.related[0]
                .message
                .contains("fixture/game::domain")
    }));
}

#[test]
fn reports_two_local_aliases_of_one_trait_as_duplicate_instances() {
    let domain_source = "pub type Badge = | Active\n\
                         pub trait Render<A> { fn render value: A -> String }\n";
    let main_source = "import { Badge, Render as First, Render as Second } from \"./domain\"\n\n\
                       instance First<Badge> { fn render value: Badge -> String = \"first\" }\n\
                       instance Second<Badge> { fn render value: Badge -> String = \"second\" }\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0202" && diagnostic.message_key == "trait.instance-duplicate"
    }));
}

#[test]
fn compares_provider_nominals_by_canonical_identity() {
    let domain_source = "pub type Prefix =\n  | Prefix String\n\npub trait Render<A> {\n  fn render prefix: Prefix -> value: A -> String\n}\n";
    let main_source = "import { Prefix, Render } from \"./domain\"\n\nnewtype Score = Int\n\ninstance Render<Score> {\n  fn render prefix: Prefix -> value: Score -> String = \"score\"\n}\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    let imported = resolved
        .imports
        .iter()
        .find(|import| import.export.namespace == "trait")
        .unwrap();
    assert!(imported
        .scheme_type_bindings
        .as_ref()
        .unwrap()
        .iter()
        .any(|binding| binding.spelling == "Prefix"
            && binding.canonical == "fixture/game::domain::Prefix"));

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
}

#[test]
fn alpha_normalizes_imported_generic_methods_and_prelude_constraints() {
    let domain_source =
        "pub trait Convert<F<_>> {\n  fn convert<A> value: F<A> -> F<A>\n  where Show<A>\n}\n";
    // The implementation module owns the outer constructor; importing Convert
    // does not authorize an instance for a prelude-owned Either constructor.
    let main_source = "import { Convert } from \"./domain\"\n\ntype Choice<E, A> = | Choice A\ninstance Convert<Choice<String, _>> {\n  fn convert<B> value: Choice<String, B> -> Choice<String, B>\n  where Show<B> =\n    value\n}\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
}

#[test]
fn alias_head_cannot_create_an_owner_for_an_imported_nominal() {
    let domain = "pub type Label = | Label\npub alias Name = Label\npub trait Render<A> { fn render value: A -> String }\n";
    for (head, declaration, allowed) in [
        ("Name", "", false),
        ("LocalName", "alias LocalName = Name\n", false),
        ("Owned", "newtype Owned = Name\n", true),
    ] {
        let source = format!("// 雫\nimport {{ Name, Render as Display }} from \"./domain\"\n{declaration}instance Display<{head}> {{ fn render value: {head} -> String = \"label\" }}\n");
        let linked = linked_program(&source, [("./domain", "fixture/game::domain", domain)]);
        let result = analyze_linked_module(
            seseragi_syntax::parse_diagnostics("main-λ.ssrg", &source),
            linked,
            &source,
        );
        if allowed {
            result.unwrap();
        } else {
            let diagnostics = result.unwrap_err();
            assert!(
                diagnostics
                    .diagnostics
                    .iter()
                    .any(|d| d.message_key == "trait.instance-orphan"),
                "{diagnostics:#?}"
            );
            assert_eq!(diagnostics.source, "main-λ.ssrg");
        }
    }
}

#[test]
fn resolves_provider_local_method_constraints_by_trait_identity() {
    let domain_source = "pub trait Labeled<A> {\n  fn label value: A -> String\n}\n\npub trait Render<A> {\n  fn render value: A -> String\n  where Labeled<A>\n}\n";
    let main_source = "import { Labeled, Render } from \"./domain\"\n\nnewtype Score = Int\n\ninstance Render<Score> {\n  fn render value: Score -> String\n  where Labeled<Score> =\n    \"score\"\n}\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    let render = resolved
        .imports
        .iter()
        .find(|import| import.export.name == "Render")
        .unwrap();
    assert!(render
        .contract_trait_bindings
        .as_ref()
        .unwrap()
        .iter()
        .any(|binding| binding.spelling == "Labeled"
            && binding.canonical == "fixture/game::domain::trait(Labeled)"));

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
}

#[test]
fn rejects_a_signature_that_breaks_an_imported_trait_contract() {
    let domain_source = "pub trait Render<A> {\n  fn render value: A -> String\n}\n";
    let main_source = "import { Render } from \"./domain\"\n\nnewtype Score = Int\n\ninstance Render<Score> {\n  fn render value: Score -> Int = 0\n}\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0101"
            && diagnostic.message_key == "trait.instance-method-signature-mismatch"
    }));
}

#[test]
fn rejects_a_missing_imported_trait_method() {
    let domain_source = "pub trait Render<A> {\n  fn render value: A -> String\n}\n";
    let main_source =
        "import { Render } from \"./domain\"\n\nnewtype Score = Int\n\ninstance Render<Score> {}\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0101" && diagnostic.message_key == "trait.instance-method-missing"
    }));
}
