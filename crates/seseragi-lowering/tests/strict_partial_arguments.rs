use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, TypeScriptExpr,
    TypeScriptFunction, TypeScriptStatement, TypeScriptType,
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

#[test]
fn constrained_first_callback_and_saved_prefix_use_checked_type_arguments() {
    let source = "fn transform<E,A,B> callback:(A -> Either<E,B>) -> value:A -> (Int -> Either<E,B>) where Show<A> = \\_stage:Int -> callback value\n\
                  fn choose value:Int -> Either<String,Int> = Right value\n\
                  pub fn complete -> Either<String,Int> = transform choose 7 42\n\
                  pub fn partial -> (Int -> Int -> Either<String,Int>) = transform choose\n";
    let core = lower_typed_module(type_module("checked-callback.ssrg", source));
    let ir = lower_core_module_to_typescript_ir(core);
    let TypeScriptFunction::ConstFunction { body, .. } = &ir.functions[2];
    let TypeScriptExpr::CheckedResult { value, .. } = body else {
        panic!("{body:#?}");
    };
    let TypeScriptExpr::Call {
        checked_callee_type: None,
        type_arguments,
        arguments,
        ..
    } = &**value
    else {
        panic!("{value:#?}");
    };
    // callback, original value, dictionary, then the returned function's arg.
    assert_eq!(arguments.len(), 4);
    assert_eq!(
        type_arguments,
        &[
            TypeScriptType::String,
            TypeScriptType::Number,
            TypeScriptType::Number,
        ]
    );
    assert!(matches!(
        &arguments[2],
        TypeScriptExpr::RuntimeReference { name } if name == "_ssrg_show_intShow"
    ));

    let TypeScriptFunction::ConstFunction { body, .. } = &ir.functions[3];
    let TypeScriptExpr::CheckedResult { value, .. } = body else {
        panic!("{body:#?}");
    };
    let TypeScriptExpr::Sequence { statements, .. } = &**value else {
        panic!("{value:#?}");
    };
    let TypeScriptStatement::PureLet {
        type_ref,
        initializer: TypeScriptExpr::CheckedResult { value, .. },
        ..
    } = &statements[0]
    else {
        panic!("{statements:#?}");
    };
    assert!(
        matches!(
            &**value,
            TypeScriptExpr::Call {
                arguments,
                type_arguments,
                checked_callee_type: None,
                ..
            } if arguments.len() == 1 && type_arguments == &[
                TypeScriptType::String,
                TypeScriptType::Number,
                TypeScriptType::Number,
            ]
        ),
        "{value:#?}"
    );
    let TypeScriptType::Function { parameter, result } = type_ref else {
        panic!("{type_ref:#?}");
    };
    assert_eq!(**parameter, TypeScriptType::Number);
    let TypeScriptType::Function { parameter, result } = &**result else {
        panic!("{type_ref:#?}");
    };
    assert_eq!(**parameter, TypeScriptType::Unknown);
    let TypeScriptType::Function { parameter, result } = &**result else {
        panic!("{type_ref:#?}");
    };
    assert_eq!(**parameter, TypeScriptType::Number);
    assert_eq!(
        **result,
        TypeScriptType::Either {
            error: Box::new(TypeScriptType::String),
            value: Box::new(TypeScriptType::Number),
        }
    );
}
