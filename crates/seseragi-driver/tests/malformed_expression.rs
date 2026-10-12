use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn malformed_declaration_bodies_keep_errors_instead_of_lowering_placeholders() {
    for source in [
        "pub fn same value: Int -> Int = (fn inputValue: Int => inputValue) value\n",
        "pub let value: Int = (fn inputValue: Int => inputValue) 7\n",
        "pub fn value -> Int = 1 , unexpected\n",
        "pub fn value -> Int =\n",
    ] {
        let analysis = analyze_module(CompileInput::new("bad.ssrg", "fixture/bad", source));
        assert!(
            analysis
                .diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SES-P0001"),
            "{source}: {:?}",
            analysis.diagnostics
        );
        assert!(
            compile_module(CompileInput::new("bad.ssrg", "fixture/bad", source)).is_err(),
            "{source}"
        );
        for diagnostic in &analysis.diagnostics.diagnostics {
            assert!(
                diagnostic.primary.start <= diagnostic.primary.end
                    && diagnostic.primary.end <= source.len()
            );
        }
    }
}

#[test]
fn canonical_lambda_and_empty_record_remain_valid() {
    for source in [
        "pub let same: (Int -> Int) = \\inputValue -> inputValue\n",
        "let empty = {}\n",
    ] {
        compile_module(CompileInput::new("good.ssrg", "fixture/good", source)).unwrap();
    }
}
