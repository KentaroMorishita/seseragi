use seseragi_driver::{analyze_module, compile_module, BuildProfile, CompileInput};

#[test]
fn adts_and_newtypes_are_not_unchecked_struct_record_casts() {
    for source in [
        "// 雫🙂\ntype Key = | Key { value: Int }\nlet invalid = Key { value: 42 }\n",
        "// 雫🙂\nnewtype Key = { value: Int }\nlet invalid = Key { value: 42 }\n",
        "// 雫🙂\ntype Key = | First { value: Int } | Second\nlet invalid = Key { value: 42 }\n",
        include_str!("fixtures/struct-style-adt-map.ssrg"),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input = || {
                CompileInput::new("struct-雪🙂.ssrg", "proof/struct", source).with_profile(profile)
            };
            let error =
                compile_module(input()).expect_err("ADT/newtype needs constructor application");
            let diagnostics: Vec<_> = error
                .diagnostics
                .iter()
                .filter(|d| d.message_key == "struct.construction-on-non-struct")
                .collect();
            assert!(!diagnostics.is_empty(), "{error:?}");
            for diagnostic in diagnostics {
                assert_eq!(diagnostic.code, "SES-T0101");
                assert_eq!(
                    &source[diagnostic.primary.start..diagnostic.primary.end],
                    "Key"
                );
            }
            assert_eq!(analyze_module(input()).diagnostics, error);
        }
    }
}

#[test]
fn ordinary_struct_alias_and_explicit_adt_record_payload_remain_valid() {
    for source in [
        "struct Key { value: Int }\nlet valid = Key { value: 42 }\n",
        "struct Key { value: Int }\nalias Alias = Key\nlet valid = Alias { value: 42 }\n",
        "type Key = | Key { value: Int }\nlet valid = Key ({ value: 42 })\n",
        "newtype Key = { value: Int }\nlet valid = Key ({ value: 42 })\n",
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            compile_module(
                CompileInput::new("struct-control.ssrg", "proof/struct", source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{source}\n{error:?}"));
        }
    }
}
