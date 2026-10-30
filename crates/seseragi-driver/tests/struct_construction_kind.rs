use seseragi_driver::{analyze_module, compile_module, BuildProfile, CompileInput};

#[test]
fn nested_record_struct_fields_survive_compile_and_analysis_in_both_profiles() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend-regressions/nested-record-struct-field.ssrg"
    );
    let construction = "// 雪🙂\nstruct Envelope<A> { first: A, payload: { label: String }, after: Bool }\nlet value = Envelope { first: 7, payload: { label: \"雪🙂\" }, after: True }\nfn probe value: Envelope<Int> -> Bool = value.after\n";
    for source in [original, construction] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input = || {
                CompileInput::new("nested-雪🙂.ssrg", "proof/struct", source).with_profile(profile)
            };
            let compiled = compile_module(input()).unwrap();
            assert!(analyze_module(input()).diagnostics.diagnostics.is_empty());
            assert_eq!(
                compiled.core_ir.structs[0]
                    .fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>(),
                ["first", "payload", "after"]
            );
        }
    }
}

#[test]
fn fields_after_nested_records_keep_their_actual_type_error_and_utf8_span() {
    let source = "// 雪🙂\nstruct Envelope { payload: { label: String }, after: Bool }\nlet value = Envelope { payload: { label: \"雪🙂\" }, after: 42 }\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input =
            || CompileInput::new("nested-雪🙂.ssrg", "proof/struct", source).with_profile(profile);
        let error = compile_module(input()).unwrap_err();
        let diagnostic = error
            .diagnostics
            .iter()
            .find(|d| d.message_key == "struct.field-type-mismatch")
            .unwrap_or_else(|| panic!("{error:?}"));
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            "42"
        );
        assert_eq!(analyze_module(input()).diagnostics, error);
    }
}

#[test]
fn adts_and_newtypes_are_not_unchecked_struct_record_casts() {
    for source in [
        "// 雫🙂\ntype Key = | Key { value: Int }\nlet invalid = Key { value: 42 }\n",
        "// 雫🙂\nnewtype Key = { value: Int }\nlet invalid = Key { value: 42 }\n",
        "// 雫🙂\ntype Key = | First { value: Int } | Second\nlet invalid = Key { value: 42 }\n",
        "// 雫🙂\nalias Key = Int\nlet invalid = Key {}\n",
        "// 雫🙂\nimport * as maps from \"std/map\"\nalias Key = maps.Map<String, Int>\nlet invalid = Key {}\n",
        "// 雫🙂\ntype Wrapped = | Wrap Int\nalias Key = Wrapped\nlet invalid = Key {}\n",
        "// 雫🙂\nlet invalid = Int {}\n",
        "// 雫🙂\nstruct Box<A> { value: A }\nalias Key<A> = Box<A>\nlet invalid = Key<Int, Bool> { value: 42 }\n",
        include_str!("fixtures/struct-style-adt-map.ssrg"),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input = || {
                CompileInput::new("struct-雪🙂.ssrg", "proof/struct", source).with_profile(profile)
            };
            let error =
                compile_module(input()).expect_err("construction needs a checked struct target");
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
                    if source.contains("let invalid = Int {") { "Int" } else { "Key" }
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
        "struct Box<A> { value: A }\nalias Alias<A> = Box<A>\nlet valid = Alias<Int> { value: 42 }\n",
        "struct Pair<A, B> { first: A, second: B }\nalias Swap<A, B> = Pair<B, A>\nlet valid = Swap<Int, String> { first: \"snow\", second: 42 }\n",
        "struct Box<A> { value: A }\nalias Fixed = Box<Int>\nlet valid = Fixed { value: 42 }\n",
        "struct Box<A> { value: A }\nalias Alias<A> = Box<A>\nfn make<A> value: A -> Alias<A> = Alias<A> { value }\nlet valid = make 42\n",
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

#[test]
fn aliases_check_fields_against_the_canonical_struct_arguments() {
    for (source, key, expected) in [
        ("struct Box { value: Int }\nalias Key = Box\nlet invalid = Key { value: False }\n", "struct.field-type-mismatch", "False"),
        ("struct Box<A> { value: A }\nalias Key = Box<Int>\nlet invalid = Key { value: False }\n", "struct.field-type-mismatch", "False"),
        ("struct Pair<A, B> { first: A, second: B }\nalias Key<A> = Pair<Int, A>\nlet invalid = Key<String> { first: False, second: \"snow\" }\n", "struct.field-type-mismatch", "False"),
        ("struct Pair<A, B> { first: A, second: B }\nalias Key<A> = Pair<A, A>\nlet invalid = Key<Int> { first: 1, second: False }\n", "struct.field-type-mismatch", "False"),
        ("import * as maps from \"std/map\"\nstruct Store { values: maps.Map<String, Int> }\nalias Key = Store\nlet invalid = Key { values: {} }\n", "struct.field-type-mismatch", "{}"),
        ("struct Box { value: Int }\nalias Key = Box\nlet invalid = Key {}\n", "struct.field-missing", "Key"),
    ] {
        let source=format!("// 雫🙂\n{source}");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input=||CompileInput::new("alias-fields-雪🙂.ssrg","proof/struct",&source).with_profile(profile);
            let error=compile_module(input()).expect_err("alias fields require their resolved struct contracts");
            let diagnostic=error.diagnostics.iter().find(|d|d.message_key==key).unwrap_or_else(||panic!("{source}\n{error:?}"));
            assert_eq!(&source[diagnostic.primary.start..diagnostic.primary.end],expected);
            assert_eq!(analyze_module(input()).diagnostics,error);
        }
    }
}
