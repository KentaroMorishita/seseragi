use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn prelude_stdin_requirement_matches_the_imported_standard_provider() {
    for (prefix, requirement, failure) in [
        ("", "Stdin", "StdinError"),
        ("", "stdin: Stdin", "StdinError"),
        (
            "import {Stdin as Input, StdinError as InputError} from \"std/stdin\"\n",
            "stdin: Input",
            "InputError",
        ),
        (
            "alias Input = Stdin\nalias InputError = StdinError\n",
            "stdin: Input",
            "InputError",
        ),
    ] {
        for (import, operation) in [
            ("import * as stdin from \"std/stdin\"\n", "stdin.readLine"),
            ("", "readLine"),
        ] {
            let source = format!("{import}{prefix}pub effect fn read -> Maybe<String> with {requirement} fails {failure} = {operation} ()\n");
            for profile in [BuildProfile::Development, BuildProfile::Release] {
                compile_module(
                    CompileInput::new("stdin.ssrg", "fixture/stdin-provider", &source)
                        .with_profile(profile),
                )
                .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
            }
        }
    }
}

#[test]
fn same_named_nominal_and_binder_do_not_satisfy_the_standard_provider_contract() {
    for declaration in [
        "struct Stdin { token: Int }\npub effect fn read -> Maybe<String> with Stdin fails StdinError = stdin.readLine ()",
        "pub effect fn read<Stdin> value: Stdin -> Maybe<String> with Stdin fails StdinError = stdin.readLine ()",
        "struct Fake { token: Int }\nalias Stdin = Fake\npub effect fn read -> Maybe<String> with Stdin fails StdinError = stdin.readLine ()",
        "struct StdinError { token: Int }\npub effect fn read -> Maybe<String> with Stdin fails StdinError = stdin.readLine ()",
    ] {
        let source = format!("import * as stdin from \"std/stdin\"\n{declaration}\n");
        let errors = compile_module(CompileInput::new("shadow.ssrg", "fixture/stdin-shadow", &source))
            .expect_err("a same-spelled type cannot prove the standard provider contract");
        assert!(errors.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-E0001"
                && diagnostic.primary.start < diagnostic.primary.end
                && matches!(diagnostic.message_key.as_str(),
                    "effect.explicit-environment-mismatch" | "effect.explicit-failure-mismatch")
        }), "{source}\n{errors:#?}");
    }
}

#[test]
fn implicit_prelude_read_line_keeps_provider_provenance_without_type_annotations() {
    let source = "pub effect fn read = readLine ()\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile_module(
            CompileInput::new("implicit.ssrg", "fixture/implicit-stdin", source)
                .with_profile(profile),
        )
        .unwrap();
    }
}
