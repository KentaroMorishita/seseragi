mod bytes_ops;
mod collection_ops;
mod core;
mod display_ops;
mod effect_ops;
mod emit;
mod equality_ops;
mod int_ops;
mod iterator_ops;
mod json_ops;
mod list_ops;
mod numeric_ops;
mod prelude_ops;
mod provider_service_ops;
mod range_ops;
mod runtime_modules;
mod runtime_types;
mod signal_ops;
mod span;
mod standard_ops;
mod stream_ops;
mod sum_ops;
mod trait_method_ops;
mod typescript;
mod web_html_ops;

pub use core::{
    lower_typed_module, CoreAdt, CoreAdtVariant, CoreAlias, CoreBinding, CoreCallEvidence,
    CoreComprehensionClause, CoreDecisionBinding, CoreDecisionBranch, CoreDecisionProjection,
    CoreDecisionTest, CoreExpr, CoreForeignMember, CoreForeignModule, CoreFunction, CoreInstance,
    CoreInstanceConstraint, CoreInstanceEvidence, CoreInstanceImplementation, CoreInstanceMethod,
    CoreModule, CoreModuleDependency, CoreModuleImport, CoreMonadDoStatement, CoreParameter,
    CorePattern, CoreRecordField, CoreRecordPatternField, CoreRecordValueItem,
    CoreShowPayloadEvidence, CoreStatement, CoreStruct, CoreStructField, CoreTemplatePart,
    CoreTraitDispatch, CoreType,
};
pub use emit::{
    emit_typescript_module, emit_typescript_module_with_output_paths, GeneratedBundle,
    GeneratedInstance, GeneratedModule, GeneratedOutputPaths, GeneratedOutputs, GeneratedRuntime,
    SourceMap,
};
pub use runtime_modules::runtime_provided_modules;
pub(crate) use span::source_span;
pub use span::SourceSpan;
pub use typescript::{
    lower_core_module_to_typescript_ir, lower_core_module_to_typescript_ir_with_plan,
    TypeScriptAdt, TypeScriptAdtVariant, TypeScriptAlias, TypeScriptBinding,
    TypeScriptDecisionBinding, TypeScriptDecisionBranch, TypeScriptDecisionProjection,
    TypeScriptDecisionTest, TypeScriptDerivedShowField, TypeScriptDerivedShowPayload,
    TypeScriptDerivedShowVariant, TypeScriptExpr, TypeScriptForeignMember, TypeScriptForeignModule,
    TypeScriptForeignNamespace, TypeScriptForeignOpaqueType, TypeScriptForeignValue,
    TypeScriptFunction, TypeScriptImport, TypeScriptInstance, TypeScriptInstanceConstraint,
    TypeScriptInstanceImplementation, TypeScriptInstanceMethod, TypeScriptLoweringError,
    TypeScriptModule, TypeScriptOutputPlan, TypeScriptParameter, TypeScriptRecordTypeField,
    TypeScriptRecordValueItem, TypeScriptShowDictionaryReference, TypeScriptSourceImport,
    TypeScriptSourceImportBinding, TypeScriptStatement, TypeScriptStruct, TypeScriptType,
    TypeScriptTypeImport,
};

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_semantics::{type_module, TypedModule, TypedModuleDependency, TypedModuleImport};

    use seseragi_syntax::{ByteSpan, Visibility};

    #[test]
    fn lowers_public_let_to_core_binding() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);

        assert_eq!(core.stage, "core-ir");
        assert_eq!(core.module, "artifact/basic");
        assert_eq!(core.bindings.len(), 1);
        assert!(matches!(core.bindings[0].value, CoreExpr::Integer { .. }));
        assert!(core.functions.is_empty());
    }

    #[test]
    fn lowers_foreign_typescript_bindings_with_explicit_boundary_codecs() {
        let source = concat!(
            "foreign \"typescript\" from \"./host.mjs\" {\n",
            "  opaque type Handle\n",
            "  pure constructor fn make value: Int -> Handle = \"Handle\"\n",
            "  pure fn inspect value: Js.Nullable<String> -> Js.MutableArray<Int>\n",
            "  task fn load handle: Handle -> Js.Unknown\n",
            "}\n",
        );
        let typed = type_module("artifact/foreign/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let generated = emit_typescript_module(typescript, source);

        assert!(generated
            .typescript
            .contains("type Handle = object & { readonly [__ssrg$foreign$brand$Handle]: true };"));
        assert!(generated
            .typescript
            .contains("[{ nullable: \"string\" }], { mutableArray: \"int\" }"));
        assert!(generated
            .typescript
            .contains("[\"opaque\"], \"js-unknown\""));
        assert!(!generated.typescript.contains("\"unknown\""));
    }

    #[test]
    fn lowers_nominal_structs_to_branded_types_and_object_values() {
        let source = concat!(
            "pub struct User { name: String, score: Int }\n",
            "fn rename user: User -> User = User { ...user, name: \"Mio\" }\n",
            "pub fn answer -> String = (User { name: \"Aki\", score: 42 } |> rename).name\n",
        );
        let typed = type_module("artifact/struct-user/main.ssrg", source);
        let core = lower_typed_module(typed);

        assert_eq!(core.structs.len(), 1);
        assert_eq!(core.structs[0].name, "User");
        let typescript = lower_core_module_to_typescript_ir(core);
        assert_eq!(typescript.structs.len(), 1);
        assert_eq!(typescript.structs[0].name, "User");
        let generated = emit_typescript_module(typescript, source);
        assert!(generated
            .typescript
            .contains("declare const __ssrg$brand$User: unique symbol;"));
        assert!(generated.typescript.contains("export type User = {"));
        assert!(generated.typescript.contains("as unknown as User"));
    }

    #[test]
    fn lowers_structural_record_values_and_required_field_access() {
        let source = concat!(
            "fn profile name: String -> score: Int -> { name: String, score: Int } = { name, score }\n",
            "pub fn displayName user: { name: String } -> String = user.name\n",
            "pub fn answer -> String = displayName (profile \"Mio\" 42)\n",
        );
        let typed = type_module("artifact/record-profile/main.ssrg", source);
        let core = lower_typed_module(typed);

        assert!(matches!(core.functions[0].body, CoreExpr::Record { .. }));
        assert!(matches!(
            core.functions[1].body,
            CoreExpr::FieldAccess { ref field, .. } if field == "name"
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains(
            "const profile = (name: string) => (score: number) => ({ \"name\": name, \"score\": score } as const)"
        ), "{}", bundle.typescript);
        assert!(bundle.typescript.contains(
            "export const displayName = (user: { readonly \"name\": string }) => (user)[\"name\"]"
        ), "{}", bundle.typescript);
    }

    #[test]
    fn lowers_optional_record_field_access_to_maybe_presence() {
        let source = "pub fn optionalId user: { id?: String } -> Maybe<String> = user.id\n";
        let typed = type_module("artifact/record-optional/main.ssrg", source);
        let core = lower_typed_module(typed);

        assert!(matches!(
            core.functions[0].body,
            CoreExpr::OptionalFieldAccess { ref field, .. } if field == "id"
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);
        assert!(
            bundle
                .typescript
                .contains("Object.prototype.hasOwnProperty.call($ssrg_record, \"id\")"),
            "{}",
            bundle.typescript
        );
        assert!(
            bundle
                .typescript
                .contains("? _ssrg_maybe_Just($ssrg_record[\"id\"]) : _ssrg_maybe_Nothing"),
            "{}",
            bundle.typescript
        );
        assert!(bundle
            .metadata
            .runtime
            .requirements
            .contains(&"core.maybe.just".to_owned()));
        assert!(bundle
            .metadata
            .runtime
            .requirements
            .contains(&"core.maybe.nothing".to_owned()));
    }

    #[test]
    fn lowers_record_spread_and_late_field_override_in_source_order() {
        let source = concat!(
            "pub fn relabel base: { label: Int, name: String } -> label: String -> { label: String, name: String } =\n",
            "  { ...base, label }\n",
        );
        let typed = type_module("artifact/record-spread/main.ssrg", source);
        let core = lower_typed_module(typed);

        let CoreExpr::Record { items, .. } = &core.functions[0].body else {
            panic!("expected record value");
        };
        assert!(matches!(items[0], CoreRecordValueItem::Spread { .. }));
        assert!(matches!(
            items[1],
            CoreRecordValueItem::Field { ref name, .. } if name == "label"
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);
        assert!(
            bundle
                .typescript
                .contains("({ ...base, \"label\": label } as const)"),
            "{}",
            bundle.typescript
        );
    }

    #[test]
    fn preserves_linked_dependency_edges_and_canonical_imports_in_core_ir() {
        let source = "import { increment as next } from \"./domain\"\n";
        let typed = TypedModule {
            schema: 1,
            stage: "typed-hir".to_owned(),
            source: source.to_owned(),
            module: "fixture/game::main".to_owned(),
            foreign_modules: Vec::new(),
            external_type_bindings: Vec::new(),
            module_dependencies: vec![TypedModuleDependency {
                specifier: "./domain".to_owned(),
                module: "fixture/game::domain".to_owned(),
                origin: ByteSpan {
                    start: 0,
                    end: source.len(),
                },
                imports: vec![TypedModuleImport {
                    namespace: "value".to_owned(),
                    imported: "increment".to_owned(),
                    local: "next".to_owned(),
                    canonical: "fixture/game::domain::increment".to_owned(),
                    origin: ByteSpan { start: 22, end: 26 },
                }],
            }],
            instances: Vec::new(),
            declarations: Vec::new(),
        };

        let core = lower_typed_module(typed);

        assert_eq!(core.module_dependencies.len(), 1);
        assert_eq!(core.module_dependencies[0].specifier, "./domain");
        assert_eq!(core.module_dependencies[0].imports.len(), 1);
        assert_eq!(
            core.module_dependencies[0].imports[0].canonical,
            "fixture/game::domain::increment"
        );
        assert_eq!(core.module_dependencies[0].origin.source, source);
    }

    #[test]
    fn lowers_console_println_effect_operation() {
        let typed = type_module(
            "artifact/effect-main/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n",
        );
        let core = lower_typed_module(typed);

        assert!(core.bindings.is_empty());
        assert_eq!(core.functions.len(), 1);
        assert_eq!(core.functions[0].parameters[0].id, "unit");
        assert!(matches!(
            core.functions[0].body,
            CoreExpr::EffectOperation { .. }
        ));
    }

    #[test]
    fn lowers_core_binding_to_typescript_const() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.stage, "typescript-ir");
        assert_eq!(typescript.runtime_requirements, vec!["core.int"]);
        assert_eq!(typescript.bindings.len(), 1);
        assert!(typescript.functions.is_empty());
    }

    #[test]
    fn lowers_templates_through_show_dictionaries_and_string_concatenation() {
        let source = "pub type Badge deriving Show = | Active\n\
                      pub fn label name: String -> badge: Badge -> String = `Hello ${name}: ${badge}`\n";
        let typed = type_module("artifact/template-show/main.ssrg", source);
        let core = lower_typed_module(typed);
        assert!(matches!(
            &core.functions[0].body,
            CoreExpr::Template { parts, .. }
                if matches!(
                    &parts[3],
                    CoreTemplatePart::Interpolation {
                        evidence: Some(CoreCallEvidence {
                            evidence: CoreInstanceEvidence::Local { identity, .. },
                            ..
                        }),
                        ..
                    } if identity == "Show<artifact/template-show::Badge>"
                )
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        assert_eq!(
            typescript.runtime_requirements,
            vec![
                "core.adt",
                "core.show.dictionary",
                "core.show.bounded",
                "core.string",
                "core.string.show"
            ]
        );
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                body: TypeScriptExpr::Binary { .. },
                ..
            }
        ));

        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle
            .typescript
            .contains("_ssrg_show_stringShow[\"show\"](name)"));
        assert!(bundle
            .typescript
            .contains("__ssrg$instance$Show$0[\"show\"](badge)"));
        assert!(bundle.typescript.contains("\"Hello \" +"));
    }

    #[test]
    fn lowers_conditional_collection_display_factories_with_nested_evidence() {
        let source = "pub fn render values: Array<Maybe<String>> -> String = show values\n\
                      pub fn inspect value: Either<String, List<Bool>> -> String = debug value\n";
        let typed = type_module("artifact/collection-display/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        for requirement in [
            "core.array.show",
            "core.maybe.show",
            "core.string.show",
            "core.either.debug",
            "core.list.debug",
            "core.string.debug",
            "core.bool.debug",
        ] {
            assert!(
                typescript
                    .runtime_requirements
                    .iter()
                    .any(|actual| actual == requirement),
                "missing runtime requirement {requirement}"
            );
        }

        let bundle = emit_typescript_module(typescript, source);
        for factory in [
            "_ssrg_show_arrayShow",
            "_ssrg_show_maybeShow",
            "_ssrg_debug_eitherDebug",
            "_ssrg_debug_listDebug",
        ] {
            assert!(
                bundle.typescript.contains(factory),
                "missing {factory} in {}",
                bundle.typescript
            );
        }
        assert!(bundle.typescript.contains("_ssrg_show_stringShow"));
        assert!(bundle.typescript.contains("_ssrg_debug_stringDebug"));
        assert!(bundle.typescript.contains("_ssrg_debug_boolDebug"));
    }

    #[test]
    fn lowers_the_complete_primitive_display_matrix() {
        let source = "pub fn inspectInt value: Int -> String = debug value\n\
                      pub fn renderFloat value: Float -> String = show value\n\
                      pub fn inspectFloat value: Float -> String = debug value\n\
                      pub fn renderNever value: Never -> String = show value\n\
                      pub fn inspectNever value: Never -> String = debug value\n";
        let typed = type_module("artifact/primitive-display/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        for requirement in [
            "core.int.debug",
            "core.float64.show",
            "core.float64.debug",
            "core.never.show",
            "core.never.debug",
        ] {
            assert!(
                typescript
                    .runtime_requirements
                    .iter()
                    .any(|actual| actual == requirement),
                "missing runtime requirement {requirement}"
            );
        }

        let bundle = emit_typescript_module(typescript, source);
        for dictionary in [
            "_ssrg_debug_intDebug",
            "_ssrg_show_floatShow",
            "_ssrg_debug_floatDebug",
            "_ssrg_show_neverShow",
            "_ssrg_debug_neverDebug",
        ] {
            assert!(
                bundle.typescript.contains(dictionary),
                "missing {dictionary} in {}",
                bundle.typescript
            );
        }
        assert!(bundle.typescript.contains("renderFloat = (value: number)"));
        assert!(bundle.typescript.contains("renderNever = (value: never)"));
    }

    #[test]
    fn lowers_non_expanding_range_display_with_nested_evidence() {
        let source = "pub fn render value: Range<Int> -> String = show value\n\
                      pub fn inspect value: Range<Int> -> String = debug value\n\
                      pub fn inspectMany values: Array<Range<Int>> -> String = debug values\n";
        let typed = type_module("artifact/range-display/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        for requirement in [
            "core.range.show",
            "core.range.debug",
            "core.int.show",
            "core.int.debug",
            "core.array.debug",
        ] {
            assert!(
                typescript
                    .runtime_requirements
                    .iter()
                    .any(|actual| actual == requirement),
                "missing runtime requirement {requirement}"
            );
        }

        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains("_ssrg_show_rangeShow<number>"));
        assert!(bundle.typescript.contains("_ssrg_debug_rangeDebug<number>"));
        assert!(bundle.typescript.contains("_ssrg_debug_arrayDebug"));
    }

    #[test]
    fn lowers_structural_tuple_and_record_display_from_compiler_descriptors() {
        let source = "pub fn render value: (Int, String) -> String = show value\n\
                      pub fn inspect value: { zeta?: String, alpha: Int } -> String = debug value\n\
                      pub fn inspectNested value: { pairs: Array<(Int, Range<Int>)> } -> String = debug value\n";
        let typed = type_module("artifact/structural-display/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        for requirement in [
            "core.tuple.show",
            "core.record.debug",
            "core.tuple.debug",
            "core.array.debug",
            "core.range.debug",
            "core.int.show",
            "core.string.show",
            "core.int.debug",
            "core.string.debug",
        ] {
            assert!(
                typescript
                    .runtime_requirements
                    .iter()
                    .any(|actual| actual == requirement),
                "missing runtime requirement {requirement}"
            );
        }

        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains("_ssrg_show_tupleShow"));
        assert!(bundle.typescript.contains("_ssrg_debug_recordDebug"));
        assert!(bundle.typescript.contains("[\"alpha\", \"zeta\"] as const"));
        assert!(bundle.typescript.contains("[false, true] as const"));
        assert!(!bundle.typescript.contains("Object.keys"));
        assert!(!bundle.typescript.contains("JSON.stringify"));
    }

    #[test]
    fn lowers_adt_constructors_to_tagged_typescript_values() {
        let source = "\
pub type Hand =
  | Rock
  | Paper
  | Scissors

pub type Label =
  | Missing
  | Present String

pub let opening: Hand = Rock

pub fn wrap value: String -> Label =
  Present value
";
        let typed = type_module("artifact/adt-constructors/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.adts.len(), 2);
        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.adt", "core.string"]
        );
        assert_eq!(typescript.adts[0].name, "Hand");
        assert_eq!(
            typescript.adts[1].variants[1].payload,
            Some(TypeScriptType::String)
        );
        assert!(typescript.imports.is_empty());

        let bundle = emit_typescript_module(typescript, source);
        assert_eq!(
            bundle.metadata.exports,
            vec!["Rock", "Paper", "Scissors", "Missing", "Present", "opening", "wrap"]
        );
        assert_eq!(
            bundle.typescript,
            "\
export type Hand =
  | { readonly tag: \"Rock\" }
  | { readonly tag: \"Paper\" }
  | { readonly tag: \"Scissors\" };
export const Rock: Hand = { tag: \"Rock\" } as const;
export const Paper: Hand = { tag: \"Paper\" } as const;
export const Scissors: Hand = { tag: \"Scissors\" } as const;
export type Label =
  | { readonly tag: \"Missing\" }
  | { readonly tag: \"Present\"; readonly value: string };
export const Missing: Label = { tag: \"Missing\" } as const;
export const Present = (value: string): Label => ({ tag: \"Present\", value } as const);
export const wrap = (value: string) => Present(value)
export const opening: Hand = Rock;
"
        );
        assert_eq!(
            bundle.source_map.names,
            vec![
                "Hand", "Rock", "Paper", "Scissors", "Label", "Missing", "Present", "wrap",
                "Present", "opening"
            ]
        );
        assert_eq!(
            bundle.source_map.mappings,
            "AAAAA;;;;AACIC;AACAC;AACAC;AAEJC;;;AACIC;AACAC;AAIJC;AAFAE"
        );
    }

    #[test]
    fn keeps_opaque_and_private_adt_constructors_out_of_runtime_exports() {
        let source = "\
pub opaque type Token =
  | Token String

type Internal =
  | Hidden
";
        let typed = type_module("artifact/opaque-adts/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains("export type Token ="));
        assert!(bundle
            .typescript
            .contains("const Token = (value: string): Token =>"));
        assert!(!bundle.typescript.contains("export const Token"));
        assert!(bundle.typescript.contains("type Internal ="));
        assert!(bundle
            .typescript
            .contains("const Hidden: Internal = { tag: \"Hidden\" } as const;"));
        assert!(bundle.metadata.exports.is_empty());
    }

    #[test]
    fn lowers_core_effect_to_typescript_imported_call() {
        let typed = type_module(
            "artifact/effect-main/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n",
        );
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.unit", "effect.console.println", "core.string"]
        );
        assert_eq!(typescript.imports[0].local, "_ssrg_console_println");
        assert_eq!(typescript.functions.len(), 1);
    }

    #[test]
    fn lowers_string_binding_to_typescript_string_const() {
        let source = "pub let greeting: String = \"hello\"\n";
        let typed = type_module("artifact/string-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.string"]);
        assert_eq!(
            bundle.typescript,
            "export const greeting: string = \"hello\";\n"
        );
    }

    #[test]
    fn lowers_boolean_binding_to_typescript_boolean_const() {
        let source = "pub let enabled: Bool = True\n";
        let typed = type_module("artifact/bool-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.bool"]);
        assert_eq!(bundle.typescript, "export const enabled: boolean = true;\n");
    }

    #[test]
    fn lowers_identity_function_to_typescript_arrow_function() {
        let source = "pub fn identity value: Int -> Int = value\n";
        let typed = type_module("artifact/identity-fn/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.int"]);
        assert_eq!(bundle.metadata.exports, vec!["identity"]);
        assert_eq!(
            bundle.typescript,
            "export const identity = (value: number) => value\n"
        );
    }

    #[test]
    fn preserves_generic_function_binders_through_typescript_emission() {
        let source = "pub fn first<A, B> left: A -> right: B -> A = left\n";
        let typed = type_module("artifact/generic-first/main.ssrg", source);
        let core = lower_typed_module(typed);

        assert_eq!(core.functions[0].type_parameters, vec!["A", "B"]);

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                type_parameters,
                parameters,
                ..
            } if type_parameters == &["A", "B"]
                && parameters[0].type_name == "A"
                && parameters[1].type_name == "B"
        ));

        let bundle = emit_typescript_module(typescript, source);
        assert_eq!(
            bundle.typescript,
            "export const first = <A, B,>(left: A) => (right: B) => left\n"
        );
    }

    #[test]
    fn preserves_higher_kinded_binders_and_constraint_identity_in_every_ir() {
        let source = "\
pub trait Lift<F<_>> {
  fn lift<A> value: A -> F<A>
}

pub fn liftValue<F<_>, A> value: A -> F<A>
where Lift<F> =
  lift value
";
        let typed = type_module("artifact/hkt-constraint/main.ssrg", source);
        let core = lower_typed_module(typed);
        let function = &core.functions[0];

        assert_eq!(
            function.type_parameters,
            vec![
                seseragi_syntax::TypeParameter::constructor("F", 1),
                seseragi_syntax::TypeParameter::value("A"),
            ]
        );
        assert_eq!(
            function.constraints[0].trait_identity.as_deref(),
            Some("artifact/hkt-constraint::trait(Lift)")
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                type_parameters,
                constraints,
                ..
            } if type_parameters[0].arity == 1
                && type_parameters[0].name == "F"
                && constraints[0].trait_identity.as_deref()
                    == Some("artifact/hkt-constraint::trait(Lift)")
        ));
    }

    #[test]
    fn preserves_higher_kinded_alias_binders_in_core_and_typescript_ir() {
        let source = "pub alias StateT<S, M<_>, A> = S -> M<(A, S)>\n";
        let core = lower_typed_module(type_module("artifact/hkt-alias/main.ssrg", source));

        assert_eq!(core.aliases.len(), 1);
        assert_eq!(
            core.aliases[0].type_parameters,
            vec![
                seseragi_syntax::TypeParameter::value("S"),
                seseragi_syntax::TypeParameter::constructor("M", 1),
                seseragi_syntax::TypeParameter::value("A"),
            ]
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        assert_eq!(typescript.aliases.len(), 1);
        assert_eq!(typescript.aliases[0].type_parameters[1].name, "M");
        assert_eq!(typescript.aliases[0].type_parameters[1].arity, 1);
    }

    #[test]
    fn preserves_local_higher_kinded_constraint_metadata() {
        let source = "\
pub trait Lift<F<_>> {
  fn lift<A> value: A -> F<A>
}

pub fn outer<F<_>, A> value: A -> F<A>
where Lift<F> = {
  fn local<G<_>, B> item: B -> G<B>
  where Lift<G> =
    lift item

  local value
}
";
        let core = lower_typed_module(type_module(
            "artifact/local-hkt-constraint/main.ssrg",
            source,
        ));
        assert!(matches!(
            &core.functions[0].body,
            CoreExpr::Sequence { statements, .. }
                if matches!(
                    statements.as_slice(),
                    [CoreStatement::LocalFunction {
                        type_parameters,
                        constraints,
                        ..
                    }] if type_parameters[0].arity == 1
                        && constraints[0].trait_identity.as_deref()
                            == Some("artifact/local-hkt-constraint::trait(Lift)")
                )
        ));
    }

    #[test]
    fn lowers_multi_parameter_function_to_typescript_arrow_function() {
        let source = "pub fn first left: Int -> right: Int -> Int = left\n";
        let typed = type_module("artifact/first-fn/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.int"]);
        assert_eq!(
            bundle.typescript,
            "export const first = (left: number) => (right: number) => left\n"
        );
    }

    #[test]
    fn lowers_direct_self_tail_calls_to_a_constant_stack_loop() {
        let source = "\
pub fn sum current: Int -> total: Int -> Int =
  if current == 0 then total else sum (current - 1) (total + current)
";
        let typed = type_module("artifact/self-tail-loop/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains("const $ssrg$tail = Symbol();"));
        assert!(bundle.typescript.contains("while (true)"));
        assert!(bundle.typescript.contains(
            "({ [$ssrg$tail]: [_ssrg_int_subtract(current, 1), _ssrg_int_add(total, current)] } as never)"
        ));
        assert!(bundle
            .typescript
            .contains("current = $ssrg$arguments[0]; total = $ssrg$arguments[1]; continue;"));
    }

    #[test]
    fn does_not_apply_pure_tail_call_loop_to_effect_functions() {
        let source = "\
effect fn count remaining: Int -> Unit =
  do {
    current <- succeed remaining
    match current {
      0 -> succeed ()
      _ -> count (remaining - 1)
    }
  }
";
        let typed = type_module("artifact/effect-tail-loop/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_effect: true,
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);
        assert!(
            !bundle.typescript.contains("while (true)"),
            "{}",
            bundle.typescript
        );
        assert!(bundle
            .typescript
            .contains("count(_ssrg_int_subtract(remaining, 1))"));
    }

    #[test]
    fn lowers_local_match_tail_calls_but_leaves_non_tail_calls_recursive() {
        let source = "\
fn fibonacci current: Int -> Int =
  if current <= 1 then current else fibonacci (current - 1) + fibonacci (current - 2)

pub fn listLength values: List<Int> -> Int = {
  fn loop remaining: List<Int> -> total: Int -> Int =
    match remaining {
      `[] -> total
      `[_, ...rest] -> loop rest (total + 1)
    }

  loop values 0
}
";
        let typed = type_module("artifact/local-match-tail-loop/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle
                .typescript
                .matches("const $ssrg$tail = Symbol();")
                .count(),
            1
        );
        assert!(bundle
            .typescript
            .contains("fibonacci(_ssrg_int_subtract(current, 1))"));
        assert!(bundle
            .typescript
            .contains("({ [$ssrg$tail]: [rest, _ssrg_int_add(total, 1)] } as never)"));
    }

    #[test]
    fn lowers_integer_add_function_to_checked_runtime_call() {
        let source = "pub fn add x: Int -> y: Int -> Int = x + y\n";
        let typed = type_module("artifact/add-fn/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.metadata.runtime.requirements,
            vec!["core.int", "core.int.add"]
        );
        assert_eq!(
            bundle.typescript,
            "import { add as _ssrg_int_add } from \"@seseragi/runtime/int\"\n\nexport const add = (x: number) => (y: number) => _ssrg_int_add(x, y)\n"
        );
    }

    #[test]
    fn lowers_direct_standard_trait_methods_through_their_operator_abi() {
        let source = "pub let addTwenty: Int -> Int = add 20\n\
                      pub fn values unit: Unit -> (Bool, Int, Float, String) =\n\
                        (eq 21 21, addTwenty 22, mul 6.0 7.0, add \"sese\" \"ragi\")\n";
        let typed = type_module("artifact/direct-trait-methods/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .typescript
            .contains("export const addTwenty: (argument: number) => number = (_argument1) => _ssrg_int_add(20, _argument1)"));
        assert!(bundle
            .typescript
            .contains("[21 === 21, addTwenty(22), 6.0 * 7.0, \"sese\" + \"ragi\"] as const"));
        assert!(bundle
            .metadata
            .runtime
            .requirements
            .contains(&"core.int.add".to_owned()));
        assert!(!bundle
            .metadata
            .runtime
            .requirements
            .contains(&"core.int.add-dictionary".to_owned()));
    }

    #[test]
    fn freshens_runtime_import_that_collides_with_user_function() {
        let source = "pub fn _ssrg_int_add value: Int -> Int = value\npub fn add x: Int -> y: Int -> Int = x + y\n";
        let typed = type_module("artifact/runtime-name-collision/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .typescript
            .contains("import { add as _ssrg_int_add_1 } from \"@seseragi/runtime/int\""));
        assert!(bundle
            .typescript
            .contains("export const _ssrg_int_add = (value: number) => value"));
        assert!(bundle
            .typescript
            .contains("export const add = (x: number) => (y: number) => _ssrg_int_add_1(x, y)"));
    }

    #[test]
    fn freshens_a_conditional_dictionary_factory_call() {
        let source = "pub fn _ssrg_show_arrayShow value: String -> String = value\n\
             pub fn render values: Array<String> -> String = show values\n";
        let typed = type_module("artifact/display-factory-name-collision/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(
            bundle
                .typescript
                .contains("arrayShow as _ssrg_show_arrayShow_1"),
            "{}",
            bundle.typescript
        );
        assert!(bundle
            .typescript
            .contains("_ssrg_show_arrayShow_1<string>(_ssrg_show_stringShow)[\"show\"](values)"));
    }

    #[test]
    fn lowers_module_qualified_pure_call_without_runtime_helper_import() {
        let source = "pub fn invoke value: Int -> Int = default value\n";
        let origin = SourceSpan {
            source: "main.ssrg".to_owned(),
            start: 0,
            end: source.len(),
        };
        let int_type = CoreType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        };
        let core = CoreModule {
            foreign_modules: Vec::new(),
            schema: 1,
            stage: "core-ir".to_owned(),
            module: "artifact/calls".to_owned(),
            external_type_bindings: Vec::new(),
            module_dependencies: Vec::new(),
            adts: Vec::new(),
            aliases: Vec::new(),
            structs: Vec::new(),
            instances: Vec::new(),
            bindings: Vec::new(),
            functions: vec![CoreFunction {
                symbol: "artifact/calls::invoke".to_owned(),
                visibility: Visibility::Public,
                origin: origin.clone(),
                is_effect: false,
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                parameters: vec![CoreParameter {
                    id: "value".to_owned(),
                    kind: "named".to_owned(),
                    type_ref: int_type.clone(),
                }],
                body: CoreExpr::Call {
                    callee: "artifact/calls::default".to_owned(),
                    arguments: vec![CoreExpr::Variable {
                        name: "value".to_owned(),
                        evidence: Vec::new(),
                        type_ref: int_type.clone(),
                        origin: origin.clone(),
                    }],
                    evidence: Vec::new(),
                    deferred_evidence_parameters: Vec::new(),
                    deferred_evidence_type_constructor_parameters: Vec::new(),
                    trait_dispatch: None,
                    type_ref: int_type,
                    origin,
                },
            }],
        };

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript.clone(), source);

        assert_eq!(typescript.runtime_requirements, vec!["core.int"]);
        assert!(typescript
            .runtime_requirements
            .iter()
            .all(|requirement| !requirement.starts_with("effect.")));
        assert!(typescript.imports.is_empty());
        assert_eq!(
            bundle.typescript,
            "export const invoke = (value: number) => _default(value)\n"
        );
        assert_eq!(bundle.source_map.names, vec!["invoke", "_default"]);
    }

    #[test]
    fn lowers_typed_pure_function_call_without_runtime_helper_import() {
        let source = "\
pub fn identity value: Int -> Int = value
pub fn useIdentity value: Int -> Int = identity value
";
        let typed = type_module("artifact/calls/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            ..
        } = &core.functions[1].body
        else {
            panic!("expected pure call in second function body");
        };

        assert_eq!(callee, "artifact/calls::identity");
        assert_eq!(arguments.len(), 1);
        assert_eq!(
            type_ref,
            &CoreType::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
            }
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript.clone(), source);

        assert_eq!(typescript.runtime_requirements, vec!["core.int"]);
        assert!(typescript
            .runtime_requirements
            .iter()
            .all(|requirement| !requirement.starts_with("effect.")));
        assert!(typescript.imports.is_empty());
        assert_eq!(
            bundle.typescript,
            "export const identity = (value: number) => value\nexport const useIdentity = (value: number) => identity(value)\n"
        );
        assert_eq!(
            bundle.source_map.names,
            vec!["identity", "useIdentity", "identity"]
        );
    }

    #[test]
    fn lowers_partial_application_to_curried_typescript_call() {
        let source = "pub fn add left: Int -> right: Int -> Int = left + right\npub fn addTo value: Int -> (Int -> Int) = add value\n";
        let typed = type_module("artifact/partial-call/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .metadata
            .runtime
            .requirements
            .iter()
            .all(|requirement| !requirement.starts_with("effect.")));
        assert!(bundle.typescript.contains(
            "export const add = (left: number) => (right: number) => _ssrg_int_add(left, right)"
        ));
        assert!(bundle
            .typescript
            .contains("export const addTo = (value: number) => add(value)"));
    }

    #[test]
    fn deduplicates_runtime_helper_imports_across_functions() {
        let source = "\
pub effect fn first -> Unit
with Console
fails ConsoleError =
  println \"one\"

pub effect fn second -> Unit
with Console
fails ConsoleError =
  println \"two\"
";
        let typed = type_module("artifact/two-effects/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.imports.len(), 1);
        assert_eq!(typescript.imports[0].feature, "effect.console.println");
    }

    #[test]
    fn sanitizes_typescript_parameter_and_variable_names() {
        let source = "pub fn pick default: Int -> Int = default\n";
        let typed = type_module("artifact/reserved-param/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.typescript,
            "export const pick = (_default: number) => _default\n"
        );
    }

    #[test]
    fn lowers_top_level_pattern_bindings_from_one_evaluation() {
        let source = concat!(
            "fn makePair unit: Unit -> (Int, Int) = (1, 2)\n",
            "let (left, right) = makePair ()\n",
            "pub let result: Int = left + right\n",
        );
        let typed = type_module("artifact/pattern-top/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.typescript.matches("makePair(undefined)").count(), 1);
        assert!(bundle.typescript.contains("const left: number"));
        assert!(bundle.typescript.contains("const right: number"));
        assert!(bundle.typescript.contains("export const result: number"));
    }

    #[test]
    fn lowers_block_and_do_pattern_bindings_to_projection_statements() {
        let source = concat!(
            "pub fn addPair pair: (Int, Int) -> Int = {\n",
            "  let (left, right) = pair\n",
            "  left + right\n",
            "}\n",
            "pub effect fn main = do {\n",
            "  let (left, right) = (1, 2)\n",
            "  succeed (left + right)\n",
            "}\n",
        );
        let typed = type_module("artifact/pattern-local/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.matches("const left: number").count() >= 2);
        assert!(bundle.typescript.matches("const right: number").count() >= 2);
        assert!(bundle.typescript.contains("__ssrg$pattern$"));
    }

    #[test]
    fn lowers_succeed_final_do_result_to_cold_effect() {
        let source = "pub effect fn main -> Unit =\n  do { succeed () }\n";
        let typed = type_module("artifact/effect-do/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.unit", "effect.core.succeed"]
        );
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                body: TypeScriptExpr::RuntimeCall { callee, .. },
                ..
            } if callee == "_ssrg_effect_succeed"
        ));
    }

    #[test]
    fn lowers_succeed_value_with_its_concrete_success_type() {
        let source = "pub effect fn ready = succeed \"ready\"\n";
        let typed = type_module("artifact/effect-succeed-value/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::EffectOperation {
            success, arguments, ..
        } = &core.functions[0].body
        else {
            panic!("expected effect operation");
        };
        assert_eq!(
            success,
            &CoreType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert!(
            matches!(arguments.as_slice(), [CoreExpr::String { value, .. }] if value == "ready")
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle
            .typescript
            .contains("_ssrg_effect_succeed(\"ready\")"));
    }

    #[test]
    fn lowers_adt_failure_to_a_cold_runtime_effect() {
        let source = "pub type AppError = | Invalid\npub effect fn reject = fail Invalid\n";
        let typed = type_module("artifact/effect-fail-adt/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::EffectOperation {
            failure, success, ..
        } = &core.functions[0].body
        else {
            panic!("expected effect operation");
        };
        assert_eq!(
            failure,
            &CoreType::Named {
                name: "AppError".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert_eq!(
            success,
            &CoreType::Named {
                name: "Never".to_owned(),
                arguments: Vec::new(),
            }
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(typescript
            .runtime_requirements
            .iter()
            .any(|requirement| requirement == "effect.core.fail"));
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains("_ssrg_effect_fail(Invalid)"));
        assert!(!bundle.typescript.contains("throw"));
        assert!(!bundle.typescript.contains("await"));
    }

    #[test]
    fn lowers_adt_failure_mapping_to_nested_cold_runtime_calls() {
        let source = "pub type HandInputError = | UnknownHand String\npub type AppError = | InvalidHand HandInputError\npub effect fn rejectUnknownHand input: String = mapError InvalidHand (fail (UnknownHand input))\n";
        let typed = type_module("artifact/effect-map-error-adt/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::EffectOperation {
            operation,
            failure,
            success,
            arguments,
            ..
        } = &core.functions[0].body
        else {
            panic!("expected mapped effect operation");
        };
        assert_eq!(operation, "effect.mapError");
        assert!(matches!(failure, CoreType::Named { name, .. } if name == "AppError"));
        assert!(matches!(success, CoreType::Named { name, .. } if name == "Never"));
        assert!(matches!(
            arguments.as_slice(),
            [CoreExpr::Variable { name: mapper, .. }, CoreExpr::EffectOperation { operation: source, .. }]
                if mapper == "artifact/effect-map-error-adt::InvalidHand"
                    && source == "effect.fail"
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(typescript
            .runtime_requirements
            .iter()
            .any(|requirement| requirement == "effect.core.mapError"));
        assert!(typescript
            .runtime_requirements
            .iter()
            .any(|requirement| requirement == "effect.core.fail"));
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle
            .typescript
            .contains("_ssrg_effect_mapError(InvalidHand, _ssrg_effect_fail(UnknownHand(input)))"));
        assert!(!bundle.typescript.contains("throw"));
        assert!(!bundle.typescript.contains("await"));
    }

    #[test]
    fn lowers_single_sync_effect_as_do_result() {
        let source =
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { println \"hello\" }\n";
        let typed = type_module("artifact/effect-do-println/main.ssrg", source);
        let core = lower_typed_module(typed);
        assert!(matches!(
            &core.functions[0].body,
            CoreExpr::EffectOperation { operation, success, .. }
                if operation == "console.println" && success == &CoreType::Named { name: "Unit".to_owned(), arguments: Vec::new() }
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.unit", "effect.console.println", "core.string"]
        );
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                body: TypeScriptExpr::RuntimeCall { .. },
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => _ssrg_console_println(\"hello\")"
        ));
    }

    #[test]
    fn lowers_single_async_operation_as_cold_value_producing_effect() {
        let source = "pub effect fn main =\n  do { readLine () }\n";
        let typed = type_module("artifact/effect-do-read-line/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::EffectOperation { success, .. } = &core.functions[0].body else {
            panic!("expected readLine do result");
        };
        assert_eq!(
            success,
            &CoreType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }],
            }
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        assert_eq!(
            typescript.runtime_requirements,
            vec![
                "core.unit",
                "effect.stdin.readLine",
                "core.maybe",
                "core.string"
            ]
        );
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                body: TypeScriptExpr::RuntimeCall { .. },
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .typescript
            .contains("export const main = (_unit: undefined) => _ssrg_stdin_readLine()"));
    }

    #[test]
    fn lowers_multiple_effect_do_statements_to_typescript_sequence() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {\n    println \"one\"\n    println \"two\"\n  }\n";
        let typed = type_module("artifact/effect-do-multiple/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(\"one\"), () => _ssrg_console_println(\"two\"))"
        ));
    }

    #[test]
    fn lowers_pure_do_let_without_flat_map() {
        let source =
            "pub effect fn main =\n  do {\n    let message = \"hello\"\n    println message\n  }\n";
        let typed = type_module("artifact/effect-do-pure-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::Sequence { statements, .. } = &core.functions[0].body else {
            panic!("expected do sequence");
        };
        assert!(matches!(
            statements.as_slice(),
            [CoreStatement::PureLet {
                name,
                value: CoreExpr::String { value, .. },
                ..
            }] if name == "message" && value == "hello"
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(typescript
            .runtime_requirements
            .iter()
            .all(|requirement| requirement != "effect.core.flatMap"));
        assert!(typescript
            .imports
            .iter()
            .all(|import| import.feature != "effect.core.flatMap"));
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains(
            "(() => { const message: string = \"hello\"; return _ssrg_console_println(message); })()"
        ));
    }

    #[test]
    fn keeps_pure_do_let_inside_the_preceding_effect_continuation() {
        let source = "pub effect fn main =\n  do {\n    line <- readLine ()\n    let copy = line\n    succeed copy\n  }\n";
        let typed = type_module("artifact/effect-do-bind-pure-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert!(typescript
            .runtime_requirements
            .iter()
            .any(|requirement| requirement == "effect.core.flatMap"));
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains(
            "_ssrg_effect_flatMap(_ssrg_stdin_readLine(), (line: { readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: string }) => (() => { const copy: { readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: string } = line; return _ssrg_effect_succeed(copy); })())"
        ));
    }

    #[test]
    fn lowers_do_bind_statement_to_typescript_const() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {\n    ignored <- print \"hello\"\n    println \"done\"\n  }\n";
        let typed = type_module("artifact/effect-do-bind/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "_ssrg_effect_flatMap(_ssrg_console_print(\"hello\"), (ignored: undefined) => _ssrg_console_println(\"done\"))"
        ));
    }

    #[test]
    fn lowers_async_stdin_bind_to_cold_flat_map_chain() {
        let source =
            "pub effect fn main =\n  do {\n    first <- readLine ()\n    second <- readLine ()\n    succeed ()\n  }\n";
        let typed = type_module("artifact/effect-stdin-read-line/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::Sequence { statements, .. } = &core.functions[0].body else {
            panic!("expected do block sequence");
        };
        let CoreStatement::Bind {
            value:
                CoreExpr::EffectOperation {
                    requirements,
                    failure,
                    success,
                    ..
                },
            ..
        } = &statements[0]
        else {
            panic!("expected readLine bind");
        };
        assert_eq!(
            requirements,
            &CoreType::Record {
                closed: true,
                fields: vec![CoreRecordField {
                    name: "stdin".to_owned(),
                    optional: false,
                    type_ref: CoreType::Named {
                        name: "Stdin".to_owned(),
                        arguments: Vec::new(),
                    },
                }],
            }
        );
        assert_eq!(
            failure,
            &CoreType::Named {
                name: "StdinError".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert_eq!(
            success,
            &CoreType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }],
            }
        );
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.metadata.runtime.requirements,
            vec![
                "core.unit",
                "effect.core.flatMap",
                "effect.stdin.readLine",
                "core.maybe",
                "core.string",
                "effect.core.succeed"
            ]
        );
        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_stdin_readLine(), (first: { readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: string }) => _ssrg_effect_flatMap(_ssrg_stdin_readLine(), (second: { readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: string }) => _ssrg_effect_succeed(undefined)))"
        ));
    }

    #[test]
    fn emits_basic_typescript_module() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, "pub let answer: Int = 42\n");

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.int"]);
        assert_eq!(bundle.metadata.exports, vec!["answer"]);
        assert_eq!(bundle.metadata.outputs.typescript, "main.ts");
        assert_eq!(bundle.metadata.outputs.source_map, "main.ts.map");
        assert_eq!(bundle.typescript, "export const answer: number = 42;\n");
        assert_eq!(bundle.source_map.file, "main.ts");
        assert_eq!(bundle.source_map.names, vec!["answer"]);
        assert_eq!(bundle.source_map.mappings, "AAAAA");
    }

    #[test]
    fn emits_module_functions_before_top_level_value_initializers() {
        let source = "fn make value: Int -> Maybe<Int> = Just value\n\
                      let first: Maybe<Int> = make 42\n\
                      pub let second: Maybe<Int> = first\n";
        let typed = type_module("artifact/top-level-initialization/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        let function = bundle
            .typescript
            .find("const make =")
            .expect("generated function");
        let initializer = bundle
            .typescript
            .find("const first:")
            .expect("generated first top-level initializer");
        let dependent = bundle
            .typescript
            .find("export const second:")
            .expect("generated dependent top-level initializer");
        assert!(
            function < initializer,
            "function must be initialized before its caller:\n{}",
            bundle.typescript
        );
        assert!(
            initializer < dependent,
            "top-level values must retain source order:\n{}",
            bundle.typescript
        );
    }

    #[test]
    fn lowers_float_literals_to_typescript_numbers_without_recovery_holes() {
        let source = "pub let values: Array<Float> = [1.0, 2.3, -0.0, 6.022e23]\n";
        let typed = type_module("artifact/float-literal/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::Array { elements, .. } = &core.bindings[0].value else {
            panic!("expected Core Float array");
        };
        assert!(matches!(
            elements.as_slice(),
            [
                CoreExpr::Float64 { value: first, .. },
                CoreExpr::Float64 { value: second, .. },
                CoreExpr::Unary {
                    operator,
                    operand,
                    type_ref: CoreType::Named {
                        name,
                        arguments,
                    },
                    ..
                },
                CoreExpr::Float64 { value: exponent, .. }
            ] if first == "1.0"
                && second == "2.3"
                && operator == "-"
                && matches!(operand.as_ref(), CoreExpr::Float64 { value, .. } if value == "0.0")
                && name == "Float"
                && arguments.is_empty()
                && exponent == "6.022e23"
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.float64"]);
        assert!(bundle.typescript.contains("[1.0, 2.3, -(0.0), 6.022e23]"));
        assert!(!bundle.typescript.contains(" = _"));
    }

    #[test]
    fn lowers_unary_values_through_formal_ir_and_checked_int_negation() {
        let source = "pub struct Snapshot {\n  number: Int,\n  ratio: Float,\n  flag: Bool,\n}\n\npub let negative = -2\npub let negativeZero = -0.0\npub let inverted = !True\npub let values: Array<Int> = [-1, -2, -3]\npub let floats: Array<Float> = [-1.0, -0.0, -6.022e23]\npub let flags: Array<Bool> = [!True, !False]\npub let snapshot: Snapshot = Snapshot { number: -2, ratio: -0.0, flag: !True }\n";
        let typed = type_module("artifact/unary-values/main.ssrg", source);
        let core = lower_typed_module(typed);

        assert!(matches!(
            &core.bindings[0].value,
            CoreExpr::Unary {
                operator,
                operand,
                type_ref: CoreType::Named {
                    name,
                    arguments,
                },
                ..
            } if operator == "-"
                && matches!(operand.as_ref(), CoreExpr::Integer { value, .. } if value == "2")
                && name == "Int"
                && arguments.is_empty()
        ));
        assert!(matches!(
            &core.bindings[1].value,
            CoreExpr::Unary {
                operator,
                operand,
                type_ref: CoreType::Named {
                    name,
                    arguments,
                },
                ..
            } if operator == "-"
                && matches!(operand.as_ref(), CoreExpr::Float64 { value, .. } if value == "0.0")
                && name == "Float"
                && arguments.is_empty()
        ));
        assert!(matches!(
            &core.bindings[2].value,
            CoreExpr::Unary {
                operator,
                operand,
                type_ref: CoreType::Named {
                    name,
                    arguments,
                },
                ..
            } if operator == "!"
                && matches!(operand.as_ref(), CoreExpr::Boolean { value: true, .. })
                && name == "Bool"
                && arguments.is_empty()
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(matches!(
            &typescript.bindings[0],
            TypeScriptBinding::Const {
                initializer: TypeScriptExpr::RuntimeCall { callee, arguments },
                ..
            }
                if callee == "_ssrg_int_subtract"
                    && matches!(
                        arguments.as_slice(),
                        [
                            TypeScriptExpr::Number { value: zero },
                            TypeScriptExpr::Number { value: two }
                        ] if zero == "0" && two == "2"
                    )
        ));
        assert!(matches!(
            &typescript.bindings[1],
            TypeScriptBinding::Const {
                initializer: TypeScriptExpr::Unary {
                    operator,
                    operand
                },
                ..
            } if operator == "-"
                && matches!(operand.as_ref(), TypeScriptExpr::Number { value } if value == "0.0")
        ));
        assert!(matches!(
            &typescript.bindings[2],
            TypeScriptBinding::Const {
                initializer: TypeScriptExpr::Unary {
                    operator,
                    operand
                },
                ..
            } if operator == "!"
                && matches!(operand.as_ref(), TypeScriptExpr::Boolean { value: true })
        ));

        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle
            .metadata
            .runtime
            .requirements
            .contains(&"core.int.subtract".to_owned()));
        assert!(bundle
            .typescript
            .contains("export const negative: number = _ssrg_int_subtract(0, 2);"));
        assert!(bundle
            .typescript
            .contains("export const negativeZero: number = -(0.0);"));
        assert!(bundle
            .typescript
            .contains("export const inverted: boolean = !(true);"));
        assert!(!bundle.typescript.contains(" = _;"));
    }

    #[test]
    fn emits_persistent_list_literals_through_the_runtime_abi() {
        let source = "pub fn values -> List<Int> = `[1, 2, 3]\n";
        let typed = type_module("artifact/list-literal/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "import { fromArray as _ssrg_list_from_array } from \"@seseragi/runtime/list\""
        ));
        assert!(bundle.typescript.contains(
            "export const values = (_unit: undefined) => _ssrg_list_from_array([1, 2, 3])"
        ));
        assert!(bundle
            .metadata
            .runtime
            .requirements
            .contains(&"core.list.from-array".to_owned()));
    }

    #[test]
    fn emits_project_selected_output_paths_into_metadata_and_source_map() {
        let source = "pub let answer: Int = 42\n";
        let typed = type_module("artifact/output-paths/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module_with_output_paths(
            typescript,
            source,
            GeneratedOutputPaths::new("dist/game/main.ts", "dist/game/main.ts.map"),
        );

        assert_eq!(bundle.metadata.outputs.typescript, "dist/game/main.ts");
        assert_eq!(bundle.metadata.outputs.source_map, "dist/game/main.ts.map");
        assert_eq!(bundle.source_map.file, "dist/game/main.ts");
        assert_eq!(
            bundle.source_map.sources,
            vec!["seseragi://artifact/output-paths"]
        );
    }

    #[test]
    fn maps_generated_declaration_to_its_original_source_line() {
        let source = "// generated code keeps this offset\n\npub let answer: Int = 42\n";
        let typed = type_module("artifact/source-map/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.source_map.mappings, "AAEAA");
    }

    #[test]
    fn emits_effect_typescript_module() {
        let source =
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n";
        let typed = type_module("artifact/effect-main/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .typescript
            .contains("import { println as _ssrg_console_println }"));
        assert_eq!(bundle.metadata.exports, vec!["main"]);
        assert_eq!(bundle.source_map.names, vec!["main", "println"]);
        assert_eq!(bundle.source_map.mappings, ";;AAAAA");
    }

    #[test]
    fn emits_unit_result_as_plain_undefined() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {}\n";
        let typed = type_module("artifact/effect-do/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.typescript,
            "export const main = (_unit: undefined) => undefined\n"
        );
    }
}
