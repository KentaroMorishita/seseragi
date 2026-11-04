use seseragi_driver::{compile_core_module, compile_module, BuildProfile, CompileInput};

fn accepts(body: &str) {
    let source = format!("import * as queue from \"std/queue\"\n{body}\n");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input = CompileInput::new("queue.ssrg", "fixture/queue", &source).with_profile(profile);
        let native =
            compile_core_module(input).unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        let ts = compile_module(input).unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        assert_eq!(native.core_ir, ts.core_ir);
        assert!(native.diagnostics.diagnostics.is_empty());
        assert!(ts.diagnostics.diagnostics.is_empty());
        let seseragi_semantics::TypedDecl::EffectFn {
            body: seseragi_semantics::TypedExpr::DoBlock { statements, .. },
            ..
        } = ts.typed_hir.declarations.last().unwrap()
        else {
            panic!("expected main do block");
        };
        for statement in statements {
            if let seseragi_semantics::TypedDoStatement::Bind { pattern, .. } = statement {
                let value = serde_json::to_value(pattern).unwrap();
                assert!(!has_free_element(&value), "{pattern:#?}");
            }
        }
    }
}

fn has_free_element(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(fields) => {
            (fields.get("kind").and_then(|v| v.as_str()) == Some("named")
                && fields.get("name").and_then(|v| v.as_str()) == Some("A"))
                || fields.values().any(has_free_element)
        }
        serde_json::Value::Array(values) => values.iter().any(has_free_element),
        _ => false,
    }
}

#[test]
fn do_binds_infer_queue_elements_from_later_uses() {
    for body in [
        "pub effect fn main = do { q <- queue.unbounded (); queue.offer 1 q }",
        "pub effect fn main = do { q <- queue.bounded 1; queue.tryOffer 1 q }",
        "pub effect fn main = do { q <- queue.unbounded (); let alias = q; queue.offer 1 alias }",
        "pub effect fn main = do { q <- queue.unbounded (); queue.size q; queue.offer 1 q }",
        "pub effect fn main = do { q <- queue.unbounded (); n <- queue.take q; queue.offer 1 q; queue.offer n q }",
        "pub effect fn main = do { ints <- queue.unbounded (); strings <- queue.unbounded (); queue.offer 1 ints; queue.offer \"one\" strings }",
        "pub effect fn main = do { q <- queue.unbounded (); let offer = queue.offer; offer 1 q }",
        "pub effect fn main = do { (q, _) <- succeed (queue.unbounded<Int> (), ()); r <- q; queue.offer 1 r }",
    ] {
        accepts(body);
    }
}

#[test]
fn explicit_and_expected_queue_element_types_still_work() {
    for body in [
        "pub effect fn main = do { q <- queue.unbounded<Int> (); queue.offer 1 q }",
        "pub effect fn main = do { q <- queue.unbounded<String> (); queue.offer \"one\" q }",
        "pub effect fn main = do { let task: Task<queue.Queue<Int>> = queue.unbounded (); q <- task; queue.offer 1 q }",
        "pub effect fn main = do { q <- queue.unbounded (); let task: Task<queue.Queue<Int>> = succeed q; task }",
    ] {
        accepts(body);
    }
}

#[test]
fn queue_binds_are_monomorphic_and_reject_conflicting_elements() {
    for body in [
        "pub effect fn main = do { q <- queue.unbounded<Int> (); queue.offer \"bad\" q }",
        "pub effect fn main = do { q <- queue.unbounded (); queue.offer 1 q; queue.offer \"bad\" q }",
        "pub effect fn main = do { q <- queue.unbounded (); let alias = q; queue.offer 1 q; queue.offer \"bad\" alias }",
        "pub effect fn main = do { let task: Task<queue.Queue<Int>> = queue.unbounded (); q <- task; queue.offer \"bad\" q }",
        "pub effect fn main = do { q <- queue.unbounded<Int> 1; queue.offer 1 q }",
        "pub effect fn main<Unit> value: Unit = do { q <- queue.unbounded<Int> value; queue.offer 1 q }",
        "type Effect<A, B, C> = | FakeEffect\nfn fake -> Effect<Int, Int, Int> = FakeEffect\npub effect fn main = do { q <- fake (); succeed () }",
    ] {
        let source = format!("import * as queue from \"std/queue\"\n{body}\n");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input =
                CompileInput::new("queue.ssrg", "fixture/queue", &source).with_profile(profile);
            let native = compile_core_module(input).expect_err("native frontend must reject");
            let ts = compile_module(input).expect_err("TS frontend must reject");
            assert_eq!(native, ts);
            assert!(
                ts.diagnostics.iter().any(|d| d.code == "SES-T0101"),
                "{source}\n{ts:#?}"
            );
        }
    }
}

#[test]
fn queue_element_inference_keeps_distinct_caller_type_parameters() {
    let source = r#"
import * as queue from "std/queue"
type A = | LocalA
effect fn enqueue<A> value: A = do { q <- queue.unbounded (); queue.offer value q }
effect fn again<A> value: A = do { q <- queue.unbounded (); queue.offer value q }
pub effect fn main = do { enqueue 1; again "one" }
"#;
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input = CompileInput::new("queue.ssrg", "fixture/queue", source).with_profile(profile);
        let native = compile_core_module(input).unwrap_or_else(|error| panic!("{error:#?}"));
        let ts = compile_module(input).unwrap_or_else(|error| panic!("{error:#?}"));
        assert_eq!(native.core_ir, ts.core_ir);
        assert!(native.diagnostics.diagnostics.is_empty());
        assert!(ts.diagnostics.diagnostics.is_empty());
    }
}
