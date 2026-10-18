use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
};
use seseragi_semantics::type_module;

#[test]
fn constrained_partials_stage_supplied_source_calls_before_remaining_lambdas() {
    for declaration in [
        "fn prefix<A> value: A -> suffix: String -> String where Show<A> = show value + suffix\npub fn probe -> Unit = { let partial = prefix [9007199254740991 + 1]; () }",
        "pub fn probe -> Unit = { fn prefix<A> value: A -> suffix: String -> String where Show<A> = show value + suffix; let partial = prefix [9007199254740991 + 1]; () }",
        "type Token = | Token\ntrait Render<T> { fn render<A> tag: T -> value: A -> suffix: String -> String where Show<A> }\ninstance Render<Token> { fn render<A> tag: Token -> value: A -> suffix: String -> String where Show<A> = show value + suffix }\npub fn probe -> Unit = { let partial = render Token [9007199254740991 + 1]; () }",
    ] {
        let core = lower_typed_module(type_module("strict-partial.ssrg", declaration));
        let ir = lower_core_module_to_typescript_ir(core);
        let output = emit_typescript_module(ir, declaration).typescript;
        // An eager binding must precede the deferred suffix parameter. The
        // executable fixture checks semantics; this guards the staging boundary.
        let stage = output.find("const __ssrg$partial$stage").expect(&output);
        let supplied = output[stage..].find("9007199254740991").unwrap() + stage;
        let deferred = output[stage..].find("=> __ssrg$partial$stage").expect(&output) + stage;
        assert!(supplied < deferred, "{output}");
        assert_eq!(output.matches("9007199254740991").count(), 1, "{output}");
    }
}
