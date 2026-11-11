//! `do` blocks take `pure` and narrower Effects on both targets: `pure` in an
//! `effect fn` is the succeeding Effect, and an Effect that needs fewer
//! services or fails with Never runs inside a wider Effect's `do`.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-do-effect-widening-{}-{}",
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

fn runs_on_both_targets(source: &str, stdout: &str) {
    let scratch = Scratch::new();
    let file = scratch.0.join("main.ssrg");
    fs::write(&file, source).unwrap();
    let ts = successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&file),
        &scratch.0,
    ));
    assert_eq!(String::from_utf8_lossy(&ts.stdout), stdout);
    let native = successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["run", "--target", "native"])
            .arg(&file),
        &scratch.0,
    ));
    assert_eq!(String::from_utf8_lossy(&native.stdout), stdout);
}

fn refused(source: &str, message: &str) {
    let scratch = Scratch::new();
    let file = scratch.0.join("main.ssrg");
    fs::write(&file, source).unwrap();
    let output = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&file),
        &scratch.0,
    );
    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(message), "{stderr}");
}

#[test]
fn pure_in_an_effect_function_do_is_the_succeeding_effect() {
    runs_on_both_targets(
        "pub effect fn main -> Unit with Console fails ConsoleError =\n  do {\n    x <- pure 1\n    println (show x)\n  }\n",
        "1\n",
    );
    runs_on_both_targets(
        "effect fn report n: Int -> Unit with Console fails ConsoleError =\n  do {\n    if n % 2 == 0 then println (show n) else pure ()\n    println \"done\"\n  }\n\npub effect fn main -> Unit with Console fails ConsoleError = do {\n  report 1\n  report 2\n}\n",
        "done\n2\ndone\n",
    );
}

#[test]
fn a_narrower_effect_runs_in_an_effect_returning_function_do() {
    runs_on_both_targets(
        "import * as effects from \"std/effect\"\n\nfn load n: Int -> Effect<{ console: Console }, ConsoleError, { n: Int }> =\n  do {\n    println \"loading\"\n    effects.succeed { n: n }\n  }\n\npub effect fn main -> Unit with Console fails ConsoleError =\n  do {\n    r <- load 1\n    println (show r.n)\n  }\n",
        "loading\n1\n",
    );
}

#[test]
fn a_never_failure_or_narrower_stream_widens_on_both_targets() {
    runs_on_both_targets(
        "import * as effects from \"std/effect\"\n\nfn widen<R> action: Effect<R, Never, Int> -> Effect<R, ConsoleError, Int> = do { action }\n\npub effect fn main -> Unit with Console fails ConsoleError = do {\n  n <- widen (effects.succeed 5)\n  println (show n)\n}\n",
        "5\n",
    );
    runs_on_both_targets(
        "import { Stream } from \"std/stream\"\nimport * as streams from \"std/stream\"\n\nfn source -> Stream<{ console: Console }, ConsoleError, Int> = do {\n  let narrow: Stream<{}, Never, Int> = streams.fromArray [1, 2, 3]\n  narrow\n}\n\npub effect fn main -> Unit with Console fails ConsoleError = do {\n  items <- streams.runCollect (source ())\n  println (show items)\n}\n",
        "[1, 2, 3]\n",
    );
}

#[test]
fn a_do_still_refuses_a_different_failure_or_monad() {
    refused(
        "import * as effects from \"std/effect\"\n\nfn load n: Int -> Effect<{ console: Console }, ConsoleError, Int> =\n  do {\n    println \"loading\"\n    effects.fail \"other\"\n  }\n\npub effect fn main -> Unit with Console fails ConsoleError = do {\n  n <- load 1\n  println (show n)\n}\n",
        "Do monad constructor mismatch",
    );
    refused(
        "fn load n: Int -> Effect<{ console: Console }, ConsoleError, Int> =\n  do {\n    println \"loading\"\n    Just n\n  }\n\npub effect fn main -> Unit with Console fails ConsoleError = do {\n  n <- load 1\n  println (show n)\n}\n",
        "Do monad constructor mismatch",
    );
}

/// `Effect<R, E, _>` and `Effect<R, E>` name the same partly applied
/// constructor, so either spelling is the base of `MaybeT` on both targets.
#[test]
fn a_transformer_do_over_an_effect_type_checks_and_runs() {
    let lesson = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/lessons/20-maybe-transformer.ssrg"),
    )
    .unwrap();
    let written = "MaybeT<Effect<{ console: Console }, ConsoleError, _>, A>";
    assert!(lesson.contains(written));
    runs_on_both_targets(&lesson, "Hello, Aki!\n");
    runs_on_both_targets(
        &lesson.replace(
            written,
            "MaybeT<Effect<{ console: Console }, ConsoleError>, A>",
        ),
        "Hello, Aki!\n",
    );
}

#[test]
fn an_impl_method_with_an_unannotated_self_is_reported_at_its_declaration() {
    refused(
        "struct Box<A> deriving Show { value: A }\n\nimpl<A> Box<A> {\n  pub fn mapBox<B> self -> f: (A -> B) -> Box<B> = Box { value: f self.value }\n}\n\npub effect fn main -> Unit with Console fails ConsoleError =\n  println $ show $ (Box { value: 1 }).mapBox (\\x -> x + 1)\n",
        "4:3: error[SES-P0001]: Ordinary functions require parameter and result type annotations",
    );
}
