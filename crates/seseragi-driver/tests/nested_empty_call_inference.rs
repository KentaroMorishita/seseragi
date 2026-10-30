use seseragi_driver::{analyze_module, compile_module, BuildProfile, CompileInput};

#[test]
fn original_nested_empty_collection_infers_later_siblings_in_both_profiles() {
    let original = include_str!("../../../examples/spec/fixtures/rust-backend-regressions/inferred-nested-empty-collection.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input = || {
            CompileInput::new("nested-雪🙂.ssrg", "proof/nested", original).with_profile(profile)
        };
        compile_module(input()).unwrap();
        assert!(analyze_module(input()).diagnostics.diagnostics.is_empty());
    }
}

#[test]
fn generic_collection_calls_share_sibling_equations_without_erasing_nested_shape() {
    for (kind, value) in [
        ("Array", "[[], [\"雪\"], []]"),
        ("Array", "[[\"雪\"], [], [\"🙂\"]]"),
        ("Array", "[[], [[1, 2], []], [[3]]]"),
        ("Array", "[`[], `[1, 2], `[]]"),
        ("List", "`[`[], `[\"雪\"], `[]]"),
        ("List", "`[`[\"雪\"], `[], `[\"🙂\"]]"),
        ("List", "`[[], [1, 2], []]"),
    ] {
        let source = format!("fn keep<A> values: {kind}<A> -> {kind}<A> = values\npub effect fn main = println (show (keep {value}))\n");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input = || {
                CompileInput::new("nested-shape.ssrg", "proof/nested", &source)
                    .with_profile(profile)
            };
            compile_module(input()).unwrap_or_else(|error| panic!("{source}\n{error:?}"));
            assert!(analyze_module(input()).diagnostics.diagnostics.is_empty());
        }
    }
}

#[test]
fn inference_preserves_nominal_and_collection_mismatches_and_their_utf8_ranges() {
    for (source, key, rejected) in [
        ("newtype One = Int\nnewtype Two = Int\nfn keep<A> values: Array<Array<A>> -> Array<Array<A>> = values\npub fn probe -> Array<Array<One>> = keep [[], [One 1], [Two 2]]\n", "array.element-type-mismatch", "Two 2"),
        ("fn keep<A> values: Array<Array<A>> -> Array<Array<A>> = values\npub fn probe -> Array<Array<Int>> = keep [[], [1], [True]]\n", "array.element-type-mismatch", "True"),
        ("fn keep<A> values: List<List<A>> -> List<List<A>> = values\npub fn probe -> List<List<Int>> = keep `[`[], `[1], `[True]]\n", "list.element-type-mismatch", "True"),
        ("fn keep<A> values: Array<A> -> Array<A> = values\npub effect fn main = println (show (keep [[], `[1, 2], `[]]))\n", "array.element-type-mismatch", "`[1, 2]"),
    ] {
        let source = format!("// 雪🙂\n{source}");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input = || CompileInput::new("nested-invalid-雪🙂.ssrg", "proof/nested", &source).with_profile(profile);
            let error = compile_module(input()).unwrap_err();
            let diagnostic = error.diagnostics.iter().find(|d| d.message_key == key).unwrap_or_else(|| panic!("{source}\n{error:?}"));
            let range = &source[diagnostic.primary.start..diagnostic.primary.end];
            assert!(range.contains(rejected), "{source}\n{range}");
            assert_eq!(diagnostic.code, "SES-T0101");
            assert_eq!(analyze_module(input()).diagnostics, error);
        }
    }
}

#[test]
fn wholly_empty_generic_collections_still_need_context_and_explicit_types_remain_valid() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for expression in ["keep [[], []]", "keep<Int> [[], []]"] {
            let source = format!("fn keep<A> values: Array<Array<A>> -> Array<Array<A>> = values\npub fn probe -> Array<Array<Int>> = {expression}\n");
            compile_module(
                CompileInput::new("empty-context.ssrg", "proof/nested", &source)
                    .with_profile(profile),
            )
            .unwrap();
        }
        let source = "fn keep<A> values: Array<Array<A>> -> Array<Array<A>> = values\npub effect fn main = println (show (keep [[], []]))\n";
        assert!(compile_module(
            CompileInput::new("empty-unknown.ssrg", "proof/nested", source).with_profile(profile)
        )
        .is_err());
    }
}
