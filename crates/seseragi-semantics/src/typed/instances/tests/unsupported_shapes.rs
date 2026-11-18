use crate::{semantic_diagnostics, type_module};
use seseragi_syntax::DiagnosticArtifact;

fn diagnostics(name: &str, source: &str) -> DiagnosticArtifact {
    semantic_diagnostics(format!("artifact/{name}/main.ssrg"), source)
}

fn overlaps(source: &str, start: usize, end: usize, snippet: &str) -> bool {
    let Some(snippet_start) = source.find(snippet) else {
        return false;
    };
    let snippet_end = snippet_start + snippet.len();
    start < snippet_end && end > snippet_start
}

fn assert_at(artifact: &DiagnosticArtifact, source: &str, code: &str, key: &str, snippet: &str) {
    assert!(
        artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == code
                && diagnostic.message_key == key
                && overlaps(
                    source,
                    diagnostic.primary.start,
                    diagnostic.primary.end,
                    snippet,
                )
        }),
        "{code} {key} at {snippet:?}\n{artifact:#?}"
    );
}

#[test]
fn a_requirement_merge_outside_an_effect_is_rejected() {
    let source = "struct Console { token: Int }\nalias Logged<A> = A & { console: Console }\n";
    let artifact = diagnostics("merge-outside-effect", source);
    assert_eq!(artifact.diagnostics.len(), 1);
    assert_at(
        &artifact,
        source,
        "SES-T0501",
        "type.requirement-merge-invalid-position",
        "A & { console: Console }",
    );
}

#[test]
fn a_requirement_merge_alias_is_not_an_instance_constructor() {
    let source = "struct Console { token: Int }\nalias Logged<A> = A & { console: Console }\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Logged> { fn tag<A> value: (A & { console: Console }) -> String = \"logged\" }\n";
    let artifact = diagnostics("merge-constructor-head", source);
    assert_at(
        &artifact,
        source,
        "SES-T0501",
        "type.requirement-merge-invalid-position",
        "alias Logged<A> = A & { console: Console }",
    );
    assert_at(
        &artifact,
        source,
        "SES-T0501",
        "type.requirement-merge-invalid-position",
        "(A & { console: Console })",
    );
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "trait.instance-head-unresolved",
        "instance Tag<Logged>",
    );
}

#[test]
fn show_for_an_effect_alias_is_an_orphan_and_the_method_does_not_match() {
    let source = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ninstance Show<Logged<{}>> { fn show value: Effect<{ console: Console }, Never, Int> -> String = \"logged\" }\n";
    let artifact = diagnostics("merge-inside-effect-show", source);
    assert_at(
        &artifact,
        source,
        "SES-T0202",
        "trait.instance-orphan",
        "instance Show<Logged<{}>>",
    );
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "trait.instance-method-signature-mismatch",
        "fn show value: Effect<{ console: Console }, Never, Int> -> String",
    );
}

#[test]
fn an_effect_alias_with_a_requirement_merge_registers_for_a_local_trait() {
    let source = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ntrait Tag<A> { fn tag value: A -> String }\ninstance Tag<Logged<{}>> { fn tag value: Logged<{}> -> String = \"logged\" }\n";
    let name = "effect-merge-local";
    let artifact = diagnostics(name, source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    let typed = type_module(format!("artifact/{name}/main.ssrg"), source);
    assert_eq!(
        typed.instances[0].identity,
        "artifact/effect-merge-local::trait(Tag)<std/prelude::Effect<{}&{console:artifact/effect-merge-local::Console},std/prelude::Never,std/prelude::Int>>"
    );
}

#[test]
fn a_collapsed_requirement_row_does_not_match_the_alias_method() {
    let source = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ntrait Tag<A> { fn tag value: A -> String }\ninstance Tag<Logged<{}>> { fn tag value: Effect<{ console: Console }, Never, Int> -> String = \"logged\" }\n";
    let artifact = diagnostics("effect-merge-expanded", source);
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "trait.instance-method-signature-mismatch",
        "fn tag value: Effect<{ console: Console }, Never, Int> -> String",
    );
    assert!(
        !artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "trait.instance-head-unresolved"),
        "{artifact:#?}"
    );
}

#[test]
fn an_applied_higher_kinded_alias_head_stays_unresolved() {
    let source = "type Box<A> = | Box A\nalias Apply<F<_>, A> = F<A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Apply<Box>> { fn tag<A> value: Box<A> -> String = \"box\" }\npub fn probe value: Box<Int> -> String = tag<Apply<Box>, Int> value\n";
    let artifact = diagnostics("hkt-applied-head", source);
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "trait.instance-head-unresolved",
        "instance Tag<Apply<Box>>",
    );
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "call.argument-type-mismatch",
        "tag<Apply<Box>, Int> value",
    );
}

#[test]
fn an_open_higher_kinded_alias_head_stays_unresolved() {
    let source = "type Box<A> = | Box A\nalias Apply<F<_>, A> = F<A>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance<F<_>> Tag<Apply<F>> { fn tag<A> value: F<A> -> String = \"apply\" }\n";
    let artifact = diagnostics("hkt-open-inverse", source);
    assert_eq!(artifact.diagnostics.len(), 1);
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "trait.instance-head-unresolved",
        "instance<F<_>> Tag<Apply<F>>",
    );
}

#[test]
fn partial_type_function_heads_are_unresolved_rather_than_compared() {
    let source = "type Box<A> = | Box A\nalias Id<F<_>, A> = F<A>\nalias Also<F<_>, A> = F<A>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance Tag<Id<Box>> { fn tag<A> value: Box<A> -> String = \"id\" }\ninstance Tag<Also<Box>> { fn tag<A> value: Box<A> -> String = \"also\" }\n";
    let artifact = diagnostics("partial-type-functions", source);
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "trait.instance-head-unresolved",
        "instance Tag<Id<Box>>",
    );
    assert_at(
        &artifact,
        source,
        "SES-T0101",
        "trait.instance-head-unresolved",
        "instance Tag<Also<Box>>",
    );
    assert!(
        !artifact.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.message_key.as_str(),
                "trait.instance-duplicate" | "trait.instance-coherence-unproved"
            )
        }),
        "{artifact:#?}"
    );
}

#[test]
fn an_opaque_alias_eta_registers_the_expanded_owner() {
    let source = "pub opaque type Token<A> = | Token A\nalias Hidden<A> = Token<A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Hidden> { fn tag<A> value: Token<A> -> String = \"hidden\" }\n";
    let name = "opaque-alias-eta";
    let artifact = diagnostics(name, source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    let typed = type_module(format!("artifact/{name}/main.ssrg"), source);
    assert_eq!(
        typed.instances[0].identity,
        "artifact/opaque-alias-eta::trait(Tag)<artifact/opaque-alias-eta::Token>"
    );
}
