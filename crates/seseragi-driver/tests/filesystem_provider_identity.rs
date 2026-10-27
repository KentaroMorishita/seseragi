use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_lowering::CoreType;

const TRANSFORM: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-transform.ssrg");

#[test]
fn original_file_transform_keeps_the_selected_provider_owner_in_every_declared_effect() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("filesystem.ssrg", "fixture/filesystem", TRANSFORM)
                .with_profile(profile),
        )
        .expect("the original with FileSystem contract must match imported fs operations");
        for name in ["transform", "main"] {
            let function = compiled
                .core_ir
                .functions
                .iter()
                .find(|function| function.symbol == format!("fixture/filesystem::{name}"))
                .unwrap();
            let CoreType::Function { result, .. } = function.value_type.as_ref().unwrap() else {
                panic!("{function:?}")
            };
            let CoreType::Named { name, arguments } = result.as_ref() else {
                panic!("{result:?}")
            };
            assert_eq!(name, "std/prelude::Effect");
            let CoreType::Record { closed, fields } = &arguments[0] else {
                panic!("{arguments:?}")
            };
            assert!(*closed);
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name, "fileSystem");
            assert!(matches!(&fields[0].type_ref,
                CoreType::ExternalNamed { canonical, arguments, .. }
                if canonical == "std/fs::FileSystem" && arguments.is_empty()));
        }
        assert!(compiled
            .typed_hir
            .external_type_bindings
            .iter()
            .any(|binding| binding.spelling == "FileSystem"
                && binding.canonical == "std/fs::FileSystem"
                && binding.arity == Some(0)
                && binding.provider.as_ref().is_some_and(
                    |provider| provider.module == "std/fs" && provider.export == "FileSystem"
                )));
    }
}

#[test]
fn explicit_fields_aliases_and_renamed_provider_imports_keep_the_same_identity() {
    let explicit = TRANSFORM.replace("with FileSystem", "with fileSystem: FileSystem");
    let alias = format!(
        "alias Disk = FileSystem\n{}",
        TRANSFORM.replace("with FileSystem", "with fileSystem: Disk")
    );
    let imported = format!(
        "import {{FileSystem as Disk}} from \"std/fs\"\n{}",
        TRANSFORM.replace("with FileSystem", "with fileSystem: Disk")
    );
    let scoped_control =
        format!("fn identity<FileSystem> value: FileSystem -> FileSystem = value\n{TRANSFORM}");
    for source in [&explicit, &alias, &imported, &scoped_control] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            compile_module(
                CompileInput::new("filesystem.ssrg", "fixture/filesystem", source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        }
    }
}

#[test]
fn local_and_scoped_same_named_types_cannot_satisfy_the_provider_requirement() {
    for declaration in ["struct FileSystem {}\n", "alias FileSystem = Int\n", ""] {
        let binder = if declaration.is_empty() {
            "<FileSystem>"
        } else {
            ""
        };
        let source = format!(
            "import * as fs from \"std/fs\"\nimport * as paths from \"std/path\"\n{declaration}pub effect fn wrong{binder} -> String with FileSystem fails fs.FileTextError = fs.readTextUtf8 (paths.current ())\n"
        );
        let requirement = source.find("with FileSystem").unwrap() + "with ".len();
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let error = compile_module(
                CompileInput::new("filesystem.ssrg", "fixture/filesystem", &source)
                    .with_profile(profile),
            )
            .expect_err("a local declaration or binder cannot prove a standard provider owner");
            let diagnostic = error
                .diagnostics
                .iter()
                .find(|diagnostic| {
                    diagnostic.code == "SES-E0001"
                        && diagnostic.message_key == "effect.explicit-environment-mismatch"
                })
                .unwrap_or_else(|| panic!("{source}\n{error:#?}"));
            assert_eq!(diagnostic.primary.start, requirement);
            assert_eq!(
                &source[diagnostic.primary.start..diagnostic.primary.end],
                "FileSystem"
            );
        }
    }
}

#[test]
fn filesystem_and_clock_requirements_keep_distinct_canonical_provider_owners() {
    let source = "import * as fs from \"std/fs\"\nimport * as paths from \"std/path\"\nimport * as clock from \"std/clock\"\nimport * as time from \"std/time\"\nimport {Clock} from \"std/clock\"\neffect fn wait pause: time.Duration -> Unit with Clock = clock.sleep pause\npub effect fn readAfter pause: time.Duration -> String with FileSystem, Clock fails fs.FileTextError = do { wait pause; fs.readTextUtf8 (paths.current ()) }\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("providers.ssrg", "fixture/providers", source).with_profile(profile),
        )
        .expect("existing provider imports and implicit FileSystem must compose");
        let function = compiled
            .core_ir
            .functions
            .iter()
            .find(|function| function.symbol == "fixture/providers::readAfter")
            .unwrap();
        let CoreType::Function { result, .. } = function.value_type.as_ref().unwrap() else {
            panic!("{function:?}")
        };
        let CoreType::Named { arguments, .. } = result.as_ref() else {
            panic!("{result:?}")
        };
        let CoreType::Record { fields, .. } = &arguments[0] else {
            panic!("{arguments:?}")
        };
        assert_eq!(fields.len(), 2);
        for (field, owner) in [
            ("fileSystem", "std/fs::FileSystem"),
            ("clock", "std/clock::Clock"),
        ] {
            assert!(fields.iter().any(|value| value.name == field
                && matches!(&value.type_ref, CoreType::ExternalNamed { canonical, arguments, .. }
                    if canonical == owner && arguments.is_empty())), "{fields:?}");
        }
    }
    let wrong_owner = source.replace("import {Clock} from \"std/clock\"", "struct Clock {}");
    let error = compile_module(CompileInput::new(
        "providers.ssrg",
        "fixture/providers",
        &wrong_owner,
    ))
    .expect_err("a same-named local Clock cannot satisfy the imported provider");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-E0001"
                && diagnostic.message_key == "effect.explicit-environment-mismatch"),
        "{error:#?}"
    );
}
