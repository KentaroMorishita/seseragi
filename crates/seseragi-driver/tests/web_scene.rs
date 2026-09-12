use seseragi_driver::{compile_module, CompileInput};

#[test]
fn compiles_svg_pointer_wheel_and_dom_measurement_from_one_standard_contract() {
    let source = include_str!(
        "../../../examples/spec/fixtures/projects/web-scene-interaction/src/main.ssrg"
    );
    let compiled = compile_module(CompileInput::new(
        "main.ssrg",
        "fixture/web-scene-interaction",
        source,
    ))
    .expect("web scene fixture must compile");
    let typescript = &compiled.generated.typescript;

    for expected in [
        "@seseragi/runtime/svg",
        "_ssrg_svg_toHtml",
        "_ssrg_dom_capturePointer",
        "_ssrg_dom_releasePointer",
        "_ssrg_dom_measure",
        "_ssrg_dom_observeResize",
        "_ssrg_dom_disconnect",
    ] {
        assert!(
            typescript.contains(expected),
            "missing {expected}\n{typescript}"
        );
    }
}
