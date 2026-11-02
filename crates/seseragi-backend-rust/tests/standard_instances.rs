use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_module, CompileInput};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn standard_instances_match_ts_with_selected_user_dictionaries_and_saved_stages() {
    let (native, typescript) = execute(include_str!(
        "../../../examples/spec/fixtures/rust-backend/standard-instances.ssrg"
    ));
    assert!(typescript.status.success(), "{typescript:?}");
    assert_eq!(native.stdout, typescript.stdout);
    assert_eq!(
        native.stdout,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/standard-instances.stdout")
    );
}

fn execute(source: &str) -> (Output, Output) {
    let compiled = compile_module(CompileInput::new(
        "instances.ssrg",
        "native/instances",
        source,
    ))
    .unwrap();
    let artifact = emit_rust_module(&compiled.core_ir, "probe").unwrap();
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "seseragi-standard-instances-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    let rust = scratch.0.join("program.rs");
    let binary = scratch.0.join("program");
    fs::write(&rust, artifact.source).unwrap();
    let build = Command::new("rustc")
        .args(["--edition=2021", "-o"])
        .arg(&binary)
        .arg(&rust)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let native = Command::new(&binary)
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(native.status.success(), "{native:?}");
    assert!(
        String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
        "{native:?}"
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let ts = scratch.0.join("program.ts");
    fs::write(
        &ts,
        format!(
            "{}\nconsole.log(probe());\n",
            compiled.generated.typescript.replace(
                "\"@seseragi/runtime/",
                &format!("\"{}/runtime/ts/src/", root.display())
            )
        ),
    )
    .unwrap();
    let typescript = Command::new("bun").arg(&ts).output().unwrap();
    (native, typescript)
}

#[test]
fn non_empty_ord_passes_parent_eq_in_ts_and_native() {
    let (native, typescript) = execute(
        r#"
import { NonEmptyList } from "std/non-empty-list"
fn inherited<A> left: A -> right: A -> Bool where Ord<A> = left == right
fn singleton value: Int -> NonEmptyList<Int> = pure value
pub fn probe -> String = show (inherited (singleton 1) (singleton 1))
"#,
    );
    assert_eq!(native.stdout, b"True\n");
    assert!(typescript.status.success(), "{typescript:?}");
    assert_eq!(typescript.stdout, native.stdout);
}

#[test]
fn standard_instances_reject_forged_providers_kinds_factories_and_children() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
    let sources = [
        "fn check<A> value: A -> Bool where Eq<A> = value == value\npub fn probe -> Bool = check [1]",
        "fn check<A> value: A -> Bool where Eq<A> = value == value\npub fn probe -> Bool = check `[1]",
        "fn check<A> value: A -> Bool where Eq<A> = value == value\npub fn probe -> Bool = check (Just 1)",
        "fn check<A> value: A -> Bool where Eq<A> = value == value\nfn value -> Either<String, Int> = Right 1\npub fn probe -> Bool = check (value ())",
        "import * as maps from \"std/map\"\nfn check<A> value: A -> Bool where Eq<A> = value == value\npub fn probe -> Bool = check (maps.singleton 1 2)",
        "import * as sets from \"std/set\"\nfn check<A> value: A -> Bool where Eq<A> = value == value\npub fn probe -> Bool = check (sets.singleton 1)",
        "import {NonEmptyList} from \"std/non-empty-list\"\nfn check<A> value: A -> Bool where Eq<A> = value == value\nfn value -> NonEmptyList<Int> = pure 1\npub fn probe -> Bool = check (value ())",
        "import {NonEmptyList} from \"std/non-empty-list\"\nfn check<A> value: A -> Ordering where Ord<A> = compare value value\nfn value -> NonEmptyList<Int> = pure 1\npub fn probe -> Ordering = check (value ())",
        "import {NonEmptyList} from \"std/non-empty-list\"\nfn check<A> value: A -> Int where Hash<A> = hash value\nfn value -> NonEmptyList<Int> = pure 1\npub fn probe -> Int = check (value ())",
        "fn check<A> value: A -> Ordering where Ord<A> = compare value value\npub fn probe -> Ordering = check True",
        "fn check<A> value: A -> Ordering where Ord<A> = compare value value\npub fn probe -> Ordering = check \"雪🙂\"",
        "fn check<A> value: A -> Ordering where Ord<A> = compare value value\npub fn probe -> Ordering = check ()",
        "import {NonEmptyList} from \"std/non-empty-list\"\nimport * as arrays from \"std/array\"\nfn check<C,A> value: C -> Array<A> where Iterable<C,A> = arrays.fromIterable value\nfn value -> NonEmptyList<Int> = pure 1\npub fn probe -> Array<Int> = check (value ())",
        "import {NonEmptyList} from \"std/non-empty-list\"\nfn check<C> value: C -> Int where Reducible<C,Int> = sum value\nfn value -> NonEmptyList<Int> = pure 1\npub fn probe -> Int = check (value ())",
        "import {NonEmptyList} from \"std/non-empty-list\"\nimport * as maybe from \"std/maybe\"\nfn check<F<_>,A> value: F<Maybe<A>> -> Maybe<F<A>> where Traversable<F> = maybe.sequence value\nfn value -> NonEmptyList<Maybe<Int>> = pure (Just 1)\npub fn probe -> Maybe<NonEmptyList<Int>> = check (value ())",
    ];
    for source in sources {
        let core = compile_module(CompileInput::new("proof.ssrg", "native/proof", source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..12 {
            let mut bad = core.clone();
            let CoreExpr::Call { evidence, .. } = &mut bad
                .functions
                .iter_mut()
                .find(|f| f.symbol.ends_with("::probe"))
                .unwrap()
                .body
            else {
                panic!("call")
            };
            let selected = &mut evidence[0];
            let child = selected.clone();
            let CoreInstanceEvidence::Standard {
                identity,
                type_arguments,
                evidence_arguments,
            } = &mut selected.evidence
            else {
                panic!("standard")
            };
            match mutation {
                0 => *identity = format!("counterfeit::{}", selected.constraint.name),
                1 => selected.constraint.trait_identity = Some("counterfeit::Eq".into()),
                2 => selected.constraint.name = "Show".into(),
                3 => selected.constraint.arguments.clear(),
                4 => selected.constraint.arguments[0] = CoreType::Hole,
                5 => type_arguments.push(CoreType::Hole),
                6 => evidence_arguments.push(child),
                7 if !evidence_arguments.is_empty() => {
                    evidence_arguments[0].constraint.trait_identity = Some("counterfeit::Eq".into())
                }
                8 if !evidence_arguments.is_empty() => {
                    evidence_arguments[0].constraint.arguments = vec![CoreType::Hole]
                }
                9 if !evidence_arguments.is_empty() => {
                    evidence_arguments[0].constraint.name = "Show".into()
                }
                10 if !type_arguments.is_empty() => {
                    type_arguments[0] = CoreType::Named {
                        name: "Float".into(),
                        arguments: vec![],
                    }
                }
                _ => {
                    selected.constraint.arguments[0] = CoreType::Function {
                        parameter: Box::new(CoreType::Hole),
                        result: Box::new(CoreType::Hole),
                    }
                }
            }
            assert!(
                emit_rust_module(&bad, "probe").is_err(),
                "mutation {mutation}: {source}"
            );
        }
    }
}
