#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use support::run;

#[test]
fn math_cbrt_keeps_the_original_previously_unsupported_source() {
    let source = "import * as math from \"std/math\"\npub fn probe -> Float = math.cbrt 4.0\n";
    let core = compile_core_module(CompileInput::new("bad.ssrg", "native/bad", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "probe").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|capability| capability == "math"));
    assert!(artifact.source.contains("std/math::cbrt"));
}

#[test]
fn unsupported_features_have_target_codes_and_source_ranges() {
    run(
        "pub fn probe -> List<Int> = `[n | n <- `[1, 2]]\n",
        "`[1, 2]",
    );
    let source =
        "import * as effects from \"std/effect\"\npub effect fn probe -> Int = effects.race (effects.succeed 1) (effects.succeed 2)\n";
    let core = compile_core_module(CompileInput::new("bad.ssrg", "native/bad", source))
        .unwrap()
        .core_ir;
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    let origin = error.origin.unwrap();
    assert!(origin.end > origin.start);
    assert!(error.message.contains("race"));
}
