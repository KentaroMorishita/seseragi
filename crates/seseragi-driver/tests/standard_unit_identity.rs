use seseragi_driver::{compile_core_module, compile_module, BuildProfile, CompileInput};

fn accepts(source: &str) {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input =
            CompileInput::new("main.ssrg", "fixture/standard-unit", source).with_profile(profile);
        let native = compile_core_module(input.clone())
            .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        let ts = compile_module(input).unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        assert!(native.diagnostics.diagnostics.is_empty());
        assert!(ts.diagnostics.diagnostics.is_empty());
        assert_eq!(native.core_ir, ts.core_ir);
    }
}

fn rejects(source: &str, argument: &str) {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input =
            CompileInput::new("main.ssrg", "fixture/standard-unit", source).with_profile(profile);
        let native = compile_core_module(input.clone())
            .expect_err("native must reject the source argument before backend admission");
        let ts = compile_module(input).expect_err("TS must reject the same source argument");
        assert_eq!(native, ts);
        assert_eq!(ts.diagnostics.len(), 1, "{source}\n{ts:#?}");
        let diagnostic = &ts.diagnostics[0];
        assert_eq!(diagnostic.code, "SES-T0101");
        assert_eq!(diagnostic.message_key, "call.argument-type-mismatch");
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            argument
        );
    }
}

#[test]
fn standard_queue_rejects_a_local_nominal_unit() {
    let source = "import * as queue from \"std/queue\"\ntype Unit = | FakeUnit\nimport * as effects from \"std/effect\"\npub effect fn main = do { queue.unbounded<Int> FakeUnit; effects.succeed () }\n";
    rejects(source, "FakeUnit");
}

#[test]
fn standard_queue_accepts_the_unit_literal_under_a_local_nominal_unit() {
    let source = "import * as queue from \"std/queue\"\ntype Unit = | FakeUnit\nimport * as effects from \"std/effect\"\npub effect fn main = do { queue.unbounded<Int> (); effects.succeed () }\n";
    accepts(source);
}

#[test]
fn renamed_and_aliased_nominals_cannot_satisfy_standard_unit_parameters() {
    for (declaration, argument) in [
        ("type Unit = | FakeUnit", "FakeUnit"),
        ("type Token = | FakeUnit", "FakeUnit"),
        ("type Token = | FakeUnit\nalias Unit = Token", "FakeUnit"),
        ("type Unit = | FakeUnit\nalias Token = Unit", "FakeUnit"),
        ("struct Unit { value: Int }", "Unit { value: 1 }"),
    ] {
        for application in [
            format!("queue.unbounded<Int> {argument}"),
            format!("{{ let saved = queue.unbounded<Int>; saved {argument} }}"),
        ] {
            let source = format!("import * as queue from \"std/queue\"\nimport * as effects from \"std/effect\"\n{declaration}\npub effect fn main = do {{ {application}; effects.succeed () }}\n");
            rejects(&source, argument);
        }
    }
}

#[test]
fn unit_literals_and_saved_values_keep_their_identity_through_standard_calls() {
    for declaration in [
        "",
        "type Unit = | FakeUnit",
        "struct Unit { value: Int }",
        "type Token = | FakeUnit\nalias Unit = Token",
    ] {
        for application in [
            "queue.unbounded<Int> ()",
            "{ let unit = (); queue.unbounded<Int> unit }",
            "{ let saved = queue.unbounded<Int>; saved () }",
        ] {
            let source = format!("import * as queue from \"std/queue\"\n{declaration}\npub effect fn main -> queue.Queue<Int> = {application}\n");
            accepts(&source);
        }
    }
}

#[test]
fn a_renamed_standard_function_keeps_its_unit_parameter() {
    accepts("import { unbounded as create } from \"std/queue\"\ntype Unit = | FakeUnit\npub effect fn main = create<Int> ()\n");
    rejects("import { unbounded as create } from \"std/queue\"\ntype Unit = | FakeUnit\npub effect fn main = create<Int> FakeUnit\n", "FakeUnit");
}

#[test]
fn a_source_unit_owner_still_controls_its_own_arguments_and_generic_results() {
    accepts("import * as queue from \"std/queue\"\ntype Unit = | FakeUnit\nfn own value: Unit -> Unit = value\npub effect fn main = do { let value = own FakeUnit; queue.unbounded<Unit> () }\n");
    rejects("type Unit = | FakeUnit\nfn own value: Unit -> Unit = value\npub effect fn main = succeed (own ())\n", "()");
    rejects("import * as queue from \"std/queue\"\nimport * as effects from \"std/effect\"\npub effect fn main<Unit> value: Unit = do { queue.unbounded<Int> value; effects.succeed () }\n", "value");
}
