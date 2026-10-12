use seseragi_driver::{compile_module, CompileInput};

#[test]
fn lowers_json_factory_then_returned_decoder_without_effect_binding() {
    let source =
        include_str!("../../../examples/spec/fixtures/compile/json-returned-decoders.ssrg");
    let compiled = compile_module(CompileInput::new(
        "json-returned-decoders.ssrg",
        "fixture/json-returned-decoders",
        source,
    ))
    .unwrap();
    let emitted = compiled.generated.typescript;
    for factory in [
        "field",
        "optionalField",
        "index",
        "array",
        "record",
        "oneOf",
        "map",
        "flatMap",
    ] {
        assert!(
            emitted.contains(&format!("= _ssrg_json_{factory}(")),
            "missing separately evaluated factory {factory}: {emitted}"
        );
        assert!(
            !emitted.contains(&format!("_ssrg_effect_flatMap(_ssrg_json_{factory}(")),
            "pure returned call must not enter effect binding: {emitted}"
        );
    }
    assert!(
        emitted.contains("return __ssrg$json$returned("),
        "{emitted}"
    );
    assert!(
        !emitted.contains("_argument2"),
        "returned callable is not a factory parameter: {emitted}"
    );
}
