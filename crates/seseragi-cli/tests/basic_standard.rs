use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-basic-standard-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

struct Case<'a> {
    name: &'a str,
    stdout: &'a [u8],
    input: Option<&'a [u8]>,
    output: Option<&'a [u8]>,
    exit: i32,
    failure: Option<&'a str>,
    defect: bool,
    native_defect: &'a str,
    hash_seed: Option<&'a str>,
}
impl<'a> Case<'a> {
    fn success(stdout: &'a [u8]) -> Self {
        Self {
            name: "success",
            stdout,
            input: None,
            output: None,
            exit: 0,
            failure: None,
            defect: false,
            native_defect: "deliberate",
            hash_seed: None,
        }
    }
}

fn compare(source: &Path, cases: &[Case<'_>]) {
    let scratch = Scratch::new();
    let original = source.is_file().then(|| fs::read(source).unwrap());
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let mut programs = vec![(
        "native".to_owned(),
        native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        None,
        true,
    )];
    for (name, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let out = scratch.0.join(name);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", name, "--out-dir"])
                .arg(&out),
            &scratch.0,
        ));
        let mut files = if source.is_dir() {
            let loaded = seseragi_project::load_local_project(source).unwrap();
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
                    .unwrap();
            assert_eq!(compiled.compiled.modules.len(), 2);
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let relative = Path::new(&module.generated.metadata.outputs.typescript);
                assert!(
                    !relative.is_absolute()
                        && !relative
                            .components()
                            .any(|c| c == std::path::Component::ParentDir)
                );
                let raw = out.join("raw").join(relative);
                fs::create_dir_all(raw.parent().unwrap()).unwrap();
                fs::write(&raw, &module.generated.typescript).unwrap();
                files.push(raw);
            }
            files
        } else {
            let compiled = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new(
                    "basic-text-math.ssrg",
                    "basic/text",
                    std::str::from_utf8(original.as_ref().unwrap()).unwrap(),
                )
                .with_profile(profile),
            )
            .unwrap();
            let raw = out.join("raw-original.ts");
            fs::write(&raw, compiled.generated.typescript).unwrap();
            vec![raw]
        };
        if name == "development" {
            files.push(out.join("entry.ts"));
        }
        let config = serde_json::json!({"compilerOptions": {
            "strict": true, "noEmit": true, "skipLibCheck": false,
            "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
            "allowImportingTsExtensions": true, "types": ["bun", "node"],
            "typeRoots": [root().join("node_modules/@types")], "baseUrl": root(),
            "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
            "lib": ["ES2023", "DOM", "DOM.Iterable"]
        }, "files": files});
        let config_path = out.join("raw-tsconfig.json");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(&config_path),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["bun", "node"] {
            programs.push((
                format!("{name}-{engine}"),
                runtime(engine, &scratch.0),
                Some(out.join(manifest["entry"].as_str().unwrap())),
                false,
            ));
        }
    }
    assert_eq!(programs.len(), 5);
    for (label, executable, argument, is_native) in programs {
        for case in cases {
            let label = format!("{label}-{}", case.name);
            let directory = scratch.0.join(&label);
            fs::create_dir(&directory).unwrap();
            if let Some(input) = case.input {
                fs::write(directory.join("input.txt"), input).unwrap();
                fs::write(directory.join("output.txt"), b"old output\n").unwrap();
            }
            let mut command = Command::new(&executable);
            command.current_dir(&directory).env("PATH", "");
            if let Some(seed) = case.hash_seed {
                command.env("SESERAGI_HASH_SEED", seed);
            }
            if let Some(argument) = &argument {
                command.arg(argument);
            }
            if is_native {
                command.env("SESERAGI_NATIVE_METRICS", "1");
            }
            let result = bounded(&mut command, &directory);
            assert_eq!(result.status.code(), Some(case.exit), "{label}: {result:?}");
            assert_eq!(result.stdout, case.stdout, "{label}");
            let stderr = String::from_utf8_lossy(&result.stderr);
            if is_native {
                assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
                if case.defect {
                    assert!(stderr.contains(case.native_defect), "{label}: {stderr}");
                }
            } else if case.defect {
                assert_eq!(stderr, "seseragi: runtime defect\n", "{label}");
            } else if case.failure.is_none() {
                assert!(stderr.is_empty(), "{label}: {stderr}");
            }
            if let Some(failure) = case.failure {
                assert!(stderr.contains(failure), "{label}: {stderr}");
            }
            if let Some(input) = case.input {
                assert_eq!(
                    fs::read(directory.join("input.txt")).unwrap(),
                    input,
                    "{label}"
                );
                assert_eq!(
                    fs::read(directory.join("output.txt")).unwrap(),
                    case.output.unwrap_or(b"old output\n"),
                    "{label}"
                );
            }
        }
    }
    if let Some(original) = original {
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
fn basic_long_effect_sequence_keeps_every_statement_in_both_profiles() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-long-effect-sequence.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-long-effect-sequence.stdout"
        ))],
    );
}

#[test]
fn inferred_builtin_array_owner_keeps_original_strict_ts_bun_node_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-regressions/inferred-builtin-array-owner.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend-regressions/inferred-builtin-array-owner.stdout"
        ))],
    );
}

#[test]
fn basic_text_and_math_match_original_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-text-math.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-math.stdout"
        ))],
    );
}

#[test]
fn basic_text_boundaries_keep_original_unicode_empty_saved_and_generic_results() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-text-boundaries.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-boundaries.stdout"
        ))],
    );
}

#[test]
fn basic_text_boundaries_path_package_keeps_opaque_imported_saved_predicates() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-text-boundaries"),
        &[Case::success(b"True|True|True|False|True\n")],
    );
}

#[test]
fn basic_text_boundaries_keep_cold_replay_and_release_ref_cycles_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-text-boundary-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-boundary-effects.stdout"
        ))],
    );
}

#[test]
fn basic_text_boundaries_argument_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-text-boundary-effects.ssrg"
    );
    assert_eq!(original.matches("prefix \"雪🙂\"").count(), 1);
    let path = scratch.0.join("text-boundary-defect.ssrg");
    fs::write(
        &path,
        original.replace(
            "prefix \"雪🙂\"",
            "text.startsWith (show (42 / 0)) \"雪🙂\"",
        ),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "argument-defect",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_bytes_match_original_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-bytes.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-bytes.stdout"
        ))],
    );
}

#[test]
fn basic_either_operations_keep_original_selected_branches_traversals_and_returned_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-either-operations.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-either-operations.stdout"
        ))],
    );
}
#[test]
fn basic_either_operations_keep_cold_captured_callbacks_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-either-operations-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-either-operations-effects.stdout"
        ))],
    );
}
#[test]
fn basic_either_operations_source_callback_and_returned_stage_defects_release_every_root() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-either-operations-effects.ssrg"
    );
    let needle = "either.traverse choose [1,2,3]";
    assert_eq!(original.matches(needle).count(), 1);
    for expression in [
        "either.traverse (callback (42 / 0)) [1,2,3]",
        "either.traverse choose (if 42 / 0 == 0 then [1] else [2])",
        "either.traverse (\\value:Int -> if value == 2 then Right (value / 0) else Left \"first\") [1,2,3]",
        "{let _selected = either.mapLeft (\\message:String -> message + show (42 / 0)) (left ());Right [1]}",
        "{let _selected = either.mapRight (\\value:Int -> value / 0) (right ());Right [1]}",
        "{let _selected = either.bimap (errorCallback (42 / 0)) (\\value:Int -> value + 1) (right ());Right [1]}",
        "{let _selected = either.bimap (\\message:String -> message) (\\value:Int -> value + 1) (if 42 / 0 == 0 then left () else right ());Right [1]}",
        "{let _selected = either.fold (\\message:String -> \\value:Int -> value + 1) (\\number:Int -> \\value:Int -> number + value / 0) (right ()) 42;Right [1]}",
    ] {
        let expression = format!("{{let result:Either<String,Array<Int>> = {expression};result}}");
        let scratch=Scratch::new();let path=scratch.0.join("either-defect.ssrg");fs::write(&path,original.replace(needle,&expression)).unwrap();
        compare(&path,&[Case {stdout:b"created\nacquired\ncold\n[11, 12, 13]\ncold\n[11, 12, 13]\nreleased\n",exit:70,defect:true,native_defect:"Int division by zero",..Case::success(b"")}]);
    }
}
#[test]
fn basic_either_operations_path_package_preserves_opaque_error_element_hkt_alias_and_dictionaries()
{
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-either-operations"),
        &[Case::success(
            b"right:box:7\nleft:missing\nright:empty\nright:box:9\nright:box:9\nerror:missing\n",
        )],
    );
}
#[test]
fn basic_either_operations_original_hkt_partial_sum_context_matches_strict_ts_and_native() {
    compare(
        &root()
            .join("examples/spec/fixtures/rust-backend-regressions/hkt-partial-sum-context.ssrg"),
        &[Case::success("42\n雪\n".as_bytes())],
    );
}

#[test]
fn basic_either_operations_original_saved_runtime_type_regression_matches_strict_ts_and_native() {
    compare(
        &root().join(
            "examples/spec/fixtures/rust-backend-regressions/either-saved-runtime-types.ssrg",
        ),
        &[Case::success(b"[2, 3]\n")],
    );
}
#[test]
fn basic_either_operations_host_error_and_global_this_source_names_keep_raw_strict_and_runtime_behavior(
) {
    for name in ["host-error-shadow", "host-global-this-binding"] {
        compare(
            &root().join(format!(
                "examples/spec/fixtures/rust-backend-regressions/{name}.ssrg"
            )),
            &[Case::success(b"42\n")],
        );
    }
}

#[test]
fn basic_maybe_operations_keep_original_fallbacks_traversals_saved_and_returned_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-maybe-operations.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-maybe-operations.stdout"
        ))],
    );
}
#[test]
fn basic_maybe_operations_keep_cold_captured_callbacks_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-maybe-operations-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-maybe-operations-effects.stdout"
        ))],
    );
}
#[test]
fn basic_maybe_operations_source_callback_and_returned_stage_defects_release_every_root() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-maybe-operations-effects.ssrg"
    );
    let needle = "maybe.traverse choose [1,2,3]";
    assert_eq!(original.matches(needle).count(), 1);
    for expression in [
        "maybe.traverse (callback (42 / 0)) [1,2,3]",
        "maybe.traverse choose (if 42 / 0 == 0 then [1] else [2])",
        "maybe.traverse (\\value:Int -> if value == 2 then Just (value / 0) else Nothing) [1,2,3]",
        "{let _selected = maybe.withDefault (42 / 0) (Just 7);Just [1]}",
        "{let _selected = maybe.orElse (Just (42 / 0)) (Just 7);Just [1]}",
        "{let _selected = maybe.withDefault (\\value:Int -> value + 1) (Just (\\value:Int -> value / 0)) 42;Just [1]}",
        "{let _selected = maybe.withDefault (\\value:Int -> value + 1) (Just (\\value:Int -> value + 2)) (42 / 0);Just [1]}",
        "{let _selected = maybe.withDefault (\\left:Int -> \\right:Int -> left + right) (Just (\\left:Int -> if left / 0 == 0 then (\\right:Int -> right) else (\\right:Int -> right))) 1 (42 / 0);Just [1]}",
    ] {
        let scratch = Scratch::new();
        let path = scratch.0.join("maybe-defect.ssrg");
        fs::write(&path, original.replace(needle, expression)).unwrap();
        compare(&path, &[Case {
            stdout: b"created\nacquired\ncold\n[11, 12, 13]\ncold\n[11, 12, 13]\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }]);
    }
}
#[test]
fn basic_maybe_operations_path_package_preserves_opaque_hkt_alias_and_imported_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-maybe-operations"),
        &[Case::success(
            "box:7\nnone\nempty\nbox:9\nbox:9\n雪🙂\n".as_bytes(),
        )],
    );
}
#[test]
fn basic_standard_returned_application_original_regression_matches_strict_ts_and_native() {
    compare(
        &root().join(
            "examples/spec/fixtures/rust-backend-regressions/standard-returned-application.ssrg",
        ),
        &[Case::success(b"42\n43\n")],
    );
}

#[test]
fn basic_standard_conditional_maybe_callback_original_regression_matches_strict_ts_and_native() {
    compare(
        &root().join(
            "examples/spec/fixtures/rust-backend-regressions/conditional-maybe-callback.ssrg",
        ),
        &[Case::success(b"none\n")],
    );
}

#[test]
fn basic_reduce_until_keeps_early_stop_empty_saved_generic_and_function_accumulators() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-reduce-until.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-reduce-until.stdout"
        ))],
    );
}
#[test]
fn basic_reduce_until_keeps_cold_captured_callbacks_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-reduce-until-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-reduce-until-effects.stdout"
        ))],
    );
}
#[test]
fn basic_reduce_until_source_iterate_next_and_both_callback_defects_release_every_root() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-reduce-until-effects.ssrg"
    );
    let needle = "control.reduceUntil 0 add [1,2,3]";
    assert_eq!(original.matches(needle).count(), 1);
    for expression in [
        "control.reduceUntil (42 / 0) add [1,2,3]",
        "control.reduceUntil 0 (callback (42 / 0)) [1,2,3]",
        "control.reduceUntil 0 add (if 42 / 0 == 0 then [1] else [2])",
        "control.reduceUntil 0 add (Broken ())",
        "control.reduceUntil 0 add (iterator.unfold (\\state:Int -> Just (42 / 0,state + 1)) 0)",
        "control.reduceUntil 0 (\\total:Int -> \\value:Int -> control.Next (total + value / 0)) [1,2,3]",
        "control.reduceUntil 0 brokenCallback [1,2,3]",
    ] {
        let scratch=Scratch::new();let path=scratch.0.join("reduce-until-defect.ssrg");fs::write(&path,original.replace(needle,expression)).unwrap();
        compare(&path,&[Case {stdout:b"created\nacquired\ncold\n141\ncold\n141\nreleased\n",exit:70,defect:true,native_defect:"Int division by zero",..Case::success(b"")}]);
    }
}
#[test]
fn basic_reduce_until_path_package_keeps_imported_opaque_source_accumulator_and_control_aliases() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-reduce-until"),
        &[Case::success("3\n3\n13\ndone:雪\n".as_bytes())],
    );
}

#[test]
fn basic_iterable_build_keeps_custom_standard_saved_sources_order_duplicates_and_empty_inputs() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-iterable-build.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-iterable-build.stdout"
        ))],
    );
}
#[test]
fn basic_iterable_build_keeps_cold_captured_dictionaries_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-iterable-build-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-iterable-build-effects.stdout"
        ))],
    );
}
#[test]
fn basic_iterable_build_iterate_step_hash_eq_and_source_defects_release_every_root() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-iterable-build-effects.ssrg"
    );
    let needle = "arrays.fromIterable (iterator.unfold advance 0)";
    assert_eq!(original.matches(needle).count(), 1);
    for expression in [
        "arrays.fromIterable (iterator.unfold (\\state:Int -> if state == 1 then Just (42 / 0,state + 1) else Just (state,state + 1)) 0)",
        "lists.fromIterable (Broken ())",
        "sets.fromIterable [BadHash 1]",
        "maps.fromEntries [(BadEq 1,\"first\"),(BadEq 2,\"second\")]",
        "lists.fromIterable (if 42 / 0 == 0 then [1] else [2])",
    ] {
        let scratch=Scratch::new();let path=scratch.0.join("iterable-build-defect.ssrg");fs::write(&path,original.replace(needle,expression)).unwrap();
        compare(&path,&[Case {stdout:b"created\nacquired\ncold\n[1, 2, 1]|`[1, 2, 1]|[1, 2]|[(1, new), (2, second)]\ncold\n[1, 2, 1]|`[1, 2, 1]|[1, 2]|[(1, new), (2, second)]\nreleased\n",exit:70,defect:true,native_defect:"Int division by zero",..Case::success(b"")}]);
    }
}
#[test]
fn basic_iterable_build_path_package_keeps_imported_opaque_sources_keys_and_generic_factories() {
    compare(&root().join("examples/spec/fixtures/rust-backend-projects/basic-iterable-build"),&[Case::success(b"[3, 1, 3, 2]\n`[3, 1, 3, 2]\n[3, 1, 2]\n[(key:1, new), (key:2, second)]\n[(key:1, new), (key:2, second)]\n[3, 1, 3, 2]\n")]);
}

#[test]
fn basic_iterator_original_minimal_saved_runtime_regression_matches_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-regressions/iterator-saved-runtime.ssrg"),
        &[Case::success(b"0\n")],
    );
}

#[test]
fn basic_iterator_keeps_lazy_persistent_replay_tails_generic_saved_and_returned_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-iterator.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-iterator.stdout"
        ))],
    );
}
#[test]
fn basic_iterator_keeps_cold_captured_state_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-iterator-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-iterator-effects.stdout"
        ))],
    );
}
#[test]
fn basic_iterator_step_source_arguments_and_first_observation_defects_release_every_root() {
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-iterator-effects.ssrg");
    let needle = "iterator.next (iterator.unfold (advance 42) 1)";
    assert_eq!(original.matches(needle).count(), 1);
    for expression in [
        "iterator.next (iterator.unfold (\\state:Int -> Just (42 / 0,state + 1)) 0)",
        "iterator.unfold (advance (42 / 0)) 0",
        "iterator.unfold (advance 42) (42 / 0)",
    ] {
        let scratch = Scratch::new();
        let path = scratch.0.join("iterator-defect.ssrg");
        fs::write(&path, original.replace(needle, expression)).unwrap();
        compare(
            &path,
            &[Case {
                stdout: b"created\nacquired\ncold\n42:43\ncold\n42:43\nreleased\n",
                exit: 70,
                defect: true,
                native_defect: "Int division by zero",
                ..Case::success(b"")
            }],
        );
    }
    let needle = "refs.make (iterator.unfold (advance 42) 0)";
    assert_eq!(original.matches(needle).count(), 1);
    let scratch = Scratch::new();
    let path = scratch.0.join("iterator-lazy-defect.ssrg");
    fs::write(
        &path,
        original.replace(
            needle,
            "refs.make (iterator.unfold (\\state:Int -> Just (42 / 0,state + 1)) 0)",
        ),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}
#[test]
fn basic_iterator_path_package_keeps_opaque_state_aliases_and_generic_factories() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-iterator"),
        &[Case::success(b"cell:0\ncell:0\ncell:0:cell:1\nend\n")],
    );
}

#[test]
fn basic_sequence_order_keeps_stable_ties_selected_dictionaries_saved_and_empty_inputs() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-sequence-order.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-sequence-order.stdout"
        ))],
    );
}
#[test]
fn basic_sequence_order_keeps_cold_captures_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-sequence-order-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-sequence-order-effects.stdout"
        ))],
    );
}
#[test]
fn basic_sequence_order_key_compare_hash_eq_and_final_argument_defects_release_every_root() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-sequence-order-effects.ssrg"
    );
    let needle = "arrays.sortBy (\\value:Int -> value % 10) [22,11,42,31]";
    assert_eq!(original.matches(needle).count(), 1);
    for expression in [
        "arrays.sortBy (\\value:Int -> value / 0) [1,2,3]",
        "lists.groupBy (\\value:Int -> value / 0) `[1,2,3]",
        "arrays.sort [Broken 1,Broken 2]",
        "arrays.groupBy (\\value:Int -> BadHash value) [1,2]",
        "lists.groupBy (\\value:Int -> BadEq value) `[1,2]",
        "lists.sortBy (\\value:Int -> value) (if 42 / 0 == 0 then `[1] else `[2])",
    ] {
        let scratch = Scratch::new();
        let path = scratch.0.join("sequence-order-defect.ssrg");
        fs::write(&path, original.replace(needle, expression)).unwrap();
        compare(&path,&[Case {stdout:b"created\nacquired\ncold\n[11, 31, 22, 42]|[(0, `[22, 42]), (1, `[11, 31])]\ncold\n[11, 31, 22, 42]|[(0, `[22, 42]), (1, `[11, 31])]\nreleased\n",exit:70,defect:true,native_defect:"Int division by zero",..Case::success(b"")}]);
    }
}
#[test]
fn basic_sequence_order_path_package_keeps_opaque_keys_aliases_and_generic_factories() {
    compare(&root().join("examples/spec/fixtures/rust-backend-projects/basic-sequence-order"),&[Case::success(b"`[cell:3, cell:2, cell:1, cell:1]\n[22, 42, 11, 31]\n[(key:1, `[cell:1, cell:3, cell:1]), (key:0, `[cell:2])]\n`[cell:1, cell:3, cell:2, cell:1]\n")]);
}

#[test]
fn basic_list_transform_keeps_order_output_types_empty_inputs_and_saved_callbacks() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-list-transform.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-list-transform.stdout"
        ))],
    );
}
#[test]
fn basic_list_transform_keeps_cold_captures_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-list-transform-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-list-transform-effects.stdout"
        ))],
    );
}
#[test]
fn basic_list_transform_callback_and_final_argument_defects_release_every_root() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-list-transform-effects.ssrg"
    );
    let needle = "lists.flatMap (\\value:Int -> `[value,value + 1]) `[1,2,3]";
    assert_eq!(original.matches(needle).count(), 1);
    for expression in [
        "lists.flatMap (\\value:Int -> if value == 2 then `[42 / 0] else `[value]) `[1,2,3]",
        "lists.filterMap (\\value:Int -> if value == 2 then Just (42 / 0) else Just value) `[1,2,3]",
        "lists.flatMap (\\value:Int -> `[value]) (if 42 / 0 == 0 then `[1] else `[2])",
    ] {
        let scratch = Scratch::new();
        let path = scratch.0.join("list-transform-defect.ssrg");
        fs::write(&path, original.replace(needle, expression)).unwrap();
        compare(&path, &[Case {
            stdout: b"created\nacquired\ncold\n`[129, -129, 131, -131]\ncold\n`[129, -129, 131, -131]\nreleased\n",
            exit: 70, defect: true, native_defect: "Int division by zero",
            ..Case::success(b"")
        }]);
    }
}
#[test]
fn basic_list_transform_path_package_keeps_opaque_elements_aliases_and_generic_factories() {
    compare(&root().join("examples/spec/fixtures/rust-backend-projects/basic-list-transform"), &[Case::success(b"`[cell:1, cell:10, cell:3, cell:30]\n`[saved:1, saved:2, saved:3]\n`[cell:1, cell:2, cell:3]\n`[]\n")]);
}

#[test]
fn basic_array_rest_keeps_constructors_order_callbacks_and_immutable_inputs() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array-rest.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array-rest.stdout"
        ))],
    );
}
#[test]
fn basic_array_rest_keeps_cold_captures_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array-rest-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array-rest-effects.stdout"
        ))],
    );
}
#[test]
fn basic_array_rest_callback_and_final_argument_defects_release_every_root() {
    for expression in [
        "arrays.flatMap (\\value:Int -> if value == 2 then [42 / 0] else [value]) [1,2,3]",
        "arrays.filterMap (\\value:Int -> if value == 2 then Just (42 / 0) else Just value) [1,2,3]",
        "arrays.flatMap (\\value:Int -> [value]) (if 42 / 0 == 0 then [1] else [2])",
    ] {
        let scratch=Scratch::new();let source=include_str!("../../../examples/spec/fixtures/rust-backend/basic-array-rest-effects.ssrg").replace("arrays.flatMap (\\value:Int -> [value,value + 1]) [1,2,3]",expression);
        let path=scratch.0.join("array-rest-defect.ssrg");fs::write(&path,source).unwrap();
        compare(&path,&[Case {stdout:b"created\nacquired\ncold\n[129, -129, 131, -131, 99]\ncold\n[129, -129, 131, -131, 99]\nreleased\n",exit:70,defect:true,native_defect:"Int division by zero",..Case::success(b"")}]);
    }
}
#[test]
fn basic_array_rest_path_package_keeps_opaque_elements_aliases_and_generic_factories() {
    compare(&root().join("examples/spec/fixtures/rust-backend-projects/basic-array-rest"),&[Case::success(b"[cell:1, cell:3, cell:4]\n[cell:4, cell:3, cell:1]\ncell:1\n[cell:3, cell:4]\nnone\nnone\nTrue\n[cell:1, cell:2, cell:3]\n")]);
}

#[test]
fn basic_array_matches_original_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array.stdout"
        ))],
    );
}

#[test]
fn basic_big_int_matches_original_raw_strict_ts_and_native_without_precision_loss() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int.stdout"
        ))],
    );
}

#[test]
fn basic_big_int_keeps_saved_exact_values_cold_and_releases_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-effects.stdout"
        ))],
    );
}

#[test]
fn basic_big_int_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("big-int-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(&path,&[Case{name:"defect",stdout:b"created\nacquired\ncold\n900719925474099200000000000000001\ncold\n900719925474099200000000000000001\nreleased\n",exit:70,defect:true,..Case::success(b"")}]);
}

#[test]
fn basic_big_int_path_package_preserves_opaque_values_and_captured_formatters() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-big-int"),
        &[Case::success(
            b"900719925474099200000000000001|900719925474099200000000000001|-9007199254740991\n",
        )],
    );
}

#[test]
fn basic_array_infix_descending_comparisons_match_original_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array-ordering.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array-ordering.stdout"
        ))],
    );
}

#[test]
fn basic_sequence_array_saved_and_generic_operations_match_original_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array-sequence.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array-sequence.stdout"
        ))],
    );
}

#[test]
fn basic_concat_keeps_order_empty_inputs_generic_saved_functions_and_original_strict_ts() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-sequence-concat.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-sequence-concat.stdout"
        ))],
    );
}

#[test]
fn basic_set_operations_keep_order_representatives_snapshots_and_generic_saved_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-set-operations.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-set-operations.stdout"
        ))],
    );
}

#[test]
fn basic_map_operations_keep_key_representatives_order_snapshots_and_changed_value_types() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-map-operations.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-map-operations.stdout"
        ))],
    );
}

#[test]
fn basic_map_upsert_keeps_representatives_order_snapshots_and_noncommutative_resolvers() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-map-upsert.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-map-upsert.stdout"
        ))],
    );
}

#[test]
fn basic_set_map_keeps_first_output_representatives_order_snapshots_and_output_key_types() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-set-map.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-set-map.stdout"
        ))],
    );
}

#[test]
fn basic_set_map_path_package_keeps_opaque_keys_and_saved_imported_generic_stages() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-set-map"),
        &[Case::success(b"1|2|1|2|True|True\n")],
    );
}

#[test]
fn basic_set_map_keeps_returned_callbacks_cold_and_releases_ref_cycles_after_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-set-map-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-set-map-effects.stdout"
        ))],
    );
}

#[test]
fn basic_set_map_mapper_and_returned_callback_defects_release_partial_results_and_run_finalizers() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-set-map-effects.ssrg");
    for (name, old, replacement) in [
        ("mapper", "sets.map (item callback) source", "sets.map (\\value: Int -> if value == 3 then item callback (42 / 0) else item callback value) source"),
        ("returned", "effects.succeed 42", "effects.succeed (42 / 0)"),
    ] {
        assert_eq!(original.matches(old).count(), 1);
        let path = scratch.0.join(format!("set-map-{name}-defect.ssrg"));
        fs::write(&path, original.replace(old, replacement)).unwrap();
        compare(&path, &[Case { name, stdout: b"created\nacquired\ncold\nreleased\n", exit: 70, defect: true, native_defect: "Int division by zero", ..Case::success(b"") }]);
    }
}

#[test]
fn basic_map_upsert_path_package_keeps_opaque_input_output_keys_and_saved_generic_stages() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-map-upsert"),
        &[Case::success(b"1|2|[120, 3]|[121, 3]|[121, 34]|[120, 3]\n")],
    );
}

#[test]
fn basic_map_upsert_keeps_returned_functions_cold_and_releases_ref_cycles_after_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-map-upsert-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-map-upsert-effects.stdout"
        ))],
    );
}

#[test]
fn basic_map_upsert_callback_defects_run_finalizers_and_release_partial_results() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-map-upsert-effects.ssrg");
    for (name, old, replacement) in [
        ("update", "maps.upsert \"callback\" (keep callback) stored", "maps.upsert \"callback\" (\\_current -> if (42 / 0) == 0 then callback else callback) stored"),
        ("merge", "maps.mergeWith resolve right updated", "maps.mergeWith (\\_left -> \\_right -> if (42 / 0) == 0 then callback else callback) right updated"),
        ("key", "maps.mapKeysWith resolve (\\_key: String -> 1) merged", "maps.mapKeysWith resolve (\\_key: String -> 1 / 0) merged"),
        ("collision", "maps.mapKeysWith resolve (\\_key: String -> 1) merged", "maps.mapKeysWith (\\_left -> \\_right -> if (42 / 0) == 0 then callback else callback) (\\_key: String -> 1) merged"),
    ] {
        assert_eq!(original.matches(old).count(), 1);
        let path = scratch.0.join(format!("map-upsert-{name}-defect.ssrg"));
        fs::write(&path, original.replace(old, replacement)).unwrap();
        compare(&path, &[Case { name, stdout: b"created\nacquired\ncold\nreleased\n", exit: 70, defect: true, native_defect: "Int division by zero", ..Case::success(b"") }]);
    }
}

#[test]
fn basic_map_operations_path_package_keeps_opaque_values_imported_generic_partials_and_captures() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-map-operations"),
        &[Case::success(b"[40]|[(first, 40), (second, 2)]|42|42\n")],
    );
}

#[test]
fn basic_map_operations_keep_cold_replay_and_release_callback_ref_cycles_after_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-map-operation-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-map-operation-effects.stdout"
        ))],
    );
}

#[test]
fn basic_map_operations_callback_defects_unwind_partial_results_and_run_finalizers() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-map-operation-effects.ssrg"
    );
    for (name, old, replacement) in [
        (
            "predicate",
            "value > 0",
            "if value == 21 then value / 0 > 0 else value > 0",
        ),
        (
            "mapper",
            "value + 1",
            "if value == 21 then value / 0 else value + 1",
        ),
    ] {
        assert_eq!(original.matches(old).count(), 1);
        let path = scratch.0.join(format!("map-{name}-defect.ssrg"));
        fs::write(&path, original.replace(old, replacement)).unwrap();
        compare(
            &path,
            &[Case {
                name,
                stdout: b"created\nacquired\ncold\nreleased\n",
                exit: 70,
                defect: true,
                native_defect: "Int division by zero",
                ..Case::success(b"")
            }],
        );
    }
}

#[test]
fn basic_set_operations_path_package_keeps_opaque_keys_and_imported_selected_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-set-operations"),
        &[Case::success(b"40|2|3|2|True|3\n")],
    );
}

#[test]
fn basic_set_operations_keep_cold_replay_and_release_ref_cycles_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-set-operation-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-set-operation-effects.stdout"
        ))],
    );
}

#[test]
fn basic_set_operations_predicate_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-set-operation-effects.ssrg"
    );
    assert_eq!(original.matches("detail.identity == 2").count(), 1);
    let path = scratch.0.join("set-predicate-defect.ssrg");
    fs::write(
        &path,
        original.replace("detail.identity == 2", "detail.identity / 0 == 2"),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "predicate-defect",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn inferred_nested_empty_collection_keeps_original_strict_ts_bun_node_and_native() {
    compare(
        &root().join(
            "examples/spec/fixtures/rust-backend-regressions/inferred-nested-empty-collection.ssrg",
        ),
        &[Case::success(
            b"[[], [\xe9\x9b\xaa, \xf0\x9f\x99\x82], [e\xcc\x81], []]\n",
        )],
    );
    compare(
        &root().join("examples/spec/fixtures/rust-backend-regressions/inferred-empty-concat.ssrg"),
        &[Case::success(
            b"[\xe9\x9b\xaa, \xf0\x9f\x99\x82, e\xcc\x81]\n",
        )],
    );
}

#[test]
fn basic_concat_path_package_keeps_opaque_elements_imported_generic_functions_and_captures() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-concat"),
        &[Case::success(b"40|2|41|3|42\n")],
    );
}

#[test]
fn basic_concat_keeps_callbacks_cold_and_releases_ref_cycles_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-concat-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-concat-effects.stdout"
        ))],
    );
}

#[test]
fn basic_concat_returned_callback_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-concat-effects.ssrg");
    assert_eq!(original.matches("effects.succeed 42").count(), 1);
    let path = scratch.0.join("concat-callback-defect.ssrg");
    fs::write(
        &path,
        original.replace("effects.succeed 42", "effects.succeed (42 / 0)"),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "callback-defect",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_sequence_list_saved_and_generic_operations_match_original_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-list-sequence.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-list-sequence.stdout"
        ))],
    );
}

#[test]
fn basic_sequence_keeps_saved_callbacks_cold_and_releases_ref_cycles_after_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-sequence-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-sequence-effects.stdout"
        ))],
    );
}

#[test]
fn basic_sequence_callback_defect_runs_original_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-sequence-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("sequence-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n42\ncold\n42\n84\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_sequence_path_package_keeps_generic_saved_functions_and_captured_elements() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-sequence"),
        &[Case::success(
            b"[(1, True), (2, False)]|`[`[1, 2], `[2, 3]]|1|42|2\n",
        )],
    );
}

#[test]
fn basic_sequence_fault_inside_fold_callback_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-sequence-effects.ssrg");
    assert_eq!(source.matches("effects.succeed 42").count(), 1);
    let path = scratch.0.join("sequence-fold-callback-defect.ssrg");
    fs::write(&path,source.replace("effects.succeed 42","effects.succeed (arrays.reduceRight 0 (\\value: Int -> \\acc: Int -> value / acc) [1])")).unwrap();
    compare(
        &path,
        &[Case {
            name: "callback-defect",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_array_releases_saved_readers_after_cold_replay_and_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array-effects.stdout"
        ))],
    );
}

#[test]
fn basic_array_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-array-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("array-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n128\ncold\n128\n256\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_array_path_package_preserves_generic_stages_and_captured_callbacks() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-array"),
        &[Case::success(b"[2, 2]:3:4:42\n")],
    );
}

#[test]
fn basic_bytes_path_package_keeps_canonical_buffers_and_saved_readers() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-bytes"),
        &[Case::success(b"[0, 128, 255]:0:128:255:-1\nbad:256\n")],
    );
}

#[test]
fn basic_bytes_release_saved_buffers_after_cold_replay_and_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-bytes-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-bytes-effects.stdout"
        ))],
    );
}

#[test]
fn basic_bytes_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-bytes-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("bytes-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n128\ncold\n128\n510\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_standard_calls_remain_cold_and_release_saved_functions_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-text-math-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-math-effects.stdout"
        ))],
    );
}

#[test]
fn basic_standard_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-text-math-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout:
                b"created\nacquired\ncold\n9.0|16.0\nTrue\ncold\n9.0|16.0\nTrue\n8.0\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_hash_keeps_exact_public_values_and_selected_generic_and_saved_methods() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-hash.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-hash.stdout"
        ))],
    );
}

#[test]
fn basic_map_preserves_snapshots_collision_keys_and_generic_saved_readers() {
    let output = include_bytes!("../../../examples/spec/fixtures/rust-backend/basic-map.stdout");
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-map.ssrg"),
        &[
            Case::success(output),
            Case {
                name: "minimum-seed",
                hash_seed: Some("-9223372036854775808"),
                ..Case::success(output)
            },
            Case {
                name: "maximum-seed",
                hash_seed: Some("9223372036854775807"),
                ..Case::success(output)
            },
            Case {
                name: "zero-seed",
                hash_seed: Some("+000"),
                ..Case::success(output)
            },
        ],
    );
}

#[test]
fn basic_map_keeps_saved_snapshots_cold_and_releases_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-map-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-map-effects.stdout"
        ))],
    );
}

#[test]
fn basic_map_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-map-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("map-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n42\ncold\n42\n84\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_map_path_package_preserves_generic_evidence_and_saved_callbacks() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-map"),
        &[Case::success(b"1:3:[wave, snow]:2:42\n")],
    );
}

#[test]
fn basic_collections_set_preserves_order_representatives_snapshots_and_saved_functions() {
    let output = include_bytes!("../../../examples/spec/fixtures/rust-backend/basic-set.stdout");
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-set.ssrg"),
        &[
            Case::success(output),
            Case {
                name: "minimum-seed",
                hash_seed: Some("-9223372036854775808"),
                ..Case::success(output)
            },
            Case {
                name: "maximum-seed",
                hash_seed: Some("9223372036854775807"),
                ..Case::success(output)
            },
            Case {
                name: "zero-seed",
                hash_seed: Some("+000"),
                ..Case::success(output)
            },
        ],
    );
}

#[test]
fn basic_collections_map_contains_and_entries_preserve_order_snapshots_and_generic_evidence() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-map-entries.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-map-entries.stdout"
        ))],
    );
}

#[test]
fn basic_collections_keep_cold_callbacks_and_release_reference_cycles_after_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-collections-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-collections-effects.stdout"
        ))],
    );
}

#[test]
fn basic_collections_defect_runs_the_finalizer_and_releases_reference_cycles() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-collections-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("collections-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n42|42\ncold\n42|42\n84\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_collections_path_package_keeps_generic_partial_and_saved_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-collections"),
        &[Case::success(
            "[雫🙂]:[雫🙂, 🌊]:[🌊, 雫🙂]:True:False:True:42\n".as_bytes(),
        )],
    );
}

#[test]
fn basic_decimal_keeps_exact_digits_exponents_errors_and_saved_operations() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal.stdout"
        ))],
    );
}

#[test]
fn basic_int_radix_keeps_boundaries_errors_and_saved_partial_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-int-radix.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-int-radix.stdout"
        ))],
    );
}

#[test]
fn basic_int_radix_keeps_cold_partial_captures_and_typed_failure_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-int-radix-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-int-radix-effects.stdout"
        ))],
    );
}

#[test]
fn basic_int_radix_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-int-radix-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("int-radix-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n42\ncold\n42\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_int_radix_path_package_keeps_opaque_values_and_imported_partial_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-int-radix"),
        &[Case::success(
            b"16\n-9007199254740991\n2gosa7pa2gv\nInvalidIntRadix 1\n",
        )],
    );
}

#[test]
fn basic_int_checked_keeps_exact_arithmetic_errors_and_saved_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-int-checked.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-int-checked.stdout"
        ))],
    );
}

#[test]
fn basic_int_checked_keeps_cold_captured_functions_and_typed_failure_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-int-checked-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-int-checked-effects.stdout"
        ))],
    );
}

#[test]
fn basic_int_checked_clamp_defect_keeps_partial_stages_finalizer_and_root_zero() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-int-clamp-defect.ssrg"),
        &[Case {
            name: "clamp-defect",
            stdout: b"created\nacquired\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Seseragi Int clamp lower bound exceeds upper bound",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_int_checked_path_package_keeps_opaque_values_and_imported_error_owners() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-int-checked"),
        &[Case::success(b"overflow\n42\npower overflow\n")],
    );
}

#[test]
fn basic_math_remaining_keeps_complete_surface_and_original_math_contract() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-math-remaining.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-math-remaining.stdout"
        ))],
    );
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-text-math.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-math.stdout"
        ))],
    );
}

#[test]
fn basic_math_remaining_keeps_cold_partial_captures_and_typed_failure_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-math-remaining-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-math-remaining-effects.stdout"
        ))],
    );
}

#[test]
fn basic_math_remaining_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-math-remaining-effects.ssrg"
    );
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("math-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n5.0\ncold\n5.0\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_math_remaining_path_package_keeps_opaque_values_and_saved_binary_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-math-remaining"),
        &[Case::success(b"5.0\n-0.0\n6.283185307179586\nTrue\n")],
    );
}

#[test]
fn basic_big_radix_keeps_exact_values_errors_and_saved_partial_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-radix.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-radix.stdout"
        ))],
    );
}

#[test]
fn basic_big_radix_keeps_cold_captured_values_and_typed_failure_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-radix-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-radix-effects.stdout"
        ))],
    );
}

#[test]
fn basic_big_radix_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-big-int-radix-effects.ssrg"
    );
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("big-radix-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(&path,&[Case {name:"defect",stdout:b"created\nacquired\ncold\nffffffffffffffffffffffffffffffff\ncold\nffffffffffffffffffffffffffffffff\nreleased\n",exit:70,defect:true,..Case::success(b"") }]);
}

#[test]
fn basic_big_radix_path_package_keeps_opaque_values_and_imported_error_owners() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-big-int-radix"),
        &[Case::success(
            b"f5lxx1zz5pnorynqglhzmsp33\n9007199254740991\noutside\n",
        )],
    );
}

#[test]
fn basic_big_keys_keeps_standard_dictionaries_collision_and_original_radix_contract() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-keys.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-keys.stdout"
        ))],
    );
}

#[test]
fn basic_big_keys_keeps_cold_partial_captures_and_typed_failure_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-key-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-key-effects.stdout"
        ))],
    );
}

#[test]
fn basic_big_keys_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-key-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("big-key-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\nLess:1236334116\ncold\nLess:1236334116\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_big_keys_equality_operators_preserve_exact_values_saved_stages_and_hash_collisions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-equality.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-equality.stdout"
        ))],
    );
}

#[test]
fn basic_big_arithmetic_keeps_exact_operations_constants_and_checked_errors() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-arithmetic.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-arithmetic.stdout"
        ))],
    );
}

#[test]
fn basic_big_arithmetic_keeps_cold_captures_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-big-int-arithmetic-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-arithmetic-effects.stdout"
        ))],
    );
}

#[test]
fn basic_big_arithmetic_defects_keep_each_final_argument_and_release_every_root() {
    for (expression, message) in [
        ("big.fromInt 7 / big.fromInt 0", "BigInt division by zero"),
        ("big.fromInt 7 % big.fromInt 0", "BigInt remainder by zero"),
        ("big.fromInt 7 ** (-1)", "BigInt negative exponent"),
    ] {
        let scratch = Scratch::new();
        let source = include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-big-int-arithmetic-effects.ssrg"
        )
        .replace(
            "big.checkedDivide (big.fromInt 0) (big.fromInt 7)",
            &format!("big.checkedDivide ({expression}) (big.fromInt 7)"),
        );
        let path = scratch.0.join("big-arithmetic-defect.ssrg");
        fs::write(&path, source).unwrap();
        compare(&path, &[Case {
            stdout: b"created\nacquired\ncold\n4294967296:18446744073709551616\ncold\n4294967296:18446744073709551616\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: message,
            ..Case::success(b"")
        }]);
    }
}

#[test]
fn basic_big_arithmetic_path_package_keeps_opaque_generic_stages_and_error_owners() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-big-int-arithmetic"),
        &[Case::success(b"key:4638558931|key:1475714835504168960|key:-680564733841876926926749214863536422912|key:0|key:1|key:343591635|zero\n")],
    );
}

#[test]
fn basic_decimal_arithmetic_keeps_exact_signed_operations_and_saved_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-arithmetic.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-arithmetic.stdout"
        ))],
    );
}
#[test]
fn basic_decimal_arithmetic_keeps_cold_captures_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-arithmetic-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-arithmetic-effects.stdout"
        ))],
    );
}
#[test]
fn basic_decimal_arithmetic_defects_preserve_final_argument_and_resource_failure_cleanup() {
    for (expression, message) in [
        (
            "add (decimal.fromInt 1) (decimal.fromInt (42 / 0))",
            "Int division by zero",
        ),
        (
            "add (value \"1e100000000000000000000000\") (decimal.fromInt 2)",
            "Decimal canonical spelling exceeds host limits",
        ),
    ] {
        let scratch = Scratch::new();
        let source = include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-arithmetic-effects.ssrg"
        )
        .replace("add (decimal.fromInt 1) (decimal.fromInt 2)", expression);
        let path = scratch.0.join("decimal-arithmetic-defect.ssrg");
        fs::write(&path, source).unwrap();
        compare(
            &path,
            &[Case {
                stdout: b"created\nacquired\ncold\n2.5:1.5625\ncold\n2.5:1.5625\nreleased\n",
                exit: 70,
                defect: true,
                native_defect: message,
                ..Case::success(b"")
            }],
        );
    }
}
#[test]
fn basic_decimal_arithmetic_path_package_keeps_opaque_values_and_imported_generic_methods() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-decimal-arithmetic"),
        &[Case::success(
            b"cell:-1.25|cell:3.75|cell:-3.125\n0:1:cell:1.25\n",
        )],
    );
}

#[test]
fn basic_decimal_division_keeps_exact_and_rounded_results_and_quantize_modes() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-division.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-division.stdout"
        ))],
    );
}
#[test]
fn basic_decimal_division_keeps_cold_partial_factories_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-division-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-division-effects.stdout"
        ))],
    );
}
#[test]
fn basic_decimal_division_defects_keep_source_stages_and_resource_failure_cleanup() {
    for (expression,message) in [
        ("decimal.divide (chosen (42 / 0)) (decimal.fromInt 0) (decimal.fromInt 7)", "Int division by zero"),
        ("decimal.divide (chosen 4) (decimal.fromInt (42 / 0)) (decimal.fromInt 7)", "Int division by zero"),
        ("decimal.divide (chosen 4) (decimal.fromInt 0) (decimal.fromInt (42 / 0))", "Int division by zero"),
        ("decimal.divideExact (decimal.fromInt 2) (decimal.quantize 1 numbers.HalfEven (decimal.fromInt (42 / 0)))", "Int division by zero"),
        ("decimal.divide (chosen 9007199254740991) (value \"1000000000000000000001\") (decimal.fromInt 1)", "Decimal canonical spelling exceeds host limits"),
    ] {
        let scratch=Scratch::new();
        let source=include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-division-effects.ssrg").replace("decimal.divide (chosen 4) (decimal.fromInt 0) (decimal.fromInt 7)",expression);
        let path=scratch.0.join("decimal-division-defect.ssrg");fs::write(&path, source).unwrap();
        compare(&path,&[Case {
            stdout:b"created\nacquired\ncold\n0.4167:1.2:15.625\ncold\n0.4167:1.2:15.625\nreleased\n",
            exit:70,defect:true,native_defect:message,..Case::success(b"")
        }]);
    }
}
#[test]
fn basic_decimal_division_path_package_keeps_opaque_values_aliases_and_returned_functions() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-decimal-division"),
        &[Case::success(
            b"cell:15.625|cell:0.4167|cell:1.2\nzero\nnon-terminating\n",
        )],
    );
}

#[test]
fn basic_decimal_context_keeps_modes_safe_int_precision_and_saved_getters() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-context.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-context.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_context_keeps_cold_captures_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-context-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-context-effects.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_context_checks_its_final_rounding_argument_before_nonpositive_failure() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-decimal-context-effects.ssrg"
    )
    .replace(
        "decimal.context 0 numbers.Ceiling",
        "decimal.context 0 (if 42 / 0 == 0 then numbers.Ceiling else numbers.HalfEven)",
    );
    let path = scratch.0.join("decimal-context-defect.ssrg");
    fs::write(&path, source).unwrap();
    compare(
        &path,
        &[Case {
            stdout: b"created\nacquired\ncold\n34\ncold\n34\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_decimal_context_path_package_keeps_opaque_values_and_canonical_mode_error_aliases() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-decimal-context"),
        &[Case::success(b"34:Ceiling:0\n")],
    );
}

#[test]
fn basic_decimal_conversion_preserves_integer_float_bounds_and_exact_rounding() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-conversion.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-conversion.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_conversion_keeps_cold_captures_typed_failure_and_finalizer_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-conversion-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-conversion-effects.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_conversion_evaluates_the_final_float_argument_and_releases_every_root() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-decimal-conversion-effects.ssrg"
    )
    .replace(
        "decimal.fromFloat context (1.0 / 0.0)",
        "decimal.fromFloat context (if 42 / 0 == 0 then 0.1 else 0.2)",
    );
    let path = scratch.0.join("decimal-conversion-defect.ssrg");
    fs::write(&path, source).unwrap();
    compare(&path, &[Case {
        stdout: b"created\nacquired\ncold\n0.10000000000000001\ncold\n0.10000000000000001\nreleased\n",
        exit: 70,
        defect: true,
        native_defect: "Int division by zero",
        ..Case::success(b"")
    }]);
}

#[test]
fn basic_decimal_conversion_path_package_preserves_opaque_values_and_error_aliases() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-decimal-conversion"),
        &[Case::success(
            b"cell:0.10000000000000001|0.1|not-integral\ncell:2|2.0|2\nnot-finite\n",
        )],
    );
}

#[test]
fn basic_big_keys_path_package_keeps_opaque_values_and_imported_generic_methods() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-big-int-keys"),
        &[Case::success(
            b"Less|True|1236334116|1236334116|2|2|updated|second|key:343591635|key:4294967296\n",
        )],
    );
}

#[test]
fn basic_decimal_keeps_captured_values_cold_and_releases_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-effects.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("decimal-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n900719925474099200000000000000001.001\ncold\n900719925474099200000000000000001.001\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_decimal_host_limit_defect_preserves_parse_and_runs_the_finalizer() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-effects.ssrg");
    let original = "900719925474099200000000000000001.001";
    assert_eq!(source.matches(original).count(), 1);
    let path = scratch.0.join("decimal-host-limit.ssrg");
    fs::write(
        &path,
        source.replace(original, "1e99999999999999999999999999999999999999999"),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "host-limit",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Decimal canonical spelling exceeds host limits",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_decimal_path_package_keeps_opaque_values_and_saved_formatters() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-decimal"),
        &[Case::success(
            b"42:900719925474099200000000000000001.001:True:False\n",
        )],
    );
}

#[test]
fn basic_json_decoders_match_original_strict_ts_bun_node_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-decoders.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-decoders.stdout"
        ))],
    );
}
#[test]
fn basic_json_decoders_keep_cold_replay_domain_failure_and_captured_ref_cycle() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-decoder-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-decoder-effects.stdout"
        ))],
    );
}
#[test]
fn basic_json_decoders_defect_after_replay_runs_finalizer_and_releases_roots() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-decoder-effects.ssrg"
    );
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("decoder-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n42\ncold\n42\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}
#[test]
fn basic_json_decoders_fault_inside_mapper_runs_finalizer_and_releases_roots() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-decoder-effects.ssrg"
    );
    let original = "{\\\"n\\\":41}";
    assert_eq!(source.matches(original).count(), 1);
    let path = scratch.0.join("decoder-mapper-defect.ssrg");
    fs::write(&path, source.replace(original, "{\\\"n\\\":0}")).unwrap();
    compare(
        &path,
        &[Case {
            name: "mapper-defect",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}
#[test]
fn basic_json_decoders_path_package_keeps_opaque_rules_and_saved_generic_stages() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-decoders"),
        &[Case::success(b"2:Just 7:Just 9\n")],
    );
}

#[test]
fn inferred_json_decoder_package_keeps_original_input_and_passes_strict_ts() {
    let scratch = Scratch::new();
    let package = scratch.0.join("inferred-json-decoders");
    copy_package(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-decoders"),
        &package,
    );
    let original = include_bytes!(
        "../../../examples/spec/fixtures/rust-backend-regressions/json-decoder-inferred-partial-main.ssrg"
    );
    fs::write(package.join("src/main.ssrg"), original).unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    compare(&package, &[Case::success(b"2:Just 7:Just 9\n")]);
    assert_eq!(fs::read(package.join("src/main.ssrg")).unwrap(), original);
}
#[test]
fn basic_json_decoders_native_host_limit_is_defect_with_finalizer_and_root_cleanup() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-decoder-effects.ssrg"
    );
    let original = "let decoder = select input";
    assert_eq!(source.matches(original).count(), 1);
    let path = scratch.0.join("decoder-host-limit.ssrg");
    fs::write(
        &path,
        source.replace(original, "let decoder = chain 512 (select input)"),
    )
    .unwrap();
    let out = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        &scratch.0,
    ));
    let result = bounded(
        Command::new(out.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(70), "{result:?}");
    assert_eq!(result.stdout, b"created\nacquired\ncold\nreleased\n");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("JSON decoder nesting exceeds host limits")
            && stderr.contains("managed-live=0"),
        "{stderr}"
    );
}

#[test]
fn basic_json_codecs_match_original_strict_ts_bun_node_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-codecs.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-codecs.stdout"
        ))],
    );
}

#[test]
fn basic_json_codecs_keep_cold_replay_domain_failures_finalizers_and_ref_cycles() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-codec-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-codec-effects.stdout"
        ))],
    );
}

#[test]
fn basic_json_codecs_defect_runs_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-json-codec-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("codec-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(&path,&[Case{name:"defect",stdout:"created\nacquired\ncold\n{\"answer\":9007199254740992000000000001.1,\"snow\":\"雪🙂\"}\ncold\n{\"answer\":9007199254740992000000000001.1,\"snow\":\"雪🙂\"}\nreleased\n".as_bytes(),exit:70,defect:true,..Case::success(b"")}]);
}

#[test]
fn basic_json_codecs_package_keeps_opaque_values_and_saved_generic_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-codecs"),
        &[Case::success("42:42:\"雪🙂\":True\n".as_bytes())],
    );
}

#[test]
fn basic_json_containers_preserve_original_strict_ts_bun_node_and_native_results() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-containers.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-containers.stdout"
        ))],
    );
}

#[test]
fn json_record_prototype_field_keeps_the_original_source_and_strict_cross_target_results() {
    compare(
        &root().join(
            "examples/spec/fixtures/rust-backend-regressions/json-record-prototype-field.ssrg",
        ),
        &[Case::success(b"{\"__proto__\":\"snow\"}\n")],
    );
}

#[test]
fn basic_json_derived_structs_keep_original_strict_ts_bun_node_and_native_results() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-derived-structs.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-derived-structs.stdout"
        ))],
    );
}

#[test]
fn basic_json_derived_newtypes_keep_transparent_original_cross_target_results() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-derived-newtypes.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-derived-newtypes.stdout"
        ))],
    );
}

#[test]
fn basic_json_derived_adts_keep_original_strict_ts_bun_node_and_native_results() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-derived-adts.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-derived-adts.stdout"
        ))],
    );
}

#[test]
fn basic_json_derived_adts_keep_the_original_canonical_spec_source() {
    compare(
        &root().join("examples/spec/artifacts/schema-1/derived-json-codecs/main.ssrg"),
        &[Case::success(concat!(
            "{\"name\":\"count\",\"value\":3}\n",
            "{\"name\":\"count\",\"value\":3}\n",
            "7\n",
            "{\"tag\":\"Branch\",\"value\":[{\"tag\":\"Leaf\",\"value\":1},{\"tag\":\"Leaf\",\"value\":2}]}\n",
            "{\"tag\":\"Branch\",\"value\":[{\"tag\":\"Leaf\",\"value\":1},{\"tag\":\"Leaf\",\"value\":2}]}\n",
            "{\"tag\":\"Named\",\"value\":{\"count\":2,\"label\":\"ready\"}}\n",
            "{\"tag\":\"Named\",\"value\":{\"count\":2,\"label\":\"ready\"}}\n"
        ).as_bytes())],
    );
}

#[test]
fn basic_json_derived_adts_package_keeps_generic_opaque_and_saved_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-derived-adts"),
        &[Case::success("{\"tag\":\"Both\",\"value\":[7,[1,null,2]]}:{\"tag\":\"Both\",\"value\":[7,[1,null,2]]}:[{\"tag\":\"Both\",\"value\":[7,\"雪🙂\"]}]\n".as_bytes())],
    );
}

#[test]
fn basic_json_derived_adts_keep_cold_replay_failure_defects_and_captured_ref_cycles() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let old = original
        .lines()
        .find(|line| line.starts_with("  let render = reader<Rows> "))
        .unwrap();
    let literal = old.split_once("reader<Rows> ").unwrap().1;
    let wire: String = serde_json::from_str(literal).unwrap();
    let wrapped = format!("{{\"tag\":\"Wrapped\",\"value\":{wire}}}");
    let replacement = format!(
        "  let render = reader<Wrapped<Rows>> {}",
        serde_json::to_string(&wrapped).unwrap()
    );
    let source = format!(
        "type Wrapped<A> deriving JsonEncode, JsonDecode = | Wrapped A\n{}",
        original.replace(old, &replacement)
    );
    let original_expected = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.stdout"
    );
    let expected = original_expected
        .lines()
        .map(|line| {
            if line.starts_with('{') {
                format!("{{\"tag\":\"Wrapped\",\"value\":{line}}}\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect::<String>();
    let prefix = expected.find("recovered: typed").unwrap();
    for (name, changed, output, defect, message) in [
        (
            "cold-replay",
            source.clone(),
            expected.as_bytes(),
            false,
            "",
        ),
        (
            "body-defect",
            source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
            &expected.as_bytes()[..prefix],
            true,
            "deliberate",
        ),
        (
            "encoder-defect",
            source.replace(
                "9007199254740992000000000001.100",
                "1e99999999999999999999999999999999999999999",
            ),
            b"created\nacquired\ncold\nreleased\n".as_slice(),
            true,
            "Decimal canonical spelling exceeds host limits",
        ),
    ] {
        let path = scratch.0.join(format!("adt-{name}.ssrg"));
        fs::write(&path, changed).unwrap();
        compare(
            &path,
            &[Case {
                name,
                stdout: output,
                exit: if defect { 70 } else { 0 },
                defect,
                native_defect: message,
                ..Case::success(b"")
            }],
        );
    }
}

#[test]
fn basic_json_derived_adts_recursive_encode_and_decode_unwind_the_host_limit() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let old = original
        .lines()
        .find(|line| line.starts_with("  let render = "))
        .unwrap();
    let declaration="type Deep deriving JsonEncode, JsonDecode = | End | Next Deep\nfn deep n: Int -> Deep = if n <= 0 then End else Next (deep (n - 1))\nfn wire n: Int -> String = if n <= 0 then \"{\\\"tag\\\":\\\"End\\\"}\" else \"{\\\"tag\\\":\\\"Next\\\",\\\"value\\\":\" + wire (n - 1) + \"}\"\n";
    for (name,render) in [
        ("encode","  let render = \\_: Unit -> json.encodeString (deep 512)"),
        ("decode","  let render = \\_: Unit -> match json.decodeString<Deep> (wire 512) { Left _ -> \"domain\"; Right value -> json.encodeString value }"),
    ] {
        let path=scratch.0.join(format!("adt-host-limit-{name}.ssrg"));fs::write(&path,format!("{declaration}{}",original.replace(old,render))).unwrap();
        let out=scratch.0.join(format!("native-{name}"));
        successful(bounded(Command::new(env!("CARGO_BIN_EXE_seseragi")).args(["experimental","build-native"]).arg(&path).args(["--entry","main","--out-dir"]).arg(&out),&scratch.0));
        let result=bounded(Command::new(out.join(format!("program{}",std::env::consts::EXE_SUFFIX))).env("SESERAGI_NATIVE_METRICS","1"),&scratch.0);
        assert_eq!(result.status.code(),Some(70),"{name}:{result:?}");assert_eq!(result.stdout,b"created\nacquired\ncold\nreleased\n");
        let stderr=String::from_utf8_lossy(&result.stderr);assert!(stderr.contains("JSON codec nesting exceeds host limits")&&stderr.contains("managed-live=0"),"{name}:{stderr}");
    }
}

#[test]
fn newtype_literal_representations_keep_original_strict_release_types_and_values() {
    compare(
        &root().join(
            "examples/spec/fixtures/rust-backend-regressions/newtype-literal-representations.ssrg",
        ),
        &[Case::success(
            "{\"value\":[\"雪🙂\"]}|[\"雪🙂\"]|[[1,2],\"雪🙂\"]\n".as_bytes(),
        )],
    );
}

#[test]
fn basic_json_derived_newtypes_package_keeps_generic_opaque_and_saved_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-derived-newtypes"),
        &[Case::success(
            "{\"a\":7,\"z\":[1,null,2]}:{\"a\":7,\"z\":[1,null,2]}:[{\"a\":7,\"z\":\"雪🙂\"}]\n"
                .as_bytes(),
        )],
    );
}

#[test]
fn basic_json_derived_newtypes_keep_cold_replay_failure_defects_and_captured_ref_cycles() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let declaration = "alias Rows = { rows: Array<(Int, List<Maybe<json.Json>>)> }";
    assert_eq!(original.matches(declaration).count(), 1);
    let source=original.replace(declaration,"newtype Rows deriving JsonEncode, JsonDecode = { rows: Array<(Int, List<Maybe<json.Json>>)> }");
    let expected = include_bytes!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.stdout"
    );
    let prefix = expected
        .windows(b"recovered: typed".len())
        .position(|part| part == b"recovered: typed")
        .unwrap();
    for (name, changed, output, defect, message) in [
        (
            "cold-replay",
            source.clone(),
            expected.as_slice(),
            false,
            "",
        ),
        (
            "body-defect",
            source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
            &expected[..prefix],
            true,
            "deliberate",
        ),
        (
            "encoder-defect",
            source.replace(
                "9007199254740992000000000001.100",
                "1e99999999999999999999999999999999999999999",
            ),
            b"created\nacquired\ncold\nreleased\n".as_slice(),
            true,
            "Decimal canonical spelling exceeds host limits",
        ),
    ] {
        let path = scratch.0.join(format!("newtype-{name}.ssrg"));
        fs::write(&path, changed).unwrap();
        compare(
            &path,
            &[Case {
                name,
                stdout: output,
                exit: if defect { 70 } else { 0 },
                defect,
                native_defect: message,
                ..Case::success(b"")
            }],
        );
    }
}

#[test]
fn basic_json_derived_newtypes_recursive_encode_and_decode_unwind_the_host_limit() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let old = original
        .lines()
        .find(|line| line.starts_with("  let render = "))
        .unwrap();
    let declaration="newtype Deep deriving JsonEncode, JsonDecode = Maybe<Deep>\nfn deep n: Int -> Deep = Deep (if n <= 0 then Nothing else Just (deep (n - 1)))\n";
    for (name,render) in [
        ("encode","  let render = \\_: Unit -> json.encodeString (deep 512)"),
        ("decode","  let render = \\_: Unit -> match json.decodeString<Deep> \"1\" { Left _ -> \"domain\"; Right value -> json.encodeString value }"),
    ] {
        let path=scratch.0.join(format!("newtype-host-limit-{name}.ssrg"));
        fs::write(&path,format!("{declaration}{}",original.replace(old,render))).unwrap();
        let out=scratch.0.join(format!("native-{name}"));
        successful(bounded(Command::new(env!("CARGO_BIN_EXE_seseragi")).args(["experimental","build-native"]).arg(&path).args(["--entry","main","--out-dir"]).arg(&out),&scratch.0));
        let result=bounded(Command::new(out.join(format!("program{}",std::env::consts::EXE_SUFFIX))).env("SESERAGI_NATIVE_METRICS","1"),&scratch.0);
        assert_eq!(result.status.code(),Some(70),"{name}:{result:?}");
        assert_eq!(result.stdout,b"created\nacquired\ncold\nreleased\n");
        let stderr=String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains("JSON codec nesting exceeds host limits") && stderr.contains("managed-live=0"),"{name}:{stderr}");
    }
}

#[test]
fn basic_json_derived_structs_package_keeps_saved_generic_opaque_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-derived-structs"),
        &[Case::success("{\"z\":[1,null,2],\"a\":{\"value\":7}}:{\"z\":[1,null,2],\"a\":{\"value\":7}}:[{\"z\":\"雪🙂\",\"a\":{\"value\":7}}]\n".as_bytes())],
    );
}

#[test]
fn basic_json_derived_structs_keep_cold_replay_and_unwind_typed_and_encoder_defects() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let declaration = "alias Rows = { rows: Array<(Int, List<Maybe<json.Json>>)> }";
    assert_eq!(original.matches(declaration).count(), 1);
    let source = original.replace(declaration, "struct Rows deriving JsonEncode, JsonDecode { rows: Array<(Int, List<Maybe<json.Json>>)> }");
    let expected = include_bytes!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.stdout"
    );
    let prefix = expected
        .windows(b"recovered: typed".len())
        .position(|part| part == b"recovered: typed")
        .unwrap();
    for (name, changed, output, defect, message) in [
        (
            "cold-replay",
            source.clone(),
            expected.as_slice(),
            false,
            "",
        ),
        (
            "typed-defect",
            source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
            &expected[..prefix],
            true,
            "deliberate",
        ),
        (
            "encoder-defect",
            source.replace(
                "9007199254740992000000000001.100",
                "1e99999999999999999999999999999999999999999",
            ),
            b"created\nacquired\ncold\nreleased\n".as_slice(),
            true,
            "Decimal canonical spelling exceeds host limits",
        ),
    ] {
        let path = scratch.0.join(format!("derived-{name}.ssrg"));
        fs::write(&path, changed).unwrap();
        compare(
            &path,
            &[Case {
                name,
                stdout: output,
                exit: if defect { 70 } else { 0 },
                defect,
                native_defect: message,
                ..Case::success(b"")
            }],
        );
    }
}

#[test]
fn basic_json_derived_structs_recursive_codec_limit_unwinds_with_finalizer_and_root_cleanup() {
    let scratch = Scratch::new();
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let old = original
        .lines()
        .find(|line| line.starts_with("  let render = "))
        .unwrap();
    let declarations = "struct Deep deriving JsonEncode { next: Maybe<Deep> }\nfn deep n: Int -> Deep = Deep { next: if n <= 0 then Nothing else Just (deep (n - 1)) }\n";
    let path = scratch.0.join("derived-host-limit.ssrg");
    fs::write(
        &path,
        format!(
            "{declarations}{}",
            original.replace(
                old,
                "  let render = \\_: Unit -> json.encodeString (deep 512)"
            )
        ),
    )
    .unwrap();
    let out = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        &scratch.0,
    ));
    let result = bounded(
        Command::new(out.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(70), "{result:?}");
    assert_eq!(result.stdout, b"created\nacquired\ncold\nreleased\n");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("JSON codec nesting exceeds host limits")
            && stderr.contains("managed-live=0"),
        "{stderr}"
    );
}

#[test]
fn nested_record_struct_fields_keep_the_original_cross_target_input() {
    compare(
        &root().join(
            "examples/spec/fixtures/rust-backend-regressions/nested-record-struct-field.ssrg",
        ),
        &[Case::success(
            "{\"first\":7,\"payload\":{\"label\":\"雪🙂\"},\"after\":true}\n".as_bytes(),
        )],
    );
}

#[test]
fn basic_json_containers_keep_cold_replay_failures_finalizers_and_ref_cycles() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.stdout"
        ))],
    );
}

#[test]
fn basic_json_containers_defects_release_the_captured_values_and_every_root() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let expected = include_bytes!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.stdout"
    );
    let prefix = expected
        .windows(b"recovered: typed".len())
        .position(|part| part == b"recovered: typed")
        .unwrap();
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    assert_eq!(
        source.matches("9007199254740992000000000001.100").count(),
        1
    );
    for (name, changed, output, message) in [
        (
            "body-defect",
            source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
            &expected[..prefix],
            "deliberate",
        ),
        (
            "encoder-defect",
            source.replace(
                "9007199254740992000000000001.100",
                "1e99999999999999999999999999999999999999999",
            ),
            b"created\nacquired\ncold\nreleased\n".as_slice(),
            "Decimal canonical spelling exceeds host limits",
        ),
    ] {
        let path = scratch.0.join(format!("{name}.ssrg"));
        fs::write(&path, changed).unwrap();
        compare(
            &path,
            &[Case {
                name,
                stdout: output,
                exit: 70,
                defect: true,
                native_defect: message,
                ..Case::success(b"")
            }],
        );
    }
}

#[test]
fn basic_json_containers_package_keeps_opaque_elements_and_imported_user_codecs() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-containers"),
        &[Case::success(
            b"{\"flags\":[true,false],\"rows\":[[1,\"one\"],[2,null]]}:[1,2]:[1,2]:[2,null,3]\n",
        )],
    );
}

#[test]
fn basic_json_containers_native_host_limit_unwinds_with_finalizer_and_root_cleanup() {
    let scratch = Scratch::new();
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-container-effects.ssrg"
    );
    let original = source
        .lines()
        .find(|line| line.starts_with("  let render = "))
        .unwrap();
    let declaration = "type Deep = | Bottom | Next Deep\nfn deep n: Int -> Deep = if n <= 0 then Bottom else Next (deep (n - 1))\ninstance JsonEncode<Deep> { fn encodeJson value: Deep -> json.Json = match value { Bottom -> json.JsonNull; Next inner -> encodeJson<Maybe<Deep>> (Just inner) } }\n";
    let path = scratch.0.join("container-host-limit.ssrg");
    fs::write(
        &path,
        format!(
            "{declaration}{}",
            source.replace(
                original,
                "  let render = \\_: Unit -> json.encodeString (deep 512)"
            )
        ),
    )
    .unwrap();
    let out = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        &scratch.0,
    ));
    let result = bounded(
        Command::new(out.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(70), "{result:?}");
    assert_eq!(result.stdout, b"created\nacquired\ncold\nreleased\n");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("JSON codec nesting exceeds host limits")
            && stderr.contains("managed-live=0"),
        "{stderr}"
    );
}

#[test]
fn basic_json_codecs_encoding_defect_runs_finalizer_and_releases_captured_json() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-json-codec-effects.ssrg");
    let original = "{\\\"answer\\\":9007199254740992000000000001.100,\\\"snow\\\":\\\"雪🙂\\\"}";
    assert_eq!(source.matches(original).count(), 1);
    let path = scratch.0.join("codec-encoding-defect.ssrg");
    fs::write(
        &path,
        source.replace(original, "1e99999999999999999999999999999999999999999"),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "encoding-defect",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Decimal canonical spelling exceeds host limits",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_json_values_errors_paths_and_saved_functions_match_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json.stdout"
        ))],
    );
}

#[test]
fn basic_json_saved_documents_remain_cold_and_release_reference_cycles_after_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-json-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-json-effects.stdout"
        ))],
    );
}

#[test]
fn basic_json_defect_runs_the_finalizer_and_releases_reference_cycles() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-json-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("json-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: "created\nacquired\ncold\n{\"answer\":9007199254740992000000000001.1,\"snow\":\"雪🙂\"}\ncold\n{\"answer\":9007199254740992000000000001.1,\"snow\":\"雪🙂\"}\nreleased\n".as_bytes(),
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_json_host_limit_defect_keeps_cold_parse_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-json-effects.ssrg");
    let original = "{\\\"answer\\\":9007199254740992000000000001.100,\\\"snow\\\":\\\"雪🙂\\\"}";
    assert_eq!(source.matches(original).count(), 1);
    let path = scratch.0.join("json-host-limit.ssrg");
    fs::write(
        &path,
        source.replace(original, "1e99999999999999999999999999999999999999999"),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "host-limit",
            stdout: b"created\nacquired\ncold\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Decimal canonical spelling exceeds host limits",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_json_path_package_keeps_public_values_and_saved_parsers_and_formatters() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json"),
        &[Case::success(
            "{\"n\":42}:{\"snow\":\"雪🙂\",\"n\":9007199254740992000000000001.1}:[null,0.00125]\n"
                .as_bytes(),
        )],
    );
}

#[test]
fn basic_json_resolves_internal_object_seed_at_parse_time_and_runs_the_finalizer() {
    let scratch = Scratch::new();
    let path = scratch.0.join("json-seed.ssrg");
    fs::write(&path, "import * as json from \"std/json\"\nimport * as effects from \"std/effect\"\neffect fn ignore _error: ConsoleError -> Unit with Console = effects.succeed ()\neffect fn say message: String -> Unit with Console = println message |> effects.recover ignore\npub effect fn main = do {\n  say \"created\"\n  effects.scoped (do {\n    effects.acquireRelease (say \"acquired\") (\\_: Unit -> say \"released\")\n    let parsed = json.parse \"{}\"\n    match parsed { Right value -> say (json.stringify value); Left _ -> say \"syntax\" }\n  })\n}\n").unwrap();
    // Process TS artifacts select their seed in the entry prelude and clear
    // inherited environment overrides. Native artifacts read that environment
    // directly, so this invalid-host-configuration probe is native-specific.
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let result = bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_HASH_SEED", "invalid")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(70), "{result:?}");
    assert_eq!(result.stdout, b"created\nacquired\nreleased\n");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("SESERAGI_HASH_SEED must be a signed 64-bit integer"),
        "{stderr}"
    );
    assert!(stderr.contains("managed-live=0"), "{stderr}");
}

#[test]
fn basic_json_file_package_preserves_exact_numbers_and_failed_outputs() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_package(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-json-files"),
        &package,
    );
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    let lock = fs::read(package.join("seseragi.lock")).unwrap();
    compare(
        &package,
        &[
            Case {
                name: "exact-values",
                input: Some(" { \"n\":9007199254740992000000000001.100, \"snow\":\"雪🙂\", \"values\":[null,1.25e-3] }\n".as_bytes()),
                output: Some("{\"n\":9007199254740992000000000001.1,\"snow\":\"雪🙂\",\"values\":[null,0.00125]}\n".as_bytes()),
                ..Case::success(b"")
            },
            Case {
                name: "syntax",
                input: Some(b"[1,]"),
                exit: 1,
                failure: Some("invalid JSON"),
                ..Case::success(b"")
            },
            Case {
                name: "duplicate",
                input: Some("{\"雪\":1,\"\\u96EA\":2}".as_bytes()),
                exit: 1,
                failure: Some("invalid JSON"),
                ..Case::success(b"")
            },
            Case {
                name: "utf8",
                input: Some(b"\xff"),
                exit: 1,
                failure: Some("read failure"),
                ..Case::success(b"")
            },
        ],
    );
    assert_eq!(fs::read(package.join("seseragi.lock")).unwrap(), lock);
}

#[test]
fn basic_map_invalid_seed_stops_before_application_effects() {
    check_invalid_collection_seed("maps", "std/map", "maps.singleton \"snow\" 1", "maps.size");
}

#[test]
fn basic_collections_set_invalid_seed_stops_before_application_effects() {
    check_invalid_collection_seed("sets", "std/set", "sets.singleton \"snow\"", "sets.size");
}

fn check_invalid_collection_seed(alias: &str, module: &str, singleton: &str, size: &str) {
    let scratch = Scratch::new();
    let source = scratch.0.join("startup.ssrg");
    fs::write(&source, format!("import * as {alias} from \"{module}\"\npub effect fn main = do {{\n  println \"MUST_NOT_RUN\"\n  let values = {singleton}\n  println (show ({size} values))\n}}\n")).unwrap();
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    for seed in ["invalid", "9223372036854775808", "-9223372036854775809"] {
        let output = bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .env("PATH", "")
                .env("SESERAGI_HASH_SEED", seed)
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(output.status.code(), Some(70), "{seed}: {output:?}");
        assert!(
            output.stdout.is_empty(),
            "application ran: {seed}: {output:?}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("SESERAGI_HASH_SEED must be a signed 64-bit integer"),
            "{seed}: {stderr}"
        );
        assert!(stderr.contains("managed-live=0"), "{seed}: {stderr}");
    }
}

#[test]
fn existing_user_hash_still_executes_in_program_wasm_on_bun_and_node() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/basic-hash-user.ssrg");
    let original = fs::read(&source).unwrap();
    let out = scratch.0.join("wasm");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-wasm"])
            .arg(&source)
            .args(["--export", "probe", "--out-dir"])
            .arg(&out),
        &scratch.0,
    ));
    let script = scratch.0.join("user-hash.mjs");
    fs::write(&script, "import assert from 'node:assert/strict';import {readFile} from 'node:fs/promises';import {pathToFileURL} from 'node:url';import {resolve} from 'node:path';const dir=process.argv[2];const {loadProgram}=await import(pathToFileURL(resolve(dir,'program.mjs')));const bytes=await readFile(resolve(dir,'program.wasm'));assert.deepEqual(WebAssembly.Module.imports(await WebAssembly.compile(bytes)),[]);const p=await loadProgram(bytes);for(const value of [-42,0,42])assert.equal(p.exports.probe(value),value+100);p.dispose();assert.equal(p.ownership().managedLiveAtClose,0);console.log('user-hash-wasm:ok');").unwrap();
    for name in ["bun", "node"] {
        let output = successful(bounded(
            Command::new(runtime(name, &scratch.0))
                .arg(&script)
                .arg(&out),
            &scratch.0,
        ));
        assert_eq!(output.stdout, b"user-hash-wasm:ok\n");
    }
    assert_eq!(fs::read(source).unwrap(), original);
}

fn copy_package(source: &Path, target: &Path) {
    fs::create_dir(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let output = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_package(&entry.path(), &output);
        } else {
            fs::copy(entry.path(), output).unwrap();
        }
    }
}

#[test]
fn standard_text_math_path_package_transforms_files_and_preserves_failed_outputs() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_package(
        &root().join("examples/spec/fixtures/rust-backend-projects/text-math-csv"),
        &package,
    );
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    let lock = fs::read(package.join("seseragi.lock")).unwrap();
    let cases = [
        Case {
            name: "valid",
            input: Some("\u{85} 9m, 16m ,#skip,,25m\u{3000}".as_bytes()),
            output: Some(b"3.0\n4.0\n5.0\n"),
            ..Case::success(b"")
        },
        Case {
            name: "domain",
            input: Some(b"-1m,0m"),
            output: Some(b"NaN\n0.0\n"),
            ..Case::success(b"")
        },
        Case {
            name: "parse",
            input: Some(b"9m,not-a-float"),
            exit: 1,
            failure: Some("NumberFailure"),
            ..Case::success(b"")
        },
        Case {
            name: "utf8",
            input: Some(b"\xff"),
            exit: 1,
            failure: Some("TextFailure"),
            ..Case::success(b"")
        },
    ];
    compare(&package, &cases);
    assert_eq!(fs::read(package.join("seseragi.lock")).unwrap(), lock);
}

#[test]
fn basic_decimal_keys_keep_exact_public_hashes_saved_methods_shadowing_and_colliding_map_keys() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-keys.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-keys.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_ord_keeps_exact_sign_scale_order_saved_methods_and_inherited_eq() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-ord.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-ord.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_ord_keeps_the_original_zero_regression_input_on_every_target() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-regressions/decimal-zero-order.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend-regressions/decimal-zero-order.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_ord_path_package_keeps_opaque_keys_imported_partials_and_inherited_eq() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-decimal-ord"),
        &[Case::success(b"Less|Less|True|True|True\n")],
    );
}

#[test]
fn basic_decimal_ord_keeps_cold_partials_typed_failure_finalizers_and_ref_cycles() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-ord-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-ord-effects.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_ord_argument_and_returned_callback_defects_release_roots_and_run_finalizers() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-ord-effects.ssrg");
    for (name, old, replacement, stdout) in [
        (
            "argument",
            "withZero (value \"0.01\")",
            "withZero (value \"invalid\")",
            &b"created\nacquired\ncold\nreleased\n"[..],
        ),
        (
            "returned",
            "effects.succeed 42",
            "effects.succeed (42 / 0)",
            &b"created\nacquired\ncold\nLess\nTrue\nreleased\n"[..],
        ),
    ] {
        assert_eq!(original.matches(old).count(), 1);
        let path = scratch.0.join(format!("decimal-ord-{name}-defect.ssrg"));
        fs::write(&path, original.replace(old, replacement)).unwrap();
        compare(
            &path,
            &[Case {
                name,
                stdout,
                exit: 70,
                defect: true,
                native_defect: "Int division by zero",
                ..Case::success(b"")
            }],
        );
    }
}

#[test]
fn basic_decimal_keys_path_package_keeps_opaque_keys_and_saved_imported_generic_methods() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-decimal-keys"),
        &[Case::success(
            b"Just 10|Just 42|[10, 20]|[42, 20]|True|True\n",
        )],
    );
}

#[test]
fn basic_decimal_keys_keep_cold_replay_typed_failure_finalizers_and_ref_cycles() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-decimal-key-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-decimal-key-effects.stdout"
        ))],
    );
}

#[test]
fn basic_decimal_keys_argument_and_returned_callback_defects_run_finalizers_and_release_roots() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-decimal-key-effects.ssrg");
    for (name, old, replacement, stdout) in [
        (
            "key",
            "maps.get (value \"1.200\") stored",
            "maps.get (value \"invalid\") stored",
            &b"created\nacquired\ncold\nreleased\n"[..],
        ),
        (
            "returned",
            "effects.succeed 42",
            "effects.succeed (42 / 0)",
            &b"created\nacquired\ncold\nTrue\nTrue\nreleased\n"[..],
        ),
    ] {
        assert_eq!(original.matches(old).count(), 1);
        let path = scratch.0.join(format!("decimal-key-{name}-defect.ssrg"));
        fs::write(&path, original.replace(old, replacement)).unwrap();
        compare(
            &path,
            &[Case {
                name,
                stdout,
                exit: 70,
                defect: true,
                native_defect: "Int division by zero",
                ..Case::success(b"")
            }],
        );
    }
}

#[test]
fn prelude_reducible_parent_projects_the_selected_iterable() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/reducible-parent-iterable.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/reducible-parent-iterable.stdout"
        ))],
    );
}

#[test]
fn prelude_collections_match_ts_with_selected_dictionaries_saved_stages_and_cold_replay() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/prelude-collections.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/prelude-collections.stdout"
        ))],
    );
}

#[test]
fn prelude_collections_keep_cold_failure_stop_finalizers_and_root_cleanup() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/prelude-collections-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/prelude-collections-effects.stdout"
        ))],
    );
}

#[test]
fn prelude_collections_callback_defect_keeps_finalizer_and_root_cleanup() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/prelude-collections-effects.ssrg"
    );
    let needle = "if value == 2 then effects.fail \"typed\" else effects.succeed ()";
    assert_eq!(original.matches(needle).count(), 1);
    let scratch = Scratch::new();
    let path = scratch.0.join("for-each-defect.ssrg");
    fs::write(
        &path,
        original.replace(
            needle,
            "if value == 2 then say (show (42 / 0)) else effects.succeed ()",
        ),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "callback-defect",
            stdout: b"created\nacquired\n0\n1\n2\nreleased\n",
            exit: 70,
            defect: true,
            native_defect: "Int division by zero",
            ..Case::success(b"")
        }],
    );
}

#[test]
fn remaining_prelude_matches_ts_with_selected_providers_short_circuit_and_saved_stages() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/prelude-remaining.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/prelude-remaining.stdout"
        ))],
    );
}

#[test]
fn prelude_read_line_matches_ts_with_remaining_stdin_and_eof() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/prelude-read-line.ssrg");
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    for (index, (input, expected)) in [
        (b"".as_slice(), "created\nEOF\nEOF\nEOF\nEOF\n"),
        (
            "\u{feff}first\n\u{feff}last".as_bytes(),
            "created\nline:first\nline:last\nEOF\nEOF\n",
        ),
        (
            "雪🙂\r\n\nlast\r".as_bytes(),
            "created\nline:雪🙂\nline:\nline:last\r\nEOF\n",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let input_path = scratch.0.join(format!("input-{index}"));
        fs::write(&input_path, input).unwrap();
        for is_native in [false, true] {
            let mut command = if is_native {
                Command::new(native.join("program"))
            } else {
                let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
                command.arg("run").arg(&source);
                command
            };
            command.stdin(std::process::Stdio::from(
                fs::File::open(&input_path).unwrap(),
            ));
            let result = successful(bounded(&mut command, &scratch.0));
            assert_eq!(
                result.stdout,
                expected.as_bytes(),
                "native={is_native}, input={index}"
            );
            assert!(result.stderr.is_empty(), "{result:?}");
        }
    }
}

#[test]
fn remaining_prelude_native_partials_keep_original_source_and_output() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/prelude-native-partials.ssrg");
    let out = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        &scratch.0,
    ));
    let result = successful(bounded(
        Command::new(out.join("program")).env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    ));
    assert_eq!(
        result.stdout,
        include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/prelude-native-partials.stdout"
        )
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("managed-live=0"),
        "{result:?}"
    );
}

#[test]
fn standard_instances_match_ts_with_generic_saved_and_user_dictionaries() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/standard-instances.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/standard-instances.stdout"
        ))],
    );
}

#[test]
fn standard_applicatives_keep_effect_order_cold_replay_and_user_supertraits() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/standard-applicative-effect.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/standard-applicative-effect.stdout"
        ))],
    );
}
