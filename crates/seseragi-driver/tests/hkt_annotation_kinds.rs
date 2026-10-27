use seseragi_driver::{
    compile_module, compile_project, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;

const DATA: &str = "type Box<F<_>, A> = | Box F<A>\nstruct Holder<F<_>, A> { value: F<A> }\n";

fn assert_kind_rejected(source: &str, argument: &str) {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let error = compile_module(
            CompileInput::new("annotation-雫.ssrg", "fixture/hkt-annotation", source)
                .with_profile(profile),
        )
        .unwrap_err();
        let diagnostic = error
            .diagnostics
            .iter()
            .find(|d| d.code == "SES-T0604" && d.message_key == "type.argument-kind-mismatch")
            .unwrap_or_else(|| panic!("{source}\n{error:#?}"));
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            argument,
            "{source}\n{error:#?}"
        );
        assert_ne!(diagnostic.message(), diagnostic.message_key);
        assert!(!diagnostic.helps().is_empty());
    }
}

#[test]
fn unused_adt_and_struct_annotations_reject_incompatible_argument_kinds() {
    for (annotation, argument) in [
        ("Box<Int, Int>", "Int"),
        ("Holder<Either, Int>", "Either"),
        ("Box<Maybe<Int>, Int>", "Maybe<Int>"),
        ("Holder<Maybe, Maybe>", "Maybe"),
        // The one remaining slot requires a constructor, not a value type.
        ("Holder<Box<_, Int>, Int>", "Box<_, Int>"),
    ] {
        assert_kind_rejected(
            &format!("// 雫🙂\n{DATA}pub fn wrong value: {annotation} -> Int = 0\n"),
            argument,
        );
    }
}

#[test]
fn declared_constructor_and_fixed_prefix_arguments_remain_valid() {
    let source = format!(
        "{DATA}\n\
         fn maybe value: Box<Maybe, Int> -> Unit = ()\n\
         fn either value: Holder<Either<String, _>, Int> -> Unit = ()\n\
         fn higher value: Holder<Box<Maybe, _>, Int> -> Unit = ()\n\
         fn generic<F<_>, A> value: Box<F, A> -> Unit = ()\n\
         fn aliasCase value: Box<Optional<_>, Int> -> Unit = ()\n\
         alias Optional<A> = Maybe<A>\n"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile_module(
            CompileInput::new("valid-kind.ssrg", "fixture/hkt-annotation", &source)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
    }
}

#[test]
fn scoped_binder_kinds_are_independent_of_same_named_builtins_and_nominals() {
    let valid = format!(
        "{DATA}\n\
         type Int<A> = | OwnInt A\n\
         fn nominal value: Box<Int, String> -> Unit = ()\n\
         fn sibling<Maybe<_>, A> value: Holder<Maybe, A> -> Unit = ()\n\
         pub fn outer -> Unit = {{\n\
           fn local<Maybe<_>, A> value: Box<Maybe, A> -> Unit = ()\n\
           ()\n\
         }}\n"
    );
    compile_module(CompileInput::new(
        "scoped-kind.ssrg",
        "fixture/hkt-annotation",
        &valid,
    ))
    .unwrap_or_else(|error| panic!("{valid}\n{error:#?}"));

    for source in [
        format!("{DATA}fn wrong<Maybe, A> value: Box<Maybe, A> -> Unit = ()\n"),
        format!("{DATA}type Maybe = | LocalMaybe\nfn wrong value: Box<Maybe, Int> -> Unit = ()\n"),
        format!("{DATA}pub fn outer -> Unit = {{ fn local<Maybe, A> value: Box<Maybe, A> -> Unit = (); () }}\n"),
    ] {
        assert_kind_rejected(&source, "Maybe");
    }
}

#[test]
fn unused_nested_field_and_do_let_annotations_are_checked() {
    for source in [
        format!("{DATA}type Envelope = | Wrapped Box<Int, Int>\n"),
        format!("{DATA}struct Envelope {{ wrapped: Holder<Int, Int> }}\n"),
        format!("{DATA}pub effect fn main = do {{ let unused: Box<Int, Int> -> Int = \\value -> 0; println \"ok\" }}\n"),
    ] {
        assert_kind_rejected(&source, "Int");
    }
}

fn imported_project(
    source: &str,
    profile: BuildProfile,
) -> Result<seseragi_driver::CompiledProject, seseragi_driver::ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    graph
        .add_module("fixture/model::lib".to_owned(), [])
        .unwrap();
    graph
        .add_module(
            "fixture/app::facade".to_owned(),
            [("../model/lib".to_owned(), "fixture/model::lib".to_owned())],
        )
        .unwrap();
    graph
        .add_module(
            "fixture/app::main".to_owned(),
            [("./facade".to_owned(), "fixture/app::facade".to_owned())],
        )
        .unwrap();
    compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "model/lib.ssrg",
                "fixture/model::lib",
                "pub type Box<F<_>, A> = | Box F<A>\npub struct Holder<F<_>, A> { value: F<A> }\n",
                "dist/model.js",
            )
            .with_profile(profile),
            ProjectModuleInput::new(
                "app/facade.ssrg",
                "fixture/app::facade",
                "pub import { Box, Holder } from \"../model/lib\"\n",
                "dist/facade.js",
            )
            .with_profile(profile),
            ProjectModuleInput::new("app/main.ssrg", "fixture/app::main", source, "dist/main.js")
                .with_profile(profile),
        ],
    )
}

#[test]
fn imported_nominal_parameters_keep_declared_kinds_through_a_facade() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        imported_project(
            "import { Box as Foreign, Holder } from \"./facade\"\ntype Box = | LocalBox\nfn valid value: Foreign<Maybe, Int> -> Unit = ()\nfn prefix value: Holder<Either<String, _>, Int> -> Unit = ()\n",
            profile,
        ).unwrap_or_else(|error| panic!("{error:#?}"));
        let source = "import { Box as Foreign } from \"./facade\"\ntype Box<F<_>, A> = | LocalBox F<A>\nfn wrong value: Foreign<Int, Int> -> Unit = ()\n";
        let error = imported_project(source, profile).unwrap_err();
        let seseragi_driver::ProjectCompileError::Diagnostics { modules } = &error else {
            panic!("{error:#?}");
        };
        let diagnostic = modules
            .iter()
            .flat_map(|module| &module.diagnostics.diagnostics)
            .find(|d| d.code == "SES-T0604" && d.message_key == "type.argument-kind-mismatch")
            .unwrap_or_else(|| panic!("{error:#?}"));
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            "Int"
        );
    }
}

#[test]
fn equal_constructor_kinds_do_not_merge_distinct_nominal_owners() {
    let source = format!(
        "{DATA}type First<A> = | First A\ntype Second<A> = | Second A\nfn wrong value: Box<First, Int> -> Box<Second, Int> = value\n"
    );
    let error = compile_module(CompileInput::new(
        "kind-owner.ssrg",
        "fixture/hkt-annotation",
        &source,
    ))
    .unwrap_err();
    assert!(
        error.diagnostics.iter().any(|d| d.code == "SES-T0101"),
        "{error:#?}"
    );
    assert!(
        error
            .diagnostics
            .iter()
            .all(|d| d.message_key != "type.argument-kind-mismatch"),
        "{error:#?}"
    );
}
