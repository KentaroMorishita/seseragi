use seseragi_driver::{analyze_module, compile_module, CompileInput};

fn input(source: &str) -> CompileInput<'_> {
    CompileInput::new("argument.ssrg", "fixture/argument", source)
}

fn reject(source: &str) {
    let errors = compile_module(input(source)).expect_err("distinct semantic arguments must fail");
    assert!(
        errors.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0101"
                && diagnostic.message_key == "call.argument-type-mismatch"
        }),
        "{source}\n{errors:#?}"
    );
    assert_eq!(analyze_module(input(source)).diagnostics, errors);
}

#[test]
fn same_name_primitive_values_cannot_enter_nominal_function_parameters() {
    for (name, literal, field_type, field_value) in [
        ("Bool", "True", "Int", "1"),
        ("Int", "1", "Bool", "True"),
        ("Float", "(-1.0)", "Bool", "True"),
        ("Char", "'a'", "Bool", "True"),
        ("String", "\"ok\"", "Bool", "True"),
        ("Unit", "()", "Bool", "True"),
    ] {
        let declaration = format!(
            "struct {name} {{value: {field_type}}}\nfn read value: {name} -> {field_type} = value.value\n"
        );
        reject(&format!(
            "{declaration}pub fn main -> {field_type} = read {literal}\n"
        ));
        let source = format!(
            "{declaration}pub fn main -> {field_type} = read ({name} {{value: {field_value}}})\n"
        );
        compile_module(input(&source)).unwrap_or_else(|errors| panic!("{source}\n{errors:#?}"));
    }
}

#[test]
fn unconstrained_generic_calls_reject_mixed_primitive_and_nominal_arguments_in_both_orders() {
    for (primitive, nominal) in [
        ("True", "Bool {value: 1}"),
        ("(True, 1)", "(Bool {value: 1}, 1)"),
        ("{flag: True}", "{flag: Bool {value: 1}}"),
    ] {
        for (left, right) in [(primitive, nominal), (nominal, primitive)] {
            reject(&format!(
                "struct Bool {{value: Int}}\nfn first<A> left: A -> right: A -> A = left\npub effect fn main = do {{let selected = first ({left}) ({right}); println \"unreachable\"}}\n"
            ));
        }
        for same in [primitive, nominal] {
            let source = format!(
                "struct Bool {{value: Int}}\nfn first<A> left: A -> right: A -> A = left\npub effect fn main = do {{let selected = first ({same}) ({same}); println \"valid\"}}\n"
            );
            compile_module(input(&source)).unwrap_or_else(|errors| panic!("{source}\n{errors:#?}"));
        }
    }
}
