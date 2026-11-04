use seseragi_semantics::{semantic_diagnostics, type_module, TypedDecl, TypedType};

#[test]
fn inferred_read_line_alias_keeps_the_stdin_effect_contract() {
    let source = "pub effect fn main = { let saved = readLine; saved () }\n";
    let diagnostics = semantic_diagnostics("saved-read-line.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    let typed = type_module("saved-read-line.ssrg", source);
    let TypedDecl::EffectFn { effect, .. } = &typed.declarations[0] else {
        panic!("{typed:#?}");
    };
    let TypedType::Record { closed, fields } = &effect.environment else {
        panic!("{effect:#?}");
    };
    assert!(*closed);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "stdin");
    assert!(!fields[0].optional);
    assert_eq!(fields[0].type_ref, primitive("Stdin"));
    assert_eq!(effect.failure, primitive("StdinError"));
    assert_eq!(
        effect.success,
        TypedType::Named {
            name: "Maybe".into(),
            arguments: vec![primitive("String")],
        }
    );
}

fn primitive(name: &str) -> TypedType {
    TypedType::ExternalNamed {
        name: name.into(),
        canonical: format!("std/prelude::{name}"),
        arguments: vec![],
    }
}

#[test]
fn read_line_values_keep_direct_annotated_and_inferred_calls() {
    for expression in [
        "readLine ()",
        "{ let saved = readLine; saved () }",
        "{ let saved = (readLine); let alias = saved; alias () }",
        "{ let saved = readLine; let unit = (); saved unit }",
        "{ let saved: Unit -> Effect<{stdin:Stdin},StdinError,Maybe<String>> = readLine; saved () }",
        "{ let saved = keep readLine; saved () }",
        "{ let (saved, _) = (readLine, ()); saved () }",
    ] {
        let source = format!(
            "fn keep<A> value: A -> A = value\npub effect fn main -> Maybe<String> with Stdin fails StdinError = {expression}\n"
        );
        let diagnostics = semantic_diagnostics("read-line-values.ssrg", &source);
        assert!(diagnostics.diagnostics.is_empty(), "{source}\n{diagnostics:#?}");
    }
}

#[test]
fn saved_read_line_rejects_wrong_arguments_and_effect_contracts() {
    for (source, message) in [
        ("pub effect fn main = { let saved = readLine; saved 1 }\n", "call.argument-type-mismatch"),
        ("pub effect fn main -> Maybe<String> fails StdinError = { let saved = readLine; saved () }\n", "effect.explicit-environment-mismatch"),
        ("pub effect fn main -> Maybe<String> with Stdin fails String = { let saved = readLine; saved () }\n", "effect.explicit-failure-mismatch"),
        ("pub effect fn main -> String with Stdin fails StdinError = { let saved = readLine; saved () }\n", "effect.explicit-success-mismatch"),
        ("fn wrong -> (Unit -> Effect<{},StdinError,Maybe<String>>) = readLine\n", "function.return-type-mismatch"),
        ("fn wrong -> (Unit -> Effect<{stdin:Stdin},String,Maybe<String>>) = readLine\n", "function.return-type-mismatch"),
        ("fn wrong -> (Unit -> Effect<{stdin:Stdin},StdinError,String>) = readLine\n", "function.return-type-mismatch"),
    ] {
        let diagnostics = semantic_diagnostics("wrong-read-line.ssrg", source);
        assert_eq!(diagnostics.diagnostics.len(), 1, "{source}\n{diagnostics:#?}");
        assert_eq!(diagnostics.diagnostics[0].message_key, message, "{source}\n{diagnostics:#?}");
    }
}

#[test]
fn user_read_line_values_keep_their_own_type_and_identity() {
    let source = "fn readLine value: Int -> Int = value\npub fn main -> Int = { let saved = readLine; saved 42 }\n";
    let diagnostics = semantic_diagnostics("own-read-line.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}
