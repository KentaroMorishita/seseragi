use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn standard_numeric_binary_operators_keep_canonical_owners_across_alias_spellings() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for alias in ["big", "renamed"] {
            for operator in ["+", "-", "*", "/", "%", "**"] {
                let right = if operator == "**" {
                    "Int".into()
                } else {
                    format!("{alias}.BigInt")
                };
                let source = format!("import * as {alias} from \"std/big-int\"\nalias Number = {alias}.BigInt\npub fn operation left:Number -> right:{right} -> Number = left {operator} right\n");
                compile_module(
                    CompileInput::new("big-雪🙂.ssrg", "proof/big", &source).with_profile(profile),
                )
                .unwrap();
                for expression in [
                    format!("({operator}) left"),
                    format!("{{ let saved = ({operator}) left; saved }}"),
                ] {
                    let source = format!("import * as {alias} from \"std/big-int\"\npub fn partial left:{alias}.BigInt -> ({right} -> {alias}.BigInt) = {expression}\n");
                    compile_module(
                        CompileInput::new("partial-雪🙂.ssrg", "proof/partial", &source)
                            .with_profile(profile),
                    )
                    .unwrap();
                }
            }
        }
        for operator in ["+", "-", "*"] {
            let source = format!("import * as renamed from \"std/decimal\"\nalias Number = renamed.Decimal\npub fn operation left:Number -> right:Number -> Number = left {operator} right\n");
            compile_module(
                CompileInput::new("decimal-雪🙂.ssrg", "proof/decimal", &source)
                    .with_profile(profile),
            )
            .unwrap();
        }
    }
}

#[test]
fn nominal_numeric_display_names_do_not_select_a_standard_arithmetic_owner() {
    for name in ["BigInt", "Decimal"] {
        let source = format!("newtype {name} = Int\npub fn operation left:{name} -> right:{name} -> {name} = left + right\n");
        assert!(compile_module(CompileInput::new(
            "nominal-雪🙂.ssrg",
            "proof/nominal",
            &source
        ))
        .is_err());
    }
}
