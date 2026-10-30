use seseragi_driver::{analyze_module, compile_module, BuildProfile, CompileInput};

#[test]
fn checked_newtype_literals_keep_record_array_and_tuple_representations_in_both_profiles() {
    let source="// 雪🙂\nnewtype Row<A> = { value: A }\nnewtype Items<A> = Array<A>\nnewtype Pair<A> = (A, String)\npub fn row -> Row<Array<String>> = Row ({ value: [\"雪🙂\"] })\npub fn items -> Items<String> = Items [\"雪🙂\"]\npub fn pair -> Pair<Array<Int>> = Pair ([1,2], \"雪🙂\")\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input = || {
            CompileInput::new("newtype-雪🙂.ssrg", "proof/newtypes", source).with_profile(profile)
        };
        compile_module(input()).unwrap();
        assert!(analyze_module(input()).diagnostics.diagnostics.is_empty());
    }
}

#[test]
fn representation_casts_still_require_checked_source_arguments() {
    for source in [
        "newtype Row = { value: String }\nlet invalid = Row ({ value: 42 })\n",
        "newtype Items = Array<String>\nlet invalid = Items [42]\n",
        "newtype First = String\nnewtype Second = String\nlet invalid = First (Second \"snow\")\n",
    ] {
        let source = format!("// 雪🙂\n{source}");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input = || {
                CompileInput::new("newtype-雪🙂.ssrg", "proof/newtypes", &source)
                    .with_profile(profile)
            };
            let error = compile_module(input())
                .expect_err("newtype representation must be checked before erasure");
            assert!(error.diagnostics.iter().any(|d| d.code == "SES-T0101"));
            assert_eq!(analyze_module(input()).diagnostics, error);
        }
    }
}
