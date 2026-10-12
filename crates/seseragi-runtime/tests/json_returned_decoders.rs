#![cfg(not(target_arch = "wasm32"))]

#[test]
fn returned_json_decoders_preserve_factory_partial_and_value_application() {
    let source =
        include_str!("../../../examples/spec/fixtures/compile/json-returned-decoders.ssrg");
    let compiled = seseragi_driver::compile_module(seseragi_driver::CompileInput::new(
        "json-returned-decoders.ssrg",
        "fixture/json-returned-decoders",
        source,
    ))
    .expect("returned decoder source must compile");
    let directory = std::env::temp_dir().join(format!(
        "seseragi-json-returned-decoder-{}",
        std::process::id()
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    seseragi_runtime::build_main(
        &compiled,
        &directory,
        seseragi_runtime::BuildTarget::Process,
    )
    .unwrap();
    let output = std::process::Command::new("bun")
        .arg(directory.join("entry.ts"))
        .output()
        .unwrap();
    if output.status.success() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}\nartifact: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        directory.display()
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "7\n7\nabsent\n7\nbad\n7\n[7,8]\n[[\"count\",7]]\n8\n8\n9\n10\n11\nbad\n"
    );
    let typescript = &compiled.generated.typescript;
    assert!(
        typescript.contains("const __ssrg$json$returned"),
        "{typescript}"
    );
    assert!(typescript.contains("__ssrg$json$returned("), "{typescript}");
    assert!(
        !typescript.contains("_argument2"),
        "factory references must not curry the returned decoder argument: {typescript}"
    );
}
