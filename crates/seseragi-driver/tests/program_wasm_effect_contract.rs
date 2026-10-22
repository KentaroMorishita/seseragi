use seseragi_driver::{compile_core_module, compile_module, CompileInput};
use seseragi_lowering::CoreType;

#[test]
fn an_effect_declaration_keeps_source_stages_separate_from_its_success_function() {
    let source = "import * as effects from \"std/effect\"\n\
        pub effect fn selected prefix: String -> _trigger: Unit -> (Int -> String) fails String =\n\
          effects.succeed (\\_value: Int -> prefix)\n";
    let input = || CompileInput::new("effect-stages.ssrg", "fixture/effect-stages", source);
    let standalone = compile_core_module(input()).unwrap();
    let ts = compile_module(input()).unwrap();
    assert_eq!(standalone.core_ir, ts.core_ir);
    let function = &standalone.core_ir.functions[0];
    assert!(function.is_effect);
    assert_eq!(function.parameters.len(), 2);
    let mut result = function
        .value_type
        .as_ref()
        .expect("checked declaration type");
    for _ in &function.parameters {
        let CoreType::Function { result: next, .. } = result else {
            panic!("missing checked source stage: {result:?}")
        };
        result = next;
    }
    let CoreType::Named { name, arguments } = result else {
        panic!("missing Effect contract")
    };
    assert_eq!(name, "std/prelude::Effect");
    assert_eq!(arguments.len(), 3);
    assert!(
        matches!(&arguments[0], CoreType::Record { closed: true, fields } if fields.is_empty())
    );
    assert!(matches!(arguments[2], CoreType::Function { .. }));
    assert_ne!(arguments[1], CoreType::Hole);
}
