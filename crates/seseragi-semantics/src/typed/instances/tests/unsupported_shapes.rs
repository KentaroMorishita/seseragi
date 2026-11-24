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
fn show_for_an_effect_alias_is_an_orphan_while_its_collapsed_method_matches() {
    let source = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ninstance Show<Logged<{}>> { fn show value: Effect<{ console: Console }, Never, Int> -> String = \"logged\" }\n";
    let artifact = diagnostics("merge-inside-effect-show", source);
    assert_eq!(artifact.diagnostics.len(), 1, "{artifact:#?}");
    assert_at(
        &artifact,
        source,
        "SES-T0202",
        "trait.instance-orphan",
        "instance Show<Logged<{}>>",
    );
}

#[test]
fn an_effect_alias_with_a_requirement_merge_registers_collapsed_for_a_local_trait() {
    let source = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ntrait Tag<A> { fn tag value: A -> String }\ninstance Tag<Logged<{}>> { fn tag value: Logged<{}> -> String = \"logged\" }\n";
    let name = "effect-merge-local";
    let artifact = diagnostics(name, source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    let typed = type_module(format!("artifact/{name}/main.ssrg"), source);
    assert_eq!(
        typed.instances[0].identity,
        "artifact/effect-merge-local::trait(Tag)<std/prelude::Effect<{console:artifact/effect-merge-local::Console},std/prelude::Never,std/prelude::Int>>"
    );
}

#[test]
fn a_collapsed_requirement_row_matches_the_alias_method_and_detects_duplicates() {
    let source = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ntrait Tag<A> { fn tag value: A -> String }\ninstance Tag<Logged<{}>> { fn tag value: Effect<{ console: Console }, Never, Int> -> String = \"logged\" }\n";
    let artifact = diagnostics("effect-merge-expanded", source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");

    let duplicate = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ntrait Tag<A> { fn tag value: A -> String }\ninstance Tag<Logged<{}>> { fn tag value: Effect<{ console: Console }, Never, Int> -> String = \"alias\" }\ninstance Tag<Effect<{ console: Console }, Never, Int>> { fn tag value: Effect<{ console: Console }, Never, Int> -> String = \"direct\" }\n";
    let artifact = diagnostics("effect-merge-duplicate", duplicate);
    assert_at(
        &artifact,
        duplicate,
        "SES-T0202",
        "trait.instance-duplicate",
        "instance Tag<Effect<{ console: Console }, Never, Int>>",
    );

    let commutative = "struct Console { token: Int }\nalias Left<R> = Effect<R & { console: Console }, Never, Int>\nalias Right<R> = Effect<{ console: Console } & R, Never, Int>\ntrait Tag<A> { fn tag value: A -> String }\ninstance<R> Tag<Left<R>> { fn tag value: Effect<R & { console: Console }, Never, Int> -> String = \"left\" }\ninstance<S> Tag<Right<S>> { fn tag value: Effect<{ console: Console } & S, Never, Int> -> String = \"right\" }\n";
    let artifact = diagnostics("effect-merge-commutative", commutative);
    assert_at(
        &artifact,
        commutative,
        "SES-T0202",
        "trait.instance-duplicate",
        "instance<S> Tag<Right<S>>",
    );

    let unproved = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ntrait Tag<A> { fn tag value: A -> String }\ninstance<R> Tag<Logged<R>> { fn tag value: Effect<R & { console: Console }, Never, Int> -> String = \"generic\" }\ninstance Tag<Effect<{ console: Console }, Never, Int>> { fn tag value: Effect<{ console: Console }, Never, Int> -> String = \"concrete\" }\n";
    let artifact = diagnostics("effect-merge-unproved", unproved);
    assert_at(
        &artifact,
        unproved,
        "SES-T0101",
        "trait.instance-coherence-unproved",
        "instance Tag<Effect<{ console: Console }, Never, Int>>",
    );

    let hkt = "struct Console { token: Int }\nalias Logged<R> = Effect<R & { console: Console }, Never, Int>\ntrait Tag<F<_>> { fn tag<R> value: F<R> -> String }\ninstance Tag<Logged> { fn tag<R> value: Effect<R & { console: Console }, Never, Int> -> String = \"hkt\" }\n";
    let artifact = diagnostics("effect-merge-hkt", hkt);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
}

#[test]
fn an_applied_higher_kinded_alias_head_registers_and_selects_the_expanded_owner() {
    let source = "type Box<A> = | Box A\nalias Apply<F<_>, A> = F<A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Apply<Box>> { fn tag<A> value: Box<A> -> String = \"box\" }\npub fn probe value: Box<Int> -> String = tag<Apply<Box>, Int> value\n";
    let name = "hkt-applied-head";
    let artifact = diagnostics(name, source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    let typed = type_module(format!("artifact/{name}/main.ssrg"), source);
    assert_eq!(
        typed.instances[0].identity,
        "artifact/hkt-applied-head::trait(Tag)<artifact/hkt-applied-head::Box>"
    );
}

#[test]
fn an_open_higher_kinded_alias_head_reduces_to_its_constructor_parameter() {
    let source = "type Box<A> = | Box A\nalias Apply<F<_>, A> = F<A>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance<F<_>> Tag<Apply<F>> { fn tag<A> value: F<A> -> String = \"apply\" }\n";
    let name = "hkt-open-inverse";
    let artifact = diagnostics(name, source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    let typed = type_module(format!("artifact/{name}/main.ssrg"), source);
    assert_eq!(
        typed.instances[0].identity,
        "artifact/hkt-open-inverse::trait(Tag)<$0>"
    );

    let duplicate = "alias Apply<F<_>, A> = F<A>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance<F<_>> Tag<Apply<F>> { fn tag<A> value: F<A> -> String = \"apply\" }\ninstance<G<_>> Tag<G> { fn tag<A> value: G<A> -> String = \"direct\" }\n";
    let artifact = diagnostics("hkt-open-duplicate", duplicate);
    assert_at(
        &artifact,
        duplicate,
        "SES-T0202",
        "trait.instance-duplicate",
        "instance<G<_>> Tag<G>",
    );

    // A non-eta higher-order type function with a free constructor parameter
    // still cannot be inverted or lowered and remains a positioned refusal.
    let unprovable = "type Box<A> = | Box A\nalias Wrap<F<_>, A> = Box<F<A>>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance<F<_>> Tag<Wrap<F>> { fn tag<A> value: Box<F<A>> -> String = \"wrap\" }\n";
    let artifact = diagnostics("hkt-open-wrap-refused", unprovable);
    assert_at(
        &artifact,
        unprovable,
        "SES-T0101",
        "trait.instance-head-unresolved",
        "instance<F<_>> Tag<Wrap<F>>",
    );
}

#[test]
fn partial_type_function_heads_are_compared_for_duplicates_and_coherence() {
    let source = "type Box<A> = | Box A\nalias Id<F<_>, A> = F<A>\nalias Also<F<_>, A> = F<A>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance Tag<Id<Box>> { fn tag<A> value: Box<A> -> String = \"id\" }\ninstance Tag<Also<Box>> { fn tag<A> value: Box<A> -> String = \"also\" }\n";
    let artifact = diagnostics("partial-type-functions", source);
    assert_eq!(artifact.diagnostics.len(), 1, "{artifact:#?}");
    assert_at(
        &artifact,
        source,
        "SES-T0202",
        "trait.instance-duplicate",
        "instance Tag<Also<Box>>",
    );

    let unproved = "type Pair<A, B> = | Pair (A, B)\nalias Left<F<_, _>, A> = F<A, Int>\nalias Right<F<_, _>, A> = F<Int, A>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance Tag<Left<Pair>> { fn tag<A> value: Pair<A, Int> -> String = \"left\" }\ninstance Tag<Right<Pair>> { fn tag<A> value: Pair<Int, A> -> String = \"right\" }\n";
    let artifact = diagnostics("partial-type-functions-unproved", unproved);
    assert_at(
        &artifact,
        unproved,
        "SES-T0101",
        "trait.instance-coherence-unproved",
        "instance Tag<Right<Pair>>",
    );

    let disjoint = "type Pair<A, B> = | Pair (A, B)\ntype Other<A, B> = | Other (A, B)\nalias Left<F<_, _>, A> = F<A, Int>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance Tag<Left<Pair>> { fn tag<A> value: Pair<A, Int> -> String = \"pair\" }\ninstance Tag<Left<Other>> { fn tag<A> value: Other<A, Int> -> String = \"other\" }\n";
    let artifact = diagnostics("partial-type-functions-disjoint", disjoint);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
}

#[test]
fn an_opaque_alias_and_hkt_opaque_alias_register_the_expanded_owner() {
    let source = "pub opaque type Token<A> = | Token A\nalias Hidden<A> = Token<A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Hidden> { fn tag<A> value: Token<A> -> String = \"hidden\" }\n";
    let name = "opaque-alias-eta";
    let artifact = diagnostics(name, source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    let typed = type_module(format!("artifact/{name}/main.ssrg"), source);
    assert_eq!(
        typed.instances[0].identity,
        "artifact/opaque-alias-eta::trait(Tag)<artifact/opaque-alias-eta::Token>"
    );

    let hkt_opaque = "pub opaque type Wrap<F<_>, A> = | Wrap (F<A>)\ntype Box<A> = | Box A\nalias Apply<F<_>, A> = F<A>\nalias AliasWrap<F<_>, A> = Wrap<F, A>\ntrait Tag<G<_>> { fn tag<A> value: G<A> -> String }\ninstance Tag<AliasWrap<Box>> { fn tag<A> value: Wrap<Box, A> -> String = \"wrap\" }\ninstance Tag<Apply<Token>> { fn tag<A> value: Token<A> -> String = \"token\" }\npub opaque type Token<A> = | Token A\n";
    let name = "opaque-hkt-alias";
    let artifact = diagnostics(name, hkt_opaque);
    assert!(artifact.diagnostics.is_empty(), "{artifact:#?}");
    let typed = type_module(format!("artifact/{name}/main.ssrg"), hkt_opaque);
    assert_eq!(
        typed.instances[0].identity,
        "artifact/opaque-hkt-alias::trait(Tag)<artifact/opaque-hkt-alias::Wrap<artifact/opaque-hkt-alias::Box>>"
    );
    assert_eq!(
        typed.instances[1].identity,
        "artifact/opaque-hkt-alias::trait(Tag)<artifact/opaque-hkt-alias::Token>"
    );
}
