use seseragi_driver::{analyze_module, compile_module, BuildProfile, CompileInput};

fn reject_argument(source: &str, argument: &str) {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input = || {
            CompileInput::new("console-雫.ssrg", "fixture/console-contract", source)
                .with_profile(profile)
        };
        let error = compile_module(input()).expect_err("Console takes canonical String");
        let diagnostic = error
            .diagnostics
            .iter()
            .find(|d| d.code == "SES-T0101" && d.message_key == "call.argument-type-mismatch")
            .unwrap_or_else(|| panic!("{source}\n{error:#?}"));
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            argument,
            "{source}\n{error:#?}"
        );
        assert_eq!(analyze_module(input()).diagnostics, error);
    }
}

#[test]
fn prelude_and_imported_console_reject_every_non_string_value_family() {
    for (declarations, value) in [
        ("", "True"),
        ("", "42"),
        ("", "1.5"),
        ("", "'a'"),
        ("", "()"),
        ("", "[1, 2]"),
        ("", "`[1, 2]"),
        ("", "(1, True)"),
        ("", "{value: 1}"),
        ("", "Just 1"),
        ("fn identity value: Int -> Int = value\n", "identity"),
        ("struct String {value: Int}\n", "String {value: 1}"),
        ("opaque struct Secret {value: Int}\n", "Secret {value: 1}"),
    ] {
        for name in ["print", "println", "console.print", "console.println"] {
            reject_argument(
                &format!("// 雫🙂\nimport * as console from \"std/console\"\n{declarations}pub effect fn main = {name} ({value})\n"),
                &format!("({value})"),
            );
        }
    }
}

#[test]
fn saved_and_computed_console_callables_keep_the_string_argument_contract() {
    for binding in ["print", "println", "console.print", "console.println"] {
        reject_argument(
            &format!("import * as console from \"std/console\"\npub effect fn main = do {{ let output = {binding}; output True }}\n"),
            "True",
        );
        reject_argument(
            &format!(
                "import * as console from \"std/console\"\npub effect fn main = ({binding}) 42\n"
            ),
            "42",
        );
    }
    reject_argument(
        "import {println as output} from \"std/console\"\npub effect fn main = output True\n",
        "True",
    );
}

#[test]
fn console_never_identifies_a_scoped_string_binder_as_the_prelude_type() {
    for output in ["print", "println", "console.print", "console.println"] {
        reject_argument(
            &format!("import * as console from \"std/console\"\neffect fn bad<String> value: String = {output} value\npub effect fn main = bad 42\n"),
            "value",
        );
    }
}

#[test]
fn ordinary_string_arguments_distinguish_scoped_binders_from_intrinsic_values() {
    reject_argument(
        "fn textLength value: String -> Int = 1\npub fn unsound<String> value: String -> Int = textLength value\n",
        "value",
    );
    reject_argument(
        "fn textLength value: String -> Int = 1\npub fn unsound<String<_>> value: String<Int> -> Int = textLength value\n",
        "value",
    );
    for source in [
        "fn identity<String> value: String -> String = value\npub fn control -> String = identity \"valid\"\n",
        "alias Text = String\nfn textLength value: Text -> Int = 1\npub fn control -> Int = textLength \"valid\"\n",
        "fn identity<A> value: A -> A = value\npub fn number -> Int = identity 42\npub fn text -> String = identity \"valid\"\n",
    ] {
        compile_module(CompileInput::new("generic-control.ssrg", "fixture/console", source))
            .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
    }
}

#[test]
fn strings_aliases_explicit_show_and_user_shadowed_functions_remain_valid() {
    for source in [
        "pub effect fn main = do { print \"hello\"; println \" world\"; println `${42}`; println (show True); printValue [1, 2] }\n",
        "alias Text = String\nfn message -> Text = \"alias\"\npub effect fn main = println (message ())\n",
        "struct String {value: Int}\npub effect fn main = do { print \"canonical\"; println \" string\" }\n",
        "effect fn print value: Int = succeed value\neffect fn println value: Bool = succeed value\npub effect fn main = println True\n",
        "import * as console from \"std/console\"\npub effect fn main = do { let output = print; output \"prelude\"; let qualified = console.println; qualified \"qualified\" }\n",
        "import {println as output} from \"std/console\"\npub effect fn main = output \"renamed\"\n",
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            compile_module(
                CompileInput::new("console.ssrg", "fixture/console-contract", source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        }
    }
}

#[test]
fn explicit_console_requirements_and_inferred_cleanup_keep_the_same_owner() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/unit-binder-function-effects.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("cleanup.ssrg", "fixture/console", source).with_profile(profile),
        )
        .expect("the existing declared acquisition and inferred finalizer stay compatible");
        let acquire = compiled
            .core_ir
            .functions
            .iter()
            .find(|function| function.symbol == "fixture/console::acquire")
            .unwrap();
        let seseragi_lowering::CoreType::Function { result, .. } =
            acquire.value_type.as_ref().unwrap()
        else {
            panic!("{acquire:?}")
        };
        let seseragi_lowering::CoreType::Named { name, arguments } = result.as_ref() else {
            panic!("{result:?}")
        };
        assert_eq!(name, "std/prelude::Effect");
        let seseragi_lowering::CoreType::Record { closed, fields } = &arguments[0] else {
            panic!("{arguments:?}")
        };
        assert!(*closed);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "console");
        assert!(matches!(&fields[0].type_ref,
            seseragi_lowering::CoreType::ExternalNamed { canonical, arguments, .. }
            if canonical == "std/prelude::Console" && arguments.is_empty()));
    }
    for source in [
        "struct Console {value: Int}\npub effect fn main -> Unit\nwith Console\nfails ConsoleError = println \"bad owner\"\n",
        "pub effect fn main<Console> value: Console -> Unit\nwith Console\nfails ConsoleError = println \"bad binder\"\n",
    ] {
        let error = compile_module(CompileInput::new("shadow.ssrg", "fixture/console", source))
            .expect_err("a local owner cannot satisfy the Prelude Console requirement");
        assert!(error.diagnostics.iter().any(|diagnostic|
            diagnostic.code == "SES-E0001" && diagnostic.message_key == "effect.explicit-environment-mismatch"), "{error:#?}");
    }
}
