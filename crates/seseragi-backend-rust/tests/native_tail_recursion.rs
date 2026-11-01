#[path = "support/native.rs"]
mod support;

use std::process::Command;
use support::{emit, run, rust_id, Scratch};

#[test]
fn scalar_loop_operators_preserve_nested_arguments_ordering_and_negation() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/scalar-tail-operators.ssrg");
    let artifact = emit(source);
    let walk = rust_id("native/main::walk");
    let direct = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {walk}_direct(")))
        .unwrap();
    assert!(!direct.contains(".clone()"));
    assert!(direct.contains("int_less_equal_value("));
    assert!(direct.contains("int_less_value("));
    assert!(direct.contains("int_multiply_value("));
    assert!(direct.contains(".checked_neg()"));
    run(source, "(4, -4, 9007199254740991)");
}

#[test]
fn scalar_global_reads_preserve_defect_order_zero_step_and_partial_lifetimes() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/scalar-global.ssrg");
    let artifact = emit(source);
    let sum = rust_id("native/main::sum");
    let retained = rust_id("native/main::retained");
    let direct = artifact
        .source
        .lines()
        .find(|l| l.starts_with(&format!("fn {sum}_direct(")))
        .unwrap();
    assert!(direct.contains("get_global_int(0)"));
    assert!(direct.contains("int_add_value("));
    assert!(!direct.contains("binary(") && !direct.contains(".clone()"));
    let fallback = artifact
        .source
        .lines()
        .find(|l| l.starts_with(&format!("fn {retained}_direct(")))
        .unwrap();
    assert!(fallback.contains("get_global(0)") && fallback.contains("binary("));
    let end = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..end].to_owned();
    for (name, op, result) in [
        ("int_add_value", "+", "i64"),
        ("int_subtract_value", "-", "i64"),
        ("int_equal_value", "==", "bool"),
    ] {
        let marker = format!("fn {name}(a: i64, b: i64) -> {result} {{");
        assert_eq!(generated.matches(&marker).count(), 1);
        generated = generated.replace(&marker, &format!("{marker} TRACE.with(|trace| trace.borrow_mut().push(format!(\"{op}:{{a}}:{{b}}\")));"));
    }
    let marker = "fn read_global<T>(index: usize, read: impl FnOnce(&V) -> T) -> T {";
    assert_eq!(generated.matches(marker).count(), 1);
    generated = generated.replace(
        marker,
        &format!(
            "{marker} TRACE.with(|trace| trace.borrow_mut().push(format!(\"global:{{index}}\")));"
        ),
    );
    generated.push_str(&r#"
thread_local! { static TRACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; }
fn observe(f: fn(Vec<V>,V,V)->V,n:i64,seed:i64)->(Result<i64,String>,Vec<String>) {
 TRACE.with(|t|t.borrow_mut().clear());
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||f(vec![],checked_int(n),checked_int(seed))));
 let result=match result { Ok(V::Int(n))=>Ok(n), Ok(_)=>panic!("wrong result"), Err(e)=>Err(e.downcast_ref::<String>().cloned().or_else(||e.downcast_ref::<&str>().map(|s|(*s).into())).unwrap()) };
 (result,TRACE.with(|t|t.borrow().clone()))
}
fn main() {
 std::panic::set_hook(Box::new(|_|{}));
 for n in [0,1] {
  let actual=observe(<SUM>_direct,n,42);let ordinary=observe(<RETAINED>_direct,n,42);assert_eq!(actual,ordinary);
  if n==0 { assert_eq!(actual.0,Ok(42));assert!(!actual.1.iter().any(|s|s.starts_with("global:"))); }
  else { assert!(actual.0.unwrap_err().contains("uninitialized global"));assert_eq!(actual.1.last().unwrap(),"global:0"); }
 }
 initialize_globals();
 const MAX:i64=9007199254740991;let mut cases=Vec::new();
 for n in 0..=10 {for seed in [0,1,MAX,MAX-1,-MAX,-MAX+1] {cases.push((n,seed));}}
 cases.extend([(-MAX,0),(-MAX,-MAX),(-MAX,MAX)]);
 for (n,seed) in cases {
  let actual=observe(<SUM>_direct,n,seed);let ordinary=observe(<RETAINED>_direct,n,seed);assert_eq!(actual,ordinary,"n={n} seed={seed}");
  if n>=0 {let exact=i128::from(seed)+i128::from(n)*3;if (-i128::from(MAX)..=i128::from(MAX)).contains(&exact) {assert_eq!(actual.0,Ok(exact as i64));}else {assert!(actual.0.unwrap_err().contains("Int overflow"));}}
  else {assert!(actual.0.unwrap_err().contains("Int overflow"));assert_eq!(actual.1.last().unwrap(),&format!("-:{}:1",-MAX));}
 }
 let partial=<SUM>(vec![]).apply(checked_int(40));assert_eq!(partial.apply(checked_int(1)).render(),"121");assert_eq!(partial.apply(checked_int(2)).render(),"122");drop(partial);
 let result=<PROBE>(vec![]).apply(V::Unit);assert_eq!(result.render(),"(6, 7, 4, 22, True)");drop(result);
 clear_globals();clear_dictionaries();assert_eq!(LIVE.with(Cell::get),0);
 println!("69 scalar/global traces agree; zero-step, uninitialized, partial and probe pass");
}
"#.replace("<SUM>",&sum).replace("<RETAINED>",&retained).replace("<PROBE>",&rust_id(&artifact.metadata.entry)));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn scalar_tail_helpers_preserve_abi_checked_boundaries_and_primitive_order() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/scalar-tail-loop.ssrg");
    let artifact = emit(source);
    let sum = rust_id("native/main::sum");
    let retained = rust_id("native/main::retained");
    let direct = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {sum}_direct(")))
        .unwrap();
    assert!(direct.contains("loop {"));
    assert!(direct.contains("int_add_value("));
    assert!(!direct.contains(".clone()"));
    assert!(!direct.contains("binary("));
    assert!(!direct.contains("Shared::"));
    let fallback = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {retained}_direct(")))
        .unwrap();
    assert!(fallback.contains(".clone()") && fallback.contains("binary("));
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    for (name, op, result) in [
        ("int_add_value", "+", "i64"),
        ("int_subtract_value", "-", "i64"),
        ("int_equal_value", "==", "bool"),
    ] {
        let marker = format!("fn {name}(a: i64, b: i64) -> {result} {{");
        assert_eq!(generated.matches(&marker).count(), 1);
        generated = generated.replace(
            &marker,
            &format!("{marker} TRACE.with(|trace| trace.borrow_mut().push((\"{op}\", a, b)));"),
        );
    }
    let harness = r#"
thread_local! { static TRACE: RefCell<Vec<(&'static str, i64, i64)>> = const { RefCell::new(Vec::new()) }; }
fn outcome(f: fn(Vec<V>, V, V) -> V, n: i64, seed: i64) -> (Result<i64, String>, Vec<(&'static str, i64, i64)>) {
 TRACE.with(|t| t.borrow_mut().clear());
 let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(vec![], checked_int(n), checked_int(seed))));
 let outcome = match value {
  Ok(V::Int(n)) => Ok(n),
  Ok(_) => panic!("wrong result"),
  Err(e) => Err(e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| (*s).into())).unwrap()),
 };
 (outcome, TRACE.with(|t| t.borrow().clone()))
}

fn main() {
 std::panic::set_hook(Box::new(|_| {}));
 const MAX: i64 = 9_007_199_254_740_991;
 let mut cases = Vec::new();
 for n in 0..=10 { for seed in [0, 1, MAX, MAX-1, -MAX, -MAX+1] { cases.push((n,seed)); } }
 cases.extend([(-MAX,0),(-MAX,-MAX),(-MAX,MAX)]);
 for (n,seed) in cases {
  let scalar = outcome(<SUM>_direct,n,seed);
  let ordinary = outcome(<RETAINED>_direct,n,seed);
  assert_eq!(scalar, ordinary, "n={n} seed={seed}");
  if n >= 0 {
   let exact = i128::from(seed) + i128::from(n) * i128::from(n+1) / 2;
   if (-i128::from(MAX)..=i128::from(MAX)).contains(&exact) { assert_eq!(scalar.0, Ok(exact as i64)); }
   else { assert_eq!(scalar.0.as_ref().map_err(String::as_str), Err("Seseragi Int overflow")); }
  } else {
   assert_eq!(scalar.0.as_ref().map_err(String::as_str), Err("Seseragi Int overflow"));
   assert_eq!(scalar.1.last(), Some(&("-", -MAX, 1)));
  }
 }
 let partial = <SUM>(vec![]).apply(checked_int(40));
 assert_eq!(partial.apply(checked_int(1)).render(), "821");
 assert_eq!(partial.apply(checked_int(2)).render(), "822");
 drop(partial);
 let result = <PROBE>(vec![]).apply(V::Unit);
 assert_eq!(result.render(), "(821, 822, 821, True, 35, 42, 3628800)");
 drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}
"#.replace("<SUM>", &sum).replace("<RETAINED>", &retained).replace("<PROBE>", &rust_id(&artifact.metadata.entry));
    generated.push_str(&harness);
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pure_tail_recursion_preserves_snapshots_partial_calls_and_changing_evidence() {
    for (source, expected) in [
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/tail-recursion.ssrg"),
            "500000500000",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/tail-capture.ssrg"),
            "(100009, 100019)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/tail-evidence.ssrg"),
            "(35, 42)",
        ),
    ] {
        let artifact = emit(source);
        assert_eq!(artifact.metadata.self_tail_calls, 1);
        let main = artifact.source.rfind("fn main()").unwrap();
        let generated = format!(
            "{}\nfn main() {{ std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), {expected:?}); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}).unwrap().join().unwrap(); }}",
            &artifact.source[..main], rust_id(&artifact.metadata.entry),
        );
        let scratch = Scratch::new();
        let output = Command::new(scratch.compile(&generated))
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    // Returning a lambda is not a tail call at the point of constructing it.
    let artifact = emit("fn later n: Int -> (Unit -> Int) = \\_ -> if n == 0 then 42 else later (n - 1) ()\npub fn probe -> Int = later 5 ()\n");
    assert_eq!(artifact.metadata.self_tail_calls, 0);
    run("fn later n: Int -> (Unit -> Int) = \\_ -> if n == 0 then 42 else later (n - 1) ()\npub fn probe -> Int = later 5 ()\n", "42");
}
