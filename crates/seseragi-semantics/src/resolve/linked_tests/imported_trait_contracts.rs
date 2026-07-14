use super::linked_program;
use crate::{analyze_linked_module, resolve_linked_module, SymbolNamespace};

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
        "pub trait Convert<F> {\n  fn convert<A> value: F<A> -> F<A>\n  where Show<A>\n}\n";
    let main_source = "import { Convert } from \"./domain\"\n\ninstance Convert<Either<String, _>> {\n  fn convert<B> value: Either<String, B> -> Either<String, B>\n  where Show<B> =\n    value\n}\n";
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
